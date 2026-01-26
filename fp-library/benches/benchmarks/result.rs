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

pub fn bench_result(c: &mut Criterion) {
	let val_ok: Result<i32, i32> = Ok(42);
	let input_desc = "Ok(42)";

	// Map (ResultWithErrBrand - maps Ok)
	{
		let mut group = c.benchmark_group("Result Map");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val_ok).map(|x| x * 2))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				map::<ResultWithErrBrand<i32>, _, _, _>(|x| x * 2, std::hint::black_box(val_ok))
			})
		});
		group.finish();
	}

	// Fold Right
	{
		let mut group = c.benchmark_group("Result Fold Right");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val_ok).map_or(0, |x| x + 0))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				fold_right::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(
					|x, acc| x + acc,
					0,
					std::hint::black_box(val_ok),
				)
			})
		});
		group.finish();
	}

	// Fold Left
	{
		let mut group = c.benchmark_group("Result Fold Left");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val_ok).map_or(0, |x| 0 + x))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				fold_left::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(
					|acc, x| acc + x,
					0,
					std::hint::black_box(val_ok),
				)
			})
		});
		group.finish();
	}

	// Traverse (Option)
	{
		let mut group = c.benchmark_group("Result Traverse");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val_ok).map(|x| Some(x * 2)).transpose())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				traverse::<ResultWithErrBrand<i32>, OptionBrand, _, _, _>(
					|x| Some(x * 2),
					std::hint::black_box(val_ok),
				)
			})
		});
		group.finish();
	}

	// Sequence (Option)
	let val_opt: Result<Option<i32>, i32> = Ok(Some(42));
	let input_desc_opt = "Ok(Some(42))";
	{
		let mut group = c.benchmark_group("Result Sequence");
		group.bench_with_input(
			BenchmarkId::new("std", input_desc_opt),
			&input_desc_opt,
			|b, &_| b.iter(|| std::hint::black_box(val_opt).transpose()),
		);
		group.bench_with_input(BenchmarkId::new("fp", input_desc_opt), &input_desc_opt, |b, &_| {
			b.iter(|| {
				sequence::<ResultWithErrBrand<i32>, OptionBrand, _>(std::hint::black_box(val_opt))
			})
		});
		group.finish();
	}

	// Bind
	{
		let mut group = c.benchmark_group("Result Bind");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val_ok).and_then(|x| Ok(x * 2)))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				bind::<ResultWithErrBrand<i32>, _, _, _>(
					std::hint::black_box(val_ok),
					|x| Ok(x * 2),
				)
			})
		});
		group.finish();
	}
}
