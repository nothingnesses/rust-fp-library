//! Reference-counted free functor with `Clone` support.
//!
//! [`RcCoyoneda`] is a variant of [`Coyoneda`](crate::types::Coyoneda) that wraps
//! its inner layers in [`Rc`](std::rc::Rc) instead of [`Box`], making it `Clone`.
//! This enables sharing and reuse of deferred map chains without consuming the value.
//!
//! ## Trade-offs vs `Coyoneda`
//!
//! - **Clone:** `RcCoyoneda` is `Clone` (cheap refcount bump). `Coyoneda` is not.
//! - **Allocation:** Each [`map`](RcCoyoneda::map) allocates 2 `Rc` values (one for
//!   the layer, one for the function). `Coyoneda` allocates 1 `Box`.
//! - **`lower_ref`:** [`lower_ref`](RcCoyoneda::lower_ref) borrows `&self` and
//!   clones `F::Of<'a, A>` at the base layer. [`lower`](crate::types::Coyoneda::lower)
//!   consumes `self` without cloning.
//! - **Thread safety:** `RcCoyoneda` is `!Send`. Use
//!   [`ArcCoyoneda`](crate::types::ArcCoyoneda) for thread-safe contexts.
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
//! let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).map(|x| x * 2);
//!
//! // Clone is cheap (refcount bump).
//! let coyo2 = coyo.clone();
//!
//! assert_eq!(coyo.lower_ref(), vec![4, 6, 8]);
//! assert_eq!(coyo2.lower_ref(), vec![4, 6, 8]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::RcCoyonedaBrand,
			classes::{
				CloneableFn,
				Foldable,
				Functor,
				Monoid,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
		std::rc::Rc,
	};

	// -- Inner trait (borrow-based lowering for Rc) --

	/// Trait for lowering an `RcCoyoneda` value back to its underlying functor
	/// via a shared reference.
	///
	/// Unlike the `CoyonedaInner` trait which consumes `Box<Self>`,
	/// this trait borrows `&self`, enabling `Clone` on the outer `Rc` wrapper.
	/// The base layer clones `F::Of<'a, A>` to produce an owned value from
	/// the borrow.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The output type of the accumulated mapping function."
	)]
	#[document_parameters("The trait object reference.")]
	trait RcCoyonedaLowerRef<'a, F, A: 'a>: 'a
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		/// Lower to the concrete functor by applying accumulated functions via `F::map`.
		#[document_signature]
		///
		#[document_returns("The underlying functor value with accumulated functions applied.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(42));
		/// assert_eq!(coyo.lower_ref(), Some(42));
		/// ```
		fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor;
	}

	// -- Base layer: wraps F<A> directly (identity mapping, clones on lower) --

	/// Base layer created by [`RcCoyoneda::lift`]. Wraps `F A` with no mapping.
	/// Clones the underlying value on each call to `lower_ref`.
	struct RcCoyonedaBase<'a, F, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the value inside the functor."
	)]
	#[document_parameters("The base layer instance.")]
	impl<'a, F, A: 'a> RcCoyonedaLowerRef<'a, F, A> for RcCoyonedaBase<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		<F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Clone,
	{
		/// Returns the wrapped value by cloning.
		#[document_signature]
		///
		#[document_returns("A clone of the underlying functor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// assert_eq!(coyo.lower_ref(), vec![1, 2, 3]);
		/// ```
		fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			self.fa.clone()
		}
	}

	// -- Map layer: wraps an inner RcCoyoneda and adds an Rc-wrapped function --

	/// Map layer created by [`RcCoyoneda::map`]. Stores the inner value (Rc-wrapped)
	/// and an Rc-wrapped function to apply at lower time.
	struct RcCoyonedaMapLayer<'a, F, B: 'a, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		inner: Rc<dyn RcCoyonedaLowerRef<'a, F, B> + 'a>,
		func: Rc<dyn Fn(B) -> A + 'a>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The input type of this layer's mapping function.",
		"The output type of this layer's mapping function."
	)]
	#[document_parameters("The map layer instance.")]
	impl<'a, F, B: 'a, A: 'a> RcCoyonedaLowerRef<'a, F, A> for RcCoyonedaMapLayer<'a, F, B, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Lowers the inner value, then applies this layer's function via `F::map`.
		#[document_signature]
		///
		#[document_returns("The underlying functor value with this layer's function applied.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// assert_eq!(coyo.lower_ref(), vec![2, 3, 4]);
		/// ```
		fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			let lowered = self.inner.lower_ref();
			let func = self.func.clone();
			F::map(move |b| (*func)(b), lowered)
		}
	}

	// -- Outer type --

	/// Reference-counted free functor with `Clone` support.
	///
	/// `RcCoyoneda` wraps its inner layers in [`Rc`], making the entire structure
	/// cheaply cloneable (refcount bump). This enables sharing and reuse of
	/// deferred map chains without consuming the value.
	///
	/// See the [module documentation](crate::types::rc_coyoneda) for trade-offs
	/// and examples.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	pub struct RcCoyoneda<'a, F, A: 'a>(Rc<dyn RcCoyonedaLowerRef<'a, F, A> + 'a>)
	where
		F: Kind_cdc7cd43dac7585f + 'a;

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	#[document_parameters("The `RcCoyoneda` instance to clone.")]
	impl<'a, F, A: 'a> Clone for RcCoyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Clones the `RcCoyoneda` by bumping the reference count. O(1).
		#[document_signature]
		///
		#[document_returns("A new `RcCoyoneda` sharing the same inner layers.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// let coyo2 = coyo.clone();
		/// assert_eq!(coyo.lower_ref(), coyo2.lower_ref());
		/// ```
		fn clone(&self) -> Self {
			RcCoyoneda(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	#[document_parameters("The `RcCoyoneda` instance.")]
	impl<'a, F, A: 'a> RcCoyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Lift a value of `F A` into `RcCoyoneda F A`.
		///
		/// Requires `F::Of<'a, A>: Clone` because [`lower_ref`](RcCoyoneda::lower_ref)
		/// borrows `&self` and must produce an owned value by cloning at the base layer.
		#[document_signature]
		///
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns("An `RcCoyoneda` wrapping the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(42));
		/// assert_eq!(coyo.lower_ref(), Some(42));
		/// ```
		pub fn lift(fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>) -> Self
		where
			<F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Clone, {
			RcCoyoneda(Rc::new(RcCoyonedaBase {
				fa,
			}))
		}

		/// Lower the `RcCoyoneda` back to the underlying functor `F`.
		///
		/// Applies accumulated mapping functions via `F::map`. Requires `F: Functor`.
		/// Unlike [`Coyoneda::lower`](crate::types::Coyoneda::lower), this borrows
		/// `&self` rather than consuming it, cloning the base value.
		#[document_signature]
		///
		#[document_returns("The underlying functor value with all accumulated functions applied.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// assert_eq!(coyo.lower_ref(), vec![2, 3, 4]);
		/// // Can call again since it borrows.
		/// assert_eq!(coyo.lower_ref(), vec![2, 3, 4]);
		/// ```
		pub fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			self.0.lower_ref()
		}

		/// Map a function over the `RcCoyoneda` value.
		///
		/// Wraps the function in [`Rc`] so it does not need to implement `Clone`.
		/// The function is applied when [`lower_ref`](RcCoyoneda::lower_ref) is called.
		#[document_signature]
		///
		#[document_type_parameters("The new output type after applying the function.")]
		///
		#[document_parameters("The function to apply.")]
		///
		#[document_returns("A new `RcCoyoneda` with the function stored for deferred application.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x * 2).map(|x| x + 1);
		/// assert_eq!(coyo.lower_ref(), Some(11));
		/// ```
		pub fn map<B: 'a>(
			self,
			f: impl Fn(A) -> B + 'a,
		) -> RcCoyoneda<'a, F, B> {
			RcCoyoneda(Rc::new(RcCoyonedaMapLayer {
				inner: self.0,
				func: Rc::new(f),
			}))
		}
	}

	// -- Brand --

	impl_kind! {
		impl<F: Kind_cdc7cd43dac7585f + 'static> for RcCoyonedaBrand<F> {
			type Of<'a, A: 'a>: 'a = RcCoyoneda<'a, F, A>;
		}
	}

	// -- Functor implementation --

	#[document_type_parameters("The brand of the underlying type constructor.")]
	impl<F: Kind_cdc7cd43dac7585f + 'static> Functor for RcCoyonedaBrand<F> {
		/// Maps a function over the `RcCoyoneda` value by adding a new mapping layer.
		///
		/// Does not require `F: Functor`. The function is stored and applied at
		/// [`lower_ref`](RcCoyoneda::lower_ref) time.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the current output.",
			"The type of the new output."
		)]
		///
		#[document_parameters("The function to apply.", "The `RcCoyoneda` value.")]
		///
		#[document_returns("A new `RcCoyoneda` with the function stored for deferred application.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// let mapped = map::<RcCoyonedaBrand<VecBrand>, _, _>(|x| x * 10, coyo);
		/// assert_eq!(mapped.lower_ref(), vec![10, 20, 30]);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	// -- Foldable implementation --

	#[document_type_parameters("The brand of the underlying foldable functor.")]
	impl<F: Functor + Foldable + 'static> Foldable for RcCoyonedaBrand<F> {
		/// Folds the `RcCoyoneda` by lowering to the underlying functor and delegating.
		///
		/// Requires `F: Functor` (for lowering) and `F: Foldable`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters(
			"The function to map each element to a monoid.",
			"The `RcCoyoneda` structure to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		/// let result =
		/// 	fold_map::<RcFnBrand, RcCoyonedaBrand<VecBrand>, _, _>(|x: i32| x.to_string(), coyo);
		/// assert_eq!(result, "102030".to_string());
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			F::fold_map::<FnBrand, A, M>(func, fa.lower_ref())
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use crate::{
		brands::*,
		functions::*,
		types::*,
	};

	#[test]
	fn lift_lower_ref_identity_option() {
		let coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(42));
		assert_eq!(coyo.lower_ref(), Some(42));
	}

	#[test]
	fn lift_lower_ref_identity_vec() {
		let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		assert_eq!(coyo.lower_ref(), vec![1, 2, 3]);
	}

	#[test]
	fn clone_and_lower_ref() {
		let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		let coyo2 = coyo.clone();
		assert_eq!(coyo.lower_ref(), vec![2, 3, 4]);
		assert_eq!(coyo2.lower_ref(), vec![2, 3, 4]);
	}

	#[test]
	fn chained_maps() {
		let result = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3])
			.map(|x| x + 1)
			.map(|x| x * 2)
			.map(|x| x.to_string())
			.lower_ref();
		assert_eq!(result, vec!["4", "6", "8"]);
	}

	#[test]
	fn functor_identity_law() {
		let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let result = map::<RcCoyonedaBrand<VecBrand>, _, _>(identity, coyo).lower_ref();
		assert_eq!(result, vec![1, 2, 3]);
	}

	#[test]
	fn functor_composition_law() {
		let f = |x: i32| x + 1;
		let g = |x: i32| x * 2;

		let coyo1 = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let left = map::<RcCoyonedaBrand<VecBrand>, _, _>(compose(f, g), coyo1).lower_ref();

		let coyo2 = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let right = map::<RcCoyonedaBrand<VecBrand>, _, _>(
			f,
			map::<RcCoyonedaBrand<VecBrand>, _, _>(g, coyo2),
		)
		.lower_ref();

		assert_eq!(left, right);
	}

	// -- Foldable tests --

	#[test]
	fn fold_map_on_mapped() {
		let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let result =
			fold_map::<RcFnBrand, RcCoyonedaBrand<VecBrand>, _, _>(|x: i32| x.to_string(), coyo);
		assert_eq!(result, "102030".to_string());
	}

	#[test]
	fn lower_ref_multiple_times() {
		let coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x + 1);
		assert_eq!(coyo.lower_ref(), Some(6));
		assert_eq!(coyo.lower_ref(), Some(6));
		assert_eq!(coyo.lower_ref(), Some(6));
	}

	#[test]
	fn map_on_none_stays_none() {
		let result = RcCoyoneda::<OptionBrand, _>::lift(None::<i32>)
			.map(|x| x + 1)
			.map(|x| x * 2)
			.lower_ref();
		assert_eq!(result, None);
	}
}
