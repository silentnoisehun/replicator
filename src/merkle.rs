use sha2::{Sha256, Digest};
use ed25519_dalek::Signature;
use crate::eku::Eku;
use crate::crypto::HopeKeyPair;

pub const BATCH_SIZE: usize = 32;

/// SHA-256(left || right) — belső csomópontok
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(left);
    h.update(right);
    h.finalize().into()
}

/// Merkle-fa — levelek az EKU-k chain_hash()-ei
/// Páratlan levélszám esetén az utolsó leaf duplikálódik (standard gyakorlat)
pub struct MerkleTree {
    pub leaves: Vec<[u8; 32]>,
    levels: Vec<Vec<[u8; 32]>>,
    pub root: [u8; 32],
}

impl MerkleTree {
    pub fn build(leaves: Vec<[u8; 32]>) -> Option<Self> {
        if leaves.is_empty() { return None; }

        let mut levels: Vec<Vec<[u8; 32]>> = vec![leaves.clone()];

        loop {
            let current = levels.last().unwrap();
            if current.len() == 1 { break; }

            let mut next = Vec::new();
            let mut i = 0;
            while i < current.len() {
                let left  = current[i];
                let right = if i + 1 < current.len() { current[i + 1] } else { current[i] };
                next.push(hash_pair(&left, &right));
                i += 2;
            }
            levels.push(next);
        }

        let root = levels.last().unwrap()[0];
        Some(Self { leaves, levels, root })
    }

    /// Merkle Proof az `index`-edik levélhez — HopeVM plugin ellenőrzéséhez
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() { return None; }

        let mut siblings = Vec::new();
        let mut is_right  = Vec::new();
        let mut idx = index;

        for level in &self.levels[..self.levels.len() - 1] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let sibling = if sibling_idx < level.len() {
                level[sibling_idx]
            } else {
                level[idx] // duplikált utolsó levél
            };
            siblings.push(sibling);
            is_right.push(idx % 2 != 0);
            idx /= 2;
        }

        Some(MerkleProof { leaf: self.leaves[index], siblings, is_right, root: self.root })
    }
}

/// Merkle Proof — HopeVM plugin kapja meg, self-contained ellenőrzés
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf: [u8; 32],
    pub siblings: Vec<[u8; 32]>,
    pub is_right: Vec<bool>,
    pub root: [u8; 32],
}

impl MerkleProof {
    /// Ellenőrzi hogy a leaf valóban része a root-nak — O(log N)
    pub fn verify(&self) -> bool {
        let mut current = self.leaf;
        for (sibling, &right) in self.siblings.iter().zip(self.is_right.iter()) {
            current = if right {
                hash_pair(sibling, &current)
            } else {
                hash_pair(&current, sibling)
            };
        }
        current == self.root
    }
}

/// Aláírt batch — Root-to-Root hash láncolás, replay attack batch szinten
pub struct SignedBatch {
    pub batch_seq: u64,
    pub root: [u8; 32],
    pub prev_root_hash: [u8; 32], // az előző SignedBatch root-jának SHA-256-a
    pub signature: [u8; 64],
    pub tree: MerkleTree,
    pub eku_count: usize,
}

impl SignedBatch {
    /// `prev_root_hash` biztosítja a batch-szintű láncfolytonosságot
    pub fn verify_chain(&self, expected_prev: &[u8; 32]) -> bool {
        self.prev_root_hash == *expected_prev
    }
}

/// Aszinkron Merkle Batch Signing pipeline
/// - BATCH_SIZE vagy flush() esetén lezárja a köteget és aláírja
/// - Kritikus parancshoz: flush() azonnal zárja a fát, nem vár
pub struct BatchSigner {
    keypair: HopeKeyPair,
    pending: Vec<Eku>,
    batch_seq: u64,
    last_root_hash: [u8; 32], // Root-to-Root lánc előző tagja
}

impl BatchSigner {
    pub fn new(keypair: HopeKeyPair) -> Self {
        Self {
            keypair,
            pending: Vec::with_capacity(BATCH_SIZE),
            batch_seq: 0,
            last_root_hash: [0u8; 32],
        }
    }

    /// EKU event bekerül a pufferbe. Ha elérte a BATCH_SIZE-t, automatikusan flushole.
    pub fn push(&mut self, eku: Eku) -> Option<SignedBatch> {
        self.pending.push(eku);
        if self.pending.len() >= BATCH_SIZE {
            self.flush()
        } else {
            None
        }
    }

