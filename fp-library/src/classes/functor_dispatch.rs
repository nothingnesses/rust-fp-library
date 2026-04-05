//! Unified dispatch for mapping operations.
//!
//! Provides a single [`map`] free function that dispatches to either
//! [`Functor::map`](crate::classes::Functor::map) (when the closure takes owned values)
//! or [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map) (when the closure
//! takes references), using marker-type dispatch resolved by the compiler.
//!
//! ## How it works
//!
//! The dispatch uses two zero-sized marker types ([`Val`] and [`Ref`]) and a
//! [`FunctorDispatch`] trait with separate blanket implementations for each marker.
//! The compiler selects the correct implementation based on the closure's
//! argument type:
//!
//! - A closure `Fn(A) -> B` satisfies `FunctorDispatch<..., Val>`, which calls
//!   [`Functor::map`](crate::classes::Functor::map).
//! - A closure `Fn(&A) -> B` satisfies `FunctorDispatch<..., Ref>`, which calls
//!   [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map).
//!
//! The `Marker` type parameter is inferred automatically. Callers write
//! `map::<Brand, _, _, _>(...)` and never need to specify the marker
//! explicitly. The dispatch is resolved at compile time with no runtime cost.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! // Closure takes owned i32 -> dispatches to Functor::map
//! let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
//! assert_eq!(y, Some(10));
//!
//! // Closure takes &i32 -> dispatches to RefFunctor::ref_map
//! let lazy = RcLazy::pure(10);
//! let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				Functor,
				Lift,
				RefFunctor,
				RefLift,
				RefSemimonad,
				Semimonad,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- Marker types --

	/// Marker type indicating the closure receives owned values.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `A` (not `&A`). Routes to [`Functor::map`](crate::classes::Functor::map).
	pub struct Val;

	/// Marker type indicating the closure receives references.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `&A`. Routes to [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map).
	pub struct Ref;

	// -- Closure mode --

	/// Trait that maps a closure mode marker ([`Val`] or [`Ref`]) to the
	/// corresponding `dyn Fn` trait object type.
	///
	/// Used by [`CloneFn`](crate::classes::CloneFn) to parameterize
	/// the `Deref` target of wrapped closures. `Val` produces
	/// `dyn Fn(A) -> B` (by-value), `Ref` produces `dyn Fn(&A) -> B`
	/// (by-reference).
	pub trait ClosureMode {
		/// The unsized closure trait object type for this mode.
		type Target<'a, A: 'a, B: 'a>: ?Sized + 'a;

		/// The unsized closure trait object type for this mode with `Send + Sync` bounds.
		type SendTarget<'a, A: 'a, B: 'a>: ?Sized + 'a;
	}

	impl ClosureMode for Val {
		type SendTarget<'a, A: 'a, B: 'a> = dyn 'a + Fn(A) -> B + Send + Sync;
		type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(A) -> B;
	}

	impl ClosureMode for Ref {
		type SendTarget<'a, A: 'a, B: 'a> = dyn 'a + Fn(&A) -> B + Send + Sync;
		type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(&A) -> B;
	}

	// -- Dispatch trait --

	/// Trait that routes a map operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically. Either [`Val`] or [`Ref`]."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FunctorDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Perform the dispatched map operation.
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: Fn(A) -> B -> Functor::map --

	/// Routes `Fn(A) -> B` closures to [`Functor::map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, F> FunctorDispatch<'a, Brand, A, B, Val> for F
	where
		Brand: Functor,
		A: 'a,
		B: 'a,
		F: Fn(A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::map(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> B -> RefFunctor::ref_map --

	/// Routes `Fn(&A) -> B` closures to [`RefFunctor::ref_map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, F> FunctorDispatch<'a, Brand, A, B, Ref> for F
	where
		Brand: RefFunctor,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::pure(10);
		/// let result = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
		/// assert_eq!(*result.evaluate(), 20);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_map(self, fa)
		}
	}

	// -- Unified free function --

	/// Maps a function over the values in a functor context.
	///
	/// Dispatches to either [`Functor::map`] or [`RefFunctor::ref_map`]
	/// based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(A) -> B`), dispatches to
	///   [`Functor::map`].
	/// - If the closure takes references (`Fn(&A) -> B`), dispatches to
	///   [`RefFunctor::ref_map`].
	///
	/// The `Marker` type parameter is inferred automatically by the compiler
	/// from the closure's argument type. Callers write `map::<Brand, _, _, _>(...)`
	/// and never need to specify `Marker` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Owned: dispatches to Functor::map
	/// let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
	/// assert_eq!(y, Some(10));
	///
	/// // By-ref: dispatches to RefFunctor::ref_map
	/// let lazy = RcLazy::pure(10);
	/// let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
	/// assert_eq!(*mapped.evaluate(), 20);
	/// ```
	pub fn map<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		f: impl FunctorDispatch<'a, Brand, A, B, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch(fa)
	}

	// -- BindDispatch --

	/// Trait that routes a bind operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is inferred from the closure's argument type:
	/// `Fn(A) -> Of<B>` resolves to [`Val`], `Fn(&A) -> Of<B>` resolves to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value inside the monad.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait BindDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Perform the dispatched bind operation.
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch_bind(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Routes `Fn(A) -> Of<B>` closures to [`Semimonad::bind`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The input type.",
		"The output type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, F> BindDispatch<'a, Brand, A, B, Val> for F
	where
		Brand: Semimonad,
		A: 'a,
		B: 'a,
		F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch_bind(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::bind(ma, self)
		}
	}

	/// Routes `Fn(&A) -> Of<B>` closures to [`RefSemimonad::ref_bind`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The input type.",
		"The output type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, F> BindDispatch<'a, Brand, A, B, Ref> for F
	where
		Brand: RefSemimonad,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::pure(5);
		/// let result = bind::<LazyBrand<RcLazyConfig>, _, _, _>(lazy, |x: &i32| {
		/// 	Lazy::<_, RcLazyConfig>::new({
		/// 		let v = *x;
		/// 		move || v * 2
		/// 	})
		/// });
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn dispatch_bind(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_bind(ma, self)
		}
	}

	/// Sequences a monadic computation with a function that produces the next computation.
	///
	/// Dispatches to either [`Semimonad::bind`] or [`RefSemimonad::ref_bind`]
	/// based on the closure's argument type.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value inside the monad.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The monadic value.", "The function to apply to the value.")]
	///
	#[document_returns("The result of sequencing the computation.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn bind<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl BindDispatch<'a, Brand, A, B, Marker>,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch_bind(ma)
	}

	// -- Lift2Dispatch --

	/// Trait that routes a lift2 operation to the appropriate type class method.
	///
	/// `Fn(A, B) -> C` resolves to [`Val`], `Fn(&A, &B) -> C` resolves to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift2Dispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, Marker> {
		/// Perform the dispatched lift2 operation.
		#[document_signature]
		#[document_parameters("The first context.", "The second context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
		/// assert_eq!(z, Some(3));
		/// ```
		fn dispatch_lift2(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>);
	}

	/// Routes `Fn(A, B) -> C` closures to [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The first type.",
		"The second type.",
		"The result type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, F> Lift2Dispatch<'a, Brand, A, B, C, Val> for F
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
		F: Fn(A, B) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters("The first context.", "The second context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
		/// assert_eq!(z, Some(3));
		/// ```
		fn dispatch_lift2(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::lift2(self, fa, fb)
		}
	}

	/// Routes `Fn(&A, &B) -> C` closures to [`RefLift::ref_lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The first type.",
		"The second type.",
		"The result type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, C, F> Lift2Dispatch<'a, Brand, A, B, C, Ref> for F
	where
		Brand: RefLift,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(&A, &B) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters("The first context.", "The second context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let x = RcLazy::pure(3);
		/// let y = RcLazy::pure(4);
		/// let z = lift2::<LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32, b: &i32| *a + *b, x, y);
		/// assert_eq!(*z.evaluate(), 7);
		/// ```
		fn dispatch_lift2(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::ref_lift2(self, fa, fb)
		}
	}

	/// Lifts a binary function into a functor context.
	///
	/// Dispatches to either [`Lift::lift2`] or [`RefLift::ref_lift2`]
	/// based on the closure's argument types.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The function to lift.", "The first context.", "The second context.")]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
	/// assert_eq!(z, Some(3));
	/// ```
	pub fn lift2<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, Marker>(
		f: impl Lift2Dispatch<'a, Brand, A, B, C, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		f.dispatch_lift2(fa, fb)
	}

	// -- Lift3Dispatch --

	/// Trait that routes a lift3 operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First type.",
		"Second type.",
		"Third type.",
		"Result type.",
		"Dispatch marker."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift3Dispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, D: 'a, Marker> {
		/// Perform the dispatched lift3 operation.
		#[document_signature]
		#[document_parameters("First context.", "Second context.", "Third context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift3::<OptionBrand, _, _, _, _, _>(|a, b, c| a + b + c, Some(1), Some(2), Some(3));
		/// assert_eq!(r, Some(6));
		/// ```
		fn dispatch_lift3(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>);
	}

	/// Routes `Fn(A, B, C) -> D` closures through [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, D, F> Lift3Dispatch<'a, Brand, A, B, C, D, Val> for F
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: 'a,
		F: Fn(A, B, C) -> D + 'a,
	{
		#[document_signature]
		#[document_parameters("First context.", "Second context.", "Third context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift3::<OptionBrand, _, _, _, _, _>(|a, b, c| a + b + c, Some(1), Some(2), Some(3));
		/// assert_eq!(r, Some(6));
		/// ```
		fn dispatch_lift3(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) {
			Brand::lift2(move |(a, b), c| self(a, b, c), Brand::lift2(|a, b| (a, b), fa, fb), fc)
		}
	}

	/// Routes `Fn(&A, &B, &C) -> D` closures through [`RefLift::ref_lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, C, D, F> Lift3Dispatch<'a, Brand, A, B, C, D, Ref> for F
	where
		Brand: RefLift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
		D: 'a,
		F: Fn(&A, &B, &C) -> D + 'a,
	{
		#[document_signature]
		#[document_parameters("First context.", "Second context.", "Third context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let a = RcLazy::pure(1);
		/// let b = RcLazy::pure(2);
		/// let c = RcLazy::pure(3);
		/// let r = lift3::<LazyBrand<RcLazyConfig>, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32, c: &i32| *a + *b + *c,
		/// 	a,
		/// 	b,
		/// 	c,
		/// );
		/// assert_eq!(*r.evaluate(), 6);
		/// ```
		fn dispatch_lift3(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) {
			Brand::ref_lift2(
				move |(a, b): &(A, B), c: &C| self(a, b, c),
				Brand::ref_lift2(|a: &A, b: &B| (a.clone(), b.clone()), fa, fb),
				fc,
			)
		}
	}

	/// Lifts a ternary function into a functor context.
	///
	/// Dispatches to [`Lift::lift2`] or [`RefLift::ref_lift2`] based on the closure's argument types.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First type.",
		"Second type.",
		"Third type.",
		"Result type.",
		"Dispatch marker."
	)]
	#[document_parameters(
		"The function to lift.",
		"First context.",
		"Second context.",
		"Third context."
	)]
	#[document_returns("A new context containing the result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let r = lift3::<OptionBrand, _, _, _, _, _>(|a, b, c| a + b + c, Some(1), Some(2), Some(3));
	/// assert_eq!(r, Some(6));
	/// ```
	pub fn lift3<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, D: 'a, Marker>(
		f: impl Lift3Dispatch<'a, Brand, A, B, C, D, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) {
		f.dispatch_lift3(fa, fb, fc)
	}

	// -- Lift4Dispatch --

	/// Trait that routes a lift4 operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"Dispatch marker."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift4Dispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		E: 'a,
		Marker,
	> {
		/// Perform the dispatched lift4 operation.
		#[document_signature]
		#[document_parameters("First.", "Second.", "Third.", "Fourth.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift4::<OptionBrand, _, _, _, _, _, _>(
		/// 	|a, b, c, d| a + b + c + d,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// );
		/// assert_eq!(r, Some(10));
		/// ```
		fn dispatch_lift4(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>);
	}

	/// Routes `Fn(A, B, C, D) -> E` closures through [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, D, E, Func> Lift4Dispatch<'a, Brand, A, B, C, D, E, Val> for Func
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: 'a,
		Func: Fn(A, B, C, D) -> E + 'a,
	{
		#[document_signature]
		#[document_parameters("First.", "Second.", "Third.", "Fourth.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift4::<OptionBrand, _, _, _, _, _, _>(
		/// 	|a, b, c, d| a + b + c + d,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// );
		/// assert_eq!(r, Some(10));
		/// ```
		fn dispatch_lift4(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
			Brand::lift2(
				move |((a, b), c), d| self(a, b, c, d),
				Brand::lift2(move |(a, b), c| ((a, b), c), Brand::lift2(|a, b| (a, b), fa, fb), fc),
				fd,
			)
		}
	}

	/// Routes `Fn(&A, &B, &C, &D) -> E` closures through [`RefLift::ref_lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, C, D, E, Func> Lift4Dispatch<'a, Brand, A, B, C, D, E, Ref> for Func
	where
		Brand: RefLift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: 'a,
		E: 'a,
		Func: Fn(&A, &B, &C, &D) -> E + 'a,
	{
		#[document_signature]
		#[document_parameters("First.", "Second.", "Third.", "Fourth.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let r = lift4::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32, c: &i32, d: &i32| *a + *b + *c + *d,
		/// 	RcLazy::pure(1),
		/// 	RcLazy::pure(2),
		/// 	RcLazy::pure(3),
		/// 	RcLazy::pure(4),
		/// );
		/// assert_eq!(*r.evaluate(), 10);
		/// ```
		fn dispatch_lift4(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
			Brand::ref_lift2(
				move |((a, b), c): &((A, B), C), d: &D| self(a, b, c, d),
				Brand::ref_lift2(
					move |(a, b): &(A, B), c: &C| ((a.clone(), b.clone()), c.clone()),
					Brand::ref_lift2(|a: &A, b: &B| (a.clone(), b.clone()), fa, fb),
					fc,
				),
				fd,
			)
		}
	}

	/// Lifts a quaternary function into a functor context.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"Dispatch marker."
	)]
	#[document_parameters("The function to lift.", "First.", "Second.", "Third.", "Fourth.")]
	#[document_returns("Result context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let r = lift4::<OptionBrand, _, _, _, _, _, _>(
	/// 	|a, b, c, d| a + b + c + d,
	/// 	Some(1),
	/// 	Some(2),
	/// 	Some(3),
	/// 	Some(4),
	/// );
	/// assert_eq!(r, Some(10));
	/// ```
	pub fn lift4<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, D: 'a, E: 'a, Marker>(
		f: impl Lift4Dispatch<'a, Brand, A, B, C, D, E, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
		f.dispatch_lift4(fa, fb, fc, fd)
	}

	// -- Lift5Dispatch --

	/// Trait that routes a lift5 operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"Dispatch marker."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift5Dispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		E: 'a,
		G: 'a,
		Marker,
	> {
		/// Perform the dispatched lift5 operation.
		#[document_signature]
		#[document_parameters("1st.", "2nd.", "3rd.", "4th.", "5th.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift5::<OptionBrand, _, _, _, _, _, _, _>(
		/// 	|a, b, c, d, e| a + b + c + d + e,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// 	Some(5),
		/// );
		/// assert_eq!(r, Some(15));
		/// ```
		fn dispatch_lift5(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			fe: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>);
	}

	/// Routes `Fn(A, B, C, D, E) -> G` closures through [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, D, E, G, Func> Lift5Dispatch<'a, Brand, A, B, C, D, E, G, Val> for Func
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: Clone + 'a,
		G: 'a,
		Func: Fn(A, B, C, D, E) -> G + 'a,
	{
		#[document_signature]
		#[document_parameters("1st.", "2nd.", "3rd.", "4th.", "5th.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift5::<OptionBrand, _, _, _, _, _, _, _>(
		/// 	|a, b, c, d, e| a + b + c + d + e,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// 	Some(5),
		/// );
		/// assert_eq!(r, Some(15));
		/// ```
		fn dispatch_lift5(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			fe: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>) {
			Brand::lift2(
				move |(((a, b), c), d), e| self(a, b, c, d, e),
				Brand::lift2(
					move |((a, b), c), d| (((a, b), c), d),
					Brand::lift2(
						move |(a, b), c| ((a, b), c),
						Brand::lift2(|a, b| (a, b), fa, fb),
						fc,
					),
					fd,
				),
				fe,
			)
		}
	}

	/// Routes `Fn(&A, &B, &C, &D, &E) -> G` closures through [`RefLift::ref_lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, C, D, E, G, Func> Lift5Dispatch<'a, Brand, A, B, C, D, E, G, Ref> for Func
	where
		Brand: RefLift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: 'a,
		G: 'a,
		Func: Fn(&A, &B, &C, &D, &E) -> G + 'a,
	{
		#[document_signature]
		#[document_parameters("1st.", "2nd.", "3rd.", "4th.", "5th.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let r = lift5::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32, c: &i32, d: &i32, e: &i32| *a + *b + *c + *d + *e,
		/// 	RcLazy::pure(1),
		/// 	RcLazy::pure(2),
		/// 	RcLazy::pure(3),
		/// 	RcLazy::pure(4),
		/// 	RcLazy::pure(5),
		/// );
		/// assert_eq!(*r.evaluate(), 15);
		/// ```
		fn dispatch_lift5(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			fe: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>) {
			Brand::ref_lift2(
				move |(((a, b), c), d): &(((A, B), C), D), e: &E| self(a, b, c, d, e),
				Brand::ref_lift2(
					move |((a, b), c): &((A, B), C), d: &D| {
						(((a.clone(), b.clone()), c.clone()), d.clone())
					},
					Brand::ref_lift2(
						move |(a, b): &(A, B), c: &C| ((a.clone(), b.clone()), c.clone()),
						Brand::ref_lift2(|a: &A, b: &B| (a.clone(), b.clone()), fa, fb),
						fc,
					),
					fd,
				),
				fe,
			)
		}
	}

	/// Lifts a quinary function into a functor context.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"Dispatch marker."
	)]
	#[document_parameters("The function to lift.", "1st.", "2nd.", "3rd.", "4th.", "5th.")]
	#[document_returns("Result context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let r = lift5::<OptionBrand, _, _, _, _, _, _, _>(
	/// 	|a, b, c, d, e| a + b + c + d + e,
	/// 	Some(1),
	/// 	Some(2),
	/// 	Some(3),
	/// 	Some(4),
	/// 	Some(5),
	/// );
	/// assert_eq!(r, Some(15));
	/// ```
	pub fn lift5<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		E: 'a,
		G: 'a,
		Marker,
	>(
		f: impl Lift5Dispatch<'a, Brand, A, B, C, D, E, G, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
		fe: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>) {
		f.dispatch_lift5(fa, fb, fc, fd, fe)
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::map,
		crate::{
			brands::*,
			types::*,
		},
	};

	#[test]
	fn test_val_option() {
		let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_vec() {
		let result = map::<VecBrand, _, _, _>(|x: i32| x + 1, vec![1, 2, 3]);
		assert_eq!(result, vec![2, 3, 4]);
	}

	#[test]
	fn test_ref_lazy() {
		let lazy = RcLazy::pure(10);
		let result = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
		assert_eq!(*result.evaluate(), 20);
	}

	#[test]
	fn test_val_none() {
		let result = map::<OptionBrand, i32, i32, _>(|x| x * 2, None);
		assert_eq!(result, None);
	}
}

