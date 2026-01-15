//! Compile-fail tests for fp-macros.
//!
//! These tests verify that the macros produce helpful error messages
//! when given invalid input.

#[test]
fn compile_fail_tests() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/ui/*.rs");
}
