//! Dispatch for semimonad operations:
//! [`Semimonad`](crate::classes::Semimonad) and
//! [`RefSemimonad`](crate::classes::RefSemimonad).
//!
//! Provides the following dispatch traits and unified free functions:
//!
//! - [`BindDispatch`] + [`explicit::bind`], [`explicit::bind_flipped`]
//! - [`ComposeKleisliDispatch`] + [`compose_kleisli`], [`compose_kleisli_flipped`]
//! - [`JoinDispatch`] + [`explicit::join`]
//!
//! Each routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! 	types::*,
//! };
//!
//! // Owned: dispatches to Semimonad::bind
//! let result = bind::<OptionBrand, _, _, _, _>(Some(5), |x: i32| Some(x * 2));
//! assert_eq!(result, Some(10));
//!
//! // By-ref: dispatches to RefSemimonad::ref_bind
//! let lazy = RcLazy::pure(5);
//! let result = bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&lazy, |x: &i32| {
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
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a bind operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is inferred from the closure's argument type:
	/// `Fn(A) -> Of<B>` resolves to [`Val`](crate::dispatch::Val),
	/// `Fn(&A) -> Of<B>` resolves to [`Ref`](crate::dispatch::Ref).
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value inside the monad.",
		"The type of the result.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait BindDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker> {
		/// Perform the dispatched bind operation.
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = bind::<OptionBrand, _, _, _, _>(Some(5), |x: i32| Some(x * 2));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			ma: FA,
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
	impl<'a, Brand, A, B, F>
		BindDispatch<
			'a,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
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
		/// 	functions::explicit::*,
		/// };
		/// let result = bind::<OptionBrand, _, _, _, _>(Some(5), |x: i32| Some(x * 2));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::bind(ma, self)
		}
	}

	/// Routes `Fn(&A) -> Of<B>` closures to [`RefSemimonad::ref_bind`].
	///
	/// The container must be passed by reference (`&ma`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The brand.",
		"The input type.",
		"The output type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, F>
		BindDispatch<
			'a,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefSemimonad,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("A reference to the monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::pure(5);
		/// let result = bind::<LazyBrand<RcLazyConfig>, _, _, _, _>(&lazy, |x: &i32| {
		/// 	Lazy::<_, RcLazyConfig>::new({
		/// 		let v = *x;
		/// 		move || v * 2
		/// 	})
		/// });
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn dispatch(
			self,
			ma: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_bind(ma, self)
		}
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
	impl<'a, Brand, A, B, C, F, G> ComposeKleisliDispatch<'a, Brand, A, B, C, Val> for (F, G)
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
	impl<'a, Brand, A, B, C, F, G> ComposeKleisliDispatch<'a, Brand, A, B, C, Ref> for (F, G)
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
			Brand::ref_bind(&(self.0(&a)), self.1)
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
	/// Delegates to [`ComposeKleisliDispatch`] by swapping the tuple
	/// elements.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The higher-kinded type brand.",
		"The input type.",
		"The intermediate type.",
		"The output type.",
		"The second arrow type (`B -> Of<C>`).",
		"The first arrow type (`A -> Of<B>`).",
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
	/// let result = compose_kleisli_flipped::<OptionBrand, _, _, _, _, _, _>(
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
		F,
		G,
		Marker,
	>(
		gf: (F, G),
		a: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		(G, F): ComposeKleisliDispatch<'a, Brand, A, B, C, Marker>, {
		ComposeKleisliDispatch::dispatch((gf.1, gf.0), a)
	}

	// -- JoinDispatch --

	/// Trait that routes a join operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the container type; callers never specify it directly.
	/// Owned containers resolve to [`Val`], borrowed containers resolve to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value(s) inside the inner layer.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The container implementing this dispatch.")]
	pub trait JoinDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, Marker> {
		/// Perform the dispatched join operation.
		#[document_signature]
		///
		#[document_returns("A container with one layer of nesting removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = join::<OptionBrand, _, _>(Some(Some(5)));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	// -- Val: owned container -> Semimonad::bind(id) --

	/// Routes owned containers to [`Semimonad::bind`] with identity.
	#[document_type_parameters("The lifetime.", "The brand.", "The inner element type.")]
	#[document_parameters("The nested monadic value.")]
	impl<'a, Brand, A> JoinDispatch<'a, Brand, A, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: Semimonad,
		A: 'a,
	{
		#[document_signature]
		///
		#[document_returns("A container with one layer of nesting removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = join::<OptionBrand, _, _>(Some(Some(5)));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::bind(self, |ma| ma)
		}
	}

	// -- Ref: borrowed container -> RefSemimonad::ref_bind(clone) --

	/// Routes borrowed containers to [`RefSemimonad::ref_bind`] with clone.
	#[document_type_parameters("The lifetime.", "The brand.", "The inner element type.")]
	#[document_parameters("A reference to the nested monadic value.")]
	impl<'a, Brand, A> JoinDispatch<'a, Brand, A, Ref> for &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: RefSemimonad,
		A: 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		#[document_signature]
		///
		#[document_returns("A container with one layer of nesting removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let x = Some(Some(5));
		/// let result = join::<OptionBrand, _, _>(&x);
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_bind(self, |ma| ma.clone())
		}
	}

	// -- Inference wrappers --

	/// Sequences a monadic computation, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ma`
	/// via the `Slot` trait. For multi-brand types, the closure's input type
	/// disambiguates which brand applies. For diagonal cases, use
	/// [`explicit::bind`](crate::functions::explicit::bind) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value inside the monad.",
		"The type of the result.",
		"The brand, inferred via Slot from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The monadic value (owned for Val, borrowed for Ref).",
		"The function to apply to the value."
	)]
	///
	#[document_returns("The result of sequencing the computation.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let result = bind(Some(5), |x: i32| Some(x * 2));
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn bind<'a, FA, A: 'a, B: 'a, Brand>(
		ma: FA,
		f: impl BindDispatch<'a, Brand, A, B, FA, <FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(ma)
	}

	/// Sequences a monadic computation (flipped argument order), inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ma`
	/// via the `Slot` trait. For multi-brand types, the closure's input type
	/// disambiguates which brand applies. For diagonal cases, use
	/// [`explicit::bind_flipped`](crate::functions::explicit::bind_flipped) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The input element type.",
		"The output element type.",
		"The brand, inferred via Slot from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The function to apply to each element.",
		"The monadic value (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The result of binding the function over the value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let result = bind_flipped(|x: i32| Some(x * 2), Some(5));
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn bind_flipped<'a, FA, A: 'a, B: 'a, Brand>(
		f: impl BindDispatch<'a, Brand, A, B, FA, <FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
		ma: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(ma)
	}

	/// Removes one layer of monadic nesting, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the single `Slot` impl
	/// on FA. For single-brand types, this resolves uniquely without a
	/// closure. For multi-brand types, use
	/// [`explicit::join`](crate::functions::explicit::join) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the inner layer.",
		"The brand, inferred via Slot from FA.",
		"The inner container type (e.g., `Option<i32>` for `Option<Option<i32>>`), inferred automatically."
	)]
	///
	#[document_parameters("The nested monadic value (owned or borrowed).")]
	///
	#[document_returns("A container with one layer of nesting removed.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(join(Some(Some(5))), Some(5));
	///
	/// let x = Some(Some(5));
	/// assert_eq!(join(&x), Some(5));
	/// ```
	pub fn join<'a, FA, A: 'a, Brand, MidA: 'a>(
		mma: FA
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, MidA>
			+ JoinDispatch<'a, Brand, A, <FA as Slot_cdc7cd43dac7585f<'a, Brand, MidA>>::Marker>, {
		mma.dispatch()
	}

	// -- Explicit dispatch free functions --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Sequences a monadic computation with a function that produces the next computation.
		///
		/// Dispatches to either [`Semimonad::bind`] or [`RefSemimonad::ref_bind`]
		/// based on the closure's argument type.
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler from the closure's argument type and the container argument.
		/// Callers write `bind::<Brand, _, _, _, _>(...)` and never need to specify
		/// `Marker` or `FA` explicitly.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the monad.",
			"The type of the value inside the monad.",
			"The type of the result.",
			"The container type (owned or borrowed), inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters(
			"The monadic value (owned for Val, borrowed for Ref).",
			"The function to apply to the value."
		)]
		///
		#[document_returns("The result of sequencing the computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = bind::<OptionBrand, _, _, _, _>(Some(5), |x: i32| Some(x * 2));
		/// assert_eq!(result, Some(10));
		/// ```
		pub fn bind<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>(
			ma: FA,
			f: impl BindDispatch<'a, Brand, A, B, FA, Marker>,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			f.dispatch(ma)
		}

		/// Binds a monadic value to a function (flipped argument order).
		///
		/// Dispatches to [`Semimonad::bind`] or [`RefSemimonad::ref_bind`]
		/// based on whether the closure takes `A` or `&A`. Delegates to
		/// [`BindDispatch`] internally.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The higher-kinded type brand.",
			"The input element type.",
			"The output element type.",
			"The container type (owned or borrowed), inferred from the argument.",
			"Marker type, inferred from the closure."
		)]
		///
		#[document_parameters(
			"The function to apply to each element.",
			"The monadic value to bind over (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns("The result of binding the function over the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // By-value
		/// let result = bind_flipped::<OptionBrand, _, _, _, _>(|x: i32| Some(x * 2), Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		pub fn bind_flipped<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>(
			f: impl BindDispatch<'a, Brand, A, B, FA, Marker>,
			ma: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			f.dispatch(ma)
		}

		/// Removes one layer of monadic nesting.
		///
		/// Dispatches to either [`Semimonad::bind`] with identity or
		/// [`RefSemimonad::ref_bind`] with clone, based on whether the
		/// container is owned or borrowed.
		///
		/// The `Marker` type parameter is inferred automatically by the
		/// compiler from the container argument. Callers write
		/// `join::<Brand, _>(...)` and never need to specify `Marker` explicitly.
		///
		/// The dispatch is resolved at compile time with no runtime cost.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the monad.",
			"The type of the value(s) inside the inner layer.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters("The nested monadic value (owned or borrowed).")]
		///
		#[document_returns("A container with one layer of nesting removed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned: dispatches via Semimonad::bind(id)
		/// let y = join::<OptionBrand, _, _>(Some(Some(5)));
		/// assert_eq!(y, Some(5));
		///
		/// // By-ref: dispatches via RefSemimonad::ref_bind(clone)
		/// let x = Some(Some(5));
		/// let y = join::<OptionBrand, _, _>(&x);
		/// assert_eq!(y, Some(5));
		/// ```
		pub fn join<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, Marker>(
			mma: impl JoinDispatch<'a, Brand, A, Marker>
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			mma.dispatch()
		}
	}
}

pub use inner::*;
