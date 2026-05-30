pub const LAYER_SIZE: usize = 32;
pub const Z8_LAYERS:  usize = 8;

pub fn kernel_size() -> usize {
    std::mem::size_of::<CornKernel>()
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct CornKernel {
    pub layers:      [[u8; LAYER_SIZE]; Z8_LAYERS],
    pub active_mask: u8,
    pub zoom_depth:  u8,
    pub genome_tag:  u16,
    pub seq:         u32,
}

impl CornKernel {
    pub fn empty() -> Self {
        unsafe { std::mem::zeroed() }
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

    pub fn deep_read(&self) -> &[u8; 256] {
        unsafe { std::mem::transmute(&self.layers) }
    }

    pub fn flatten(&self) -> Vec<u8> {
        (0..Z8_LAYERS)
            .filter(|&d| (self.active_mask & (1 << d)) != 0)
            .flat_map(|d| self.layers[d].iter().copied())
            .collect()
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                kernel_size(),
            )
        }
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
