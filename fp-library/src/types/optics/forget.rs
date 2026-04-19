//! The `Forget` profunctor, used for folds and getters.
//!
//! `Forget<P, R, A, B>` wraps a function `A -> R`, ignoring the `B` parameter.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				FnBrand,
				optics::*,
			},
			classes::{
				monoid::Monoid,
				profunctor::{
					Choice,
					Cochoice,
					Profunctor,
					Strong,
					Wander,
				},
				*,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// The `Forget` profunctor.
	///
	/// `Forget<P, R, A, B>` is a profunctor that ignores its second type argument `B`
	/// and instead stores a function from `A` to `R`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand.",
		"The return type of the function.",
		"The input type of the function.",
		"The ignored type."
	)]
	pub struct Forget<'a, PointerBrand, R, A, B>(
		pub Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, R>),
		PhantomData<B>,
	)
	where
		PointerBrand: ToDynCloneFn,
		R: 'a,
		A: 'a;

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand.",
		"The return type of the function.",
		"The input type of the function.",
		"The ignored type."
	)]
	#[document_parameters("The forget instance.")]
	impl<'a, PointerBrand, R, A, B> Forget<'a, PointerBrand, R, A, B>
	where
		PointerBrand: ToDynCloneFn,
		R: 'a,
		A: 'a,
	{
		/// Creates a new `Forget` instance.
		#[document_signature]
		///
		#[document_parameters("The function to wrap.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Forget,
		/// };
		///
		/// let forget = Forget::<RcBrand, i32, String, i32>::new(|s: String| s.len() as i32);
		/// // Access via the underlying function wrapper, which implements Deref
		/// assert_eq!((forget.0)("hello".to_string()), 5);
		/// ```
		pub fn new(f: impl Fn(A) -> R + 'a) -> Self {
			Forget(<FnBrand<PointerBrand> as LiftFn>::new(f), PhantomData)
		}

		/// Runs the `Forget` profunctor on an input.
		#[document_signature]
		#[document_parameters("The input value.")]
		///
		#[document_returns("The result of applying the underlying function to the input.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Forget,
		/// };
		///
		/// let forget = Forget::<RcBrand, i32, String, i32>::new(|s: String| s.len() as i32);
		/// assert_eq!(forget.run("hello".to_string()), 5);
		/// ```
		pub fn run(
			&self,
			a: A,
		) -> R {
			(self.0)(a)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand.",
		"The return type of the function.",
		"The input type of the function.",
		"The ignored type."
	)]
	#[document_parameters("The forget instance.")]
	impl<'a, PointerBrand, R, A, B> Clone for Forget<'a, PointerBrand, R, A, B>
	where
		PointerBrand: ToDynCloneFn,
		R: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Forget` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Forget,
		/// };
		///
		/// let forget = Forget::<RcBrand, i32, String, i32>::new(|s: String| s.len() as i32);
		/// let cloned = forget.clone();
		/// assert_eq!(cloned.run("hello".to_string()), 5);
		/// ```
		fn clone(&self) -> Self {
			Forget(self.0.clone(), PhantomData)
		}
	}

	impl_kind! {
		impl<PointerBrand: ToDynCloneFn + 'static, R: 'static> for ForgetBrand<PointerBrand, R> {
			#[document_default]
			type Of<'a, A: 'a, B: 'a>: 'a = Forget<'a, PointerBrand, R, A, B>;
		}
	}

	#[document_type_parameters("The pointer brand.", "The return type of the function.")]
	impl<PointerBrand: ToDynCloneFn + 'static, R: 'static> Profunctor for ForgetBrand<PointerBrand, R> {
		/// Maps functions over the input and output of the `Forget` profunctor.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the new structure.",
			"The target type of the new structure.",
			"The source type of the original structure.",
			"The target type of the original structure."
		)]
		///
		#[document_parameters(
			"The function to apply to the input.",
			"The function to apply to the output.",
			"The forget instance to transform."
		)]
		#[document_returns("A transformed `Forget` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let forget: Forget<RcBrand, usize, String, usize> = Forget::new(|s: String| s.len());
		///
		/// let transformed = <ForgetBrand<RcBrand, usize> as Profunctor>::dimap(
		/// 	|s: &str| s.to_string(),
		/// 	|s: usize| s,
		/// 	forget,
		/// );
		/// assert_eq!(transformed.run("hello"), 5);
		/// ```
		fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			ab: impl Fn(A) -> B + 'a,
			_cd: impl Fn(C) -> D + 'a,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>) {
			Forget::new(move |a| (pbc.0)(ab(a)))
		}
	}

	#[document_type_parameters("The pointer brand.", "The return type of the function.")]
	impl<PointerBrand: ToDynCloneFn + 'static, R: 'static> Strong for ForgetBrand<PointerBrand, R> {
		/// Lifts the `Forget` profunctor to operate on the first component of a tuple.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The type of the first component.",
			"The type of the second component.",
			"The target type of the first component."
		)]
		///
		#[document_parameters("The forget instance to transform.")]
		#[document_returns("A transformed `Forget` instance that operates on tuples.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let forget: Forget<RcBrand, usize, String, usize> = Forget::new(|s: String| s.len());
		///
		/// let transformed = <ForgetBrand<RcBrand, usize> as Strong>::first::<String, usize, i32>(forget);
		/// assert_eq!(transformed.run(("hello".to_string(), 42)), 5);
		/// ```
		fn first<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
			Forget::new(move |(a, _)| (pab.0)(a))
		}
	}

	#[document_type_parameters("The pointer brand.", "The return type of the function.")]
	impl<PointerBrand: ToDynCloneFn + 'static, R: 'static + Monoid + Clone> Wander
		for ForgetBrand<PointerBrand, R>
	{
		/// Lifts the `Forget` profunctor to operate on a structure using a traversal.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The source type of the focus.",
			"The target type of the focus."
		)]
		///
		#[document_parameters("The traversal function.", "The forget instance to transform.")]
		#[document_returns("A transformed `Forget` instance that operates on structures.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let forget: Forget<RcBrand, String, String, String> = Forget::new(|x: String| x);
		///
		/// // We use a manual implementation for the example to avoid complex trait bounds
		/// let transformed = Forget::<RcBrand, String, Vec<String>, Vec<String>>::new(|v| v.join(""));
		/// assert_eq!(transformed.run(vec!["a".to_string(), "b".to_string()]), "ab".to_string());
		/// ```
		fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone>(
			traversal: impl crate::classes::optics::traversal::TraversalFunc<'a, S, T, A, B> + 'a,
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>) {
			use crate::brands::ConstBrand;
			Forget::new(move |s| {
				let pab = pab.clone();
				(traversal.apply::<ConstBrand<R>>(
					Box::new(move |a| crate::types::const_val::Const::new((pab.0)(a))),
					s,
				))
				.0
			})
		}
	}

	#[document_type_parameters("The pointer brand.", "The return type of the function.")]
	impl<PointerBrand: ToDynCloneFn + 'static, R: 'static + Monoid> Choice
		for ForgetBrand<PointerBrand, R>
	{
		/// Lifts the `Forget` profunctor to operate on the left component of a `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The type of the left component.",
			"The type of the target left component.",
			"The type of the right component."
		)]
		///
		#[document_parameters("The forget instance to transform.")]
		#[document_returns("A transformed `Forget` instance that operates on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let forget: Forget<RcBrand, String, String, String> = Forget::new(|x: String| x);
		///
		/// let transformed =
		/// 	<ForgetBrand<RcBrand, String> as Choice>::left::<String, String, String>(forget);
		/// assert_eq!(transformed.run(Err("hello".to_string())), "hello".to_string());
		/// assert_eq!(transformed.run(Ok("world".to_string())), "".to_string());
		/// ```
		fn left<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		{
			Forget::new(move |r| match r {
				Err(a) => (pab.0)(a),
				Ok(_) => R::empty(),
			})
		}

		/// Lifts the `Forget` profunctor to operate on the right component of a `Result`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The type of the left component.",
			"The type of the right component.",
			"The target type of the right component."
		)]
		///
		#[document_parameters("The forget instance to transform.")]
		#[document_returns("A transformed `Forget` instance that operates on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let forget: Forget<RcBrand, String, String, String> = Forget::new(|x: String| x);
		///
		/// let transformed =
		/// 	<ForgetBrand<RcBrand, String> as Choice>::right::<String, String, String>(forget);
		/// assert_eq!(transformed.run(Ok("hello".to_string())), "hello".to_string());
		/// assert_eq!(transformed.run(Err("world".to_string())), "".to_string());
		/// ```
		fn right<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		{
			Forget::new(move |r| match r {
				Ok(a) => (pab.0)(a),
				Err(_) => R::empty(),
			})
		}
	}

	#[document_type_parameters("The pointer brand.", "The return type of the function.")]
	impl<PointerBrand: ToDynCloneFn + 'static, R: 'static> Cochoice for ForgetBrand<PointerBrand, R> {
		/// Extracts a `Forget` profunctor from one operating on the left (Err) variant of a `Result`.
		///
		/// Given a `Forget` that operates on `Result<C, A>`, produces a `Forget` that operates
		/// on `A` by wrapping the input in `Err` before applying the original function.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting profunctor.",
			"The output type of the resulting profunctor.",
			"The type of the alternative (Ok) variant."
		)]
		///
		#[document_parameters("The forget instance to extract from.")]
		#[document_returns("A `Forget` profunctor operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let forget: Forget<RcBrand, String, Result<i32, String>, Result<i32, String>> =
		/// 	Forget::new(|r: Result<i32, String>| match r {
		/// 		Err(s) => s,
		/// 		Ok(n) => n.to_string(),
		/// 	});
		///
		/// let extracted =
		/// 	<ForgetBrand<RcBrand, String> as Cochoice>::unleft::<String, String, i32>(forget);
		/// assert_eq!(extracted.run("hello".to_string()), "hello".to_string());
		/// ```
		fn unleft<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			Forget::new(move |a| (pab.0)(Err(a)))
		}

		/// Extracts a `Forget` profunctor from one operating on the right (Ok) variant of a `Result`.
		///
		/// Given a `Forget` that operates on `Result<A, C>`, produces a `Forget` that operates
		/// on `A` by wrapping the input in `Ok` before applying the original function.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting profunctor.",
			"The output type of the resulting profunctor.",
			"The type of the alternative (Err) variant."
		)]
		///
		#[document_parameters("The forget instance to extract from.")]
		#[document_returns("A `Forget` profunctor operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let forget: Forget<RcBrand, String, Result<String, i32>, Result<String, i32>> =
		/// 	Forget::new(|r: Result<String, i32>| match r {
		/// 		Ok(s) => s,
		/// 		Err(n) => n.to_string(),
		/// 	});
		///
		/// let extracted =
		/// 	<ForgetBrand<RcBrand, String> as Cochoice>::unright::<String, String, i32>(forget);
		/// assert_eq!(extracted.run("hello".to_string()), "hello".to_string());
		/// ```
		fn unright<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			Forget::new(move |a| (pab.0)(Ok(a)))
		}
	}
}
pub use inner::*;
