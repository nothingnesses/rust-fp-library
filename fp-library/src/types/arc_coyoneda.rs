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

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::ArcCoyonedaBrand,
			classes::{
				Lift,
				NaturalTransformation,
				*,
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
		fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor;
	}

	// -- Base layer --

	/// Base layer created by [`ArcCoyoneda::lift`]. Wraps `F A` with no mapping.
	/// Clones the underlying value on each call to `lower_ref`.
	struct ArcCoyonedaBase<'a, F, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f<Of<'a, A>: Send + Sync> + 'a, {
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
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
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync,
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
		fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
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

	// Send + Sync auto-derived: both fields are `Arc<dyn ... + Send + Sync>`.
	// `F` only appears inside erased trait object bounds, not as concrete field
	// data, so the compiler does not need `F::Of` to be `Send`/`Sync`.
	// Compile-time assertions at the bottom of this module guard against regressions.

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
		fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
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

	// -- New layer: wraps F<B> and an Arc-wrapped function (single allocation) --

	/// New layer created by [`ArcCoyoneda::new`]. Stores the functor value and an
	/// Arc-wrapped function, implementing `ArcCoyonedaLowerRef` directly in a single
	/// Arc allocation.
	struct ArcCoyonedaNewLayer<'a, F, B: 'a, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f<Of<'a, B>: Send + Sync> + 'a, {
		fb: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		func: Arc<dyn Fn(B) -> A + Send + Sync + 'a>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The input type of the stored function.",
		"The output type of the stored function."
	)]
	#[document_parameters("The new layer instance.")]
	impl<'a, F, B: 'a, A: 'a> ArcCoyonedaLowerRef<'a, F, A> for ArcCoyonedaNewLayer<'a, F, B, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone + Send + Sync,
	{
		/// Applies the stored function to the stored functor value via `F::map`.
		#[document_signature]
		///
		#[document_returns("The underlying functor value with the stored function applied.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(coyo.lower_ref(), vec![2, 4, 6]);
		/// ```
		fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor, {
			let func = self.func.clone();
			F::map(move |b| (*func)(b), self.fb.clone())
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
		pub fn lift(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> Self
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync, {
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
		pub fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
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
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync, {
			ArcCoyoneda::lift(self.lower_ref())
		}

		/// Fold the `ArcCoyoneda` by lowering and delegating to `F::fold_map`.
		///
		/// Non-consuming alternative to the `Foldable` trait method, which takes
		/// the value by move. This borrows `&self` via `lower_ref`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The function to map each element to a monoid.")]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		/// let result = coyo.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		/// assert_eq!(result, "102030");
		/// // Can still use coyo after folding.
		/// assert_eq!(coyo.lower_ref(), vec![10, 20, 30]);
		/// ```
		pub fn fold_map<FnBrand: LiftFn + 'a, M>(
			&self,
			func: impl Fn(A) -> M + 'a,
		) -> M
		where
			F: Functor + Foldable,
			A: Clone,
			M: Monoid + 'a, {
			F::fold_map::<FnBrand, A, M>(func, self.lower_ref())
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

		/// Create an `ArcCoyoneda` from a function and a functor value.
		///
		/// This is more efficient than `lift(fb).map(f)` because it creates
		/// a single layer instead of two.
		#[document_signature]
		///
		#[document_type_parameters("The input type of the function.")]
		///
		#[document_parameters("The function to defer.", "The functor value.")]
		///
		#[document_returns("An `ArcCoyoneda` wrapping the value with the deferred function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(coyo.lower_ref(), vec![2, 4, 6]);
		/// ```
		pub fn new<B: 'a>(
			f: impl Fn(B) -> A + Send + Sync + 'a,
			fb: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Self
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone + Send + Sync, {
			ArcCoyoneda(Arc::new(ArcCoyonedaNewLayer {
				fb,
				func: Arc::new(f),
			}))
		}

		/// Apply a natural transformation to change the underlying functor.
		///
		/// Lowers to `F A` (applying all accumulated maps), transforms via the
		/// natural transformation, then re-lifts into `ArcCoyoneda G A`.
		///
		/// Requires `F: Functor` for lowering.
		#[document_signature]
		///
		#[document_type_parameters("The target functor brand.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("A new `ArcCoyoneda` over the target functor `G`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// struct VecToOption;
		/// impl NaturalTransformation<VecBrand, OptionBrand> for VecToOption {
		/// 	fn transform<'a, A: 'a>(
		/// 		&self,
		/// 		fa: Vec<A>,
		/// 	) -> Option<A> {
		/// 		fa.into_iter().next()
		/// 	}
		/// }
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![10, 20, 30]);
		/// let hoisted = coyo.hoist(VecToOption);
		/// assert_eq!(hoisted.lower_ref(), Some(10));
		/// ```
		pub fn hoist<G: Kind_cdc7cd43dac7585f + 'a>(
			self,
			nat: impl NaturalTransformation<F, G>,
		) -> ArcCoyoneda<'a, G, A>
		where
			F: Functor,
			Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync, {
			ArcCoyoneda::lift(nat.transform(self.lower_ref()))
		}

		/// Wrap a value in the `ArcCoyoneda` context by delegating to `F::pure`.
		///
		/// Requires `F::Of<'a, A>: Clone + Send + Sync` for the base layer.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcCoyoneda` containing the pure value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<OptionBrand, i32>::pure(42);
		/// assert_eq!(coyo.lower_ref(), Some(42));
		/// ```
		pub fn pure(a: A) -> Self
		where
			F: Pointed,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone + Send + Sync, {
			ArcCoyoneda::lift(F::pure(a))
		}

		/// Chain `ArcCoyoneda` computations by lowering, binding, and re-lifting.
		///
		/// This is a fusion barrier: all accumulated maps are applied before binding.
		///
		/// Requires `F: Functor` (for lowering) and `F: Semimonad` (for binding).
		#[document_signature]
		///
		#[document_type_parameters("The output type of the bound computation.")]
		///
		#[document_parameters(
			"The function to apply to the inner value, returning a new `ArcCoyoneda`."
		)]
		///
		#[document_returns("A new `ArcCoyoneda` containing the bound result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(5));
		/// let result = coyo.bind(|x| ArcCoyoneda::<OptionBrand, _>::lift(Some(x * 2)));
		/// assert_eq!(result.lower_ref(), Some(10));
		/// ```
		pub fn bind<B: 'a>(
			self,
			func: impl Fn(A) -> ArcCoyoneda<'a, F, B> + 'a,
		) -> ArcCoyoneda<'a, F, B>
		where
			F: Functor + Semimonad,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone + Send + Sync, {
			ArcCoyoneda::lift(F::bind(self.lower_ref(), move |a| func(a).lower_ref()))
		}

		/// Apply a function inside an `ArcCoyoneda` to a value inside another.
		///
		/// Both arguments are lowered and delegated to `F::apply`, then re-lifted.
		/// This is a fusion barrier.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The type of the function input.",
			"The type of the function output."
		)]
		///
		#[document_parameters(
			"The `ArcCoyoneda` containing the function(s).",
			"The `ArcCoyoneda` containing the value(s)."
		)]
		///
		#[document_returns("A new `ArcCoyoneda` containing the applied result(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // For thread-safe functors, prefer lift2 over apply.
		/// // The apply method requires CloneableFn::Of to be Send + Sync,
		/// // which standard FnBrands do not satisfy (CloneableFn::Of wraps
		/// // dyn Fn without Send + Sync bounds). Use lift2 instead:
		/// let a = ArcCoyoneda::<OptionBrand, _>::lift(Some(3));
		/// let b = ArcCoyoneda::<OptionBrand, _>::lift(Some(4));
		/// let result = a.lift2(|x, y| x + y, b);
		/// assert_eq!(result.lower_ref(), Some(7));
		/// ```
		pub fn apply<FnBrand: LiftFn + 'a, B: Clone + 'a, C: 'a>(
			ff: ArcCoyoneda<'a, F, <FnBrand as CloneableFn>::Of<'a, B, C>>,
			fa: ArcCoyoneda<'a, F, B>,
		) -> ArcCoyoneda<'a, F, C>
		where
			F: Functor + Semiapplicative,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone + Send + Sync, {
			ArcCoyoneda::lift(F::apply::<FnBrand, B, C>(ff.lower_ref(), fa.lower_ref()))
		}

		/// Lift a binary function into the `ArcCoyoneda` context.
		///
		/// Both arguments are lowered and delegated to `F::lift2`, then re-lifted.
		/// This is a fusion barrier.
		#[document_signature]
		///
		#[document_type_parameters("The type of the second value.", "The type of the result.")]
		///
		#[document_parameters("The binary function to apply.", "The second `ArcCoyoneda` value.")]
		///
		#[document_returns("An `ArcCoyoneda` containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let a = ArcCoyoneda::<OptionBrand, _>::lift(Some(3));
		/// let b = ArcCoyoneda::<OptionBrand, _>::lift(Some(4));
		/// let result = a.lift2(|x, y| x + y, b);
		/// assert_eq!(result.lower_ref(), Some(7));
		/// ```
		pub fn lift2<B: Clone + 'a, C: 'a>(
			self,
			func: impl Fn(A, B) -> C + 'a,
			fb: ArcCoyoneda<'a, F, B>,
		) -> ArcCoyoneda<'a, F, C>
		where
			F: Functor + Lift,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone + Send + Sync, {
			ArcCoyoneda::lift(F::lift2(func, self.lower_ref(), fb.lower_ref()))
		}
	}

	// -- Brand --

	impl_kind! {
		impl<F: Kind_cdc7cd43dac7585f + 'static> for ArcCoyonedaBrand<F> {
			type Of<'a, A: 'a>: 'a = ArcCoyoneda<'a, F, A>;
		}
	}

	// -- Brand-level type class instances --
	//
	// ArcCoyonedaBrand implements only Foldable. It does NOT implement Functor,
	// Pointed, Lift, Semiapplicative, or Semimonad, for two independent reasons:
	//
	// 1. Functor: the HKT Functor::map signature lacks Send + Sync bounds on its
	//    closure parameter, so closures passed to map cannot be stored inside
	//    Arc-wrapped layers. This is the same limitation as SendThunkBrand.
	//
	// 2. Pointed, Lift, Semiapplicative, Semimonad: even if Functor were available,
	//    these traits require constructing an ArcCoyoneda, which needs
	//    F::Of<'a, A>: Clone + Send + Sync. This bound cannot be expressed in the
	//    trait method signatures (same blocker as RcCoyonedaBrand; see rc_coyoneda.rs).
	//
	// Use RcCoyonedaBrand when HKT polymorphism is needed, or work with ArcCoyoneda
	// directly via its inherent methods (pure, apply, bind, lift2).

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
			FnBrand: LiftFn + 'a, {
			F::fold_map::<FnBrand, A, M>(func, fa.lower_ref())
		}
	}

	// -- Debug --

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	#[document_parameters("The `ArcCoyoneda` instance.")]
	impl<'a, F, A: 'a> core::fmt::Debug for ArcCoyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Formats the `ArcCoyoneda` as an opaque value.
		///
		/// The inner layers and functions cannot be inspected, so the output
		/// is always `ArcCoyoneda(<opaque>)`.
		#[document_signature]
		///
		#[document_parameters("The formatter.")]
		///
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// assert_eq!(format!("{:?}", coyo), "ArcCoyoneda(<opaque>)");
		/// ```
		fn fmt(
			&self,
			f: &mut core::fmt::Formatter<'_>,
		) -> core::fmt::Result {
			f.write_str("ArcCoyoneda(<opaque>)")
		}
	}

	// -- From<ArcCoyoneda> for Coyoneda --

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying functor.",
		"The type of the values."
	)]
	impl<'a, F, A: 'a> From<ArcCoyoneda<'a, F, A>> for crate::types::Coyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + Functor + 'a,
	{
		/// Convert an [`ArcCoyoneda`] into a [`Coyoneda`](crate::types::Coyoneda)
		/// by lowering to the underlying functor and re-lifting.
		///
		/// This applies all accumulated maps via `F::map` and clones the base value.
		#[document_signature]
		///
		#[document_parameters("The `ArcCoyoneda` to convert.")]
		///
		#[document_returns("A `Coyoneda` containing the lowered value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let arc_coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x + 1);
		/// let coyo: Coyoneda<OptionBrand, i32> = arc_coyo.into();
		/// assert_eq!(coyo.lower(), Some(6));
		/// ```
		fn from(arc: ArcCoyoneda<'a, F, A>) -> Self {
			crate::types::Coyoneda::lift(arc.lower_ref())
		}
	}

	// -- Compile-time assertions for Send/Sync --

	// These assertions verify that the compiler auto-derives Send/Sync
	// correctly for each layer type. If any field type changes in a way
	// that breaks Send/Sync, these assertions will fail at compile time.
	const _: () = {
		fn _assert_send<T: Send>() {}
		fn _assert_sync<T: Sync>() {}

		// ArcCoyonedaBase: Send/Sync via associated type bounds on Kind
		fn _check_base<'a, F: Kind_cdc7cd43dac7585f<Of<'a, A>: Send + Sync> + 'a, A: 'a>() {
			_assert_send::<ArcCoyonedaBase<'a, F, A>>();
			_assert_sync::<ArcCoyonedaBase<'a, F, A>>();
		}

		// ArcCoyonedaMapLayer: unconditionally Send + Sync
		// (both fields are Arc<dyn ... + Send + Sync>)
		fn _check_map_layer<'a, F: Kind_cdc7cd43dac7585f + 'a, B: 'a, A: 'a>() {
			_assert_send::<ArcCoyonedaMapLayer<'a, F, B, A>>();
			_assert_sync::<ArcCoyonedaMapLayer<'a, F, B, A>>();
		}

		// ArcCoyonedaNewLayer: Send/Sync via associated type bounds on Kind
		fn _check_new_layer<
			'a,
			F: Kind_cdc7cd43dac7585f<Of<'a, B>: Send + Sync> + 'a,
			B: 'a,
			A: 'a,
		>() {
			_assert_send::<ArcCoyonedaNewLayer<'a, F, B, A>>();
			_assert_sync::<ArcCoyonedaNewLayer<'a, F, B, A>>();
		}
	};
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

	// -- Property-based tests --

	mod property {
		use {
			crate::{
				brands::*,
				functions::*,
				types::*,
			},
			quickcheck_macros::quickcheck,
		};

		#[quickcheck]
		fn functor_identity_vec(v: Vec<i32>) -> bool {
			let coyo = ArcCoyoneda::<VecBrand, _>::lift(v.clone());
			coyo.map(identity).lower_ref() == v
		}

		#[quickcheck]
		fn functor_identity_option(x: Option<i32>) -> bool {
			let coyo = ArcCoyoneda::<OptionBrand, _>::lift(x);
			coyo.map(identity).lower_ref() == x
		}

		#[quickcheck]
		fn functor_composition_vec(v: Vec<i32>) -> bool {
			let f = |x: i32| x.wrapping_add(1);
			let g = |x: i32| x.wrapping_mul(2);

			let left = ArcCoyoneda::<VecBrand, _>::lift(v.clone()).map(compose(f, g)).lower_ref();
			let right = ArcCoyoneda::<VecBrand, _>::lift(v).map(g).map(f).lower_ref();
			left == right
		}

		#[quickcheck]
		fn functor_composition_option(x: Option<i32>) -> bool {
			let f = |x: i32| x.wrapping_add(1);
			let g = |x: i32| x.wrapping_mul(2);

			let left = ArcCoyoneda::<OptionBrand, _>::lift(x).map(compose(f, g)).lower_ref();
			let right = ArcCoyoneda::<OptionBrand, _>::lift(x).map(g).map(f).lower_ref();
			left == right
		}

		#[quickcheck]
		fn collapse_preserves_value(v: Vec<i32>) -> bool {
			let coyo = ArcCoyoneda::<VecBrand, _>::lift(v)
				.map(|x: i32| x.wrapping_add(1))
				.map(|x: i32| x.wrapping_mul(2));
			let before = coyo.lower_ref();
			let after = coyo.collapse().lower_ref();
			before == after
		}

		#[quickcheck]
		fn foldable_consistency_vec(v: Vec<i32>) -> bool {
			let coyo = ArcCoyoneda::<VecBrand, _>::lift(v.clone()).map(|x: i32| x.wrapping_add(1));
			let via_coyoneda: String = fold_map::<RcFnBrand, ArcCoyonedaBrand<VecBrand>, _, _>(
				|x: i32| x.to_string(),
				coyo,
			);
			let direct: String = fold_map::<RcFnBrand, VecBrand, _, _>(
				|x: i32| x.to_string(),
				v.iter().map(|x| x.wrapping_add(1)).collect::<Vec<_>>(),
			);
			via_coyoneda == direct
		}
	}
}
