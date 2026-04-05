//! The `Bazaar` profunctor, used to characterize traversals.
//!
//! `Bazaar<A, B, S, T>` wraps a decomposition function `S -> BazaarList<A, B, T>` that extracts
//! a list of foci from a source and provides a rebuild function.
//!
//! This is a Rust port of PureScript's `Data.Lens.Internal.Bazaar`, specialized to the function
//! profunctor (`p = (->)`). The rank-2 polymorphism over applicatives is handled via the
//! [`BazaarList`] decomposition: instead of storing `forall f. Applicative f => (a -> f b) -> s -> f t`,
//! we store `s -> (Vec<a>, Vec<b> -> t)` and interpret it with any applicative at run time.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				VecBrand,
				optics::*,
			},
			classes::{
				ApplyFirst,
				ApplySecond,
				Lift,
				optics::traversal::TraversalFunc,
				profunctor::{
					Choice,
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
	};

	/// Type alias to extract the pointer brand from a `CloneFn` implementor.
	type Ptr<FunctionBrand> = <FunctionBrand as CloneFn>::PointerBrand;

	// -- BazaarList --

	/// A decomposed traversal structure: a list of foci paired with a rebuild function.
	///
	/// `BazaarList` is [`Applicative`](crate::classes::Applicative) in `T`, which is the
	/// key property that enables [`Bazaar`] to defer the choice of applicative until
	/// [`run_bazaar`] time.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The type of focus values extracted from the source.",
		"The type of replacement values used during reconstruction.",
		"The result type after reconstruction."
	)]
	pub struct BazaarList<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, T: 'a> {
		/// The list of focus values extracted from the source.
		pub foci: Vec<A>,
		/// A function that reconstructs the target from a list of replacement values.
		pub rebuild: <FunctionBrand as CloneFn>::Of<'a, Vec<B>, T>,
	}

	impl_kind! {
		impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> for BazaarListBrand<FunctionBrand, A, B> {
			type Of<'a, T: 'a>: 'a = BazaarList<'a, FunctionBrand, A, B, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Functor
		for BazaarListBrand<FunctionBrand, A, B>
	{
		/// Maps a function over the result type of a `BazaarList`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters("The function to apply.", "The bazaar list to map over.")]
		///
		#[document_returns("A new `BazaarList` with the same foci but a transformed rebuild.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bl = BazaarList::<RcFnBrand, i32, i32, i32> {
		/// 	foci: vec![1, 2],
		/// 	rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs.iter().sum()),
		/// };
		/// let mapped = map::<BazaarListBrand<RcFnBrand, i32, i32>, _, _, _>(|t: i32| t * 10, bl);
		/// assert_eq!((mapped.rebuild)(vec![3, 4]), 70);
		/// ```
		fn map<'a, T: 'a, U: 'a>(
			func: impl Fn(T) -> U + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, T>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, U>) {
			let rebuild = fa.rebuild;
			BazaarList {
				foci: fa.foci,
				rebuild: <FunctionBrand as LiftFn>::new(move |bs: Vec<B>| func((*rebuild)(bs))),
			}
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Pointed
		for BazaarListBrand<FunctionBrand, A, B>
	{
		/// Wraps a value in a `BazaarList` with no foci.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `BazaarList` with empty foci that ignores its input.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bl = pure::<BazaarListBrand<RcFnBrand, i32, i32>, _>(42);
		/// assert_eq!((bl.rebuild)(vec![]), 42);
		/// ```
		fn pure<'a, T: 'a>(a: T) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, T>) {
			let a = Ptr::<FunctionBrand>::take_cell_new(a);
			BazaarList {
				foci: vec![],
				rebuild: <FunctionBrand as LiftFn>::new(move |_: Vec<B>| {
					// SAFETY: take_cell_take is called exactly once per the optics rebuild contract
					#[allow(clippy::expect_used)]
					Ptr::<FunctionBrand>::take_cell_take(&a)
						.expect("BazaarList::pure rebuild called more than once")
				}),
			}
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Lift
		for BazaarListBrand<FunctionBrand, A, B>
	{
		/// Lifts a binary function to combine two `BazaarList` values.
		///
		/// Concatenates the foci from both lists and splits the replacement vector
		/// at the boundary when rebuilding.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The result type of the first `BazaarList`.",
			"The result type of the second `BazaarList`.",
			"The combined result type."
		)]
		///
		#[document_parameters(
			"The binary function to combine results.",
			"The first `BazaarList`.",
			"The second `BazaarList`."
		)]
		///
		#[document_returns(
			"A `BazaarList` with concatenated foci whose rebuild splits and delegates."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bl1 = BazaarList::<RcFnBrand, i32, i32, i32> {
		/// 	foci: vec![1],
		/// 	rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// };
		/// let bl2 = BazaarList::<RcFnBrand, i32, i32, i32> {
		/// 	foci: vec![2],
		/// 	rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// };
		/// let combined = lift2::<BazaarListBrand<RcFnBrand, i32, i32>, _, _, _>(|a, b| a + b, bl1, bl2);
		/// assert_eq!(combined.foci, vec![1, 2]);
		/// assert_eq!((combined.rebuild)(vec![10, 20]), 30);
		/// ```
		fn lift2<'a, T, U, V>(
			func: impl Fn(T, U) -> V + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, T>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, V>)
		where
			T: Clone + 'a,
			U: Clone + 'a,
			V: 'a, {
			let split_at = fa.foci.len();
			let mut foci = fa.foci;
			foci.extend(fb.foci);
			let rebuild_a = fa.rebuild;
			let rebuild_b = fb.rebuild;
			BazaarList {
				foci,
				rebuild: <FunctionBrand as LiftFn>::new(move |mut bs: Vec<B>| {
					let right = bs.split_off(split_at);
					func((*rebuild_a)(bs), (*rebuild_b)(right))
				}),
			}
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Semiapplicative
		for BazaarListBrand<FunctionBrand, A, B>
	{
		/// Applies a `BazaarList` of functions to a `BazaarList` of values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The input type.",
			"The output type."
		)]
		///
		#[document_parameters(
			"The `BazaarList` containing the function.",
			"The `BazaarList` containing the value."
		)]
		///
		#[document_returns("A `BazaarList` with the function applied.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bl_f = BazaarList::<RcFnBrand, i32, i32, _> {
		/// 	foci: vec![],
		/// 	rebuild: lift_fn_new::<RcFnBrand, _, _>(|_: Vec<i32>| {
		/// 		lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2)
		/// 	}),
		/// };
		/// let bl_a = BazaarList::<RcFnBrand, i32, i32, i32> {
		/// 	foci: vec![5],
		/// 	rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// };
		/// let result = apply::<RcFnBrand, BazaarListBrand<RcFnBrand, i32, i32>, _, _>(bl_f, bl_a);
		/// assert_eq!((result.rebuild)(vec![7]), 14);
		/// ```
		fn apply<'a, FnB: 'a + CloneFn, T: 'a + Clone, U: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnB as CloneFn>::Of<'a, T, U>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, T>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, U>) {
			let split_at = ff.foci.len();
			let mut foci = ff.foci;
			foci.extend(fa.foci);
			let rebuild_f = ff.rebuild;
			let rebuild_a = fa.rebuild;
			BazaarList {
				foci,
				rebuild: <FunctionBrand as LiftFn>::new(move |mut bs: Vec<B>| {
					let right = bs.split_off(split_at);
					(*rebuild_f)(bs)((*rebuild_a)(right))
				}),
			}
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> ApplyFirst
		for BazaarListBrand<FunctionBrand, A, B>
	{
	}
	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> ApplySecond
		for BazaarListBrand<FunctionBrand, A, B>
	{
	}

	// -- Bazaar --

	/// The `Bazaar` profunctor, used to characterize traversals.
	///
	/// Wraps a decomposition function from `S` to [`BazaarList`], which extracts
	/// all focus values and provides a rebuild function. Use [`run_bazaar`] to
	/// interpret the `Bazaar` with any [`Applicative`](crate::classes::Applicative).
	///
	/// This is a port of PureScript's `Bazaar ((->) :: Type -> Type -> Type) a b s t`,
	/// specialized to the function profunctor.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The type of focus values extracted from the source.",
		"The type of replacement values used during reconstruction.",
		"The source type.",
		"The target type."
	)]
	pub struct Bazaar<'a, FunctionBrand: LiftFn + 'a, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Decomposes a source into a [`BazaarList`] of foci and a rebuild function.
		pub run: <FunctionBrand as CloneFn>::Of<'a, S, BazaarList<'a, FunctionBrand, A, B, T>>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The type of focus values extracted from the source.",
		"The type of replacement values used during reconstruction.",
		"The source type.",
		"The target type."
	)]
	impl<'a, FunctionBrand: LiftFn, A: 'a, B: 'a, S: 'a, T: 'a> Bazaar<'a, FunctionBrand, A, B, S, T> {
		/// Creates a new `Bazaar` instance.
		#[document_signature]
		///
		#[document_parameters("The decomposition function from source to `BazaarList`.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bazaar =
		/// 	Bazaar::<RcFnBrand, i32, i32, i32, i32>::new(lift_fn_new::<RcFnBrand, _, _>(|s: i32| {
		/// 		BazaarList {
		/// 			foci: vec![s],
		/// 			rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// 		}
		/// 	}));
		/// let bl = (bazaar.run)(42);
		/// assert_eq!(bl.foci, vec![42]);
		/// assert_eq!((bl.rebuild)(vec![100]), 100);
		/// ```
		pub fn new(
			run: <FunctionBrand as CloneFn>::Of<'a, S, BazaarList<'a, FunctionBrand, A, B, T>>
		) -> Self {
			Bazaar {
				run,
			}
		}
	}

	/// Interprets a [`Bazaar`] with a specific [`Applicative`](crate::classes::Applicative).
	///
	/// Given a handler `A -> F B` and a source `S`, decomposes the source via the Bazaar,
	/// maps each focus through the handler, sequences the results, and rebuilds the target.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The type of focus values.",
		"The type of replacement values.",
		"The source type.",
		"The target type.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The handler function that lifts each focus into the applicative.",
		"The source value.",
		"The bazaar to interpret."
	)]
	///
	#[document_returns("The target wrapped in the applicative context.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		optics::*,
	/// 		*,
	/// 	},
	/// 	functions::*,
	/// 	types::optics::*,
	/// };
	///
	/// let bazaar =
	/// 	Bazaar::<RcFnBrand, i32, i32, Vec<i32>, Vec<i32>>::new(lift_fn_new::<RcFnBrand, _, _>(
	/// 		|s: Vec<i32>| {
	/// 			let len = s.len();
	/// 			BazaarList {
	/// 				foci: s,
	/// 				rebuild: lift_fn_new::<RcFnBrand, _, _>(move |bs: Vec<i32>| {
	/// 					bs.into_iter().take(len).collect()
	/// 				}),
	/// 			}
	/// 		},
	/// 	));
	/// let result = run_bazaar::<RcFnBrand, _, _, _, _, OptionBrand>(
	/// 	|x: i32| Some(x + 1),
	/// 	vec![1, 2, 3],
	/// 	&bazaar,
	/// );
	/// assert_eq!(result, Some(vec![2, 3, 4]));
	/// ```
	pub fn run_bazaar<'a, FunctionBrand, A, B, S, T, F>(
		handler: impl Fn(A) -> Apply!(<F as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>) + 'a,
		s: S,
		bazaar: &Bazaar<'a, FunctionBrand, A, B, S, T>,
	) -> Apply!(<F as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, T>)
	where
		FunctionBrand: LiftFn + 'static,
		A: 'a + Clone,
		B: 'a + Clone,
		S: 'a,
		T: 'a,
		F: crate::classes::Applicative,
		Apply!(<F as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<B>>): Clone,
		Apply!(<F as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, T>): Clone, {
		let bl = (bazaar.run)(s);
		let f_bs: Vec<Apply!(<F as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>)> =
			bl.foci.into_iter().map(&handler).collect();
		let f_vec_b: Apply!(<F as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<B>>) =
			VecBrand::sequence::<'a, _, F>(f_bs);
		let rebuild = bl.rebuild;
		F::map(move |bs| (*rebuild)(bs), f_vec_b)
	}

	// -- BazaarBrand --

	impl_kind! {
		impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> for BazaarBrand<FunctionBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Bazaar<'a, FunctionBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Profunctor
		for BazaarBrand<FunctionBrand, A, B>
	{
		/// Maps functions over the input and output of the `Bazaar` profunctor.
		///
		/// Corresponds to PureScript's `dimap f g (Bazaar b) = Bazaar \pafb s -> g <$> b pafb (f s)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The new source type.",
			"The original source type.",
			"The original target type.",
			"The new target type."
		)]
		///
		#[document_parameters(
			"The contravariant function to apply to the source.",
			"The covariant function to apply to the target.",
			"The bazaar instance to transform."
		)]
		///
		#[document_returns("A transformed `Bazaar` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bazaar =
		/// 	Bazaar::<RcFnBrand, i32, i32, i32, i32>::new(lift_fn_new::<RcFnBrand, _, _>(|s: i32| {
		/// 		BazaarList {
		/// 			foci: vec![s],
		/// 			rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// 		}
		/// 	}));
		/// let dimapped = <BazaarBrand<RcFnBrand, i32, i32> as Profunctor>::dimap(
		/// 	|s: String| s.parse::<i32>().unwrap(),
		/// 	|t: i32| t.to_string(),
		/// 	bazaar,
		/// );
		/// let bl = (dimapped.run)("42".to_string());
		/// assert_eq!(bl.foci, vec![42]);
		/// assert_eq!((bl.rebuild)(vec![100]), "100".to_string());
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a>(
			st: impl Fn(S) -> T + 'a,
			uv: impl Fn(U) -> V + 'a,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>) {
			let run = puv.run;
			let uv = <FunctionBrand as LiftFn>::new(uv);
			Bazaar::new(<FunctionBrand as LiftFn>::new(move |s: S| {
				let bl = (*run)(st(s));
				let rebuild = bl.rebuild;
				let uv = uv.clone();
				BazaarList {
					foci: bl.foci,
					rebuild: <FunctionBrand as LiftFn>::new(move |bs: Vec<B>| {
						(*uv)((*rebuild)(bs))
					}),
				}
			}))
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Strong
		for BazaarBrand<FunctionBrand, A, B>
	{
		/// Lifts the `Bazaar` profunctor to operate on the first component of a tuple.
		///
		/// Corresponds to PureScript's `first (Bazaar b) = Bazaar (\pafb (Tuple x y) -> flip Tuple y <$> b pafb x)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type.",
			"The target type.",
			"The type of the second component."
		)]
		///
		#[document_parameters("The bazaar instance to lift.")]
		///
		#[document_returns("A `Bazaar` that operates on pairs.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bazaar =
		/// 	Bazaar::<RcFnBrand, i32, i32, i32, i32>::new(lift_fn_new::<RcFnBrand, _, _>(|s: i32| {
		/// 		BazaarList {
		/// 			foci: vec![s],
		/// 			rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// 		}
		/// 	}));
		/// let lifted = <BazaarBrand<RcFnBrand, i32, i32> as Strong>::first::<i32, i32, String>(bazaar);
		/// let bl = (lifted.run)((42, "hello".to_string()));
		/// assert_eq!(bl.foci, vec![42]);
		/// assert_eq!((bl.rebuild)(vec![100]), (100, "hello".to_string()));
		/// ```
		fn first<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
			let run = pab.run;
			Bazaar::new(<FunctionBrand as LiftFn>::new(move |(s, c): (S, C)| {
				let bl = (*run)(s);
				let rebuild = bl.rebuild;
				let c = Ptr::<FunctionBrand>::take_cell_new(c);
				BazaarList {
					foci: bl.foci,
					rebuild: <FunctionBrand as LiftFn>::new(move |bs: Vec<B>| {
						// SAFETY: take_cell_take is called exactly once per the optics rebuild contract
						#[allow(clippy::expect_used)]
						let c = Ptr::<FunctionBrand>::take_cell_take(&c)
							.expect("BazaarList rebuild called more than once");
						((*rebuild)(bs), c)
					}),
				}
			}))
		}

		/// Lifts the `Bazaar` profunctor to operate on the second component of a tuple.
		///
		/// Corresponds to PureScript's `second (Bazaar b) = Bazaar (\pafb (Tuple x y) -> Tuple x <$> b pafb y)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type.",
			"The target type.",
			"The type of the first component."
		)]
		///
		#[document_parameters("The bazaar instance to lift.")]
		///
		#[document_returns("A `Bazaar` that operates on pairs.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bazaar =
		/// 	Bazaar::<RcFnBrand, i32, i32, i32, i32>::new(lift_fn_new::<RcFnBrand, _, _>(|s: i32| {
		/// 		BazaarList {
		/// 			foci: vec![s],
		/// 			rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// 		}
		/// 	}));
		/// let lifted = <BazaarBrand<RcFnBrand, i32, i32> as Strong>::second::<i32, i32, String>(bazaar);
		/// let bl = (lifted.run)(("hello".to_string(), 42));
		/// assert_eq!(bl.foci, vec![42]);
		/// assert_eq!((bl.rebuild)(vec![100]), ("hello".to_string(), 100));
		/// ```
		fn second<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, S), (C, T)>) {
			let run = pab.run;
			Bazaar::new(<FunctionBrand as LiftFn>::new(move |(c, s): (C, S)| {
				let bl = (*run)(s);
				let rebuild = bl.rebuild;
				let c = Ptr::<FunctionBrand>::take_cell_new(c);
				BazaarList {
					foci: bl.foci,
					rebuild: <FunctionBrand as LiftFn>::new(move |bs: Vec<B>| {
						// SAFETY: take_cell_take is called exactly once per the optics rebuild contract
						#[allow(clippy::expect_used)]
						let c = Ptr::<FunctionBrand>::take_cell_take(&c)
							.expect("BazaarList rebuild called more than once");
						(c, (*rebuild)(bs))
					}),
				}
			}))
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static, B: 'static> Choice
		for BazaarBrand<FunctionBrand, A, B>
	{
		/// Lifts the `Bazaar` profunctor to operate on the `Err` variant of a `Result`.
		///
		/// Corresponds to PureScript's `left (Bazaar b) = Bazaar (\pafb e -> bitraverse (b pafb) pure e)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type.",
			"The target type.",
			"The type of the `Ok` variant."
		)]
		///
		#[document_parameters("The bazaar instance to lift.")]
		///
		#[document_returns("A `Bazaar` that operates on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bazaar =
		/// 	Bazaar::<RcFnBrand, i32, i32, i32, i32>::new(lift_fn_new::<RcFnBrand, _, _>(|s: i32| {
		/// 		BazaarList {
		/// 			foci: vec![s],
		/// 			rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// 		}
		/// 	}));
		/// let lifted = <BazaarBrand<RcFnBrand, i32, i32> as Choice>::left::<i32, i32, String>(bazaar);
		/// let bl_err = (lifted.run)(Err(42));
		/// assert_eq!(bl_err.foci, vec![42]);
		/// assert_eq!((bl_err.rebuild)(vec![100]), Err(100));
		/// let bl_ok = (lifted.run)(Ok("hello".to_string()));
		/// assert_eq!(bl_ok.foci.len(), 0);
		/// assert_eq!((bl_ok.rebuild)(vec![]), Ok("hello".to_string()));
		/// ```
		fn left<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, S>, Result<C, T>>)
		{
			let run = pab.run;
			Bazaar::new(<FunctionBrand as LiftFn>::new(move |r: Result<C, S>| match r {
				Err(s) => {
					let bl = (*run)(s);
					let rebuild = bl.rebuild;
					BazaarList {
						foci: bl.foci,
						rebuild: <FunctionBrand as LiftFn>::new(move |bs: Vec<B>| {
							Err((*rebuild)(bs))
						}),
					}
				}
				Ok(c) => {
					let c = Ptr::<FunctionBrand>::take_cell_new(c);
					BazaarList {
						foci: vec![],
						rebuild: <FunctionBrand as LiftFn>::new(move |_: Vec<B>| {
							// SAFETY: take_cell_take is called exactly once per the optics rebuild contract
							#[allow(clippy::expect_used)]
							Ok(Ptr::<FunctionBrand>::take_cell_take(&c)
								.expect("BazaarList rebuild called more than once"))
						}),
					}
				}
			}))
		}

		/// Lifts the `Bazaar` profunctor to operate on the `Ok` variant of a `Result`.
		///
		/// Corresponds to PureScript's `right (Bazaar b) = Bazaar (\pafb e -> traverse (b pafb) e)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The source type.",
			"The target type.",
			"The type of the `Err` variant."
		)]
		///
		#[document_parameters("The bazaar instance to lift.")]
		///
		#[document_returns("A `Bazaar` that operates on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let bazaar =
		/// 	Bazaar::<RcFnBrand, i32, i32, i32, i32>::new(lift_fn_new::<RcFnBrand, _, _>(|s: i32| {
		/// 		BazaarList {
		/// 			foci: vec![s],
		/// 			rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// 		}
		/// 	}));
		/// let lifted = <BazaarBrand<RcFnBrand, i32, i32> as Choice>::right::<i32, i32, String>(bazaar);
		/// let bl_ok = (lifted.run)(Ok(42));
		/// assert_eq!(bl_ok.foci, vec![42]);
		/// assert_eq!((bl_ok.rebuild)(vec![100]), Ok(100));
		/// let bl_err = (lifted.run)(Err("oops".to_string()));
		/// assert_eq!(bl_err.foci.len(), 0);
		/// assert_eq!((bl_err.rebuild)(vec![]), Err("oops".to_string()));
		/// ```
		fn right<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<S, C>, Result<T, C>>)
		{
			let run = pab.run;
			Bazaar::new(<FunctionBrand as LiftFn>::new(move |r: Result<S, C>| match r {
				Ok(s) => {
					let bl = (*run)(s);
					let rebuild = bl.rebuild;
					BazaarList {
						foci: bl.foci,
						rebuild: <FunctionBrand as LiftFn>::new(move |bs: Vec<B>| {
							Ok((*rebuild)(bs))
						}),
					}
				}
				Err(c) => {
					let c = Ptr::<FunctionBrand>::take_cell_new(c);
					BazaarList {
						foci: vec![],
						rebuild: <FunctionBrand as LiftFn>::new(move |_: Vec<B>| {
							// SAFETY: take_cell_take is called exactly once per the optics rebuild contract
							#[allow(clippy::expect_used)]
							Err(Ptr::<FunctionBrand>::take_cell_take(&c)
								.expect("BazaarList rebuild called more than once"))
						}),
					}
				}
			}))
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The focus type.",
		"The replacement type."
	)]
	impl<FunctionBrand: LiftFn + 'static, A: 'static + Clone, B: 'static + Clone> Wander
		for BazaarBrand<FunctionBrand, A, B>
	{
		/// Lifts the `Bazaar` profunctor through a traversal.
		///
		/// Corresponds to PureScript's `wander w (Bazaar f) = Bazaar (\pafb s -> w (f pafb) s)`.
		/// Uses `BazaarListBrand` as the applicative to decompose the traversal structure.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The outer source type.",
			"The outer target type.",
			"The inner source type (focus of the traversal).",
			"The inner target type."
		)]
		///
		#[document_parameters("The traversal function.", "The bazaar instance to compose with.")]
		///
		#[document_returns("A `Bazaar` that traverses the outer structure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		Applicative,
		/// 		optics::traversal::TraversalFunc,
		/// 		profunctor::*,
		/// 	},
		/// 	functions::*,
		/// 	kinds::*,
		/// 	types::optics::*,
		/// };
		///
		/// // A traversal over Vec elements
		/// #[derive(Clone)]
		/// struct VecTraversal;
		/// impl<'a, X: 'a + Clone> TraversalFunc<'a, Vec<X>, Vec<X>, X, X> for VecTraversal {
		/// 	fn apply<M: Applicative>(
		/// 		&self,
		/// 		f: Box<dyn Fn(X) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, X>) + 'a>,
		/// 		s: Vec<X>,
		/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<X>>) {
		/// 		s.into_iter().fold(M::pure(vec![]), |acc, a| {
		/// 			M::lift2(
		/// 				|mut v: Vec<X>, x: X| {
		/// 					v.push(x);
		/// 					v
		/// 				},
		/// 				acc,
		/// 				f(a),
		/// 			)
		/// 		})
		/// 	}
		/// }
		///
		/// // Identity bazaar: each element maps to itself
		/// let id_bazaar =
		/// 	Bazaar::<RcFnBrand, i32, i32, i32, i32>::new(lift_fn_new::<RcFnBrand, _, _>(|s: i32| {
		/// 		BazaarList {
		/// 			foci: vec![s],
		/// 			rebuild: lift_fn_new::<RcFnBrand, _, _>(|bs: Vec<i32>| bs[0]),
		/// 		}
		/// 	}));
		/// let wandered =
		/// 	<BazaarBrand<RcFnBrand, i32, i32> as Wander>::wander::<Vec<i32>, Vec<i32>, i32, i32>(
		/// 		VecTraversal,
		/// 		id_bazaar,
		/// 	);
		/// let bl = (wandered.run)(vec![10, 20, 30]);
		/// assert_eq!(bl.foci, vec![10, 20, 30]);
		/// assert_eq!((bl.rebuild)(vec![1, 2, 3]), vec![1, 2, 3]);
		/// ```
		fn wander<'a, S: 'a, T: 'a, A2: 'a, B2: 'a + Clone>(
			traversal: impl TraversalFunc<'a, S, T, A2, B2> + 'a,
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A2, B2>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>) {
			let run = pab.run;
			Bazaar::new(<FunctionBrand as LiftFn>::new(move |s: S| {
				let run = run.clone();
				traversal.apply::<BazaarListBrand<FunctionBrand, A, B>>(
					Box::new(move |a2: A2| (*run)(a2)),
					s,
				)
			}))
		}
	}
}
pub use inner::*;

impl<'a, FB: crate::classes::clone_fn::LiftFn + 'static, A: Clone + 'a, B: 'a, T: 'a> Clone
	for BazaarList<'a, FB, A, B, T>
where
	<FB as crate::classes::clone_fn::CloneFn>::Of<'a, Vec<B>, T>: Clone,
{
	fn clone(&self) -> Self {
		BazaarList {
			foci: self.foci.clone(),
			rebuild: self.rebuild.clone(),
		}
	}
}
