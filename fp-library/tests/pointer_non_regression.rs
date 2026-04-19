#![expect(clippy::unwrap_used, reason = "Tests use panicking operations for brevity and clarity")]

//! Non-regression tests for the pointer and coercion trait APIs.
//!
//! These tests exercise every public method on `Pointer`, `RefCountedPointer`,
//! `SendRefCountedPointer`, `UnsizedCoercible`, and `SendUnsizedCoercible`
//! through both trait method syntax and free functions. They serve as a
//! safety net during the flatten-pointer-hierarchy refactor.

use fp_library::{
	brands::*,
	classes::*,
};

// -- Pointer --

#[test]
fn pointer_new_rc() {
	let ptr = <RcBrand as Pointer>::new(42);
	assert_eq!(*ptr, 42);
}

#[test]
fn pointer_new_arc() {
	let ptr = <ArcBrand as Pointer>::new(42);
	assert_eq!(*ptr, 42);
}

#[test]
fn pointer_new_free_fn_rc() {
	let ptr = pointer::new::<RcBrand, _>(42);
	assert_eq!(*ptr, 42);
}

#[test]
fn pointer_new_free_fn_arc() {
	let ptr = pointer::new::<ArcBrand, _>(42);
	assert_eq!(*ptr, 42);
}

// -- RefCountedPointer: cloneable_new --

#[test]
fn ref_counted_pointer_cloneable_new_rc() {
	let ptr = <RcBrand as RefCountedPointer>::cloneable_new(42);
	assert_eq!(*ptr, 42);
	let clone = ptr.clone();
	assert_eq!(*clone, 42);
}

#[test]
fn ref_counted_pointer_cloneable_new_arc() {
	let ptr = <ArcBrand as RefCountedPointer>::cloneable_new(42);
	assert_eq!(*ptr, 42);
	let clone = ptr.clone();
	assert_eq!(*clone, 42);
}

#[test]
fn ref_counted_pointer_cloneable_new_free_fn_rc() {
	let ptr = ref_counted_pointer::cloneable_new::<RcBrand, _>(42);
	assert_eq!(*ptr, 42);
}

#[test]
fn ref_counted_pointer_cloneable_new_free_fn_arc() {
	let ptr = ref_counted_pointer::cloneable_new::<ArcBrand, _>(42);
	assert_eq!(*ptr, 42);
}

// -- RefCountedPointer: try_unwrap --

#[test]
fn ref_counted_pointer_try_unwrap_rc_sole_ref() {
	let ptr = <RcBrand as RefCountedPointer>::cloneable_new(42);
	assert_eq!(RcBrand::try_unwrap(ptr), Ok(42));
}

#[test]
fn ref_counted_pointer_try_unwrap_rc_multiple_refs() {
	let ptr = <RcBrand as RefCountedPointer>::cloneable_new(42);
	let _clone = ptr.clone();
	assert!(RcBrand::try_unwrap(ptr).is_err());
}

#[test]
fn ref_counted_pointer_try_unwrap_arc_sole_ref() {
	let ptr = <ArcBrand as RefCountedPointer>::cloneable_new(42);
	assert_eq!(ArcBrand::try_unwrap(ptr), Ok(42));
}

#[test]
fn ref_counted_pointer_try_unwrap_free_fn() {
	let ptr = ref_counted_pointer::cloneable_new::<RcBrand, _>(42);
	assert_eq!(ref_counted_pointer::try_unwrap::<RcBrand, _>(ptr), Ok(42));
}

// -- RefCountedPointer: take_cell --

#[test]
fn ref_counted_pointer_take_cell_rc() {
	let cell = <RcBrand as RefCountedPointer>::take_cell_new(42);
	let cell_clone = cell.clone();
	assert_eq!(RcBrand::take_cell_take(&cell), Some(42));
	assert_eq!(RcBrand::take_cell_take(&cell), None);
	assert_eq!(RcBrand::take_cell_take(&cell_clone), None);
}

#[test]
fn ref_counted_pointer_take_cell_arc() {
	let cell = <ArcBrand as RefCountedPointer>::take_cell_new(42);
	let cell_clone = cell.clone();
	assert_eq!(ArcBrand::take_cell_take(&cell), Some(42));
	assert_eq!(ArcBrand::take_cell_take(&cell), None);
	assert_eq!(ArcBrand::take_cell_take(&cell_clone), None);
}

