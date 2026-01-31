use criterion::{BenchmarkId, Criterion};
use fp_library::{
	brands::{ArcFnBrand, OptionBrand, RcFnBrand, ResultWithErrBrand},
	classes::{
		compactable::{compact, separate},
		filterable::{filter, filter_map, partition, partition_map},
		foldable::{fold_left, fold_right},
		functor::map,
		lift::lift2,
		par_foldable::par_fold_map,
		pointed::pure,
		semiapplicative::apply,
		semimonad::bind,
		traversable::{sequence, traverse},
		witherable::{wilt, wither},
	},
	functions::{cloneable_fn_new, send_cloneable_fn_new},
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
				traverse::<OptionBrand, _, _, ResultWithErrBrand<i32>, _>(
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
				sequence::<OptionBrand, _, ResultWithErrBrand<i32>>(std::hint::black_box(val_res))
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

	// Filter
	{
		let mut group = c.benchmark_group("Option Filter");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val).filter(|x| x % 2 == 0))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| filter::<OptionBrand, _, _>(|x| x % 2 == 0, std::hint::black_box(val)))
		});
		group.finish();
	}

	// Filter Map
	{
		let mut group = c.benchmark_group("Option Filter Map");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				std::hint::black_box(val).and_then(|x| if x % 2 == 0 { Some(x * 2) } else { None })
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				filter_map::<OptionBrand, _, _, _>(
					|x| if x % 2 == 0 { Some(x * 2) } else { None },
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Partition
	{
		let mut group = c.benchmark_group("Option Partition");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				let v = std::hint::black_box(val);
				if v.map_or(false, |x| x % 2 == 0) { (v, None) } else { (None, v) }
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| partition::<OptionBrand, _, _>(|x| x % 2 == 0, std::hint::black_box(val)))
		});
		group.finish();
	}

	// Partition Map
	{
		let mut group = c.benchmark_group("Option Partition Map");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				std::hint::black_box(val).map_or((None, None), |x| {
					if x % 2 == 0 { (Some(x * 2), None) } else { (None, Some(x)) }
				})
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				partition_map::<OptionBrand, _, _, _, _>(
					|x| if x % 2 == 0 { Ok(x * 2) } else { Err(x) },
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Compact
	let val_nested = Some(Some(42));
	let input_desc_nested = "Some(Some(42))";
	{
		let mut group = c.benchmark_group("Option Compact");
		group.bench_with_input(
			BenchmarkId::new("std", input_desc_nested),
			&input_desc_nested,
			|b, &_| b.iter(|| std::hint::black_box(val_nested).flatten()),
		);
		group.bench_with_input(
			BenchmarkId::new("fp", input_desc_nested),
			&input_desc_nested,
			|b, &_| b.iter(|| compact::<OptionBrand, _>(std::hint::black_box(val_nested))),
		);
		group.finish();
	}

	// Separate
	let val_res_sep: Option<Result<i32, i32>> = Some(Ok(42));
	let input_desc_res_sep = "Some(Ok(42))";
	{
		let mut group = c.benchmark_group("Option Separate");
		group.bench_with_input(
			BenchmarkId::new("std", input_desc_res_sep),
			&input_desc_res_sep,
			|b, &_| {
				b.iter(|| match std::hint::black_box(val_res_sep) {
					Some(Ok(x)) => (Some(x), None),
					Some(Err(e)) => (None, Some(e)),
					None => (None, None),
				})
			},
		);
		group.bench_with_input(
			BenchmarkId::new("fp", input_desc_res_sep),
			&input_desc_res_sep,
			|b, &_| b.iter(|| separate::<OptionBrand, _, _>(std::hint::black_box(val_res_sep))),
		);
		group.finish();
	}

	// Wither
	{
		let mut group = c.benchmark_group("Option Wither");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				std::hint::black_box(val)
					.map(|x| if x % 2 == 0 { Some(Some(x * 2)) } else { Some(None) })
					.unwrap_or(Some(None))
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				wither::<OptionBrand, OptionBrand, _, _, _>(
					|x| Some(if x % 2 == 0 { Some(x * 2) } else { None }),
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Wilt
	{
		let mut group = c.benchmark_group("Option Wilt");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				std::hint::black_box(val)
					.map(|x| if x % 2 == 0 { Some(Ok(x * 2)) } else { Some(Err(x)) })
					.unwrap_or(Some(Ok(0))) // Dummy
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				wilt::<OptionBrand, OptionBrand, _, _, _, _>(
					|x| Some(if x % 2 == 0 { Ok(x * 2) } else { Err(x) }),
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Lift2
	{
		let val2 = Some(10);
		let mut group = c.benchmark_group("Option Lift2");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::hint::black_box(val).zip(std::hint::black_box(val2)).map(|(x, y)| x + y))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				lift2::<OptionBrand, _, _, _, _>(
					|x, y| x + y,
					std::hint::black_box(val),
					std::hint::black_box(val2),
				)
			})
		});
		group.finish();
	}

	// Pure
	{
		let mut group = c.benchmark_group("Option Pure");
		group.bench_with_input(BenchmarkId::new("std", "42"), &42, |b, &i| b.iter(|| Some(i)));
		group.bench_with_input(BenchmarkId::new("fp", "42"), &42, |b, &i| {
			b.iter(|| pure::<OptionBrand, _>(i))
		});
		group.finish();
	}

	// Apply
	{
		let f = Some(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		let mut group = c.benchmark_group("Option Apply");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| match (std::hint::black_box(f.clone()), std::hint::black_box(val)) {
				(Some(f), Some(x)) => Some(f(x)),
				_ => None,
			})
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				apply::<RcFnBrand, OptionBrand, _, _>(
					std::hint::black_box(f.clone()),
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}

	// Par Fold Map
	{
		let mut group = c.benchmark_group("Option Par Fold Map");
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| {
				par_fold_map::<ArcFnBrand, OptionBrand, _, _>(
					send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string()),
					std::hint::black_box(val),
				)
			})
		});
		group.finish();
	}
}
