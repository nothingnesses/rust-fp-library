use fp_macros::document_fields;

// Test named struct
#[document_fields(
    x: "The x coordinate",
    y: "The y coordinate"
)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

// Test tuple struct
#[document_fields("The wrapped value")]
pub struct Wrapper(pub i32);

// Test tuple struct with multiple fields
#[document_fields("The first value", "The second value", "The third value")]
pub struct Triple(pub i32, pub String, pub bool);

// Test struct with lifetimes and generics
#[document_fields(
    data: "The wrapped data"
)]
pub struct Container<'a, T> {
	pub data: &'a T,
}

#[test]
fn test_structs_compile() {
	let _p = Point { x: 1, y: 2 };
	let _w = Wrapper(42);
	let _t = Triple(1, "hello".to_string(), true);
	let value = 100;
	let _c = Container { data: &value };
}
