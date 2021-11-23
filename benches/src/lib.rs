use criterion::{criterion_group, criterion_main, Criterion};
use gameboy::GameBoy;

fn op_r_imm() {
    let mut gb = GameBoy::new("/mnt/i/Dev/gb-rs/04.gb");
    for _ in 0..12301800 {
        gb.tick();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cpu_instrs", |b| b.iter(|| op_r_imm()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
