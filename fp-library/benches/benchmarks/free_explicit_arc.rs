// Criterion benches for the concrete multi-shot thread-safe form
// `FreeExplicit<'a, F, A, ArcBrand>`. Three shapes: bind-deep, bind-wide,
// and peel-and-handle via `to_view`. Bind walks the spine recursively
// (O(N)) with `Arc<dyn Fn + Send + Sync>` continuations; the atomic
// refcount traffic is the expected delta vs the Rc store.

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
			FreeExplicit,
			Identity,
		},
	},
};

fn build_spine(depth: usize) -> FreeExplicit<'static, IdentityBrand, i32, ArcBrand> {
	let mut program: FreeExplicit<'static, IdentityBrand, i32, ArcBrand> =
		FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::pure(0);
	for _ in 0 .. depth {
		program = FreeExplicit::wrap(Identity(std::sync::Arc::new(program)));
	}
	program
}

pub fn bench_free_explicit_arc(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("FreeExplicit<ArcBrand>");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind-deep + evaluate", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
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

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| {
				b.iter_batched(
					|| build_spine(k),
					FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::evaluate,
					BatchSize::SmallInput,
				)
			},
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
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

	group.bench_function("peel-and-handle (Pure, to_view)", |b| {
		b.iter_batched(
			|| FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::pure(42),
			FreeExplicit::<'static, IdentityBrand, i32, ArcBrand>::to_view,
			BatchSize::SmallInput,
		)
	});

	group.finish();
}
