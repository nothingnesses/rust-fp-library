use {
	criterion::{
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::*,
		functions::*,
	},
};

pub fn bench_result(c: &mut Criterion) {
	let val_ok: Result<i32, i32> = Ok(42);
	let input_desc = "Ok(42)";

	// Map (ResultErrAppliedBrand - maps Ok)
	{
		let mut group = c.benchmark_group("Result Map");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val_ok).map(|x| x * 2))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				explicit::map::<ResultErrAppliedBrand<i32>, _, _, _>(
					|x| x * 2,
					std::hint::black_box(val_ok),
				)
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
				explicit::fold_right::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, _, _>(
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
				explicit::fold_left::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, _, _>(
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
				explicit::traverse::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, OptionBrand, _, _>(
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
				sequence::<ResultErrAppliedBrand<i32>, _, OptionBrand>(std::hint::black_box(
					val_opt,
				))
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
				explicit::bind::<ResultErrAppliedBrand<i32>, _, _, _, _>(
					std::hint::black_box(val_ok),
					|x| Ok(x * 2),
				)
			})
		});
		group.finish();
	}

	// Lift2
	{
		let val2: Result<i32, i32> = Ok(10);
		let mut group = c.benchmark_group("Result Lift2");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				std::hint::black_box(val_ok).and_then(|x| std::hint::black_box(val2).map(|y| x + y))
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				explicit::lift2::<ResultErrAppliedBrand<i32>, _, _, _, _, _, _>(
					|x, y| x + y,
					std::hint::black_box(val_ok),
					std::hint::black_box(val2),
				)
			})
		});
		group.finish();
	}

	// Pure
	{
		let mut group = c.benchmark_group("Result Pure");
		group.bench_with_input(BenchmarkId::new("std", "42"), &42, |b, &i| {
			b.iter(|| Ok::<_, i32>(i))
		});
		group.bench_with_input(BenchmarkId::new("fp", "42"), &42, |b, &i| {
			b.iter(|| pure::<ResultErrAppliedBrand<i32>, _>(i))
		});
		group.finish();
	}

	// Apply
	{
		let f: Result<_, i32> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		let mut group = c.benchmark_group("Result Apply");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| match (std::hint::black_box(f.clone()), std::hint::black_box(val_ok)) {
				(Ok(f), Ok(x)) => Ok(f(x)),
				(Err(e), _) => Err(e),
				(_, Err(e)) => Err(e),
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
					std::hint::black_box(f.clone()),
					std::hint::black_box(val_ok),
				)
			})
		});
		group.finish();
	}
}
