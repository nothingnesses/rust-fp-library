use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::types::cat_list::CatList;
use std::collections::LinkedList;
use std::hint::black_box;

/// Benchmarks for CatList operations.
///
/// Compares CatList performance against Vec and LinkedList for common operations.
///
/// Key performance characteristics tested:
/// - Cons (Prepend): Should be O(1), comparable to LinkedList, faster than Vec::insert(0).
/// - Snoc (Append element): Should be O(1), comparable to Vec::push.
/// - Append (Concatenation): Should be O(1), significantly faster than Vec::append (O(n)).
/// - Uncons (Head/Tail): Should be amortized O(1), comparable to LinkedList::pop_front.
/// - Left-Associated Append: Tests the "Reflection without Remorse" advantage - O(1) vs O(n²).
/// - Iteration: Measures the overhead of iterating through the flattened structure.
/// - Nested Uncons: Tests uncons performance on deeply nested structures.
pub fn bench_cat_list(c: &mut Criterion) {
	let size = 1000;
	let input_desc = format!("Size {}", size);

	// Cons (Prepend)
	{
		let mut group = c.benchmark_group("CatList Cons");
		group.bench_with_input(BenchmarkId::new("CatList", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| CatList::empty(),
				|mut list| {
					for i in 0..s {
						list = list.cons(i);
					}
					list
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("LinkedList", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| LinkedList::new(),
				|mut list| {
					for i in 0..s {
						list.push_front(i);
					}
					list
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("Vec", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| Vec::new(),
				|mut list| {
					for i in 0..s {
						list.insert(0, i);
					}
					list
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Snoc (Append element)
	{
		let mut group = c.benchmark_group("CatList Snoc");
		group.bench_with_input(BenchmarkId::new("CatList", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| CatList::empty(),
				|mut list| {
					for i in 0..s {
						list = list.snoc(i);
					}
					list
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("Vec", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| Vec::new(),
				|mut list| {
					for i in 0..s {
						list.push(i);
					}
					list
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Append (Concatenation)
	{
		let list1: CatList<i32> = (0..size).collect();
		let list2: CatList<i32> = (0..size).collect();
		let vec1: Vec<i32> = (0..size).collect();
		let vec2: Vec<i32> = (0..size).collect();

		let mut group = c.benchmark_group("CatList Append");
		group.bench_with_input(BenchmarkId::new("CatList", &input_desc), &size, |b, &_| {
			b.iter_batched(
				|| (list1.clone(), list2.clone()),
				|(l1, l2)| l1.append(l2),
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("Vec", &input_desc), &size, |b, &_| {
			b.iter_batched(
				|| (vec1.clone(), vec2.clone()),
				|(mut v1, v2)| {
					v1.extend(v2);
					v1
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Uncons (Head/Tail)
	{
		let list: CatList<i32> = (0..size).collect();
		let vec: Vec<i32> = (0..size).collect();
		let linked_list: LinkedList<i32> = (0..size).collect();

		let mut group = c.benchmark_group("CatList Uncons");
		group.bench_with_input(BenchmarkId::new("CatList", &input_desc), &size, |b, &_| {
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
		group.bench_with_input(BenchmarkId::new("Vec (remove(0))", &input_desc), &size, |b, &_| {
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
		group.bench_with_input(BenchmarkId::new("LinkedList", &input_desc), &size, |b, &_| {
			b.iter_batched(
				|| linked_list.clone(),
				|mut l| {
					while l.pop_front().is_some() {}
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Left-Associated Append (The "Torture Test")
	// This is the key benchmark demonstrating CatList's "Reflection without Remorse" advantage.
	// Pattern: ((list ++ a) ++ b) ++ c ... (left-associated appends)
	// CatList: O(n) total, Vec: O(n²) total
	{
		let mut group = c.benchmark_group("CatList Left-Assoc Append");
		group.bench_with_input(BenchmarkId::new("CatList", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| CatList::singleton(0i32),
				|mut list| {
					for i in 1..s {
						// Left-associated: (list ++ singleton(i))
						list = list.append(CatList::singleton(i));
					}
					list
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("Vec", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| vec![0i32],
				|mut v| {
					for i in 1..s {
						// Vec extend is O(m) where m is the size of the appended data
						// But repeated left-associated appends lead to O(n²) total
						v.extend(vec![i]);
					}
					v
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("LinkedList", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| {
					let mut l = LinkedList::new();
					l.push_back(0i32);
					l
				},
				|mut l| {
					for i in 1..s {
						let mut other = LinkedList::new();
						other.push_back(i);
						l.append(&mut other);
					}
					l
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Iteration (Measures overhead of flattening the internal structure)
	// CatList iteration involves dynamic flattening, which is more expensive than Vec iteration.
	{
		let cat_list: CatList<i32> = (0..size).collect();
		let vec_list: Vec<i32> = (0..size).collect();
		let linked_list: LinkedList<i32> = (0..size).collect();

		let mut group = c.benchmark_group("CatList Iteration");
		group.bench_with_input(BenchmarkId::new("CatList", &input_desc), &size, |b, &_| {
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
		group.bench_with_input(BenchmarkId::new("Vec", &input_desc), &size, |b, &_| {
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
		group.bench_with_input(BenchmarkId::new("LinkedList", &input_desc), &size, |b, &_| {
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
		group.finish();
	}

	// Nested Uncons (Tests uncons on deeply nested structures created via left-associated appends)
	// This verifies that the flattening logic in uncons is efficient.
	{
		// Build a deeply nested CatList via left-associated appends
		let nested_cat_list: CatList<i32> = (0..size).fold(CatList::empty(), |acc, i| {
			if acc.is_empty() { CatList::singleton(i) } else { acc.append(CatList::singleton(i)) }
		});

		let mut group = c.benchmark_group("CatList Nested Uncons");
		group.bench_with_input(
			BenchmarkId::new("CatList (nested)", &input_desc),
			&size,
			|b, &_| {
				b.iter_batched(
					|| nested_cat_list.clone(),
					|mut l| {
						while let Some((_, tail)) = l.uncons() {
							l = tail;
						}
					},
					BatchSize::SmallInput,
				)
			},
		);
		// Compare with a flat CatList built via snoc (simpler structure)
		let flat_cat_list: CatList<i32> = (0..size).collect();
		group.bench_with_input(BenchmarkId::new("CatList (flat)", &input_desc), &size, |b, &_| {
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
		group.finish();
	}
}
