use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::types::cat_list::CatList;
use std::collections::LinkedList;

/// Benchmarks for CatList operations.
///
/// Compares CatList performance against Vec and LinkedList for common operations.
///
/// Key performance characteristics tested:
/// - Cons (Prepend): Should be O(1), comparable to LinkedList, faster than Vec::insert(0).
/// - Snoc (Append element): Should be O(1), comparable to Vec::push.
/// - Append (Concatenation): Should be O(1), significantly faster than Vec::append (O(n)).
/// - Uncons (Head/Tail): Should be amortized O(1), comparable to LinkedList::pop_front.
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
					while let Some(_) = l.pop_front() {}
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}
}
