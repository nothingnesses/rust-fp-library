//! Single-value tuple with [`Functor`](crate::classes::Functor), [`Applicative`](crate::classes::Applicative), [`Monad`](crate::classes::Monad), [`MonadRec`](crate::classes::MonadRec), [`Foldable`](crate::classes::Foldable), [`Traversable`](crate::classes::Traversable), and parallel folding instances.
//!
//! A trivial wrapper using the native Rust 1-tuple `(A,)`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::Tuple1Brand,
			classes::{
				Applicative,
				ApplyFirst,
				ApplySecond,
				CloneableFn,
				Foldable,
				Functor,
				Lift,
				MonadRec,
				Monoid,
				Pointed,
				Semiapplicative,
				Semimonad,
				Traversable,
			},
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	impl_kind! {
		for Tuple1Brand {
			type Of<A> = (A,);
		}
	}

	impl_kind! {
		for Tuple1Brand {
			type Of<'a, A: 'a>: 'a = (A,);
		}
	}

	impl Functor for Tuple1Brand {
		/// Maps a function over the value in the tuple.
		///
		/// This method applies a function to the value inside the 1-tuple, producing a new 1-tuple with the transformed value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the tuple.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply.", "The tuple to map over.")]
		///
		#[document_returns("A new 1-tuple containing the result of applying the function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (5,);
		/// let y = map::<Tuple1Brand, _, _, _>(|i| i * 2, x);
		/// assert_eq!(y, (10,));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(func(fa.0),)
		}
	}

	impl Lift for Tuple1Brand {
		/// Lifts a binary function into the tuple context.
		///
		/// This method lifts a binary function to operate on values within the 1-tuple context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first tuple's value.",
			"The type of the second tuple's value.",
			"The return type of the function."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first tuple.",
			"The second tuple."
		)]
		///
		#[document_returns("A new 1-tuple containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (1,);
		/// let y = (2,);
		/// let z = lift2::<Tuple1Brand, _, _, _>(|a, b| a + b, x, y);
		/// assert_eq!(z, (3,));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: 'a,
			B: 'a,
			C: 'a, {
			(func(fa.0, fb.0),)
		}
	}

	impl Pointed for Tuple1Brand {
		/// Wraps a value in a 1-tuple.
		///
		/// This method wraps a value in a 1-tuple context.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A 1-tuple containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = pure::<Tuple1Brand, _>(5);
		/// assert_eq!(x, (5,));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			(a,)
		}
	}

	impl ApplyFirst for Tuple1Brand {}
	impl ApplySecond for Tuple1Brand {}

	impl Semiapplicative for Tuple1Brand {
		/// Applies a wrapped function to a wrapped value.
		///
		/// This method applies a function wrapped in a 1-tuple to a value wrapped in a 1-tuple.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The tuple containing the function.",
			"The tuple containing the value."
		)]
		///
		#[document_returns("A new 1-tuple containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = (cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2),);
		/// let x = (5,);
		/// let y = apply::<RcFnBrand, Tuple1Brand, _, _>(f, x);
		/// assert_eq!(y, (10,));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(ff.0(fa.0),)
		}
	}

	impl Semimonad for Tuple1Brand {
		/// Chains 1-tuple computations.
		///
		/// This method chains two 1-tuple computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first tuple.",
			"The function to apply to the value inside the tuple."
		)]
		///
		#[document_returns("The result of applying `f` to the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (5,);
		/// let y = bind::<Tuple1Brand, _, _>(x, |i| (i * 2,));
		/// assert_eq!(y, (10,));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			func(ma.0)
		}
	}

	impl Foldable for Tuple1Brand {
		/// Folds the 1-tuple from the right.
		///
		/// This method performs a right-associative fold of the 1-tuple. Since it contains only one element, this is equivalent to applying the function to the element and the initial value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The tuple to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (5,);
		/// let y = fold_right::<RcFnBrand, Tuple1Brand, _, _>(|a, b| a + b, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(fa.0, initial)
		}

		/// Folds the 1-tuple from the left.
		///
		/// This method performs a left-associative fold of the 1-tuple. Since it contains only one element, this is equivalent to applying the function to the initial value and the element.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The tuple to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (5,);
		/// let y = fold_left::<RcFnBrand, Tuple1Brand, _, _>(|b, a| b + a, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(initial, fa.0)
		}

		/// Maps the value to a monoid and returns it.
		///
		/// This method maps the element of the 1-tuple to a monoid.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters(
			"The thread-safe function to map each element to a monoid.",
			"The tuple to fold."
		)]
		///
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (5,);
		/// let y = fold_map::<RcFnBrand, Tuple1Brand, _, _>(|a: i32| a.to_string(), x);
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			func(fa.0)
		}
	}

	impl Traversable for Tuple1Brand {
		/// Traverses the 1-tuple with an applicative function.
		///
		/// This method maps the element of the 1-tuple to a computation, evaluates it, and wraps the result in the applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The tuple to traverse."
		)]
		///
		#[document_returns("The 1-tuple wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (5,);
		/// let y = traverse::<Tuple1Brand, _, _, OptionBrand>(|a| Some(a * 2), x);
		/// assert_eq!(y, Some((10,)));
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			F::map(|b| (b,), func(ta.0))
		}

		/// Sequences a 1-tuple of applicative.
		///
		/// This method evaluates the computation inside the 1-tuple and wraps the result in the applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The tuple containing the applicative value.")]
		///
		#[document_returns("The 1-tuple wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (Some(5),);
		/// let y = sequence::<Tuple1Brand, _, OptionBrand>(x);
		/// assert_eq!(y, Some((5,)));
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			F::map(|a| (a,), ta.0)
		}
	}

	impl MonadRec for Tuple1Brand {
		/// Performs tail-recursive monadic computation over 1-tuples.
		///
		/// Since the 1-tuple has no effect, this simply loops on the inner value
		/// until the step function returns [`ControlFlow::Break`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("A 1-tuple containing the result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<Tuple1Brand, _, _>(
		/// 	|n| {
		/// 		if n < 10 { (ControlFlow::Continue(n + 1),) } else { (ControlFlow::Break(n),) }
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, (10,));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut current = initial;
			loop {
				match func(current).0 {
					ControlFlow::Continue(next) => current = next,
					ControlFlow::Break(b) => return (b,),
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {

	use {
		crate::{
			brands::{
				OptionBrand,
				RcFnBrand,
				Tuple1Brand,
			},
			classes::{
				CloneableFn,
				functor_dispatch::map,
				pointed::pure,
				semiapplicative::apply,
				semimonad::bind,
			},
			functions::{
				compose,
				identity,
			},
		},
		quickcheck_macros::quickcheck,
	};

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		let x = (x,);
		map::<Tuple1Brand, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		let x = (x,);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<Tuple1Brand, _, _, _>(compose(f, g), x)
			== map::<Tuple1Brand, _, _, _>(f, map::<Tuple1Brand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: i32) -> bool {
		let v = (v,);
		apply::<RcFnBrand, Tuple1Brand, _, _>(
			pure::<Tuple1Brand, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, Tuple1Brand, _, _>(
			pure::<Tuple1Brand, _>(<RcFnBrand as CloneableFn>::new(f)),
			pure::<Tuple1Brand, _>(x),
		) == pure::<Tuple1Brand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: i32,
		u_val: i32,
		v_val: i32,
	) -> bool {
		let w = (w,);
		let v_fn = move |x: i32| x.wrapping_mul(v_val);
		let u_fn = move |x: i32| x.wrapping_add(u_val);

		let v = pure::<Tuple1Brand, _>(<RcFnBrand as CloneableFn>::new(v_fn));
		let u = pure::<Tuple1Brand, _>(<RcFnBrand as CloneableFn>::new(u_fn));

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, Tuple1Brand, _, _>(v.clone(), w);
		let rhs = apply::<RcFnBrand, Tuple1Brand, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let composed = move |x| u_fn(v_fn(x));
		let uv = pure::<Tuple1Brand, _>(<RcFnBrand as CloneableFn>::new(composed));

		let lhs = apply::<RcFnBrand, Tuple1Brand, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<Tuple1Brand, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, Tuple1Brand, _, _>(u.clone(), pure::<Tuple1Brand, _>(y));

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, Tuple1Brand, _, _>(pure::<Tuple1Brand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| (x.wrapping_mul(2),);
		bind::<Tuple1Brand, _, _>(pure::<Tuple1Brand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: i32) -> bool {
		let m = (m,);
		bind::<Tuple1Brand, _, _>(m, pure::<Tuple1Brand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: i32) -> bool {
		let m = (m,);
		let f = |x: i32| (x.wrapping_mul(2),);
		let g = |x: i32| (x.wrapping_add(1),);
		bind::<Tuple1Brand, _, _>(bind::<Tuple1Brand, _, _>(m, f), g)
			== bind::<Tuple1Brand, _, _>(m, |x| bind::<Tuple1Brand, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests the `map` function.
	#[test]
	fn map_test() {
		assert_eq!(map::<Tuple1Brand, _, _, _>(|x: i32| x + 1, (1,)), (2,));
	}

	/// Tests the `bind` function.
	#[test]
	fn bind_test() {
		assert_eq!(bind::<Tuple1Brand, _, _>((1,), |x| (x + 1,)), (2,));
	}

	/// Tests the `fold_right` function.
	#[test]
	fn fold_right_test() {
		assert_eq!(
			crate::classes::foldable::fold_right::<RcFnBrand, Tuple1Brand, _, _>(
				|x: i32, acc| x + acc,
				0,
				(1,)
			),
			1
		);
	}

	/// Tests the `fold_left` function.
	#[test]
	fn fold_left_test() {
		assert_eq!(
			crate::classes::foldable::fold_left::<RcFnBrand, Tuple1Brand, _, _>(
				|acc, x: i32| acc + x,
				0,
				(1,)
			),
			1
		);
	}

	/// Tests the `traverse` function.
	#[test]
	fn traverse_test() {
		assert_eq!(
			crate::classes::traversable::traverse::<Tuple1Brand, _, _, OptionBrand>(
				|x: i32| Some(x + 1),
				(1,)
			),
			Some((2,))
		);
	}

	// MonadRec tests

	/// Tests the MonadRec identity law: `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		tail_rec_m::<Tuple1Brand, _, _>(|a| (ControlFlow::Break(a),), x) == (x,)
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m`.
	#[test]
	fn monad_rec_sum_range() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result = tail_rec_m::<Tuple1Brand, _, _>(
			|(n, acc)| {
				if n == 0 {
					(ControlFlow::Break(acc),)
				} else {
					(ControlFlow::Continue((n - 1, acc + n)),)
				}
			},
			(100i64, 0i64),
		);
		assert_eq!(result, (5050,));
	}

	/// Tests stack safety: `tail_rec_m` handles large iteration counts.
	#[test]
	fn monad_rec_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<Tuple1Brand, _, _>(
			|acc| {
				if acc < iterations {
					(ControlFlow::Continue(acc + 1),)
				} else {
					(ControlFlow::Break(acc),)
				}
			},
			0i64,
		);
		assert_eq!(result, (iterations,));
	}
}
