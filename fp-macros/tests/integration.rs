use fp_macros::{Apply, def_kind, impl_kind};
use std::fmt::Display;

// ===========================================================================
// Test 1: Simple Kind (No Lifetimes)
// ===========================================================================

// Define a Kind with 1 type parameter and no bounds
def_kind!((), (T), ());

struct Wrapper<T>(T);
struct WrapperBrand;

impl_kind! {
	impl for WrapperBrand {
		type Of<T> = Wrapper<T>;
	}
}

#[test]
fn test_wrapper_kind() {
	let w = Wrapper(42);
	assert_eq!(w.0, 42);
}

#[test]
fn test_apply_macro_simple() {
	type Applied = Apply!(
		brand: WrapperBrand,
		signature: (T), // No output bounds
		types: (i32)
	);

	let w: Applied = Wrapper(100);
	assert_eq!(w.0, 100);
}

// ===========================================================================
// Test 2: Kind with Lifetimes
// ===========================================================================

// Define a Kind with 1 lifetime and 1 type bounded by that lifetime
def_kind!(('a), (T: 'a), ());

struct RefWrapper<'a, T: 'a>(&'a T);
struct RefWrapperBrand;

impl_kind! {
	impl for RefWrapperBrand {
		// T must be bounded by 'a to match def_kind and struct definition
		type Of<'a, T: 'a> = RefWrapper<'a, T>;
	}
}

#[test]
fn test_ref_wrapper_kind() {
	let val = 42;
	let w = RefWrapper(&val);
	assert_eq!(*w.0, 42);
}

#[test]
fn test_apply_macro_with_lifetime() {
	type Applied<'a> = Apply!(
		brand: RefWrapperBrand,
		signature: ('a, T: 'a), // Match def_kind signature
		lifetimes: ('a),
		types: (i32)
	);

	let val = 100;
	let w: Applied = RefWrapper(&val);
	assert_eq!(*w.0, 100);
}

// ===========================================================================
// Test 3: Kind with Bounds
// ===========================================================================

// Define a Kind where the type parameter must implement Display
def_kind!((), (T: Display), ());

struct DisplayWrapper<T: Display>(T);
struct DisplayWrapperBrand;

impl_kind! {
	impl for DisplayWrapperBrand {
		type Of<T: Display> = DisplayWrapper<T>;
	}
}

#[test]
fn test_bounded_kind() {
	type Applied = Apply!(
		brand: DisplayWrapperBrand,
		signature: (T: Display),
		types: (String)
	);

	let w: Applied = DisplayWrapper("hello".to_string());
	assert_eq!(w.0, "hello");
}

// ===========================================================================
// Test 4: Kind with Output Bounds
// ===========================================================================

// Define a Kind where the output type must implement Clone
// We must also require T: Clone because CloneWrapper<T> only implements Clone if T: Clone
def_kind!((), (T: Clone), (Clone));

#[derive(Clone)]
struct CloneWrapper<T>(T);
struct CloneWrapperBrand;

impl_kind! {
	impl for CloneWrapperBrand {
		// We must specify the output bounds here so impl_kind! generates the correct Kind name
		type Of<T: Clone>: Clone = CloneWrapper<T>;
	}
}

#[test]
fn test_output_bounded_kind() {
	// This test just verifies that the code compiles and the trait is found
	let _ = CloneWrapper(10);
}

#[test]
fn test_apply_output_bounded() {
	type Applied = Apply!(
		brand: CloneWrapperBrand,
		signature: (T: Clone) -> Clone,
		types: (i32)
	);

	let w: Applied = CloneWrapper(10);
	let _ = w.clone();
}
