// Criterion benches for the Explicit single-shot single-thread variant
// `FreeExplicit<'a, F, A>`. Three shapes: bind-deep (build a `Wrap` spine,
// then bind once and evaluate), bind-wide (chained binds over `Pure`),
// peel-and-handle (single-step `evaluate` on a `Pure` value via the
// public consuming API). `bind-deep` surfaces the O(N) cost of naive
// recursive `bind` walking an existing spine.

use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::IdentityBrand,
		types::{
			FreeExplicit,
			Identity,
		},
	},
};

fn build_spine(depth: usize) -> FreeExplicit<'static, IdentityBrand, i32> {
	let mut program: FreeExplicit<'static, IdentityBrand, i32> = FreeExplicit::pure(0);
	for _ in 0 .. depth {
		program = FreeExplicit::wrap(Identity(Box::new(program)));
	}
	program
}

pub fn bench_free_explicit(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("FreeExplicit");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind-deep + evaluate", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
				|program| program.bind(|x| FreeExplicit::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});
	}

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| {
				b.iter_batched(|| build_spine(k), FreeExplicit::evaluate, BatchSize::SmallInput)
			},
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: FreeExplicit<'static, IdentityBrand, i32> = FreeExplicit::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| FreeExplicit::pure(x + 1));
				}
				program.evaluate()
			})
		});
	}

	group.bench_function("peel-and-handle (Pure, evaluate)", |b| {
		b.iter_batched(
			|| FreeExplicit::<'static, IdentityBrand, i32>::pure(42),
			FreeExplicit::evaluate,
			BatchSize::SmallInput,
		)
	});

	group.finish();
}
