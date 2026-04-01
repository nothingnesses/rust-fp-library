use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::VecBrand,
		classes::functor::map,
		types::{
			ArcCoyoneda,
			Coyoneda,
			CoyonedaExplicit,
			RcCoyoneda,
		},
	},
};

pub fn bench_coyoneda(c: &mut Criterion) {
	let size = 1000;
	let v_orig: Vec<i32> = (0 .. size).collect();
	let depths: &[usize] = &[1, 10, 100];

	let mut group = c.benchmark_group("Coyoneda");

	for &depth in depths {
		// Direct: chain k calls to map::<VecBrand, _, _>(|x: i32| x + 1, v).
		group.bench_with_input(BenchmarkId::new("Direct", depth), &depth, |b, &k| {
			b.iter_batched(
				|| v_orig.clone(),
				|v| {
					let mut result = v;
					for _ in 0 .. k {
						result = map::<VecBrand, _, _>(|x: i32| x + 1, result);
					}
					result
				},
				BatchSize::SmallInput,
			)
		});

		// Coyoneda: lift, chain k .map calls, then .lower().
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

		// CoyonedaExplicit: lift, chain k .map calls, then .lower().
		group.bench_with_input(BenchmarkId::new("CoyonedaExplicit", depth), &depth, |b, &k| {
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
		});

		// RcCoyoneda: lift, chain k .map calls, then .lower_ref().
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

		// ArcCoyoneda: lift, chain k .map calls, then .lower_ref().
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

		// RcCoyoneda repeated lower_ref: measures re-evaluation cost.
		group.bench_with_input(BenchmarkId::new("RcCoyoneda_3x_lower", depth), &depth, |b, &k| {
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

		// ArcCoyoneda repeated lower_ref: measures re-evaluation cost.
		group.bench_with_input(BenchmarkId::new("ArcCoyoneda_3x_lower", depth), &depth, |b, &k| {
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

		// RcCoyoneda clone + map + lower_ref pattern.
		group.bench_with_input(BenchmarkId::new("RcCoyoneda_clone_map", depth), &depth, |b, &k| {
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

		// ArcCoyoneda clone + map + lower_ref pattern.
		group.bench_with_input(
			BenchmarkId::new("ArcCoyoneda_clone_map", depth),
			&depth,
			|b, &k| {
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
			},
		);
	}

	group.finish();
}
