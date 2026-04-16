// Seventh Slot POC: validates that the Marker-via-Slot pattern
// generalises from `map` (Functor) to `bind` (Semimonad), including
// for multi-brand types via `ResultErrAppliedBrand<E>`'s existing
// Semimonad and RefSemimonad impls.
//
// POC 5 established the pattern for Functor::map:
//   - Slot carries Brand as a trait parameter (for coherence).
//   - Slot carries Marker as an associated-type projection keyed on
//     FA's reference-ness (for early Marker commitment in dispatch).
// All four cases (Val/Ref x single/multi-brand) work in a single
// `map(f, fa)` signature.
//
// This POC verifies the same pattern applies to `bind`, which has a
// structurally different signature: the closure returns a container
// of the same brand (`Fn(A) -> Of<B>` rather than `Fn(A) -> B`).
// Brand therefore appears in two positions in the signature rather
// than one.
//
// Multi-brand coverage: the library's `ResultErrAppliedBrand<E>`
// implements `Semimonad` and `RefSemimonad` (matching PureScript's
// `Bind (Either e)` instance). `ResultOkAppliedBrand<T>` does not
// have a Semimonad instance, so the diagonal-disambiguation cases
// from POC 5 don't apply here - there's only one multi-brand
// candidate per (FA, A). Still, the POC exercises the same
// inference machinery across Val/Ref x single/multi-brand for
// bind's closure-returns-container signature shape.

#![allow(unused_imports, reason = "Kind is used inside Apply! macro expansion")]

use {
	fp_library::{
		brands::{
			LazyBrand,
			OptionBrand,
			ResultErrAppliedBrand,
			VecBrand,
		},
		dispatch::{
			Ref,
			Val,
			semimonad::BindDispatch,
		},
		kinds::Kind_cdc7cd43dac7585f,
		types::{
			Lazy,
			RcLazy,
			lazy::LazyConfig,
		},
	},
	fp_macros::{
		Apply,
		Kind,
	},
};

// -------------------------------------------------------------------------
// Slot (arity-1, same shape as POC 5's SlotM).
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait SlotB_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
	type Marker;
}

impl<'a, A: 'a> SlotB_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {
	type Marker = Val;
}

impl<'a, A: 'a> SlotB_cdc7cd43dac7585f<'a, VecBrand, A> for Vec<A> {
	type Marker = Val;
}

impl<'a, A: 'a, Config: LazyConfig> SlotB_cdc7cd43dac7585f<'a, LazyBrand<Config>, A>
	for Lazy<'a, A, Config>
{
	type Marker = Val;
}

// Multi-brand: ResultErrAppliedBrand<E> fixes E and is Semimonad over
// the Ok side. ResultOkAppliedBrand<T> is Functor-only (no Semimonad),
// so only one multi-brand direction is exercised here - but that's
// enough to confirm the dispatch machinery works for multi-brand
// bind.
impl<'a, A: 'a, E: 'static> SlotB_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
	type Marker = Val;
}

impl<'a, T: ?Sized, Brand, A: 'a> SlotB_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: SlotB_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
	type Marker = Ref;
}

// -------------------------------------------------------------------------
// Unified bind: Marker projected from Slot.
// -------------------------------------------------------------------------

pub fn bind<'a, FA, A: 'a, B: 'a, Brand>(
	ma: FA,
	f: impl BindDispatch<'a, Brand, A, B, FA, <FA as SlotB_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: SlotB_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(ma)
}

// -------------------------------------------------------------------------
// Val tests: owned containers, Fn(A) -> Of<B> closures.
// -------------------------------------------------------------------------

#[test]
fn val_option_some() {
	let r: Option<i32> = bind(Some(5), |x: i32| Some(x * 2));
	assert_eq!(r, Some(10));
}

#[test]
fn val_option_none_passthrough() {
	let r: Option<i32> = bind(None::<i32>, |x: i32| Some(x * 2));
	assert_eq!(r, None);
}

#[test]
fn val_option_short_circuit() {
	let r: Option<i32> = bind(Some(5), |_: i32| None);
	assert_eq!(r, None);
}

#[test]
fn val_vec() {
	let r: Vec<i32> = bind(vec![1, 2, 3], |x: i32| vec![x, x * 10]);
	assert_eq!(r, vec![1, 10, 2, 20, 3, 30]);
}

// Note: LazyBrand<RcLazyConfig> only implements RefSemimonad, not
// Semimonad. Val bind is not defined for Lazy, so no val_lazy test.

// -------------------------------------------------------------------------
// Ref tests: borrowed containers, Fn(&A) -> Of<B> closures.
// -------------------------------------------------------------------------

#[test]
fn ref_option_some() {
	let opt = Some(5);
	let r: Option<i32> = bind(&opt, |x: &i32| Some(*x * 2));
	assert_eq!(r, Some(10));
	assert_eq!(opt, Some(5));
}

#[test]
fn ref_option_none_passthrough() {
	let opt: Option<i32> = None;
	let r: Option<i32> = bind(&opt, |x: &i32| Some(*x * 2));
	assert_eq!(r, None);
}

#[test]
fn ref_vec() {
	let v = vec![1, 2, 3];
	let r: Vec<i32> = bind(&v, |x: &i32| vec![*x, *x * 10]);
	assert_eq!(r, vec![1, 10, 2, 20, 3, 30]);
	assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn ref_lazy() {
	let lazy = RcLazy::pure(5);
	let r: RcLazy<i32> = bind(&lazy, |x: &i32| {
		let v = *x;
		RcLazy::new(move || v * 3)
	});
	assert_eq!(*r.evaluate(), 15);
}

// -------------------------------------------------------------------------
// Multi-brand tests: Result via ResultErrAppliedBrand<E>.
// -------------------------------------------------------------------------

#[test]
fn val_result_ok() {
	let r: Result<i32, String> = bind(Ok::<i32, String>(5), |x: i32| Ok(x * 2));
	assert_eq!(r, Ok(10));
}

#[test]
fn val_result_err_passthrough() {
	let r: Result<i32, String> = bind(Err::<i32, String>("boom".into()), |x: i32| Ok(x * 2));
	assert_eq!(r, Err("boom".to_string()));
}

#[test]
fn val_result_short_circuit_to_err() {
	let r: Result<i32, String> = bind(Ok::<i32, String>(5), |_: i32| Err("halt".into()));
	assert_eq!(r, Err("halt".to_string()));
}

#[test]
fn ref_result_ok() {
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = bind(&ok, |x: &i32| Ok(*x * 2));
	assert_eq!(r, Ok(10));
	assert_eq!(ok, Ok(5));
}

#[test]
fn ref_result_err_passthrough() {
	let err: Result<i32, String> = Err("boom".into());
	let r: Result<i32, String> = bind(&err, |x: &i32| Ok(*x * 2));
	assert_eq!(r, Err("boom".to_string()));
	assert_eq!(err, Err("boom".to_string()));
}

#[test]
fn ref_result_short_circuit_to_err() {
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = bind(&ok, |_: &i32| Err("halt".into()));
	assert_eq!(r, Err("halt".to_string()));
}
