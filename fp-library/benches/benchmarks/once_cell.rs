use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::{
	brands::OnceCellBrand,
	classes::once::{
		get as once_get, get_or_init as once_get_or_init, new as once_new, set as once_set,
		take as once_take,
	},
};
use std::cell::OnceCell;

pub fn bench_once_cell(c: &mut Criterion) {
	let input_desc = "OnceCell";

	// New
	{
		let mut group = c.benchmark_group("OnceCell New");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| OnceCell::<i32>::new())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| once_new::<OnceCellBrand, i32>())
		});
		group.finish();
	}

	// Get
	{
		let cell = OnceCell::new();
		let _ = cell.set(42);
		let mut group = c.benchmark_group("OnceCell Get");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| cell.get())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| once_get::<OnceCellBrand, _>(&cell))
		});
		group.finish();
	}

	// Set
	{
		let mut group = c.benchmark_group("OnceCell Set");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(|| OnceCell::new(), |cell| cell.set(42), BatchSize::SmallInput)
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| once_new::<OnceCellBrand, i32>(),
				|cell| once_set::<OnceCellBrand, _>(&cell, 42),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Get Or Init
	{
		let mut group = c.benchmark_group("OnceCell Get Or Init");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| OnceCell::new(),
				|cell| {
					cell.get_or_init(|| 42);
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| once_new::<OnceCellBrand, i32>(),
				|cell| {
					once_get_or_init::<OnceCellBrand, _, _>(&cell, || 42);
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Take
	{
		let mut group = c.benchmark_group("OnceCell Take");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| {
					let cell = OnceCell::new();
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
					let cell = once_new::<OnceCellBrand, i32>();
					let _ = once_set::<OnceCellBrand, _>(&cell, 42);
					cell
				},
				|mut cell| once_take::<OnceCellBrand, _>(&mut cell),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}
}
