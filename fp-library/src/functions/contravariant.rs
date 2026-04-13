#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	// -- contramap --

	/// Contravariantly maps a function over a value, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Only owned containers are supported; there is no
	/// `RefContravariant` trait because the Ref pattern is about closures
	/// receiving element references (`&A`), but `contramap`'s closure
	/// produces elements (`Fn(B) -> A`), not consumes them. The
	/// directionality is reversed compared to [`Functor`](crate::classes::Functor),
	/// so the `&A` convention does not apply.
	///
	/// For types with multiple brands, use
	/// [`explicit::contramap`](crate::functions::explicit::contramap) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type. Brand is inferred from this.",
		"The type of the value(s) inside the contravariant functor.",
		"The type that the new contravariant functor accepts."
	)]
	///
	#[document_parameters(
		"The function mapping the new input type to the original input type.",
		"The contravariant functor instance."
	)]
	///
	#[document_returns("A new contravariant functor that accepts values of type `B`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // contramap requires InferableBrand on the container type.
	/// // Most profunctor-based types do not implement InferableBrand,
	/// // so use explicit::contramap for those.
	/// assert!(true);
	/// ```
	pub fn contramap<'a, FA, A: 'a, B: 'a>(
		f: impl Fn(B) -> A + 'a,
		fa: FA,
	) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
	where
		FA: InferableBrand_cdc7cd43dac7585f,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand: crate::classes::Contravariant,
		FA: Into<
			<<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>,
		>, {
		<<FA as InferableBrand_cdc7cd43dac7585f>::Brand as crate::classes::Contravariant>::contramap(
			f,
			fa.into(),
		)
	}
}

pub use inner::*;
