//! Dispatch for [`Semiapplicative::apply`](crate::classes::Semiapplicative::apply).
//!
//! Provides the [`FnBrandSlot`] trait for FnBrand inference and an inference
//! wrapper [`apply`] that uses dual Slot bounds to infer both Brand and FnBrand
//! from the container types.
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
//! let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
//! let x = Some(5);
//! let y = explicit::apply::<RcFnBrand, OptionBrand, _, _>(f, x);
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				CloneFn,
				Semiapplicative,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Maps a concrete wrapped-function type back to its `FnBrand`.
	///
	/// Each concrete wrapper type (`Rc<dyn Fn(A) -> B>`, `Arc<dyn Fn(A) -> B>`)
	/// has a unique `FnBrandSlot` impl keyed on its `FnBrand`. The solver uses
	/// this to infer `FnBrand` from the element type extracted by the Brand Slot.
	#[document_type_parameters(
		"The function-wrapping brand (e.g., RcFnBrand, ArcFnBrand).",
		"The input type of the wrapped function.",
		"The output type of the wrapped function."
	)]
	pub trait FnBrandSlot<FnBrand, A, B> {}

	impl<'a, A: 'a, B: 'a> FnBrandSlot<crate::brands::RcFnBrand, A, B>
		for std::rc::Rc<dyn 'a + Fn(A) -> B>
	{
	}

	impl<'a, A: 'a, B: 'a> FnBrandSlot<crate::brands::ArcFnBrand, A, B>
		for std::sync::Arc<dyn 'a + Fn(A) -> B + Send + Sync>
	{
	}

	// -- Inference wrapper --

	/// Applies a container of functions to a container of values, inferring
	/// both Brand and FnBrand from the container types.
	///
	/// Brand is resolved by intersecting the Slot bounds on FF (the function
	/// container) and FA (the value container). FnBrand is resolved via
	/// [`FnBrandSlot`] from the wrapper type W extracted by the Brand Slot.
	/// No turbofish arguments are needed.
	///
	/// For types where inference fails, use
	/// [`explicit::apply`](crate::functions::explicit::apply) with a turbofish.
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
	/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let x = Some(5);
	/// let y: Option<i32> = apply(f, x);
	/// assert_eq!(y, Some(10));
	/// ```
	#[allow_named_generics]
	pub fn apply<'a, FnBrand, Brand, A, B, W, FF, FA>(
		ff: FF,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FnBrand: CloneFn + 'a,
		Brand: Semiapplicative,
		A: Clone + 'a,
		B: 'a,
		W: 'a + Clone + FnBrandSlot<FnBrand, A, B>,
		FF: Slot_cdc7cd43dac7585f<'a, Brand, W>
			+ Into<
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, W>),
			>,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>
			+ Into<
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			>,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, W>): Into<
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
		>, {
		let ff_branded: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, W>) = ff.into();
		let ff_cast: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>) =
			ff_branded.into();
		Brand::apply::<FnBrand, A, B>(ff_cast, fa.into())
	}
}

pub use inner::*;
