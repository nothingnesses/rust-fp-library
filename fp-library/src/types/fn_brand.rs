//! Reference-counted cloneable function wrappers with [`Semigroupoid`](crate::classes::Semigroupoid) and [`Category`](crate::classes::Category) instances.
//!
//! Provides the [`FnBrand`](crate::brands::FnBrand) abstraction for wrapping closures in `Rc<dyn Fn>` or `Arc<dyn Fn>` for use in higher-kinded contexts.
//!
//! ### Hierarchy Unification
//!
//! `FnBrand` Kind implementation has been updated to use [`Kind_266801a817966495`](crate::kinds::Kind_266801a817966495), which enforces
//! that input and output types outlive the function wrapper's lifetime. This change allows
//! `FnBrand` to be used consistently across the unified profunctor and arrow hierarchies, while
//! supporting non-static types where the lifetimes are correctly tracked.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				Category,
				CloneableFn,
				Function,
				RefCountedPointer,
				Semigroupoid,
				SendCloneableFn,
				SendUnsizedCoercible,
				UnsizedCoercible,
				profunctor::{
					Choice,
					Closed,
					Profunctor,
					Strong,
					Wander,
				},
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::document_parameters,
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
		/// ### Returns
		///
		/// The wrapped function.
		///
		/// ### Examples
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
		/// ### Returns
		///
		/// The wrapped cloneable function.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
		/// ### Returns
		///
		/// The composed morphism (from B to D).
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
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
		/// ### Returns
		///
		/// The identity morphism.
		///
		/// ### Examples
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
			"The new output type (covariant position).",
			"The type of the contravariant function.",
			"The type of the covariant function."
		)]
		///
		#[document_parameters(
			"The contravariant function to apply to the input.",
			"The covariant function to apply to the output.",
			"The profunctor instance."
		)]
		///
		/// ### Returns
		///
		/// A new profunctor instance with transformed input and output types.
		///
		/// ### Examples
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
		fn dimap<'a, A, B: 'a, C: 'a, D, FuncAB, FuncCD>(
			ab: FuncAB,
			cd: FuncCD,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>)
		where
			FuncAB: Fn(A) -> B + 'a,
			FuncCD: Fn(C) -> D + 'a, {
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
		/// ### Returns
		///
		/// A new function that operates on pairs.
		///
		/// ### Examples
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
		/// ### Returns
		///
		/// A new function that operates on Result types.
		///
		/// ### Examples
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
		/// ### Returns
		///
		/// A new function that operates on functions.
		///
		/// ### Examples
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
			"The target type of the focus.",
			"The type of the traversal function."
		)]
		///
		#[document_parameters("The traversal function.", "The profunctor instance.")]
		///
		/// ### Returns
		///
		/// A new function that operates on the structure.
		///
		/// ### Examples
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
		/// struct ListTraversal;
		/// impl<'a, A: 'a> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
		/// 	fn apply<M: Applicative>(
		/// 		&self,
		/// 		_f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
		/// 		_s: Vec<A>,
		/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
		/// 		unreachable!()
		/// 	}
		/// }
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let _g = <RcFnBrand as Wander>::wander::<Vec<i32>, Vec<i32>, i32, i32, _>(ListTraversal, f);
		/// ```
		fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a, TFunc>(
			traversal: TFunc,
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		where
			TFunc: crate::classes::optics::traversal::TraversalFunc<'a, S, T, A, B> + 'a, {
			P::coerce_fn(move |s| {
				let pab = pab.clone();
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
		/// ### Returns
		///
		/// The wrapped thread-safe cloneable function.
		///
		/// ### Examples
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
			classes::{
				category::Category,
				cloneable_fn::CloneableFn,
				semigroupoid::Semigroupoid,
			},
		},
		quickcheck_macros::quickcheck,
	};

	// Semigroupoid Laws

	/// Tests the associativity law for Semigroupoid.
	#[quickcheck]
	fn semigroupoid_associativity(x: i32) -> bool {
		let f = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_add(1));
		let g = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_mul(2));
		let h = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_sub(3));

		let lhs = RcFnBrand::compose(f.clone(), RcFnBrand::compose(g.clone(), h.clone()));
		let rhs = RcFnBrand::compose(RcFnBrand::compose(f, g), h);

		lhs(x) == rhs(x)
	}

	// Category Laws

	/// Tests the left identity law for Category.
	#[quickcheck]
	fn category_left_identity(x: i32) -> bool {
		let f = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(id, f.clone());
		let rhs = f;

		lhs(x) == rhs(x)
	}

	/// Tests the right identity law for Category.
	#[quickcheck]
	fn category_right_identity(x: i32) -> bool {
		let f = <RcFnBrand as CloneableFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(f.clone(), id);
		let rhs = f;

		lhs(x) == rhs(x)
	}
}