    /// Kritikus / szinkron parancs esetén azonnal lezárja a jelenlegi köteget.
    /// Üres pufferrel is biztonságos — None-t ad vissza.
    pub fn flush(&mut self) -> Option<SignedBatch> {
        if self.pending.is_empty() { return None; }

        let events = std::mem::take(&mut self.pending);
        let leaves: Vec<[u8; 32]> = events.iter().map(|e| e.chain_hash()).collect();

        let tree = MerkleTree::build(leaves)?;

        // Root-to-Root lánc: az előző batch root-jának SHA-256-a
        let prev_root_hash = self.last_root_hash;
        let mut msg = tree.root.to_vec();
        msg.extend_from_slice(&prev_root_hash);
        msg.extend_from_slice(&self.batch_seq.to_le_bytes());

        use ed25519_dalek::Signer;
        let sig = self.keypair.signing.sign(&msg);
        let mut signature = [0u8; 64];
        signature.copy_from_slice(sig.to_bytes().as_ref());

        // Frissítjük a lánc referenciát
        let mut h = Sha256::new();
        h.update(&tree.root);
        self.last_root_hash = h.finalize().into();

        let batch = SignedBatch {
            batch_seq: self.batch_seq,
            root: tree.root,
            prev_root_hash,
            signature,
            tree,
            eku_count: events.len(),
        };

        self.batch_seq += 1;
        Some(batch)
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Ellenőrzi a batch root aláírását
    pub fn verify_batch(&self, batch: &SignedBatch, prev_root_hash: &[u8; 32]) -> bool {
        use ed25519_dalek::Verifier;
        if !batch.verify_chain(prev_root_hash) { return false; }

        let mut msg = batch.root.to_vec();
        msg.extend_from_slice(prev_root_hash);
        msg.extend_from_slice(&batch.batch_seq.to_le_bytes());

        if let Ok(sig) = Signature::from_slice(&batch.signature) {
            self.keypair.verifying.verify(&msg, &sig).is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eku::{Eku, EkuHeader, EkuType};
    use crate::crypto::HopeKeyPair;

    fn make_eku(seq: u64) -> Eku {
        let sender: [u8; 16] = *b"MERKLE-TEST-0001";
        let chain:  [u8; 16] = *b"MERKLE-CHAIN-000";
        let h = EkuHeader::new(EkuType::Execute, 0x01, sender, seq, chain, 4);
        Eku::new(h, format!("payload-{}", seq).into_bytes())
    }

    #[test]
    fn merkle_tree_builds() {
        let leaves: Vec<[u8; 32]> = (0..4).map(|i| {
            let mut h = Sha256::new();
            h.update(&[i as u8; 32]);
            h.finalize().into()
        }).collect();
        let tree = MerkleTree::build(leaves).unwrap();
        assert_ne!(tree.root, [0u8; 32]);
    }

    #[test]
    fn merkle_proof_verifies() {
        let leaves: Vec<[u8; 32]> = (0..8).map(|i| {
            let mut h = Sha256::new();
            h.update(&[i as u8]);
            h.finalize().into()
        }).collect();
        let tree = MerkleTree::build(leaves).unwrap();

        for i in 0..8 {
            let proof = tree.proof(i).unwrap();
            assert!(proof.verify(), "proof {} nem valid", i);
        }
    }

    #[test]
    fn proof_fails_on_tampered_leaf() {
        let leaves: Vec<[u8; 32]> = (0..4).map(|i| {
            let mut h = Sha256::new();
            h.update(&[i as u8]);
            h.finalize().into()
        }).collect();
        let tree = MerkleTree::build(leaves).unwrap();
        let mut proof = tree.proof(0).unwrap();
        proof.leaf[0] ^= 0xFF;
        assert!(!proof.verify());
    }

    #[test]
    fn odd_leaf_count_works() {
        let leaves: Vec<[u8; 32]> = (0..5).map(|i| {
            let mut h = Sha256::new();
            h.update(&[i as u8]);
            h.finalize().into()
        }).collect();
        let tree = MerkleTree::build(leaves).unwrap();
        for i in 0..5 {
            assert!(tree.proof(i).unwrap().verify());
        }
    }

    #[test]
    fn batch_signer_auto_flush_at_limit() {
        let kp = HopeKeyPair::generate();
        let mut signer = BatchSigner::new(kp);
        let mut result = None;
        for i in 0..BATCH_SIZE {
            result = signer.push(make_eku(i as u64));
        }
        let batch = result.expect("BATCH_SIZE elérésénél automatikus flush");
        assert_eq!(batch.eku_count, BATCH_SIZE);
    }

    #[test]
    fn flush_critical_partial_batch() {
        let kp = HopeKeyPair::generate();
        let mut signer = BatchSigner::new(kp);
        signer.push(make_eku(1));
        signer.push(make_eku(2));
        let batch = signer.flush().expect("partial flush működik");
        assert_eq!(batch.eku_count, 2);
        assert_eq!(signer.pending_count(), 0);
    }

    #[test]
    fn root_to_root_chain_links() {
        let kp = HopeKeyPair::generate();
        let mut signer = BatchSigner::new(kp);

        // BATCH_SIZE-1 event + manuális flush = b1 (genesis, prev=[0;32])
        for i in 0..(BATCH_SIZE - 1) { signer.push(make_eku(i as u64)); }
        let b1 = signer.flush().unwrap();
        assert_eq!(b1.prev_root_hash, [0u8; 32]);

        // Második batch — prev_root_hash = SHA-256(b1.root)
        for i in 0..(BATCH_SIZE - 1) { signer.push(make_eku((BATCH_SIZE + i) as u64)); }
        let b2 = signer.flush().unwrap();
        let expected: [u8; 32] = Sha256::digest(&b1.root).into();
        assert_eq!(b2.prev_root_hash, expected);
    }

    #[test]
    fn chain_tamper_detected() {
        let kp = HopeKeyPair::generate();
        let mut signer = BatchSigner::new(kp);
        for i in 0..(BATCH_SIZE - 1) { signer.push(make_eku(i as u64)); }
        let b1 = signer.flush().unwrap();

        let wrong_prev = [0xFFu8; 32];
        assert!(!b1.verify_chain(&wrong_prev));
        assert!(b1.verify_chain(&[0u8; 32]));
    }

    #[test]
    fn batch_signature_verifies() {
        let kp = HopeKeyPair::generate();
        let mut signer = BatchSigner::new(kp);
        for i in 0..4 { signer.push(make_eku(i as u64)); }
        let batch = signer.flush().unwrap();
        assert!(signer.verify_batch(&batch, &[0u8; 32]));
    }
}
