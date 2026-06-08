// Criterion benches for the Erased single-thread variant `Free<F, A>`. Three
// shapes: bind-deep (build a `Wrap` spine, then bind once and evaluate),
// bind-wide (chained binds over `Pure`), peel-and-handle (single-step
// `to_view`). `bind-deep` surfaces the O(1) bind cost on `Free` (the spine
// walk happens during `evaluate`, not during `bind`); the same shape over
// `FreeExplicit` walks the spine inside `bind` itself. `Free<IdentityBrand, _>`
// is layout-cyclic because `Identity<T>` provides no indirection, so the
// bench picks `ThunkBrand` (whose `Thunk<A>` holds a boxed closure)
// instead.

use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::ThunkBrand,
		types::{
			Free,
			Thunk,
		},
	},
};

fn build_spine(depth: usize) -> Free<ThunkBrand, i32> {
	let mut program: Free<ThunkBrand, i32> = Free::pure(0);
	for _ in 0 .. depth {
		program = Free::wrap(Thunk::new(move || program));
	}
	program
}

pub fn bench_free(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("Free");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind-deep + evaluate", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
				|program| program.bind(|x| Free::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});
	}

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| b.iter_batched(|| build_spine(k), Free::evaluate, BatchSize::SmallInput),
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: Free<ThunkBrand, i32> = Free::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x| Free::pure(x + 1));
				}
				program.evaluate()
			})
		});
	}

	group.bench_function("peel-and-handle (Pure, to_view)", |b| {
		b.iter_batched(|| Free::<ThunkBrand, i32>::pure(42), Free::to_view, BatchSize::SmallInput)
	});

	group.finish();
}
