//! The free functor for any type constructor in the HKT/Brand system.
//!
//! `Coyoneda` wraps a value of type `F B` alongside an accumulated function `B -> A`,
//! enabling deferred mapping. It provides a [`Functor`](crate::classes::Functor) instance
//! for any type constructor `F` with the appropriate [`Kind`](crate::kinds) signature,
//! even if `F` itself is not a `Functor`. The `Functor` bound on `F` is only required
//! when calling [`lower`](Coyoneda::lower).
//!
//! ## Map fusion
//!
//! Each call to [`map`](Coyoneda::map) wraps the previous value in a new layer that
//! stores the mapping function. At [`lower`](Coyoneda::lower) time, each layer applies
//! its function via `F::map`. For k chained maps, `lower` makes k calls to `F::map`.
//!
//! For true single-pass fusion on eager brands like [`VecBrand`](crate::brands::VecBrand),
//! compose functions before mapping:
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Single traversal: compose first, then map once
//! let result = map::<VecBrand, _, _>(
//! 	compose(|x: i32| x.to_string(), compose(|x| x * 2, |x: i32| x + 1)),
//! 	vec![1, 2, 3],
//! );
//! assert_eq!(result, vec!["4", "6", "8"]);
//! ```
//!
//! ## Comparison with PureScript
//!
//! This implementation is based on PureScript's
//! [`Data.Coyoneda`](https://github.com/purescript/purescript-free/blob/master/src/Data/Coyoneda.purs).
//! PureScript uses `Exists` for existential quantification; Rust uses layered trait
//! objects to hide the intermediate type at each mapping stage.
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
		fn lower(self: Box<Self>) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor;
	}

	// -- Base layer: wraps F<A> directly (identity mapping) --

	/// Base layer created by [`Coyoneda::lift`]. Wraps `F A` with no mapping.
	struct CoyonedaBase<'a, F, A: 'a>
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
		fn lower(self: Box<Self>) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			self.fa
		}
	}

	// -- Map layer: wraps an inner Coyoneda and adds a mapping function --

	/// Map layer created by [`Coyoneda::map`]. Stores the inner value and a function
	/// to apply on top of it at [`lower`](Coyoneda::lower) time.
	struct CoyonedaMapLayer<'a, F, B: 'a, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		inner: Box<dyn CoyonedaInner<'a, F, B> + 'a>,
		func: Box<dyn Fn(B) -> A + 'a>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the underlying type constructor.",
		"The input type of this layer's mapping function.",
		"The output type of this layer's mapping function."
	)]
	#[document_parameters("The map layer instance.")]
	impl<'a, F, B: 'a, A: 'a> CoyonedaInner<'a, F, A> for CoyonedaMapLayer<'a, F, B, A>
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
		fn lower(self: Box<Self>) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			let lowered = self.inner.lower();
			F::map(self.func, lowered)
		}
	}

	// -- Outer type --

	/// The free functor over a type constructor `F`.
	///
	/// `Coyoneda` provides a [`Functor`] instance for any type constructor `F` with the
	/// appropriate [`Kind`](crate::kinds) signature, even if `F` itself does not implement
	/// [`Functor`]. The `Functor` constraint is only needed when calling [`lower`](Coyoneda::lower).
	///
	/// See the [module documentation](self) for usage and performance notes.
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
		/// Lift a value of `F A` into `Coyoneda F A`.
		///
		/// This wraps the value directly with no mapping. O(1).
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
		pub fn lift(fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>) -> Self {
			Coyoneda(Box::new(CoyonedaBase {
				fa,
			}))
		}

		/// Lower the `Coyoneda` back to the underlying functor `F`.
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
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1).lower();
		///
		/// assert_eq!(result, vec![2, 3, 4]);
		/// ```
		pub fn lower(self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
		where
			F: Functor, {
			self.0.lower()
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
				func: Box::new(f),
			}))
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
		/// let mapped = map::<CoyonedaBrand<VecBrand>, _, _>(|x| x * 10, coyo);
		/// assert_eq!(mapped.lower(), vec![10, 20, 30]);
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
	impl<F: Functor + Foldable + 'static> Foldable for CoyonedaBrand<F> {
		/// Folds the `Coyoneda` by lowering to the underlying functor and delegating.
		///
		/// This first applies all accumulated mapping functions via [`lower`](Coyoneda::lower),
		/// then folds the resulting `F A` using `F`'s [`Foldable`] instance.
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
		/// let result = fold_map::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _>(|x: i32| x.to_string(), coyo);
		/// assert_eq!(result, "102030".to_string());
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			F::fold_map::<FnBrand, A, M>(func, fa.lower())
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
		let result = map::<CoyonedaBrand<VecBrand>, _, _>(identity, coyo).lower();
		assert_eq!(result, vec![1, 2, 3]);
	}

	#[test]
	fn functor_composition_law() {
		// map(compose(f, g), fa) = map(f, map(g, fa))
		let f = |x: i32| x + 1;
		let g = |x: i32| x * 2;

		let coyo1 = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let left = map::<CoyonedaBrand<VecBrand>, _, _>(compose(f, g), coyo1).lower();

		let coyo2 = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let right =
			map::<CoyonedaBrand<VecBrand>, _, _>(f, map::<CoyonedaBrand<VecBrand>, _, _>(g, coyo2))
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
			map::<CoyonedaBrand<OptionBrand>, _, _>(|x: i32| x.to_string(), coyo);
		assert_eq!(result.lower(), Some("10".to_string()));
	}

	#[test]
	fn lift_lower_roundtrip_preserves_value() {
		// lift followed by lower should be the identity
		let original = vec![10, 20, 30];
		let roundtrip = Coyoneda::<VecBrand, _>::lift(original.clone()).lower();
		assert_eq!(roundtrip, original);
	}

	// -- Foldable tests --

	#[test]
	fn fold_map_on_lifted_vec() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		let result =
			fold_map::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _>(|x: i32| x.to_string(), coyo);
		assert_eq!(result, "123".to_string());
	}

	#[test]
	fn fold_map_on_mapped_coyoneda() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 10);
		let result =
			fold_map::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _>(|x: i32| x.to_string(), coyo);
		assert_eq!(result, "102030".to_string());
	}

	#[test]
	fn fold_right_on_coyoneda() {
		let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x * 2);
		let result =
			fold_right::<RcFnBrand, CoyonedaBrand<VecBrand>, _, _>(|a: i32, b: i32| a + b, 0, coyo);
		assert_eq!(result, 12); // (1*2) + (2*2) + (3*2) = 2 + 4 + 6 = 12
	}

	#[test]
	fn fold_left_on_coyoneda() {
		let coyo = Coyoneda::<OptionBrand, _>::lift(Some(5)).map(|x| x + 1);
		let result = fold_left::<RcFnBrand, CoyonedaBrand<OptionBrand>, _, _>(
			|acc: i32, a: i32| acc + a,
			10,
			coyo,
		);
		assert_eq!(result, 16); // 10 + (5 + 1) = 16
	}

	#[test]
	fn fold_map_on_none_is_empty() {
		let coyo = Coyoneda::<OptionBrand, i32>::lift(None).map(|x| x + 1);
		let result =
			fold_map::<RcFnBrand, CoyonedaBrand<OptionBrand>, _, _>(|x: i32| x.to_string(), coyo);
		assert_eq!(result, String::new());
	}
}
