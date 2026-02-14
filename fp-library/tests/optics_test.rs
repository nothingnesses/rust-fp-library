use fp_library::{
	brands::*,
	classes::{Choice, Profunctor, Strong},
	functions::*,
	types::optics::*,
};

#[test]
fn test_lens_view_and_set() {
	#[derive(Clone, Debug, PartialEq)]
	struct Person {
		name: String,
		age: i32,
	}

	let age_lens = Lens::new(|p: &Person| p.age, |p: Person, age: i32| Person { age, ..p });

	let person = Person { name: "Alice".to_string(), age: 30 };

	// Test view
	assert_eq!(age_lens.view(&person), 30);

	// Test set
	let updated = age_lens.set(person.clone(), 31);
	assert_eq!(updated.age, 31);
	assert_eq!(updated.name, "Alice");
}

#[test]
fn test_lens_over() {
	#[derive(Clone, Debug, PartialEq)]
	struct Counter {
		count: i32,
	}

	let count_lens = Lens::new(|c: &Counter| c.count, |c: Counter, count: i32| Counter { count });

	let counter = Counter { count: 5 };
	let incremented = count_lens.over(counter, |x| x + 1);

	assert_eq!(incremented.count, 6);
}

#[test]
fn test_prism_preview_and_review() {
	let ok_prism: Prism<Result<i32, String>, i32> =
		Prism::new(|r: Result<i32, String>| r.ok(), |x: i32| Ok(x));

	// Test preview
	assert_eq!(ok_prism.preview(Ok(42)), Some(42));
	assert_eq!(ok_prism.preview(Err("error".to_string())), None);

	// Test review
	assert_eq!(ok_prism.review(42), Ok(42));
}

#[test]
fn test_profunctor_dimap() {
	// Test that functions are profunctors
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = dimap::<RcFnBrand, _, _, _, _, _, _>(|x: i32| x * 2, |x: i32| x - 1, f);

	assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
}

#[test]
fn test_profunctor_lmap() {
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = lmap::<RcFnBrand, _, _, _, _>(|x: i32| x * 2, f);

	assert_eq!(g(10), 21); // (10 * 2) + 1 = 21
}

#[test]
fn test_profunctor_rmap() {
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = rmap::<RcFnBrand, _, _, _, _>(|x: i32| x * 2, f);

	assert_eq!(g(10), 22); // (10 + 1) * 2 = 22
}

#[test]
fn test_strong_first() {
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = first::<RcFnBrand, _, _, i32>(f);

	assert_eq!(g((10, 20)), (11, 20));
}

#[test]
fn test_strong_second() {
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = second::<RcFnBrand, _, _, i32>(f);

	assert_eq!(g((20, 10)), (20, 11));
}

#[test]
fn test_choice_left() {
	// left: Result<C, A> -> Result<C, B>, transforms the Err variant
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = left::<RcFnBrand, _, _, String>(f);

	assert_eq!(g(Err(10)), Err(11));
	assert_eq!(g(Ok("success".to_string())), Ok("success".to_string()));
}

#[test]
fn test_choice_right() {
	// right: Result<A, C> -> Result<B, C>, transforms the Ok variant
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = right::<RcFnBrand, _, _, String>(f);

	assert_eq!(g(Ok(10)), Ok(11));
	assert_eq!(g(Err("error".to_string())), Err("error".to_string()));
}
