/// Spine IPC integrációs tesztek
/// Valós mmap fájl — írás → olvasás → bytemuck validáció teljes ciklus
use hope::spine::Spine;
use hope::corn_kernel::{CornKernel, Z8Saturator};
use std::fs;

fn tmp_spine_path(name: &str) -> String {
    format!("D:/replicator-build/test-spine-{}.bin", name)
}

fn open_spine(name: &str) -> Spine {
    let path = tmp_spine_path(name);
    let id: [u8; 16] = *b"TEST-SPINE-INTEG";
    Spine::open(&path, id).expect("Spine::open failed")
}

#[test]
fn spine_write_read_roundtrip() {
    let mut spine = open_spine("roundtrip");
    let mut k = CornKernel::empty();
    k.write_layer(0, b"integration test payload!");
    k.genome_tag = 0xCAFE;

    let seq = spine.write(&k);
    let recovered = spine.read(seq).expect("read visszaad kernelt");

    assert_eq!(recovered.genome_tag, 0xCAFE);
    assert_eq!(recovered.active_mask, k.active_mask);
    assert_eq!(&recovered.layers[0][..25], b"integration test payload!");
}

#[test]
fn spine_read_validates_bytes() {
    let mut spine = open_spine("validation");
    let k = CornKernel::empty();
    let seq = spine.write(&k);

    // from_bytes_validated-on megy keresztül — nem crashel érvényes adaton
    let result = spine.read(seq);
    assert!(result.is_some());
}

#[test]
fn spine_ring_wraps_correctly() {
    let mut spine = open_spine("ring");
    let start = spine.writer_seq();

    // 70 írás — túllépi a 64-es ring kapacitást
    for i in 0..70u8 {
        let mut k = CornKernel::empty();
        k.genome_tag = i as u16;
        spine.write(&k);
    }

    assert_eq!(spine.writer_seq(), start + 70);

    // A ring-en belüli friss seq-ek olvashatók
    let last_seq = start + 69;
    assert!(spine.read(last_seq).is_some());

    // A legújabb slot valóban a 69-es genome_tag-et tartalmazza
    let last = spine.read(last_seq).unwrap();
    assert_eq!(last.genome_tag, 69u16);
}

#[test]
fn spine_write_increments_seq() {
    let mut spine = open_spine("seq");
    let before = spine.writer_seq();
    let k = CornKernel::empty();
    let s1 = spine.write(&k);
    let s2 = spine.write(&k);
    assert_eq!(s1, before);
    assert_eq!(s2, before + 1);
    assert_eq!(spine.writer_seq(), before + 2);
}

#[test]
fn spine_z8_saturator_roundtrip() {
    let mut spine = open_spine("z8");
    let mut sat = Z8Saturator::new(0xBEEF);
    for chunk in b"teljes z8 saturator pipeline teszt!".chunks(32) {
        sat.saturate(chunk);
    }
    let seq = spine.write(&sat.kernel);
    let r = spine.read(seq).expect("z8 kernel visszaolvasható");
    assert_eq!(r.genome_tag, 0xBEEF);
    assert!(r.active_mask != 0);
}
