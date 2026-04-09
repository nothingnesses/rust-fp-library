//! Validates direct by-reference traverse implementations (Approach A).
//! These avoid cloning the container by iterating via .iter() and using
//! applicative operations to build the result.

use fp_library::{
	brands::*,
	classes::*,
	kinds::*,
	types::*,
};

// -- Vec direct by-reference traverse --

fn vec_ref_traverse_direct<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
	func: impl Fn(&A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, B> + 'a,
	ta: &Vec<A>,
) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, Vec<B>>
where
	Vec<B>: Clone,
	<F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Clone, {
	let len = ta.len();
	ta.iter().fold(
		F::pure::<Vec<B>>(Vec::with_capacity(len)),
		|acc: <F as Kind_cdc7cd43dac7585f>::Of<'a, Vec<B>>, a| {
			F::lift2(
				|mut v: Vec<B>, b: B| {
					v.push(b);
					v
				},
				acc,
				func(a),
			)
		},
	)
}

// -- Option direct by-reference traverse --

fn option_ref_traverse_direct<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
	func: impl Fn(&A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, B> + 'a,
	ta: &Option<A>,
) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, Option<B>>
where
	Option<B>: Clone,
	<F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Clone, {
	match ta {
		Some(a) => F::map(|b| Some(b), func(a)),
		None => F::pure(None),
	}
}

// -- Identity direct by-reference traverse --

fn identity_ref_traverse_direct<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
	func: impl Fn(&A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, B> + 'a,
	ta: &Identity<A>,
) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, Identity<B>>
where
	Identity<B>: Clone,
	<F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Clone, {
	F::map(Identity, func(&ta.0))
}

// -- Result (ErrApplied) direct by-reference traverse --

fn result_err_ref_traverse_direct<
	'a,
	E: Clone + 'a,
	A: 'a + Clone,
	B: 'a + Clone,
	F: Applicative,
>(
	func: impl Fn(&A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, B> + 'a,
	ta: &Result<A, E>,
) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, Result<B, E>>
where
	Result<B, E>: Clone,
	<F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Clone, {
	match ta {
		Ok(a) => F::map(Ok, func(a)),
		Err(e) => F::pure(Err(e.clone())),
	}
}

// -- Pair (FirstApplied) direct by-reference traverse --

fn pair_first_ref_traverse_direct<
	'a,
	First: Clone + 'a,
	A: 'a + Clone,
	B: 'a + Clone,
	F: Applicative,
>(
	func: impl Fn(&A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, B> + 'a,
	ta: &Pair<First, A>,
) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, Pair<First, B>>
where
	Pair<First, B>: Clone,
	<F as Kind_cdc7cd43dac7585f>::Of<'a, B>: Clone, {
	let first = ta.0.clone();
	F::map(move |b| Pair(first.clone(), b), func(&ta.1))
}

#[cfg(test)]
mod tests {
	use super::*;

	// -- Vec tests --

	#[test]
	fn vec_traverse_option_all_some() {
		let v = vec![1, 2, 3];
		let result: Option<Vec<i32>> =
			vec_ref_traverse_direct::<_, _, OptionBrand>(|x: &i32| Some(*x * 2), &v);
		assert_eq!(result, Some(vec![2, 4, 6]));
		assert_eq!(v, vec![1, 2, 3]); // original preserved
	}

	#[test]
	fn vec_traverse_option_short_circuit() {
		let v = vec![1, -1, 3];
		let result: Option<Vec<i32>> = vec_ref_traverse_direct::<_, _, OptionBrand>(
			|x: &i32| if *x > 0 { Some(*x) } else { None },
			&v,
		);
		assert_eq!(result, None);
		assert_eq!(v, vec![1, -1, 3]); // original preserved
	}

	#[test]
	fn vec_traverse_empty() {
		let v: Vec<i32> = vec![];
		let result: Option<Vec<i32>> =
			vec_ref_traverse_direct::<_, _, OptionBrand>(|x: &i32| Some(*x), &v);
		assert_eq!(result, Some(vec![]));
	}

	#[test]
	fn vec_traverse_identity() {
		let v = vec![1, 2, 3];
		let result: Identity<Vec<i32>> =
			vec_ref_traverse_direct::<_, _, IdentityBrand>(|x: &i32| Identity(*x * 2), &v);
		assert_eq!(result, Identity(vec![2, 4, 6]));
	}

	#[test]
	fn vec_traverse_consistency_with_owned() {
		// Direct ref traverse should produce the same result as owned traverse
		let v = vec![1, 2, 3];
		let ref_result: Option<Vec<String>> =
			vec_ref_traverse_direct::<_, _, OptionBrand>(|x: &i32| Some(x.to_string()), &v);
		let owned_result: Option<Vec<String>> =
			VecBrand::traverse::<i32, String, OptionBrand>(|x: i32| Some(x.to_string()), v);
		assert_eq!(ref_result, owned_result);
	}

	// -- Option tests --

	#[test]
	fn option_traverse_some() {
		let v = Some(42);
		let result: Vec<Option<String>> =
			option_ref_traverse_direct::<_, _, VecBrand>(|x: &i32| vec![x.to_string()], &v);
		assert_eq!(result, vec![Some("42".to_string())]);
		assert_eq!(v, Some(42)); // preserved
	}

	#[test]
	fn option_traverse_none() {
		let v: Option<i32> = None;
		let result: Vec<Option<String>> =
			option_ref_traverse_direct::<_, _, VecBrand>(|x: &i32| vec![x.to_string()], &v);
		assert_eq!(result, vec![None]);
	}

	// -- Identity tests --

	#[test]
	fn identity_traverse() {
		let v = Identity(42);
		let result: Option<Identity<String>> =
			identity_ref_traverse_direct::<_, _, OptionBrand>(|x: &i32| Some(x.to_string()), &v);
		assert_eq!(result, Some(Identity("42".to_string())));
	}

	// -- Result tests --

	#[test]
	fn result_traverse_ok() {
		let v: Result<i32, String> = Ok(42);
		let result: Option<Result<String, String>> =
			result_err_ref_traverse_direct::<_, _, _, OptionBrand>(
				|x: &i32| Some(x.to_string()),
				&v,
			);
		assert_eq!(result, Some(Ok("42".to_string())));
	}

	#[test]
	fn result_traverse_err_passthrough() {
		let v: Result<i32, String> = Err("bad".to_string());
		let result: Option<Result<String, String>> =
			result_err_ref_traverse_direct::<_, _, _, OptionBrand>(
				|x: &i32| Some(x.to_string()),
				&v,
			);
		assert_eq!(result, Some(Err("bad".to_string())));
		assert_eq!(v, Err("bad".to_string())); // preserved
	}

	// -- Pair tests --

	#[test]
	fn pair_traverse() {
		let v = Pair("hello", 42);
		let result: Option<Pair<&str, String>> =
			pair_first_ref_traverse_direct::<_, _, _, OptionBrand>(
				|x: &i32| Some(x.to_string()),
				&v,
			);
		assert_eq!(result, Some(Pair("hello", "42".to_string())));
		assert_eq!(v, Pair("hello", 42)); // preserved
	}
}
