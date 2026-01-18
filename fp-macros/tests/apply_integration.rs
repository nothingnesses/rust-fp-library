use fp_macros::{Apply, def_kind, impl_kind};

// Define a `Kind` with 1 type parameter and no bounds
def_kind!(
	type Of<T>;
);

struct BoxWrapper<T>(Box<T>);
struct BoxBrand;

impl_kind! {
	impl for BoxBrand {
		type Of<T> = BoxWrapper<T>;
	}
}

trait GetValue {
	fn get_value(&self) -> i32;
}

struct Container;

impl Container {
	fn wrap(val: i32) -> Apply!(<BoxBrand as Kind!( type Of<T>; )>::Of<i32>) {
		BoxWrapper(Box::new(val))
	}

	fn unwrap(w: Apply!(<BoxBrand as Kind!( type Of<T>; )>::Of<i32>)) -> i32 {
		*w.0
	}
}

#[test]
fn test_apply_in_fn_signature() {
	let w = Container::wrap(42);
	assert_eq!(Container::unwrap(w), 42);
}

#[test]
fn test_apply_in_struct_field() {
	struct Item {
		item: Apply!(<BoxBrand as Kind!( type Of<T>; )>::Of<String>),
	}

	let i = Item { item: BoxWrapper(Box::new("hello".to_string())) };
	assert_eq!(*i.item.0, "hello");
}

// Test using Apply! in impl block type
impl GetValue for Apply!(<BoxBrand as Kind!( type Of<T>; )>::Of<i32>) {
	fn get_value(&self) -> i32 {
		*self.0
	}
}

#[test]
fn test_apply_in_impl_block() {
	let w: Apply!(<BoxBrand as Kind!( type Of<T>; )>::Of<i32>) = BoxWrapper(Box::new(999));
	assert_eq!(w.get_value(), 999);
}

// Test nested Apply! usage
#[test]
fn test_nested_apply() {
	// This is a bit contrived but tests macro expansion order/nesting
	// Apply! inside Apply! signature

	// We need a `Kind` that takes another `Kind`'s output
	// But Apply! returns a concrete type, so it's just a type parameter

	struct NestedWrapper<T>(T);
	struct NestedBrand;

	impl_kind! {
		impl for NestedBrand {
			type Of<T> = NestedWrapper<T>;
		}
	}

	type Nested = Apply!(<NestedBrand as Kind!( type Of<T>; )>::Of<
		Apply!(<BoxBrand as Kind!( type Of<T>; )>::Of<i32>)
	>);

	let n: Nested = NestedWrapper(BoxWrapper(Box::new(123)));
	assert_eq!(*(n.0).0, 123);
}
