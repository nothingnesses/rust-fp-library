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
			IdentityBrand,
			ThunkBrand,
		},
		types::{
			ArcFree,
			ArcFreeExplicit,
			Free,
			FreeExplicit,
			Identity,
			RcFree,
			RcFreeExplicit,
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
					let mut program: RcFree<IdentityBrand, i32> = RcFree::pure(0);
					for _ in 0 .. k {
						program = RcFree::wrap(Identity(program));
					}
					program
				},
				|program| program.bind(|x: i32| RcFree::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("ArcFree", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: ArcFree<IdentityBrand, i32> = ArcFree::pure(0);
					for _ in 0 .. k {
						program = ArcFree::wrap(Identity(program));
					}
					program
				},
				|program| program.bind(|x: i32| ArcFree::pure(x + 1)).evaluate(),
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
					let mut program: RcFreeExplicit<'static, IdentityBrand, i32> =
						RcFreeExplicit::pure(0);
					for _ in 0 .. k {
						program = RcFreeExplicit::wrap(Identity(program));
					}
					program
				},
				|program| program.bind(|x: i32| RcFreeExplicit::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("ArcFreeExplicit", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: ArcFreeExplicit<'static, IdentityBrand, i32> =
						ArcFreeExplicit::pure(0);
					for _ in 0 .. k {
						program = ArcFreeExplicit::wrap(Identity(program));
					}
					program
				},
				|program| program.bind(|x: i32| ArcFreeExplicit::pure(x + 1)).evaluate(),
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
				let mut program: RcFree<IdentityBrand, i32> = RcFree::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| RcFree::pure(x + 1));
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("ArcFree", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: ArcFree<IdentityBrand, i32> = ArcFree::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| ArcFree::pure(x + 1));
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
				let mut program: RcFreeExplicit<'static, IdentityBrand, i32> =
					RcFreeExplicit::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| RcFreeExplicit::pure(x + 1));
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("ArcFreeExplicit", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: ArcFreeExplicit<'static, IdentityBrand, i32> =
					ArcFreeExplicit::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| ArcFreeExplicit::pure(x + 1));
				}
				program.evaluate()
			})
		});
	}

	group.finish();
}
