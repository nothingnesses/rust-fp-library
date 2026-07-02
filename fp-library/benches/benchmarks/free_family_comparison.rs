// Cross-variant comparison bench for the six-variant Free family. Documents
// the O(1) (Erased family: `Free`, `RcFree`, `ArcFree`) vs O(N) (Explicit
// family: `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) bind-cost
// asymmetry under a single `BenchmarkGroup`, so the criterion output shows
// the six variants side by side at each depth. The two shapes covered here
// are the ones where the asymmetry is qualitatively different: bind-deep
// (the Explicit family walks the spine inside `bind`; the Erased family
// only snocs onto the CatList) and bind-wide (chained binds over `Pure`).
// Per-variant benches in the sibling files cover the rest of the surface.

use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::{
			ArcBrand,
			IdentityBrand,
			RcBrand,
			ThunkBrand,
		},
		types::{
			Free,
			FreeExplicit,
			Identity,
			Thunk,
		},
	},
};

pub fn bench_free_family_comparison(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	// -- bind-deep + evaluate --

	let mut group = c.benchmark_group("FreeFamily/bind-deep + evaluate");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("Free", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: Free<ThunkBrand, i32> = Free::pure(0);
					for _ in 0 .. k {
						program = Free::wrap(Thunk::new(move || program));
					}
					program
				},
				|program| program.bind(|x| Free::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("RcFree", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: Free<IdentityBrand, i32, RcBrand> =
						Free::<IdentityBrand, i32, RcBrand>::pure(0);
					for _ in 0 .. k {
						program = Free::wrap(Identity(program));
					}
					program
				},
				|program| {
					program
						.bind(|x: i32| Free::<IdentityBrand, i32, RcBrand>::pure(x + 1))
						.evaluate()
				},
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("ArcFree", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: Free<IdentityBrand, i32, ArcBrand> =
						Free::<IdentityBrand, i32, ArcBrand>::pure(0);
					for _ in 0 .. k {
						program = Free::wrap(Identity(program));
					}
					program
				},
				|program| {
					program
						.bind(|x: i32| Free::<IdentityBrand, i32, ArcBrand>::pure(x + 1))
						.evaluate()
				},
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("FreeExplicit", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: FreeExplicit<'static, IdentityBrand, i32> =
						FreeExplicit::pure(0);
					for _ in 0 .. k {
						program = FreeExplicit::wrap(Identity(Box::new(program)));
					}
					program
				},
				|program| program.bind(|x| FreeExplicit::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("RcFreeExplicit", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: FreeExplicit<'static, IdentityBrand, i32, RcBrand> =
						FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::pure(0);
					for _ in 0 .. k {
						program = FreeExplicit::wrap(Identity(std::rc::Rc::new(program)));
					}
					program
				},
				|program| {
					program
						.bind(|x: i32| {
							FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::pure(x + 1)
						})
						.evaluate()
				},
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("ArcFreeExplicit", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: FreeExplicit<'static, IdentityBrand, i32, ArcBrand> =
						FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::pure(0);
					for _ in 0 .. k {
						program = FreeExplicit::wrap(Identity(std::sync::Arc::new(program)));
					}
					program
				},
				|program| {
					program
						.bind(|x: i32| {
							FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::pure(x + 1)
						})
						.evaluate()
				},
				BatchSize::SmallInput,
			)
		});
	}

	group.finish();

	// -- bind-wide + evaluate --

	let mut group = c.benchmark_group("FreeFamily/bind-wide + evaluate");

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("Free", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: Free<ThunkBrand, i32> = Free::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x| Free::pure(x + 1));
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("RcFree", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: Free<IdentityBrand, i32, RcBrand> =
					Free::<IdentityBrand, i32, RcBrand>::pure(0);
				for _ in 0 .. k {
					program =
						program.bind(|x: i32| Free::<IdentityBrand, i32, RcBrand>::pure(x + 1));
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("ArcFree", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: Free<IdentityBrand, i32, ArcBrand> =
					Free::<IdentityBrand, i32, ArcBrand>::pure(0);
				for _ in 0 .. k {
					program =
						program.bind(|x: i32| Free::<IdentityBrand, i32, ArcBrand>::pure(x + 1));
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("FreeExplicit", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: FreeExplicit<'static, IdentityBrand, i32> = FreeExplicit::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| FreeExplicit::pure(x + 1));
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("RcFreeExplicit", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: FreeExplicit<'static, IdentityBrand, i32, RcBrand> =
					FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| {
						FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::pure(x + 1)
					});
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("ArcFreeExplicit", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: FreeExplicit<'static, IdentityBrand, i32, ArcBrand> =
					FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| {
						FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::pure(x + 1)
					});
				}
				program.evaluate()
			})
		});
	}

	group.finish();
}
