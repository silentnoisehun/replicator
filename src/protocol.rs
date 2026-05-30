pub const MSG_REGISTER: u32 = 0x01;
pub const MSG_GET_COLLECTIVE: u32 = 0x02;
pub const MSG_PULSE: u32 = 0x03;
pub const RESP_OK: u8 = 0x00;
pub const RECORD_SIZE: usize = 128;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CloneRecord {
    pub id: [u8; 32],
    pub genome: [u8; 32],
    pub chain: [u8; 12],
    pub parent_chain: [u8; 12],
    pub traits: u64,
    pub generation: u32,
    pub born_ts: u32,
    pub _pad: [u8; 24],
}
