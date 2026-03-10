//! Compile-fail and compile-pass tests for fp-macros.
//!
//! Compile-fail tests verify that macros produce helpful error messages.
//! Compile-pass tests verify that warning-emitting macros don't block compilation.

#[test]
fn compile_fail_tests() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/ui/*.rs");
	t.pass("tests/compile-pass/*.rs");
}
