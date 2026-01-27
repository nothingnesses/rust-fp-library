use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::types::cat_queue::CatQueue;
use std::collections::VecDeque;

/// Benchmarks for CatQueue operations.
///
/// Compares CatQueue performance against VecDeque for common operations.
///
/// Key performance characteristics tested:
/// - Snoc (Enqueue): Should be O(1) amortized, comparable to VecDeque::push_back.
/// - Cons (Prepend): Should be O(1) amortized, comparable to VecDeque::push_front.
/// - Uncons (Dequeue front): Should be O(1) amortized, comparable to VecDeque::pop_front.
/// - Unsnoc (Dequeue back): Should be O(1) amortized, comparable to VecDeque::pop_back.
/// - Sliding Window: Tests interleaved snoc/uncons operations for amortization stability.
pub fn bench_cat_queue(c: &mut Criterion) {
	let size = 1000;
	let input_desc = format!("Size {}", size);

	// Snoc (Enqueue to back)
	{
		let mut group = c.benchmark_group("CatQueue Snoc");
		group.bench_with_input(BenchmarkId::new("CatQueue", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| CatQueue::empty(),
				|mut q| {
					for i in 0..s {
						q = q.snoc(i);
					}
					q
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("VecDeque", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| VecDeque::new(),
				|mut q| {
					for i in 0..s {
						q.push_back(i);
					}
					q
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Cons (Prepend to front)
	{
		let mut group = c.benchmark_group("CatQueue Cons");
		group.bench_with_input(BenchmarkId::new("CatQueue", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| CatQueue::empty(),
				|mut q| {
					for i in 0..s {
						q = q.cons(i);
					}
					q
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("VecDeque", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| VecDeque::new(),
				|mut q| {
					for i in 0..s {
						q.push_front(i);
					}
					q
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Uncons (Dequeue from front)
	{
		let q: CatQueue<i32> = (0..size).fold(CatQueue::empty(), |q, i| q.snoc(i));
		let vd: VecDeque<i32> = (0..size).collect();

		let mut group = c.benchmark_group("CatQueue Uncons");
		group.bench_with_input(BenchmarkId::new("CatQueue", &input_desc), &size, |b, &_| {
			b.iter_batched(
				|| q.clone(),
				|mut q| {
					while let Some((_, tail)) = q.uncons() {
						q = tail;
					}
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("VecDeque", &input_desc), &size, |b, &_| {
			b.iter_batched(
				|| vd.clone(),
				|mut q| {
					while q.pop_front().is_some() {}
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Unsnoc (Dequeue from back)
	{
		let q: CatQueue<i32> = (0..size).fold(CatQueue::empty(), |q, i| q.snoc(i));
		let vd: VecDeque<i32> = (0..size).collect();

		let mut group = c.benchmark_group("CatQueue Unsnoc");
		group.bench_with_input(BenchmarkId::new("CatQueue", &input_desc), &size, |b, &_| {
			b.iter_batched(
				|| q.clone(),
				|mut q| {
					while let Some((_, tail)) = q.unsnoc() {
						q = tail;
					}
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("VecDeque", &input_desc), &size, |b, &_| {
			b.iter_batched(
				|| vd.clone(),
				|mut q| {
					while q.pop_back().is_some() {}
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// Sliding Window (Mixed operations: tests amortization stability)
	// Pre-fill with 100 items, then snoc 1 / uncons 1 for 'size' iterations
	{
		let prefill = 100;
		let q_prefilled: CatQueue<i32> = (0..prefill).fold(CatQueue::empty(), |q, i| q.snoc(i));
		let vd_prefilled: VecDeque<i32> = (0..prefill).collect();

		let mut group = c.benchmark_group("CatQueue Sliding Window");
		group.bench_with_input(BenchmarkId::new("CatQueue", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| q_prefilled.clone(),
				|mut q| {
					for i in 0..s {
						q = q.snoc(i);
						// Safe to unwrap: we just added an element, so queue is never empty
						let (_, tail) = q.uncons().unwrap();
						q = tail;
					}
					q
				},
				BatchSize::SmallInput,
			)
		});
		group.bench_with_input(BenchmarkId::new("VecDeque", &input_desc), &size, |b, &s| {
			b.iter_batched(
				|| vd_prefilled.clone(),
				|mut q| {
					for i in 0..s {
						q.push_back(i);
						q.pop_front();
					}
					q
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}
}
