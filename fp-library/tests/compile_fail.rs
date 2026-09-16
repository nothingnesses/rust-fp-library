#[test]
#[cfg(feature = "effects")]
fn compile_fail_tests() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/ui/*.rs");
}
