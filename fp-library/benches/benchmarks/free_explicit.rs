// Criterion bench answering Q3 from `tests/free_explicit_poc.rs`:
// how expensive is a naive recursive `FreeExplicit::bind` on
// left-associated chains at increasing depths?
//
// The definition of `FreeExplicit` is duplicated here (minimally) so the
// bench does not depend on any `src/` module; if `FreeExplicit` is later
// promoted to `src/`, this bench should `use` it from there instead.

// `Kind` is referenced only inside `Kind!(...)` macro invocations in the
// enum definition below. rustc's unused-import analysis does not see the
// macro call as a direct use of the imported name, so this import looks
// dead even though removing it breaks compilation.
#[expect(
	unused_imports,
	reason = "Kind is referenced via the Kind!(...) macro below, which rustc does not detect as a direct use."
)]
use fp_macros::Kind;
use {
	criterion::{
		BatchSize,
		BenchmarkId,
		Criterion,
	},
	fp_library::{
		Apply,
		brands::IdentityBrand,
		classes::Functor,
		kinds::*,
		types::Identity,
	},
	std::rc::Rc,
};

// -- Minimal FreeExplicit (matches the POC at tests/free_explicit_poc.rs) --

enum FreeExplicit<'a, F, A: 'a>
where
	F: Kind_cdc7cd43dac7585f + 'a, {
	Pure(A),
	Wrap(
		Apply!(
			<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Box<FreeExplicit<'a, F, A>>>
		),
	),
}

impl<'a, F, A: 'a> FreeExplicit<'a, F, A>
where
	F: Functor + 'a,
{
	fn bind<B: 'a>(
		self,
		f: impl Fn(A) -> FreeExplicit<'a, F, B> + 'a,
	) -> FreeExplicit<'a, F, B> {
		let boxed: Rc<dyn Fn(A) -> FreeExplicit<'a, F, B> + 'a> = Rc::new(f);
		self.bind_boxed(boxed)
	}

	fn bind_boxed<B: 'a>(
		self,
		f: Rc<dyn Fn(A) -> FreeExplicit<'a, F, B> + 'a>,
	) -> FreeExplicit<'a, F, B> {
		match self {
			FreeExplicit::Pure(a) => f(a),
			FreeExplicit::Wrap(fa) => {
				let f_outer = Rc::clone(&f);
				FreeExplicit::Wrap(F::map(
					move |inner: Box<FreeExplicit<'a, F, A>>| -> Box<FreeExplicit<'a, F, B>> {
						let f_inner = Rc::clone(&f_outer);
						Box::new((*inner).bind_boxed(f_inner))
					},
					fa,
				))
			}
		}
	}
}

impl<'a, A: 'a> FreeExplicit<'a, IdentityBrand, A> {
	fn evaluate_identity(self) -> A {
		let mut current = self;
		loop {
			match current {
				FreeExplicit::Pure(a) => return a,
				FreeExplicit::Wrap(Identity(boxed)) => current = *boxed,
			}
		}
	}
}

// -- Bench --

fn build_spine(depth: usize) -> FreeExplicit<'static, IdentityBrand, i32> {
	let mut program: FreeExplicit<'static, IdentityBrand, i32> = FreeExplicit::Pure(0);
	for _ in 0 .. depth {
		program = FreeExplicit::Wrap(Identity(Box::new(program)));
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
				|program| program.bind(|x| FreeExplicit::Pure(x + 1)).evaluate_identity(),
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
				b.iter_batched(
					|| build_spine(k),
					FreeExplicit::evaluate_identity,
					BatchSize::SmallInput,
				)
			},
		);
	}

	group.finish();
}
