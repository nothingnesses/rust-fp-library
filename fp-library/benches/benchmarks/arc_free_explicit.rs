// Criterion benches for the Explicit multi-shot thread-safe variant
// `ArcFreeExplicit<'a, F, A>`. Three shapes: bind-deep, bind-wide,
// peel-and-handle. Bind walks the spine recursively (O(N)) and the
// continuation is `Arc<dyn Fn + Send + Sync>`; the outer `Arc<Inner>`
// makes Clone unconditionally O(1) (atomic refcount bump). The atomic
// increment is the expected delta vs `RcFreeExplicit`.

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

pub fn bench_arc_free_explicit(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("ArcFreeExplicit");

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
