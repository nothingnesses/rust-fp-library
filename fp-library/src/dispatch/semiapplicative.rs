//! Dispatch for [`Semiapplicative::apply`](crate::classes::Semiapplicative::apply)
//! and [`RefSemiapplicative::ref_apply`](crate::classes::RefSemiapplicative::ref_apply).
//!
//! Provides unified Val/Ref dispatch via the [`ApplyDispatch`] trait, the
//! [`FnBrandSlot`] trait for FnBrand inference, and an inference wrapper
//! [`apply`] that infers both Brand and FnBrand from the container types.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! };
//!
//! // Val: owned containers, Fn(A) -> B closures
//! let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
//! let x = Some(5);
//! let y: Option<i32> = apply(f, x);
//! assert_eq!(y, Some(10));
//!
//! // Ref: borrowed containers, Fn(&A) -> B closures
//! let f = Some(std::rc::Rc::new(|x: &i32| *x * 2) as std::rc::Rc<dyn Fn(&i32) -> i32>);
//! let x = Some(5);
//! let y: Option<i32> = apply(&f, &x);
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				CloneFn,
				RefSemiapplicative,
				Semiapplicative,
			},
			dispatch::{
				ClosureMode,
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Maps a concrete wrapped-function type back to its `FnBrand`,
	/// parameterized by closure mode.
	///
	/// Each concrete wrapper type has impls for both Val and Ref modes:
	/// - Val: `Rc<dyn Fn(A) -> B>` -> `RcFnBrand`
	/// - Ref: `Rc<dyn Fn(&A) -> B>` -> `RcFnBrand`
	///
	/// The Mode parameter disambiguates the two, since
	/// `Rc<dyn Fn(&A) -> B>` would otherwise match the Val impl with
	/// `A` = `&A'`.
	#[document_type_parameters(
		"The function-wrapping brand (e.g., RcFnBrand, ArcFnBrand).",
		"The input type of the wrapped function.",
		"The output type of the wrapped function.",
		"The closure mode (Val or Ref)."
	)]
	pub trait FnBrandSlot<FnBrand, A, B, Mode = Val> {}

	// -- RcFnBrand impls --

	/// Maps `Rc<dyn Fn(A) -> B>` back to `RcFnBrand` (Val mode).
	#[document_type_parameters(
		"The lifetime of the wrapped function.",
		"The input type of the wrapped function.",
		"The output type of the wrapped function."
	)]
	impl<'a, A: 'a, B: 'a> FnBrandSlot<crate::brands::RcFnBrand, A, B, Val>
		for std::rc::Rc<dyn 'a + Fn(A) -> B>
	{
	}

	/// Maps `Rc<dyn Fn(&A) -> B>` back to `RcFnBrand` (Ref mode).
	#[document_type_parameters(
		"The lifetime of the wrapped function.",
		"The input type of the wrapped function.",
		"The output type of the wrapped function."
	)]
	impl<'a, A: 'a, B: 'a> FnBrandSlot<crate::brands::RcFnBrand, A, B, Ref>
		for std::rc::Rc<dyn 'a + Fn(&A) -> B>
	{
	}

	// -- ArcFnBrand impls --

	/// Maps `Arc<dyn Fn(A) -> B + Send + Sync>` back to `ArcFnBrand` (Val mode).
	#[document_type_parameters(
		"The lifetime of the wrapped function.",
		"The input type of the wrapped function.",
		"The output type of the wrapped function."
	)]
	impl<'a, A: 'a, B: 'a> FnBrandSlot<crate::brands::ArcFnBrand, A, B, Val>
		for std::sync::Arc<dyn 'a + Fn(A) -> B + Send + Sync>
	{
	}

	/// Maps `Arc<dyn Fn(&A) -> B + Send + Sync>` back to `ArcFnBrand` (Ref mode).
	#[document_type_parameters(
		"The lifetime of the wrapped function.",
		"The input type of the wrapped function.",
		"The output type of the wrapped function."
	)]
	impl<'a, A: 'a, B: 'a> FnBrandSlot<crate::brands::ArcFnBrand, A, B, Ref>
		for std::sync::Arc<dyn 'a + Fn(&A) -> B + Send + Sync>
	{
	}

	// -- ApplyDispatch trait --

	/// Trait that routes an apply operation to the appropriate type class method.
	///
	/// The `Marker` type parameter selects Val or Ref dispatch:
	/// - Val: routes to [`Semiapplicative::apply`] (owned containers, `Fn(A) -> B`)
	/// - Ref: routes to [`RefSemiapplicative::ref_apply`] (borrowed containers, `Fn(&A) -> B`)
	#[document_type_parameters(
		"The lifetime of the values.",
		"The function-wrapping brand.",
		"The brand of the applicative.",
		"The type of the value(s) inside the value container.",
		"The result type after applying the function.",
		"The concrete wrapped-function type.",
		"The value container type.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The function container implementing this dispatch.")]
	pub trait ApplyDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		W: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched apply operation.
		#[document_signature]
		///
		#[document_parameters("The value container to apply the function(s) to.")]
		///
		#[document_returns("A new container with the function(s) applied to the value(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let y: Option<i32> = apply(f, Some(5));
		/// assert_eq!(y, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val impl: owned containers -> Semiapplicative::apply --

	/// Routes owned containers to [`Semiapplicative::apply`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The function-wrapping brand.",
		"The brand of the applicative.",
		"The type of the value(s) inside the value container.",
		"The result type after applying the function.",
		"The concrete wrapped-function type."
	)]
	#[document_parameters("The owned function container.")]
	impl<'a, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, W>)
	where
		FnBrand: CloneFn + 'a,
		Brand: Semiapplicative,
		A: Clone + 'a,
		B: 'a,
		W: Clone + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, W>): Into<
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
		>,
	{
		#[document_signature]
		///
		#[document_parameters("The value container to apply the function(s) to.")]
		///
		#[document_returns("A new container with the function(s) applied to the value(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let y: Option<i32> = apply(f, Some(5));
		/// assert_eq!(y, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// -- Ref impl: borrowed containers -> RefSemiapplicative::ref_apply --

	/// Routes borrowed containers to [`RefSemiapplicative::ref_apply`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The function-wrapping brand.",
		"The brand of the applicative.",
		"The type of the value(s) inside the value container.",
		"The result type after applying the function.",
		"The concrete wrapped-function type."
	)]
	#[document_parameters("The borrowed function container.")]
	impl<'a, 'b, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, W>)
	where
		'a: 'b,
		FnBrand: CloneFn<Ref> + 'a,
		Brand: RefSemiapplicative,
		A: 'a,
		B: 'a,
		W: 'a,
		&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, W>): Into<
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
		>,
	{
		#[document_signature]
		///
		#[document_parameters("The borrowed value container to apply the function(s) to.")]
		///
		#[document_returns("A new container with the function(s) applied to the value(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = Some(std::rc::Rc::new(|x: &i32| *x * 2) as std::rc::Rc<dyn Fn(&i32) -> i32>);
		/// let x = Some(5);
		/// let y: Option<i32> = apply(&f, &x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// -- Inference wrapper --

	/// Applies a container of functions to a container of values, inferring
	/// Brand, FnBrand, and Val/Ref dispatch from the container types.
	///
	/// - Brand is resolved via dual Slot bounds on FF and FA.
	/// - FnBrand is resolved via [`FnBrandSlot`] from the wrapper type W.
	/// - Val/Ref dispatch is resolved via the Marker projected from FA's Slot.
	/// - The `CloneFn` mode is tied to the Marker via `CloneFn<Marker>`.
	///
	/// No turbofish arguments are needed for single-brand types.
	///
	/// For types where inference fails (e.g., multi-brand types like Result),
	/// use [`explicit::apply`](crate::functions::explicit::apply) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The function-wrapping brand, inferred via FnBrandSlot.",
		"The brand, inferred via Slot from FF and FA.",
		"The type of the value(s) inside the value container.",
		"The result type after applying the function.",
		"The concrete wrapped-function type, inferred from FF's element type.",
		"The function container type.",
		"The value container type."
	)]
	///
	#[document_parameters(
		"The container of function(s) to apply.",
		"The container of value(s) to apply the function(s) to."
	)]
	///
	#[document_returns("A new container with the function(s) applied to the value(s).")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// // Val: owned containers
	/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let y: Option<i32> = apply(f, Some(5));
	/// assert_eq!(y, Some(10));
	///
	/// // Ref: borrowed containers
	/// let f = Some(std::rc::Rc::new(|x: &i32| *x * 2) as std::rc::Rc<dyn Fn(&i32) -> i32>);
	/// let x = Some(5);
	/// let y: Option<i32> = apply(&f, &x);
	/// assert_eq!(y, Some(10));
	/// ```
	#[allow_named_generics]
	pub fn apply<'a, FnBrand, Brand, A, B, W, FF, FA>(
		ff: FF,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		W: 'a,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>,
		<FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker: ClosureMode,
		FnBrand: CloneFn<<FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker> + 'a,
		W: FnBrandSlot<FnBrand, A, B, <FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
		FF: Slot_cdc7cd43dac7585f<'a, Brand, W>
			+ ApplyDispatch<
				'a,
				FnBrand,
				Brand,
				A,
				B,
				W,
				FA,
				<FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
			>, {
		ff.dispatch(fa)
	}
}

pub use inner::*;
