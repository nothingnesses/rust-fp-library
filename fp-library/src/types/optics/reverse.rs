//! The `Reverse` profunctor, for reversing optic constraints.
//!
//! `Reverse<'a, InnerP, OuterP, S, T, A, B>` wraps a function `InnerP::Of<'a, B, A> -> InnerP::Of<'a, T, S>`.
//! It "reverses" the profunctor structure of `InnerP`:
//!
//! - `InnerP: Profunctor` -> `ReverseBrand<InnerP, OuterP, S, T>: Profunctor`
//! - `InnerP: Choice` -> `ReverseBrand<InnerP, OuterP, S, T>: Cochoice`
//! - `InnerP: Cochoice` -> `ReverseBrand<InnerP, OuterP, S, T>: Choice`
//! - `InnerP: Strong` -> `ReverseBrand<InnerP, OuterP, S, T>: Costrong`
//! - `InnerP: Costrong` -> `ReverseBrand<InnerP, OuterP, S, T>: Strong`
//!
//! This is a port of PureScript's [`Data.Lens.Internal.Re`](https://pursuit.purescript.org/packages/purescript-profunctor-lenses/docs/Data.Lens.Internal.Re).

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
				CloneableFn,
				Monoid,
				UnsizedCoercible,
				optics::{
					FoldOptic,
					GetterOptic,
					IsoOptic,
					LensOptic,
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
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// The `Reverse` profunctor.
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
	pub struct Reverse<
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
		Reverse<'a, InnerP, PointerBrand, S, T, A, B>
	{
		/// Creates a new `Reverse` instance by wrapping a function.
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
		/// 		optics::*,
		/// 	},
		/// 	types::optics::{
		/// 		Reverse,
		/// 		Tagged,
		/// 	},
		/// };
		///
		/// // Reverse wraps a function from `Tagged<B, A>` to `Tagged<T, S>`.
		/// let rev =
		/// 	Reverse::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|tagged: Tagged<i32, i32>| {
		/// 		Tagged::new(tagged.0 + 1)
		/// 	});
		/// assert_eq!((rev.run)(Tagged::new(41)).0, 42);
		/// ```
		pub fn new(
			f: impl 'a
			+ Fn(
				Apply!(<InnerP as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, B, A>),
			) -> Apply!(<InnerP as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, T, S>)
		) -> Self {
			Reverse {
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
	#[document_parameters("The `Reverse` instance.")]
	impl<'a, InnerP: Profunctor, PointerBrand: UnsizedCoercible, S: 'a, T: 'a, A: 'a, B: 'a> Clone
		for Reverse<'a, InnerP, PointerBrand, S, T, A, B>
	{
		#[document_signature]
		#[document_returns("A new `Reverse` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	types::optics::{
		/// 		Reverse,
		/// 		Tagged,
		/// 	},
		/// };
		///
		/// let rev = Reverse::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| {
		/// 	Tagged::new(t.0)
		/// });
		/// let cloned = rev.clone();
		/// assert_eq!((cloned.run)(Tagged::new(42)).0, 42);
		/// ```
		fn clone(&self) -> Self {
			Reverse {
				run: self.run.clone(),
			}
		}
	}

	impl_kind! {
		impl<
			InnerP: Profunctor + 'static,
			PointerBrand: UnsizedCoercible + 'static,
			S: 'static,
			T: 'static,
		> for ReverseBrand<InnerP, PointerBrand, S, T> {
			#[document_default]
			type Of<'a, A: 'a, B: 'a>: 'a = Reverse<'a, InnerP, PointerBrand, S, T, A, B>;
		}
	}

	/// `Profunctor` instance for `ReverseBrand<InnerP, OuterP, S, T>` whenever `InnerP: Profunctor`.
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
	> Profunctor for ReverseBrand<InnerP, PointerBrand, S, T>
	{
		/// Maps over both arguments of `Reverse`, swapping the roles of `f` and `g` on the inner profunctor.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The new contravariant type.",
			"The original contravariant type.",
			"The original covariant type.",
			"The new covariant type."
		)]
		///
		#[document_parameters(
			"The contravariant function `A -> B`.",
			"The covariant function `C -> D`.",
			"The `Reverse` instance to transform."
		)]
		#[document_returns("A transformed `Reverse` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::profunctor::Profunctor,
		/// 	types::optics::{
		/// 		Reverse,
		/// 		Tagged,
		/// 	},
		/// };
		///
		/// // rev.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let rev =
		/// 	Reverse::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|tagged: Tagged<i32, i32>| {
		/// 		Tagged::new(tagged.0)
		/// 	});
		/// // dimap(ab=|x| x*2, cd=|x| x+1, rev).run(Tagged(5))
		/// //   = rev.run(TaggedBrand::dimap(cd, ab, Tagged(5)))
		/// //   = rev.run(Tagged(ab(5))) = rev.run(Tagged(10)) = Tagged(10)
		/// let transformed = <ReverseBrand<TaggedBrand, RcBrand, i32, i32> as Profunctor>::dimap(
		/// 	|x: i32| x * 2,
		/// 	|x: i32| x + 1,
		/// 	rev,
		/// );
		/// assert_eq!((transformed.run)(Tagged::new(5)).0, 10);
		/// ```
		fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			ab: impl Fn(A) -> B + 'a,
			cd: impl Fn(C) -> D + 'a,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>) {
			let r = pbc.run;
			let ab = <FnBrand<PointerBrand> as CloneableFn>::new(ab);
			let cd = <FnBrand<PointerBrand> as CloneableFn>::new(cd);
			Reverse::new(move |pda| {
				let ab = ab.clone();
				let cd = cd.clone();
				(*r)(InnerP::dimap(move |c| (*cd)(c), move |a| (*ab)(a), pda))
			})
		}
	}

	/// `Cochoice` instance for `ReverseBrand<InnerP, OuterP, S, T>` whenever `InnerP: Choice`.
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
		Cochoice for ReverseBrand<InnerP, PointerBrand, S, T>
	{
		/// Extracts from a `Reverse` that operates on `Result` types using `InnerP::left`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Reverse`.",
			"The output type of the resulting `Reverse`.",
			"The type of the `Ok` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance operating on `Result` types.")]
		///
		#[document_returns("A `Reverse` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::profunctor::Cochoice,
		/// 	types::optics::{
		/// 		Reverse,
		/// 		Tagged,
		/// 	},
		/// };
		///
		/// // rev.run: Tagged<Result<String, i32>, Result<String, i32>> -> Tagged<i32, i32>
		/// let rev =
		/// 	Reverse::<TaggedBrand, RcBrand, i32, i32, Result<String, i32>, Result<String, i32>>::new(
		/// 		|t: Tagged<Result<String, i32>, Result<String, i32>>| Tagged::new(t.0.unwrap_err() + 1),
		/// 	);
		/// // unleft(rev).run(Tagged(41)) = rev.run(TaggedBrand::left(Tagged(41)))
		/// //   = rev.run(Tagged(Err(41))) = Tagged(42)
		/// let result =
		/// 	<ReverseBrand<TaggedBrand, RcBrand, i32, i32> as Cochoice>::unleft::<i32, i32, String>(rev);
		/// assert_eq!((result.run)(Tagged::new(41)).0, 42);
		/// ```
		fn unleft<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Reverse::new(move |pba| (*r)(InnerP::left(pba)))
		}

		/// Extracts from a `Reverse` that operates on `Result` types using `InnerP::right`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Reverse`.",
			"The output type of the resulting `Reverse`.",
			"The type of the `Err` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance operating on `Result` types.")]
		///
		#[document_returns("A `Reverse` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::profunctor::Cochoice,
		/// 	types::optics::{
		/// 		Reverse,
		/// 		Tagged,
		/// 	},
		/// };
		///
		/// // rev.run: Tagged<Result<i32, String>, Result<i32, String>> -> Tagged<i32, i32>
		/// let rev =
		/// 	Reverse::<TaggedBrand, RcBrand, i32, i32, Result<i32, String>, Result<i32, String>>::new(
		/// 		|t: Tagged<Result<i32, String>, Result<i32, String>>| Tagged::new(t.0.unwrap() + 1),
		/// 	);
		/// // unright(rev).run(Tagged(41)) = rev.run(TaggedBrand::right(Tagged(41)))
		/// //   = rev.run(Tagged(Ok(41))) = Tagged(42)
		/// let result =
		/// 	<ReverseBrand<TaggedBrand, RcBrand, i32, i32> as Cochoice>::unright::<i32, i32, String>(
		/// 		rev,
		/// 	);
		/// assert_eq!((result.run)(Tagged::new(41)).0, 42);
		/// ```
		fn unright<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Reverse::new(move |pba| (*r)(InnerP::right(pba)))
		}
	}

	/// `Choice` instance for `ReverseBrand<InnerP, OuterP, S, T>` whenever `InnerP: Cochoice`.
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
	> Choice for ReverseBrand<InnerP, PointerBrand, S, T>
	{
		/// Lifts `Reverse` to operate on the `Err` variant of a `Result` using `InnerP::unleft`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the `Ok` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance to lift.")]
		///
		#[document_returns("A `Reverse` instance operating on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::profunctor::Choice,
		/// 	types::optics::{
		/// 		Forget,
		/// 		Reverse,
		/// 	},
		/// };
		///
		/// // rev wraps a getter transformer: (i32 -> i32) -> (i32 -> i32)
		/// let rev = Reverse::<ForgetBrand<RcBrand, i32>, RcBrand, i32, i32, i32, i32>::new(
		/// 	|f: Forget<'_, RcBrand, i32, i32, i32>| Forget::new(move |x: i32| f.run(x) + 1),
		/// );
		/// // left(rev).run(getter) = rev.run(unleft(getter))
		/// //   unleft wraps input in Err: unleft(|r| r.unwrap_err()) = identity
		/// //   rev.run(identity) = |x| x + 1
		/// let result = <ReverseBrand<ForgetBrand<RcBrand, i32>, RcBrand, i32, i32> as Choice>::left::<
		/// 	i32,
		/// 	i32,
		/// 	String,
		/// >(rev);
		/// let transformed = (result.run)(Forget::new(|r: Result<String, i32>| r.unwrap_err()));
		/// assert_eq!(transformed.run(41), 42);
		/// ```
		fn left<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		{
			let r = pab.run;
			Reverse::new(move |p| (*r)(InnerP::unleft(p)))
		}

		/// Lifts `Reverse` to operate on the `Ok` variant of a `Result` using `InnerP::unright`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the `Err` variant (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance to lift.")]
		///
		#[document_returns("A `Reverse` instance operating on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::profunctor::Choice,
		/// 	types::optics::{
		/// 		Forget,
		/// 		Reverse,
		/// 	},
		/// };
		///
		/// // rev wraps a getter transformer: (i32 -> i32) -> (i32 -> i32)
		/// let rev = Reverse::<ForgetBrand<RcBrand, i32>, RcBrand, i32, i32, i32, i32>::new(
		/// 	|f: Forget<'_, RcBrand, i32, i32, i32>| Forget::new(move |x: i32| f.run(x) + 1),
		/// );
		/// // right(rev).run(getter) = rev.run(unright(getter))
		/// //   unright wraps input in Ok: unright(|r| r.unwrap()) = identity
		/// //   rev.run(identity) = |x| x + 1
		/// let result = <ReverseBrand<ForgetBrand<RcBrand, i32>, RcBrand, i32, i32> as Choice>::right::<
		/// 	i32,
		/// 	i32,
		/// 	String,
		/// >(rev);
		/// let transformed = (result.run)(Forget::new(|r: Result<i32, String>| r.unwrap()));
		/// assert_eq!(transformed.run(41), 42);
		/// ```
		fn right<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		{
			let r = pab.run;
			Reverse::new(move |p| (*r)(InnerP::unright(p)))
		}
	}

	/// `Costrong` instance for `ReverseBrand<InnerP, OuterP, S, T>` whenever `InnerP: Strong`.
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
		Costrong for ReverseBrand<InnerP, PointerBrand, S, T>
	{
		/// Extracts from a `Reverse` that operates on the first component of a pair using `InnerP::first`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Reverse`.",
			"The output type of the resulting `Reverse`.",
			"The type of the second component (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance operating on pair types.")]
		///
		#[document_returns("A `Reverse` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		profunctor::Costrong,
		/// 	},
		/// 	types::optics::Reverse,
		/// };
		///
		/// // rev.run: Rc<dyn Fn((i32, String)) -> (i32, String)> -> Rc<dyn Fn(i32) -> i32>
		/// let rev = Reverse::<RcFnBrand, RcBrand, i32, i32, (i32, String), (i32, String)>::new(
		/// 	|f: std::rc::Rc<dyn Fn((i32, String)) -> (i32, String)>| {
		/// 		cloneable_fn_new::<RcFnBrand, _, _>(move |x: i32| f((x, String::new())).0)
		/// 	},
		/// );
		/// // unfirst(rev).run(g) = rev.run(RcFnBrand::first(g))
		/// //   where RcFnBrand::first(g)((x, s)) = (g(x), s)
		/// //   so rev.run(first(g))(x) = first(g)((x, "")).0 = g(x)
		/// let result =
		/// 	<ReverseBrand<RcFnBrand, RcBrand, i32, i32> as Costrong>::unfirst::<i32, i32, String>(rev);
		/// let add_one = cloneable_fn_new::<RcFnBrand, i32, i32>(|x: i32| x + 1);
		/// assert_eq!(((result.run)(add_one))(41), 42);
		/// ```
		fn unfirst<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Reverse::new(move |pba| (*r)(InnerP::first(pba)))
		}

		/// Extracts from a `Reverse` that operates on the second component of a pair using `InnerP::second`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the resulting `Reverse`.",
			"The output type of the resulting `Reverse`.",
			"The type of the first component (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance operating on pair types.")]
		///
		#[document_returns("A `Reverse` instance operating on the unwrapped types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		profunctor::Costrong,
		/// 	},
		/// 	types::optics::Reverse,
		/// };
		///
		/// // rev.run: Rc<dyn Fn((String, i32)) -> (String, i32)> -> Rc<dyn Fn(i32) -> i32>
		/// let rev = Reverse::<RcFnBrand, RcBrand, i32, i32, (String, i32), (String, i32)>::new(
		/// 	|f: std::rc::Rc<dyn Fn((String, i32)) -> (String, i32)>| {
		/// 		cloneable_fn_new::<RcFnBrand, _, _>(move |x: i32| f((String::new(), x)).1)
		/// 	},
		/// );
		/// // unsecond(rev).run(g) = rev.run(RcFnBrand::second(g))
		/// //   where RcFnBrand::second(g)((s, x)) = (s, g(x))
		/// //   so rev.run(second(g))(x) = second(g)(("", x)).1 = g(x)
		/// let result =
		/// 	<ReverseBrand<RcFnBrand, RcBrand, i32, i32> as Costrong>::unsecond::<i32, i32, String>(rev);
		/// let add_one = cloneable_fn_new::<RcFnBrand, i32, i32>(|x: i32| x + 1);
		/// assert_eq!(((result.run)(add_one))(41), 42);
		/// ```
		fn unsecond<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			let r = pab.run;
			Reverse::new(move |pba| (*r)(InnerP::second(pba)))
		}
	}

	/// `Strong` instance for `ReverseBrand<InnerP, OuterP, S, T>` whenever `InnerP: Costrong`.
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
	> Strong for ReverseBrand<InnerP, PointerBrand, S, T>
	{
		/// Lifts `Reverse` to operate on the first component of a pair using `InnerP::unfirst`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the second component (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance to lift.")]
		///
		#[document_returns("A `Reverse` instance operating on pair types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::profunctor::Strong,
		/// 	types::optics::{
		/// 		Reverse,
		/// 		Tagged,
		/// 	},
		/// };
		///
		/// // rev.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let rev = Reverse::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| {
		/// 	Tagged::new(t.0 + 1)
		/// });
		/// // first(rev).run(Tagged((41, "hi"))) = rev.run(TaggedBrand::unfirst(Tagged((41, "hi"))))
		/// //   = rev.run(Tagged(41)) = Tagged(42)
		/// let result =
		/// 	<ReverseBrand<TaggedBrand, RcBrand, i32, i32> as Strong>::first::<i32, i32, &str>(rev);
		/// assert_eq!((result.run)(Tagged::<(i32, &str), (i32, &str)>::new((41, "hi"))).0, 42);
		/// ```
		fn first<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
			let r = pab.run;
			Reverse::new(move |p| (*r)(InnerP::unfirst(p)))
		}

		/// Lifts `Reverse` to operate on the second component of a pair using `InnerP::unsecond`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The input type of the profunctor.",
			"The output type of the profunctor.",
			"The type of the first component (threaded through)."
		)]
		///
		#[document_parameters("The `Reverse` instance to lift.")]
		///
		#[document_returns("A `Reverse` instance operating on pair types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::profunctor::Strong,
		/// 	types::optics::{
		/// 		Reverse,
		/// 		Tagged,
		/// 	},
		/// };
		///
		/// // rev.run: Tagged<i32, i32> -> Tagged<i32, i32>
		/// let rev = Reverse::<TaggedBrand, RcBrand, i32, i32, i32, i32>::new(|t: Tagged<i32, i32>| {
		/// 	Tagged::new(t.0 + 1)
		/// });
		/// // second(rev).run(Tagged(("hi", 41))) = rev.run(TaggedBrand::unsecond(Tagged(("hi", 41))))
		/// //   = rev.run(Tagged(41)) = Tagged(42)
		/// let result =
		/// 	<ReverseBrand<TaggedBrand, RcBrand, i32, i32> as Strong>::second::<i32, i32, &str>(rev);
		/// assert_eq!((result.run)(Tagged::<(&str, i32), (&str, i32)>::new(("hi", 41))).0, 42);
		/// ```
		fn second<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>) {
			let r = pab.run;
			Reverse::new(move |p| (*r)(InnerP::unsecond(p)))
		}
	}

	/// A reversed optic, produced by the [`reverse`] combinator.
	///
	/// `ReversedOptic` wraps an inner optic and implements reversed optic traits by
	/// evaluating the inner optic with `ReverseBrand<ConcreteP>` as the profunctor.
	///
	/// Corresponds to PureScript's `re :: Optic (Re p a b) s t a b -> Optic p b a t s`.
	///
	/// The reversed optic swaps the roles of source/target and focus types:
	/// - An optic `S -> T, A -> B` becomes `B -> A, T -> S` (for review)
	/// - A simple optic `S <-> A` becomes `A <-> S` (for getter/fold)
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function pointer brand used by the `Reverse` profunctor.",
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

	/// Reverses an optic using the `Reverse` profunctor.
	///
	/// Given an optic from `S -> T` focusing on `A -> B`, produces a reversed optic
	/// that can be used as:
	/// - An [`IsoOptic`] from `B, A` to `T, S` (when the inner optic implements [`IsoOptic`])
	/// - A [`ReviewOptic`] from `B -> A` focusing on `T -> S` (when the inner optic
	///   implements [`LensOptic`] - covers isos and lenses)
	/// - A [`GetterOptic`] from `A` to `S` (when the inner optic implements [`PrismOptic`]
	///   with simple types `S = T, A = B`)
	/// - A [`FoldOptic`] from `A` to `S` (same conditions as getter)
	///
	/// Corresponds to PureScript's `re t = unwrap (t (Reverse identity))`.
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
	/// 	brands::{
	/// 		optics::*,
	/// 		*,
	/// 	},
	/// 	classes::optics::ReviewOptic,
	/// 	types::optics::{
	/// 		LensPrime,
	/// 		Tagged,
	/// 		reverse,
	/// 	},
	/// };
	///
	/// // Create a lens for (i32, String) focusing on the first element
	/// let fst: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
	///
	/// // Reverse it to get a ReviewOptic
	/// let reversed = reverse::<RcBrand, _, _, _, _, _>(fst);
	///
	/// // Use ReviewOptic: given a Tagged<(i32, String), (i32, String)>, produce Tagged<i32, i32>
	/// let result = ReviewOptic::evaluate(&reversed, Tagged::new((42, "hello".to_string())));
	/// assert_eq!(result.0, 42);
	/// ```
	pub fn reverse<'a, PointerBrand, S, T, A, B, O>(
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

	/// `ReviewOptic` for `ReversedOptic` - reversing any optic >= `Lens`.
	///
	/// `ReverseBrand<TaggedBrand>` has `Strong` (from `TaggedBrand: Costrong`),
	/// satisfying the `P: Strong` bound required by [`LensOptic::evaluate`].
	///
	/// This covers `Iso` and `Lens`, matching the PureScript semantics where
	/// `Re Tagged` has `Strong` (from `Tagged: Costrong`) but not `Choice`
	/// (since `Tagged` does not implement `Cochoice`).
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
		O: LensOptic<'a, S, T, A, B>,
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
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::ReviewOptic,
		/// 	types::optics::{
		/// 		LensPrime,
		/// 		Tagged,
		/// 		reverse,
		/// 	},
		/// };
		///
		/// // reverse(lens) as a review: given the source, extracts the focus.
		/// let lens: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let reversed = reverse::<RcBrand, _, _, _, _, _>(lens);
		/// let result = ReviewOptic::evaluate(&reversed, Tagged::new((42, "hello".to_string())));
		/// assert_eq!(result.0, 42);
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, T, S>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, B, A>) {
			// Reverse identity: Reverse<TaggedBrand, PB, A, B, A, B>
			// wraps Tagged<B, A> -> Tagged<B, A> (identity)
			let rev_identity = Reverse::<TaggedBrand, PointerBrand, A, B, A, B>::new(|x| x);
			// Evaluate inner optic with P = ReverseBrand<TaggedBrand, PB, A, B>
			// ReverseBrand<TaggedBrand> has Strong (from TaggedBrand: Costrong),
			// satisfying LensOptic's P: Strong bound.
			// Input: P::Of<A, B> = Reverse<TaggedBrand, PB, A, B, A, B> (our identity)
			// Output: P::Of<S, T> = Reverse<TaggedBrand, PB, A, B, S, T>
			let result =
				self.inner.evaluate::<ReverseBrand<TaggedBrand, PointerBrand, A, B>>(rev_identity);
			// Reverse<TaggedBrand, PB, A, B, S, T> wraps Tagged<T, S> -> Tagged<B, A>
			(result.run)(pab)
		}
	}

	/// `IsoOptic` for `ReversedOptic` - reversing an iso to an iso.
	///
	/// `ReverseBrand<P>` has `Profunctor` whenever `P: Profunctor`, which is all that
	/// [`IsoOptic::evaluate`] requires. This means `reverse(iso)` is itself an iso,
	/// matching the PureScript semantics where `re :: Iso s t a b -> Iso b a t s`.
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
	impl<'a, PointerBrand, S, T, A, B, O> IsoOptic<'a, B, A, T, S>
		for ReversedOptic<'a, PointerBrand, S, T, A, B, O>
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: IsoOptic<'a, S, T, A, B>,
		S: 'a + 'static,
		T: 'a + 'static,
		A: 'a + 'static,
		B: 'a + 'static,
	{
		/// Evaluates the reversed optic with any profunctor, producing an iso in the reverse direction.
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::IsoOptic,
		/// 	functions::cloneable_fn_new,
		/// 	types::optics::{
		/// 		IsoPrime,
		/// 		reverse,
		/// 	},
		/// };
		///
		/// // An iso between (i32,) and i32
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let reversed = reverse::<RcBrand, _, _, _, _, _>(iso);
		/// // reverse(iso) is itself an iso from i32 to (i32,)
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|(x,): (i32,)| (x * 2,));
		/// let modifier = IsoOptic::evaluate::<RcFnBrand>(&reversed, f);
		/// assert_eq!(modifier(21), 42);
		/// ```
		fn evaluate<P: Profunctor + 'static>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, T, S>),
		) -> Apply!(<P as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, B, A>) {
			// Reverse identity: Reverse<P, PB, A, B, A, B>
			// wraps P::Of<B, A> -> P::Of<B, A> (identity)
			let rev_identity = Reverse::<P, PointerBrand, A, B, A, B>::new(|x| x);
			// Evaluate inner iso with ReverseBrand<P, PB, A, B>
			// IsoOptic::evaluate needs Profunctor, and ReverseBrand<P>: Profunctor when P: Profunctor
			let result = self.inner.evaluate::<ReverseBrand<P, PointerBrand, A, B>>(rev_identity);
			// Reverse<P, PB, A, B, S, T> wraps P::Of<T, S> -> P::Of<B, A>
			(result.run)(pab)
		}
	}

	/// `GetterOptic` for `ReversedOptic` - reversing prism-like optics to getters.
	///
	/// `ReverseBrand<ForgetBrand<Q, R>>` has `Choice` (from `ForgetBrand: Cochoice`)
	/// but NOT `Strong` (since `ForgetBrand` is not `Costrong`), so only
	/// [`PrismOptic`]'s `P: Choice` bound can be satisfied.
	///
	/// This means `reverse(prism)` and `reverse(iso)` produce getters, but `reverse(lens)` does not.
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
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::GetterOptic,
		/// 	types::optics::{
		/// 		Forget,
		/// 		PrismPrime,
		/// 		reverse,
		/// 	},
		/// };
		///
		/// // reverse(prism) as a getter: given A, extract S via the prism's review
		/// let some_prism: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let reversed = reverse::<RcBrand, _, _, _, _, _>(some_prism);
		/// let forget = Forget::<RcBrand, Option<i32>, Option<i32>, Option<i32>>::new(|o| o);
		/// let result = GetterOptic::evaluate::<Option<i32>, RcBrand>(&reversed, forget);
		/// assert_eq!(result.run(42), Some(42));
		/// ```
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, S, S>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, X: 'b, Y: 'b>: 'b; )>::Of<'a, A, A>)
		{
			// Reverse identity: Reverse<ForgetBrand<Q, R>, PB, A, A, A, A>
			// wraps Forget<Q, R, A, A> -> Forget<Q, R, A, A> (identity)
			let rev_identity = Reverse::<ForgetBrand<Q, R>, PointerBrand, A, A, A, A>::new(|x| x);
			// Evaluate inner optic with P = ReverseBrand<ForgetBrand<Q, R>, PB, A, A>
			// Input: P::Of<A, A> = Reverse<ForgetBrand<Q, R>, PB, A, A, A, A> (our identity)
			// Output: P::Of<S, S> = Reverse<ForgetBrand<Q, R>, PB, A, A, S, S>
			let result = self
				.inner
				.evaluate::<ReverseBrand<ForgetBrand<Q, R>, PointerBrand, A, A>>(rev_identity);
			// Reverse<ForgetBrand<Q, R>, PB, A, A, S, S> wraps Forget<Q, R, S, S> -> Forget<Q, R, A, A>
			(result.run)(pab)
		}
	}

	/// `FoldOptic` for `ReversedOptic` - reversing prism-like optics to folds.
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
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::FoldOptic,
		/// 	types::optics::{
		/// 		Forget,
		/// 		PrismPrime,
		/// 		reverse,
		/// 	},
		/// };
		///
		/// // reverse(prism) as a fold with String monoid
		/// let some_prism: PrismPrime<RcBrand, Option<i32>, i32> = PrismPrime::from_option(|o| o, Some);
		/// let reversed = reverse::<RcBrand, _, _, _, _, _>(some_prism);
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
			let rev_identity = Reverse::<ForgetBrand<Q, R>, PointerBrand, A, A, A, A>::new(|x| x);
			let result = self
				.inner
				.evaluate::<ReverseBrand<ForgetBrand<Q, R>, PointerBrand, A, A>>(rev_identity);
			(result.run)(pab)
		}
	}
}
pub use inner::*;
