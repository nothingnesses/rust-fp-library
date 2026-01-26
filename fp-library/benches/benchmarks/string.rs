use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::classes::{monoid::empty, semigroup::append};

pub fn bench_string(c: &mut Criterion) {
	let s1 = "Hello".to_string();
	let s2 = "World".to_string();
	let input_desc = "String";

	// Append
	{
		let mut group = c.benchmark_group("String Append");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(|| (s1.clone(), s2.clone()), |(a, b)| a + &b, BatchSize::SmallInput)
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| (s1.clone(), s2.clone()),
				|(a, b)| append(a, b),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Empty
	{
		let mut group = c.benchmark_group("String Empty");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| String::new())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| empty::<String>())
		});
		group.finish();
	}
}
