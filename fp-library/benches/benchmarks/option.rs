use criterion::{BenchmarkId, Criterion};
use fp_library::{
	brands::{OptionBrand, RcFnBrand, ResultWithErrBrand},
	classes::{
		foldable::{fold_left, fold_right},
		functor::map,
		semimonad::bind,
		traversable::{sequence, traverse},
	},
};

pub fn bench_option(c: &mut Criterion) {
	let val = Some(42);
	let input_desc = "Some(42)";

	// Map
	{
		let mut group = c.benchmark_group("Option Map");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val).map(|x| x * 2))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| map::<OptionBrand, _, _, _>(|x| x * 2, std::hint::black_box(val)))
		});
		group.finish();
	}

	// Fold Right (match)
	{
		let mut group = c.benchmark_group("Option Fold Right");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val).map_or(0, |x| x + 0))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				fold_right::<RcFnBrand, OptionBrand, _, _, _>(
					|x, acc| x + acc,
					0,
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Fold Left
	{
		let mut group = c.benchmark_group("Option Fold Left");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val).map_or(0, |x| 0 + x))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				fold_left::<RcFnBrand, OptionBrand, _, _, _>(
					|acc, x| acc + x,
					0,
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Traverse (Result)
	{
		let mut group = c.benchmark_group("Option Traverse");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val).map(|x| Ok::<i32, i32>(x * 2)).transpose())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				traverse::<OptionBrand, ResultWithErrBrand<i32>, _, _, _>(
					|x| Ok(x * 2),
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Sequence (Result)
	let val_res: Option<Result<i32, i32>> = Some(Ok(42));
	let input_desc_res = "Some(Ok(42))";
	{
		let mut group = c.benchmark_group("Option Sequence");
		group.bench_with_input(
			BenchmarkId::new("std", input_desc_res),
			&input_desc_res,
			|b, &_| b.iter(|| std::hint::black_box(val_res).transpose()),
		);
		group.bench_with_input(BenchmarkId::new("fp", input_desc_res), &input_desc_res, |b, &_| {
			b.iter(|| {
				sequence::<OptionBrand, ResultWithErrBrand<i32>, _>(std::hint::black_box(val_res))
			})
		});
		group.finish();
	}

	// Bind
	{
		let mut group = c.benchmark_group("Option Bind");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val).and_then(|x| Some(x * 2)))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| bind::<OptionBrand, _, _, _>(std::hint::black_box(val), |x| Some(x * 2)))
		});
		group.finish();
	}
}
