use {
	core::ops::ControlFlow,
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::ThunkBrand,
		types::{
			ArcLazy,
			ArcLazyConfig,
			Free,
			Lazy,
			RcLazyConfig,
			Thunk,
			Trampoline,
		},
	},
	std::hint::black_box,
};

pub fn bench_lazy(c: &mut Criterion) {
	// ── Thunk ────────────────────────────────────────────────────────────

	// Thunk: new + evaluate (baseline)
	{
		let mut group = c.benchmark_group("Thunk Baseline");
		group.bench_function("new+evaluate", |b| {
			b.iter(|| {
				let thunk = Thunk::new(|| black_box(42));
				thunk.evaluate()
			})
		});
		group.finish();
	}

	// Thunk: map chains
	{
		let mut group = c.benchmark_group("Thunk Map Chain");
		for &depth in &[1, 10, 100] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter(|| {
					let mut thunk = Thunk::new(|| 0i64);
					for _ in 0 .. d {
						thunk = thunk.map(|x| x + 1);
					}
					thunk.evaluate()
				})
			});
		}
		group.finish();
	}

	// Thunk: bind chains
	{
		let mut group = c.benchmark_group("Thunk Bind Chain");
		for &depth in &[1, 10, 100] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter(|| {
					let mut thunk = Thunk::new(|| 0i64);
					for _ in 0 .. d {
						thunk = thunk.bind(|x| Thunk::pure(x + 1));
					}
					thunk.evaluate()
				})
			});
		}
		group.finish();
	}

	// ── Trampoline ───────────────────────────────────────────────────────

	// Trampoline: new + evaluate (baseline)
	{
		let mut group = c.benchmark_group("Trampoline Baseline");
		group.bench_function("new+evaluate", |b| {
			b.iter(|| {
				let task = Trampoline::new(|| black_box(42));
				task.evaluate()
			})
		});
		group.finish();
	}

	// Trampoline: bind chains
	{
		let mut group = c.benchmark_group("Trampoline Bind Chain");
		for &depth in &[100, 1000, 10000] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter(|| {
					let mut task = Trampoline::pure(0i64);
					for _ in 0 .. d {
						task = task.bind(|x| Trampoline::pure(x + 1));
					}
					task.evaluate()
				})
			});
		}
		group.finish();
	}

	// Trampoline: map chains
	{
		let mut group = c.benchmark_group("Trampoline Map Chain");
		for &depth in &[100, 1000, 10000] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter(|| {
					let mut task = Trampoline::pure(0i64);
					for _ in 0 .. d {
						task = task.map(|x| x + 1);
					}
					task.evaluate()
				})
			});
		}
		group.finish();
	}

	// Trampoline: tail_rec_m deep recursion (countdown from 10000)
	{
		let mut group = c.benchmark_group("Trampoline tail_rec_m");
		let target = 10000u64;
		group.bench_with_input(BenchmarkId::new("countdown", target), &target, |b, &n| {
			b.iter(|| {
				Trampoline::tail_rec_m(
					|state: u64| {
						if state == 0 {
							Trampoline::pure(ControlFlow::Break(0u64))
						} else {
							Trampoline::pure(ControlFlow::Continue(state - 1))
						}
					},
					n,
				)
				.evaluate()
			})
		});
		group.finish();
	}

	// Trampoline vs hand-written iterative loop
	{
		let depth = 10000u64;
		let mut group = c.benchmark_group("Trampoline vs Iterative");
		group.bench_with_input(BenchmarkId::new("trampoline", depth), &depth, |b, &n| {
			b.iter(|| {
				Trampoline::tail_rec_m(
					|state: u64| {
						if state == 0 {
							Trampoline::pure(ControlFlow::Break(0u64))
						} else {
							Trampoline::pure(ControlFlow::Continue(state - 1))
						}
					},
					n,
				)
				.evaluate()
			})
		});
		group.bench_with_input(BenchmarkId::new("iterative", depth), &depth, |b, &n| {
			b.iter(|| {
				let mut state = n;
				while state > 0 {
					state -= 1;
				}
				black_box(state)
			})
		});
		group.finish();
	}

	// ── Lazy (RcLazy) ────────────────────────────────────────────────────

	// RcLazy: first-access time
	{
		let mut group = c.benchmark_group("RcLazy First Access");
		group.bench_function("evaluate_fresh", |b| {
			b.iter_batched(
				|| Lazy::<_, RcLazyConfig>::new(|| black_box(42)),
				|lazy| *lazy.evaluate(),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// RcLazy: cached-access time
	{
		let mut group = c.benchmark_group("RcLazy Cached Access");
		group.bench_function("evaluate_cached", |b| {
			let lazy = Lazy::<_, RcLazyConfig>::new(|| 42);
			// Force first evaluation so the value is cached.
			let _ = lazy.evaluate();
			b.iter(|| *lazy.evaluate())
		});
		group.finish();
	}

	// RcLazy: ref_map chains
	{
		let mut group = c.benchmark_group("RcLazy ref_map Chain");
		for &depth in &[1, 10, 100] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter_batched(
					|| {
						let mut lazy = Lazy::<_, RcLazyConfig>::new(|| 0i64);
						for _ in 0 .. d {
							lazy = lazy.ref_map(|x| *x + 1);
						}
						lazy
					},
					|lazy| *lazy.evaluate(),
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// ── Lazy (ArcLazy) ──────────────────────────────────────────────────

	// ArcLazy: first-access time
	{
		let mut group = c.benchmark_group("ArcLazy First Access");
		group.bench_function("evaluate_fresh", |b| {
			b.iter_batched(
				|| ArcLazy::new(|| black_box(42)),
				|lazy| *lazy.evaluate(),
				BatchSize::SmallInput,
			)
		});
		group.finish();
	}

	// ArcLazy: cached-access time
	{
		let mut group = c.benchmark_group("ArcLazy Cached Access");
		group.bench_function("evaluate_cached", |b| {
			let lazy = ArcLazy::new(|| 42);
			let _ = lazy.evaluate();
			b.iter(|| *lazy.evaluate())
		});
		group.finish();
	}

	// ArcLazy: ref_map chains
	{
		let mut group = c.benchmark_group("ArcLazy ref_map Chain");
		for &depth in &[1, 10, 100] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter_batched(
					|| {
						let mut lazy: Lazy<'_, i64, ArcLazyConfig> = ArcLazy::new(|| 0i64);
						for _ in 0 .. d {
							lazy = lazy.ref_map(|x| *x + 1);
						}
						lazy
					},
					|lazy| *lazy.evaluate(),
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// ── Free ─────────────────────────────────────────────────────────────

	// Free: left-associated bind chains
	{
		let mut group = c.benchmark_group("Free Left-Assoc Bind");
		for &depth in &[100, 1000, 10000] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter(|| {
					let mut free = Free::<ThunkBrand, _>::pure(0i64);
					for _ in 0 .. d {
						free = free.bind(|x| Free::pure(x + 1));
					}
					free.evaluate()
				})
			});
		}
		group.finish();
	}

	// Free: right-associated bind chains
	{
		let mut group = c.benchmark_group("Free Right-Assoc Bind");
		for &depth in &[100, 1000, 10000] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter(|| {
					fn build_right(n: i64) -> Free<ThunkBrand, i64> {
						if n == 0 {
							Free::pure(0)
						} else {
							Free::pure(n)
								.bind(move |x| build_right(x - 1).bind(|y| Free::pure(y + 1)))
						}
					}
					build_right(d as i64).evaluate()
				})
			});
		}
		group.finish();
	}

	// Free: evaluate for various depths (pure + wrap)
	{
		let mut group = c.benchmark_group("Free Evaluate");
		for &depth in &[100, 1000, 10000] {
			group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &d| {
				b.iter(|| {
					let mut free = Free::<ThunkBrand, _>::pure(0i64);
					for _ in 0 .. d {
						free = Free::wrap(Thunk::new(move || free.bind(|x| Free::pure(x + 1))));
					}
					free.evaluate()
				})
			});
		}
		group.finish();
	}
}
