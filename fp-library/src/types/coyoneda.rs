//! The free functor, providing a [`Functor`](crate::classes::Functor) instance for any
//! type constructor with the appropriate [`Kind`](crate::kinds) signature.
//!
//! `Coyoneda F` is a `Functor` even when `F` itself is not. The `Functor` bound on `F`
//! is only required when calling [`lower`](Coyoneda::lower) to extract the underlying
//! value. This is the defining property of the free functor construction.
//!
//! ## Performance characteristics
//!
//! Each call to [`map`](Coyoneda::map) wraps the previous value in a new layer that
//! stores the mapping function inline (one heap allocation for the layer itself; the
//! function is not separately boxed). At [`lower`](Coyoneda::lower) time, each layer
//! applies its function via `F::map`. For k chained maps, `lower` makes k calls to
//! `F::map`.
//!
//! This is a consequence of Rust's dyn-compatibility rules: composing functions across
//! an existential boundary requires generic methods on trait objects, which Rust does not
//! support. For true single-pass fusion on eager brands like
//! [`VecBrand`](crate::brands::VecBrand), compose functions before mapping:
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Single traversal: compose first, then map once.
//! let result = map_explicit::<VecBrand, _, _, _, _>(
//! 	compose(|x: i32| x.to_string(), compose(|x| x * 2, |x: i32| x + 1)),
//! 	vec![1, 2, 3],
//! );
//! assert_eq!(result, vec!["4", "6", "8"]);
//! ```
//!
//! For single-pass fusion that applies one `F::map` regardless of chain depth,
//! see [`CoyonedaExplicit`](crate::types::CoyonedaExplicit).
//!
//! ## Stack safety
//!
//! Each chained [`map`](Coyoneda::map) adds a layer of recursion to
//! [`lower`](Coyoneda::lower). Deep chains (thousands of maps) can overflow the stack.
//! Three mitigations are available:
//!
//! 1. **`stacker` feature (automatic).** Enable the `stacker` feature flag to use
//!    adaptive stack growth in `lower`. This is transparent and handles arbitrarily
//!    deep chains with near-zero overhead when the stack is sufficient.
//! 2. **[`collapse`](Coyoneda::collapse) (manual).** Call `collapse()` periodically
//!    to flatten accumulated layers. Requires `F: Functor`.
//! 3. **[`CoyonedaExplicit`](crate::types::CoyonedaExplicit) with `.boxed()`.** An
//!    alternative type that composes functions at the type level, producing a single
//!    `F::map` call at lower time regardless of chain depth.
//!
//! ## Limitations
//!
//! All limitations stem from a single root cause: Rust trait objects cannot have methods
//! with generic type parameters (the vtable is fixed at compile time). This prevents
//! "opening" the existential type `B` hidden inside each layer.
//!
//! - **No map fusion.** PureScript's Coyoneda composes `f <<< k` eagerly so that `lower`
//!   calls `F::map` exactly once regardless of how many maps were chained. This Rust
//!   implementation cannot compose functions across the trait-object boundary because the
//!   required `map_inner<C>` method is generic over `C`. Each layer calls `F::map`
//!   independently, so k chained maps produce k calls to `F::map` at `lower` time, the
//!   same cost as chaining `F::map` directly.
//!
//! - **[`Foldable`](crate::classes::Foldable) requires `F: Functor`.** PureScript's
//!   `Foldable` for `Coyoneda` only needs `Foldable f` because `unCoyoneda` opens the
//!   existential to compose the fold function with the accumulated mapping function,
//!   folding the original `F B` in a single pass. This implementation cannot add a
//!   `fold_map_inner` method to the inner trait because it is generic over the monoid type
//!   `M`, breaking dyn-compatibility. Instead, it lowers first (requiring `F: Functor`),
//!   then folds.
//!
//! - **[`hoist`](Coyoneda::hoist) requires `F: Functor`.** PureScript's `hoistCoyoneda`
//!   applies the natural transformation directly to the hidden `F B` via `unCoyoneda`. A
//!   `hoist_inner<G>` method would be generic over the target brand `G`, so this
//!   implementation lowers first, transforms, then re-lifts.
//!
//! - **No `unCoyoneda`.** PureScript's rank-2 eliminator
//!   `(forall b. (b -> a) -> f b -> r) -> Coyoneda f a -> r` has no Rust equivalent
//!   because closures cannot be polymorphic over type parameters. Operation-specific
//!   methods on the inner trait are used instead.
//!
//! - **Not `Clone`.** The inner trait object `Box<dyn CoyonedaInner>` is not `Clone`.
//!   This prevents implementing [`Traversable`](crate::classes::Traversable) (which
//!   requires `Self::Of<'a, B>: Clone`) and
//!   [`Semiapplicative`](crate::classes::Semiapplicative). An `Rc`/`Arc`-wrapped variant
//!   would address this.
//!
//! - **Missing type class instances.** PureScript provides `Traversable`, `Extend`,
//!   `Comonad`, `Eq`, `Ord`, and others. This implementation currently provides
//!   [`Functor`](crate::classes::Functor), [`Pointed`](crate::classes::Pointed),
//!   [`Foldable`](crate::classes::Foldable), [`Lift`](crate::classes::Lift),
//!   [`Semiapplicative`](crate::classes::Semiapplicative),
//!   [`Semimonad`](crate::classes::Semimonad), and [`Monad`](crate::classes::Monad)
//!   (via blanket impl).
//!
//! ## Comparison with PureScript
//!
//! This implementation is based on PureScript's
//! [`Data.Coyoneda`](https://github.com/purescript/purescript-free/blob/master/src/Data/Coyoneda.purs).
//! PureScript uses `Exists` for existential quantification and composes functions eagerly
//! (single `F::map` at `lower` time regardless of how many maps were chained). Rust uses
//! layered trait objects because dyn-compatibility prevents the generic `map_inner<C>`
//! method needed for eager composition.
//!
//! The PureScript API also provides `unCoyoneda` (a rank-2 eliminator), `coyoneda`
//! (a general constructor), and `hoistCoyoneda` (natural transformation). This
//! implementation provides [`Coyoneda::new`] (equivalent to `coyoneda`) and
//! [`Coyoneda::hoist`] (equivalent to `hoistCoyoneda`, but requires `F: Functor`).
//! A direct equivalent of `unCoyoneda` is not possible in Rust without rank-2 types.
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
//! let v = vec![1, 2, 3];
//!
//! // Lift into Coyoneda, chain maps, then lower.
//! let result = Coyoneda::<VecBrand, _>::lift(v)
//! 	.map(|x| x + 1)
//! 	.map(|x| x * 2)
//! 	.map(|x| x.to_string())
//! 	.lower();
//!
//! assert_eq!(result, vec!["4", "6", "8"]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::CoyonedaBrand,
			classes::*,
			impl_kind,
			kinds::*,
			types::CoyonedaExplicit,
		},
		fp_macros::*,
	};

	// -- Inner trait (existential witness) --

	/// Trait for lowering a `Coyoneda` value back to its underlying functor.
	///
	/// Each implementor stores a value of type `F B` for some hidden type `B`,
	/// along with any accumulated mapping functions needed to produce `F A`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The output type of the accumulated mapping function."
	)]
	#[document_parameters("The boxed trait object to consume.")]
	trait CoyonedaInner<'a, F, A: 'a>: 'a
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
		/// let coyo = Coyoneda::<OptionBrand, _>::lift(Some(42));
		/// assert_eq!(coyo.lower(), Some(42));
		/// ```
		fn lower(self: Box<Self>) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor;
	}

	// -- Base layer: wraps F<A> directly (identity mapping) --

	/// Base layer created by [`Coyoneda::lift`]. Wraps `F A` with no mapping.
	struct CoyonedaBase<'a, F, A: 'a>
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
	impl<'a, F, A: 'a> CoyonedaInner<'a, F, A> for CoyonedaBase<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Returns the wrapped value directly without calling `F::map`.
		#[document_signature]
		///
		#[document_returns("The underlying functor value, unchanged.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// assert_eq!(coyo.lower(), vec![1, 2, 3]);
		/// ```
		fn lower(self: Box<Self>) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor, {
			self.fa
		}
	}

	// -- Map layer: wraps an inner Coyoneda and adds a mapping function --

	/// Map layer created by [`Coyoneda::map`]. Stores the inner value and a function
	/// to apply on top of it at [`lower`](Coyoneda::lower) time.
	///
	/// The function type `Func` is stored inline rather than boxed, eliminating one
	/// heap allocation per [`map`](Coyoneda::map) call. The `Func` parameter is erased
	/// by the outer `Box<dyn CoyonedaInner>` and does not appear in the public API.
	struct CoyonedaMapLayer<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		inner: Box<dyn CoyonedaInner<'a, F, B> + 'a>,
		func: Func,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The input type of this layer's mapping function.",
		"The output type of this layer's mapping function.",
		"The type of this layer's mapping function."
	)]
	#[document_parameters("The map layer instance.")]
	impl<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> CoyonedaInner<'a, F, A>
		for CoyonedaMapLayer<'a, F, B, A, Func>
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
		/// let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// assert_eq!(coyo.lower(), vec![2, 3, 4]);
		/// ```
		fn lower(self: Box<Self>) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor, {
			#[cfg(feature = "stacker")]
			{
				stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
					let lowered = self.inner.lower();
					F::map(self.func, lowered)
				})
			}
			#[cfg(not(feature = "stacker"))]
			{
				let lowered = self.inner.lower();
				F::map(self.func, lowered)
			}
		}
	}

	// -- New layer: wraps F<B> and a function B -> A directly (used by Coyoneda::new) --

	/// Layer created by [`Coyoneda::new`]. Stores the functor value and mapping
	/// function directly, avoiding the extra box that wrapping a [`CoyonedaBase`]
	/// inside a [`CoyonedaMapLayer`] would require.
	///
	/// The function type `Func` is stored inline rather than boxed, eliminating one
	/// heap allocation per [`new`](Coyoneda::new) call. The `Func` parameter is erased
	/// by the outer `Box<dyn CoyonedaInner>` and does not appear in the public API.
	struct CoyonedaNewLayer<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		fb: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		func: Func,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the underlying functor.",
		"The output type of the mapping function.",
		"The type of the mapping function."
	)]
	#[document_parameters("The new layer instance.")]
	impl<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> CoyonedaInner<'a, F, A>
		for CoyonedaNewLayer<'a, F, B, A, Func>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Applies the stored function to the stored functor value via `F::map`.
		#[document_signature]
		///
		#[document_returns("The underlying functor value with the function applied.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<VecBrand, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(coyo.lower(), vec![2, 4, 6]);
		/// ```
		fn lower(self: Box<Self>) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor, {
			F::map(self.func, self.fb)
		}
	}

	// -- Outer type --

	/// The free functor over a type constructor `F`.
	///
	/// `Coyoneda` provides a [`Functor`] instance for any type constructor `F` with the
	/// appropriate [`Kind`](crate::kinds) signature, even if `F` itself does not implement
	/// [`Functor`]. The `Functor` constraint is only needed when calling [`lower`](Coyoneda::lower).
	///
	/// This type is not `Clone`, `Send`, or `Sync`. It wraps a `Box<dyn CoyonedaInner>`,
	/// so each value is single-owner and consumed by [`lower`](Coyoneda::lower).
	///
	/// See the [module documentation](crate::types::coyoneda) for limitations and performance notes.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	pub struct Coyoneda<'a, F, A: 'a>(Box<dyn CoyonedaInner<'a, F, A> + 'a>)
	where
		F: Kind_cdc7cd43dac7585f + 'a;

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	#[document_parameters("The `Coyoneda` instance.")]
	impl<'a, F, A: 'a> Coyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Construct a `Coyoneda` from a function and a functor value.
		///
		/// This is the general constructor corresponding to PureScript's `coyoneda`.
		/// It stores `fb` alongside `f` as a single deferred mapping step.
		/// [`lift`](Coyoneda::lift) is equivalent to `new(|a| a, fa)`.
		#[document_signature]
		///
		#[document_type_parameters("The type of the values in the underlying functor.")]
		///
		#[document_parameters("The function to defer.", "The functor value.")]
		///
		#[document_returns("A `Coyoneda` wrapping the value with the deferred function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<VecBrand, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(coyo.lower(), vec![2, 4, 6]);
		/// ```
		pub fn new<B: 'a>(
			f: impl Fn(B) -> A + 'a,
			fb: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Self {
			Coyoneda(Box::new(CoyonedaNewLayer {
				fb,
				func: f,
			}))
		}

		/// Lift a value of `F A` into `Coyoneda F A`.
		///
		/// This wraps the value directly with no mapping. O(1).
		/// Equivalent to `Coyoneda::new(|a| a, fa)`.
		#[document_signature]
		///
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns("A `Coyoneda` wrapping the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<OptionBrand, _>::lift(Some(42));
		/// assert_eq!(coyo.lower(), Some(42));
		/// ```
		pub fn lift(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> Self {
			Coyoneda(Box::new(CoyonedaBase {
				fa,
			}))
		}

		/// Lower the `Coyoneda` back to the underlying functor `F`.
		///
		/// Applies accumulated mapping functions via `F::map`. Requires `F: Functor`.
		/// For k chained maps, this makes k calls to `F::map` (one per layer).
		/// See the [module-level performance notes](crate::types::coyoneda#performance-characteristics) for details.
		#[document_signature]
		///
		#[document_returns("The underlying functor value with all accumulated functions applied.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).lower();
		///
		/// assert_eq!(result, vec![2, 3, 4]);
		/// ```
		pub fn lower(self) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			F: Functor, {
			self.0.lower()
		}

		/// Collapse accumulated mapping layers by lowering and re-lifting.
		///
		/// This applies all accumulated `map` layers via [`lower`](Coyoneda::lower),
		/// producing a concrete `F A`, then wraps the result back in a single-layer
		/// `Coyoneda`. Useful for bounding recursion depth in long chains:
		/// insert `collapse()` periodically to prevent stack overflow without
		/// switching to [`CoyonedaExplicit`](crate::types::CoyonedaExplicit).
		///
		/// Requires `F: Functor`. Cost: one full `lower` pass.
		#[document_signature]
		///
		#[document_returns("A fresh `Coyoneda` with a single base layer.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let mut coyo = Coyoneda::<VecBrand, _>::lift(vec![0i64]);
		/// for i in 0 .. 50 {
		/// 	coyo = coyo.map(|x| x + 1);
		/// 	if i % 25 == 24 {
		/// 		coyo = coyo.collapse();
		/// 	}
		/// }
		/// assert_eq!(coyo.lower(), vec![50i64]);
		/// ```
		pub fn collapse(self) -> Self
		where
			F: Functor, {
			Coyoneda::lift(self.lower())
		}

		/// Map a function over the `Coyoneda` value.
		///
		/// This wraps the current value in a new layer that stores `f`. The function
		/// is applied when [`lower`](Coyoneda::lower) is called. O(1).
		#[document_signature]
		///
		#[document_type_parameters("The new output type after applying the function.")]
		///
		#[document_parameters("The function to apply.")]
		///
		#[document_returns("A new `Coyoneda` with the function stored for deferred application.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x * 2).map(|x| x + 1);
		///
		/// assert_eq!(coyo.lower(), Some(11));
		/// ```
		pub fn map<B: 'a>(
			self,
			f: impl Fn(A) -> B + 'a,
		) -> Coyoneda<'a, F, B> {
			Coyoneda(Box::new(CoyonedaMapLayer {
				inner: self.0,
				func: f,
			}))
		}

		/// Apply a natural transformation to the underlying functor.
		///
		/// Transforms a `Coyoneda<F, A>` into a `Coyoneda<G, A>` by lowering to `F`,
		/// applying the natural transformation `F ~> G`, then lifting back into
		/// `Coyoneda<G, A>`. Requires `F: Functor` (for lowering).
		///
		/// Corresponds to PureScript's `hoistCoyoneda`. Note: the PureScript version
		/// does not require `F: Functor` because it can open the existential directly
		/// via `unCoyoneda`. In Rust, dyn-compatibility prevents this.
		#[document_signature]
		///
		#[document_type_parameters("The brand of the target functor.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("A new `Coyoneda` over the target functor `G`.")]
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
		/// let coyo = Coyoneda::<VecBrand, _>::lift(vec![10, 20, 30]);
		/// let hoisted: Coyoneda<OptionBrand, i32> = coyo.hoist(VecToOption);
		/// assert_eq!(hoisted.lower(), Some(10));
		/// ```
		pub fn hoist<G: Kind_cdc7cd43dac7585f + 'a>(
			self,
			nat: impl NaturalTransformation<F, G>,
		) -> Coyoneda<'a, G, A>
		where
			F: Functor, {
			Coyoneda::lift(nat.transform(self.lower()))
		}
	}

	// -- Brand --

	impl_kind! {
		impl<F: Kind_cdc7cd43dac7585f + 'static> for CoyonedaBrand<F> {
			type Of<'a, A: 'a>: 'a = Coyoneda<'a, F, A>;
		}
	}

	// -- Functor implementation --

	#[document_type_parameters("The brand of the underlying type constructor.")]
	impl<F: Kind_cdc7cd43dac7585f + 'static> Functor for CoyonedaBrand<F> {
		/// Maps a function over the `Coyoneda` value by adding a new mapping layer.
		///
		/// Does not require `F: Functor`. The function is stored and applied at
		/// [`lower`](Coyoneda::lower) time.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the current output.",
			"The type of the new output."
		)]
		///
		#[document_parameters("The function to apply.", "The `Coyoneda` value.")]
		///
		#[document_returns("A new `Coyoneda` with the function stored for deferred application.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// let mapped = map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(|x| x * 10, coyo);
		/// assert_eq!(mapped.lower(), vec![10, 20, 30]);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	// -- Pointed implementation --

	#[document_type_parameters("The brand of the underlying pointed functor.")]
	impl<F: Pointed + 'static> Pointed for CoyonedaBrand<F> {
		/// Wraps a value in a `Coyoneda` context by delegating to `F::pure` and lifting.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `Coyoneda` containing the pure value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let coyo: Coyoneda<OptionBrand, i32> = pure::<CoyonedaBrand<OptionBrand>, _>(42);
		/// assert_eq!(coyo.lower(), Some(42));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Coyoneda::lift(F::pure(a))
		}
	}

	// -- Foldable implementation --

	#[document_type_parameters("The brand of the underlying foldable functor.")]
	impl<F: Functor + Foldable + 'static> Foldable for CoyonedaBrand<F> {
		/// Folds the `Coyoneda` by lowering to the underlying functor and delegating.
		///
		/// This first applies all accumulated mapping functions via [`lower`](Coyoneda::lower),
		/// then folds the resulting `F A` using `F`'s [`Foldable`] instance.
		///
		/// Note: unlike PureScript's `Foldable` for `Coyoneda`, this requires `F: Functor`
		/// because the layered trait-object encoding does not support composing fold functions
		/// through the existential boundary (doing so would require generic methods on the
		/// inner trait, which are not dyn-compatible).
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
			"The `Coyoneda` structure to fold."
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
		/// let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		///
		/// let result =
		/// 	fold_map::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _, _, _>(|x: i32| x.to_string(), coyo);
		/// assert_eq!(result, "102030".to_string());
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: LiftFn + 'a, {
			F::fold_map::<FnBrand, A, M>(func, fa.lower())
		}
	}

	// -- Lift implementation --

	#[document_type_parameters("The brand of the underlying type constructor.")]
	impl<F: Functor + Lift + 'static> Lift for CoyonedaBrand<F> {
		/// Lifts a binary function into the `Coyoneda` context by lowering both
		/// arguments and delegating to `F::lift2`.
		///
		/// Requires `F: Functor` (for lowering) and `F: Lift`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first `Coyoneda` value.",
			"The second `Coyoneda` value."
		)]
		///
		#[document_returns("A `Coyoneda` containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let a = Coyoneda::<OptionBrand, _>::lift(Some(3));
		/// let b = Coyoneda::<OptionBrand, _>::lift(Some(4));
		/// let result = lift2::<CoyonedaBrand<OptionBrand>, _, _, _, _, _, _>(|x, y| x + y, a, b);
		/// assert_eq!(result.lower(), Some(7));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			Coyoneda::lift(F::lift2(func, fa.lower(), fb.lower()))
		}
	}

	// -- ApplyFirst / ApplySecond --

	#[document_type_parameters("The brand of the underlying type constructor.")]
	impl<F: Functor + Lift + 'static> ApplyFirst for CoyonedaBrand<F> {}
	#[document_type_parameters("The brand of the underlying type constructor.")]
	impl<F: Functor + Lift + 'static> ApplySecond for CoyonedaBrand<F> {}

	// -- Semiapplicative implementation --

	#[document_type_parameters("The brand of the underlying type constructor.")]
	impl<F: Functor + Semiapplicative + 'static> Semiapplicative for CoyonedaBrand<F> {
		/// Applies a `Coyoneda`-wrapped function to a `Coyoneda`-wrapped value by
		/// lowering both and delegating to `F::apply`.
		///
		/// Requires `F: Functor` (for lowering) and `F: Semiapplicative`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The `Coyoneda` containing the function(s).",
			"The `Coyoneda` containing the value(s)."
		)]
		///
		#[document_returns("A `Coyoneda` containing the result(s) of applying the function(s).")]
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
		/// let ff = Coyoneda::<OptionBrand, _>::lift(Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2)));
		/// let fa = Coyoneda::<OptionBrand, _>::lift(Some(5));
		/// let result = apply::<RcFnBrand, CoyonedaBrand<OptionBrand>, _, _>(ff, fa);
		/// assert_eq!(result.lower(), Some(10));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Coyoneda::lift(F::apply::<FnBrand, A, B>(ff.lower(), fa.lower()))
		}
	}

	// -- Semimonad implementation --

	#[document_type_parameters("The brand of the underlying type constructor.")]
	impl<F: Functor + Semimonad + 'static> Semimonad for CoyonedaBrand<F> {
		/// Chains `Coyoneda` computations by lowering to the underlying functor,
		/// binding via `F::bind`, then re-lifting the result.
		///
		/// Requires `F: Functor` (for lowering) and `F: Semimonad` (for binding).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value in the input `Coyoneda`.",
			"The type of the value in the output `Coyoneda`."
		)]
		///
		#[document_parameters(
			"The input `Coyoneda` value.",
			"The function to apply to the inner value, returning a new `Coyoneda`."
		)]
		///
		#[document_returns("A new `Coyoneda` containing the bound result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<OptionBrand, _>::lift(Some(5));
		/// let result = bind::<CoyonedaBrand<OptionBrand>, _, _, _, _>(coyo, |x| {
		/// 	Coyoneda::<OptionBrand, _>::lift(Some(x * 2))
		/// });
		/// assert_eq!(result.lower(), Some(10));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Coyoneda::lift(F::bind(ma.lower(), move |a| func(a).lower()))
		}
	}

	// -- From<Coyoneda> for CoyonedaExplicit --

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying foldable functor.",
		"The type of the values."
	)]
	impl<'a, F, A: 'a> From<Coyoneda<'a, F, A>> for CoyonedaExplicit<'a, F, A, A, fn(A) -> A>
	where
		F: Kind_cdc7cd43dac7585f + Functor + 'a,
	{
		/// Convert a [`Coyoneda`] into a [`CoyonedaExplicit`] by lowering to the
		/// underlying functor and re-lifting.
		///
		/// This calls [`lower()`](Coyoneda::lower), which applies all accumulated
		/// mapping layers via `F::map`. For eager containers like `Vec`, this
		/// allocates and traverses the full container. The cost is proportional to
		/// the number of chained maps and the container size.
		#[document_signature]
		///
		#[document_parameters("The `Coyoneda` to convert.")]
		///
		#[document_returns(
			"A `CoyonedaExplicit` in identity position (B = A) wrapping the lowered value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// let explicit: CoyonedaExplicit<VecBrand, i32, i32, fn(i32) -> i32> = coyo.into();
		/// assert_eq!(explicit.lower(), vec![2, 3, 4]);
		/// ```
		fn from(coyo: Coyoneda<'a, F, A>) -> Self {
			CoyonedaExplicit::lift(coyo.lower())
		}
	}

	// -- Debug --

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The current output type."
	)]
	#[document_parameters("The `Coyoneda` instance.")]
	impl<'a, F, A: 'a> core::fmt::Debug for Coyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Formats the `Coyoneda` as an opaque value.
		///
		/// The inner layers and functions cannot be inspected, so the output
		/// is always `Coyoneda(<opaque>)`.
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
		/// let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// assert_eq!(format!("{:?}", coyo), "Coyoneda(<opaque>)");
		/// ```
		fn fmt(
			&self,
			f: &mut core::fmt::Formatter<'_>,
		) -> core::fmt::Result {
			f.write_str("Coyoneda(<opaque>)")
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use crate::{
		brands::*,
		classes::*,
		functions::*,
		types::*,
	};

	#[test]
	fn lift_lower_identity_option() {
		let coyo = Coyoneda::<OptionBrand, _>::lift(Some(42));
		assert_eq!(coyo.lower(), Some(42));
	}

	#[test]
	fn lift_lower_identity_none() {
		let coyo = Coyoneda::<OptionBrand, i32>::lift(None);
		assert_eq!(coyo.lower(), None);
	}

	#[test]
	fn lift_lower_identity_vec() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		assert_eq!(coyo.lower(), vec![1, 2, 3]);
	}

	#[test]
	fn new_constructor() {
		let coyo = Coyoneda::<VecBrand, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		assert_eq!(coyo.lower(), vec![2, 4, 6]);
	}

	#[test]
	fn new_is_equivalent_to_lift_then_map() {
		let f = |x: i32| x.to_string();
		let v = vec![1, 2, 3];

		let via_new = Coyoneda::<VecBrand, _>::new(f, v.clone()).lower();
		let via_lift_map = Coyoneda::<VecBrand, _>::lift(v).map(f).lower();

		assert_eq!(via_new, via_lift_map);
	}

	#[test]
	fn single_map_option() {
		let result = Coyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x * 2).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn chained_maps_vec() {
		let result = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3])
			.map(|x| x + 1)
			.map(|x| x * 2)
			.map(|x| x.to_string())
			.lower();
		assert_eq!(result, vec!["4", "6", "8"]);
	}

	#[test]
	fn functor_identity_law() {
		// map(identity, fa) = fa
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let result = map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(identity, coyo).lower();
		assert_eq!(result, vec![1, 2, 3]);
	}

	#[test]
	fn functor_composition_law() {
		// map(compose(f, g), fa) = map(f, map(g, fa))
		let f = |x: i32| x + 1;
		let g = |x: i32| x * 2;

		let coyo1 = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let left =
			map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(compose(f, g), coyo1).lower();

		let coyo2 = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let right = map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(
			f,
			map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(g, coyo2),
		)
		.lower();

		assert_eq!(left, right);
	}

	#[test]
	fn many_chained_maps() {
		let mut coyo = Coyoneda::<VecBrand, _>::lift(vec![0i64]);
		for _ in 0 .. 100 {
			coyo = coyo.map(|x| x + 1);
		}
		assert_eq!(coyo.lower(), vec![100i64]);
	}

	// -- Collapse tests --

	#[test]
	fn collapse_preserves_value() {
		let coyo =
			Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).map(|x| x * 2).collapse();
		assert_eq!(coyo.lower(), vec![4, 6, 8]);
	}

	#[test]
	fn collapse_then_map() {
		let coyo =
			Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).collapse().map(|x| x * 2);
		assert_eq!(coyo.lower(), vec![4, 6, 8]);
	}

	#[test]
	fn collapse_periodic_in_loop() {
		let mut coyo = Coyoneda::<VecBrand, _>::lift(vec![0i64]);
		for i in 0 .. 50 {
			coyo = coyo.map(|x| x + 1);
			if i % 25 == 24 {
				coyo = coyo.collapse();
			}
		}
		assert_eq!(coyo.lower(), vec![50i64]);
	}

	#[test]
	fn map_on_none_stays_none() {
		let result =
			Coyoneda::<OptionBrand, _>::lift(None::<i32>).map(|x| x + 1).map(|x| x * 2).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn map_free_function_dispatches_to_brand() {
		let coyo = Coyoneda::<OptionBrand, _>::lift(Some(10));
		let result: Coyoneda<OptionBrand, String> =
			map_explicit::<CoyonedaBrand<OptionBrand>, _, _, _, _>(|x: i32| x.to_string(), coyo);
		assert_eq!(result.lower(), Some("10".to_string()));
	}

	#[test]
	fn lift_lower_roundtrip_preserves_value() {
		let original = vec![10, 20, 30];
		let roundtrip = Coyoneda::<VecBrand, _>::lift(original.clone()).lower();
		assert_eq!(roundtrip, original);
	}

	// -- Pointed tests --

	#[test]
	fn pointed_pure_option() {
		let coyo: Coyoneda<OptionBrand, i32> = pure::<CoyonedaBrand<OptionBrand>, _>(42);
		assert_eq!(coyo.lower(), Some(42));
	}

	#[test]
	fn pointed_pure_vec() {
		let coyo: Coyoneda<VecBrand, i32> = pure::<CoyonedaBrand<VecBrand>, _>(42);
		assert_eq!(coyo.lower(), vec![42]);
	}

	// -- Hoist tests --

	struct VecToOption;
	impl NaturalTransformation<VecBrand, OptionBrand> for VecToOption {
		fn transform<'a, A: 'a>(
			&self,
			fa: Vec<A>,
		) -> Option<A> {
			fa.into_iter().next()
		}
	}

	#[test]
	fn hoist_vec_to_option() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![10, 20, 30]);
		let hoisted: Coyoneda<OptionBrand, i32> = coyo.hoist(VecToOption);
		assert_eq!(hoisted.lower(), Some(10));
	}

	#[test]
	fn hoist_with_maps() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let hoisted = coyo.hoist(VecToOption);
		assert_eq!(hoisted.lower(), Some(10));
	}

	#[test]
	fn hoist_then_map() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![5, 10, 15]);
		let hoisted = coyo.hoist(VecToOption).map(|x: i32| x.to_string());
		assert_eq!(hoisted.lower(), Some("5".to_string()));
	}

	// -- Foldable tests --

	#[test]
	fn fold_map_on_lifted_vec() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let result = fold_map::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _, _, _>(
			|x: i32| x.to_string(),
			coyo,
		);
		assert_eq!(result, "123".to_string());
	}

	#[test]
	fn fold_map_on_mapped_coyoneda() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let result = fold_map::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _, _, _>(
			|x: i32| x.to_string(),
			coyo,
		);
		assert_eq!(result, "102030".to_string());
	}

	#[test]
	fn fold_right_on_coyoneda() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 2);
		let result = fold_right::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _, _, _>(
			|a: i32, b: i32| a + b,
			0,
			coyo,
		);
		assert_eq!(result, 12); // (1*2) + (2*2) + (3*2) = 2 + 4 + 6 = 12
	}

	#[test]
	fn fold_left_on_coyoneda() {
		let coyo = Coyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x + 1);
		let result = fold_left::<RcFnBrand, CoyonedaBrand<OptionBrand>, _, _, _, _>(
			|acc: i32, a: i32| acc + a,
			10,
			coyo,
		);
		assert_eq!(result, 16); // 10 + (5 + 1) = 16
	}

	#[test]
	fn fold_map_on_none_is_empty() {
		let coyo = Coyoneda::<OptionBrand, i32>::lift(None).map(|x| x + 1);
		let result = fold_map::<RcFnBrand, CoyonedaBrand<OptionBrand>, _, _, _, _>(
			|x: i32| x.to_string(),
			coyo,
		);
		assert_eq!(result, String::new());
	}

	// -- Lift tests --

	#[test]
	fn lift2_option_both_some() {
		let a = Coyoneda::<OptionBrand, _>::lift(Some(3));
		let b = Coyoneda::<OptionBrand, _>::lift(Some(4));
		let result = lift2::<CoyonedaBrand<OptionBrand>, _, _, _, _, _, _>(|x, y| x + y, a, b);
		assert_eq!(result.lower(), Some(7));
	}

	#[test]
	fn lift2_option_one_none() {
		let a = Coyoneda::<OptionBrand, _>::lift(Some(3));
		let b = Coyoneda::<OptionBrand, i32>::lift(None);
		let result = lift2::<CoyonedaBrand<OptionBrand>, _, _, _, _, _, _>(|x, y| x + y, a, b);
		assert_eq!(result.lower(), None);
	}

	#[test]
	fn lift2_vec() {
		let a = Coyoneda::<VecBrand, _>::lift(vec![1, 2]);
		let b = Coyoneda::<VecBrand, _>::lift(vec![10, 20]);
		let result = lift2::<CoyonedaBrand<VecBrand>, _, _, _, _, _, _>(|x, y| x + y, a, b);
		assert_eq!(result.lower(), vec![11, 21, 12, 22]);
	}

	#[test]
	fn lift2_with_prior_maps() {
		let a = Coyoneda::<OptionBrand, _>::lift(Some(3)).map(|x| x * 2);
		let b = Coyoneda::<OptionBrand, _>::lift(Some(4)).map(|x| x + 1);
		let result = lift2::<CoyonedaBrand<OptionBrand>, _, _, _, _, _, _>(|x, y| x + y, a, b);
		assert_eq!(result.lower(), Some(11)); // (3*2) + (4+1)
	}

	// -- Semiapplicative tests --

	#[test]
	fn apply_option_some() {
		let ff =
			Coyoneda::<OptionBrand, _>::lift(Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2)));
		let fa = Coyoneda::<OptionBrand, _>::lift(Some(5));
		let result = apply::<RcFnBrand, CoyonedaBrand<OptionBrand>, _, _>(ff, fa);
		assert_eq!(result.lower(), Some(10));
	}

	#[test]
	fn apply_option_none_fn() {
		let ff = Coyoneda::<OptionBrand, _>::lift(None::<<RcFnBrand as CloneFn>::Of<'_, i32, i32>>);
		let fa = Coyoneda::<OptionBrand, _>::lift(Some(5));
		let result = apply::<RcFnBrand, CoyonedaBrand<OptionBrand>, _, _>(ff, fa);
		assert_eq!(result.lower(), None);
	}

	#[test]
	fn apply_vec() {
		let ff = Coyoneda::<VecBrand, _>::lift(vec![
			lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1),
			lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 10),
		]);
		let fa = Coyoneda::<VecBrand, _>::lift(vec![2i32, 3]);
		let result = apply::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _>(ff, fa);
		assert_eq!(result.lower(), vec![3, 4, 20, 30]);
	}

	// -- Semimonad tests --

	#[test]
	fn bind_option_some() {
		let coyo = Coyoneda::<OptionBrand, _>::lift(Some(5));
		let result = bind::<CoyonedaBrand<OptionBrand>, _, _, _, _>(coyo, |x| {
			Coyoneda::<OptionBrand, _>::lift(Some(x * 2))
		});
		assert_eq!(result.lower(), Some(10));
	}

	#[test]
	fn bind_option_none() {
		let coyo = Coyoneda::<OptionBrand, i32>::lift(None);
		let result = bind::<CoyonedaBrand<OptionBrand>, _, _, _, _>(coyo, |x| {
			Coyoneda::<OptionBrand, _>::lift(Some(x * 2))
		});
		assert_eq!(result.lower(), None);
	}

	#[test]
	fn bind_vec() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1i32, 2, 3]);
		let result = bind::<CoyonedaBrand<VecBrand>, _, _, _, _>(coyo, |x| {
			Coyoneda::<VecBrand, _>::lift(vec![x, x * 10])
		});
		assert_eq!(result.lower(), vec![1, 10, 2, 20, 3, 30]);
	}

	#[test]
	fn bind_after_map() {
		let coyo = Coyoneda::<OptionBrand, _>::lift(Some(3)).map(|x| x * 2);
		let result = bind::<CoyonedaBrand<OptionBrand>, _, _, _, _>(coyo, |x| {
			Coyoneda::<OptionBrand, _>::lift(Some(x + 1))
		});
		assert_eq!(result.lower(), Some(7)); // (3 * 2) + 1
	}

	// -- From conversion tests --

	#[test]
	fn from_coyoneda_to_explicit() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		let explicit: CoyonedaExplicit<VecBrand, i32, i32, fn(i32) -> i32> = coyo.into();
		assert_eq!(explicit.lower(), vec![2, 3, 4]);
	}

	#[test]
	fn from_coyoneda_then_map_lower() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		let result = CoyonedaExplicit::<VecBrand, i32, i32, fn(i32) -> i32>::from(coyo)
			.map(|x| x * 2)
			.lower();
		assert_eq!(result, vec![4, 6, 8]);
	}

	#[test]
	fn from_roundtrip_preserves_semantics() {
		let original = vec![1, 2, 3];

		// CoyonedaExplicit -> Coyoneda -> CoyonedaExplicit
		let explicit = CoyonedaExplicit::<VecBrand, _, _, _>::lift(original.clone()).map(|x| x + 1);
		let coyo: Coyoneda<VecBrand, i32> = explicit.into();
		let back: CoyonedaExplicit<VecBrand, i32, i32, fn(i32) -> i32> = coyo.into();
		assert_eq!(back.lower(), vec![2, 3, 4]);
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
			let coyo = Coyoneda::<VecBrand, _>::lift(v.clone());
			map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(identity, coyo).lower() == v
		}

		#[quickcheck]
		fn functor_identity_option(x: Option<i32>) -> bool {
			let coyo = Coyoneda::<OptionBrand, _>::lift(x);
			map_explicit::<CoyonedaBrand<OptionBrand>, _, _, _, _>(identity, coyo).lower() == x
		}

		#[quickcheck]
		fn functor_composition_vec(v: Vec<i32>) -> bool {
			let f = |x: i32| x.wrapping_add(1);
			let g = |x: i32| x.wrapping_mul(2);

			let left = map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(
				compose(f, g),
				Coyoneda::<VecBrand, _>::lift(v.clone()),
			)
			.lower();

			let right = map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(
				f,
				map_explicit::<CoyonedaBrand<VecBrand>, _, _, _, _>(
					g,
					Coyoneda::<VecBrand, _>::lift(v),
				),
			)
			.lower();

			left == right
		}
	}
}
