use criterion::{BatchSize, BenchmarkId, Criterion};
use fp_library::types::cat_queue::CatQueue;
use std::collections::VecDeque;

/// Benchmarks for CatQueue operations.
///
/// Compares CatQueue performance against VecDeque for common operations.
///
/// Key performance characteristics tested:
/// - Snoc (Enqueue): Should be O(1) amortized, comparable to VecDeque::push_back.
/// - Uncons (Dequeue): Should be O(1) amortized, comparable to VecDeque::pop_front.
pub fn bench_cat_queue(c: &mut Criterion) {
	let size = 1000;
	let input_desc = format!("Size {}", size);

	// Snoc (Enqueue)
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

	// Uncons (Dequeue)
	{
		let q: CatQueue<i32> = (0..size)
			.map(|i| i)
			.collect::<Vec<_>>()
			.into_iter()
			.fold(CatQueue::empty(), |q, i| q.snoc(i));
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
					while let Some(_) = q.pop_front() {}
				},
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}
}
