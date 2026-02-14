use fp_library::{brands::*, functions::*, types::optics::*};

#[test]
fn test_lens_optic() {
	#[derive(Clone, Debug, PartialEq)]
	struct Person {
		name: String,
		age: i32,
	}

	let age_lens: LensPrime<RcBrand, Person, i32> =
		LensPrime::new(|p: Person| p.age, |(p, age)| Person { age, ..p });

	let person = Person { name: "Alice".to_string(), age: 30 };

	// To use 'evaluate', we must provide a concrete Profunctor implementation.
	// `RcFnBrand` provides a Strong + Choice Profunctor for reference-counted closures.
	//
	// Profunctor encoding of a Lens:
	//   Lens S A = forall p. Strong p => p A A -> p S S
	//
	// When p is a Function (->):
	//   evaluate :: (A -> A) -> (S -> S)
	//
	// Passing a modification function `f: A -> A` returns a function `S -> S`
	// that applies `f` to the focused field.

	let modify_age = |x: i32| x + 1;

	// Wrap the closure in the Profunctor (RcFnBrand)
	let p_modify = cloneable_fn_new::<RcFnBrand, _, _>(modify_age);

	// Evaluate the optic to get the modifier function
	let modifier = age_lens.evaluate::<RcFnBrand>(p_modify);

	let updated = modifier(person.clone());
	assert_eq!(updated.age, 31);
	assert_eq!(updated.name, "Alice");
}

#[test]
fn test_composition() {
	#[derive(Clone, Debug, PartialEq)]
	struct Inner {
		val: i32,
	}

	#[derive(Clone, Debug, PartialEq)]
	struct Outer {
		inner: Inner,
	}

	let outer_lens: LensPrime<RcBrand, Outer, Inner> =
		LensPrime::new(|o: Outer| o.inner.clone(), |(_, i)| Outer { inner: i });
	let inner_lens: LensPrime<RcBrand, Inner, i32> =
		LensPrime::new(|i: Inner| i.val, |(_, v)| Inner { val: v });

	// Compose: Outer -> Inner -> i32
	// O1: Lens<Outer, Inner>
	// O2: Lens<Inner, i32>
	let composed = optics_compose(outer_lens, inner_lens);

	let obj = Outer { inner: Inner { val: 10 } };

	let modify_val = |x: i32| x * 2;
	let p_modify = cloneable_fn_new::<RcFnBrand, _, _>(modify_val);

	let modifier = composed.evaluate::<RcFnBrand>(p_modify);

	let result = modifier(obj);
	assert_eq!(result.inner.val, 20);
}

#[test]
fn test_profunctor_dimap() {
	// Test that functions are profunctors
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = dimap::<RcFnBrand, _, _, _, _, _, _>(|x: i32| x * 2, |x: i32| x - 1, f);

	assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
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
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = left::<RcFnBrand, _, _, String>(f);

	// `left` lifts a profunctor transformation `p a b` to `p (Result a c) (Result b c)`.
	// Following the PureScript/Haskell convention for Choice:
	//   left  :: p a b -> p (Either a c) (Either b c)
	//   Err(a) -> Err(f(a))
	//   Ok(c)  -> Ok(c)

	assert_eq!(g(Err(10)), Err(11));
	assert_eq!(g(Ok("success".to_string())), Ok("success".to_string()));
}

#[test]
fn test_choice_right() {
	let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	let g = right::<RcFnBrand, _, _, String>(f);

	// `right` lifts a profunctor transformation `p a b` to `p (Result c a) (Result c b)`.
	//   right :: p a b -> p (Either c a) (Either c b)
	//   Ok(a)  -> Ok(f(a))
	//   Err(c) -> Err(c)

	assert_eq!(g(Ok(10)), Ok(11));
	assert_eq!(g(Err("error".to_string())), Err("error".to_string()));
}

#[test]
fn test_lens_polymorphic() {
	#[derive(Clone, Debug, PartialEq)]
	struct Poly<A> {
		val: A,
	}

	// Lens that changes Poly<i32> to Poly<String>
	let l: Lens<RcBrand, Poly<i32>, Poly<String>, i32, String> =
		Lens::new(|p: Poly<i32>| p.val, |(_, s)| Poly { val: s });

	let p = Poly { val: 42 };
	assert_eq!(l.view(p.clone()), 42);

	let p2 = l.set(p, "hello".to_string());
	assert_eq!(p2.val, "hello".to_string());
}

#[test]
fn test_lens_prime_over() {
	let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
	assert_eq!(l.over(10, |x| x + 5), 15);
}

#[test]
fn test_composed_deep() {
	#[derive(Clone, Debug, PartialEq)]
	struct C {
		val: i32,
	}
	#[derive(Clone, Debug, PartialEq)]
	struct B {
		c: C,
	}
	#[derive(Clone, Debug, PartialEq)]
	struct A {
		b: B,
	}

	let a_b: LensPrime<RcBrand, A, B> = LensPrime::new(|a: A| a.b.clone(), |(_, b)| A { b });
	let b_c: LensPrime<RcBrand, B, C> = LensPrime::new(|b: B| b.c.clone(), |(_, c)| B { c });
	let c_val: LensPrime<RcBrand, C, i32> = LensPrime::new(|c: C| c.val, |(_, val)| C { val });

	let a_c = optics_compose(a_b, b_c);
	let a_val = optics_compose(a_c, c_val);

	let obj = A { b: B { c: C { val: 1 } } };

	// Composed optics don't have .view()/.set() directly, but can be used via evaluate
	let modifier =
		a_val.evaluate::<RcFnBrand>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 10));
	let result = modifier(obj.clone());
	assert_eq!(result.b.c.val, 11);

	// We can also use evaluate with Category::identity to just view (using Forget profunctor would be better but we don't have it here)
	// For now, let's just test that evaluate works on the composed optic.
}
