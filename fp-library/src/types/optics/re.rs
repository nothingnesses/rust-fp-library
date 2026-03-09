//! The `Re` profunctor, for reversing optic constraints.
//!
//! `Re<'a, InnerP, OuterP, S, T, A, B>` wraps a function `InnerP::Of<'a, B, A> -> InnerP::Of<'a, T, S>`.
//! It "reverses" the profunctor structure of `InnerP`:
//!
//! - `InnerP: Profunctor` → `ReBrand<InnerP, OuterP, S, T>: Profunctor`
//! - `InnerP: Choice` → `ReBrand<InnerP, OuterP, S, T>: Cochoice`
//! - `InnerP: Cochoice` → `ReBrand<InnerP, OuterP, S, T>: Choice`
//! - `InnerP: Strong` → `ReBrand<InnerP, OuterP, S, T>: Costrong`
//! - `InnerP: Costrong` → `ReBrand<InnerP, OuterP, S, T>: Strong`
//!
//! This is a port of PureScript's [`Data.Lens.Internal.Re`](https://pursuit.purescript.org/packages/purescript-profunctor-lenses/docs/Data.Lens.Internal.Re).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				Monoid,
				UnsizedCoercible,
				optics::{
					AffineTraversalOptic,
					FoldOptic,
					GetterOptic,
					PrismOptic,
					ReviewOptic,
				},
				profunctor::{
					Choice,
					Cochoice,
					Costrong,
					Profunctor,
					Strong,
				},
			},
			impl_kind,
			kinds::*,
			types::optics::{
				forget::ForgetBrand,
				tagged::TaggedBrand,
			},
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// The `Re` profunctor.
	///
	/// Wraps a function `InnerP::Of<'a, B, A> -> InnerP::Of<'a, T, S>`, reversing
	/// the role of the inner profunctor's type arguments.
	///
	/// Corresponds to PureScript's `newtype Re p s t a b = Re (p b a -> p t s)`.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The inner profunctor brand whose instances are reversed.",
		"The outer cloneable function pointer brand for wrapping the `run` function.",
		"The fixed source type (outer structure, contravariant).",
		"The fixed target type (outer structure, covariant).",
		"The varying input type (contravariant position).",
		"The varying output type (covariant position)."
	)]
	pub struct Re<
		'a,
		InnerP: Profunctor,
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	> {
		/// The wrapped function `InnerP::Of<B, A> -> InnerP::Of<T, S>`.
		pub run: <FnBrand<PointerBrand> as CloneableFn>::Of<
			'a,
			Apply!(<InnerP as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, B, A>),
			Apply!(<InnerP as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, T, S>),
		>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The inner profunctor brand.",
		"The outer cloneable function pointer brand.",
		"The fixed source type.",
		"The fixed target type.",
		"The varying input type.",
		"The varying output type."
	)]
	impl<'a, InnerP: Profunctor, PointerBrand: UnsizedCoercible, S: 'a, T: 'a, A: 'a, B: 'a>
		Re<'a, InnerP, PointerBrand, S, T, A, B>
	{
		/// Creates a new `Re` instance by wrapping a function.
		#[document_signature]
		///
		#[document_parameters("The function `InnerP::Of<B, A> -> InnerP::Of<T, S>` to wrap.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	types::optics::{
		/// 		Re,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // Re wraps a function from `Tagged<B, A>` to `Tagged<T, S>`.
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|tagged: Tagged<i32, i32>| {
		/// 	Tagged::new(tagged.0 + 1)
		/// });
		/// assert_eq!((re.run)(Tagged::new(41)).0, 42);
		/// ```
		pub fn new(
			f: impl 'a
			+ Fn(
				Apply!(<InnerP as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, B, A>),
			) -> Apply!(<InnerP as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, T, S>)
		) -> Self {
			Re {
				run: <FnBrand<PointerBrand> as CloneableFn>::new(f),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The inner profunctor brand.",
		"The outer cloneable function pointer brand.",
		"The fixed source type.",
		"The fixed target type.",
		"The varying input type.",
		"The varying output type."
	)]
	#[document_parameters("The `Re` instance.")]
	impl<'a, InnerP: Profunctor, PointerBrand: UnsizedCoercible, S: 'a, T: 'a, A: 'a, B: 'a> Clone
		for Re<'a, InnerP, PointerBrand, S, T, A, B>
	{
		#[document_signature]
		#[document_returns("A new `Re` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	types::optics::{
		/// 		Re,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// let re =
		/// 	Re::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| Tagged::new(t.0));
		/// let cloned = re.clone();
		/// assert_eq!((cloned.run)(Tagged::new(42)).0, 42);
		/// ```
		fn clone(&self) -> Self {
			Re {
				run: self.run.clone(),
			}
		}
	}

	/// Brand for the `Re` profunctor.
	///
	/// `ReBrand<InnerP, OuterP, S, T>` fixes the inner profunctor `InnerP` and the outer
	/// types `S` and `T`, leaving `A` and `B` free for kind application.
	#[document_type_parameters(
		"The inner profunctor brand whose instances are reversed.",
		"The outer cloneable function pointer brand for wrapping the `run` function.",
		"The fixed source type.",
		"The fixed target type."
	)]
	pub struct ReBrand<InnerP, PointerBrand, S, T>(PhantomData<(InnerP, PointerBrand, S, T)>);

	impl_kind! {
		impl<
			InnerP: Profunctor + 'static,
			PointerBrand: UnsizedCoercible + 'static,
			S: 'static,
			T: 'static,
		> for ReBrand<InnerP, PointerBrand, S, T> {
			#[document_default]
			type Of<'a, A: 'a, B: 'a>: 'a = Re<'a, InnerP, PointerBrand, S, T, A, B>;
		}
	}

	/// `Profunctor` instance for `ReBrand<InnerP, OuterP, S, T>` whenever `InnerP: Profunctor`.
	///
	/// Corresponds to:
	/// ```purescript
	/// instance profunctorRe :: Profunctor p => Profunctor (Re p s t) where
	///   dimap f g (Re r) = Re (r <<< dimap g f)
	/// ```
	#[document_type_parameters(
		"The inner profunctor brand.",
		"The outer cloneable function pointer brand.",
		"The fixed source type.",
		"The fixed target type."
	)]
	impl<
		InnerP: Profunctor + 'static,
		PointerBrand: UnsizedCoercible + 'static,
		S: 'static,
		T: 'static,
	> Profunctor for ReBrand<InnerP, PointerBrand, S, T>
	{
		/// Maps over both arguments of `Re`, swapping the roles of `f` and `g` on the inner profunctor.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The new contravariant type.",
			"The original contravariant type.",
			"The original covariant type.",
			"The new covariant type.",
			"The type of the contravariant function.",
			"The type of the covariant function."
		)]
		///
		#[document_parameters(
			"The contravariant function `A -> B`.",
			"The covariant function `C -> D`.",
			"The `Re` instance to transform."
		)]
		#[document_returns("A transformed `Re` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::profunctor::Profunctor,
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // re.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|tagged: Tagged<i32, i32>| {
		/// 	Tagged::new(tagged.0)
		/// });
		/// // dimap(ab=|x| x*2, cd=|x| x+1, re).run(Tagged(5))
		/// //   = re.run(TaggedBrand::dimap(cd, ab, Tagged(5)))
		/// //   = re.run(Tagged(ab(5))) = re.run(Tagged(10)) = Tagged(10)
		/// let transformed = <ReBrand<TaggedBrand, RcBrand, i32, i32> as Profunctor>::dimap(
		/// 	|x: i32| x * 2,
		/// 	|x: i32| x + 1,
		/// 	re,
		/// );
		/// assert_eq!((transformed.run)(Tagged::new(5)).0, 10);
		/// ```
		fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, FuncAB, FuncCD>(
			ab: FuncAB,
			cd: FuncCD,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>)
		where
			FuncAB: Fn(A) -> B + 'a,
			FuncCD: Fn(C) -> D + 'a, {
			let r = pbc.run;
			let ab = <FnBrand<PointerBrand> as CloneableFn>::new(ab);
			let cd = <FnBrand<PointerBrand> as CloneableFn>::new(cd);
			Re::new(move |pda| {
				let ab = ab.clone();
				let cd = cd.clone();
				(*r)(InnerP::dimap(move |c| (*cd)(c), move |a| (*ab)(a), pda))
			})
		}
	}

	/// `Cochoice` instance for `ReBrand<InnerP, OuterP, S, T>` whenever `InnerP: Choice`.
	///
	/// Corresponds to:
	/// ```purescript
	/// instance choiceRe :: Choice p => Cochoice (Re p s t) where
	///   unleft  (Re r) = Re (r <<< left)
	///   unright (Re r) = Re (r <<< right)
	/// ```
	#[document_type_parameters(
		"The inner `Choice` profunctor brand.",
		"The outer cloneable function pointer brand.",
		"The fixed source type.",
		"The fixed target type."
	)]
	impl<InnerP: Choice + 'static, PointerBrand: UnsizedCoercible + 'static, S: 'static, T: 'static>
		Cochoice for ReBrand<InnerP, PointerBrand, S, T>
	{
		/// Extracts from a `Re` that operates on `Result` types using `InnerP::left`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Re`.",
			"The output type of the resulting `Re`.",
			"The type of the `Ok` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance operating on `Result` types.")]
		///
		#[document_returns("A `Re` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::profunctor::Cochoice,
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // re.run: Tagged<Result<String, i32>, Result<String, i32>> -> Tagged<i32, i32>
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, Result<String, i32>, Result<String, i32>>::new(
		/// 	|t: Tagged<Result<String, i32>, Result<String, i32>>| Tagged::new(t.0.unwrap_err() + 1),
		/// );
		/// // unleft(re).run(Tagged(41)) = re.run(TaggedBrand::left(Tagged(41)))
		/// //   = re.run(Tagged(Err(41))) = Tagged(42)
		/// let result =
		/// 	<ReBrand<TaggedBrand, RcBrand, i32, i32> as Cochoice>::unleft::<i32, i32, String>(re);
		/// assert_eq!((result.run)(Tagged::new(41)).0, 42);
		/// ```
		fn unleft<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Re::new(move |pba| (*r)(InnerP::left(pba)))
		}

		/// Extracts from a `Re` that operates on `Result` types using `InnerP::right`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Re`.",
			"The output type of the resulting `Re`.",
			"The type of the `Err` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance operating on `Result` types.")]
		///
		#[document_returns("A `Re` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::profunctor::Cochoice,
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // re.run: Tagged<Result<i32, String>, Result<i32, String>> -> Tagged<i32, i32>
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, Result<i32, String>, Result<i32, String>>::new(
		/// 	|t: Tagged<Result<i32, String>, Result<i32, String>>| Tagged::new(t.0.unwrap() + 1),
		/// );
		/// // unright(re).run(Tagged(41)) = re.run(TaggedBrand::right(Tagged(41)))
		/// //   = re.run(Tagged(Ok(41))) = Tagged(42)
		/// let result =
		/// 	<ReBrand<TaggedBrand, RcBrand, i32, i32> as Cochoice>::unright::<i32, i32, String>(re);
		/// assert_eq!((result.run)(Tagged::new(41)).0, 42);
		/// ```
		fn unright<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Re::new(move |pba| (*r)(InnerP::right(pba)))
		}
	}

	/// `Choice` instance for `ReBrand<InnerP, OuterP, S, T>` whenever `InnerP: Cochoice`.
	///
	/// Corresponds to:
	/// ```purescript
	/// instance cochoiceRe :: Cochoice p => Choice (Re p s t) where
	///   left  (Re r) = Re (r <<< unleft)
	///   right (Re r) = Re (r <<< unright)
	/// ```
	#[document_type_parameters(
		"The inner `Cochoice` profunctor brand.",
		"The outer cloneable function pointer brand.",
		"The fixed source type.",
		"The fixed target type."
	)]
	impl<
		InnerP: Cochoice + 'static,
		PointerBrand: UnsizedCoercible + 'static,
		S: 'static,
		T: 'static,
	> Choice for ReBrand<InnerP, PointerBrand, S, T>
	{
		/// Lifts `Re` to operate on the `Err` variant of a `Result` using `InnerP::unleft`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the `Ok` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance to lift.")]
		///
		#[document_returns("A `Re` instance operating on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::profunctor::Choice,
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // re.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| {
		/// 	Tagged::new(t.0 + 1)
		/// });
		/// // left(re).run(Tagged(Err(41))) = re.run(TaggedBrand::unleft(Tagged(Err(41))))
		/// //   = re.run(Tagged(41)) = Tagged(42)
		/// let result = <ReBrand<TaggedBrand, RcBrand, i32, i32> as Choice>::left::<i32, i32, String>(re);
		/// assert_eq!(
		/// 	(result.run)(Tagged::<Result<String, i32>, Result<String, i32>>::new(Err(41))).0,
		/// 	42
		/// );
		/// ```
		fn left<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		{
			let r = pab.run;
			Re::new(move |p| (*r)(InnerP::unleft(p)))
		}

		/// Lifts `Re` to operate on the `Ok` variant of a `Result` using `InnerP::unright`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the `Err` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance to lift.")]
		///
		#[document_returns("A `Re` instance operating on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::profunctor::Choice,
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // re.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| {
		/// 	Tagged::new(t.0 + 1)
		/// });
		/// // right(re).run(Tagged(Ok(41))) = re.run(TaggedBrand::unright(Tagged(Ok(41))))
		/// //   = re.run(Tagged(41)) = Tagged(42)
		/// let result = <ReBrand<TaggedBrand, RcBrand, i32, i32> as Choice>::right::<i32, i32, String>(re);
		/// assert_eq!((result.run)(Tagged::<Result<i32, String>, Result<i32, String>>::new(Ok(41))).0, 42);
		/// ```
		fn right<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		{
			let r = pab.run;
			Re::new(move |p| (*r)(InnerP::unright(p)))
		}
	}

	/// `Costrong` instance for `ReBrand<InnerP, OuterP, S, T>` whenever `InnerP: Strong`.
	///
	/// Corresponds to:
	/// ```purescript
	/// instance strongRe :: Strong p => Costrong (Re p s t) where
	///   unfirst  (Re r) = Re (r <<< first)
	///   unsecond (Re r) = Re (r <<< second)
	/// ```
	#[document_type_parameters(
		"The inner `Strong` profunctor brand.",
		"The outer cloneable function pointer brand.",
		"The fixed source type.",
		"The fixed target type."
	)]
	impl<InnerP: Strong + 'static, PointerBrand: UnsizedCoercible + 'static, S: 'static, T: 'static>
		Costrong for ReBrand<InnerP, PointerBrand, S, T>
	{
		/// Extracts from a `Re` that operates on the first component of a pair using `InnerP::first`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Re`.",
			"The output type of the resulting `Re`.",
			"The type of the second component (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance operating on pair types.")]
		///
		#[document_returns("A `Re` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		profunctor::Costrong,
		/// 	},
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 	},
		/// };
		///
		/// // re.run: Rc<dyn Fn((i32, String)) -> (i32, String)> -> Rc<dyn Fn(i32) -> i32>
		/// let re = Re::<RcFnBrand, RcBrand, i32, i32, (i32, String), (i32, String)>::new(
		/// 	|f: std::rc::Rc<dyn Fn((i32, String)) -> (i32, String)>| {
		/// 		cloneable_fn_new::<RcFnBrand, _, _>(move |x: i32| f((x, String::new())).0)
		/// 	},
		/// );
		/// // unfirst(re).run(g) = re.run(RcFnBrand::first(g))
		/// //   where RcFnBrand::first(g)((x, s)) = (g(x), s)
		/// //   so re.run(first(g))(x) = first(g)((x, "")).0 = g(x)
		/// let result =
		/// 	<ReBrand<RcFnBrand, RcBrand, i32, i32> as Costrong>::unfirst::<i32, i32, String>(re);
		/// let add_one = cloneable_fn_new::<RcFnBrand, i32, i32>(|x: i32| x + 1);
		/// assert_eq!(((result.run)(add_one))(41), 42);
		/// ```
		fn unfirst<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Re::new(move |pba| (*r)(InnerP::first(pba)))
		}

		/// Extracts from a `Re` that operates on the second component of a pair using `InnerP::second`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Re`.",
			"The output type of the resulting `Re`.",
			"The type of the first component (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance operating on pair types.")]
		///
		#[document_returns("A `Re` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		profunctor::Costrong,
		/// 	},
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 	},
		/// };
		///
		/// // re.run: Rc<dyn Fn((String, i32)) -> (String, i32)> -> Rc<dyn Fn(i32) -> i32>
		/// let re = Re::<RcFnBrand, RcBrand, i32, i32, (String, i32), (String, i32)>::new(
		/// 	|f: std::rc::Rc<dyn Fn((String, i32)) -> (String, i32)>| {
		/// 		cloneable_fn_new::<RcFnBrand, _, _>(move |x: i32| f((String::new(), x)).1)
		/// 	},
		/// );
		/// // unsecond(re).run(g) = re.run(RcFnBrand::second(g))
		/// //   where RcFnBrand::second(g)((s, x)) = (s, g(x))
		/// //   so re.run(second(g))(x) = second(g)(("", x)).1 = g(x)
		/// let result =
		/// 	<ReBrand<RcFnBrand, RcBrand, i32, i32> as Costrong>::unsecond::<i32, i32, String>(re);
		/// let add_one = cloneable_fn_new::<RcFnBrand, i32, i32>(|x: i32| x + 1);
		/// assert_eq!(((result.run)(add_one))(41), 42);
		/// ```
		fn unsecond<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Re::new(move |pba| (*r)(InnerP::second(pba)))
		}
	}

	/// `Strong` instance for `ReBrand<InnerP, OuterP, S, T>` whenever `InnerP: Costrong`.
	///
	/// Corresponds to:
	/// ```purescript
	/// instance costrongRe :: Costrong p => Strong (Re p s t) where
	///   first  (Re r) = Re (r <<< unfirst)
	///   second (Re r) = Re (r <<< unsecond)
	/// ```
	#[document_type_parameters(
		"The inner `Costrong` profunctor brand.",
		"The outer cloneable function pointer brand.",
		"The fixed source type.",
		"The fixed target type."
	)]
	impl<
		InnerP: Costrong + 'static,
		PointerBrand: UnsizedCoercible + 'static,
		S: 'static,
		T: 'static,
	> Strong for ReBrand<InnerP, PointerBrand, S, T>
	{
		/// Lifts `Re` to operate on the first component of a pair using `InnerP::unfirst`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the second component (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance to lift.")]
		///
		#[document_returns("A `Re` instance operating on pair types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::profunctor::Strong,
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // re.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| {
		/// 	Tagged::new(t.0 + 1)
		/// });
		/// // first(re).run(Tagged((41, "hi"))) = re.run(TaggedBrand::unfirst(Tagged((41, "hi"))))
		/// //   = re.run(Tagged(41)) = Tagged(42)
		/// let result = <ReBrand<TaggedBrand, RcBrand, i32, i32> as Strong>::first::<i32, i32, &str>(re);
		/// assert_eq!((result.run)(Tagged::<(i32, &str), (i32, &str)>::new((41, "hi"))).0, 42);
		/// ```
		fn first<'a, A: 'a, B: 'a, C>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
			let r = pab.run;
			Re::new(move |p| (*r)(InnerP::unfirst(p)))
		}

		/// Lifts `Re` to operate on the second component of a pair using `InnerP::unsecond`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the first component (threaded through)."
		)]
		///
		#[document_parameters("The `Re` instance to lift.")]
		///
		#[document_returns("A `Re` instance operating on pair types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::profunctor::Strong,
		/// 	types::optics::{
		/// 		Re,
		/// 		ReBrand,
		/// 		Tagged,
		/// 		TaggedBrand,
		/// 	},
		/// };
		///
		/// // re.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let re = Re::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| {
		/// 	Tagged::new(t.0 + 1)
		/// });
		/// // second(re).run(Tagged(("hi", 41))) = re.run(TaggedBrand::unsecond(Tagged(("hi", 41))))
		/// //   = re.run(Tagged(41)) = Tagged(42)
		/// let result = <ReBrand<TaggedBrand, RcBrand, i32, i32> as Strong>::second::<i32, i32, &str>(re);
		/// assert_eq!((result.run)(Tagged::<(&str, i32), (&str, i32)>::new(("hi", 41))).0, 42);
		/// ```
		fn second<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>) {
			let r = pab.run;
			Re::new(move |p| (*r)(InnerP::unsecond(p)))
		}
	}

	/// A reversed optic, produced by the [`re`] combinator.
	///
	/// `ReversedOptic` wraps an inner optic and implements reversed optic traits by
	/// evaluating the inner optic with `ReBrand<ConcreteP>` as the profunctor.
	///
	/// Corresponds to PureScript's `re :: Optic (Re p a b) s t a b -> Optic p b a t s`.
	///
	/// The reversed optic swaps the roles of source/target and focus types:
	/// - An optic `S → T, A → B` becomes `B → A, T → S` (for review)
	/// - A simple optic `S ↔ A` becomes `A ↔ S` (for getter/fold)
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function pointer brand used by the `Re` profunctor.",
		"The source type of the original optic.",
		"The target type of the original optic.",
		"The focus source type of the original optic.",
		"The focus target type of the original optic.",
		"The inner optic type."
	)]
	pub struct ReversedOptic<'a, PointerBrand, S, T, A, B, O>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		inner: O,
		_phantom: PhantomData<(&'a (), PointerBrand, S, T, A, B)>,
	}

	/// Reverses an optic using the `Re` profunctor.
	///
	/// Given an optic from `S → T` focusing on `A → B`, produces a reversed optic
	/// that can be used as:
	/// - A [`ReviewOptic`] from `B → A` focusing on `T → S` (when the inner optic
	///   implements [`AffineTraversalOptic`])
	/// - A [`GetterOptic`] from `A` to `S` (when the inner optic implements [`PrismOptic`]
	///   with simple types `S = T, A = B`)
	/// - A [`FoldOptic`] from `A` to `S` (same conditions as getter)
	///
	/// Corresponds to PureScript's `re t = unwrap (t (Re identity))`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function pointer brand.",
		"The source type of the original optic.",
		"The target type of the original optic.",
		"The focus source type of the original optic.",
		"The focus target type of the original optic.",
		"The inner optic type."
	)]
	///
	#[document_parameters("The optic to reverse.")]
	///
	#[document_returns("A [`ReversedOptic`] wrapping the inner optic.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::optics::ReviewOptic,
	/// 	types::optics::{
	/// 		PrismPrime,
	/// 		Tagged,
	/// 		re,
	/// 	},
	/// };
	///
	/// // Create a prism for Option<i32>
	/// let some_prism: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
	///
	/// // Reverse it to get a ReviewOptic
	/// let reversed = re::<RcBrand, _, _, _, _, _>(some_prism);
	///
	/// // Use ReviewOptic: given a Tagged<Option<i32>, Option<i32>>, produce Tagged<i32, i32>
	/// let result = ReviewOptic::evaluate(&reversed, Tagged::new(Some(42)));
	/// assert_eq!(result.0, 42);
	/// ```
	pub fn re<'a, PointerBrand, S, T, A, B, O>(
		optic: O
	) -> ReversedOptic<'a, PointerBrand, S, T, A, B, O>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		ReversedOptic {
			inner: optic,
			_phantom: PhantomData,
		}
	}

	/// `ReviewOptic` for `ReversedOptic` — reversing any optic ≥ `AffineTraversal`.
	///
	/// `ReBrand<TaggedBrand>` has both `Strong` (from `TaggedBrand: Costrong`) and
	/// `Choice` (from `TaggedBrand: Cochoice`), satisfying the `P: Strong + Choice`
	/// bound required by [`AffineTraversalOptic::evaluate`].
	///
	/// This covers `Iso`, `Lens`, `Prism`, and `AffineTraversal` uniformly.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function pointer brand.",
		"The source type of the original optic.",
		"The target type of the original optic.",
		"The focus source type of the original optic.",
		"The focus target type of the original optic.",
		"The inner optic type."
	)]
	#[document_parameters("The reversed optic instance.")]
	impl<'a, PointerBrand, S, T, A, B, O> ReviewOptic<'a, B, A, T, S>
		for ReversedOptic<'a, PointerBrand, S, T, A, B, O>
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: AffineTraversalOptic<'a, S, T, A, B>,
		S: 'a + 'static,
		T: 'a + 'static,
		A: 'a + 'static,
		B: 'a + 'static,
	{
		/// Evaluates the reversed optic with `Tagged`, producing a review in the reverse direction.
		#[document_signature]
		#[document_parameters("The tagged profunctor value.")]
		#[document_returns("The transformed tagged profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::ReviewOptic,
		/// 	types::optics::{
		/// 		PrismPrime,
		/// 		Tagged,
		/// 		re,
		/// 	},
		/// };
		///
		/// let some_prism: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let reversed = re::<RcBrand, _, _, _, _, _>(some_prism);
		/// let result = ReviewOptic::evaluate(&reversed, Tagged::new(Some(42)));
		/// assert_eq!(result.0, 42);
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, T, S>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, B, A>) {
			// Re identity: Re<TaggedBrand, PB, A, B, A, B>
			// wraps Tagged<B, A> -> Tagged<B, A> (identity)
			let re_identity = Re::<TaggedBrand, PointerBrand, A, B, A, B>::new(|x| x);
			// Evaluate inner optic with P = ReBrand<TaggedBrand, PB, A, B>
			// Input: P::Of<A, B> = Re<TaggedBrand, PB, A, B, A, B> (our identity)
			// Output: P::Of<S, T> = Re<TaggedBrand, PB, A, B, S, T>
			let result =
				self.inner.evaluate::<ReBrand<TaggedBrand, PointerBrand, A, B>>(re_identity);
			// Re<TaggedBrand, PB, A, B, S, T> wraps Tagged<T, S> -> Tagged<B, A>
			(result.run)(pab)
		}
	}

	/// `GetterOptic` for `ReversedOptic` — reversing prism-like optics to getters.
	///
	/// `ReBrand<ForgetBrand<Q, R>>` has `Choice` (from `ForgetBrand: Cochoice`)
	/// but NOT `Strong` (since `ForgetBrand` is not `Costrong`), so only
	/// [`PrismOptic`]'s `P: Choice` bound can be satisfied.
	///
	/// This means `re(prism)` and `re(iso)` produce getters, but `re(lens)` does not.
	/// This matches PureScript semantics.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function pointer brand.",
		"The source type of the original (simple) optic.",
		"The focus type of the original (simple) optic.",
		"The inner optic type."
	)]
	#[document_parameters("The reversed optic instance.")]
	impl<'a, PointerBrand, S, A, O> GetterOptic<'a, A, S>
		for ReversedOptic<'a, PointerBrand, S, S, A, A, O>
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: PrismOptic<'a, S, S, A, A>,
		S: 'a + 'static,
		A: 'a + 'static,
	{
		/// Evaluates the reversed optic with `Forget`, producing a getter in the reverse direction.
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The forget profunctor value.")]
		#[document_returns("The transformed forget profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::GetterOptic,
		/// 	types::optics::{
		/// 		Forget,
		/// 		PrismPrime,
		/// 		re,
		/// 	},
		/// };
		///
		/// // re(prism) as a getter: given A, extract S via the prism's review
		/// let some_prism: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let reversed = re::<RcBrand, _, _, _, _, _>(some_prism);
		/// let forget = Forget::<RcBrand, Option<i32>, Option<i32>, Option<i32>>::new(|o| o);
		/// let result = GetterOptic::evaluate::<Option<i32>, RcBrand>(&reversed, forget);
		/// assert_eq!(result.run(42), Some(42));
		/// ```
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>)
		{
			// Re identity: Re<ForgetBrand<Q, R>, PB, A, A, A, A>
			// wraps Forget<Q, R, A, A> -> Forget<Q, R, A, A> (identity)
			let re_identity = Re::<ForgetBrand<Q, R>, PointerBrand, A, A, A, A>::new(|x| x);
			// Evaluate inner optic with P = ReBrand<ForgetBrand<Q, R>, PB, A, A>
			// Input: P::Of<A, A> = Re<ForgetBrand<Q, R>, PB, A, A, A, A> (our identity)
			// Output: P::Of<S, S> = Re<ForgetBrand<Q, R>, PB, A, A, S, S>
			let result =
				self.inner.evaluate::<ReBrand<ForgetBrand<Q, R>, PointerBrand, A, A>>(re_identity);
			// Re<ForgetBrand<Q, R>, PB, A, A, S, S> wraps Forget<Q, R, S, S> -> Forget<Q, R, A, A>
			(result.run)(pab)
		}
	}

	/// `FoldOptic` for `ReversedOptic` — reversing prism-like optics to folds.
	///
	/// Same as the [`GetterOptic`] implementation but with the additional `R: Monoid + Clone` bound
	/// required by [`FoldOptic`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function pointer brand.",
		"The source type of the original (simple) optic.",
		"The focus type of the original (simple) optic.",
		"The inner optic type."
	)]
	#[document_parameters("The reversed optic instance.")]
	impl<'a, PointerBrand, S, A, O> FoldOptic<'a, A, S>
		for ReversedOptic<'a, PointerBrand, S, S, A, A, O>
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: PrismOptic<'a, S, S, A, A>,
		S: 'a + 'static,
		A: 'a + 'static,
	{
		/// Evaluates the reversed optic with `Forget` for a monoidal fold in the reverse direction.
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The forget profunctor value.")]
		#[document_returns("The transformed forget profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::optics::FoldOptic,
		/// 	types::optics::{
		/// 		Forget,
		/// 		PrismPrime,
		/// 		re,
		/// 	},
		/// };
		///
		/// // re(prism) as a fold with String monoid
		/// let some_prism: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let reversed = re::<RcBrand, _, _, _, _, _>(some_prism);
		/// let forget = Forget::<RcBrand, String, Option<i32>, Option<i32>>::new(|o: Option<i32>| {
		/// 	format!("{:?}", o)
		/// });
		/// let result = FoldOptic::evaluate::<String, RcBrand>(&reversed, forget);
		/// assert_eq!(result.run(42), "Some(42)");
		/// ```
		fn evaluate<R: 'a + Monoid + Clone + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>)
		{
			let re_identity = Re::<ForgetBrand<Q, R>, PointerBrand, A, A, A, A>::new(|x| x);
			let result =
				self.inner.evaluate::<ReBrand<ForgetBrand<Q, R>, PointerBrand, A, A>>(re_identity);
			(result.run)(pab)
		}
	}
}
pub use inner::*;
