use criterion::{black_box, criterion_group, criterion_main, Criterion};
use partyboy::gameboy::cpu::register::Register;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn reg_into() -> u16 {
    let reg = Register::new(0, 0);
    let val: u16 = reg.into();
    val
}

fn reg_inline() -> u16 {
    let reg = Register::new(0, 0);
    let val = ((reg.hi as u16) << 8) | (reg.lo as u16);
    val
}

fn reg_add_assign(reg: &mut Register, rhs: u16) {
    let val = (((reg.hi as u16) << 8) | (reg.lo as u16)).wrapping_add(rhs);
    reg.hi = ((val & 0xFF00) >> 8) as u8;
    reg.lo = (val & 0x00FF) as u8;
}

fn reg_add_assign_into(reg: &mut Register, rhs: u16) {
    let val: u16 = u16::from(*reg).wrapping_add(rhs);
    reg.hi = ((val & 0xFF00) >> 8) as u8;
    reg.lo = (val & 0x00FF) as u8;
}

fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("reg_into", |b| b.iter(|| reg_into()));
    c.bench_function("reg_inline", |b| b.iter(|| reg_inline()));

    c.bench_function("reg_add_assign", |b| {
        b.iter(|| reg_add_assign(black_box(&mut Register::new(0, 0)), black_box(5)))
    });
    c.bench_function("reg_add_assign_into", |b| {
        b.iter(|| reg_add_assign_into(black_box(&mut Register::new(0, 0)), black_box(5)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
