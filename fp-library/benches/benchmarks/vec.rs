use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::{
	brands::{RcFnBrand, ResultWithErrBrand, VecBrand},
	classes::{
		foldable::{fold_left, fold_map, fold_right},
		functor::map,
		monoid::empty,
		semigroup::append,
		semimonad::bind,
		traversable::{sequence, traverse},
	},
};

pub fn bench_vec(c: &mut Criterion) {
	let size = 1000;
	let v_orig: Vec<i32> = (0..size).collect();

	// Map
	{
		let mut group = c.benchmark_group("Vec Map");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().map(|x| x * 2).collect::<Vec<_>>(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| map::<VecBrand, _, _, _>(|x| x * 2, v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Fold Right
	// std: rev().fold()
	{
		let mut group = c.benchmark_group("Vec Fold Right");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().rev().fold(0, |acc, x| x + acc),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| fold_right::<RcFnBrand, VecBrand, _, _, _>(|x, acc| x + acc, 0, v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Fold Left
	// std: fold()
	{
		let mut group = c.benchmark_group("Vec Fold Left");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().fold(0, |acc, x| acc + x),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| fold_left::<RcFnBrand, VecBrand, _, _, _>(|acc, x| acc + x, 0, v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Fold Map
	// std: map().fold() (or just fold with accumulation)
	{
		let mut group = c.benchmark_group("Vec Fold Map");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().map(|x| x.to_string()).fold(String::new(), |acc, x| acc + &x),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| fold_map::<RcFnBrand, VecBrand, _, _, _>(|x: i32| x.to_string(), v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Traverse (Result)
	// std: map().collect::<Result<Vec<_>, _>>()
	let v_res: Vec<Result<i32, i32>> = (0..size).map(|x| Ok(x)).collect();
	{
		let mut group = c.benchmark_group("Vec Traverse");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().map(|x| Ok::<i32, i32>(x * 2)).collect::<Result<Vec<_>, _>>(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| traverse::<VecBrand, ResultWithErrBrand<i32>, _, _, _>(|x| Ok(x * 2), v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Sequence (Result)
	// std: collect::<Result<Vec<_>, _>>()
	{
		let mut group = c.benchmark_group("Vec Sequence");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_res.clone(),
				|v| v.into_iter().collect::<Result<Vec<_>, _>>(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_res.clone(),
				|v| sequence::<VecBrand, ResultWithErrBrand<i32>, _>(v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Bind
	// std: flat_map().collect()
	{
		let mut group = c.benchmark_group("Vec Bind");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().flat_map(|x| vec![x, x * 2]).collect::<Vec<_>>(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| bind::<VecBrand, _, _, _>(v, |x| vec![x, x * 2]),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Append
	let v2 = v_orig.clone();
	{
		let mut group = c.benchmark_group("Vec Append");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| (v_orig.clone(), v2.clone()),
				|(a, b)| [a, b].concat(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| (v_orig.clone(), v2.clone()),
				|(a, b)| append(a, b),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Empty
	{
		let mut group = c.benchmark_group("Vec Empty");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter(|| Vec::<i32>::new())
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter(|| empty::<Vec<i32>>())
		});
		group.finish();
	}

	// Construct
	{
		let mut group = c.benchmark_group("Vec Construct");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| (1, v_orig.clone()),
				|(x, v)| [vec![x], v].concat(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| (1, v_orig.clone()),
				|(x, v)| VecBrand::construct(x, v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Deconstruct
	{
		let mut group = c.benchmark_group("Vec Deconstruct");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.split_first().map(|(h, t)| (h.clone(), t.to_vec())),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(|| v_orig.clone(), |v| VecBrand::deconstruct(&v), BatchSize::SmallInput)
		});
		group.finish();
	}
}
