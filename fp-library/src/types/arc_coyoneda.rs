//! Thread-safe reference-counted free functor with `Clone` support.
//!
//! [`ArcCoyoneda`] is a variant of [`Coyoneda`](crate::types::Coyoneda) that wraps
//! its inner layers in [`Arc`](std::sync::Arc) instead of [`Box`], making it `Clone`,
//! `Send`, and `Sync`. This enables sharing deferred map chains across threads.
//!
//! ## Trade-offs vs `RcCoyoneda`
//!
//! - **Thread safety:** `ArcCoyoneda` is `Send + Sync`. [`RcCoyoneda`](crate::types::RcCoyoneda)
//!   is not.
//! - **Overhead:** Atomic reference counting is slightly more expensive than
//!   non-atomic (`Rc`). Use `RcCoyoneda` when thread safety is not needed.
//! - **Allocation:** Each [`map`](ArcCoyoneda::map) allocates 2 `Arc` values (one for
//!   the layer, one for the function). Same as `RcCoyoneda`.
//!
//! ## Stack safety
//!
//! Each chained [`map`](ArcCoyoneda::map) adds a layer of recursion to
//! [`lower_ref`](ArcCoyoneda::lower_ref). Deep chains (thousands of maps) can overflow the stack.
//! Three mitigations are available:
//!
//! 1. **`stacker` feature (automatic).** Enable the `stacker` feature flag to use
//!    adaptive stack growth in `lower_ref`. This is transparent and handles arbitrarily
//!    deep chains with near-zero overhead when the stack is sufficient.
//! 2. **[`collapse`](ArcCoyoneda::collapse) (manual).** Call `collapse()` periodically
//!    to flatten accumulated layers. Requires `F: Functor`.
//! 3. **[`CoyonedaExplicit`](crate::types::CoyonedaExplicit) with `.boxed()`.** An
//!    alternative that accumulates maps without adding recursion depth.
//!
//! ## HKT limitations
//!
//! `ArcCoyonedaBrand` does **not** implement [`Functor`](crate::classes::Functor).
//! The HKT trait signatures lack `Send + Sync` bounds on their closure parameters,
//! so there is no way to guarantee that closures passed to `map` are safe to store
//! inside an `Arc`-wrapped layer. Use [`RcCoyonedaBrand`](crate::brands::RcCoyonedaBrand)
//! when HKT polymorphism is needed, or work with `ArcCoyoneda` directly through its
//! inherent methods.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	types::*,
//! };
//!
//! let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).map(|x| x * 2);
//!
//! // Send + Sync: can cross thread boundaries.
//! let coyo2 = coyo.clone();
//! let handle = std::thread::spawn(move || coyo2.lower_ref());
//! assert_eq!(handle.join().unwrap(), vec![4, 6, 8]);
//! assert_eq!(coyo.lower_ref(), vec![4, 6, 8]);
//! ```

