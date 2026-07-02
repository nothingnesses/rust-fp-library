// Criterion benches for the erased multi-shot thread-safe form
// `Free<F, A, ArcBrand>`. Three shapes: bind-deep, bind-wide, and
// peel-and-handle via the consuming `to_view`. The atomic refcount traffic
// is the expected delta vs the Rc store.

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
		},
		types::{
			Free,
			Identity,
		},
	},
};

fn build_spine(depth: usize) -> Free<IdentityBrand, i32, ArcBrand> {
	let mut program: Free<IdentityBrand, i32, ArcBrand> =
		Free::<IdentityBrand, i32, ArcBrand>::pure(0);
	for _ in 0 .. depth {
		program = Free::wrap(Identity(program));
	}
	program
}

pub fn bench_arc_free(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("Free<ArcBrand>");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind-deep + evaluate", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
				|program| {
					program
						.bind(|x: i32| Free::<IdentityBrand, i32, ArcBrand>::pure(x + 1))
						.evaluate()
				},
				BatchSize::SmallInput,
			)
		});
	}

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| {
				b.iter_batched(
					|| build_spine(k),
					Free::<IdentityBrand, i32, ArcBrand>::evaluate,
					BatchSize::SmallInput,
				)
			},
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
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
	}

	group.bench_function("peel-and-handle (Pure, to_view)", |b| {
		b.iter_batched(
			|| Free::<IdentityBrand, i32, ArcBrand>::pure(42),
			Free::<IdentityBrand, i32, ArcBrand>::to_view,
			BatchSize::SmallInput,
		)
	});

	group.finish();
}
