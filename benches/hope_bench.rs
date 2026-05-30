use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hope::corn_kernel::{CornKernel, Z8Saturator};
use hope::crypto::HopeKeyPair;
use hope::eku::{Eku, EkuHeader, EkuType};

fn bench_corn_kernel(c: &mut Criterion) {
    let mut group = c.benchmark_group("CornKernel");

    group.bench_function("write_layer", |b| {
        let mut k = CornKernel::empty();
        b.iter(|| {
            k.write_layer(0, black_box(b"hello-hope-z8-data-benchmark-run!"));
        });
    });

    group.bench_function("read_layer", |b| {
        let mut k = CornKernel::empty();
        k.write_layer(0, b"hello-hope-z8-data-benchmark-run!");
        b.iter(|| {
            black_box(k.read_layer(0));
        });
    });

    group.bench_function("flatten_8_layers", |b| {
        let mut k = CornKernel::empty();
        for i in 0..8 {
            k.write_layer(i, &[i as u8; 32]);
        }
        b.iter(|| {
            black_box(k.flatten());
        });
    });

    group.finish();
}

fn bench_z8_saturator(c: &mut Criterion) {
    let mut group = c.benchmark_group("Z8Saturator");

    group.bench_function("saturate_1_layer", |b| {
        let mut sat = Z8Saturator::new(0xBEEF);
        b.iter(|| {
            sat.saturate(black_box(b"benchmark-payload-data-here!!!!!"));
        });
    });

    group.bench_function("saturate_full_8_layers", |b| {
        b.iter(|| {
            let mut sat = Z8Saturator::new(0xBEEF);
            for i in 0..8u8 {
                sat.saturate(black_box(&[i; 32]));
            }
            black_box(sat.is_full())
        });
    });

    for size in [8, 32, 128, 512] {
        group.bench_with_input(
            BenchmarkId::new("saturate_chunk", size),
            &size,
            |b, &sz| {
                let data = vec![0xABu8; sz];
                let mut sat = Z8Saturator::new(0x0001);
                b.iter(|| {
                    sat.saturate(black_box(&data[..sz.min(32)]));
                });
            },
        );
    }

    group.finish();
}

fn bench_crypto(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crypto");

    group.bench_function("keypair_generate", |b| {
        b.iter(|| black_box(HopeKeyPair::generate()));
    });

    group.bench_function("sign", |b| {
        let kp = HopeKeyPair::generate();
        let sender_id: [u8; 16] = *b"BENCH-SENDER-001";
        let chain_ref: [u8; 16] = *b"BENCH-CHAIN-0000";
        b.iter(|| {
            let header = EkuHeader::new(EkuType::Execute, 0x01, sender_id, 1, chain_ref, 32);
            let mut eku = Eku::new(header, vec![0xABu8; 32]);
            kp.sign(black_box(&mut eku));
        });
    });

    group.bench_function("verify", |b| {
        let kp = HopeKeyPair::generate();
        let sender_id: [u8; 16] = *b"BENCH-SENDER-001";
        let chain_ref: [u8; 16] = *b"BENCH-CHAIN-0000";
        let header = EkuHeader::new(EkuType::Execute, 0x01, sender_id, 1, chain_ref, 32);
        let mut eku = Eku::new(header, vec![0xABu8; 32]);
        kp.sign(&mut eku);
        b.iter(|| {
            black_box(kp.verify(&eku));
        });
    });

    group.bench_function("sign_and_verify_roundtrip", |b| {
        let kp = HopeKeyPair::generate();
        let sender_id: [u8; 16] = *b"BENCH-SENDER-001";
        let chain_ref: [u8; 16] = *b"BENCH-CHAIN-0000";
        b.iter(|| {
            let header = EkuHeader::new(EkuType::Execute, 0x01, sender_id, 1, chain_ref, 32);
            let mut eku = Eku::new(header, vec![0xABu8; 32]);
            kp.sign(&mut eku);
            black_box(kp.verify(&eku))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_corn_kernel, bench_z8_saturator, bench_crypto);
criterion_main!(benches);
