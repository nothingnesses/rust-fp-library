use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::*,
		functions::*,
	},
};

pub fn bench_ref_dispatch(c: &mut Criterion) {
	// -- Option: Val vs Ref dispatch --

	let opt_val = Some(42);
	let opt_desc = "Some(42)";

	// Map: Val vs Ref
	{
		let mut group = c.benchmark_group("Dispatch Map Option");
		group.bench_with_input(BenchmarkId::new("val", opt_desc), &opt_desc, |b, &_| {
			b.iter(|| {
				explicit::map::<OptionBrand, _, _, _>(|x| x * 2, std::hint::black_box(opt_val))
			})
		});
		group.bench_with_input(BenchmarkId::new("ref", opt_desc), &opt_desc, |b, &_| {
			b.iter(|| {
				explicit::map::<OptionBrand, _, _, _>(
					|x: &i32| *x * 2,
					std::hint::black_box(&opt_val),
				)
			})
		});
		group.finish();
	}

	// Bind: Val vs Ref
	{
		let mut group = c.benchmark_group("Dispatch Bind Option");
		group.bench_with_input(BenchmarkId::new("val", opt_desc), &opt_desc, |b, &_| {
			b.iter(|| {
				explicit::bind::<OptionBrand, _, _, _, _>(std::hint::black_box(opt_val), |x| {
					Some(x * 2)
				})
			})
		});
		group.bench_with_input(BenchmarkId::new("ref", opt_desc), &opt_desc, |b, &_| {
			b.iter(|| {
				explicit::bind::<OptionBrand, _, _, _, _>(
					std::hint::black_box(&opt_val),
					|x: &i32| Some(*x * 2),
				)
			})
		});
		group.finish();
	}

	// Lift2: Val vs Ref
	{
		let opt_a = Some(42);
		let opt_b = Some(10);
		let mut group = c.benchmark_group("Dispatch Lift2 Option");
		group.bench_with_input(BenchmarkId::new("val", opt_desc), &opt_desc, |b, &_| {
			b.iter(|| {
				explicit::lift2::<OptionBrand, _, _, _, _, _, _>(
					|x, y| x + y,
					std::hint::black_box(opt_a),
					std::hint::black_box(opt_b),
				)
			})
		});
		group.bench_with_input(BenchmarkId::new("ref", opt_desc), &opt_desc, |b, &_| {
			b.iter(|| {
				explicit::lift2::<OptionBrand, _, _, _, _, _, _>(
					|x: &i32, y: &i32| *x + *y,
					std::hint::black_box(&opt_a),
					std::hint::black_box(&opt_b),
				)
			})
		});
		group.finish();
	}

	// -- Vec: Val vs Ref dispatch --

	let size = 1000;
	let v_orig: Vec<i32> = (0 .. size).collect();

	// Map: Val vs Ref
	{
		let mut group = c.benchmark_group("Dispatch Map Vec");
		group.bench_with_input(BenchmarkId::new("val", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| explicit::map::<VecBrand, _, _, _>(|x| x * 2, v),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("ref", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| explicit::map::<VecBrand, _, _, _>(|x: &i32| *x * 2, &v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Bind: Val vs Ref
	{
		let mut group = c.benchmark_group("Dispatch Bind Vec");
		group.bench_with_input(BenchmarkId::new("val", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| explicit::bind::<VecBrand, _, _, _, _>(v, |x| vec![x, x * 2]),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("ref", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| explicit::bind::<VecBrand, _, _, _, _>(&v, |x: &i32| vec![*x, *x * 2]),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Lift2: Val vs Ref
	{
		let v_a: Vec<i32> = (0 .. 30).collect();
		let v_b: Vec<i32> = (0 .. 30).collect();
		let lift_size = 30;
		let mut group = c.benchmark_group("Dispatch Lift2 Vec");
		group.bench_with_input(BenchmarkId::new("val", lift_size), &lift_size, |b, &_| {
			b.iter_batched(
				|| (v_a.clone(), v_b.clone()),
				|(a, b)| explicit::lift2::<VecBrand, _, _, _, _, _, _>(|x, y| x + y, a, b),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("ref", lift_size), &lift_size, |b, &_| {
			b.iter_batched(
				|| (v_a.clone(), v_b.clone()),
				|(a, b)| {
					explicit::lift2::<VecBrand, _, _, _, _, _, _>(
						|x: &i32, y: &i32| *x + *y,
						&a,
						&b,
					)
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}
}
