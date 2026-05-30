/// Kriptográfiai pipeline integrációs tesztek
/// EKU létrehozás → aláírás → Merkle batch → proof verify — teljes lánc
use hope::crypto::HopeKeyPair;
use hope::eku::{Eku, EkuHeader, EkuType};
use hope::merkle::{BatchSigner, MerkleTree, BATCH_SIZE};
use sha2::{Sha256, Digest};

fn make_signed_eku(kp: &HopeKeyPair, seq: u64, prev_hash: [u8; 32]) -> Eku {
    let sender: [u8; 16] = *b"INTEG-SENDER-001";
    let chain:  [u8; 16] = *b"INTEG-CHAIN-0001";
    let header = EkuHeader::with_prev_hash(
        EkuType::Execute, 0x01, sender, seq, chain, 8, prev_hash
    );
    let mut eku = Eku::new(header, format!("payload-{}", seq).into_bytes());
    kp.sign(&mut eku);
    eku
}

#[test]
fn eku_sign_verify_with_chain() {
    let kp = HopeKeyPair::generate();
    let e1 = make_signed_eku(&kp, 1, [0u8; 32]);
    assert!(e1.is_signed());
    assert!(kp.verify(&e1));

    // e2 láncolva e1-hez
    let h1 = e1.chain_hash();
    let e2 = make_signed_eku(&kp, 2, h1);
    assert_eq!(e2.header.prev_hash, h1);
    assert!(kp.verify(&e2));
    assert!(e2.header.sequence > e1.header.sequence);
}

#[test]
fn tampered_eku_fails_verify() {
    let kp = HopeKeyPair::generate();
    let mut eku = make_signed_eku(&kp, 1, [0u8; 32]);
    assert!(kp.verify(&eku));

    eku.payload.push(0xFF); // megronjtuk
    assert!(!kp.verify(&eku));
}

#[test]
fn merkle_batch_full_pipeline() {
    let kp = HopeKeyPair::generate();
    let mut signer = BatchSigner::new(kp);

    // Batch feltöltés — auto flush BATCH_SIZE-nál
    let mut batch = None;
    let mut prev_hash = [0u8; 32];
    for i in 0..BATCH_SIZE {
        let eku = {
            let sender: [u8; 16] = *b"BATCH-SENDER-001";
            let chain:  [u8; 16] = *b"BATCH-CHAIN-0001";
            let h = EkuHeader::with_prev_hash(EkuType::Execute, 0x01, sender, i as u64, chain, 4, prev_hash);
            let mut e = Eku::new(h, vec![i as u8; 4]);
            prev_hash = e.chain_hash();
            e
        };
        batch = signer.push(eku);
    }

    let batch = batch.expect("BATCH_SIZE-nál auto flush kell");
    assert_eq!(batch.eku_count, BATCH_SIZE);
    assert_ne!(batch.root, [0u8; 32]);

    // Merkle proof minden levélre valid
    for i in 0..BATCH_SIZE {
        let proof = batch.tree.proof(i).expect("proof generálható");
        assert!(proof.verify(), "proof {} invalid", i);
    }
}

#[test]
fn merkle_proof_detects_tampering() {
    let kp = HopeKeyPair::generate();
    let mut signer = BatchSigner::new(kp);

    for i in 0..(BATCH_SIZE - 1) {
        let sender: [u8; 16] = *b"TAMPER-TEST-0001";
        let chain:  [u8; 16] = *b"TAMPER-CHAIN-000";
        let h = EkuHeader::new(EkuType::Execute, 0x01, sender, i as u64, chain, 4);
        signer.push(Eku::new(h, vec![i as u8; 4]));
    }
    let batch = signer.flush().expect("flush kell");
    let mut proof = batch.tree.proof(0).expect("proof");

    // Megrontjuk a leaf-et
    proof.leaf[0] ^= 0xFF;
    assert!(!proof.verify(), "megrontott proof nem lehet valid");
}

#[test]
fn batch_root_chain_two_batches() {
    let kp = HopeKeyPair::generate();
    let mut signer = BatchSigner::new(kp);

    for i in 0..(BATCH_SIZE - 1) {
        let sender: [u8; 16] = *b"CHAIN-SENDER-001";
        let chain:  [u8; 16] = *b"CHAIN-CHAIN-0001";
        let h = EkuHeader::new(EkuType::Execute, 0x01, sender, i as u64, chain, 4);
        signer.push(Eku::new(h, vec![i as u8; 4]));
    }
    let b1 = signer.flush().unwrap();
    assert_eq!(b1.prev_root_hash, [0u8; 32]); // genesis

    for i in 0..(BATCH_SIZE - 1) {
        let sender: [u8; 16] = *b"CHAIN-SENDER-002";
        let chain:  [u8; 16] = *b"CHAIN-CHAIN-0002";
        let h = EkuHeader::new(EkuType::Execute, 0x01, sender, (BATCH_SIZE + i) as u64, chain, 4);
        signer.push(Eku::new(h, vec![i as u8; 4]));
    }
    let b2 = signer.flush().unwrap();

    // b2.prev_root_hash == SHA256(b1.root)
    let expected: [u8; 32] = Sha256::digest(&b1.root).into();
    assert_eq!(b2.prev_root_hash, expected, "Root-to-Root láncolat megszakadt");
}
