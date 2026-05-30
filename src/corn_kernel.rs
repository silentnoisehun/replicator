use bytemuck::{Pod, Zeroable, bytes_of, try_from_bytes};

pub const LAYER_SIZE: usize = 32;
pub const Z8_LAYERS:  usize = 8;

pub fn kernel_size() -> usize {
    std::mem::size_of::<CornKernel>()
}

/// #[repr(C, align(8))] + Pod/Zeroable — bytemuck validált, UB-mentes deszarizáció
/// Layout: 256 (layers) + 1 + 1 + 2 + 4 = 264 byte, 8-byte aligned
#[repr(C, align(8))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CornKernel {
    pub layers:      [[u8; LAYER_SIZE]; Z8_LAYERS],
    pub active_mask: u8,
    pub zoom_depth:  u8,
    pub genome_tag:  u16,
    pub seq:         u32,
}

impl CornKernel {
    pub fn empty() -> Self {
        Zeroable::zeroed()
    }

    /// Biztonságos deszarizáció nyers bájtokból — bytemuck ellenőrzi a méretet és az alignmentet
    pub fn from_bytes_validated(data: &[u8]) -> Option<Self> {
        let size = kernel_size();
        if data.len() < size {
            return None;
        }
        try_from_bytes::<CornKernel>(&data[..size]).ok().copied()
    }

    pub fn write_layer(&mut self, depth: usize, data: &[u8]) {
        assert!(depth < Z8_LAYERS, "Z8: max réteg = 8");
        let len = data.len().min(LAYER_SIZE);
        self.layers[depth][..len].copy_from_slice(&data[..len]);
        self.active_mask |= 1 << depth;
        if depth + 1 > self.zoom_depth as usize {
            self.zoom_depth = (depth + 1) as u8;
        }
    }

    pub fn read_layer(&self, depth: usize) -> Option<&[u8; LAYER_SIZE]> {
        if depth < Z8_LAYERS && (self.active_mask & (1 << depth)) != 0 {
            Some(&self.layers[depth])
        } else {
            None
        }
    }

    /// Biztonságos — a layers mező pontosan 256 byte, bytemuck garantálja
    pub fn deep_read(&self) -> &[u8] {
        bytes_of(&self.layers)
    }

    pub fn flatten(&self) -> Vec<u8> {
        (0..Z8_LAYERS)
            .filter(|&d| (self.active_mask & (1 << d)) != 0)
            .flat_map(|d| self.layers[d].iter().copied())
            .collect()
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytes_of(self)
    }
}

pub struct Z8Saturator {
    pub kernel:    CornKernel,
    current_layer: usize,
    pub seq:       u32,
}

impl Z8Saturator {
    pub fn new(genome_tag: u16) -> Self {
        let mut k = CornKernel::empty();
        k.genome_tag = genome_tag;
        Self { kernel: k, current_layer: 0, seq: 0 }
    }

    pub fn saturate(&mut self, data: &[u8]) {
        self.kernel.write_layer(self.current_layer, data);
        self.seq += 1;
        self.kernel.seq = self.seq;
        self.current_layer = (self.current_layer + 1) % Z8_LAYERS;
    }

    pub fn is_full(&self) -> bool {
        self.kernel.active_mask == 0xFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_write_read() {
        let mut k = CornKernel::empty();
        k.write_layer(0, b"hello");
        let layer = k.read_layer(0).expect("layer 0 legyen aktiv");
        assert_eq!(&layer[..5], b"hello");
    }

    #[test]
    fn active_mask_tracking() {
        let mut k = CornKernel::empty();
        assert_eq!(k.active_mask, 0);
        k.write_layer(2, b"test");
        assert_eq!(k.active_mask, 0b00000100);
        k.write_layer(5, b"data");
        assert_eq!(k.active_mask, 0b00100100);
    }

    #[test]
    fn flatten_only_active() {
        let mut k = CornKernel::empty();
        k.write_layer(0, b"AAAA");
        k.write_layer(1, b"BBBB");
        let flat = k.flatten();
        assert_eq!(flat.len(), 64);
    }

    #[test]
    fn z8_saturator_fullness() {
        let mut sat = Z8Saturator::new(0xBEEF);
        assert!(!sat.is_full());
        for i in 0..8 {
            sat.saturate(&[i as u8; 32]);
        }
        assert!(sat.is_full());
    }

    #[test]
    fn saturator_seq_increments() {
        let mut sat = Z8Saturator::new(0x0001);
        sat.saturate(b"first");
        sat.saturate(b"second");
        assert_eq!(sat.seq, 2);
        assert_eq!(sat.kernel.seq, 2);
    }

    #[test]
    fn validated_roundtrip() {
        let mut k = CornKernel::empty();
        k.write_layer(0, b"validate me");
        k.genome_tag = 0xDEAD;
        let bytes = k.as_bytes().to_vec();
        let recovered = CornKernel::from_bytes_validated(&bytes).expect("valid bytes");
        assert_eq!(recovered.genome_tag, 0xDEAD);
        assert_eq!(recovered.active_mask, k.active_mask);
    }

    #[test]
    fn validated_rejects_short_input() {
        let short = vec![0u8; 10];
        assert!(CornKernel::from_bytes_validated(&short).is_none());
    }

    #[test]
    fn pod_size_correct() {
        assert_eq!(kernel_size(), 264);
    }
}
