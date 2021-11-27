use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use gameboy::GameBoy;

fn op_r_imm() {
    let mut gb = GameBoy::builder()
        .rom_path("/mnt/i/Dev/gb-rs/04.gb")
        .build()
        .unwrap();

    for _ in 0..12301800 {
        gb.tick();
    }
}

fn bench_blargg(c: &mut Criterion) {
    let mut group = c.benchmark_group("blargg");
    group.measurement_time(Duration::from_millis(12100));

    group
        .measurement_time(Duration::from_millis(12100))
        .bench_function("cpu_instrs", |b| b.iter(op_r_imm));

    group.finish();
}

criterion_group!(benches, bench_blargg);
criterion_main!(benches);
