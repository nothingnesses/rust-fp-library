//! Dispatch for [`Semimonad::bind`](crate::classes::Semimonad::bind) and
//! [`RefSemimonad::ref_bind`](crate::classes::RefSemimonad::ref_bind).
//!
//! Provides the [`BindDispatch`] trait and a unified [`bind`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
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
//! // Owned: dispatches to Semimonad::bind
//! let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
//! assert_eq!(result, Some(10));
//!
//! // By-ref: dispatches to RefSemimonad::ref_bind
//! let lazy = RcLazy::pure(5);
//! let result = bind::<LazyBrand<RcLazyConfig>, _, _, _>(lazy, |x: &i32| {
//! 	Lazy::<_, RcLazyConfig>::new({
//! 		let v = *x;
//! 		move || v * 2
//! 	})
//! });
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				RefSemimonad,
				Semimonad,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a bind operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is inferred from the closure's argument type:
	/// `Fn(A) -> Of<B>` resolves to [`Val`](crate::classes::dispatch::Val),
	/// `Fn(&A) -> Of<B>` resolves to [`Ref`](crate::classes::dispatch::Ref).
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
	impl<'a, Brand, A, B, F> BindDispatch<'a, Brand, A, B, super::super::Val> for F
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
	impl<'a, Brand, A, B, F> BindDispatch<'a, Brand, A, B, super::super::Ref> for F
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

	// -- BindFlippedDispatch --

	/// Dispatch trait for `bind_flipped` (flipped argument order).
	///
	/// Routes `Fn(A) -> Of<B>` closures to [`Semimonad::bind`] and
	/// `Fn(&A) -> Of<B>` closures to [`RefSemimonad::ref_bind`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input element type.",
		"The output element type.",
		"Marker type (`Val` or `Ref`), inferred from the closure."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait BindFlippedDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Performs the dispatched bind_flipped operation.
		#[document_signature]
		#[document_parameters("The monadic value to bind over.")]
		#[document_returns("The result of binding the closure over the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = bind_flipped::<OptionBrand, _, _, _>(|x: i32| Some(x * 2), Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input element type.",
		"The output element type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, Brand, A, B, F> BindFlippedDispatch<'a, Brand, A, B, super::super::Val> for F
	where
		Brand: Semimonad,
		A: 'a,
		B: 'a,
		F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The bound result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = bind_flipped::<OptionBrand, _, _, _>(|x: i32| Some(x * 2), Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::bind(ma, self)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input element type.",
		"The output element type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, Brand, A, B, F> BindFlippedDispatch<'a, Brand, A, B, super::super::Ref> for F
	where
		Brand: RefSemimonad,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The bound result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 5);
		/// let result = bind_flipped::<LazyBrand<RcLazyConfig>, _, _, _>(
		/// 	|x: &i32| {
		/// 		let v = *x * 2;
		/// 		RcLazy::new(move || v)
		/// 	},
		/// 	lazy,
		/// );
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn dispatch(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_bind(ma, self)
		}
	}

	/// Binds a monadic value to a function (flipped argument order).
	///
	/// Dispatches to [`Semimonad::bind`] or [`RefSemimonad::ref_bind`]
	/// based on whether the closure takes `A` or `&A`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input element type.",
		"The output element type.",
		"Marker type, inferred from the closure."
	)]
	///
	#[document_parameters(
		"The function to apply to each element.",
		"The monadic value to bind over."
	)]
	///
	#[document_returns("The result of binding the function over the value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // By-value
	/// let result = bind_flipped::<OptionBrand, _, _, _>(|x: i32| Some(x * 2), Some(5));
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn bind_flipped<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		f: impl BindFlippedDispatch<'a, Brand, A, B, Marker>,
		ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch(ma)
	}

	// -- ComposeKleisliDispatch --

	/// Dispatch trait for Kleisli composition.
	///
	/// Routes `Fn(A) -> Of<B>` closures to [`Semimonad::bind`]-based composition
	/// and `Fn(&A) -> Of<B>` closures to [`RefSemimonad::ref_bind`]-based composition.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"Marker type (`Val` or `Ref`), inferred from the closures."
	)]
	#[document_parameters("The closure pair implementing this dispatch.")]
	pub trait ComposeKleisliDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, Marker>
	{
		/// Performs the dispatched Kleisli composition.
		#[document_signature]
		#[document_parameters("The input value.")]
		#[document_returns("The result of composing f then g applied to the input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result =
		/// 	compose_kleisli::<OptionBrand, _, _, _, _>((|x: i32| Some(x + 1), |y: i32| Some(y * 2)), 5);
		/// assert_eq!(result, Some(12));
		/// ```
		fn dispatch(
			self,
			a: A,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>);
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure pair.")]
	impl<'a, Brand, A, B, C, F, G> ComposeKleisliDispatch<'a, Brand, A, B, C, super::super::Val>
		for (F, G)
	where
		Brand: Semimonad,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		G: Fn(B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The input value.")]
		#[document_returns("The composed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result =
		/// 	compose_kleisli::<OptionBrand, _, _, _, _>((|x: i32| Some(x + 1), |y: i32| Some(y * 2)), 5);
		/// assert_eq!(result, Some(12));
		/// ```
		fn dispatch(
			self,
			a: A,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::bind(self.0(a), self.1)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure pair.")]
	impl<'a, Brand, A, B, C, F, G> ComposeKleisliDispatch<'a, Brand, A, B, C, super::super::Ref>
		for (F, G)
	where
		Brand: RefSemimonad,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		G: Fn(&B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The input value.")]
		#[document_returns("The composed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let result = compose_kleisli::<LazyBrand<RcLazyConfig>, _, _, _, _>(
		/// 	(
		/// 		|x: &i32| {
		/// 			let v = *x + 1;
		/// 			RcLazy::new(move || v)
		/// 		},
		/// 		|y: &i32| {
		/// 			let v = *y * 2;
		/// 			RcLazy::new(move || v)
		/// 		},
		/// 	),
		/// 	5,
		/// );
		/// assert_eq!(*result.evaluate(), 12);
		/// ```
		fn dispatch(
			self,
			a: A,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::ref_bind(self.0(&a), self.1)
		}
	}

	/// Composes two Kleisli arrows (f then g).
	///
	/// Dispatches to [`Semimonad::bind`] or [`RefSemimonad::ref_bind`]
	/// based on whether the closures take `A`/`B` or `&A`/`&B`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"Marker type, inferred from the closures."
	)]
	///
	#[document_parameters("A tuple of (first arrow, second arrow).", "The input value.")]
	///
	#[document_returns("The result of applying f then g.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result =
	/// 	compose_kleisli::<OptionBrand, _, _, _, _>((|x: i32| Some(x + 1), |y: i32| Some(y * 2)), 5);
	/// assert_eq!(result, Some(12));
	/// ```
	pub fn compose_kleisli<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, Marker>(
		fg: impl ComposeKleisliDispatch<'a, Brand, A, B, C, Marker>,
		a: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		fg.dispatch(a)
	}

	/// Composes two Kleisli arrows (g then f), flipped argument order.
	///
	/// Dispatches to [`Semimonad::bind`] or [`RefSemimonad::ref_bind`]
	/// based on whether the closures take `B`/`A` or `&B`/`&A`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"Marker type, inferred from the closures."
	)]
	///
	#[document_parameters("A tuple of (second arrow, first arrow).", "The input value.")]
	///
	#[document_returns("The result of applying g then f.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = compose_kleisli_flipped::<OptionBrand, _, _, _, _>(
	/// 	(|y: i32| Some(y * 2), |x: i32| Some(x + 1)),
	/// 	5,
	/// );
	/// assert_eq!(result, Some(12));
	/// ```
	pub fn compose_kleisli_flipped<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		Marker,
	>(
		gf: impl ComposeKleisliFlippedDispatch<'a, Brand, A, B, C, Marker>,
		a: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		gf.dispatch(a)
	}

	// -- ComposeKleisliFlippedDispatch --

	/// Dispatch trait for flipped Kleisli composition.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"Marker type (`Val` or `Ref`), inferred from the closures."
	)]
	#[document_parameters("The closure pair implementing this dispatch.")]
	pub trait ComposeKleisliFlippedDispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		Marker,
	> {
		/// Performs the dispatched flipped Kleisli composition.
		#[document_signature]
		#[document_parameters("The input value.")]
		#[document_returns("The result of composing g then f applied to the input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = compose_kleisli_flipped::<OptionBrand, _, _, _, _>(
		/// 	(|y: i32| Some(y * 2), |x: i32| Some(x + 1)),
		/// 	5,
		/// );
		/// assert_eq!(result, Some(12));
		/// ```
		fn dispatch(
			self,
			a: A,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>);
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"The first closure type (g -> f order).",
		"The second closure type."
	)]
	#[document_parameters("The closure pair.")]
	impl<'a, Brand, A, B, C, F, G>
		ComposeKleisliFlippedDispatch<'a, Brand, A, B, C, super::super::Val> for (F, G)
	where
		Brand: Semimonad,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		G: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The input value.")]
		#[document_returns("The composed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = compose_kleisli_flipped::<OptionBrand, _, _, _, _>(
		/// 	(|y: i32| Some(y * 2), |x: i32| Some(x + 1)),
		/// 	5,
		/// );
		/// assert_eq!(result, Some(12));
		/// ```
		fn dispatch(
			self,
			a: A,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::bind(self.1(a), self.0)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"The first closure type (g -> f order).",
		"The second closure type."
	)]
	#[document_parameters("The closure pair.")]
	impl<'a, Brand, A, B, C, F, G>
		ComposeKleisliFlippedDispatch<'a, Brand, A, B, C, super::super::Ref> for (F, G)
	where
		Brand: RefSemimonad,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(&B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		G: Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The input value.")]
		#[document_returns("The composed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let result = compose_kleisli_flipped::<LazyBrand<RcLazyConfig>, _, _, _, _>(
		/// 	(
		/// 		|y: &i32| {
		/// 			let v = *y * 2;
		/// 			RcLazy::new(move || v)
		/// 		},
		/// 		|x: &i32| {
		/// 			let v = *x + 1;
		/// 			RcLazy::new(move || v)
		/// 		},
		/// 	),
		/// 	5,
		/// );
		/// assert_eq!(*result.evaluate(), 12);
		/// ```
		fn dispatch(
			self,
			a: A,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::ref_bind(self.1(&a), self.0)
		}
	}
}

pub use inner::*;
