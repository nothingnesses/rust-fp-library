#![expect(
	clippy::panic,
	reason = "Probe tests panic on unreachable Coproduct branches for clarity."
)]
// Probe: validate that a `CoproductBrand<H, T>` can be registered with
// `impl_kind!` and implement fp-library's brand-level `Functor` trait
// recursively, with `CNilBrand` as the base case.
//
// The probe answers two questions for Phase 2 step 2:
//
// 1. Can `impl_kind!` accept a generic Brand whose `Of<'a, A>` resolves
//    to a foreign value type (`frunk_core::coproduct::Coproduct`)? If
//    yes, a `CoproductBrand` is the right pattern: brand types in
//    `brands.rs`, value types pulled from frunk_core, no foreign-trait
//    on foreign-type collisions and no need for the `BrandedCoproduct`
//    wrapper at the Brand boundary.
// 2. Does the recursive `Functor` impl on `CoproductBrand<H, T>`
//    type-check, dispatching `map` to the head or tail brand based on
//    the runtime variant?
//
// The probe defines the brands inline so it does not promote anything
// to the production codebase before the design is settled. If it
// passes, Phase 2 step 2 promotes `CoproductBrand` and `CNilBrand`
// into `fp-library/src/brands.rs` proper.

// `Kind` is referenced only inside `Kind!(...)` macro invocations below.
// rustc's unused-import analysis does not see the macro call as a direct
// use, so the import looks dead even though removing it breaks compilation.
#[expect(
	unused_imports,
	reason = "Kind is referenced via the Kind!(...) macro below, which rustc does not detect as a direct use."
)]
use fp_macros::Kind;
use {
	fp_library::{
		Apply,
		brands::*,
		classes::*,
		impl_kind,
		kinds::*,
		types::run::coproduct::{
			CNil,
			Coproduct,
		},
	},
	std::marker::PhantomData,
};

// -- Brand definitions --

/// Brand for the empty coproduct row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CNilBrand;

impl_kind! {
	for CNilBrand {
		type Of<'a, A: 'a>: 'a = CNil;
	}
}

/// Brand for a coproduct row with head brand `H` and tail brand `T`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoproductBrand<H, T>(PhantomData<(H, T)>);

impl_kind! {
	impl<H: Kind_cdc7cd43dac7585f + 'static, T: Kind_cdc7cd43dac7585f + 'static>
		for CoproductBrand<H, T> {
		type Of<'a, A: 'a>: 'a = Coproduct<
			Apply!(<H as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<T as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>;
	}
}

// -- Functor impls --

impl Functor for CNilBrand {
	fn map<'a, A: 'a, B: 'a>(
		_func: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		// `CNil` is uninhabited; the `match` exhaustively rules out a
		// value, which lets the function "return" without producing
		// one.
		match fa {}
	}
}

impl<H, T> Functor for CoproductBrand<H, T>
where
	H: Functor + 'static,
	T: Functor + 'static,
{
	fn map<'a, A: 'a, B: 'a>(
		func: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			Coproduct::Inl(h) => Coproduct::Inl(<H as Functor>::map(func, h)),
			Coproduct::Inr(t) => Coproduct::Inr(<T as Functor>::map(func, t)),
		}
	}
}

// -- Tests --

type TwoEffectRow<A> = Apply!(<CoproductBrand<OptionBrand, CoproductBrand<VecBrand, CNilBrand>> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

#[test]
fn kind_resolves_two_effect_row() {
	// The `Of<'_, i32>` projection should resolve to
	// `Coproduct<Option<i32>, Coproduct<Vec<i32>, CNil>>`.
	let row: TwoEffectRow<i32> = Coproduct::inject(Some(7));
	match row {
		Coproduct::Inl(opt) => assert_eq!(opt, Some(7)),
		Coproduct::Inr(_) => panic!("expected the head Inl branch"),
	}
}

#[test]
fn functor_dispatches_to_head_branch() {
	let row: TwoEffectRow<i32> = Coproduct::inject(Some(10));
	let mapped: TwoEffectRow<i32> = <CoproductBrand<
		OptionBrand,
		CoproductBrand<VecBrand, CNilBrand>,
	> as Functor>::map(|x: i32| x + 1, row);
	match mapped {
		Coproduct::Inl(opt) => assert_eq!(opt, Some(11)),
		Coproduct::Inr(_) => panic!("expected head Inl branch"),
	}
}

#[test]
fn functor_dispatches_to_tail_branch() {
	let row: TwoEffectRow<i32> = Coproduct::inject(vec![1_i32, 2, 3]);
	let mapped: TwoEffectRow<i32> = <CoproductBrand<
		OptionBrand,
		CoproductBrand<VecBrand, CNilBrand>,
	> as Functor>::map(|x: i32| x * 10, row);
	match mapped {
		Coproduct::Inl(_) => panic!("expected tail Inr branch"),
		Coproduct::Inr(rest) => match rest {
			Coproduct::Inl(v) => assert_eq!(v, vec![10, 20, 30]),
			Coproduct::Inr(_) => panic!("expected the second-position Inl branch"),
		},
	}
}