#[test]
fn ref_counted_pointer_take_cell_free_fn_rc() {
	let cell = ref_counted_pointer::take_cell_new::<RcBrand, _>(99);
	assert_eq!(ref_counted_pointer::take_cell_take::<RcBrand, _>(&cell), Some(99));
	assert_eq!(ref_counted_pointer::take_cell_take::<RcBrand, _>(&cell), None);
}

#[test]
fn ref_counted_pointer_take_cell_free_fn_arc() {
	let cell = ref_counted_pointer::take_cell_new::<ArcBrand, _>(99);
	assert_eq!(ref_counted_pointer::take_cell_take::<ArcBrand, _>(&cell), Some(99));
	assert_eq!(ref_counted_pointer::take_cell_take::<ArcBrand, _>(&cell), None);
}

// -- SendRefCountedPointer --

#[test]
fn send_ref_counted_pointer_send_new_arc() {
	let ptr = <ArcBrand as SendRefCountedPointer>::send_new(42);
	assert_eq!(*ptr, 42);
	let clone = ptr.clone();
	assert_eq!(*clone, 42);
}

#[test]
fn send_ref_counted_pointer_send_new_free_fn() {
	let ptr = send_ref_counted_pointer::send_new::<ArcBrand, _>(42);
	assert_eq!(*ptr, 42);
}

#[test]
fn send_ref_counted_pointer_send_across_thread() {
	let ptr = send_ref_counted_pointer::send_new::<ArcBrand, _>(42);
	let handle = std::thread::spawn(move || *ptr);
	assert_eq!(handle.join().unwrap(), 42);
}

// -- UnsizedCoercible --

#[test]
fn unsized_coercible_coerce_fn_rc() {
	let f = <RcBrand as UnsizedCoercible>::coerce_fn(|x: i32| x + 1);
	assert_eq!(f(1), 2);
	let clone = f.clone();
	assert_eq!(clone(1), 2);
}

#[test]
fn unsized_coercible_coerce_fn_arc() {
	let f = <ArcBrand as UnsizedCoercible>::coerce_fn(|x: i32| x + 1);
	assert_eq!(f(1), 2);
}

#[test]
fn unsized_coercible_coerce_ref_fn_rc() {
	let f = <RcBrand as UnsizedCoercible>::coerce_ref_fn(|x: &i32| *x + 1);
	assert_eq!(f(&1), 2);
}

#[test]
fn unsized_coercible_coerce_ref_fn_arc() {
	let f = <ArcBrand as UnsizedCoercible>::coerce_ref_fn(|x: &i32| *x + 1);
	assert_eq!(f(&1), 2);
}

#[test]
fn unsized_coercible_coerce_fn_free_fn_rc() {
	let f = unsized_coercible::coerce_fn::<RcBrand, _, _>(|x: i32| x + 1);
	assert_eq!(f(1), 2);
}

#[test]
fn unsized_coercible_coerce_ref_fn_free_fn_rc() {
	let f = unsized_coercible::coerce_ref_fn::<RcBrand, _, _>(|x: &i32| *x + 1);
	assert_eq!(f(&1), 2);
}

// -- SendUnsizedCoercible --

#[test]
fn send_unsized_coercible_coerce_send_fn_arc() {
	let f = <ArcBrand as SendUnsizedCoercible>::coerce_send_fn(|x: i32| x + 1);
	assert_eq!(f(1), 2);
	let clone = f.clone();
	assert_eq!(clone(1), 2);
}

#[test]
fn send_unsized_coercible_coerce_send_ref_fn_arc() {
	let f = <ArcBrand as SendUnsizedCoercible>::coerce_send_ref_fn(|x: &i32| *x + 1);
	assert_eq!(f(&1), 2);
}

#[test]
fn send_unsized_coercible_coerce_send_fn_free_fn_arc() {
	let f = send_unsized_coercible::coerce_send_fn::<ArcBrand, _, _>(|x: i32| x + 1);
	assert_eq!(f(1), 2);
}

#[test]
fn send_unsized_coercible_coerce_send_ref_fn_free_fn_arc() {
	let f = send_unsized_coercible::coerce_send_ref_fn::<ArcBrand, _, _>(|x: &i32| *x + 1);
	assert_eq!(f(&1), 2);
}

#[test]
fn send_unsized_coercible_send_across_thread() {
	let f = <ArcBrand as SendUnsizedCoercible>::coerce_send_fn(|x: i32| x + 1);
	let handle = std::thread::spawn(move || f(10));
	assert_eq!(handle.join().unwrap(), 11);
}
