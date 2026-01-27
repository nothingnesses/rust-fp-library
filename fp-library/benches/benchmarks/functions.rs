use criterion::{BenchmarkId, Criterion};
use fp_library::functions::identity;

pub fn bench_functions(c: &mut Criterion) {
	let val = 42;
	let input_desc = "42";

	// Identity
	{
		let mut group = c.benchmark_group("Functions Identity");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::convert::identity(std::hint::black_box(val)))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| identity(std::hint::black_box(val)))
		});
		group.finish();
	}
}
