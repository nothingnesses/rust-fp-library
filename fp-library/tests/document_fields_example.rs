use fp_library::{Apply, kinds::*};
use fp_macros::document_fields;

// Example: Using document_fields on a tuple struct similar to Endomorphism
#[document_fields("The wrapped morphism from an object to itself")]
pub struct MyEndomorphism<'a, C: fp_library::classes::Category, A>(
	pub Apply!(<C as Kind!(type Of<'a, T, U>;)>::Of<'a, A, A>),
);

// Example: Using document_fields on a named struct
#[document_fields(
    value: "The wrapped value",
    metadata: "Optional metadata about the value"
)]
pub struct Tagged<T> {
	pub value: T,
	pub metadata: Option<String>,
}

// Example: Named struct with multiple fields
#[document_fields(
    x: "The x coordinate",
    y: "The y coordinate",
    z: "The z coordinate"
)]
pub struct Point3D {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

#[test]
fn test_document_fields_usage() {
	// Just verify they compile
	let point = Point3D { x: 1.0, y: 2.0, z: 3.0 };
	assert_eq!(point.x, 1.0);

	let tagged = Tagged { value: 42, metadata: Some("answer".to_string()) };
	assert_eq!(tagged.value, 42);
}
