//! Coyoneda with the intermediate type made explicit, enabling single-pass map fusion.
//!
//! [`CoyonedaExplicit`] is the same construction as [`Coyoneda`](crate::types::Coyoneda)
//! but without existential quantification over the intermediate type `B`. Where `Coyoneda`
//! hides `B` behind a trait object (enabling HKT integration), `CoyonedaExplicit` exposes
//! `B` as a type parameter (enabling compile-time function composition).
//!
//! ## Map fusion
//!
//! Each call to [`map`](CoyonedaExplicit::map) allocates one `Box<dyn Fn>` for the
//! composed function. At [`lower`](CoyonedaExplicit::lower) time, a single call to
//! `F::map` applies the fully composed function regardless of how many maps were
//! chained.
//!
//! ## Trade-offs vs `Coyoneda`
//!
//! | Property | `Coyoneda` | `CoyonedaExplicit` |
//! | -------- | ---------- | ------------------ |
//! | HKT integration | Yes (has a brand, implements `Functor`) | No |
//! | Map fusion | No (k calls to `F::map`) | Yes (1 call to `F::map`) |
//! | Heap allocation per map | 2 boxes | 1 box |
//! | Stack overflow risk | Yes (deep nesting) | Yes (deep closures) |
//! | Foldable without Functor | No | Yes |
//! | Hoist without Functor | No | Yes |
//!
//! ## When to use which
//!
//! Use `Coyoneda` when you need HKT polymorphism (e.g., writing code generic over any
//! `Functor`). Use `CoyonedaExplicit` when you need single-pass map fusion on a known
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
//! let result = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
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
			classes::{
				CloneableFn,
				Foldable,
				Functor,
				Monoid,
				NaturalTransformation,
				Pointed,
				Semiapplicative,
				Semimonad,
			},
			functions::{
				compose,
				identity,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Coyoneda with an explicit intermediate type, enabling single-pass map fusion.
	///
	/// Stores a value of type `F B` alongside a function `B -> A`. Each call to
	/// [`map`](CoyonedaExplicit::map) composes the new function with the existing one
	/// at the type level, producing a new `CoyonedaExplicit` with an updated function
	/// type but the same underlying `F B`. At [`lower`](CoyonedaExplicit::lower) time,
	/// a single `F::map` applies the fully composed function.
	///
	/// Unlike [`Coyoneda`](crate::types::Coyoneda), the intermediate type `B` is visible
	/// as a type parameter rather than hidden behind a trait object. This prevents HKT
	/// integration (no brand or `Functor` instance) but reduces lowering to a single
	/// `F::map` call, though each `map` allocates one `Box<dyn Fn>` for the composed
	/// function.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the underlying functor (the input to the accumulated function).",
		"The current output type (the output of the accumulated function)."
	)]
	pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
		func: Box<dyn Fn(B) -> A + 'a>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the underlying functor.",
		"The current output type."
	)]
	#[document_parameters("The `CoyonedaExplicit` instance.")]
	impl<'a, F, B: 'a, A: 'a> CoyonedaExplicit<'a, F, B, A>
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
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(coyo.lower(), vec![2, 4, 6]);
		/// ```
		pub fn new(
			f: impl Fn(B) -> A + 'a,
			fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
		) -> Self {
			CoyonedaExplicit {
				fb,
				func: Box::new(f),
			}
		}

		/// Map a function over the value, composing it with the accumulated function.
		///
		/// This composes `f` with the stored function. A new `Box<dyn Fn>` is allocated
		/// wrapping the composition of the new function with the stored function.
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
		/// let result =
		/// 	CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5)).map(|x| x * 2).map(|x| x + 1).lower();
		///
		/// assert_eq!(result, Some(11));
		/// ```
		pub fn map<C: 'a>(
			self,
			f: impl Fn(A) -> C + 'a,
		) -> CoyonedaExplicit<'a, F, B, C> {
			CoyonedaExplicit {
				fb: self.fb,
				func: Box::new(compose(f, self.func)),
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
		/// let result = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
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
		/// let coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![10, 20, 30]).map(|x| x * 2);
		/// let hoisted = coyo.hoist(VecToOption);
		/// assert_eq!(hoisted.lower(), Some(20));
		/// ```
		pub fn hoist<G: Kind_cdc7cd43dac7585f + 'a>(
			self,
			nat: impl NaturalTransformation<F, G>,
		) -> CoyonedaExplicit<'a, G, B, A> {
			CoyonedaExplicit {
				fb: nat.transform(self.fb),
				func: self.func,
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
		/// let result = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
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

		/// Convert this `CoyonedaExplicit` into a [`Coyoneda`](crate::types::Coyoneda),
		/// hiding the intermediate type `B` behind a trait object.
		///
		/// This is useful when you have finished building a fusion pipeline and need
		/// to pass the result into code that is generic over `Functor` via
		/// `CoyonedaBrand`.
		///
		/// Note: further `map` calls on the resulting `Coyoneda` do not fuse with
		/// the previously composed function; each adds a separate trait-object layer.
		#[document_signature]
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
		/// let explicit = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
		/// let coyo: Coyoneda<VecBrand, i32> = explicit.into_coyoneda();
		/// assert_eq!(coyo.lower(), vec![2, 3, 4]);
		/// ```
		pub fn into_coyoneda(self) -> crate::types::Coyoneda<'a, F, A> {
			crate::types::Coyoneda::new(self.func, self.fb)
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
			"The output type after applying the function."
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
		/// let ff = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2),
		/// ));
		/// let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32));
		/// let result = CoyonedaExplicit::apply::<RcFnBrand, _, _>(ff, fa).lower();
		/// assert_eq!(result, Some(10));
		/// ```
		pub fn apply<FnBrand: CloneableFn + 'a, Bf: 'a, C: 'a>(
			ff: CoyonedaExplicit<'a, F, Bf, <FnBrand as CloneableFn>::Of<'a, A, C>>,
			fa: Self,
		) -> CoyonedaExplicit<'a, F, C, C>
		where
			A: Clone,
			F: Semiapplicative, {
			CoyonedaExplicit::lift(F::apply::<FnBrand, A, C>(ff.lower(), fa.lower()))
		}

		/// Sequence a computation through a function that returns a `CoyonedaExplicit`.
		///
		/// Lowers this value to `F A`, binds via `F::bind` (where the closure lowers
		/// each returned `CoyonedaExplicit` to `F C`), then re-lifts the result.
		/// After the operation the fusion pipeline is reset: the result is a
		/// `CoyonedaExplicit` with the identity function and intermediate type `C`.
		///
		/// This is a fusion barrier: it calls `lower()` on `self`, materializing
		/// all accumulated maps before delegating to `F::bind`. Each
		/// `CoyonedaExplicit` returned by `f` is also lowered.
		#[document_signature]
		///
		#[document_type_parameters("The output type of the bound computation.")]
		///
		#[document_parameters(
			"The function to apply to each value, returning a new `CoyonedaExplicit`."
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
		/// let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32));
		/// let result = fa.bind(|x| CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(x * 2))).lower();
		/// assert_eq!(result, Some(10));
		/// ```
		pub fn bind<C: 'a>(
			self,
			f: impl Fn(A) -> CoyonedaExplicit<'a, F, C, C> + 'a,
		) -> CoyonedaExplicit<'a, F, C, C>
		where
			F: Functor + Semimonad, {
			CoyonedaExplicit::lift(F::bind(self.lower(), move |a| f(a).lower()))
		}

		/// Bind through the accumulated function directly, without an intermediate
		/// `F::map` call.
		///
		/// Unlike [`bind`](CoyonedaExplicit::bind), the callback `f` receives the
		/// mapped value (after the accumulated function is applied) and returns a
		/// raw `F::Of<'a, C>` directly. This avoids needing `F: Functor` and skips
		/// the intermediate traversal that `bind` performs via `self.lower()`.
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
		/// let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32)).map(|x| x * 2);
		/// let result = fa.flat_map(|x| Some(x + 1)).lower();
		/// assert_eq!(result, Some(11)); // (5 * 2) + 1
		/// ```
		pub fn flat_map<C: 'a>(
			self,
			f: impl Fn(A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, C> + 'a,
		) -> CoyonedaExplicit<'a, F, C, C>
		where
			F: Semimonad, {
			let func = self.func;
			CoyonedaExplicit::lift(F::bind(self.fb, move |b| f(func(b))))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The type of the values in the functor."
	)]
	impl<'a, F, A: 'a> CoyonedaExplicit<'a, F, A, A>
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
		/// let coyo = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(42));
		/// assert_eq!(coyo.lower(), Some(42));
		/// ```
		pub fn lift(fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>) -> Self {
			CoyonedaExplicit {
				fb: fa,
				func: Box::new(identity),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying pointed functor.",
		"The type of the value."
	)]
	impl<'a, F, A: 'a> CoyonedaExplicit<'a, F, A, A>
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
		/// let coyo = CoyonedaExplicit::<OptionBrand, _, _>::pure(42);
		/// assert_eq!(coyo.lower(), Some(42));
		/// ```
		pub fn pure(a: A) -> Self {
			Self::lift(F::pure(a))
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
		let coyo = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(42));
		assert_eq!(coyo.lower(), Some(42));
	}

	#[test]
	fn lift_lower_identity_none() {
		let coyo = CoyonedaExplicit::<OptionBrand, i32, i32>::lift(None);
		assert_eq!(coyo.lower(), None);
	}

	#[test]
	fn lift_lower_identity_vec() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3]);
		assert_eq!(coyo.lower(), vec![1, 2, 3]);
	}

	#[test]
	fn new_constructor() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _>::new(|x: i32| x * 2, vec![1, 2, 3]);
		assert_eq!(coyo.lower(), vec![2, 4, 6]);
	}

	#[test]
	fn new_is_equivalent_to_lift_then_map() {
		let f = |x: i32| x.to_string();
		let v = vec![1, 2, 3];

		let via_new = CoyonedaExplicit::<VecBrand, _, _>::new(f, v.clone()).lower();
		let via_lift_map = CoyonedaExplicit::<VecBrand, _, _>::lift(v).map(f).lower();

		assert_eq!(via_new, via_lift_map);
	}

	#[test]
	fn single_map_option() {
		let result = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5)).map(|x| x * 2).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn chained_maps_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
			.map(|x| x + 1)
			.map(|x| x * 2)
			.map(|x| x.to_string())
			.lower();
		assert_eq!(result, vec!["4", "6", "8"]);
	}

	#[test]
	fn functor_identity_law() {
		let result = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3]).map(identity).lower();
		assert_eq!(result, vec![1, 2, 3]);
	}

	#[test]
	fn functor_composition_law() {
		let f = |x: i32| x + 1;
		let g = |x: i32| x * 2;

		let left =
			CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3]).map(compose(f, g)).lower();

		let right = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3]).map(g).map(f).lower();

		assert_eq!(left, right);
	}

	#[test]
	fn many_chained_maps() {
		let mut coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![0i64]);
		for _ in 0 .. 100 {
			coyo = coyo.map(|x| x + 1);
		}
		assert_eq!(coyo.lower(), vec![100i64]);
	}

	#[test]
	fn map_on_none_stays_none() {
		let result = CoyonedaExplicit::<OptionBrand, _, _>::lift(None::<i32>)
			.map(|x| x + 1)
			.map(|x| x * 2)
			.lower();
		assert_eq!(result, None);
	}

	#[test]
	fn lift_lower_roundtrip_preserves_value() {
		let original = vec![10, 20, 30];
		let roundtrip = CoyonedaExplicit::<VecBrand, _, _>::lift(original.clone()).lower();
		assert_eq!(roundtrip, original);
	}

	// -- Pure tests --

	#[test]
	fn pure_option() {
		let coyo = CoyonedaExplicit::<OptionBrand, _, _>::pure(42);
		assert_eq!(coyo.lower(), Some(42));
	}

	#[test]
	fn pure_vec() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _>::pure(42);
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
		let coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![10, 20, 30]);
		let hoisted = coyo.hoist(VecToOption);
		assert_eq!(hoisted.lower(), Some(10));
	}

	#[test]
	fn hoist_preserves_accumulated_maps() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let hoisted = coyo.hoist(VecToOption);
		assert_eq!(hoisted.lower(), Some(10));
	}

	#[test]
	fn hoist_then_map() {
		let coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![5, 10, 15]);
		let hoisted = coyo.hoist(VecToOption).map(|x: i32| x.to_string());
		assert_eq!(hoisted.lower(), Some("5".to_string()));
	}

	// -- Fold tests --

	#[test]
	fn fold_map_on_lifted_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
			.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		assert_eq!(result, "123".to_string());
	}

	#[test]
	fn fold_map_on_mapped_vec() {
		let result = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
			.map(|x| x * 10)
			.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		assert_eq!(result, "102030".to_string());
	}

	#[test]
	fn fold_map_on_none_is_empty() {
		let result = CoyonedaExplicit::<OptionBrand, i32, i32>::lift(None)
			.map(|x| x + 1)
			.fold_map::<RcFnBrand, _>(|x: i32| x.to_string());
		assert_eq!(result, String::new());
	}

	// -- Conversion tests --

	#[test]
	fn into_coyoneda_preserves_semantics() {
		let explicit =
			CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3]).map(|x| x + 1).map(|x| x * 2);
		let coyo: Coyoneda<VecBrand, i32> = explicit.into_coyoneda();
		assert_eq!(coyo.lower(), vec![4, 6, 8]);
	}

	#[test]
	fn into_coyoneda_from_lift() {
		let explicit = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(42));
		let coyo: Coyoneda<OptionBrand, i32> = explicit.into_coyoneda();
		assert_eq!(coyo.lower(), Some(42));
	}

	// -- Apply tests --

	#[test]
	fn apply_some_to_some() {
		let ff =
			CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(cloneable_fn_new::<RcFnBrand, _, _>(
				|x: i32| x * 2,
			)));
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32));
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _>(ff, fa).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn apply_none_fn_to_some() {
		let ff = CoyonedaExplicit::<OptionBrand, _, _>::lift(
			None::<<RcFnBrand as CloneableFn>::Of<'_, i32, i32>>,
		);
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32));
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _>(ff, fa).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn apply_some_fn_to_none() {
		let ff =
			CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(cloneable_fn_new::<RcFnBrand, _, _>(
				|x: i32| x * 2,
			)));
		let fa = CoyonedaExplicit::<OptionBrand, i32, i32>::lift(None);
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _>(ff, fa).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn apply_vec_applies_each_fn_to_each_value() {
		let ff = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![
			cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1),
			cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 10),
		]);
		let fa = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![2i32, 3]);
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _>(ff, fa).lower();
		assert_eq!(result, vec![3, 4, 20, 30]);
	}

	#[test]
	fn apply_preserves_prior_maps_on_fa() {
		let ff =
			CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(cloneable_fn_new::<RcFnBrand, _, _>(
				|x: i32| x + 1,
			)));
		// Prior map on fa is composed and applied before apply delegates to F.
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32)).map(|x| x * 2);
		let result = CoyonedaExplicit::apply::<RcFnBrand, _, _>(ff, fa).lower();
		assert_eq!(result, Some(11)); // (5 * 2) + 1
	}

	// -- Bind tests --

	#[test]
	fn bind_some() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32));
		let result = fa.bind(|x| CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(x * 2))).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn bind_none_stays_none() {
		let fa = CoyonedaExplicit::<OptionBrand, i32, i32>::lift(None);
		let result = fa.bind(|x| CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(x * 2))).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn bind_returning_none_gives_none() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32));
		let result = fa.bind(|_| CoyonedaExplicit::<OptionBrand, i32, i32>::lift(None)).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn bind_vec_flat_maps() {
		let fa = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1i32, 2, 3]);
		let result = fa.bind(|x| CoyonedaExplicit::<VecBrand, _, _>::lift(vec![x, x * 10])).lower();
		assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
	}

	#[test]
	fn bind_uses_accumulated_maps() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(3i32)).map(|x| x * 2);
		let result = fa.bind(|x| CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(x + 1))).lower();
		assert_eq!(result, Some(7)); // (3 * 2) + 1
	}

	// -- flat_map tests --

	#[test]
	fn flat_map_option_some() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(5i32));
		let result = fa.flat_map(|x| Some(x * 2)).lower();
		assert_eq!(result, Some(10));
	}

	#[test]
	fn flat_map_option_none() {
		let fa = CoyonedaExplicit::<OptionBrand, i32, i32>::lift(None);
		let result = fa.flat_map(|x| Some(x * 2)).lower();
		assert_eq!(result, None);
	}

	#[test]
	fn flat_map_vec() {
		let fa = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1i32, 2, 3]).map(|x| x * 2);
		let result = fa.flat_map(|x| vec![x, x + 1]).lower();
		assert_eq!(result, vec![2, 3, 4, 5, 6, 7]);
	}

	#[test]
	fn flat_map_equivalent_to_bind() {
		let v = vec![1i32, 2, 3];

		let via_flat_map = CoyonedaExplicit::<VecBrand, _, _>::lift(v.clone())
			.map(|x| x + 1)
			.flat_map(|x| vec![x, x * 10])
			.lower();

		let via_bind = CoyonedaExplicit::<VecBrand, _, _>::lift(v)
			.map(|x| x + 1)
			.bind(|x| CoyonedaExplicit::<VecBrand, _, _>::lift(vec![x, x * 10]))
			.lower();

		assert_eq!(via_flat_map, via_bind);
	}

	#[test]
	fn flat_map_uses_accumulated_maps() {
		let fa = CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(3i32)).map(|x| x * 2);
		let result = fa.flat_map(|x| Some(x + 1)).lower();
		assert_eq!(result, Some(7)); // (3 * 2) + 1
	}
}
