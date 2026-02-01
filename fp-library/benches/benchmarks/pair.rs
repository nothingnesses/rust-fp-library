use criterion::{BenchmarkId, Criterion};
use fp_library::{
	brands::{ArcFnBrand, OptionBrand, PairWithFirstBrand, RcFnBrand},
	classes::{
		foldable::{fold_left, fold_right},
		functor::map,
		lift::lift2,
		par_foldable::{par_fold_map, par_fold_right},
		pointed::pure,
		semiapplicative::apply,
		semimonad::bind,
		traversable::{sequence, traverse},
	},
	functions::{cloneable_fn_new, send_cloneable_fn_new},
	types::Pair,
};

/// Benchmarks for Pair operations.
///
/// Tests Pair performance for various type class methods.
/// Pair is benchmarked using PairWithFirstBrand (functor over the second element).
pub fn bench_pair(c: &mut Criterion) {
	let val = Pair("first".to_string(), 42);
	let input_desc = "Pair(\"first\", 42)";

	// Map
	{
		let mut group = c.benchmark_group("Pair Map");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let p = std::hint::black_box(val.clone());
				(p.0, p.1 * 2)
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				map::<PairWithFirstBrand<String>, _, _, _>(
					|x| x * 2,
					std::hint::black_box(val.clone()),
				)
			})
		});
		group.finish();
	}

	// Fold Right
	{
		let mut group = c.benchmark_group("Pair Fold Right");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let p = std::hint::black_box(val.clone());
				p.1 + 0
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				fold_right::<RcFnBrand, PairWithFirstBrand<String>, _, _, _>(
					|x, acc| x + acc,
					0,
					std::hint::black_box(val.clone()),
				)
			})
		});
		group.finish();
	}

	// Fold Left
	{
		let mut group = c.benchmark_group("Pair Fold Left");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let p = std::hint::black_box(val.clone());
				0 + p.1
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				fold_left::<RcFnBrand, PairWithFirstBrand<String>, _, _, _>(
					|acc, x| acc + x,
					0,
					std::hint::black_box(val.clone()),
				)
			})
		});
		group.finish();
	}

	// Traverse (Option)
	{
		let mut group = c.benchmark_group("Pair Traverse");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let p = std::hint::black_box(val.clone());
				Some(p.1 * 2).map(|x| (p.0, x))
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				traverse::<PairWithFirstBrand<String>, _, _, OptionBrand, _>(
					|x| Some(x * 2),
					std::hint::black_box(val.clone()),
				)
			})
		});
		group.finish();
	}

	// Sequence (Option)
	let val_opt = Pair("first".to_string(), Some(42));
	let input_desc_opt = "Pair(\"first\", Some(42))";
	{
		let mut group = c.benchmark_group("Pair Sequence");
		group.bench_with_input(
			BenchmarkId::new("std", input_desc_opt),
			&input_desc_opt,
			|b, &_| {
				b.iter(|| {
					let p = std::hint::black_box(val_opt.clone());
					p.1.map(|x| (p.0, x))
				})
			},
		);
		group.bench_with_input(BenchmarkId::new("fp", input_desc_opt), &input_desc_opt, |b, &_| {
			b.iter(|| {
				sequence::<PairWithFirstBrand<String>, _, OptionBrand>(std::hint::black_box(
					val_opt.clone(),
				))
			})
		});
		group.finish();
	}

	// Bind
	{
		let mut group = c.benchmark_group("Pair Bind");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let p = std::hint::black_box(val.clone());
				let p2 = Pair("second".to_string(), p.1 * 2);
				Pair(p.0 + &p2.0, p2.1)
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				bind::<PairWithFirstBrand<String>, _, _, _>(
					std::hint::black_box(val.clone()),
					|x| Pair("second".to_string(), x * 2),
				)
			})
		});
		group.finish();
	}

	// Lift2
	{
		let val2 = Pair("second".to_string(), 10);
		let mut group = c.benchmark_group("Pair Lift2");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let p1 = std::hint::black_box(val.clone());
				let p2 = std::hint::black_box(val2.clone());
				Pair(p1.0 + &p2.0, p1.1 + p2.1)
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				lift2::<PairWithFirstBrand<String>, _, _, _, _>(
					|x, y| x + y,
					std::hint::black_box(val.clone()),
					std::hint::black_box(val2.clone()),
				)
			})
		});
		group.finish();
	}

	// Pure
	{
		let mut group = c.benchmark_group("Pair Pure");
		group.bench_with_input(BenchmarkId::new("std", "42"), &42, |b, &i| {
			b.iter(|| Pair(String::new(), i))
		});
		group.bench_with_input(BenchmarkId::new("fp", "42"), &42, |b, &i| {
			b.iter(|| pure::<PairWithFirstBrand<String>, _>(i))
		});
		group.finish();
	}

	// Apply
	{
		let f = Pair("f".to_string(), cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		let mut group = c.benchmark_group("Pair Apply");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let pf = std::hint::black_box(f.clone());
				let px = std::hint::black_box(val.clone());
				Pair(pf.0 + &px.0, pf.1(px.1))
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
					std::hint::black_box(f.clone()),
					std::hint::black_box(val.clone()),
				)
			})
		});
		group.finish();
	}

	// Par Fold Map
	{
		let mut group = c.benchmark_group("Pair Par Fold Map");
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				par_fold_map::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(
					send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string()),
					std::hint::black_box(val.clone()),
				)
			})
		});
		group.finish();
	}

	// Par Fold Right
	{
		let mut group = c.benchmark_group("Pair Par Fold Right");
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				par_fold_right::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(
					send_cloneable_fn_new::<ArcFnBrand, _, _>(|(x, acc)| x + acc),
					0,
					std::hint::black_box(val.clone()),
				)
			})
		});
		group.finish();
	}
}
