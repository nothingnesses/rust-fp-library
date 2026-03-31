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
			Coyoneda,
			CoyonedaExplicit,
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
					let mut coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(v);
					for _ in 0 .. k {
						coyo = coyo.map(|x: i32| x + 1);
					}
					coyo.lower()
				},
				BatchSize::SmallInput,
			)
		});
	}

	group.finish();
}
