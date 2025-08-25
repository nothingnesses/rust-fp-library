//! Macros for generating higher-kinded type traits and implementations.
//!
//! These macros provide a systematic way to generate traits for different kind arities,
//! allowing the library to work with types of varying complexity from concrete types
//! (kind `*`) to type constructors (kinds `* -> *`, `* -> * -> *`, etc.).

/// Generates a [`Kind` trait][crate::hkt::kinds] of a specific arity and its corresponding blanket implementation.
///
/// This macro creates traits that represent type-level applications for different kind arities.
/// Each generated trait has an `Output` associated type that represents the concrete type
/// produced when the brand is applied to the appropriate type parameters.
///
/// # Parameters
/// * `kind_trait_name`: Trait name (e.g., `Kind0L1T`).
/// * `lifetimes`: Tuple of lifetime parameters (e.g., `('a, 'b)`).
/// * `types`: Tuple of type parameters (e.g., `(A, B)`).
/// * `kind_signature`: Kind signature (e.g., `"* -> *"`).
#[macro_export]
macro_rules! make_trait_kind {
	(
		$kind_trait_name:ident,
		$lifetimes:tt,
		$types:tt,
		$kind_signature:literal
	) => {
		make_trait_kind!(
			@impl $kind_trait_name,
			$lifetimes,
			$types,
			$kind_signature
		);
	};

	(
		@impl $kind_trait_name:ident,
		(),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output;
		}
	};

	(
		@impl $kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output<$($lifetimes),*>;
		}
	};

	(
		@impl $kind_trait_name:ident,
		(),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output<$($types),*>;
		}
	};

	(
		@impl $kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output<$($lifetimes),*, $($types),*>;
		}
	};
}

/// Generates an [`Apply` type alias][crate::hkt::apply] of a specific arity.
///
/// This macro creates type aliases that simplify the usage of kind traits by providing
/// a more convenient syntax for type applications. These aliases are used throughout
/// the library to make type signatures more readable.
///
/// # Parameters
/// * `apply_alias_name`: Type alias name (e.g., `Apply0L1T`).
/// * `kind_trait_name`: Trait name (e.g., `Kind0L1T`).
/// * `lifetimes`: Tuple of lifetime parameters (e.g., `('a, 'b)`).
/// * `types`: Tuple of type parameters (e.g., `(A, B)`).
/// * `kind_signature`: Kind signature (e.g., `"* -> *"`).
#[macro_export]
macro_rules! make_type_apply {
	(
		$apply_alias_name:ident,
		$kind_trait_name:ident,
		$lifetimes:tt,
		$types:tt,
		$kind_signature:literal
	) => {
		make_type_apply!(
			@impl $apply_alias_name,
			$kind_trait_name,
			$lifetimes,
			$types,
			$kind_signature
		);
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		(),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<Brand> = <Brand as $kind_trait_name>::Output;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<$($lifetimes),*, Brand> = <Brand as $kind_trait_name>::Output<$($lifetimes),*>;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		(),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<Brand $(, $types)*> = <Brand as $kind_trait_name>::Output<$($types),*>;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<$($lifetimes),*, Brand $(, $types)*> = <Brand as $kind_trait_name>::Output<$($lifetimes),* $(, $types)*>;
	};
}