// -- Brand inference POC --
//
// Validates that a DefaultBrand trait can enable turbofish-free map calls
// by inferring the Brand from the container's concrete type. This is a
// temporary module; the trait and function will move to their own files
// if the POC succeeds.

#[cfg(test)]
mod brand_inference_poc {
	use crate::{
		brands::*,
		classes::functor_dispatch::inner::FunctorDispatch,
		kinds::Kind_cdc7cd43dac7585f,
		types::*,
	};

	/// Reverse mapping from a concrete type to its canonical brand.
	trait DefaultBrand {
		type Brand: Kind_cdc7cd43dac7585f;
	}

	impl<A> DefaultBrand for Option<A> {
		type Brand = OptionBrand;
	}

	impl<A> DefaultBrand for Vec<A> {
		type Brand = VecBrand;
	}

	impl<'a, A: 'a, Config: crate::classes::LazyConfig + 'a> DefaultBrand for Lazy<'a, A, Config> {
		type Brand = LazyBrand<Config>;
	}

	/// Temporary inference-based map function for POC validation.
	fn map_infer<'a, FA, A: 'a, B: 'a, Marker>(
		f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>,
		fa: FA,
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
	where
		FA: DefaultBrand + 'a,
		<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>, {
		f.dispatch(fa)
	}

	// -- Val dispatch (Functor::map) --

	#[test]
	fn infer_option_val() {
		let result = map_infer(|x: i32| x * 2, Some(5));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn infer_option_none() {
		let result = map_infer(|x: i32| x * 2, None::<i32>);
		assert_eq!(result, None);
	}

	#[test]
	fn infer_vec_val() {
		let result = map_infer(|x: i32| x + 1, vec![1, 2, 3]);
		assert_eq!(result, vec![2, 3, 4]);
	}

	#[test]
	fn infer_vec_strings() {
		let result = map_infer(|x: i32| x.to_string(), vec![1, 2]);
		assert_eq!(result, vec!["1", "2"]);
	}

	// -- Ref dispatch (RefFunctor::ref_map) --

	#[test]
	fn infer_lazy_ref() {
		let lazy = RcLazy::pure(10);
		let result = map_infer(|x: &i32| *x * 2, lazy);
		assert_eq!(*result.evaluate(), 20);
	}

	// Note: ArcLazy implements SendRefFunctor, not RefFunctor, so it
	// cannot be dispatched via FunctorDispatch's Ref path. This will be
	// resolved when the ref-hierarchy plan adds SendRefFunctor to the
	// dispatch system.
}
