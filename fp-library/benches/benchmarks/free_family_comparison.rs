// Cross-store comparison bench for the Free family. Documents the O(1)
// (erased `Free` at the Box/Rc/Arc stores) vs O(N) (concrete `FreeExplicit`
// at the same stores) bind-cost asymmetry under a single `BenchmarkGroup`,
// so the criterion output shows the six forms side by side at each depth.
// The two shapes covered here are the ones where the asymmetry is
// qualitatively different: bind-deep (the concrete family walks the spine
// inside `bind`; the erased family only snocs onto the CatList) and
// bind-wide (chained binds over `Pure`). Per-form benches in the sibling
// files cover the rest of the surface. The erased forms use the `ThunkBrand`
// spine (`Identity` provides no per-layer indirection, so the erased-store
// `Free` over it is layout-cyclic); the concrete forms keep `Identity` spines
// with an explicit pointer per layer.

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

		group.bench_with_input(BenchmarkId::new("Free<RcBrand>", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: Free<ThunkBrand, i32, RcBrand> =
						Free::<ThunkBrand, i32, RcBrand>::pure(0);
					for _ in 0 .. k {
						program = Free::wrap(Thunk::new(move || program));
					}
					program
				},
				|program| {
					program.bind(|x: i32| Free::<ThunkBrand, i32, RcBrand>::pure(x + 1)).evaluate()
				},
				BatchSize::SmallInput,
			)
		});

		group.bench_with_input(BenchmarkId::new("Free<ArcBrand>", depth), &depth, |b, &k| {
			b.iter_batched(
				|| {
					let mut program: Free<ThunkBrand, i32, ArcBrand> =
						Free::<ThunkBrand, i32, ArcBrand>::pure(0);
					for _ in 0 .. k {
						program = Free::wrap(Thunk::new(move || program));
					}
					program
				},
				|program| {
					program.bind(|x: i32| Free::<ThunkBrand, i32, ArcBrand>::pure(x + 1)).evaluate()
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

		group.bench_with_input(
			BenchmarkId::new("FreeExplicit<RcBrand>", depth),
			&depth,
			|b, &k| {
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
			},
		);

		group.bench_with_input(
			BenchmarkId::new("FreeExplicit<ArcBrand>", depth),
			&depth,
			|b, &k| {
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
			},
		);
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

		group.bench_with_input(BenchmarkId::new("Free<RcBrand>", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: Free<ThunkBrand, i32, RcBrand> =
					Free::<ThunkBrand, i32, RcBrand>::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| Free::<ThunkBrand, i32, RcBrand>::pure(x + 1));
				}
				program.evaluate()
			})
		});

		group.bench_with_input(BenchmarkId::new("Free<ArcBrand>", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: Free<ThunkBrand, i32, ArcBrand> =
					Free::<ThunkBrand, i32, ArcBrand>::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| Free::<ThunkBrand, i32, ArcBrand>::pure(x + 1));
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

		group.bench_with_input(
			BenchmarkId::new("FreeExplicit<RcBrand>", width),
			&width,
			|b, &k| {
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
			},
		);

		group.bench_with_input(
			BenchmarkId::new("FreeExplicit<ArcBrand>", width),
			&width,
			|b, &k| {
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
			},
		);
	}

	group.finish();
}
