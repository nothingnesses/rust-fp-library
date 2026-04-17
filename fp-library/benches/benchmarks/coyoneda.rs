use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::VecBrand,
		functions::explicit::map,
		types::{
			ArcCoyoneda,
			Coyoneda,
			CoyonedaExplicit,
			RcCoyoneda,
		},
	},
};

pub fn bench_coyoneda(c: &mut Criterion) {
	let v_orig: Vec<i32> = (0 .. 1000).collect();
	let depths: &[usize] = &[1, 5, 10, 25, 50, 100];

	// Core comparison: Direct vs all Coyoneda variants across map depths.
	{
		let mut group = c.benchmark_group("Coyoneda");
		for &depth in depths {
			group.bench_with_input(
				BenchmarkId::new("Vec map (no Coyoneda)", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| v_orig.clone(),
						|v| {
							let mut result = v;
							for _ in 0 .. k {
								result = map::<VecBrand, _, _, _, _>(|x: i32| x + 1, result);
							}
							result
						},
						BatchSize::SmallInput,
					)
				},
			);

			group.bench_with_input(BenchmarkId::new("Coyoneda", depth), &depth, |b, &k| {
				b.iter_batched(
					|| v_orig.clone(),
					|v| {
						let mut coyo = Coyoneda::<VecBrand, _>::lift(v);
						for _ in 0 .. k {
							coyo = coyo.map(|x: i32| x + 1);
						}
						coyo.lower()
					},
					BatchSize::SmallInput,
				)
			});

			group.bench_with_input(
				BenchmarkId::new("CoyonedaExplicit (boxed)", depth),
				&depth,
				|b, &k| {
					b.iter_batched(
						|| v_orig.clone(),
						|v| {
							let mut coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(v).boxed();
							for _ in 0 .. k {
								coyo = coyo.map(|x: i32| x + 1).boxed();
							}
							coyo.lower()
						},
						BatchSize::SmallInput,
					)
				},
			);

			group.bench_with_input(BenchmarkId::new("RcCoyoneda", depth), &depth, |b, &k| {
				b.iter_batched(
					|| v_orig.clone(),
					|v| {
						let mut coyo = RcCoyoneda::<VecBrand, _>::lift(v);
						for _ in 0 .. k {
							coyo = coyo.map(|x: i32| x + 1);
						}
						coyo.lower_ref()
					},
					BatchSize::SmallInput,
				)
			});

			group.bench_with_input(BenchmarkId::new("ArcCoyoneda", depth), &depth, |b, &k| {
				b.iter_batched(
					|| v_orig.clone(),
					|v| {
						let mut coyo = ArcCoyoneda::<VecBrand, _>::lift(v);
						for _ in 0 .. k {
							coyo = coyo.map(|x: i32| x + 1);
						}
						coyo.lower_ref()
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Repeated lower_ref: measures re-evaluation cost for Rc/Arc variants.
	{
		let mut group = c.benchmark_group("Coyoneda Repeated Lower");
		for &depth in depths {
			group.bench_with_input(BenchmarkId::new("RcCoyoneda", depth), &depth, |b, &k| {
				b.iter_batched(
					|| {
						let mut coyo = RcCoyoneda::<VecBrand, _>::lift(v_orig.clone());
						for _ in 0 .. k {
							coyo = coyo.map(|x: i32| x + 1);
						}
						coyo
					},
					|coyo| {
						let _ = coyo.lower_ref();
						let _ = coyo.lower_ref();
						coyo.lower_ref()
					},
					BatchSize::SmallInput,
				)
			});

			group.bench_with_input(BenchmarkId::new("ArcCoyoneda", depth), &depth, |b, &k| {
				b.iter_batched(
					|| {
						let mut coyo = ArcCoyoneda::<VecBrand, _>::lift(v_orig.clone());
						for _ in 0 .. k {
							coyo = coyo.map(|x: i32| x + 1);
						}
						coyo
					},
					|coyo| {
						let _ = coyo.lower_ref();
						let _ = coyo.lower_ref();
						coyo.lower_ref()
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Clone + map + lower_ref pattern for Rc/Arc variants.
	{
		let mut group = c.benchmark_group("Coyoneda Clone Map");
		for &depth in depths {
			group.bench_with_input(BenchmarkId::new("RcCoyoneda", depth), &depth, |b, &k| {
				b.iter_batched(
					|| {
						let mut coyo = RcCoyoneda::<VecBrand, _>::lift(v_orig.clone());
						for _ in 0 .. k {
							coyo = coyo.map(|x: i32| x + 1);
						}
						coyo
					},
					|coyo| {
						let cloned = coyo.clone();
						cloned.map(|x: i32| x * 2).lower_ref()
					},
					BatchSize::SmallInput,
				)
			});

			group.bench_with_input(BenchmarkId::new("ArcCoyoneda", depth), &depth, |b, &k| {
				b.iter_batched(
					|| {
						let mut coyo = ArcCoyoneda::<VecBrand, _>::lift(v_orig.clone());
						for _ in 0 .. k {
							coyo = coyo.map(|x: i32| x + 1);
						}
						coyo
					},
					|coyo| {
						let cloned = coyo.clone();
						cloned.map(|x: i32| x * 2).lower_ref()
					},
					BatchSize::SmallInput,
				)
			});
		}
		group.finish();
	}

	// Map fusion: CoyonedaExplicit without boxing vs Direct.
	// CoyonedaExplicit composes functions at the type level, so k maps result
	// in a single call to F::map at lower time. Without boxing, the type grows
	// with each map, so we use fixed depths via macro.
	{
		let mut group = c.benchmark_group("Coyoneda Fusion");

		macro_rules! bench_fusion {
			($depth:tt) => {
				group.bench_with_input(
					BenchmarkId::new("CoyonedaExplicit (fused)", $depth),
					&$depth,
					|b, &_| {
						b.iter_batched(
							|| v_orig.clone(),
							|v| {
								let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(v);
								seq_apply!(coyo, $depth).lower()
							},
							BatchSize::SmallInput,
						)
					},
				);
				group.bench_with_input(
					BenchmarkId::new("Vec map (no Coyoneda)", $depth),
					&$depth,
					|b, &_| {
						b.iter_batched(
							|| v_orig.clone(),
							|v| {
								let mut result = v;
								for _ in 0 .. $depth {
									result = map::<VecBrand, _, _, _, _>(|x: i32| x + 1, result);
								}
								result
							},
							BatchSize::SmallInput,
						)
					},
				);
				group.bench_with_input(
					BenchmarkId::new("CoyonedaExplicit (boxed)", $depth),
					&$depth,
					|b, &_| {
						b.iter_batched(
							|| v_orig.clone(),
							|v| {
								let mut coyo =
									CoyonedaExplicit::<VecBrand, _, _, _>::lift(v).boxed();
								for _ in 0 .. $depth {
									coyo = coyo.map(|x: i32| x + 1).boxed();
								}
								coyo.lower()
							},
							BatchSize::SmallInput,
						)
					},
				);
			};
		}

		// Apply .map(|x| x + 1) N times without boxing, producing a nested closure type.
		macro_rules! seq_apply {
			($coyo:expr, 1) => {
				$coyo.map(|x: i32| x + 1)
			};
			($coyo:expr, 5) => {
				$coyo
					.map(|x: i32| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
			};
			($coyo:expr, 10) => {
				seq_apply!($coyo, 5)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
			};
			($coyo:expr, 25) => {
				seq_apply!($coyo, 10)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
					.map(|x| x + 1)
			};
		}

		bench_fusion!(1);
		bench_fusion!(5);
		bench_fusion!(10);
		bench_fusion!(25);

		group.finish();
	}
}
