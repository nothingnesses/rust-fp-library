// POC: Can closureless functions dispatch between by-value and by-ref
// based solely on the container type (owned vs borrowed)?
//
// The idea: a single `alt(fa1, fa2)` function that calls `Alt::alt` when
// both args are owned, and `RefAlt::ref_alt` when both args are borrowed.
// No closure to drive dispatch; the container type `FA` alone determines
// the path.

use fp_library::{
	brands::*,
	classes::{
		Alt,
		RefAlt,
	},
	kinds::{
		InferableBrand_cdc7cd43dac7585f,
		Kind_cdc7cd43dac7585f,
	},
};

// -- Dispatch trait for closureless alt --

trait AltDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, Marker> {
	fn dispatch_alt(
		self,
		other: Self,
	) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>;
}

// Marker types
struct OwnedMarker;
struct BorrowedMarker;

// Val impl: FA = Brand::Of<A> (owned)
impl<'a, Brand, A> AltDispatch<'a, Brand, A, OwnedMarker>
	for <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	Brand: Alt + Kind_cdc7cd43dac7585f,
	A: 'a + Clone,
{
	fn dispatch_alt(
		self,
		other: Self,
	) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A> {
		Brand::alt(self, other)
	}
}

// Ref impl: FA = &Brand::Of<A> (borrowed)
impl<'a, 'b, Brand, A> AltDispatch<'a, Brand, A, BorrowedMarker>
	for &'b <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	Brand: RefAlt + Kind_cdc7cd43dac7585f,
	A: 'a + Clone,
{
	fn dispatch_alt(
		self,
		other: Self,
	) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A> {
		Brand::ref_alt(self, other)
	}
}

// Unified free function (explicit brand)
fn alt_unified<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, FA, Marker>(
	fa1: FA,
	fa2: FA,
) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	FA: AltDispatch<'a, Brand, A, Marker>, {
	fa1.dispatch_alt(fa2)
}

// -- Inference-based alt --

fn alt_infer<'a, FA, A: 'a + Clone, Marker>(
	fa1: FA,
	fa2: FA,
) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	FA: InferableBrand_cdc7cd43dac7585f
		+ AltDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, Marker>, {
	fa1.dispatch_alt(fa2)
}

// -- Tests --

#[test]
fn val_option_alt() {
	let result = alt_unified::<OptionBrand, _, _, _>(None::<i32>, Some(5));
	assert_eq!(result, Some(5));
}

#[test]
fn ref_option_alt() {
	let x: Option<i32> = None;
	let y: Option<i32> = Some(5);
	let result = alt_unified::<OptionBrand, _, _, _>(&x, &y);
	assert_eq!(result, Some(5));
}

#[test]
fn val_vec_alt() {
	let result = alt_unified::<VecBrand, _, _, _>(vec![1, 2], vec![3, 4]);
	assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn ref_vec_alt() {
	let x = vec![1, 2];
	let y = vec![3, 4];
	let result = alt_unified::<VecBrand, _, _, _>(&x, &y);
	assert_eq!(result, vec![1, 2, 3, 4]);
}

// -- Inference tests --

#[test]
fn infer_val_option_alt() {
	let result = alt_infer(None::<i32>, Some(5));
	assert_eq!(result, Some(5));
}

#[test]
fn infer_ref_option_alt() {
	let x: Option<i32> = None;
	let y: Option<i32> = Some(5);
	let result = alt_infer(&x, &y);
	assert_eq!(result, Some(5));
}

#[test]
fn infer_val_vec_alt() {
	let result = alt_infer(vec![1, 2], vec![3, 4]);
	assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn infer_ref_vec_alt() {
	let x = vec![1, 2];
	let y = vec![3, 4];
	let result = alt_infer(&x, &y);
	assert_eq!(result, vec![1, 2, 3, 4]);
}

// -- Test: container reuse after ref alt --

#[test]
fn ref_alt_preserves_originals() {
	let x = vec![1, 2];
	let y = vec![3, 4];
	let _result = alt_infer(&x, &y);
	// x and y are still usable
	assert_eq!(x, vec![1, 2]);
	assert_eq!(y, vec![3, 4]);
}

// -- Test: compact (single-arg closureless function) --

trait CompactDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, Marker> {
	fn dispatch_compact(self) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>;
}

impl<'a, Brand, A> CompactDispatch<'a, Brand, A, OwnedMarker>
	for <Brand as Kind_cdc7cd43dac7585f>::Of<'a, Option<A>>
where
	Brand: fp_library::classes::Compactable + Kind_cdc7cd43dac7585f,
	A: 'a,
{
	fn dispatch_compact(self) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A> {
		Brand::compact(self)
	}
}

impl<'a, 'b, Brand, A> CompactDispatch<'a, Brand, A, BorrowedMarker>
	for &'b <Brand as Kind_cdc7cd43dac7585f>::Of<'a, Option<A>>
where
	Brand: fp_library::classes::RefCompactable + Kind_cdc7cd43dac7585f,
	A: 'a + Clone,
{
	fn dispatch_compact(self) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, A> {
		Brand::ref_compact(self)
	}
}

fn compact_infer<'a, FA, A: 'a, Marker>(
	fa: FA
) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	FA: InferableBrand_cdc7cd43dac7585f
		+ CompactDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, Marker>, {
	fa.dispatch_compact()
}

// InferableBrand for Vec<Option<A>> resolves to VecBrand (since Vec<T>: InferableBrand)

#[test]
fn infer_val_vec_compact() {
	let result = compact_infer(vec![Some(1), None, Some(3)]);
	assert_eq!(result, vec![1, 3]);
}

#[test]
fn infer_ref_vec_compact() {
	let v = vec![Some(1), None, Some(3)];
	let result = compact_infer(&v);
	assert_eq!(result, vec![1, 3]);
}
