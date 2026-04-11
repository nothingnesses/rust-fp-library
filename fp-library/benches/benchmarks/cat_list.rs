use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::*,
		functions::*,
		types::cat_list::CatList,
	},
	std::{
		collections::LinkedList,
		hint::black_box,
	},
};

/// Benchmarks for CatList operations.
///
/// Compares CatList performance against Vec and LinkedList for common operations
/// across multiple input sizes to show scaling behavior.
///
/// Key performance characteristics tested:
/// - Cons (Prepend): Should be O(1), comparable to LinkedList, faster than Vec::insert(0).
/// - Snoc (Append element): Should be O(1), comparable to Vec::push.
/// - Append (Concatenation): Should be O(1), significantly faster than Vec::append (O(n)).
/// - Uncons (Head/Tail): Should be amortized O(1), comparable to LinkedList::pop_front.
/// - Left-Associated Append: Tests the "Reflection without Remorse" advantage - O(1) vs O(n^2).
/// - Iteration: Measures the overhead of iterating through the flattened structure.
/// - Nested Uncons: Tests uncons performance on deeply nested structures.
pub fn bench_cat_list(c: &mut Criterion) {
	let sizes: &[i32] = &[100, 200, 500, 1000, 2000, 5000];

	// Cons (Prepend)
	{
		let mut group = c.benchmark_group("CatList Cons");
		for &size in sizes {
			group.bench_with_input(BenchmarkId::new("CatList", size), &size, |b, &s| {
				b.iter_batched(
					|| CatList::empty(),
					|mut list| {
						for i in 0 .. s {
							list = list.cons(i);
						}
						list
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("LinkedList", size), &size, |b, &s| {
				b.iter_batched(
					|| LinkedList::new(),
					|mut list| {
						for i in 0 .. s {
							list.push_front(i);
						}
						list
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec", size), &size, |b, &s| {
				b.iter_batched(
					|| Vec::new(),
					|mut list| {
						for i in 0 .. s {
							list.insert(0, i);
						}
						list
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Snoc (Append element)
	{
		let mut group = c.benchmark_group("CatList Snoc");
		for &size in sizes {
			group.bench_with_input(BenchmarkId::new("CatList", size), &size, |b, &s| {
				b.iter_batched(
					|| CatList::empty(),
					|mut list| {
						for i in 0 .. s {
							list = list.snoc(i);
						}
						list
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec", size), &size, |b, &s| {
				b.iter_batched(
					|| Vec::new(),
					|mut list| {
						for i in 0 .. s {
							list.push(i);
						}
						list
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Append (Concatenation)
	{
		let mut group = c.benchmark_group("CatList Append");
		for &size in sizes {
			let list1: CatList<i32> = (0 .. size).collect();
			let list2: CatList<i32> = (0 .. size).collect();
			let vec1: Vec<i32> = (0 .. size).collect();
			let vec2: Vec<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList", size), &size, |b, &_| {
				b.iter_batched(
					|| (list1.clone(), list2.clone()),
					|(l1, l2)| l1.append(l2),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec", size), &size, |b, &_| {
				b.iter_batched(
					|| (vec1.clone(), vec2.clone()),
					|(mut v1, v2)| {
						v1.extend(v2);
						v1
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Uncons (Head/Tail)
	{
		let mut group = c.benchmark_group("CatList Uncons");
		for &size in sizes {
			let list: CatList<i32> = (0 .. size).collect();
			let vec: Vec<i32> = (0 .. size).collect();
			let linked_list: LinkedList<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList", size), &size, |b, &_| {
				b.iter_batched(
					|| list.clone(),
					|mut l| {
						while let Some((_, tail)) = l.uncons() {
							l = tail;
						}
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (remove(0))", size), &size, |b, &_| {
				b.iter_batched(
					|| vec.clone(),
					|mut v| {
						while !v.is_empty() {
							v.remove(0);
						}
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("LinkedList", size), &size, |b, &_| {
				b.iter_batched(
					|| linked_list.clone(),
					|mut l| {
						while l.pop_front().is_some() {}
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Append Two Lists of Size N
	// The key benchmark for CatList's O(1) append vs Vec's O(n) extend.
	// Pre-builds two lists of size N, then appends them.
	{
		let mut group = c.benchmark_group("CatList Append Two Lists");
		for &size in sizes {
			let cat1: CatList<i32> = (0 .. size).collect();
			let cat2: CatList<i32> = (size .. size * 2).collect();
			let vec1: Vec<i32> = (0 .. size).collect();
			let vec2: Vec<i32> = (size .. size * 2).collect();

			group.bench_with_input(BenchmarkId::new("CatList", size), &size, |b, &_| {
				b.iter_batched(
					|| (cat1.clone(), cat2.clone()),
					|(l1, l2)| l1.append(l2),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec", size), &size, |b, &_| {
				b.iter_batched(
					|| (vec1.clone(), vec2.clone()),
					|(mut v1, v2)| {
						v1.extend(v2);
						v1
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Left-Associated Append then Consume
	// Pattern: ((list ++ chunk) ++ chunk) ++ chunk ... then iterate the result.
	// This demonstrates CatList's O(1) append + amortized O(1) uncons vs
	// Vec's O(n) extend per step. Appends k chunks of size 10, then iterates.
	// Total elements = k * 10. CatList: O(k) append + O(k*10) iterate = O(n).
	// Vec: O(k * growing_size) append = O(n^2).
	{
		let chunk_counts: &[i32] = &[10, 50, 100, 200, 500];
		let chunk_size = 10;
		let mut group = c.benchmark_group("CatList Left-Assoc Append+Consume");
		for &chunks in chunk_counts {
			let cat_chunk: CatList<i32> = (0 .. chunk_size).collect();
			let vec_chunk: Vec<i32> = (0 .. chunk_size).collect();
			let label = chunks * chunk_size;

			group.bench_with_input(BenchmarkId::new("CatList", label), &chunks, |b, &k| {
				b.iter_batched(
					|| cat_chunk.clone(),
					|chunk| {
						let mut list = chunk;
						for _ in 1 .. k {
							list = list.append(cat_chunk.clone());
						}
						let mut sum = 0i32;
						for item in list {
							sum = sum.wrapping_add(item);
						}
						black_box(sum)
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec", label), &chunks, |b, &k| {
				b.iter_batched(
					|| vec_chunk.clone(),
					|chunk| {
						let mut v = chunk;
						for _ in 1 .. k {
							v.extend(vec_chunk.clone());
						}
						let mut sum = 0i32;
						for item in &v {
							sum = sum.wrapping_add(*item);
						}
						black_box(sum)
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Iteration (Measures overhead of flattening the internal structure)
	// CatList iteration involves dynamic flattening, which is more expensive than Vec iteration.
	{
		let mut group = c.benchmark_group("CatList Iteration");
		for &size in sizes {
			let cat_list: CatList<i32> = (0 .. size).collect();
			let vec_list: Vec<i32> = (0 .. size).collect();
			let linked_list: LinkedList<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList", size), &size, |b, &_| {
				b.iter_batched(
					|| cat_list.clone(),
					|list| {
						let mut sum = 0i32;
						for item in list {
							sum = sum.wrapping_add(item);
						}
						black_box(sum)
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| {
						let mut sum = 0i32;
						for item in v {
							sum = sum.wrapping_add(item);
						}
						black_box(sum)
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("LinkedList", size), &size, |b, &_| {
				b.iter_batched(
					|| linked_list.clone(),
					|l| {
						let mut sum = 0i32;
						for item in l {
							sum = sum.wrapping_add(item);
						}
						black_box(sum)
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Nested Uncons (Tests uncons on deeply nested structures created via left-associated appends)
	// This verifies that the flattening logic in uncons is efficient.
	{
		let mut group = c.benchmark_group("CatList Nested Uncons");
		for &size in sizes {
			let nested_cat_list: CatList<i32> = (0 .. size).fold(CatList::empty(), |acc, i| {
				if acc.is_empty() {
					CatList::singleton(i)
				} else {
					acc.append(CatList::singleton(i))
				}
			});
			let flat_cat_list: CatList<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList (nested)", size), &size, |b, &_| {
				b.iter_batched(
					|| nested_cat_list.clone(),
					|mut l| {
						while let Some((_, tail)) = l.uncons() {
							l = tail;
						}
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("CatList (flat)", size), &size, |b, &_| {
				b.iter_batched(
					|| flat_cat_list.clone(),
					|mut l| {
						while let Some((_, tail)) = l.uncons() {
							l = tail;
						}
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// -- Type class operations --

	// Fold Map: CatList vs Vec
	{
		let mut group = c.benchmark_group("CatList Fold Map");
		for &size in sizes {
			let cat_list: CatList<i32> = (0 .. size).collect();
			let vec_list: Vec<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| cat_list.clone(),
					|list| {
						fold_map::<RcFnBrand, CatListBrand, _, _, _, _>(
							|x: i32| x.to_string(),
							list,
						)
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| fold_map::<RcFnBrand, VecBrand, _, _, _, _>(|x: i32| x.to_string(), v),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (std)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| v.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(""),
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Fold Left: CatList vs Vec
	{
		let mut group = c.benchmark_group("CatList Fold Left");
		for &size in sizes {
			let cat_list: CatList<i32> = (0 .. size).collect();
			let vec_list: Vec<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| cat_list.clone(),
					|list| {
						fold_left::<RcFnBrand, CatListBrand, _, _, _, _>(
							|acc, x: i32| acc + x as i64,
							0i64,
							list,
						)
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| {
						fold_left::<RcFnBrand, VecBrand, _, _, _, _>(
							|acc, x: i32| acc + x as i64,
							0i64,
							v,
						)
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (std)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| v.into_iter().fold(0i64, |acc, x| acc + x as i64),
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Traverse (Option): CatList vs Vec
	{
		let mut group = c.benchmark_group("CatList Traverse");
		for &size in sizes {
			let cat_list: CatList<i32> = (0 .. size).collect();
			let vec_list: Vec<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| cat_list.clone(),
					|list| {
						traverse::<RcFnBrand, CatListBrand, _, _, OptionBrand, _, _>(
							|x: i32| Some(x + 1),
							list,
						)
					},
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| {
						traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
							|x: i32| Some(x + 1),
							v,
						)
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Filter: CatList vs Vec
	{
		let mut group = c.benchmark_group("CatList Filter");
		for &size in sizes {
			let cat_list: CatList<i32> = (0 .. size).collect();
			let vec_list: Vec<i32> = (0 .. size).collect();

			group.bench_with_input(BenchmarkId::new("CatList (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| cat_list.clone(),
					|list| filter_explicit::<CatListBrand, _, _, _>(|x: i32| x % 2 == 0, list),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| filter_explicit::<VecBrand, _, _, _>(|x: i32| x % 2 == 0, v),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (std)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_list.clone(),
					|v| v.into_iter().filter(|x| x % 2 == 0).collect::<Vec<_>>(),
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Compact: CatList vs Vec
	{
		let mut group = c.benchmark_group("CatList Compact");
		for &size in sizes {
			let cat_opts: CatList<Option<i32>> =
				(0 .. size).map(|x| if x % 3 == 0 { None } else { Some(x) }).collect();
			let vec_opts: Vec<Option<i32>> =
				(0 .. size).map(|x| if x % 3 == 0 { None } else { Some(x) }).collect();

			group.bench_with_input(BenchmarkId::new("CatList (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| cat_opts.clone(),
					|list| compact::<CatListBrand, _>(list),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (fp)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_opts.clone(),
					|v| compact::<VecBrand, _>(v),
					BatchSize::SmallInput,
				)
			});
			group.bench_with_input(BenchmarkId::new("Vec (std)", size), &size, |b, &_| {
				b.iter_batched(
					|| vec_opts.clone(),
					|v| v.into_iter().flatten().collect::<Vec<_>>(),
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}
}
