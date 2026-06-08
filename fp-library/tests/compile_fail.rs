#[test]
#[cfg(feature = "effects")]
fn compile_fail_tests() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/ui/*.rs");
}

#[test]
#[cfg(not(feature = "effects"))]
fn effects_feature_off_compile_fail_tests() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/ui-feature-off/*.rs");
}
