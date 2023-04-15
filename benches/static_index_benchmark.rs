use criterion::{black_box, criterion_group, criterion_main, Criterion};
use static_slicing::*;

fn potentially_panicking_index(d: &[u8; 8]) -> u8 {
    d[4]
}

fn compile_checked_index(d: &[u8; 8]) -> u8 {
	d[StaticIndex::<4>]
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("runtime checked single index", |b| b.iter(|| potentially_panicking_index(&black_box([5u8; 8]))));
    c.bench_function("compile-time checked single index", |b| b.iter(|| compile_checked_index(&black_box([5u8; 8]))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);