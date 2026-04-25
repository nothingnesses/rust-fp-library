// Criterion bench measuring the cost of naive recursive `FreeExplicit::bind`
// on left-associated chains at increasing depths.
//
// The bench imports the production `FreeExplicit` from the library; the
// shape (build a deep `Wrap` spine, then bind across it, then evaluate)
// matches the POC bench so depth-keyed measurements remain comparable.

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

// -- Bench --

fn build_spine(depth: usize) -> FreeExplicit<'static, IdentityBrand, i32> {
	let mut program: FreeExplicit<'static, IdentityBrand, i32> = FreeExplicit::pure(0);
	for _ in 0 .. depth {
		program = FreeExplicit::wrap(Identity(Box::new(program)));
	}
	program
}

pub fn bench_free_explicit(c: &mut Criterion) {
	// Depths surface the O(N) cost of `bind` walking an existing spine.
	// We stop at 10 000 because naive recursion's scaling is already
	// clear in that range; deeper measurements would burn wall-clock
	// time without changing the qualitative answer.
	let depths: &[usize] = &[10, 100, 1_000, 10_000];

	let mut group = c.benchmark_group("FreeExplicit");

	// Bind over a pre-built spine. The spine is constructed in the
	// `iter_batched` setup so only the bind + evaluate cost is measured.
	// Naive recursive bind walks the whole spine (O(depth)) and produces
	// a new spine of the same shape; evaluate then walks it again.
	for &depth in depths {
		group.bench_with_input(BenchmarkId::new("bind over Wrap spine", depth), &depth, |b, &k| {
			b.iter_batched(
				|| build_spine(k),
				|program| program.bind(|x| FreeExplicit::pure(x + 1)).evaluate(),
				BatchSize::SmallInput,
			)
		});
	}

	// Reference: cost of evaluating the spine without any bind. This
	// isolates the "walk once" cost so the bench output shows how much
	// of the `bind + evaluate` number is attributable to bind itself.
	for &depth in depths {
		group.bench_with_input(
			BenchmarkId::new("evaluate only (reference)", depth),
			&depth,
			|b, &k| {
				b.iter_batched(|| build_spine(k), FreeExplicit::evaluate, BatchSize::SmallInput)
			},
		);
	}

	group.finish();
}
