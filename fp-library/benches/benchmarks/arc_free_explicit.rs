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
		brands::IdentityBrand,
		types::{
			ArcFreeExplicit,
			Identity,
		},
	},
};

fn build_spine(depth: usize) -> ArcFreeExplicit<'static, IdentityBrand, i32> {
	let mut program: ArcFreeExplicit<'static, IdentityBrand, i32> = ArcFreeExplicit::pure(0);
	for _ in 0 .. depth {
		program = ArcFreeExplicit::wrap(Identity(program));
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
				|program| program.bind(|x: i32| ArcFreeExplicit::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});
	}

	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| {
				b.iter_batched(|| build_spine(k), ArcFreeExplicit::evaluate, BatchSize::SmallInput)
			},
		);
	}

	for &width in depths {
		group.bench_with_input(BenchmarkId::new("bind-wide + evaluate", width), &width, |b, &k| {
			b.iter(|| {
				let mut program: ArcFreeExplicit<'static, IdentityBrand, i32> =
					ArcFreeExplicit::pure(0);
				for _ in 0 .. k {
					program = program.bind(|x: i32| ArcFreeExplicit::pure(x + 1));
				}
				program.evaluate()
			})
		});
	}

	group.bench_function("peel-and-handle (Pure, to_view)", |b| {
		b.iter_batched(
			|| ArcFreeExplicit::<'static, IdentityBrand, i32>::pure(42),
			ArcFreeExplicit::to_view,
			BatchSize::SmallInput,
		)
	});

	group.bench_function("peel-and-handle (Pure, peel_ref)", |b| {
		let program: ArcFreeExplicit<'static, IdentityBrand, i32> = ArcFreeExplicit::pure(42);
		b.iter(|| program.peel_ref())
	});

	group.finish();
}
