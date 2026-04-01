//! Coyoneda with the intermediate type made explicit, enabling zero-cost map fusion.
//!
//! [`CoyonedaExplicit`] is the same construction as [`Coyoneda`](crate::types::Coyoneda)
//! but without existential quantification over the intermediate type `B`. Where `Coyoneda`
//! hides `B` behind a trait object (enabling HKT integration), `CoyonedaExplicit` exposes
//! `B` as a type parameter (enabling compile-time function composition).
//!
//! ## Map fusion
//!
//! Each call to [`map`](CoyonedaExplicit::map) composes the new function with the
//! accumulated function at compile time. No boxing, no dynamic dispatch, no heap
//! allocation. At [`lower`](CoyonedaExplicit::lower) time, a single call to `F::map`
//! applies the fully composed function regardless of how many maps were chained.
//! Use [`.boxed()`](CoyonedaExplicit::boxed) when a uniform type is needed (struct
//! fields, loops, collections).
//!
//! For chains deeper than ~20-30 maps, consider inserting `.boxed()` to bound
//! compile-time type complexity.
//!
//! ## Send / Sync
//!
//! The compiler derives `Send` automatically when `Func: Send` and
//! `F::Of<'a, B>: Send`. No separate `SendCoyonedaExplicit` type is needed. Use
//! [`.boxed_send()`](CoyonedaExplicit::boxed_send) to erase the function type while
//! preserving `Send`.
//!
//! ## Trade-offs vs `Coyoneda`
//!
//! | Property | `Coyoneda` | `CoyonedaExplicit` |
//! | -------- | ---------- | ------------------ |
//! | HKT integration | Yes (has a brand, implements `Functor`) | No |
//! | Map fusion | No (k calls to `F::map`) | Yes (1 call to `F::map`) |
//! | Heap allocation per map | 1 box (function stored inline) | 0 (1 box with `.boxed()`) |
//! | Stack overflow risk | Yes (deep nesting) | No (compiler inlines; use `.boxed()` for deep chains) |
//! | Foldable without Functor | No | Yes |
//! | Hoist without Functor | No | Yes |
//! | Pointed via brand | Yes | No |
//! | Semimonad via brand | Yes | No |
//! | `B: 'static` required for brand | No | Yes |
//!
//! ## When to use which
//!
//! Use `Coyoneda` when you need HKT polymorphism (e.g., writing code generic over any
//! `Functor`). Use `CoyonedaExplicit` when you need zero-cost map fusion on a known
//! type constructor, or when composing many maps in a performance-sensitive path.
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
//! let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
//! 	.map(|x| x + 1)
//! 	.map(|x| x * 2)
//! 	.map(|x| x.to_string())
//! 	.lower();
//!
//! // Only one call to Vec::map, applying the composed function x -> (x + 1) * 2 -> string.
//! assert_eq!(result, vec!["4", "6", "8"]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::CoyonedaExplicitBrand,
			classes::{
				Applicative,
				CloneableFn,
				Foldable,
				FoldableWithIndex,
				Functor,
				Monoid,
				NaturalTransformation,
				Pointed,
				Semiapplicative,
				Semimonad,
				Traversable,
				WithIndex,
			},
			functions::{
				compose,
				identity,
			},
			impl_kind,
			kinds::*,
			types::Coyoneda,
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// Coyoneda with an explicit intermediate type, enabling zero-cost map fusion.
	///
	/// Stores a value of type `F B` alongside a function `B -> A`. Each call to
	/// [`map`](CoyonedaExplicit::map) composes the new function with the existing one
	/// at the type level, producing a new `CoyonedaExplicit` with an updated function
	/// type but the same underlying `F B`. At [`lower`](CoyonedaExplicit::lower) time,
	/// a single `F::map` applies the fully composed function.
	///
	/// No boxing, no dynamic dispatch, no heap allocation occurs during `map`. Use
	/// [`.boxed()`](CoyonedaExplicit::boxed) as an escape hatch when a uniform type is
	/// needed (struct fields, loops, collections).
	///
	/// Unlike [`Coyoneda`](crate::types::Coyoneda), the intermediate type `B` is visible
	/// as a type parameter rather than hidden behind a trait object. This prevents HKT
	/// integration (no brand or `Functor` instance) but reduces lowering to a single
	/// `F::map` call with zero overhead per `map`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the underlying functor (the input to the accumulated function).",
		"The current output type (the output of the accumulated function).",
		"The type of the accumulated function from `B` to `A`."
	)]
	pub struct CoyonedaExplicit<
		'a,
		F,
		B: 'a,
		A: 'a,
		Func: Fn(B) -> A + 'a = Box<dyn Fn(B) -> A + 'a>,
	>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
		func: Func,
		_phantom: PhantomData<A>,
	}

	/// Type alias for a [`CoyonedaExplicit`] with a boxed function, for use in
	/// struct fields, collections, loops, and HKT brands.
	pub type BoxedCoyonedaExplicit<'a, F, B, A> =
		CoyonedaExplicit<'a, F, B, A, Box<dyn Fn(B) -> A + 'a>>;

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the underlying functor.",
		"The current output type.",
		"The type of the accumulated function."
	)]
	#[document_parameters("The `CoyonedaExplicit` instance.")]
	impl<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> CoyonedaExplicit<'a, F, B, A, Func>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Construct a `CoyonedaExplicit` from a function and a functor value.
		///
		/// Stores `fb` alongside `f` as a single deferred mapping step.
		/// [`lift`](CoyonedaExplicit::lift) is equivalent to `new(|a| a, fa)`.
		#[document_signature]
		///
		#[document_parameters("The function to defer.", "The functor value.")]
		///
		#[document_returns("A `CoyonedaExplicit` wrapping the value with the deferred function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(coyo.lower(), vec![2, 4, 6]);
		/// ```
		pub fn new(
			f: Func,
			fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
		) -> Self {
			CoyonedaExplicit {
				fb,
				func: f,
				_phantom: PhantomData,
			}
		}

		/// Map a function over the value, composing it with the accumulated function.
		///
		/// This composes `f` with the stored function. No heap allocation occurs;
		/// the composition is stored inline.
		/// At [`lower`](CoyonedaExplicit::lower) time, a single `F::map` call applies
		/// the fully composed function.
		#[document_signature]
		///
		#[document_type_parameters("The new output type after applying the function.")]
		///
		#[document_parameters("The function to compose.")]
		///
		#[document_returns("A new `CoyonedaExplicit` with the composed function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let result = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5))
		/// 	.map(|x| x * 2)
		/// 	.map(|x| x + 1)
		/// 	.lower();
		///
		/// assert_eq!(result, Some(11));
		/// ```
		pub fn map<C: 'a>(
			self,
			f: impl Fn(A) -> C + 'a,
		) -> CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C + 'a> {
			CoyonedaExplicit {
				fb: self.fb,
				func: compose(f, self.func),
				_phantom: PhantomData,
			}
		}

		/// Lower the `CoyonedaExplicit` back to the underlying functor `F`.
		///
		/// Applies the accumulated composed function in a single call to `F::map`.
		/// Requires `F: Functor`.
		#[document_signature]
		///
		#[document_returns("The underlying functor value with the composed function applied.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
		/// 	.map(|x| x + 1)
		/// 	.map(|x| x * 2)
		/// 	.lower();
		///
		/// assert_eq!(result, vec![4, 6, 8]);
		/// ```
		pub fn lower(self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			F::map(self.func, self.fb)
		}

		/// Apply a natural transformation to the underlying functor.
		///
		/// Transforms a `CoyonedaExplicit<F, B, A>` into a `CoyonedaExplicit<G, B, A>`
		/// by applying the natural transformation directly to the stored `F B`. Unlike
		/// [`Coyoneda::hoist`](crate::types::Coyoneda::hoist), this does not require
		/// `F: Functor` because the intermediate type `B` is visible.
		#[document_signature]
		///
		#[document_type_parameters("The brand of the target functor.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("A new `CoyonedaExplicit` over the target functor `G`.")]
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
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![10, 20, 30]).map(|x| x * 2);
		/// let hoisted = coyo.hoist(VecToOption);
		/// assert_eq!(hoisted.lower(), Some(20));
		/// ```
		pub fn hoist<G: Kind_cdc7cd43dac7585f + 'a>(
			self,
			nat: impl NaturalTransformation<F, G>,
		) -> CoyonedaExplicit<'a, G, B, A, Func> {
			CoyonedaExplicit {
				fb: nat.transform(self.fb),
				func: self.func,
				_phantom: PhantomData,
			}
		}

		/// Fold the structure by composing the fold function with the accumulated
		/// mapping function, then folding the original `F B` in a single pass.
		///
		/// Unlike [`Foldable for CoyonedaBrand`](crate::classes::Foldable), this does
		/// not require `F: Functor`. It only requires `F: Foldable`, matching
		/// PureScript's semantics. No intermediate `F A` is materialized.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The monoid type to fold into."
		)]
		///
		#[document_parameters("The function mapping each element to a monoid value.")]
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
		/// let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
		/// 	.map(|x| x * 10)
		/// 	.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		///
		/// assert_eq!(result, "102030".to_string());
		/// ```
		pub fn fold_map<FnBrand, M>(
			self,
			func: impl Fn(A) -> M + 'a,
		) -> M
		where
			B: Clone,
			M: Monoid + 'a,
			F: Foldable,
			FnBrand: CloneableFn + 'a, {
			F::fold_map::<FnBrand, B, M>(compose(func, self.func), self.fb)
		}

		/// Fold the structure with index by composing the fold function with the
		/// accumulated mapping function, then folding the original `F B` in a
		/// single pass.
		///
		/// This does not require `F: Functor`, only `F: FoldableWithIndex`,
		/// matching PureScript's semantics. No intermediate `F A` is materialized.
		/// The index comes from `F`'s `FoldableWithIndex` instance.
		#[document_signature]
		///
		#[document_type_parameters("The monoid type to fold into.")]
		///
		#[document_parameters("The function mapping each index and element to a monoid value.")]
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
		/// let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
		/// 	.map(|x| x * 10)
		/// 	.fold_map_with_index(|i: usize, x: i32| format!("{i}:{x}"));
		///
		/// assert_eq!(result, "0:101:202:30".to_string());
		/// ```
		pub fn fold_map_with_index<M>(
			self,
			func: impl Fn(<F as WithIndex>::Index, A) -> M + 'a,
		) -> M
		where
			B: Clone,
			M: Monoid + 'a,
			F: FoldableWithIndex, {
			let f = self.func;
			F::fold_map_with_index(move |i, b| func(i, f(b)), self.fb)
		}

		/// Traverse the structure by composing the traversal function with the
		/// accumulated mapping function, traversing the original `F B` in a
		/// single pass, and wrapping the result in `CoyonedaExplicit`.
		///
		/// This does not require `F: Functor` beyond what `F: Traversable`
		/// already implies. Matches PureScript's `Traversable (Coyoneda f)`
		/// semantics.
		#[document_signature]
		///
		#[document_type_parameters(
			"The applicative context brand.",
			"The output element type after traversal."
		)]
		///
		#[document_parameters(
			"The function mapping each element to a value in the applicative context."
		)]
		///
		#[document_returns(
			"The traversed result wrapped in the applicative context, containing a `CoyonedaExplicit` in identity position."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		/// let result: Option<CoyonedaExplicit<VecBrand, _, _, _>> =
		/// 	coyo.traverse::<OptionBrand, _>(|x| if x > 0 { Some(x) } else { None });
		/// assert_eq!(result.map(|c| c.lower()), Some(vec![10, 20, 30]));
		/// ```
		#[allow(clippy::type_complexity)]
		pub fn traverse<G: Applicative + 'a, C: 'a + Clone>(
			self,
			f: impl Fn(A) -> <G as Kind_cdc7cd43dac7585f>::Of<'a, C> + 'a,
		) -> <G as Kind_cdc7cd43dac7585f>::Of<'a, CoyonedaExplicit<'a, F, C, C, fn(C) -> C>>
		where
			B: Clone,
			F: Traversable,
			<F as Kind_cdc7cd43dac7585f>::Of<'a, C>: Clone,
			<G as Kind_cdc7cd43dac7585f>::Of<'a, C>: Clone, {
			G::map(
				|fc| CoyonedaExplicit::lift(fc),
				F::traverse::<B, C, G>(compose(f, self.func), self.fb),
			)
		}

		/// Apply a wrapped function to this value by lowering both sides, delegating to
		/// `F::apply`, and re-lifting the result.
		///
		/// This is the `Semiapplicative` operation lifted to `CoyonedaExplicit` without
		/// requiring a brand. After the operation the fusion pipeline is reset: the
		/// result is a `CoyonedaExplicit` with the identity function and intermediate
		/// type `C`.
		///
		/// This is a fusion barrier: it calls `lower()` on both arguments,
		/// materializing all accumulated maps before delegating to `F::apply`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The intermediate type of the function container.",
			"The output type after applying the function.",
			"The type of the function in the function container."
		)]
		///
		#[document_parameters(
			"The `CoyonedaExplicit` containing the wrapped function.",
			"The `CoyonedaExplicit` containing the value."
		)]
		///
		#[document_returns(
			"A `CoyonedaExplicit` containing the result of applying the function to the value."
		)]
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
		/// 	CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(cloneable_fn_new::<RcFnBrand, _, _>(
		/// 		|x: i32| x * 2,
		/// 	)));
		/// let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5i32));
		/// let result = CoyonedaExplicit::apply::<RcFnBrand, _, _, _>(ff, fa).lower();
		/// assert_eq!(result, Some(10));
		/// ```
		pub fn apply<
			FnBrand: CloneableFn + 'a,
			Bf: 'a,
			C: 'a,
			FuncF: Fn(Bf) -> <FnBrand as CloneableFn>::Of<'a, A, C> + 'a,
		>(
			ff: CoyonedaExplicit<'a, F, Bf, <FnBrand as CloneableFn>::Of<'a, A, C>, FuncF>,
			fa: Self,
		) -> CoyonedaExplicit<'a, F, C, C, fn(C) -> C>
		where
			A: Clone,
			F: Semiapplicative, {
			CoyonedaExplicit::lift(F::apply::<FnBrand, A, C>(ff.lower(), fa.lower()))
		}

		/// Bind through the accumulated function directly, composing the callback
		/// with the accumulated mapping function and delegating to `F::bind`.
		///
		/// The callback `f` receives the mapped value (after the accumulated
		/// function is applied) and returns a raw `F::Of<'a, C>` directly. This
		/// avoids needing `F: Functor` and skips an intermediate `F::map` traversal.
		/// After the operation the fusion pipeline is reset: the result is a
		/// `CoyonedaExplicit` with the identity function and intermediate type `C`.
		#[document_signature]
		///
		#[document_type_parameters("The output type of the bound computation.")]
		///
		#[document_parameters(
			"The function to apply to each mapped value, returning a raw functor value."
		)]
		///
		#[document_returns("A `CoyonedaExplicit` containing the bound result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5i32)).map(|x| x * 2);
		/// let result = fa.bind(|x| Some(x + 1)).lower();
		/// assert_eq!(result, Some(11)); // (5 * 2) + 1
		/// ```
		pub fn bind<C: 'a>(
			self,
			f: impl Fn(A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, C> + 'a,
		) -> CoyonedaExplicit<'a, F, C, C, fn(C) -> C>
		where
			F: Semimonad, {
			let func = self.func;
			CoyonedaExplicit::lift(F::bind(self.fb, move |b| f(func(b))))
		}

		/// Erase the function type by boxing it.
		///
		/// This is the escape hatch for storing in struct fields, collections, or
		/// loop accumulators where a uniform type is needed. Reintroduces one
		/// `Box` allocation and dynamic dispatch.
		#[document_signature]
		///
		#[document_returns("A `BoxedCoyonedaExplicit` with the function boxed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// let boxed: BoxedCoyonedaExplicit<VecBrand, i32, i32> = coyo.boxed();
		/// assert_eq!(boxed.lower(), vec![2, 3, 4]);
		/// ```
		pub fn boxed(self) -> BoxedCoyonedaExplicit<'a, F, B, A> {
			CoyonedaExplicit {
				fb: self.fb,
				func: Box::new(self.func),
				_phantom: PhantomData,
			}
		}

		/// Erase the function type by boxing it with `Send`.
		///
		/// Like [`boxed`](CoyonedaExplicit::boxed), but the resulting function is
		/// `Send`, allowing the value to cross thread boundaries.
		#[document_signature]
		///
		#[document_returns("A `CoyonedaExplicit` with the function boxed as `Send`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// let boxed = coyo.boxed_send();
		/// assert_eq!(boxed.lower(), vec![2, 3, 4]);
		/// ```
		pub fn boxed_send(self) -> CoyonedaExplicit<'a, F, B, A, Box<dyn Fn(B) -> A + Send + 'a>>
		where
			Func: Send, {
			CoyonedaExplicit {
				fb: self.fb,
				func: Box::new(self.func),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the functor."
	)]
	impl<'a, F, A: 'a> CoyonedaExplicit<'a, F, A, A, fn(A) -> A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Lift a value of `F A` into `CoyonedaExplicit` with the identity function.
		///
		/// This is the starting point for building a fusion pipeline. The intermediate
		/// type `B` and the output type `A` are the same.
		#[document_signature]
		///
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns("A `CoyonedaExplicit` wrapping the value with the identity function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(42));
		/// assert_eq!(coyo.lower(), Some(42));
		/// ```
		pub fn lift(fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>) -> Self {
			CoyonedaExplicit {
				fb: fa,
				func: identity as fn(A) -> A,
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying pointed functor.",
		"The type of the value."
	)]
	impl<'a, F, A: 'a> CoyonedaExplicit<'a, F, A, A, fn(A) -> A>
	where
		F: Pointed + 'a,
	{
		/// Wrap a pure value in a `CoyonedaExplicit` context.
		///
		/// Delegates to `F::pure` and wraps with the identity function.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `CoyonedaExplicit` containing the pure value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = CoyonedaExplicit::<OptionBrand, _, _, _>::pure(42);
		/// assert_eq!(coyo.lower(), Some(42));
		/// ```
		pub fn pure(a: A) -> Self {
			Self::lift(F::pure(a))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the underlying functor.",
		"The current output type.",
		"The type of the accumulated function."
	)]
	impl<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> From<CoyonedaExplicit<'a, F, B, A, Func>>
		for Coyoneda<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		/// Convert a [`CoyonedaExplicit`] into a [`Coyoneda`] by hiding the
		/// intermediate type `B` behind a trait object.
		///
		/// This is useful when you have finished building a fusion pipeline and
		/// need to pass the result into code that is generic over `Functor` via
		/// `CoyonedaBrand`.
		///
		/// Note: further `map` calls on the resulting `Coyoneda` do not fuse with
		/// the previously composed function; each adds a separate trait-object
		/// layer.
		#[document_signature]
		///
		#[document_parameters("The `CoyonedaExplicit` to convert.")]
		///
		#[document_returns("A `Coyoneda` wrapping the same value with the accumulated function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let explicit = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// let coyo: Coyoneda<VecBrand, i32> = explicit.into();
		/// assert_eq!(coyo.lower(), vec![2, 3, 4]);
		/// ```
		fn from(explicit: CoyonedaExplicit<'a, F, B, A, Func>) -> Self {
			Coyoneda::new(explicit.func, explicit.fb)
		}
	}

	// -- Brand --

	impl_kind! {
		impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> for CoyonedaExplicitBrand<F, B> {
			type Of<'a, A: 'a>: 'a = BoxedCoyonedaExplicit<'a, F, B, A>;
		}
	}

	// -- Functor for CoyonedaExplicitBrand --

	#[document_type_parameters(
		"The brand of the underlying type constructor.",
		"The type of the values in the underlying functor."
	)]
	impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> Functor for CoyonedaExplicitBrand<F, B> {
		/// Maps a function over the `BoxedCoyonedaExplicit` by composing it with the
		/// accumulated function, then re-boxing.
		///
		/// Does not require `F: Functor`. The function is composed at the type level
		/// and a single `F::map` call applies the result at
		/// [`lower`](CoyonedaExplicit::lower) time. This preserves single-pass fusion,
		/// unlike [`CoyonedaBrand`](crate::brands::CoyonedaBrand) which adds a separate
		/// layer per map.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the current output.",
			"The type of the new output."
		)]
		///
		#[document_parameters("The function to apply.", "The `BoxedCoyonedaExplicit` value.")]
		///
		#[document_returns("A new `BoxedCoyonedaExplicit` with the composed function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).boxed();
		/// let mapped = map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(|x| x * 10, coyo);
		/// assert_eq!(mapped.lower(), vec![10, 20, 30]);
		/// ```
		fn map<'a, A: 'a, C: 'a>(
			func: impl Fn(A) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			fa.map(func).boxed()
		}
	}

	// -- Foldable for CoyonedaExplicitBrand --

	#[document_type_parameters(
		"The brand of the underlying foldable type constructor.",
		"The type of the values in the underlying functor."
	)]
	impl<F: Kind_cdc7cd43dac7585f + Foldable + 'static, B: Clone + 'static> Foldable
		for CoyonedaExplicitBrand<F, B>
	{
		/// Folds the `BoxedCoyonedaExplicit` by composing the fold function with the
		/// accumulated mapping function, then folding the original `F B` in a single
		/// pass.
		///
		/// Unlike [`Foldable for CoyonedaBrand`](crate::classes::Foldable), this does
		/// not require `F: Functor`. It only requires `F: Foldable`, matching
		/// PureScript's semantics. No intermediate `F A` is materialized.
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
			"The `BoxedCoyonedaExplicit` structure to fold."
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
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x * 10).boxed();
		///
		/// let result = fold_map::<RcFnBrand, CoyonedaExplicitBrand<VecBrand, i32>, _, _>(
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
			FnBrand: CloneableFn + 'a, {
			fa.fold_map::<FnBrand, M>(func)
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
		let coyo = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(42));
		assert_eq!(coyo.lower(), Some(42));
	}

	#[test]
	fn lift_lower_identity_none() {
		let coyo = CoyonedaExplicit::<OptionBrand, i32, i32, _>::lift(None);
		assert_eq!(coyo.lower(), None);
	}

	#[test]
	fn lift_lower_identity_vec() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]);
		assert_eq!(coyo.lower(), vec![1, 2, 3]);
	}

	#[test]
	fn new_constructor() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		assert_eq!(coyo.lower(), vec![2, 4, 6]);
	}

	#[test]
	fn new_is_equivalent_to_lift_then_map() {
		let f = |x: i32| x.to_string();
		let v = vec![1, 2, 3];

		let via_new = CoyonedaExplicit::<VecBrand, _, _, _>::new(f, v.clone()).lower();
		let via_lift_map = CoyonedaExplicit::<VecBrand, _, _, _>::lift(v).map(f).lower();

		assert_eq!(via_new, via_lift_map);
	}

	#[test]
	fn single_map_option() {
		let result = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5)).map(|x| x * 2).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn chained_maps_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
			.map(|x| x + 1)
			.map(|x| x * 2)
			.map(|x| x.to_string())
			.lower();
		assert_eq!(result, vec!["4", "6", "8"]);
	}

	#[test]
	fn functor_identity_law() {
		let result =
			CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(identity).lower();
		assert_eq!(result, vec![1, 2, 3]);
	}

	#[test]
	fn functor_composition_law() {
		let f = |x: i32| x + 1;
		let g = |x: i32| x * 2;

		let left =
			CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(compose(f, g)).lower();

		let right =
			CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(g).map(f).lower();

		assert_eq!(left, right);
	}

	#[test]
	fn many_chained_maps() {
		let mut coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![0i64]).boxed();
		for _ in 0 .. 100 {
			coyo = coyo.map(|x| x + 1).boxed();
		}
		assert_eq!(coyo.lower(), vec![100i64]);
	}

	#[test]
	fn map_on_none_stays_none() {
		let result = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(None::<i32>)
			.map(|x| x + 1)
			.map(|x| x * 2)
			.lower();
		assert_eq!(result, None);
	}

	#[test]
	fn lift_lower_roundtrip_preserves_value() {
		let original = vec![10, 20, 30];
		let roundtrip = CoyonedaExplicit::<VecBrand, _, _, _>::lift(original.clone()).lower();
		assert_eq!(roundtrip, original);
	}

	// -- Pure tests --

	#[test]
	fn pure_option() {
		let coyo = CoyonedaExplicit::<OptionBrand, _, _, _>::pure(42);
		assert_eq!(coyo.lower(), Some(42));
	}

	#[test]
	fn pure_vec() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::pure(42);
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
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![10, 20, 30]);
		let hoisted = coyo.hoist(VecToOption);
		assert_eq!(hoisted.lower(), Some(10));
	}

	#[test]
	fn hoist_preserves_accumulated_maps() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let hoisted = coyo.hoist(VecToOption);
		assert_eq!(hoisted.lower(), Some(10));
	}

	#[test]
	fn hoist_then_map() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![5, 10, 15]);
		let hoisted = coyo.hoist(VecToOption).map(|x: i32| x.to_string());
		assert_eq!(hoisted.lower(), Some("5".to_string()));
	}

	// -- Fold tests --

	#[test]
	fn fold_map_on_lifted_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
			.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		assert_eq!(result, "123".to_string());
	}

	#[test]
	fn fold_map_on_mapped_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
			.map(|x| x * 10)
			.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		assert_eq!(result, "102030".to_string());
	}

	#[test]
	fn fold_map_on_none_is_empty() {
		let result = CoyonedaExplicit::<OptionBrand, i32, i32, _>::lift(None)
			.map(|x| x + 1)
			.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		assert_eq!(result, String::new());
	}

	// -- Traverse tests --

	#[test]
	fn traverse_vec_to_option_all_pass() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let result: Option<CoyonedaExplicit<VecBrand, _, _, _>> =
			coyo.traverse::<OptionBrand, _>(|x| if x > 0 { Some(x) } else { None });
		assert_eq!(result.map(|c| c.lower()), Some(vec![10, 20, 30]));
	}

	#[test]
	fn traverse_vec_to_option_one_fails() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, -2, 3]).map(|x| x * 10);
		let result: Option<CoyonedaExplicit<VecBrand, _, _, _>> =
			coyo.traverse::<OptionBrand, _>(|x| if x > 0 { Some(x) } else { None });
		assert_eq!(result.map(|c| c.lower()), None);
	}

	#[test]
	fn traverse_option_to_vec() {
		let coyo = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5)).map(|x| x * 2);
		let result: Vec<CoyonedaExplicit<OptionBrand, _, _, _>> =
			coyo.traverse::<VecBrand, _>(|x| vec![x, x + 1]);
		let lowered: Vec<Option<i32>> = result.into_iter().map(|c| c.lower()).collect();
		assert_eq!(lowered, vec![Some(10), Some(11)]);
	}

	#[test]
	fn traverse_lifted_identity() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]);
		let result: Option<CoyonedaExplicit<VecBrand, _, _, _>> =
			coyo.traverse::<OptionBrand, _>(|x| Some(x));
		assert_eq!(result.map(|c| c.lower()), Some(vec![1, 2, 3]));
	}

	// -- FoldableWithIndex tests --

	#[test]
	fn fold_map_with_index_on_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
			.map(|x| x * 10)
			.fold_map_with_index(|i: usize, x: i32| format!("{i}:{x}"));
		assert_eq!(result, "0:101:202:30".to_string());
	}

	#[test]
	fn fold_map_with_index_on_lifted_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![10, 20, 30])
			.fold_map_with_index(|i: usize, x: i32| format!("{i}:{x}"));
		assert_eq!(result, "0:101:202:30".to_string());
	}

	#[test]
	fn fold_map_with_index_on_option() {
		let result = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(42))
			.map(|x| x + 1)
			.fold_map_with_index(|_: (), x: i32| x.to_string());
		assert_eq!(result, "43".to_string());
	}

	#[test]
	fn fold_map_with_index_on_none() {
		let result = CoyonedaExplicit::<OptionBrand, i32, i32, _>::lift(None)
			.map(|x| x + 1)
			.fold_map_with_index(|_: (), x: i32| x.to_string());
		assert_eq!(result, String::new());
	}

	// -- Conversion tests --

	#[test]
	fn into_coyoneda_preserves_semantics() {
		let explicit = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
			.map(|x| x + 1)
			.map(|x| x * 2);
		let coyo: Coyoneda<VecBrand, i32> = explicit.into();
		assert_eq!(coyo.lower(), vec![4, 6, 8]);
	}

	#[test]
	fn into_coyoneda_from_lift() {
		let explicit = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(42));
		let coyo: Coyoneda<OptionBrand, i32> = explicit.into();
		assert_eq!(coyo.lower(), Some(42));
	}

	// -- Apply tests --

	#[test]
	fn apply_some_to_some() {
		let ff =
			CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(
				cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2),
			));
		let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5i32));
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _, _>(ff, fa).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn apply_none_fn_to_some() {
		let ff = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(
			None::<<RcFnBrand as CloneableFn>::Of<'_, i32, i32>>,
		);
		let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5i32));
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _, _>(ff, fa).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn apply_some_fn_to_none() {
		let ff =
			CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(
				cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2),
			));
		let fa = CoyonedaExplicit::<OptionBrand, i32, i32, _>::lift(None);
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _, _>(ff, fa).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn apply_vec_applies_each_fn_to_each_value() {
		let ff = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![
			cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1),
			cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 10),
		]);
		let fa = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![2i32, 3]);
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _, _>(ff, fa).lower();
		assert_eq!(result, vec![3, 4, 20, 30]);
	}

	#[test]
	fn apply_preserves_prior_maps_on_fa() {
		let ff =
			CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(
				cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1),
			));
		// Prior map on fa is composed and applied before apply delegates to F.
		let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5i32)).map(|x| x * 2);
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _, _>(ff, fa).lower();
		assert_eq!(result, Some(11)); // (5 * 2) + 1
	}

	// -- Bind tests --

	#[test]
	fn bind_some() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5i32));
		let result = fa.bind(|x| Some(x * 2)).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn bind_none_stays_none() {
		let fa = CoyonedaExplicit::<OptionBrand, i32, i32, _>::lift(None);
		let result = fa.bind(|x| Some(x * 2)).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn bind_returning_none_gives_none() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(5i32));
		let result = fa.bind(|_| None::<i32>).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn bind_vec() {
		let fa = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1i32, 2, 3]);
		let result = fa.bind(|x| vec![x, x * 10]).lower();
		assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
	}

	#[test]
	fn bind_uses_accumulated_maps() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(3i32)).map(|x| x * 2);
		let result = fa.bind(|x| Some(x + 1)).lower();
		assert_eq!(result, Some(7)); // (3 * 2) + 1
	}

	#[test]
	fn bind_vec_with_maps() {
		let fa = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1i32, 2, 3]).map(|x| x * 2);
		let result = fa.bind(|x| vec![x, x + 1]).lower();
		assert_eq!(result, vec![2, 3, 4, 5, 6, 7]);
	}

	// -- From conversion tests --

	#[test]
	fn from_explicit_to_coyoneda() {
		let explicit = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3])
			.map(|x| x + 1)
			.map(|x| x * 2);
		let coyo: Coyoneda<VecBrand, i32> = explicit.into();
		assert_eq!(coyo.lower(), vec![4, 6, 8]);
	}

	#[test]
	fn from_explicit_lift_only() {
		let explicit = CoyonedaExplicit::<OptionBrand, _, _, _>::lift(Some(42));
		let coyo: Coyoneda<OptionBrand, i32> = explicit.into();
		assert_eq!(coyo.lower(), Some(42));
	}

	// -- Boxed tests --

	#[test]
	fn test_boxed_erases_type() {
		fn assert_same_type<T>(
			_a: &T,
			_b: &T,
		) {
		}
		let a = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x + 1).boxed();
		let b = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![4, 5, 6]).map(|x| x * 2).boxed();
		assert_same_type(&a, &b);
		assert_eq!(a.lower(), vec![2, 3, 4]);
		assert_eq!(b.lower(), vec![8, 10, 12]);
	}

	#[test]
	fn test_boxed_send() {
		fn assert_send<T: Send>(_: &T) {}
		let coyo =
			CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x + 1).boxed_send();
		assert_send(&coyo);
		assert_eq!(coyo.lower(), vec![2, 3, 4]);
	}

	#[test]
	fn test_send_auto_derived() {
		fn assert_send<T: Send>(_: &T) {}
		// fn(i32) -> i32 is Send, Vec<i32> is Send, so the whole thing is Send.
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]);
		assert_send(&coyo);
	}

	// -- Brand tests --

	#[test]
	fn brand_functor_map() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).boxed();
		let mapped = map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(|x| x * 10, coyo);
		assert_eq!(mapped.lower(), vec![10, 20, 30]);
	}

	#[test]
	fn brand_functor_identity_law() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).boxed();
		let result = map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(identity, coyo).lower();
		assert_eq!(result, vec![1, 2, 3]);
	}

	#[test]
	fn brand_functor_composition_law() {
		let f = |x: i32| x + 1;
		let g = |x: i32| x * 2;

		let coyo1 = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).boxed();
		let left = map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(compose(f, g), coyo1).lower();

		let coyo2 = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).boxed();
		let right = map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(
			f,
			map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(g, coyo2),
		)
		.lower();

		assert_eq!(left, right);
	}

	#[test]
	fn brand_functor_chained_maps_fuse() {
		// Chaining through the brand still produces single-pass fusion.
		let coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).boxed();
		let result = map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(
			|x: i32| x.to_string(),
			map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(
				|x| x * 2,
				map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(|x| x + 1, coyo),
			),
		)
		.lower();
		assert_eq!(result, vec!["4", "6", "8"]);
	}

	#[test]
	fn brand_foldable_fold_map() {
		let coyo =
			CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x * 10).boxed();
		let result = fold_map::<RcFnBrand, CoyonedaExplicitBrand<VecBrand, i32>, _, _>(
			|x: i32| x.to_string(),
			coyo,
		);
		assert_eq!(result, "102030".to_string());
	}

	#[test]
	fn brand_foldable_fold_right() {
		let coyo =
			CoyonedaExplicit::<VecBrand, _, _, _>::lift(vec![1, 2, 3]).map(|x| x * 2).boxed();
		let result = fold_right::<RcFnBrand, CoyonedaExplicitBrand<VecBrand, i32>, _, _>(
			|a: i32, b: i32| a + b,
			0,
			coyo,
		);
		assert_eq!(result, 12); // (1*2) + (2*2) + (3*2)
	}

	#[test]
	fn brand_foldable_on_none() {
		let coyo = CoyonedaExplicit::<OptionBrand, i32, i32, _>::lift(None).map(|x| x + 1).boxed();
		let result = fold_map::<RcFnBrand, CoyonedaExplicitBrand<OptionBrand, i32>, _, _>(
			|x: i32| x.to_string(),
			coyo,
		);
		assert_eq!(result, String::new());
	}
}
