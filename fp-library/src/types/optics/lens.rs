//! Lens optics for product types.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				UnsizedCoercible,
				monoid::Monoid,
				optics::*,
				profunctor::{
					Choice,
					Strong,
					Wander,
				},
			},
			kinds::*,
			types::optics::ForgetBrand,
		},
		fp_macros::{
			document_parameters,
			document_return,
			document_type_parameters,
		},
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
	pub struct Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Internal storage.
		pub(crate) to: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, (A, <FnBrand<P> as CloneableFn>::Of<'a, B, T>)>),
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
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		/// Create a new polymorphic lens.
		/// This matches PureScript's `lens'` constructor.
		#[document_signature]
		///
		#[document_parameters("The getter/setter pair function.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		FnBrand,
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::CloneableFn,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, String, i32, String> =
		/// 	Lens::new(|x| (x, <FnBrand<RcBrand> as CloneableFn>::new(|s| s)));
		/// ```
		pub fn new(to: impl 'a + Fn(S) -> (A, <FnBrand<P> as CloneableFn>::Of<'a, B, T>)) -> Self {
			Lens {
				to: <FnBrand<P> as CloneableFn>::new(to),
			}
		}

		/// Create a new polymorphic lens from a getter and setter.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		#[document_return("A new `Lens` instance.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, String, i32, String> = Lens::from_view_set(|x| x, |(_, s)| s);
		/// ```
		pub fn from_view_set(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, B)) -> T,
		) -> Self
		where
			S: Clone, {
			let view_brand = <FnBrand<P> as CloneableFn>::new(view);
			let set_brand = <FnBrand<P> as CloneableFn>::new(set);

			Lens {
				to: <FnBrand<P> as CloneableFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					(
						view_brand(s),
						<FnBrand<P> as CloneableFn>::new(move |b| set_brand((s_clone.clone(), b))),
					)
				}),
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		#[document_return("The focus value.")]
		///
		/// ### Examples
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
		#[document_return("The updated structure.")]
		///
		/// ### Examples
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
	impl<'a, Q, P, S, T, A, B> Optic<'a, Q, S, T, A, B> for Lens<'a, P, S, T, A, B>
	where
		Q: Strong,
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
				move |(b, f): (B, <FnBrand<P> as CloneableFn>::Of<'a, B, T>)| f(b),
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
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> LensOptic<'a, S, T, A, B> for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> AffineTraversalOptic<'a, S, T, A, B>
		for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> TraversalOptic<'a, S, T, A, B> for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	impl<'a, P, S: 'a, A: 'a> GetterOptic<'a, S, A> for Lens<'a, P, S, S, A, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
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
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
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
	impl<'a, P, S: 'a, A: 'a> FoldOptic<'a, S, A> for Lens<'a, P, S, S, A, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
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
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
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
	impl<'a, Q, P, S: 'a, T: 'a, A: 'a, B: 'a> SetterOptic<'a, Q, S, T, A, B>
		for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	pub struct LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		pub(crate) to: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, (A, <FnBrand<P> as CloneableFn>::Of<'a, A, S>)>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, P, S, A> Clone for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		///
		#[document_return("A new `LensPrime` instance that is a copy of the original.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, y)| y);
		/// let cloned = l.clone();
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
	impl<'a, P, S: 'a, A: 'a> LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		/// Create a new monomorphic lens.
		/// This matches PureScript's `lens'` constructor.
		#[document_signature]
		///
		#[document_parameters("The getter/setter pair function.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RcFnBrand,
		/// 	},
		/// 	classes::CloneableFn,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> =
		/// 	LensPrime::new(|x| (x, <RcFnBrand as CloneableFn>::new(|s| s)));
		/// ```
		pub fn new(to: impl 'a + Fn(S) -> (A, <FnBrand<P> as CloneableFn>::Of<'a, A, S>)) -> Self {
			LensPrime {
				to: <FnBrand<P> as CloneableFn>::new(to),
			}
		}

		/// Create a new monomorphic lens from a getter and setter.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		#[document_return("A new `LensPrime` instance.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::from_view_set(|x: i32| x, |(_, y)| y);
		/// ```
		pub fn from_view_set(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, A)) -> S,
		) -> Self
		where
			S: Clone, {
			let view_brand = <FnBrand<P> as CloneableFn>::new(view);
			let set_brand = <FnBrand<P> as CloneableFn>::new(set);

			LensPrime {
				to: <FnBrand<P> as CloneableFn>::new(move |s: S| {
					let s_clone = s.clone();
					let set_brand = set_brand.clone();
					(
						view_brand(s),
						<FnBrand<P> as CloneableFn>::new(move |a| set_brand((s_clone.clone(), a))),
					)
				}),
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		#[document_return("The focus value.")]
		///
		/// ### Examples
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
		#[document_return("The updated structure.")]
		///
		/// ### Examples
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
		#[document_return("The updated structure.")]
		///
		/// ### Examples
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

	// Optic implementation for LensPrime<P, S, A>
	// Note: This implements monomorphic update (S -> S, A -> A)
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, Q, P, S, A> Optic<'a, Q, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		Q: Strong,
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
				move |(a, f): (A, <FnBrand<P> as CloneableFn>::Of<'a, A, S>)| f(a),
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
	impl<'a, P, S: 'a, A: 'a> AffineTraversalOptic<'a, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	impl<'a, P, S: 'a, A: 'a> TraversalOptic<'a, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	impl<'a, P, S: 'a, A: 'a> GetterOptic<'a, S, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
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
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
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
	impl<'a, P, S: 'a, A: 'a> FoldOptic<'a, S, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
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
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
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
	impl<'a, Q, P, S: 'a, A: 'a> SetterOptic<'a, Q, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
	impl<'a, P, S: 'a, A: 'a> LensOptic<'a, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_return("The transformed profunctor value.")]
		///
		/// ### Examples
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
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
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
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
