#![expect(missing_docs, reason = "benchmarks")]

use aes::cipher::{Array, BlockModeEncrypt, KeyIvInit};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::io::Cursor;
use steel_protocol::utils::Aes128Cfb8Enc;
use steel_utils::{
    FrontVec,
    codec::VarInt,
    serial::{ReadFrom, WriteTo},
};

fn bench_varint_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("varint_encode");
    let test_values = [
        ("1_byte_0", 0),
        ("1_byte_127", 127),
        ("2_byte_255", 255),
        ("2_byte_16383", 16_383),
        ("3_byte_65535", 65_535),
        ("3_byte_2097151", 2_097_151),
        ("4_byte_268435455", 268_435_455),
        ("5_byte_max", i32::MAX),
    ];

    for (name, val) in test_values {
        group.bench_with_input(BenchmarkId::new("write", name), &val, |b, &val| {
            let varint = VarInt(val);
            let mut buf = [0u8; 5];
            b.iter(|| {
                let mut cursor = Cursor::new(&mut buf[..]);
                let _ = varint.write(&mut cursor);
                black_box(cursor.position())
            });
        });

        group.bench_with_input(BenchmarkId::new("written_size", name), &val, |b, &val| {
            b.iter(|| black_box(VarInt::written_size(black_box(val))));
        });
    }
    group.finish();
}

fn bench_varint_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("varint_decode");
    let test_values = [
        ("1_byte", 127),
        ("2_byte", 16_383),
        ("3_byte", 2_097_151),
        ("4_byte", 268_435_455),
        ("5_byte", i32::MAX),
    ];

    for (name, val) in test_values {
        let varint = VarInt(val);
        let mut encoded = Vec::new();
        let _ = varint.write(&mut encoded);

        group.bench_with_input(BenchmarkId::new("read", name), &encoded, |b, encoded| {
            b.iter(|| {
                let mut cursor = Cursor::new(black_box(encoded.as_slice()));
                black_box(VarInt::read(&mut cursor).ok())
            });
        });
    }
    group.finish();
}

fn bench_varint_set_in_front(c: &mut Criterion) {
    let mut group = c.benchmark_group("varint_set_in_front");
    let test_payload_len = 1024;

    group.bench_function("set_in_front_1024b", |b| {
        b.iter(|| {
            let mut front_vec = FrontVec::capacity(10, test_payload_len);
            front_vec.extend_from_slice(&[0xAA; 1024]);
            let size = VarInt::written_size(1024);
            VarInt(1024).set_in_front(&mut front_vec, size);
            black_box(front_vec.as_slice().len())
        });
    });
    group.finish();
}

fn bench_aes128_cfb8_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("aes128_cfb8_encryption");
    let key = [0x42u8; 16];
    let iv = [0x24u8; 16];

    for size in [64, 512, 4096, 65536] {
        let mut data = vec![0xABu8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("encrypt", size), &size, |b, _| {
            b.iter(|| {
                if let Ok(mut encryptor) = Aes128Cfb8Enc::new_from_slices(&key, &iv) {
                    for chunk in data.as_chunks_mut::<1>().0 {
                        let mut out = [0u8];
                        let in_block: &Array<u8, _> = (&*chunk).into();
                        let out_block: &mut Array<u8, _> = (&mut out).into();
                        encryptor.encrypt_block_b2b(in_block, out_block);
                        chunk[0] = out[0];
                    }
                }
                black_box(data.len())
            });
        });
    }
    group.finish();
}

fn pack_bits_sim(indices: &[u32], bits: usize) -> Vec<u64> {
    let values_per_long = 64 / bits;
    let len = indices.len().div_ceil(values_per_long);
    let mut data = vec![0u64; len];

    for (i, &index) in indices.iter().enumerate() {
        let array_index = i / values_per_long;
        let offset = (i % values_per_long) * bits;
        data[array_index] |= u64::from(index) << offset;
    }

    data
}

fn bench_paletted_bit_packing(c: &mut Criterion) {
    let mut group = c.benchmark_group("paletted_bit_packing");
    let count = 4096;

    for bits in [4, 5, 6, 7, 8, 15] {
        let max_val = (1u32 << bits) - 1;
        let indices: Vec<u32> = (0..count).map(|i| (i as u32) % (max_val + 1)).collect();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("pack_section_indices", format!("{bits}_bits")),
            &bits,
            |b, &bits| {
                b.iter(|| black_box(pack_bits_sim(black_box(&indices), bits)));
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_varint_encode,
    bench_varint_decode,
    bench_varint_set_in_front,
    bench_aes128_cfb8_encryption,
    bench_paletted_bit_packing
);
criterion_main!(benches);
