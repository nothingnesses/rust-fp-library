//! Lens optics for product types.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{CloneableFn, Strong, UnsizedCoercible, monoid::Monoid, wander::Wander},
			kinds::*,
			types::optics::{FoldOptic, GetterOptic, LensOptic, Optic, SetterOptic, TraversalOptic, ForgetBrand},
		},
		fp_macros::{document_parameters, document_signature, document_type_parameters},
		std::marker::PhantomData,
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
		B: 'a,
	{
		/// Getter function.
		pub view: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		/// Setter function.
		pub set: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, B), T>),
		pub(crate) _phantom: PhantomData<P>,
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
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, String, i32, String> = Lens::new(|x| x, |(_, s)| s);
		/// ```
		pub fn new(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, B)) -> T,
		) -> Self {
			Lens {
				view: <FnBrand<P> as CloneableFn>::new(view),
				set: <FnBrand<P> as CloneableFn>::new(set),
				_phantom: PhantomData,
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, i32, i32, i32> = Lens::new(|x| x, |(_, y)| y);
		/// assert_eq!(l.view(10), 10);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.view)(s)
		}

		/// Set the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new value for the focus.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, i32, i32, i32> = Lens::new(|x| x, |(_, y)| y);
		/// assert_eq!(l.set(10, 20), 20);
		/// ```
		pub fn set(
			&self,
			s: S,
			b: B,
		) -> T {
			(self.set)((s, b))
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
		S: 'a + Clone,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let view = self.view.clone();
			let set = self.set.clone();

			Q::dimap(
				move |s: S| (view(s.clone()), s),
				move |(b, s): (B, S)| set((s, b)),
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
	impl<'a, P, S: 'a + Clone, T: 'a, A: 'a, B: 'a> LensOptic<'a, S, T, A, B>
		for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = LensOptic::evaluate(&l, f);
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
	impl<'a, P, S: 'a + Clone, T: 'a, A: 'a, B: 'a> TraversalOptic<'a, S, T, A, B>
		for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = TraversalOptic::evaluate(&l, f);
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
	impl<'a, P, S: 'a + Clone, A: 'a> GetterOptic<'a, S, A> for Lens<'a, P, S, S, A, A>
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
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::new(|(x, _)| x, |((_, s), x)| (x, s));
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
	impl<'a, P, S: 'a + Clone, A: 'a> FoldOptic<'a, S, A> for Lens<'a, P, S, S, A, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = FoldOptic::evaluate(&l, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
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
	impl<'a, Q, P, S: 'a + Clone, T: 'a, A: 'a, B: 'a> SetterOptic<'a, Q, S, T, A, B>
		for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: Lens<RcBrand, (i32, String), (i32, String), i32, i32> =
		/// 	Lens::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = SetterOptic::evaluate(&l, f);
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
		A: 'a,
	{
		pub(crate) view_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		pub(crate) set_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, A), S>),
		pub(crate) _phantom: PhantomData<P>,
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
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// let cloned = l.clone();
		/// ```
		fn clone(&self) -> Self {
			LensPrime {
				view_fn: self.view_fn.clone(),
				set_fn: self.set_fn.clone(),
				_phantom: PhantomData,
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
		/// Create a new monomorphic lens from a getter and setter.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// ```
		pub fn new(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, A)) -> S,
		) -> Self {
			LensPrime {
				view_fn: <FnBrand<P> as CloneableFn>::new(view),
				set_fn: <FnBrand<P> as CloneableFn>::new(set),
				_phantom: PhantomData,
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.view(42), 42);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.view_fn)(s)
		}

		/// Set the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new value for the focus.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.set(10, 20), 20);
		/// ```
		pub fn set(
			&self,
			s: S,
			a: A,
		) -> S {
			(self.set_fn)((s, a))
		}

		/// Update the focus of the lens in a structure using a function.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.over(10, |x| x + 1), 11);
		/// ```
		pub fn over(
			&self,
			s: S,
			f: impl Fn(A) -> A,
		) -> S
		where
			S: Clone,
		{
			let a = self.view(s.clone());
			self.set(s, f(a))
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
		S: 'a + Clone,
		A: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&l, f);
		/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let view_fn = self.view_fn.clone();
			let set_fn = self.set_fn.clone();

			// The Profunctor encoding of a Lens is:
			// lens get set = dimap (\s -> (get s, s)) (\(b, s) -> set s b) . first
			Q::dimap(
				move |s: S| (view_fn(s.clone()), s),
				move |(a, s): (A, S)| set_fn((s, a)),
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
	impl<'a, P, S: 'a + Clone, A: 'a> TraversalOptic<'a, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = TraversalOptic::evaluate(&l, f);
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
	impl<'a, P, S: 'a + Clone, A: 'a> GetterOptic<'a, S, A> for LensPrime<'a, P, S, A>
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
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
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
	impl<'a, P, S: 'a + Clone, A: 'a> FoldOptic<'a, S, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = FoldOptic::evaluate(&l, f);
		/// assert_eq!(folded.run((42, "hi".to_string())), 42);
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
	impl<'a, Q, P, S: 'a + Clone, A: 'a> SetterOptic<'a, Q, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = SetterOptic::evaluate(&l, f);
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
	impl<'a, P, S: 'a + Clone, A: 'a> LensOptic<'a, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let l: LensPrime<RcBrand, (i32, String), i32> =
		/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let modifier = LensOptic::evaluate(&l, f);
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
