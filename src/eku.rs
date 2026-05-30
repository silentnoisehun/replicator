use std::time::{SystemTime, UNIX_EPOCH};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum EkuType { 
    Query = 0x01, 
    Execute = 0x02, 
    Sync = 0x03, 
    Birth = 0x04 
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EkuHeader {
    pub version: u8, 
    pub eku_type: u8, 
    pub flags: u8, 
    pub _pad: u8,
    pub _align_pad: u32,          // explicit padding timestamp_ns 8-byte alignhoz
    pub timestamp_ns: u64,
    pub sender_id: [u8; 16],
    pub sequence: u64,
    pub mem_chain_ref: [u8; 16],
    pub payload_len: u32,
    pub _reserved: [u8; 8],
    pub _tail_pad: u32,           // méret 64 byte-ra kerekítve
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
            _reserved: [0u8; 8],
            _tail_pad: 0,
        }
    }
}

pub struct Eku { 
    pub header: EkuHeader, 
    pub payload: Vec<u8>, 
    pub signature: [u8; 64] 
}

impl Eku {
    pub fn new(header: EkuHeader, payload: Vec<u8>) -> Self {
        Self { signature: [0u8; 64], header, payload }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut header_bytes = [0u8; std::mem::size_of::<EkuHeader>()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &self.header as *const EkuHeader as *const u8,
                header_bytes.as_mut_ptr(),
                std::mem::size_of::<EkuHeader>(),
            );
        }
        buf.extend_from_slice(&header_bytes);
        buf.extend_from_slice(&self.payload);
        buf.extend_from_slice(&self.signature);
        buf
    }
}
