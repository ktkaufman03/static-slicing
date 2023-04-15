use criterion::{black_box, criterion_group, criterion_main, Criterion};
use static_slicing::*;

fn potentially_panicking_index<'a>(d: &'a [u8; 8]) -> &'a [u8; 4] {
	let tmp = &d[4..8];
	tmp.try_into().unwrap()
}

fn compile_checked_index<'a>(d: &'a [u8; 8]) -> &'a [u8; 4] {
	&d[StaticRangeIndex::<4, 4>]
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("runtime checked range index", |b| b.iter(|| {
		potentially_panicking_index(&black_box([5u8; 8]));
	}));
    c.bench_function("compile-time checked range index", |b| b.iter(|| {
		compile_checked_index(&black_box([5u8; 8]));
	}));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);