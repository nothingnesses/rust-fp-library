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
//! ## Stack safety
//!
//! Each chained [`map`](RcCoyoneda::map) adds a layer of recursion to
//! [`lower_ref`](RcCoyoneda::lower_ref). Deep chains (thousands of maps) can overflow the stack.
//! Three mitigations are available:
//!
//! 1. **`stacker` feature (automatic).** Enable the `stacker` feature flag to use
//!    adaptive stack growth in `lower_ref`. This is transparent and handles arbitrarily
//!    deep chains with near-zero overhead when the stack is sufficient.
//! 2. **[`collapse`](RcCoyoneda::collapse) (manual).** Call `collapse()` periodically
//!    to flatten accumulated layers. Requires `F: Functor`.
//! 3. **[`CoyonedaExplicit`](crate::types::CoyonedaExplicit) with `.boxed()`.** An
//!    alternative that accumulates maps without adding recursion depth.
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
				Lift,
				NaturalTransformation,
				*,
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
		fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor;
	}

	// -- Base layer: wraps F<A> directly (identity mapping, clones on lower) --

	/// Base layer created by [`RcCoyoneda::lift`]. Wraps `F A` with no mapping.
	/// Clones the underlying value on each call to `lower_ref`.
	struct RcCoyonedaBase<'a, F, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
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
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
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
		fn lower_ref(&self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
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

	// -- New layer: wraps F<B> and an Rc-wrapped function (single allocation) --

	/// New layer created by [`RcCoyoneda::new`]. Stores the functor value and an
	/// Rc-wrapped function, implementing `RcCoyonedaLowerRef` directly in a single
	/// Rc allocation.
	struct RcCoyonedaNewLayer<'a, F, B: 'a, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		fb: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		func: Rc<dyn Fn(B) -> A + 'a>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The input type of the stored function.",
		"The output type of the stored function."
	)]
	#[document_parameters("The new layer instance.")]
	impl<'a, F, B: 'a, A: 'a> RcCoyonedaLowerRef<'a, F, A> for RcCoyonedaNewLayer<'a, F, B, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
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
		/// let coyo = RcCoyoneda::<VecBrand, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
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
		pub fn lift(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> Self
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
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
		/// Requires `F: Functor` (for lowering) and `F::Of<'a, A>: Clone`
		/// (for re-lifting).
		#[document_signature]
		///
		#[document_returns("A new `RcCoyoneda` with a single base layer.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).map(|x| x * 2);
		/// let collapsed = coyo.collapse();
		/// assert_eq!(collapsed.lower_ref(), vec![4, 6, 8]);
		/// ```
		pub fn collapse(&self) -> RcCoyoneda<'a, F, A>
		where
			F: Functor,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			RcCoyoneda::lift(self.lower_ref())
		}

		/// Fold the `RcCoyoneda` by lowering and delegating to `F::fold_map`.
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
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
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

		/// Create an `RcCoyoneda` from a function and a functor value.
		///
		/// This is more efficient than `lift(fb).map(f)` because it creates
		/// a single layer instead of two.
		#[document_signature]
		///
		#[document_type_parameters("The input type of the function.")]
		///
		#[document_parameters("The function to defer.", "The functor value.")]
		///
		#[document_returns("An `RcCoyoneda` wrapping the value with the deferred function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<VecBrand, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(coyo.lower_ref(), vec![2, 4, 6]);
		/// ```
		pub fn new<B: 'a>(
			f: impl Fn(B) -> A + 'a,
			fb: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Self
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			RcCoyoneda(Rc::new(RcCoyonedaNewLayer {
				fb,
				func: Rc::new(f),
			}))
		}

		/// Apply a natural transformation to change the underlying functor.
		///
		/// Lowers to `F A` (applying all accumulated maps), transforms via the
		/// natural transformation, then re-lifts into `RcCoyoneda G A`.
		///
		/// Requires `F: Functor` for lowering.
		#[document_signature]
		///
		#[document_type_parameters("The target functor brand.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("A new `RcCoyoneda` over the target functor `G`.")]
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
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![10, 20, 30]);
		/// let hoisted = coyo.hoist(VecToOption);
		/// assert_eq!(hoisted.lower_ref(), Some(10));
		/// ```
		pub fn hoist<G: Kind_cdc7cd43dac7585f + 'a>(
			self,
			nat: impl NaturalTransformation<F, G>,
		) -> RcCoyoneda<'a, G, A>
		where
			F: Functor,
			Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			RcCoyoneda::lift(nat.transform(self.lower_ref()))
		}

		/// Wrap a value in the `RcCoyoneda` context by delegating to `F::pure`.
		///
		/// Requires `F::Of<'a, A>: Clone` for the base layer.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `RcCoyoneda` containing the pure value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<OptionBrand, i32>::pure(42);
		/// assert_eq!(coyo.lower_ref(), Some(42));
		/// ```
		pub fn pure(a: A) -> Self
		where
			F: Pointed,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			RcCoyoneda::lift(F::pure(a))
		}

		/// Chain `RcCoyoneda` computations by lowering, binding, and re-lifting.
		///
		/// This is a fusion barrier: all accumulated maps are applied before binding.
		///
		/// Requires `F: Functor` (for lowering) and `F: Semimonad` (for binding).
		#[document_signature]
		///
		#[document_type_parameters("The output type of the bound computation.")]
		///
		#[document_parameters(
			"The function to apply to the inner value, returning a new `RcCoyoneda`."
		)]
		///
		#[document_returns("A new `RcCoyoneda` containing the bound result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(5));
		/// let result = coyo.bind(|x| RcCoyoneda::<OptionBrand, _>::lift(Some(x * 2)));
		/// assert_eq!(result.lower_ref(), Some(10));
		/// ```
		pub fn bind<B: 'a>(
			self,
			func: impl Fn(A) -> RcCoyoneda<'a, F, B> + 'a,
		) -> RcCoyoneda<'a, F, B>
		where
			F: Functor + Semimonad,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			RcCoyoneda::lift(F::bind(self.lower_ref(), move |a| func(a).lower_ref()))
		}

		/// Apply a function inside an `RcCoyoneda` to a value inside another.
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
			"The `RcCoyoneda` containing the function(s).",
			"The `RcCoyoneda` containing the value(s)."
		)]
		///
		#[document_returns("A new `RcCoyoneda` containing the applied result(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let ff =
		/// 	RcCoyoneda::<OptionBrand, _>::lift(Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2)));
		/// let fa = RcCoyoneda::<OptionBrand, _>::lift(Some(5));
		/// let result = RcCoyoneda::<OptionBrand, i32>::apply::<RcFnBrand, _, _>(ff, fa);
		/// assert_eq!(result.lower_ref(), Some(10));
		/// ```
		pub fn apply<FnBrand: LiftFn + 'a, B: Clone + 'a, C: 'a>(
			ff: RcCoyoneda<'a, F, <FnBrand as CloneFn>::Of<'a, B, C>>,
			fa: RcCoyoneda<'a, F, B>,
		) -> RcCoyoneda<'a, F, C>
		where
			F: Functor + Semiapplicative,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone, {
			RcCoyoneda::lift(F::apply::<FnBrand, B, C>(ff.lower_ref(), fa.lower_ref()))
		}

		/// Lift a binary function into the `RcCoyoneda` context.
		///
		/// Both arguments are lowered and delegated to `F::lift2`, then re-lifted.
		/// This is a fusion barrier.
		#[document_signature]
		///
		#[document_type_parameters("The type of the second value.", "The type of the result.")]
		///
		#[document_parameters("The binary function to apply.", "The second `RcCoyoneda` value.")]
		///
		#[document_returns("An `RcCoyoneda` containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let a = RcCoyoneda::<OptionBrand, _>::lift(Some(3));
		/// let b = RcCoyoneda::<OptionBrand, _>::lift(Some(4));
		/// let result = a.lift2(|x, y| x + y, b);
		/// assert_eq!(result.lower_ref(), Some(7));
		/// ```
		pub fn lift2<B: Clone + 'a, C: 'a>(
			self,
			func: impl Fn(A, B) -> C + 'a,
			fb: RcCoyoneda<'a, F, B>,
		) -> RcCoyoneda<'a, F, C>
		where
			F: Functor + Lift,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone, {
			RcCoyoneda::lift(F::lift2(func, self.lower_ref(), fb.lower_ref()))
		}
	}

	// -- Brand --

	impl_kind! {
		impl<F: Kind_cdc7cd43dac7585f + 'static> for RcCoyonedaBrand<F> {
			type Of<'a, A: 'a>: 'a = RcCoyoneda<'a, F, A>;
		}
	}

	// -- Brand-level type class instances --
	//
	// RcCoyonedaBrand implements Functor and Foldable but NOT Pointed, Lift,
	// Semiapplicative, or Semimonad. The blocker is a Clone bound that cannot
	// be expressed in the trait method signatures:
	//
	// - RcCoyoneda wraps Rc<dyn RcCoyonedaLowerRef>. Constructing this requires
	//   F::Of<'a, A>: Clone because RcCoyonedaBase's RcCoyonedaLowerRef impl has
	//   that bound, needed to coerce the struct to the trait object.
	// - CoyonedaBrand avoids this because Coyoneda::lift has no Clone requirement;
	//   CoyonedaBase::lower consumes self: Box<Self> (moving, not cloning).
	// - Rust does not allow adding extra where clauses to trait method impls beyond
	//   what the trait definition specifies, so the Clone bound cannot be expressed.
	//
	// Inherent methods (pure, apply, bind, lift2) are provided instead, with the
	// Clone bound specified directly on each method.

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
		/// let mapped = map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(|x| x * 10, coyo);
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
		/// let result = fold_map_explicit::<RcFnBrand, RcCoyonedaBrand<VecBrand>, _, _, _, _>(
		/// 	|x: i32| x.to_string(),
		/// 	coyo,
		/// );
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
	#[document_parameters("The `RcCoyoneda` instance.")]
	impl<'a, F, A: 'a> core::fmt::Debug for RcCoyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Formats the `RcCoyoneda` as an opaque value.
		///
		/// The inner layers and functions cannot be inspected, so the output
		/// is always `RcCoyoneda(<opaque>)`.
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
		/// let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// assert_eq!(format!("{:?}", coyo), "RcCoyoneda(<opaque>)");
		/// ```
		fn fmt(
			&self,
			f: &mut core::fmt::Formatter<'_>,
		) -> core::fmt::Result {
			f.write_str("RcCoyoneda(<opaque>)")
		}
	}

	// -- From<RcCoyoneda> for Coyoneda --

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying functor.",
		"The type of the values."
	)]
	impl<'a, F, A: 'a> From<RcCoyoneda<'a, F, A>> for crate::types::Coyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + Functor + 'a,
	{
		/// Convert an [`RcCoyoneda`] into a [`Coyoneda`](crate::types::Coyoneda)
		/// by lowering to the underlying functor and re-lifting.
		///
		/// This applies all accumulated maps via `F::map` and clones the base value.
		#[document_signature]
		///
		#[document_parameters("The `RcCoyoneda` to convert.")]
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
		/// let rc_coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x + 1);
		/// let coyo: Coyoneda<OptionBrand, i32> = rc_coyo.into();
		/// assert_eq!(coyo.lower(), Some(6));
		/// ```
		fn from(rc: RcCoyoneda<'a, F, A>) -> Self {
			crate::types::Coyoneda::lift(rc.lower_ref())
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
		let result =
			map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(identity, coyo).lower_ref();
		assert_eq!(result, vec![1, 2, 3]);
	}

	#[test]
	fn functor_composition_law() {
		let f = |x: i32| x + 1;
		let g = |x: i32| x * 2;

		let coyo1 = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let left =
			map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(compose(f, g), coyo1).lower_ref();

		let coyo2 = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let right = map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(
			f,
			map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(g, coyo2),
		)
		.lower_ref();

		assert_eq!(left, right);
	}

	// -- Foldable tests --

	#[test]
	fn fold_map_on_mapped() {
		let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let result = fold_map_explicit::<RcFnBrand, RcCoyonedaBrand<VecBrand>, _, _, _, _>(
			|x: i32| x.to_string(),
			coyo,
		);
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
			let coyo = RcCoyoneda::<VecBrand, _>::lift(v.clone());
			map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(identity, coyo).lower_ref() == v
		}

		#[quickcheck]
		fn functor_identity_option(x: Option<i32>) -> bool {
			let coyo = RcCoyoneda::<OptionBrand, _>::lift(x);
			map_explicit::<RcCoyonedaBrand<OptionBrand>, _, _, _, _>(identity, coyo).lower_ref()
				== x
		}

		#[quickcheck]
		fn functor_composition_vec(v: Vec<i32>) -> bool {
			let f = |x: i32| x.wrapping_add(1);
			let g = |x: i32| x.wrapping_mul(2);

			let left = map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(
				compose(f, g),
				RcCoyoneda::<VecBrand, _>::lift(v.clone()),
			)
			.lower_ref();

			let right = map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(
				f,
				map_explicit::<RcCoyonedaBrand<VecBrand>, _, _, _, _>(
					g,
					RcCoyoneda::<VecBrand, _>::lift(v),
				),
			)
			.lower_ref();

			left == right
		}

		#[quickcheck]
		fn functor_composition_option(x: Option<i32>) -> bool {
			let f = |x: i32| x.wrapping_add(1);
			let g = |x: i32| x.wrapping_mul(2);

			let left = map_explicit::<RcCoyonedaBrand<OptionBrand>, _, _, _, _>(
				compose(f, g),
				RcCoyoneda::<OptionBrand, _>::lift(x),
			)
			.lower_ref();

			let right = map_explicit::<RcCoyonedaBrand<OptionBrand>, _, _, _, _>(
				f,
				map_explicit::<RcCoyonedaBrand<OptionBrand>, _, _, _, _>(
					g,
					RcCoyoneda::<OptionBrand, _>::lift(x),
				),
			)
			.lower_ref();

			left == right
		}

		#[quickcheck]
		fn foldable_consistency_vec(v: Vec<i32>) -> bool {
			let coyo = RcCoyoneda::<VecBrand, _>::lift(v.clone()).map(|x: i32| x.wrapping_add(1));
			let via_coyoneda: String =
				fold_map_explicit::<RcFnBrand, RcCoyonedaBrand<VecBrand>, _, _, _, _>(
					|x: i32| x.to_string(),
					coyo,
				);
			let direct: String = fold_map_explicit::<RcFnBrand, VecBrand, _, _, _, _>(
				|x: i32| x.to_string(),
				v.iter().map(|x| x.wrapping_add(1)).collect::<Vec<_>>(),
			);
			via_coyoneda == direct
		}

		#[quickcheck]
		fn collapse_preserves_value(v: Vec<i32>) -> bool {
			let coyo = RcCoyoneda::<VecBrand, _>::lift(v)
				.map(|x: i32| x.wrapping_add(1))
				.map(|x: i32| x.wrapping_mul(2));
			let before = coyo.lower_ref();
			let after = coyo.collapse().lower_ref();
			before == after
		}
	}
}
