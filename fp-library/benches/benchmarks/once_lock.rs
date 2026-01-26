use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::{
	brands::OnceLockBrand,
	classes::once::{
		get as once_get, get_or_init as once_get_or_init, new as once_new, set as once_set,
		take as once_take,
	},
};
use std::sync::OnceLock;

pub fn bench_once_lock(c: &mut Criterion) {
	let input_desc = "OnceLock";

	// New
	{
		let mut group = c.benchmark_group("OnceLock New");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| OnceLock::<i32>::new())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| once_new::<OnceLockBrand, i32>())
		});
		group.finish();
	}

	// Get
	{
		let cell = OnceLock::new();
		let _ = cell.set(42);
		let mut group = c.benchmark_group("OnceLock Get");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| cell.get())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| once_get::<OnceLockBrand, _>(&cell))
		});
		group.finish();
	}

	// Set
	{
		let mut group = c.benchmark_group("OnceLock Set");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(|| OnceLock::new(), |cell| cell.set(42), BatchSize::SmallInput)
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| once_new::<OnceLockBrand, i32>(),
				|cell| once_set::<OnceLockBrand, _>(&cell, 42),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Get Or Init
	{
		let mut group = c.benchmark_group("OnceLock Get Or Init");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| OnceLock::new(),
				|cell| {
					cell.get_or_init(|| 42);
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| once_new::<OnceLockBrand, i32>(),
				|cell| {
					once_get_or_init::<OnceLockBrand, _, _>(&cell, || 42);
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Take
	{
		let mut group = c.benchmark_group("OnceLock Take");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| {
					let cell = OnceLock::new();
					let _ = cell.set(42);
					cell
				},
				|mut cell| cell.take(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| {
					let cell = once_new::<OnceLockBrand, i32>();
					let _ = once_set::<OnceLockBrand, _>(&cell, 42);
					cell
				},
				|mut cell| once_take::<OnceLockBrand, _>(&mut cell),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}
}
