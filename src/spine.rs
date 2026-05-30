use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::path::Path;
use memmap2::{MmapMut, MmapOptions};

use crate::corn_kernel::CornKernel;

pub const SPINE_SIZE: usize = 2048 * 16;
pub const HEADER_SIZE: usize = 64;
pub const RING_CAPACITY: usize = 64;
pub const SLOT_SIZE: usize = 320;

// Env-ből olvassa, fallback a default path-ra — ne legyen hardcoded user home
pub fn spine_path() -> String {
    std::env::var("HOPE_SPINE_PATH")
        .unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            format!("{}\\.hope\\spine.bin", home)
        })
}

pub const SPINE_PATH: &str = ""; // deprecated — használd spine_path()

#[repr(C)]
pub struct SpineHeader {
    pub writer_seq: AtomicU64,
    pub reader_seq: AtomicU64,
    pub tick:       AtomicU64,
    pub spine_id:   [u8; 16],
}

pub struct Spine {
    mmap: MmapMut,
    pub path: String,
    // Intra-process write lock — torn write védelem ha több szál ír
    // Multi-process esetén OS-szintű file lock szükséges (jövőbeli fejlesztés)
    write_lock: Mutex<()>,
}

impl Spine {
    pub fn open<P: AsRef<Path>>(path: P, id: [u8; 16]) -> std::io::Result<Self> {
        let path_ref = path.as_ref();

        if let Some(parent) = path_ref.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path_ref)?;

        if file.metadata()?.len() < SPINE_SIZE as u64 {
            file.set_len(SPINE_SIZE as u64)?;
        }

        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        let mut spine = Self {
            mmap,
            path: path_ref.to_string_lossy().to_string(),
            write_lock: Mutex::new(()),
        };

        let header_ptr = spine.mmap.as_mut_ptr() as *mut SpineHeader;
        unsafe {
            (*header_ptr).spine_id = id;
        }

        Ok(spine)
    }

    pub fn open_default(id: [u8; 16]) -> std::io::Result<Self> {
        Self::open(spine_path(), id)
    }

    pub fn header(&self) -> &SpineHeader {
        unsafe { &*(self.mmap.as_ptr() as *const SpineHeader) }
    }

    pub fn writer_seq(&self) -> u64 {
        self.header().writer_seq.load(Ordering::SeqCst)
    }

    pub fn reader_seq(&self) -> u64 {
        self.header().reader_seq.load(Ordering::SeqCst)
    }

    pub fn write(&mut self, kernel: &CornKernel) -> u64 {
        let _guard = self.write_lock.lock().expect("spine write lock poisoned");
        let seq = self.header().writer_seq.fetch_add(1, Ordering::SeqCst);
        let slot_idx = seq as usize % RING_CAPACITY;
        let offset = HEADER_SIZE + slot_idx * SLOT_SIZE;

        let data = kernel.as_bytes();
        let len = data.len().min(SLOT_SIZE);

        unsafe {
            let dest_ptr = self.mmap.as_mut_ptr().add(offset);
            std::ptr::copy_nonoverlapping(data.as_ptr(), dest_ptr, len);
        }

        seq
    }

    pub fn read(&self, seq: u64) -> Option<CornKernel> {
        let w = self.writer_seq();
        if seq >= w { return None; }

        let slot_idx = seq as usize % RING_CAPACITY;
        let offset = HEADER_SIZE + slot_idx * SLOT_SIZE;

        unsafe {
            let ptr = self.mmap.as_ptr().add(offset) as *const CornKernel;
            Some(std::ptr::read_unaligned(ptr))
        }
    }
}
