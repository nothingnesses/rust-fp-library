// Non-regression tests for single-brand inference.
//
// These tests exercise existing inference patterns that must remain
// green throughout the multi-brand ergonomics migration. They use the
// library's public inference wrappers (not explicit:: variants) to
// validate that single-brand types continue to work with brand
// inference after each migration step.

use fp_library::{
	brands::*,
	functions::*,
};

// -- Functor (map) --

#[test]
fn nr01_map_val_option() {
	assert_eq!(map(|x: i32| x + 1, Some(5)), Some(6));
}

#[test]
fn nr02_map_val_vec() {
	assert_eq!(map(|x: i32| x + 1, vec![1, 2, 3]), vec![2, 3, 4]);
}

#[test]
fn nr03_map_ref_option() {
	let opt = Some(5);
	assert_eq!(map(|x: &i32| *x + 1, &opt), Some(6));
	assert_eq!(opt, Some(5));
}

#[test]
fn nr04_map_ref_vec() {
	let v = vec![1, 2, 3];
	assert_eq!(map(|x: &i32| *x + 1, &v), vec![2, 3, 4]);
	assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn nr05_map_type_changing() {
	assert_eq!(map(|x: i32| x.to_string(), Some(5)), Some("5".to_string()));
}

// -- Semimonad (bind) --

#[test]
fn nr06_bind_val_option_pass() {
	assert_eq!(bind(Some(5), |x: i32| if x > 3 { Some(x) } else { None }), Some(5));
}

#[test]
fn nr07_bind_val_option_none() {
	assert_eq!(bind(None::<i32>, |x: i32| Some(x + 1)), None);
}

#[test]
fn nr08_bind_val_vec() {
	assert_eq!(bind(vec![1, 2, 3], |x: i32| vec![x, x * 10]), vec![1, 10, 2, 20, 3, 30]);
}

#[test]
fn nr09_bind_ref_option() {
	let opt = Some(5);
	assert_eq!(bind(&opt, |x: &i32| Some(*x + 1)), Some(6));
	assert_eq!(opt, Some(5));
}

// -- Semimonad (join) --

#[test]
fn nr10_join_option_some() {
	assert_eq!(join(Some(Some(5))), Some(5));
}

#[test]
fn nr11_join_option_none() {
	assert_eq!(join(Some(None::<i32>)), None);
}

// -- Foldable (fold_right) --

#[test]
fn nr12a_fold_right_val_vec() {
	let result = fold_right::<RcFnBrand, _, _, _, _>(|a: i32, b: i32| a + b, 0, vec![1, 2, 3]);
	assert_eq!(result, 6);
}

// -- Filterable (filter) --

#[test]
fn nr12_filter_val_vec() {
	assert_eq!(filter(|x: i32| x > 2, vec![1, 2, 3, 4]), vec![3, 4]);
}

// -- Lift --

#[test]
fn nr13_lift2_option_both_some() {
	assert_eq!(lift2(|a: i32, b: i32| a + b, Some(1), Some(2)), Some(3));
}

#[test]
fn nr14_lift2_option_one_none() {
	assert_eq!(lift2(|a: i32, b: i32| a + b, Some(1), None::<i32>), None);
}

// -- Alt --

#[test]
fn nr15_alt_none_some() {
	assert_eq!(alt(None::<i32>, Some(5)), Some(5));
}

#[test]
fn nr16_alt_some_some() {
	assert_eq!(alt(Some(3), Some(5)), Some(3));
}

// -- Traversable (traverse) --

#[test]
fn nr17_traverse_val_vec_all_pass() {
	let result = traverse::<RcFnBrand, _, _, _, OptionBrand, _>(
		|x: i32| if x > 0 { Some(x) } else { None },
		vec![1, 2, 3],
	);
	assert_eq!(result, Some(vec![1, 2, 3]));
}

#[test]
fn nr18_traverse_val_vec_short_circuit() {
	let result = traverse::<RcFnBrand, _, _, _, OptionBrand, _>(
		|x: i32| if x > 0 { Some(x) } else { None },
		vec![1, -1, 3],
	);
	assert_eq!(result, None);
}
