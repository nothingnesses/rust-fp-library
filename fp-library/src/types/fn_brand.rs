//! Reference-counted cloneable function wrappers with [`Semigroupoid`](crate::classes::Semigroupoid) and [`Category`](crate::classes::Category) instances.
//!
//! Provides the [`FnBrand`](crate::brands::FnBrand) abstraction for wrapping closures in `Rc<dyn Fn>` or `Arc<dyn Fn>` for use in higher-kinded contexts.
//!
//! ### Hierarchy Unification
//!
//! `FnBrand` uses [`Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)`](crate::kinds::Kind_266801a817966495), which enforces
//! that input and output types outlive the function wrapper's lifetime. This allows `FnBrand` to
//! be used consistently across the unified profunctor and arrow hierarchies, while supporting
//! non-static types where the lifetimes are correctly tracked.
//!
//! ### Notes
//!
//! `FnBrand` does **not** implement `Cochoice` or `Costrong`:
//!
//! **`Cochoice`**: `unleft` would need to extract `A -> B` from `Result<C, A> -> Result<C, B>`. In Rust's `Result`, the second type parameter is `Err` (Failure), which semantically maps to the `Left` side of `Either` in this library's conventions. Implementing this is unsound for arbitrary functions because strict functions can inspect the `Ok(C)` (Right) variant or return `Ok(C)` even when given `Err(A)` (Left), violating the profunctor morphism structure required to extract the `A -> B` function.
//! **`Costrong`**: Implementing `unfirst` (`((a, c) -> (b, c)) -> (a -> b)`) is unsafe in a strict language like Rust. It requires a fixed-point iteration where the output `c` is fed back as input `c`. Since functions are strict, this would require reading uninitialized memory (UB) or non-termination if implemented naively. While lazy types like `Trampoline` can support this pattern manually, a generic `Costrong` instance cannot be safely provided for `Fn(A) -> B`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				profunctor::*,
				*,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	impl_kind! {
		impl<P: UnsizedCoercible> for FnBrand<P> {
			type Of<'a, A: 'a, B: 'a>: 'a = <P as RefCountedPointer>::CloneableOf<'a, dyn 'a + Fn(A) -> B>;
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Function for FnBrand<P> {
		type Of<'a, A: 'a, B: 'a> =
			Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>);

		/// Creates a new function wrapper.
		///
		/// This function wraps the provided closure `f` into a pointer-wrapped function.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the function and its captured data.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to wrap.", "The input value.")]
		///
		#[document_returns("The wrapped function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(5), 10);
		/// ```
		fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as Function>::Of<'a, A, B> {
			P::coerce_fn(f)
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> CloneableFn for FnBrand<P> {
		type Of<'a, A: 'a, B: 'a> =
			Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>);
		type PointerBrand = P;
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> LiftFn for FnBrand<P> {
		/// Creates a new cloneable function wrapper.
		///
		/// This function wraps the provided closure `f` into a pointer-wrapped cloneable function.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the function and its captured data.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to wrap.", "The input value.")]
		///
		#[document_returns("The wrapped cloneable function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(5), 10);
		/// ```
		fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as CloneableFn>::Of<'a, A, B> {
			P::coerce_fn(f)
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Semigroupoid for FnBrand<P> {
		/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
		///
		/// This method composes two pointer-wrapped functions `f` and `g` to produce a new function that represents the application of `g` followed by `f`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the morphisms.",
			"The source type of the first morphism.",
			"The target type of the first morphism and the source type of the second morphism.",
			"The target type of the second morphism."
		)]
		///
		#[document_parameters(
			"The second morphism to apply (from C to D).",
			"The first morphism to apply (from B to C)."
		)]
		///
		#[document_returns("The composed morphism (from B to D).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let g = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
		/// assert_eq!(h(5), 12); // (5 + 1) * 2
		/// ```
		fn compose<'a, B: 'a, C: 'a, D: 'a>(
			f: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, C, D>),
			g: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, D>) {
			P::coerce_fn(move |b| f(g(b)))
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Category for FnBrand<P> {
		/// Returns the identity morphism.
		///
		/// The identity morphism is a function that maps every object to itself, wrapped in the pointer type.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the morphism.", "The type of the object.")]
		///
		#[document_returns("The identity morphism.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let id = category_identity::<RcFnBrand, i32>();
		/// assert_eq!(id(5), 5);
		/// ```
		fn identity<'a, A>()
		-> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>) {
			P::coerce_fn(|a| a)
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Profunctor for FnBrand<P> {
		/// Maps over both arguments of the profunctor.
		///
		/// This method applies a contravariant function to the input and a covariant
		/// function to the output, transforming the function.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The new input type (contravariant position).",
			"The original input type.",
			"The original output type.",
			"The new output type (covariant position)."
		)]
		///
		#[document_parameters(
			"The contravariant function to apply to the input.",
			"The covariant function to apply to the output.",
			"The profunctor instance."
		)]
		///
		#[document_returns("A new profunctor instance with transformed input and output types.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// };
		///
		/// let f = <RcFnBrand as Profunctor>::dimap(
		/// 	|x: i32| x * 2,
		/// 	|x: i32| x - 1,
		/// 	std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>,
		/// );
		/// assert_eq!(f(10), 20); // (10 * 2) + 1 - 1 = 20
		/// ```
		fn dimap<'a, A, B: 'a, C: 'a, D>(
			ab: impl Fn(A) -> B + 'a,
			cd: impl Fn(C) -> D + 'a,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>) {
			P::coerce_fn(move |a| cd(pbc(ab(a))))
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Strong for FnBrand<P> {
		/// Lift a profunctor to operate on the first component of a pair.
		///
		/// This method takes a function `A -> B` and returns `(A, C) -> (B, C)`,
		/// threading the extra context `C` through unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the function.",
			"The output type of the function.",
			"The type of the second component (threaded through unchanged)."
		)]
		///
		#[document_parameters("The function instance to lift.")]
		///
		#[document_returns("A new function that operates on pairs.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g = <RcFnBrand as Strong>::first::<i32, i32, i32>(f);
		/// assert_eq!(g((10, 20)), (11, 20));
		/// ```
		fn first<'a, A: 'a, B: 'a, C>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
			P::coerce_fn(move |(a, c)| (pab(a), c))
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Choice for FnBrand<P> {
		/// Lift a profunctor to operate on the left (Err) variant of a Result.
		///
		/// This method takes a function `A -> B` and returns `Result<C, A> -> Result<C, B>`,
		/// threading the success context `C` through unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the function.",
			"The output type of the function.",
			"The type of the success variant (threaded through unchanged)."
		)]
		///
		#[document_parameters("The function instance to lift.")]
		///
		#[document_returns("A new function that operates on Result types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g = <RcFnBrand as Choice>::left::<i32, i32, String>(f);
		/// assert_eq!(g(Err(10)), Err(11));
		/// assert_eq!(g(Ok("success".to_string())), Ok("success".to_string()));
		/// ```
		fn left<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		{
			P::coerce_fn(move |r: Result<C, A>| -> Result<C, B> {
				match r {
					Err(a) => Err(pab(a)),
					Ok(c) => Ok(c),
				}
			})
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Closed<FnBrand<P>> for FnBrand<P> {
		/// Lift a profunctor to operate on functions.
		///
		/// This method takes a function `A -> B` and returns `(X -> A) -> (X -> B)`,
		/// where the input and output functions are wrapped in `FnBrand<P>`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type of the profunctor.",
			"The target type of the profunctor.",
			"The input type of the functions."
		)]
		///
		#[document_parameters("The function instance to lift.")]
		///
		#[document_returns("A new function that operates on functions.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g = <RcFnBrand as Closed<RcFnBrand>>::closed::<i32, i32, String>(f);
		/// let h = std::rc::Rc::new(|s: String| s.len() as i32) as std::rc::Rc<dyn Fn(String) -> i32>;
		/// let result = g(h);
		/// assert_eq!(result("hi".to_string()), 3);
		/// ```
		fn closed<'a, A: 'a, B: 'a, X: 'a + Clone>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, <FnBrand<P> as CloneableFn>::Of<'a, X, A>, <FnBrand<P> as CloneableFn>::Of<'a, X, B>>)
		{
			P::coerce_fn(move |f: <FnBrand<P> as CloneableFn>::Of<'a, X, A>| -> <FnBrand<P> as CloneableFn>::Of<'a, X, B> {
				let pab = pab.clone();
				P::coerce_fn(move |x: X| pab(f(x)))
			})
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: UnsizedCoercible> Wander for FnBrand<P> {
		/// Lift a profunctor to operate on a traversable structure.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The source type of the focus.",
			"The target type of the focus."
		)]
		///
		#[document_parameters("The traversal function.", "The profunctor instance.")]
		///
		#[document_returns("A new function that operates on the structure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		Applicative,
		/// 		optics::traversal::TraversalFunc,
		/// 		profunctor::*,
		/// 	},
		/// 	kinds::*,
		/// };
		///
		/// // A traversal over a single value (identity traversal).
		/// struct SingleTraversal;
		/// impl<'a> TraversalFunc<'a, i32, i32, i32, i32> for SingleTraversal {
		/// 	fn apply<M: Applicative>(
		/// 		&self,
		/// 		f: Box<
		/// 			dyn Fn(i32) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, i32>) + 'a,
		/// 		>,
		/// 		s: i32,
		/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, i32>) {
		/// 		f(s)
		/// 	}
		/// }
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g = <RcFnBrand as Wander>::wander::<i32, i32, i32, i32>(SingleTraversal, f);
		/// assert_eq!(g(5), 6);
		/// ```
		fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone>(
			traversal: impl crate::classes::optics::traversal::TraversalFunc<'a, S, T, A, B> + 'a,
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>) {
			P::coerce_fn(move |s| {
				let pab = pab.clone();
				// SAFETY: traversal contract guarantees Some when applying through OptionBrand
				#[allow(clippy::unwrap_used)]
				traversal
					.apply::<crate::brands::OptionBrand>(Box::new(move |a| Some(pab(a))), s)
					.unwrap()
			})
		}
	}

	#[document_type_parameters("The reference-counted pointer type.")]
	impl<P: SendUnsizedCoercible> SendCloneableFn for FnBrand<P> {
		type SendOf<'a, A: 'a, B: 'a> = P::SendOf<'a, dyn 'a + Fn(A) -> B + Send + Sync>;

		/// Creates a new thread-safe cloneable function wrapper.
		///
		/// This function wraps the provided closure `f` into a pointer-wrapped thread-safe cloneable function.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the function and its captured data.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to wrap.")]
		///
		#[document_returns("The wrapped thread-safe cloneable function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(5), 10);
		/// ```
		fn send_cloneable_fn_new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(A) -> B + Send + Sync
		) -> Self::SendOf<'a, A, B> {
			P::coerce_send_fn(f)
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			classes::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Semigroupoid Laws

	/// Tests the associativity law for Semigroupoid.
	#[quickcheck]
	fn semigroupoid_associativity(x: i32) -> bool {
		let f = <RcFnBrand as LiftFn>::new(|x: i32| x.wrapping_add(1));
		let g = <RcFnBrand as LiftFn>::new(|x: i32| x.wrapping_mul(2));
		let h = <RcFnBrand as LiftFn>::new(|x: i32| x.wrapping_sub(3));

		let lhs = RcFnBrand::compose(f.clone(), RcFnBrand::compose(g.clone(), h.clone()));
		let rhs = RcFnBrand::compose(RcFnBrand::compose(f, g), h);

		lhs(x) == rhs(x)
	}

	// Category Laws

	/// Tests the left identity law for Category.
	#[quickcheck]
	fn category_left_identity(x: i32) -> bool {
		let f = <RcFnBrand as LiftFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(id, f.clone());
		let rhs = f;

		lhs(x) == rhs(x)
	}

	/// Tests the right identity law for Category.
	#[quickcheck]
	fn category_right_identity(x: i32) -> bool {
		let f = <RcFnBrand as LiftFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(f.clone(), id);
		let rhs = f;

		lhs(x) == rhs(x)
	}

	// Profunctor Laws

	/// Tests the identity law for Profunctor.
	#[quickcheck]
	fn profunctor_identity(input: i32) -> bool {
		use crate::{
			classes::profunctor::dimap,
			functions::identity,
		};
		let p = std::rc::Rc::new(|x: i32| x.wrapping_mul(3).wrapping_add(7))
			as std::rc::Rc<dyn Fn(i32) -> i32>;
		let result = dimap::<RcFnBrand, _, _, _, _>(identity, identity, p.clone());
		result(input) == p(input)
	}

	/// Tests the composition law for Profunctor.
	#[quickcheck]
	fn profunctor_composition(input: i32) -> bool {
		use crate::{
			classes::profunctor::dimap,
			functions::compose,
		};
		let p = std::rc::Rc::new(|x: i32| x.wrapping_add(1)) as std::rc::Rc<dyn Fn(i32) -> i32>;
		let f1 = |x: i32| x.wrapping_add(10);
		let f2 = |x: i32| x.wrapping_mul(2);
		let g1 = |x: i32| x.wrapping_sub(1);
		let g2 = |x: i32| x.wrapping_mul(3);
		let lhs = dimap::<RcFnBrand, _, _, _, _>(compose(f2, f1), compose(g1, g2), p.clone());
		let rhs = dimap::<RcFnBrand, _, _, _, _>(f1, g1, dimap::<RcFnBrand, _, _, _, _>(f2, g2, p));
		lhs(input) == rhs(input)
	}

	// Contravariant Laws

	/// Tests the identity law for Contravariant.
	#[quickcheck]
	fn contravariant_identity(input: i32) -> bool {
		use crate::{
			classes::contravariant::contramap,
			functions::identity,
		};
		let fa = std::rc::Rc::new(|x: i32| x.wrapping_mul(2).wrapping_add(3))
			as std::rc::Rc<dyn Fn(i32) -> i32>;
		let result =
			contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, i32>, _, _>(identity, fa.clone());
		result(input) == fa(input)
	}

	/// Tests the composition law for Contravariant.
	#[quickcheck]
	fn contravariant_composition(input: i32) -> bool {
		use crate::{
			classes::contravariant::contramap,
			functions::compose,
		};
		type Contra = ProfunctorSecondAppliedBrand<RcFnBrand, i32>;
		let fa = std::rc::Rc::new(|x: i32| x.wrapping_mul(2).wrapping_add(3))
			as std::rc::Rc<dyn Fn(i32) -> i32>;
		let f = |x: i32| x.wrapping_add(10);
		let g = |x: i32| x.wrapping_mul(3);
		let lhs = contramap::<Contra, _, _>(compose(f, g), fa.clone());
		let rhs = contramap::<Contra, _, _>(g, contramap::<Contra, _, _>(f, fa));
		lhs(input) == rhs(input)
	}
}
