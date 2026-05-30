use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum EkuType {
    Query = 0x01,
    Execute = 0x02,
    Sync = 0x03,
    Birth = 0x04
}

/// EkuHeader — 96 byte, #[repr(C)] bináris layout
/// prev_hash: az előző slot Sha256 hash-e — replay attack ellen
/// sequence: szigorúan monoton növekvő — nonce szerepét tölti be
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EkuHeader {
    pub version: u8,
    pub eku_type: u8,
    pub flags: u8,
    pub _pad: u8,
    pub _align_pad: u32,
    pub timestamp_ns: u64,
    pub sender_id: [u8; 16],
    pub sequence: u64,
    pub mem_chain_ref: [u8; 16],
    pub payload_len: u32,
    pub _reserved: [u8; 4],
    pub prev_hash: [u8; 32], // SHA-256 hash az előző EKU-ról — láncolás
}

impl EkuHeader {
    pub fn new(
        eku_type: EkuType,
        flags: u8,
        sender_id: [u8; 16],
        sequence: u64,
        mem_chain_ref: [u8; 16],
        payload_len: u32,
    ) -> Self {
        Self::with_prev_hash(eku_type, flags, sender_id, sequence, mem_chain_ref, payload_len, [0u8; 32])
    }

    pub fn with_prev_hash(
        eku_type: EkuType,
        flags: u8,
        sender_id: [u8; 16],
        sequence: u64,
        mem_chain_ref: [u8; 16],
        payload_len: u32,
        prev_hash: [u8; 32],
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            version: 0x01,
            eku_type: eku_type as u8,
            flags,
            _pad: 0,
            _align_pad: 0,
            timestamp_ns: ts,
            sender_id,
            sequence,
            mem_chain_ref,
            payload_len,
            _reserved: [0u8; 4],
            prev_hash,
        }
    }
}

pub struct Eku {
    pub header: EkuHeader,
    pub payload: Vec<u8>,
    pub signature: [u8; 64],
}

impl Eku {
    pub fn new(header: EkuHeader, payload: Vec<u8>) -> Self {
        Self { signature: [0u8; 64], header, payload }
    }

    pub fn is_signed(&self) -> bool {
        self.signature.iter().any(|&b| b != 0)
    }

    /// SHA-256 hash az egész EKU-ról (header + payload + signature)
    /// A következő EKU prev_hash mezőjébe kerül
    pub fn chain_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        let header_bytes = self.header_bytes();
        hasher.update(&header_bytes);
        hasher.update(&self.payload);
        hasher.update(&self.signature);
        hasher.finalize().into()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = self.header_bytes().to_vec();
        buf.extend_from_slice(&self.payload);
        buf.extend_from_slice(&self.signature);
        buf
    }

    fn header_bytes(&self) -> [u8; std::mem::size_of::<EkuHeader>()] {
        let mut bytes = [0u8; std::mem::size_of::<EkuHeader>()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &self.header as *const EkuHeader as *const u8,
                bytes.as_mut_ptr(),
                bytes.len(),
            );
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_eku(seq: u64, prev: [u8; 32]) -> Eku {
        let sender: [u8; 16] = *b"TEST-SENDER-0001";
        let chain:  [u8; 16] = *b"TEST-CHAIN-00000";
        let h = EkuHeader::with_prev_hash(EkuType::Execute, 0x01, sender, seq, chain, 4, prev);
        Eku::new(h, b"test".to_vec())
    }

    #[test]
    fn chain_hash_changes_with_content() {
        let e1 = make_eku(1, [0u8; 32]);
        let e2 = make_eku(2, [0u8; 32]);
        assert_ne!(e1.chain_hash(), e2.chain_hash());
    }

    #[test]
    fn prev_hash_links_chain() {
        let e1 = make_eku(1, [0u8; 32]);
        let h1 = e1.chain_hash();
        let e2 = make_eku(2, h1);
        assert_eq!(e2.header.prev_hash, h1);
    }

    #[test]
    fn sequence_monotonic() {
        let e1 = make_eku(1, [0u8; 32]);
        let e2 = make_eku(2, e1.chain_hash());
        assert!(e2.header.sequence > e1.header.sequence);
    }
}
