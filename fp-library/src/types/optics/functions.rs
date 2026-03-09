//! Helper functions for working with optics.

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
				Function,
				Profunctor,
				UnsizedCoercible,
				applicative::Applicative,
				monoid::Monoid,
				optics::{
					indexed_traversal::IndexedTraversalFunc,
					*,
				},
				semigroup::Semigroup,
			},
			kinds::*,
			types::optics::{
				Exchange,
				Forget,
				Indexed,
				IndexedTraversal,
				Tagged,
				Traversal,
				Zipping,
			},
		},
		fp_macros::*,
	};

	/// View the focus of a lens-like optic.
	///
	/// This is a convenience function that works with any lens-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The lens optic.", "The structure to view.")]
	///
	/// ### Returns
	///
	/// The focus value.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::*,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::from_view_set(|(x, _)| x, |(_, x)| (x, "".to_string()));
	/// assert_eq!(optics_view::<RcBrand, _, _, _>(&l, (42, "hello".to_string())), 42);
	/// ```
	pub fn optics_view<'a, PointerBrand, O, S, A>(
		optic: &O,
		s: S,
	) -> A
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: GetterOptic<'a, S, A>,
		S: 'a,
		A: 'a + 'static, {
		(optic.evaluate::<A, PointerBrand>(Forget::new(|a| a)).0)(s)
	}

	/// Set the focus of a lens-like optic.
	///
	/// This is a convenience function that works with any lens-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The lens optic.", "The structure to update.", "The new focus value.")]
	///
	/// ### Returns
	///
	/// The updated structure.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::*,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_set::<RcBrand, _, _, _>(&l, (42, "hello".to_string()), 99),
	/// 	(99, "hello".to_string())
	/// );
	/// ```
	pub fn optics_set<'a, PointerBrand, O, S, A>(
		optic: &O,
		s: S,
		a: A,
	) -> S
	where
		PointerBrand: UnsizedCoercible,
		O: SetterOptic<'a, PointerBrand, S, S, A, A>,
		S: 'a,
		A: 'a + Clone, {
		let f = <FnBrand<PointerBrand> as Function>::new(move |_| a.clone());
		(optic.evaluate(f))(s)
	}

	/// Modify the focus of a lens-like optic using a function.
	///
	/// This is a convenience function that works with any lens-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters(
		"The lens optic.",
		"The structure to update.",
		"The function to apply to the focus."
	)]
	///
	/// ### Returns
	///
	/// The updated structure.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::*,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_over::<RcBrand, _, _, _>(&l, (42, "hello".to_string()), |x| x * 2),
	/// 	(84, "hello".to_string())
	/// );
	/// ```
	pub fn optics_over<'a, PointerBrand, O, S, A>(
		optic: &O,
		s: S,
		f: impl Fn(A) -> A + 'a,
	) -> S
	where
		PointerBrand: UnsizedCoercible,
		O: SetterOptic<'a, PointerBrand, S, S, A, A>,
		S: 'a,
		A: 'a, {
		let f = <FnBrand<PointerBrand> as Function>::new(f);
		(optic.evaluate(f))(s)
	}

	/// Preview the focus of a prism-like optic.
	///
	/// This is a convenience function that works with any prism-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The prism optic.", "The structure to preview.")]
	///
	/// ### Returns
	///
	/// An `Option` containing the focus value if it exists.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::*,
	/// 	types::optics::*,
	/// };
	///
	/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
	/// 	PrismPrime::new(|r: Result<i32, String>| r.map_err(|e| Err(e)), |x| Ok(x));
	/// assert_eq!(optics_preview::<RcBrand, _, _, _>(&ok_prism, Ok(42)), Some(42));
	/// assert_eq!(optics_preview::<RcBrand, _, _, _>(&ok_prism, Err("error".to_string())), None);
	/// ```
	pub fn optics_preview<'a, PointerBrand, O, S, A>(
		optic: &O,
		s: S,
	) -> Option<A>
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: FoldOptic<'a, S, A>,
		S: 'a,
		A: 'a + 'static + Clone, {
		#[derive(Clone)]
		struct First<A>(Option<A>);
		impl<A> Semigroup for First<A> {
			fn append(
				a: Self,
				b: Self,
			) -> Self {
				First(a.0.or(b.0))
			}
		}
		impl<A> Monoid for First<A> {
			fn empty() -> Self {
				First(None)
			}
		}

		let forget = Forget::new(|a| First(Some(a)));
		let result_forget = optic.evaluate::<First<A>, PointerBrand>(forget);
		(result_forget.0)(s).0
	}

	/// Review a focus value into a structure using a prism-like optic.
	///
	/// This is a convenience function that works with any prism-based optic.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The prism optic.", "The focus value.")]
	///
	/// ### Returns
	///
	/// The structure containing the focus value.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::*,
	/// 	types::optics::*,
	/// };
	///
	/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
	/// 	PrismPrime::new(|r: Result<i32, String>| r.map_err(|e| Err(e)), |x| Ok(x));
	/// assert_eq!(optics_review(&ok_prism, 42), Ok(42));
	/// ```
	pub fn optics_review<'a, O, S, A>(
		optic: &O,
		a: A,
	) -> S
	where
		O: ReviewOptic<'a, S, S, A, A>,
		S: 'a,
		A: 'a, {
		(optic.evaluate(Tagged::new(a))).0
	}

	/// Apply an isomorphism in the forward direction.
	///
	/// This is a convenience function that converts from structure to focus.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The iso optic.", "The structure to convert.")]
	///
	/// ### Returns
	///
	/// The focus value.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		RcFnBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::*,
	/// 	types::optics::*,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// assert_eq!(optics_from::<RcFnBrand, _, _, _>(&iso, (42,)), 42);
	/// ```
	pub fn optics_from<'a, FunctionBrand, O, S, A>(
		optic: &O,
		s: S,
	) -> A
	where
		FunctionBrand: CloneableFn + 'static,
		O: IsoOptic<'a, S, S, A, A>,
		S: 'a,
		A: 'a + 'static, {
		let exchange = Exchange::new(
			<FunctionBrand as CloneableFn>::new(|a| a),
			<FunctionBrand as CloneableFn>::new(|a| a),
		);
		(optic.evaluate::<ExchangeBrand<FunctionBrand, A, A>>(exchange).get)(s)
	}

	/// Apply an isomorphism in the backward direction.
	///
	/// This is a convenience function that converts from focus to structure.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The optic type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	///
	#[document_parameters("The iso optic.", "The focus value to convert.")]
	///
	/// ### Returns
	///
	/// The structure.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		RcFnBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::*,
	/// 	types::optics::*,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// assert_eq!(optics_to::<RcFnBrand, _, _, _>(&iso, 42), (42,));
	/// ```
	pub fn optics_to<'a, FunctionBrand, O, S, A>(
		optic: &O,
		a: A,
	) -> S
	where
		FunctionBrand: CloneableFn + 'static,
		O: IsoOptic<'a, S, S, A, A>,
		S: 'a,
		A: 'a + 'static, {
		let exchange = Exchange::new(
			<FunctionBrand as CloneableFn>::new(|a| a),
			<FunctionBrand as CloneableFn>::new(|a| a),
		);
		(optic.evaluate::<ExchangeBrand<FunctionBrand, A, A>>(exchange).set)(a)
	}

	/// Evaluate an optic with a profunctor.
	///
	/// This is the most general function for working with optics, allowing you to
	/// evaluate any optic with any compatible profunctor.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The optic type.",
		"The type of the structure.",
		"The target type after update.",
		"The type of the focus.",
		"The target focus type after update."
	)]
	///
	#[document_parameters("The optic.", "The profunctor value.")]
	///
	/// ### Returns
	///
	/// The transformed profunctor value.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		optics::*,
	/// 		*,
	/// 	},
	/// 	classes::optics::*,
	/// 	functions::*,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let modifier = optics_eval::<RcFnBrand, _, _, _, _, _>(&l, f);
	/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
	/// ```
	pub fn optics_eval<'a, P, O, S: 'a, T: 'a, A: 'a, B: 'a>(
		optic: &O,
		pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
	where
		P: Profunctor,
		O: Optic<'a, P, S, T, A, B>, {
		optic.evaluate(pab)
	}

	/// Zip two structures together using a grate optic and a combining function.
	///
	/// Matches PureScript's `zipWithOf :: Grate s t a b -> (a -> a -> b) -> s -> s -> t`.
	///
	/// Uses the `Zipping` profunctor internally: the grate optic lifts the combining
	/// function into a `Zipping<S, T>` (a binary function on `S`), which is then applied
	/// to the two input structures.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand for `Zipping`.",
		"The optic type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	///
	#[document_parameters(
		"The grate optic.",
		"The combining function, taking a pair `(A, A)` and returning `B`.",
		"The first structure.",
		"The second structure."
	)]
	///
	/// ### Returns
	///
	/// The combined structure.
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::{
	/// 			RcBrand,
	/// 			RcFnBrand,
	/// 			optics::*,
	/// 		},
	/// 		classes::CloneableFn,
	/// 		types::optics::{
	/// 			GratePrime,
	/// 			zip_with_of,
	/// 		},
	/// 	},
	/// 	std::rc::Rc,
	/// };
	///
	/// let grate = GratePrime::<RcBrand, (i32, i32), i32>::new(|f| {
	/// 	(
	/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.0) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
	/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.1) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
	/// 	)
	/// });
	/// let result = zip_with_of::<RcFnBrand, _, _, _, _, _>(&grate, |(a, b)| a + b, (1, 2), (10, 20));
	/// assert_eq!(result, (11, 22));
	/// ```
	pub fn zip_with_of<'a, FunctionBrand, O, S, T, A, B>(
		optic: &O,
		f: impl Fn((A, A)) -> B + 'a,
		s1: S,
		s2: S,
	) -> T
	where
		FunctionBrand: CloneableFn + 'static,
		O: GrateOptic<'a, FunctionBrand, S, T, A, B>,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		let zipping = Zipping::<FunctionBrand, A, B>::new(f);
		let result: Zipping<'a, FunctionBrand, S, T> =
			GrateOptic::<FunctionBrand, S, T, A, B>::evaluate::<ZippingBrand<FunctionBrand>>(
				optic, zipping,
			);
		(*result.run)((s1, s2))
	}

	/// View the focus and its index of an indexed lens-like optic.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed lens optic.", "The structure to view.")]
	#[document_returns("The focus value and its index.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	types::optics::*,
	/// };
	/// let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	/// 	IndexedLensPrime::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
	/// assert_eq!(optics_indexed_view::<RcBrand, _, _, _, _>(&l, (42, "hello".to_string())), (0, 42));
	/// ```
	pub fn optics_indexed_view<'a, PointerBrand, O, I, S, A>(
		optic: &O,
		s: S,
	) -> (I, A)
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: IndexedGetterOptic<'a, I, S, A>,
		I: 'a + 'static,
		S: 'a,
		A: 'a + 'static, {
		(optic.evaluate::<(I, A), PointerBrand>(Indexed::new(Forget::new(|ia| ia))).0)(s)
	}

	/// Modify the focus of an indexed lens-like optic using an indexed function.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters(
		"The indexed lens optic.",
		"The structure to update.",
		"The function to apply to the focus."
	)]
	#[document_returns("The updated structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	types::optics::*,
	/// };
	/// let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	/// 	IndexedLensPrime::from_iview_set(|(x, _)| (10, x), |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_indexed_over::<RcBrand, _, _, _, _>(&l, (42, "hello".to_string()), |i, x| x
	/// 		+ (i as i32)),
	/// 	(52, "hello".to_string())
	/// );
	/// ```
	pub fn optics_indexed_over<'a, PointerBrand, O, I, S, A>(
		optic: &O,
		s: S,
		f: impl Fn(I, A) -> A + 'a,
	) -> S
	where
		PointerBrand: UnsizedCoercible,
		O: IndexedSetterOptic<'a, PointerBrand, I, S, S, A, A>,
		I: 'a,
		S: 'a,
		A: 'a, {
		let f_brand = <FnBrand<PointerBrand> as CloneableFn>::new(move |(i, a)| f(i, a));
		(optic.evaluate(Indexed::new(f_brand)))(s)
	}

	/// Set the focus of an indexed lens-like optic.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters(
		"The indexed lens optic.",
		"The structure to update.",
		"The new focus value."
	)]
	#[document_returns("The updated structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	types::optics::*,
	/// };
	/// let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	/// 	IndexedLensPrime::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_indexed_set::<RcBrand, _, _, _, _>(&l, (42, "hello".to_string()), 99),
	/// 	(99, "hello".to_string())
	/// );
	/// ```
	pub fn optics_indexed_set<'a, PointerBrand, O, I, S, A>(
		optic: &O,
		s: S,
		a: A,
	) -> S
	where
		PointerBrand: UnsizedCoercible,
		O: IndexedSetterOptic<'a, PointerBrand, I, S, S, A, A>,
		I: 'a,
		S: 'a,
		A: 'a + Clone, {
		optics_indexed_over::<PointerBrand, _, _, _, _>(optic, s, move |_, _| a.clone())
	}

	/// Preview the focus and its index of an indexed prism-like optic.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The indexed prism optic.", "The structure to preview.")]
	#[document_returns("An `Option` containing the focus value and its index if it exists.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	types::optics::*,
	/// };
	/// let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	/// 	IndexedLensPrime::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_indexed_preview::<RcBrand, _, _, _, _>(&l, (42, "hello".to_string())),
	/// 	Some((0, 42))
	/// );
	/// ```
	pub fn optics_indexed_preview<'a, PointerBrand, O, I, S, A>(
		optic: &O,
		s: S,
	) -> Option<(I, A)>
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: IndexedFoldOptic<'a, I, S, A>,
		I: 'a + Clone + 'static,
		S: 'a,
		A: 'a + 'static + Clone, {
		#[derive(Clone)]
		struct First<A>(Option<A>);
		impl<A> Semigroup for First<A> {
			fn append(
				a: Self,
				b: Self,
			) -> Self {
				First(a.0.or(b.0))
			}
		}
		impl<A> Monoid for First<A> {
			fn empty() -> Self {
				First(None)
			}
		}

		let forget = Forget::new(|ia| First(Some(ia)));
		let result_forget = optic.evaluate::<First<(I, A)>, PointerBrand>(Indexed::new(forget));
		(result_forget.0)(s).0
	}

	/// Fold with index.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The pointer brand for the function.",
		"The optic type.",
		"The index type.",
		"The type of the structure.",
		"The type of the focus.",
		"The monoid type to fold into."
	)]
	#[document_parameters(
		"The indexed fold optic.",
		"The mapping function.",
		"The structure to fold."
	)]
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	types::optics::*,
	/// };
	/// let l: IndexedLensPrime<RcBrand, usize, (i32, String), i32> =
	/// 	IndexedLensPrime::from_iview_set(|(x, _)| (0, x), |((_, s), x)| (x, s));
	/// assert_eq!(
	/// 	optics_indexed_fold_map::<RcBrand, _, _, _, _, String>(
	/// 		&l,
	/// 		|i, x| format!("{}:{}", i, x),
	/// 		(42, "hi".to_string())
	/// 	),
	/// 	"0:42".to_string()
	/// );
	/// ```
	pub fn optics_indexed_fold_map<'a, PointerBrand, O, I, S, A, R>(
		optic: &O,
		f: impl Fn(I, A) -> R + 'a,
		s: S,
	) -> R
	where
		PointerBrand: UnsizedCoercible + 'static,
		O: IndexedFoldOptic<'a, I, S, A>,
		I: 'a,
		S: 'a,
		A: 'a,
		R: Monoid + Clone + 'a + 'static, {
		let forget = Forget::new(move |(i, a)| f(i, a));
		let result_forget = optic.evaluate::<R, PointerBrand>(Indexed::new(forget));
		(result_forget.0)(s)
	}

	/// Convert an indexed optic to a regular optic by ignoring the index.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The optic type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The indexed optic.")]
	#[document_returns("A regular optic that ignores the index.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		RcFnBrand,
	/// 		optics::*,
	/// 	},
	/// 	functions::*,
	/// 	types::optics::*,
	/// };
	/// let l = std::mem::ManuallyDrop::new(
	/// 	IndexedLensPrime::<RcBrand, usize, (i32, String), i32>::from_iview_set(
	/// 		|(x, _)| (0, x),
	/// 		|((_, s), x)| (x, s),
	/// 	),
	/// );
	/// let unindexed = optics_un_index::<RcFnBrand, _, _, _, _, _, _>(&*l);
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let modifier = optics_eval::<RcFnBrand, _, _, _, _, _>(&unindexed, f);
	/// assert_eq!(modifier((42, "hi".to_string())), (43, "hi".to_string()));
	/// ```
	pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
		optic: &'a O
	) -> impl Optic<'a, P, S, T, A, B> + 'a
	where
		P: Profunctor + 'static,
		O: IndexedOpticAdapter<'a, P, I, S, T, A, B>,
		I: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		struct UnIndex<'a, P, O, I, S, T, A, B> {
			optic: &'a O,
			_phantom: std::marker::PhantomData<(&'a (I, S, T, A, B), P)>,
		}
		impl<'a, P, O, I, S, T, A, B> Optic<'a, P, S, T, A, B> for UnIndex<'a, P, O, I, S, T, A, B>
		where
			P: Profunctor + 'static,
			O: IndexedOpticAdapter<'a, P, I, S, T, A, B>,
			I: 'a,
			S: 'a,
			T: 'a,
			A: 'a,
			B: 'a,
		{
			#[document_signature]
			#[document_parameters("The profunctor value.")]
			#[document_returns("The transformed profunctor value.")]
			#[document_examples]
			///
			/// ```
			/// use fp_library::{
			/// 	brands::{
			/// 		RcBrand,
			/// 		RcFnBrand,
			/// 		optics::*,
			/// 	},
			/// 	functions::*,
			/// 	types::optics::*,
			/// };
			/// let l = std::mem::ManuallyDrop::new(
			/// 	IndexedLensPrime::<RcBrand, usize, (i32, String), i32>::from_iview_set(
			/// 		|(x, _)| (0, x),
			/// 		|((_, s), x)| (x, s),
			/// 	),
			/// );
			/// let unindexed = optics_un_index::<RcFnBrand, _, _, _, _, _, _>(&*l);
			/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
			/// let modifier = optics_eval::<RcFnBrand, _, _, _, _, _>(&unindexed, f);
			/// assert_eq!(modifier((42, "hi".to_string())), (43, "hi".to_string()));
			/// ```
			fn evaluate(
				&self,
				pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
			) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
				self.optic.evaluate_indexed(Indexed::new(P::dimap(move |(_, a)| a, |b| b, pab)))
			}
		}
		UnIndex {
			optic,
			_phantom: std::marker::PhantomData,
		}
	}

	/// Extract only the index, discarding the focus.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The optic type.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus."
	)]
	#[document_parameters("The indexed optic.")]
	#[document_returns("A regular optic that focuses on the index.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		RcFnBrand,
	/// 		optics::*,
	/// 	},
	/// 	functions::*,
	/// 	types::optics::*,
	/// };
	/// let l = std::mem::ManuallyDrop::new(
	/// 	IndexedLensPrime::<RcBrand, usize, (i32, String), i32>::from_iview_set(
	/// 		|(x, _)| (10, x),
	/// 		|((_, s), x)| (x, s),
	/// 	),
	/// );
	/// let as_index = optics_as_index::<RcFnBrand, _, _, _, _, _, _>(&*l);
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|i: usize| i as i32 + 1);
	/// let modifier = optics_eval::<RcFnBrand, _, _, _, _, _>(&as_index, f);
	/// assert_eq!(modifier((42, "hi".to_string())), (11, "hi".to_string()));
	/// ```
	pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
		optic: &'a O
	) -> impl Optic<'a, P, S, T, I, B> + 'a
	where
		P: Profunctor + 'static,
		O: IndexedOpticAdapterDiscardsFocus<'a, P, I, S, T, A, B>,
		I: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		struct AsIndex<'a, P, O, I, S, T, A, B> {
			optic: &'a O,
			_phantom: std::marker::PhantomData<(&'a (I, S, T, A, B), P)>,
		}
		impl<'a, P, O, I, S, T, A, B> Optic<'a, P, S, T, I, B> for AsIndex<'a, P, O, I, S, T, A, B>
		where
			P: Profunctor + 'static,
			O: IndexedOpticAdapterDiscardsFocus<'a, P, I, S, T, A, B>,
			I: 'a,
			S: 'a,
			T: 'a,
			A: 'a,
			B: 'a,
		{
			#[document_signature]
			#[document_parameters("The profunctor value.")]
			#[document_returns("The transformed profunctor value.")]
			#[document_examples]
			///
			/// ```
			/// use fp_library::{
			/// 	brands::{
			/// 		RcBrand,
			/// 		RcFnBrand,
			/// 		optics::*,
			/// 	},
			/// 	functions::*,
			/// 	types::optics::*,
			/// };
			/// let l = std::mem::ManuallyDrop::new(
			/// 	IndexedLensPrime::<RcBrand, usize, (i32, String), i32>::from_iview_set(
			/// 		|(x, _)| (10, x),
			/// 		|((_, s), x)| (x, s),
			/// 	),
			/// );
			/// let as_index = optics_as_index::<RcFnBrand, _, _, _, _, _, _>(&*l);
			/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|i: usize| i as i32 + 1);
			/// let modifier = optics_eval::<RcFnBrand, _, _, _, _, _>(&as_index, f);
			/// assert_eq!(modifier((42, "hi".to_string())), (11, "hi".to_string()));
			/// ```
			fn evaluate(
				&self,
				pib: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, I, B>),
			) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
				self.optic.evaluate_indexed_discards_focus(Indexed::new(P::dimap(
					|(i, _)| i,
					|b| b,
					pib,
				)))
			}
		}
		AsIndex {
			optic,
			_phantom: std::marker::PhantomData,
		}
	}

	/// Remap index type.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The optic type.",
		"The original index type.",
		"The new index type.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The remapping function type."
	)]
	#[document_parameters("The remapping function.", "The indexed optic.")]
	#[document_returns("A reindexed optic.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		RcBrand,
	/// 		RcFnBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::optics::IndexedOpticAdapter,
	/// 	types::optics::*,
	/// };
	/// let l = std::mem::ManuallyDrop::new(
	/// 	IndexedLensPrime::<RcBrand, usize, (i32, String), i32>::from_iview_set(
	/// 		|(x, _)| (0, x),
	/// 		|((_, s), x)| (x, s),
	/// 	),
	/// );
	/// let reindexed = optics_reindexed::<RcFnBrand, _, _, String, _, _, _, _, _>(
	/// 	|i: usize| format!("{}", i),
	/// 	&*l,
	/// );
	/// let f = std::rc::Rc::new(|(i, x): (String, i32)| x + i.len() as i32)
	/// 	as std::rc::Rc<dyn Fn((String, i32)) -> i32>;
	/// let pab = Indexed::<RcFnBrand, _, _, _>::new(f);
	/// let result: std::rc::Rc<dyn Fn((i32, String)) -> (i32, String)> =
	/// 	reindexed.evaluate_indexed(pab);
	/// assert_eq!(result((42, "hi".to_string())), (43, "hi".to_string()));
	/// ```
	pub fn optics_reindexed<'a, P, O, I, J, S, T, A, B, F>(
		f: F,
		optic: &'a O,
	) -> impl IndexedOpticAdapter<'a, P, J, S, T, A, B> + 'a
	where
		P: Profunctor + 'static,
		O: IndexedOpticAdapter<'a, P, I, S, T, A, B>,
		I: 'a,
		J: 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
		F: Fn(I) -> J + Clone + 'a, {
		struct Reindexed<'a, P, O, I, J, S, T, A, B, F> {
			f: F,
			optic: &'a O,
			_phantom: std::marker::PhantomData<(&'a (I, J, S, T, A, B), P)>,
		}
		impl<'a, P, O, I, J, S, T, A, B, F> IndexedOpticAdapter<'a, P, J, S, T, A, B>
			for Reindexed<'a, P, O, I, J, S, T, A, B, F>
		where
			P: Profunctor + 'static,
			O: IndexedOpticAdapter<'a, P, I, S, T, A, B>,
			I: 'a,
			J: 'a,
			S: 'a,
			T: 'a,
			A: 'a,
			B: 'a,
			F: Fn(I) -> J + Clone + 'a,
		{
			#[document_signature]
			#[document_parameters("The indexed profunctor value.")]
			#[document_returns("The transformed profunctor value.")]
			#[document_examples]
			///
			/// ```
			/// use fp_library::{
			/// 	brands::{
			/// 		RcBrand,
			/// 		RcFnBrand,
			/// 		optics::*,
			/// 	},
			/// 	classes::optics::IndexedOpticAdapter,
			/// 	types::optics::*,
			/// };
			/// let l = std::mem::ManuallyDrop::new(
			/// 	IndexedLensPrime::<RcBrand, usize, (i32, String), i32>::from_iview_set(
			/// 		|(x, _)| (0, x),
			/// 		|((_, s), x)| (x, s),
			/// 	),
			/// );
			/// let reindexed = optics_reindexed::<RcFnBrand, _, _, String, _, _, _, _, _>(
			/// 	|i: usize| format!("{}", i),
			/// 	&*l,
			/// );
			/// let f = std::rc::Rc::new(|(i, x): (String, i32)| x + i.len() as i32)
			/// 	as std::rc::Rc<dyn Fn((String, i32)) -> i32>;
			/// let pab = Indexed::<RcFnBrand, _, _, _>::new(f);
			/// let result: std::rc::Rc<dyn Fn((i32, String)) -> (i32, String)> =
			/// 	reindexed.evaluate_indexed(pab);
			/// assert_eq!(result((42, "hi".to_string())), (43, "hi".to_string()));
			/// ```
			fn evaluate_indexed(
				&self,
				pab: Indexed<'a, P, J, A, B>,
			) -> Apply!(<P as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, T>) {
				let f = self.f.clone();
				let inner = pab.inner;
				let dimapped = P::dimap(move |(i, a)| (f(i), a), |b| b, inner);
				self.optic.evaluate_indexed(Indexed {
					inner: dimapped,
				})
			}
		}
		Reindexed {
			f,
			optic,
			_phantom: std::marker::PhantomData,
		}
	}

	/// Internal traversal function for `positions`.
	#[derive(Clone)]
	pub struct PositionsTraversalFunc<F>(F);

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source structure type.",
		"The target structure type.",
		"The source focus type.",
		"The target focus type.",
		"The traversal function type."
	)]
	#[document_parameters("The positions traversal function instance.")]
	impl<'a, S: 'a, T: 'a, A: 'a, B: 'a, F> IndexedTraversalFunc<'a, usize, S, T, A, B>
		for PositionsTraversalFunc<F>
	where
		F: TraversalFunc<'a, S, T, A, B> + Clone + 'a,
	{
		#[document_signature]
		#[document_type_parameters("The applicative context.")]
		#[document_parameters("The traversal function.", "The structure to traverse.")]
		#[document_returns("The traversed structure wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::{
		/// 		OptionBrand,
		/// 		RcBrand,
		/// 		optics::*,
		/// 	},
		/// 	classes::{
		/// 		Applicative,
		/// 		lift::Lift,
		/// 		optics::{
		/// 			IndexedTraversalFunc,
		/// 			traversal::TraversalFunc,
		/// 		},
		/// 	},
		/// 	functions::*,
		/// 	kinds::*,
		/// 	types::optics::*,
		/// };
		/// #[derive(Clone)]
		/// struct ListTraversal;
		/// impl<'a, A: 'a + Clone> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
		/// 	fn apply<M: Applicative>(
		/// 		&self,
		/// 		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
		/// 		s: Vec<A>,
		/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
		/// 		s.into_iter().fold(M::pure(vec![]), |acc, a| {
		/// 			M::lift2(
		/// 				|mut v: Vec<A>, x: A| {
		/// 					v.push(x);
		/// 					v
		/// 				},
		/// 				acc,
		/// 				f(a),
		/// 			)
		/// 		})
		/// 	}
		/// }
		/// let t = Traversal::<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
		/// let p = positions(t).traversal;
		/// let s = vec![10, 20, 30];
		/// let f = Box::new(|i: usize, a: i32| -> Option<i32> { Some(a + i as i32) });
		/// let result: Option<Vec<i32>> = IndexedTraversalFunc::apply::<OptionBrand>(&p, f, s);
		/// assert_eq!(result, Some(vec![10, 21, 32]));
		/// ```
		fn apply<M: Applicative>(
			&self,
			f: Box<
				dyn Fn(usize, A) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>) + 'a,
			>,
			s: S,
		) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, T>)
		where
			Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>): Clone, {
			let counter = std::cell::Cell::new(0usize);
			self.0.apply::<M>(
				Box::new(move |a: A| {
					let i = counter.get();
					counter.set(i + 1);
					f(i, a)
				}),
				s,
			)
		}
	}

	/// Create an indexed traversal by decorating each focus of a traversal with its position.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The source structure type.",
		"The target structure type.",
		"The source focus type.",
		"The target focus type.",
		"The traversal function type."
	)]
	#[document_returns("An indexed traversal over the positions.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	Apply,
	/// 	brands::{
	/// 		RcBrand,
	/// 		optics::*,
	/// 	},
	/// 	classes::{
	/// 		Applicative,
	/// 		lift::Lift,
	/// 		optics::traversal::TraversalFunc,
	/// 	},
	/// 	functions::*,
	/// 	kinds::*,
	/// 	types::optics::*,
	/// };
	/// #[derive(Clone)]
	/// struct ListTraversal;
	/// impl<'a, A: 'a + Clone> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
	/// 	fn apply<M: Applicative>(
	/// 		&self,
	/// 		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
	/// 		s: Vec<A>,
	/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
	/// 		s.into_iter().fold(M::pure(vec![]), |acc, a| {
	/// 			M::lift2(
	/// 				|mut v: Vec<A>, x: A| {
	/// 					v.push(x);
	/// 					v
	/// 				},
	/// 				acc,
	/// 				f(a),
	/// 			)
	/// 		})
	/// 	}
	/// }
	/// let t = Traversal::<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
	/// let l = positions(t);
	/// let s = vec![10, 20, 30];
	/// let result = optics_indexed_over::<RcBrand, _, _, _, _>(&l, s, |i, x| x + i as i32);
	/// assert_eq!(result, vec![10, 21, 32]);
	/// ```
	pub fn positions<'a, PointerBrand, S, T, A, B, F>(
		traversal: Traversal<'a, PointerBrand, S, T, A, B, F>
	) -> IndexedTraversal<'a, PointerBrand, usize, S, T, A, B, PositionsTraversalFunc<F>>
	where
		PointerBrand: UnsizedCoercible,
		F: TraversalFunc<'a, S, T, A, B> + Clone + 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		IndexedTraversal::new(PositionsTraversalFunc(traversal.traversal))
	}
}

pub use inner::*;
