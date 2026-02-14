// Test file to explore explicit lifetime annotations in function calls
//
// Key finding: Rust distinguishes between "late bound" and "early bound" lifetimes:
// - Late bound lifetimes: Cannot be explicitly specified with turbofish
// - Early bound lifetimes: Can be explicitly specified with turbofish

// === LATE BOUND LIFETIMES ===
// These lifetimes only appear in function parameters/return types
// They CANNOT be explicitly annotated when calling the function

// Simple function with a LATE BOUND lifetime parameter
fn identity(x: &str) -> &str {
	x
}

// Function with multiple LATE BOUND lifetime parameters
fn first_of_two<'a>(
	x: &'a str,
	_y: &str,
) -> &'a str {
	x
}

// Function with LATE BOUND lifetime and type parameters
fn generic_identity<T>(x: &T) -> &T {
	x
}

// === EARLY BOUND LIFETIMES ===
// These lifetimes appear in positions that require them to be known at call site
// They CAN be explicitly annotated when calling the function

// Lifetime used in a where clause - makes it EARLY BOUND
fn early_bound_where<'a, T>(x: &'a T) -> &'a T
where
	T: 'a, // Lifetime appears in where clause - early bound!
{
	x
}

// Lifetime used in a trait bound - makes it EARLY BOUND
fn early_bound_trait<'a, T: 'a>(x: &'a T) -> &'a T {
	x
}

// Lifetime used as a type parameter constraint - EARLY BOUND
fn early_bound_complex<'a, T>(x: &'a T) -> &'a T
where
	T: std::fmt::Debug + 'a,
{
	x
}

// Using const generics with lifetime - EARLY BOUND
fn early_bound_const<'a, T, const N: usize>(x: &'a [T; N]) -> &'a [T; N]
where
	T: 'a,
{
	x
}

#[cfg(test)]
mod tests {
	use super::*;

	// ===== LATE BOUND LIFETIME TESTS =====
	// These show that late bound lifetimes work with inference

	#[test]
	fn test_late_bound_implicit() {
		// Normal call - lifetimes inferred (this works)
		let s = String::from("hello");
		let result = identity(&s);
		assert_eq!(result, "hello");
	}

	#[test]
	fn test_late_bound_type_explicit() {
		// Can specify type parameters even if lifetimes are late-bound
		let x = 42;
		let result = generic_identity::<i32>(&x);
		assert_eq!(*result, 42);
	}

	#[test]
	fn test_late_bound_multiple_implicit() {
		let s1 = String::from("first");
		let s2 = String::from("second");
		let result = first_of_two(&s1, &s2);
		assert_eq!(result, "first");
	}

	// Uncommenting the following would cause compile error E0794:
	// "cannot specify lifetime arguments explicitly if late bound lifetime parameters are present"

	// #[test]
	// fn test_late_bound_explicit_fails() {
	//     let s = String::from("world");
	//     let result = identity::<'_>(&s); // ERROR!
	//     assert_eq!(result, "world");
	// }

	// ===== EARLY BOUND LIFETIME TESTS =====
	// These show that early bound lifetimes CAN be explicitly specified

	#[test]
	fn test_early_bound_where_implicit() {
		let x = 42;
		let result = early_bound_where(&x);
		assert_eq!(*result, 42);
	}

	#[test]
	fn test_early_bound_where_explicit_underscore() {
		let x = 42;
		// This works! Lifetime can be explicitly specified with underscore
		let result = early_bound_where::<'_, i32>(&x);
		assert_eq!(*result, 42);
	}

	#[test]
	fn test_early_bound_where_explicit_type_only() {
		let x = 42;
		// Can also just specify the type and let lifetime be inferred
		let result = early_bound_where::<i32>(&x);
		assert_eq!(*result, 42);
	}

	#[test]
	fn test_early_bound_trait_implicit() {
		let x = 42;
		let result = early_bound_trait(&x);
		assert_eq!(*result, 42);
	}

	#[test]
	fn test_early_bound_trait_explicit() {
		let x = 42;
		// This works! The lifetime bound makes it early bound
		let result = early_bound_trait::<'_, i32>(&x);
		assert_eq!(*result, 42);
	}

	#[test]
	fn test_early_bound_complex_explicit() {
		let x = 42;
		// Multiple constraints with lifetime still makes it early bound
		let result = early_bound_complex::<'_, i32>(&x);
		assert_eq!(*result, 42);
	}

	#[test]
	fn test_early_bound_static() {
		// Can even specify 'static explicitly with static data
		static S: &str = "static string";
		let result = early_bound_where::<'static, &str>(&S);
		assert_eq!(*result, "static string");
	}

	#[test]
	fn test_early_bound_const_implicit() {
		let arr = [1, 2, 3];
		let result = early_bound_const(&arr);
		assert_eq!(result, &[1, 2, 3]);
	}

	#[test]
	fn test_early_bound_const_explicit() {
		let arr = [1, 2, 3];
		// With const generic and where clause, lifetime is early bound
		let result = early_bound_const::<'_, i32, 3>(&arr);
		assert_eq!(result, &[1, 2, 3]);
	}
}
