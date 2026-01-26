use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::{
	brands::{ArcFnBrand, OptionBrand, RcFnBrand, ResultWithErrBrand, VecBrand},
	classes::{
		compactable::{compact, separate},
		filterable::{filter, filter_map, partition, partition_map},
		foldable::{fold_left, fold_map, fold_right},
		functor::map,
		lift::lift2,
		monoid::empty,
		par_foldable::par_fold_map,
		pointed::pure,
		semiapplicative::apply,
		semigroup::append,
		semimonad::bind,
		traversable::{sequence, traverse},
		witherable::{wilt, wither},
	},
	functions::{cloneable_fn_new, send_cloneable_fn_new},
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

	// Filter
	{
		let mut group = c.benchmark_group("Vec Filter");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().filter(|x| x % 2 == 0).collect::<Vec<_>>(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| filter::<VecBrand, _, _>(|x| x % 2 == 0, v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Filter Map
	{
		let mut group = c.benchmark_group("Vec Filter Map");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					v.into_iter()
						.filter_map(|x| if x % 2 == 0 { Some(x * 2) } else { None })
						.collect::<Vec<_>>()
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					filter_map::<VecBrand, _, _, _>(
						|x| if x % 2 == 0 { Some(x * 2) } else { None },
						v,
					)
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Partition
	{
		let mut group = c.benchmark_group("Vec Partition");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| v.into_iter().partition::<Vec<_>, _>(|x| x % 2 == 0),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| partition::<VecBrand, _, _>(|x| x % 2 == 0, v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Partition Map
	{
		let mut group = c.benchmark_group("Vec Partition Map");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					let mut oks = Vec::new();
					let mut errs = Vec::new();
					for x in v {
						if x % 2 == 0 {
							oks.push(x * 2);
						} else {
							errs.push(x);
						}
					}
					(oks, errs)
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					partition_map::<VecBrand, _, _, _, _>(
						|x| if x % 2 == 0 { Ok(x * 2) } else { Err(x) },
						v,
					)
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Compact
	let v_nested: Vec<Option<i32>> =
		(0..size).map(|x| if x % 2 == 0 { Some(x) } else { None }).collect();
	{
		let mut group = c.benchmark_group("Vec Compact");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_nested.clone(),
				|v| v.into_iter().flatten().collect::<Vec<_>>(),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_nested.clone(),
				|v| compact::<VecBrand, _>(v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Separate
	let v_res_sep: Vec<Result<i32, i32>> =
		(0..size).map(|x| if x % 2 == 0 { Ok(x) } else { Err(x) }).collect();
	{
		let mut group = c.benchmark_group("Vec Separate");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_res_sep.clone(),
				|v| {
					let mut oks = Vec::new();
					let mut errs = Vec::new();
					for res in v {
						match res {
							Ok(o) => oks.push(o),
							Err(e) => errs.push(e),
						}
					}
					(oks, errs)
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_res_sep.clone(),
				|v| separate::<VecBrand, _, _>(v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Wither
	{
		let mut group = c.benchmark_group("Vec Wither");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					let mut res = Vec::new();
					for x in v {
						if x % 2 == 0 {
							res.push(x * 2);
						}
					}
					Some(res)
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					wither::<VecBrand, OptionBrand, _, _, _>(
						|x| Some(if x % 2 == 0 { Some(x * 2) } else { None }),
						v,
					)
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Wilt
	{
		let mut group = c.benchmark_group("Vec Wilt");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					let mut oks = Vec::new();
					let mut errs = Vec::new();
					for x in v {
						if x % 2 == 0 {
							oks.push(x * 2);
						} else {
							errs.push(x);
						}
					}
					Some((oks, errs))
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					wilt::<VecBrand, OptionBrand, _, _, _, _>(
						|x| Some(if x % 2 == 0 { Ok(x * 2) } else { Err(x) }),
						v,
					)
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Lift2
	{
		let v2 = v_orig.clone();
		let mut group = c.benchmark_group("Vec Lift2");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| (v_orig.clone(), v2.clone()),
				|(v1, v2)| {
					v1.iter().flat_map(|x| v2.iter().map(move |y| x + y)).collect::<Vec<_>>()
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| (v_orig.clone(), v2.clone()),
				|(v1, v2)| lift2::<VecBrand, _, _, _, _>(|x, y| x + y, v1, v2),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Pure
	{
		let mut group = c.benchmark_group("Vec Pure");
		group.bench_with_input(BenchmarkId::new("std", "42"), &42, |b, &i| b.iter(|| vec![i]));
		group.bench_with_input(BenchmarkId::new("fp", "42"), &42, |b, &i| {
			b.iter(|| pure::<VecBrand, _>(i))
		});
		group.finish();
	}

	// Apply
	{
		let f_vec = vec![
			cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2),
			cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1),
		];
		let mut group = c.benchmark_group("Vec Apply");
		group.bench_with_input(BenchmarkId::new("std", size), &size, |b, &_| {
			b.iter_batched(
				|| (f_vec.clone(), v_orig.clone()),
				|(fs, v)| {
					fs.iter().flat_map(|f| v.iter().map(move |x| f(x.clone()))).collect::<Vec<_>>()
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| (f_vec.clone(), v_orig.clone()),
				|(fs, v)| apply::<RcFnBrand, VecBrand, _, _>(fs, v),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Par Fold Map
	{
		let mut group = c.benchmark_group("Vec Par Fold Map");
		group.bench_with_input(BenchmarkId::new("fp", size), &size, |b, &_| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					par_fold_map::<ArcFnBrand, VecBrand, _, _>(
						send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string()),
						v,
					)
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}
}
