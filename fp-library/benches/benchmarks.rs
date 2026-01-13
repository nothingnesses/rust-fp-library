use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use fp_library::{
	brands::{OnceCellBrand, OnceLockBrand, OptionBrand, ResultWithErrBrand, VecBrand},
	classes::{
		foldable::{fold_left, fold_map, fold_right},
		functor::map,
		monoid::empty,
		once::{
			get as once_get, get_or_init as once_get_or_init, new as once_new, set as once_set,
			take as once_take,
		},
		semigroup::append,
		semimonad::bind,
		traversable::{sequence, traverse},
	},
	functions::identity,
};
use std::{cell::OnceCell, sync::OnceLock};

fn bench_vec(c: &mut Criterion) {
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
				|v| fold_right::<VecBrand, _, _, _>(|x, acc| x + acc, 0, v),
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
				|v| fold_left::<VecBrand, _, _, _>(|acc, x| acc + x, 0, v),
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
				|v| fold_map::<VecBrand, _, _, _>(|x: i32| x.to_string(), v),
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

fn bench_option(c: &mut Criterion) {
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
				fold_right::<OptionBrand, _, _, _>(|x, acc| x + acc, 0, std::hint::black_box(val))
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
				fold_left::<OptionBrand, _, _, _>(|acc, x| acc + x, 0, std::hint::black_box(val))
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

fn bench_result(c: &mut Criterion) {
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
				fold_right::<ResultWithErrBrand<i32>, _, _, _>(
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
				fold_left::<ResultWithErrBrand<i32>, _, _, _>(
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

fn bench_string(c: &mut Criterion) {
	let s1 = "Hello".to_string();
	let s2 = "World".to_string();
	let input_desc = "String";

	// Append
	{
		let mut group = c.benchmark_group("String Append");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter_batched(|| (s1.clone(), s2.clone()), |(a, b)| a + &b, BatchSize::SmallInput)
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter_batched(
				|| (s1.clone(), s2.clone()),
				|(a, b)| append(a, b),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Empty
	{
		let mut group = c.benchmark_group("String Empty");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| String::new())
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| empty::<String>())
		});
		group.finish();
	}
}

fn bench_functions(c: &mut Criterion) {
	let val = 42;
	let input_desc = "42";

	// Identity
	{
		let mut group = c.benchmark_group("Functions Identity");
		group.bench_with_input(BenchmarkId::new("std", input_desc), &input_desc, |b, &_| {
			b.iter(|| std::convert::identity(std::hint::black_box(val)))
		});
		group.bench_with_input(BenchmarkId::new("fp", input_desc), &input_desc, |b, &_| {
			b.iter(|| identity(std::hint::black_box(val)))
		});
		group.finish();
	}
}

fn bench_once_cell(c: &mut Criterion) {
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

fn bench_once_lock(c: &mut Criterion) {
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

criterion_group!(
	benches,
	bench_vec,
	bench_option,
	bench_result,
	bench_string,
	bench_functions,
	bench_once_cell,
	bench_once_lock
);
criterion_main!(benches);
