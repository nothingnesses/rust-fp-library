//! Lens optics for product types.

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
				optics::*,
				profunctor::{
					Choice,
					Strong,
					Wander,
				},
				*,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// A polymorphic lens for accessing and updating a field where types can change.
	/// This matches PureScript's `Lens s t a b`.
	///
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub struct Lens<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: ToDynCloneFn,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Internal storage.
		pub(crate) to: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, (A, <FnBrand<PointerBrand> as CloneFn>::Of<'a, B, T>)>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S, T, A, B> Clone for Lens<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: ToDynCloneFn,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Lens` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		/// let cloned = l.clone();
		/// assert_eq!(cloned.view((42, "hi".to_string())), 42);
		/// ```
		fn clone(&self) -> Self {
			Lens {
				to: self.to.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> Lens<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: ToDynCloneFn,
	{
		/// Create a new polymorphic lens.
		/// This matches PureScript's `lens'` constructor.
		#[document_signature]
		///
		#[document_parameters("The getter/setter pair function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		FnBrand,
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::*,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, String, i32, String> =
		/// 	Lens::new(|x| (x, <FnBrand<RcBrand> as LiftFn>::new(|s| s)));
		/// assert_eq!(l.view(42), 42);
		/// ```
		pub fn new(
			to: impl 'a + Fn(S) -> (A, <FnBrand<PointerBrand> as CloneFn>::Of<'a, B, T>)
		) -> Self {
			Lens {
				to: <FnBrand<PointerBrand> as LiftFn>::new(to),
			}
		}

		/// Create a new polymorphic lens from a getter and setter.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		#[document_returns("A new `Lens` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, String, i32, String> = Lens::from_view_set(|x| x, |(_, s)| s);
		/// assert_eq!(l.view(42), 42);
		/// ```
		pub fn from_view_set(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, B)) -> T,
		) -> Self
		where
			S: Clone, {
			let view_brand = <FnBrand<PointerBrand> as LiftFn>::new(view);
			let set_brand = <FnBrand<PointerBrand> as LiftFn>::new(set);

			Lens {
				to: <FnBrand<PointerBrand> as LiftFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					(
						view_brand(s),
						<FnBrand<PointerBrand> as LiftFn>::new(move |b| {
							set_brand((s_clone.clone(), b))
						}),
					)
				}),
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		#[document_returns("The focus value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, i32, i32, i32> = Lens::from_view_set(|x| x, |(_, y)| y);
		/// assert_eq!(l.view(10), 10);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.to)(s).0
		}

		/// Set the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new value for the focus.")]
		///
		#[document_returns("The updated structure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, i32, i32, i32> = Lens::from_view_set(|x| x, |(_, y)| y);
		/// assert_eq!(l.set(10, 20), 20);
		/// ```
		pub fn set(
			&self,
			s: S,
			b: B,
		) -> T {
			((self.to)(s).1)(b)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, Q, PointerBrand, S, T, A, B> Optic<'a, Q, S, T, A, B>
		for Lens<'a, PointerBrand, S, T, A, B>
	where
		Q: Strong,
		PointerBrand: ToDynCloneFn,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	Optic::<RcFnBrand, _, _, _, _>::evaluate(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let to = self.to.clone();

			Q::dimap(
				move |s: S| to(s),
				move |(b, f): (B, <FnBrand<PointerBrand> as CloneFn>::Of<'a, B, T>)| f(b),
				Q::first(pab),
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> LensOptic<'a, S, T, A, B>
		for Lens<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	LensOptic::evaluate::<RcFnBrand>(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Strong>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> AffineTraversalOptic<'a, S, T, A, B>
		for Lens<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	AffineTraversalOptic::evaluate::<RcFnBrand>(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Strong + Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			LensOptic::evaluate::<Q>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> TraversalOptic<'a, S, T, A, B>
		for Lens<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	TraversalOptic::evaluate::<RcFnBrand>(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			LensOptic::evaluate::<Q>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> GetterOptic<'a, S, A> for Lens<'a, PointerBrand, S, S, A, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = GetterOptic::evaluate(&l, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate<R: 'a + 'static, Q: ToDynCloneFn + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			LensOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> FoldOptic<'a, S, A> for Lens<'a, PointerBrand, S, S, A, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = FoldOptic::evaluate(&l, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), "42".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: ToDynCloneFn + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			LensOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the lens.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, Q, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> SetterOptic<'a, Q, S, T, A, B>
		for Lens<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: ToDynCloneFn,
		Q: ToDynCloneFn,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			LensOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}

	/// A concrete lens type for accessing and updating a field in a structure where types do not change.
	/// This matches PureScript's `Lens' s a`.
	///
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
		S: 'a,
		A: 'a, {
		pub(crate) to: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, (A, <FnBrand<PointerBrand> as CloneFn>::Of<'a, A, S>)>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, PointerBrand, S, A> Clone for LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `LensPrime` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, y)| y);
		/// let cloned = l.clone();
		/// assert_eq!(cloned.view(42), 42);
		/// ```
		fn clone(&self) -> Self {
			LensPrime {
				to: self.to.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		/// Create a new monomorphic lens.
		/// This matches PureScript's `lens'` constructor.
		#[document_signature]
		///
		#[document_parameters("The getter/setter pair function.")]
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
		/// 	classes::*,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> =
		/// 	LensPrime::new(|x| (x, <RcFnBrand as LiftFn>::new(|s| s)));
		/// assert_eq!(l.view(42), 42);
		/// ```
		pub fn new(
			to: impl 'a + Fn(S) -> (A, <FnBrand<PointerBrand> as CloneFn>::Of<'a, A, S>)
		) -> Self {
			LensPrime {
				to: <FnBrand<PointerBrand> as LiftFn>::new(to),
			}
		}

		/// Create a new monomorphic lens from a getter and setter.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		#[document_returns("A new `LensPrime` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.view(10), 10);
		/// ```
		pub fn from_view_set(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, A)) -> S,
		) -> Self
		where
			S: Clone, {
			let view_brand = <FnBrand<PointerBrand> as LiftFn>::new(view);
			let set_brand = <FnBrand<PointerBrand> as LiftFn>::new(set);

			LensPrime {
				to: <FnBrand<PointerBrand> as LiftFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					(
						view_brand(s),
						<FnBrand<PointerBrand> as LiftFn>::new(move |a| {
							set_brand((s_clone.clone(), a))
						}),
					)
				}),
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		#[document_returns("The focus value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.view(42), 42);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.to)(s).0
		}

		/// Set the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new value for the focus.")]
		///
		#[document_returns("The updated structure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.set(10, 20), 20);
		/// ```
		pub fn set(
			&self,
			s: S,
			a: A,
		) -> S {
			((self.to)(s).1)(a)
		}

		/// Update the focus of the lens in a structure using a function.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		///
		#[document_returns("The updated structure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.over(10, |x| x + 1), 11);
		/// ```
		pub fn over(
			&self,
			s: S,
			f: impl Fn(A) -> A,
		) -> S {
			let (a, set) = (self.to)(s);
			set(f(a))
		}
	}

	// Optic implementation for LensPrime<PointerBrand, S, A>
	// Note: This implements monomorphic update (S -> S, A -> A)
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, Q, PointerBrand, S, A> Optic<'a, Q, S, S, A, A> for LensPrime<'a, PointerBrand, S, A>
	where
		Q: Strong,
		PointerBrand: ToDynCloneFn,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	Optic::<RcFnBrand, _, _, _, _>::evaluate(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let to = self.to.clone();

			// The Profunctor encoding of a Lens is:
			// lens get set = dimap (\s -> (get s, s)) (\(b, s) -> set s b) . first
			Q::dimap(
				move |s: S| to(s),
				move |(a, f): (A, <FnBrand<PointerBrand> as CloneFn>::Of<'a, A, S>)| f(a),
				Q::first(pab),
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> AffineTraversalOptic<'a, S, S, A, A>
		for LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	AffineTraversalOptic::evaluate::<RcFnBrand>(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Strong + Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			LensOptic::evaluate::<Q>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> TraversalOptic<'a, S, S, A, A>
		for LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	TraversalOptic::evaluate::<RcFnBrand>(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			LensOptic::evaluate::<Q>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> GetterOptic<'a, S, A> for LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = GetterOptic::evaluate(&l, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
		/// ```
		fn evaluate<R: 'a + 'static, Q: ToDynCloneFn + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			LensOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> FoldOptic<'a, S, A> for LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
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
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = FoldOptic::evaluate(&l, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), "42".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: ToDynCloneFn + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			LensOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the lens.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, Q, PointerBrand, S: 'a, A: 'a> SetterOptic<'a, Q, S, S, A, A>
		for LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
		Q: ToDynCloneFn,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			LensOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> LensOptic<'a, S, S, A, A> for LensPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: ToDynCloneFn,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			optics::*,
		/// 			*,
		/// 		},
		/// 		classes::optics::*,
		/// 		functions::*,
		/// 		types::optics::*,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::from_view_set(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier: Rc<dyn Fn((i32, String)) -> (i32, String)> =
		/// 	LensOptic::evaluate::<RcFnBrand>(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate<Q: Strong>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}
}
pub use inner::*;