#[fp_macros::document_module(no_validation)]
mod inner {
	use {
		crate::{
			brands::ArcCoyonedaBrand,
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
		std::sync::Arc,
	};

	// -- Inner trait (borrow-based lowering for Arc, requires Send + Sync) --

	/// Trait for lowering an `ArcCoyoneda` value back to its underlying functor
	/// via a shared reference. Requires `Send + Sync` for thread safety.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The output type of the accumulated mapping function."
	)]
	#[document_parameters("The trait object reference.")]
	trait ArcCoyonedaLowerRef<'a, F, A: 'a>: Send + Sync + 'a
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
		/// let coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(42));
		/// assert_eq!(coyo.lower_ref(), Some(42));
		/// ```
		fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor;
	}

	// -- Base layer --

	/// Base layer created by [`ArcCoyoneda::lift`]. Wraps `F A` with no mapping.
	/// Clones the underlying value on each call to `lower_ref`.
	struct ArcCoyonedaBase<'a, F, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>,
	}

	// SAFETY: The only field is `fa`, and we only implement ArcCoyonedaLowerRef
	// (which requires Send + Sync) when `fa: Send + Sync`.
	#[document_type_parameters("The lifetime.", "The brand.", "The value type.")]
	unsafe impl<'a, F, A: 'a> Send for ArcCoyonedaBase<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		<F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Send,
	{
	}
	#[document_type_parameters("The lifetime.", "The brand.", "The value type.")]
	unsafe impl<'a, F, A: 'a> Sync for ArcCoyonedaBase<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		<F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Sync,
	{
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the value inside the functor."
	)]
	#[document_parameters("The base layer instance.")]
	impl<'a, F, A: 'a> ArcCoyonedaLowerRef<'a, F, A> for ArcCoyonedaBase<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		<F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Clone + Send + Sync,
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
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// assert_eq!(coyo.lower_ref(), vec![1, 2, 3]);
		/// ```
		fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			self.fa.clone()
		}
	}

	// -- Map layer --

	/// Map layer created by [`ArcCoyoneda::map`]. Stores the inner value (Arc-wrapped)
	/// and an Arc-wrapped function to apply at lower time.
	struct ArcCoyonedaMapLayer<'a, F, B: 'a, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		inner: Arc<dyn ArcCoyonedaLowerRef<'a, F, B> + 'a>,
		func: Arc<dyn Fn(B) -> A + Send + Sync + 'a>,
	}

	// SAFETY: Both fields are Arc<dyn ... + Send + Sync>, which are Send + Sync.
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The input type.",
		"The output type."
	)]
	unsafe impl<'a, F, B: 'a, A: 'a> Send for ArcCoyonedaMapLayer<'a, F, B, A> where
		F: Kind_cdc7cd43dac7585f + 'a
	{
	}
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The input type.",
		"The output type."
	)]
	unsafe impl<'a, F, B: 'a, A: 'a> Sync for ArcCoyonedaMapLayer<'a, F, B, A> where
		F: Kind_cdc7cd43dac7585f + 'a
	{
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The input type of this layer's mapping function.",
		"The output type of this layer's mapping function."
	)]
	#[document_parameters("The map layer instance.")]
	impl<'a, F, B: 'a, A: 'a> ArcCoyonedaLowerRef<'a, F, A> for ArcCoyonedaMapLayer<'a, F, B, A>
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
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// assert_eq!(coyo.lower_ref(), vec![2, 3, 4]);
		/// ```
		fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			#[cfg(feature = "stacker")]
			{
				stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
					let lowered = self.inner.lower_ref();
					let func = self.func.clone();
					F::map(move |b| (*func)(b), lowered)
				})
			}
			#[cfg(not(feature = "stacker"))]
			{
				let lowered = self.inner.lower_ref();
				let func = self.func.clone();
				F::map(move |b| (*func)(b), lowered)
			}
		}
	}

	// -- Outer type --

	/// Thread-safe reference-counted free functor with `Clone` support.
	///
	/// `ArcCoyoneda` wraps its inner layers in [`Arc`], making the entire structure
	/// cheaply cloneable, `Send`, and `Sync`.
	///
	/// See the [module documentation](crate::types::arc_coyoneda) for trade-offs
	/// and examples.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	pub struct ArcCoyoneda<'a, F, A: 'a>(Arc<dyn ArcCoyonedaLowerRef<'a, F, A> + 'a>)
	where
		F: Kind_cdc7cd43dac7585f + 'a;

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	#[document_parameters("The `ArcCoyoneda` instance to clone.")]
	impl<'a, F, A: 'a> Clone for ArcCoyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Clones the `ArcCoyoneda` by bumping the atomic reference count. O(1).
		#[document_signature]
		///
		#[document_returns("A new `ArcCoyoneda` sharing the same inner layers.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// let coyo2 = coyo.clone();
		/// assert_eq!(coyo.lower_ref(), coyo2.lower_ref());
		/// ```
		fn clone(&self) -> Self {
			ArcCoyoneda(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	#[document_parameters("The `ArcCoyoneda` instance.")]
	impl<'a, F, A: 'a> ArcCoyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Lift a value of `F A` into `ArcCoyoneda F A`.
		///
		/// Requires `F::Of<'a, A>: Clone + Send + Sync` because
		/// [`lower_ref`](ArcCoyoneda::lower_ref) borrows `&self` and must produce
		/// an owned value by cloning, and `Arc` requires thread safety.
		#[document_signature]
		///
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns("An `ArcCoyoneda` wrapping the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(42));
		/// assert_eq!(coyo.lower_ref(), Some(42));
		/// ```
		pub fn lift(fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>) -> Self
		where
			<F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Clone + Send + Sync, {
			ArcCoyoneda(Arc::new(ArcCoyonedaBase {
				fa,
			}))
		}

		/// Lower the `ArcCoyoneda` back to the underlying functor `F`.
		///
		/// Applies accumulated mapping functions via `F::map`. Requires `F: Functor`.
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
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// assert_eq!(coyo.lower_ref(), vec![2, 3, 4]);
		/// ```
		pub fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			self.0.lower_ref()
		}

		/// Flatten accumulated map layers into a single base layer.
		///
		/// Resets the recursion depth by lowering and re-lifting. Useful for
		/// preventing stack overflow in deep chains when the `stacker` feature
		/// is not enabled.
		///
		/// Requires `F: Functor` (for lowering) and `F::Of<'a, A>: Clone + Send + Sync`
		/// (for re-lifting).
		#[document_signature]
		///
		#[document_returns("A new `ArcCoyoneda` with a single base layer.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).map(|x| x * 2);
		/// let collapsed = coyo.collapse();
		/// assert_eq!(collapsed.lower_ref(), vec![4, 6, 8]);
		/// ```
		pub fn collapse(&self) -> ArcCoyoneda<'a, F, A>
		where
			F: Functor,
			<F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Clone + Send + Sync, {
			ArcCoyoneda::lift(self.lower_ref())
		}

		/// Map a function over the `ArcCoyoneda` value.
		///
		/// The function must be `Send + Sync` for thread safety. It is wrapped in
		/// [`Arc`] so it does not need to implement `Clone`.
		#[document_signature]
		///
		#[document_type_parameters("The new output type after applying the function.")]
		///
		#[document_parameters("The function to apply.")]
		///
		#[document_returns(
			"A new `ArcCoyoneda` with the function stored for deferred application."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x * 2).map(|x| x + 1);
		/// assert_eq!(coyo.lower_ref(), Some(11));
		/// ```
		pub fn map<B: 'a>(
			self,
			f: impl Fn(A) -> B + Send + Sync + 'a,
		) -> ArcCoyoneda<'a, F, B> {
			ArcCoyoneda(Arc::new(ArcCoyonedaMapLayer {
				inner: self.0,
				func: Arc::new(f),
			}))
		}
	}

	// -- Brand --

	impl_kind! {
		impl<F: Kind_cdc7cd43dac7585f + 'static> for ArcCoyonedaBrand<F> {
			type Of<'a, A: 'a>: 'a = ArcCoyoneda<'a, F, A>;
		}
	}

	// Note: ArcCoyonedaBrand does NOT implement Functor. The HKT trait signatures
	// lack Send + Sync bounds on closure parameters, so there is no way to guarantee
	// that closures passed to map are safe to store inside an Arc-wrapped layer.
	// This is the same limitation as SendThunkBrand. Use RcCoyonedaBrand when HKT
	// polymorphism is needed, or work with ArcCoyoneda directly via inherent methods.

	// -- Foldable implementation --

	#[document_type_parameters("The brand of the underlying foldable functor.")]
	impl<F: Functor + Foldable + 'static> Foldable for ArcCoyonedaBrand<F> {
		/// Folds the `ArcCoyoneda` by lowering to the underlying functor and delegating.
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
			"The `ArcCoyoneda` structure to fold."
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
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		/// let result =
		/// 	fold_map::<RcFnBrand, ArcCoyonedaBrand<VecBrand>, _, _>(|x: i32| x.to_string(), coyo);
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
	fn lift_lower_ref_identity() {
		let coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(42));
		assert_eq!(coyo.lower_ref(), Some(42));
	}

	#[test]
	fn chained_maps() {
		let result = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3])
			.map(|x| x + 1)
			.map(|x| x * 2)
			.lower_ref();
		assert_eq!(result, vec![4, 6, 8]);
	}

	#[test]
	fn clone_and_lower() {
		let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		let coyo2 = coyo.clone();
		assert_eq!(coyo.lower_ref(), vec![2, 3, 4]);
		assert_eq!(coyo2.lower_ref(), vec![2, 3, 4]);
	}

	#[test]
	fn send_across_thread() {
		let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let handle = std::thread::spawn(move || coyo.lower_ref());
		assert_eq!(handle.join().unwrap(), vec![10, 20, 30]);
	}

	#[test]
	fn fold_map_on_mapped() {
		let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let result =
			fold_map::<RcFnBrand, ArcCoyonedaBrand<VecBrand>, _, _>(|x: i32| x.to_string(), coyo);
		assert_eq!(result, "102030".to_string());
	}

	#[test]
	fn map_on_none_stays_none() {
		let result = ArcCoyoneda::<OptionBrand, _>::lift(None::<i32>).map(|x| x + 1).lower_ref();
		assert_eq!(result, None);
	}
}
