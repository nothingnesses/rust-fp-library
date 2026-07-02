// Criterion benches for the Explicit multi-shot single-thread variant
// `RcFreeExplicit<'a, F, A>`. Three shapes: bind-deep, bind-wide,
// peel-and-handle. Bind walks the spine recursively (O(N)) but Clone on the
// outer `Rc<Inner>` is O(1), so `peel_ref` is meaningful as a non-consuming
// counterpart to `to_view`.

use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::{
			IdentityBrand,
			RcBrand,
		},
		types::{
			FreeExplicit,
			Identity,
		},
	},
};

fn build_spine(depth: usize) -> FreeExplicit<'static, IdentityBrand, i32, RcBrand> {
	let mut program: FreeExplicit<'static, IdentityBrand, i32, RcBrand> =
		FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::pure(0);
	for _ in 0 .. depth {
		program = FreeExplicit::wrap(Identity(std::rc::Rc::new(program)));
	}
	program
}

pub fn bench_rc_free_explicit(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("FreeExplicit<RcBrand>");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind-deep + evaluate", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
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
	}

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| {
				b.iter_batched(
					|| build_spine(k),
					FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::evaluate,
					BatchSize::SmallInput,
				)
			},
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
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
	}

	group.bench_function("peel-and-handle (Pure, to_view)", |b| {
		b.iter_batched(
			|| FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::pure(42),
			FreeExplicit::<'static, IdentityBrand, i32, RcBrand>::to_view,
			BatchSize::SmallInput,
		)
	});

	group.finish();
}
