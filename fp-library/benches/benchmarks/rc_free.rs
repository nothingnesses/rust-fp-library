// Criterion benches for the Erased multi-shot single-thread variant
// `RcFree<F, A>`. Three shapes: bind-deep, bind-wide, peel-and-handle. The
// outer `Rc<Inner>` wrapping makes Clone unconditionally O(1), so the
// non-consuming `peel_ref` is included alongside `to_view`.

use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		brands::IdentityBrand,
		types::{
			Identity,
			RcFree,
		},
	},
};

fn build_spine(depth: usize) -> RcFree<IdentityBrand, i32> {
	let mut program: RcFree<IdentityBrand, i32> = RcFree::pure(0);
	for _ in 0 .. depth {
		program = RcFree::wrap(Identity(program));
	}
	program
}

pub fn bench_rc_free(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("RcFree");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind-deep + evaluate", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
				|program| program.bind(|x: i32| RcFree::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});
	}

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| b.iter_batched(|| build_spine(k), RcFree::evaluate, BatchSize::SmallInput),
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: RcFree<IdentityBrand, i32> = RcFree::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| RcFree::pure(x + 1));
				}
				program.evaluate()
			})
		});
	}

	group.bench_function("peel-and-handle (Pure, to_view)", |b| {
		b.iter_batched(
			|| RcFree::<IdentityBrand, i32>::pure(42),
			RcFree::to_view,
			BatchSize::SmallInput,
		)
	});

	group.bench_function("peel-and-handle (Pure, peel_ref)", |b| {
		let program: RcFree<IdentityBrand, i32> = RcFree::pure(42);
		b.iter(|| program.peel_ref())
	});

	group.finish();
}
