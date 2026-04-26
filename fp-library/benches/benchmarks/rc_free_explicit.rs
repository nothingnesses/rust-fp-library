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
		brands::IdentityBrand,
		types::{
			Identity,
			RcFreeExplicit,
		},
	},
};

fn build_spine(depth: usize) -> RcFreeExplicit<'static, IdentityBrand, i32> {
	let mut program: RcFreeExplicit<'static, IdentityBrand, i32> = RcFreeExplicit::pure(0);
	for _ in 0 .. depth {
		program = RcFreeExplicit::wrap(Identity(program));
	}
	program
}

pub fn bench_rc_free_explicit(c: &mut Criterion) {
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("RcFreeExplicit");

	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind-deep + evaluate", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
				|program| program.bind(|x: i32| RcFreeExplicit::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});
	}

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| {
				b.iter_batched(|| build_spine(k), RcFreeExplicit::evaluate, BatchSize::SmallInput)
			},
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: RcFreeExplicit<'static, IdentityBrand, i32> =
					RcFreeExplicit::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| RcFreeExplicit::pure(x + 1));
				}
				program.evaluate()
			})
		});
	}

	group.bench_function("peel-and-handle (Pure, to_view)", |b| {
		b.iter_batched(
			|| RcFreeExplicit::<'static, IdentityBrand, i32>::pure(42),
			RcFreeExplicit::to_view,
			BatchSize::SmallInput,
		)
	});

	group.bench_function("peel-and-handle (Pure, peel_ref)", |b| {
		let program: RcFreeExplicit<'static, IdentityBrand, i32> = RcFreeExplicit::pure(42);
		b.iter(|| program.peel_ref())
	});

	group.finish();
}
