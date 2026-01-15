use fp_macros::{Apply, def_kind, impl_kind};

// Define a simple Kind for testing
def_kind!((), (T), ());
struct BoxWrapper<T>(Box<T>);
struct BoxBrand;

impl_kind! {
	impl for BoxBrand {
		type Of<T> = BoxWrapper<T>;
	}
}

// Test 1: Works in function signatures
#[test]
fn test_in_function_signatures() {
	fn wrap(val: i32) -> Apply!(brand: BoxBrand, signature: (i32)) {
		BoxWrapper(Box::new(val))
	}

	fn unwrap(w: Apply!(brand: BoxBrand, signature: (i32))) -> i32 {
		*w.0
	}

	let w = wrap(123);
	assert_eq!(unwrap(w), 123);
}

// Test 2: Works in struct definitions
#[test]
fn test_in_struct_definitions() {
	struct Container {
		item: Apply!(brand: BoxBrand, signature: (String)),
	}

	let c = Container { item: BoxWrapper(Box::new("test".to_string())) };

	assert_eq!(*c.item.0, "test");
}

// Test 3: Works in impl blocks
#[test]
fn test_in_impl_blocks() {
	trait GetValue {
		type Value;
		fn get_value(&self) -> Self::Value;
	}

	// Implement trait for the Applied type directly
	impl GetValue for Apply!(brand: BoxBrand, signature: (i32)) {
		type Value = i32;
		fn get_value(&self) -> i32 {
			*self.0
		}
	}

	let w: Apply!(brand: BoxBrand, signature: (i32)) = BoxWrapper(Box::new(999));
	assert_eq!(w.get_value(), 999);
}

// Test 4: Explicit Kind Mode with Lifetimes and Types
#[test]
fn test_explicit_kind_complex() {
	trait MyExplicitKind {
		type Of<'a, T: 'a>;
	}

	struct MyRef<'a, T: 'a>(&'a T);
	struct MyBrand;

	impl MyExplicitKind for MyBrand {
		type Of<'a, T: 'a> = MyRef<'a, T>;
	}

	type Applied<'x> = Apply!(
		brand: MyBrand,
		kind: MyExplicitKind,
		lifetimes: ('x),
		types: (i32)
	);

	let val = 42;
	let w: Applied = MyRef(&val);
	assert_eq!(*w.0, 42);
}

// Test 5: Nested Apply! usage
#[test]
fn test_nested_apply() {
	// BoxWrapper<BoxWrapper<i32>>
	type Nested = Apply!(
		brand: BoxBrand,
		signature: (Apply!(brand: BoxBrand, signature: (i32)))
	);

	let inner = BoxWrapper(Box::new(10));
	let outer: Nested = BoxWrapper(Box::new(inner));

	assert_eq!(*outer.0.0, 10);
}
