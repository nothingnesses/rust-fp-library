//! Macros for generating higher-kinded type traits and implementations.
//!
//! These macros provide a systematic way to generate traits for different kind arities,
//! allowing the library to work with types of varying complexity from concrete types
//! (kind `*`) to type constructors (kinds `* -> *`, `* -> * -> *`, etc.).

/// Generates a [`Kind` trait][crate::hkt::kinds] of a specific arity.
///
/// This macro creates traits that represent type-level applications for different kind arities.
/// Each generated trait has an `Output` associated type that represents the concrete type
/// produced when the brand is applied to the appropriate type parameters.
///
/// # Parameters
/// * `kind_trait_name`: Trait name (e.g., `Kind_L0_T1`).
/// * `lifetimes`: Tuple of lifetime parameters (e.g., `('a, 'b)`).
/// * `types`: Tuple of type parameters with optional bounds (e.g., `(A, B: 'a)`).
/// * `output_bounds`: Tuple containing bounds for the `Output` associated type (e.g., `(: 'a)` or `()`).
/// * `kind_signature`: Kind signature (e.g., `"* -> *"`).
///
/// # Limitations
/// * **No `where` Clauses:** No support for `where` clauses.
#[macro_export]
macro_rules! make_trait_kind {
	(
		$kind_trait_name:ident,
		$lifetimes:tt,
		$types:tt,
		$output_bounds:tt,
		$doc_string:literal
	) => {
		make_trait_kind!(
			@impl $kind_trait_name,
			$lifetimes,
			$types,
			$output_bounds,
			$doc_string
		);
	};

	(
		@impl $kind_trait_name:ident,
		(),
		(),
		($($output_bounds:tt)*),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub trait $kind_trait_name {
			type Output $($output_bounds)*;
		}
	};

	(
		@impl $kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		(),
		($($output_bounds:tt)*),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub trait $kind_trait_name {
			type Output<$($lifetimes),*> $($output_bounds)*;
		}
	};

	(
		@impl $kind_trait_name:ident,
		(),
		($($types:ident $(: $($type_bounds:tt)+)?),+),
		($($output_bounds:tt)*),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub trait $kind_trait_name {
			type Output<$($types $(: $($type_bounds)+)?),*> $($output_bounds)*;
		}
	};

	(
		@impl $kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		($($types:ident $(: $($type_bounds:tt)+)?),+),
		($($output_bounds:tt)*),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub trait $kind_trait_name {
			type Output<$($lifetimes),*, $($types $(: $($type_bounds)+)?),*> $($output_bounds)*;
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
/// * `apply_alias_name`: Type alias name (e.g., `Apply_L0_T1`).
/// * `kind_trait_name`: Trait name (e.g., `Kind_L0_T1`).
/// * `lifetimes`: Tuple of lifetime parameters (e.g., `('a, 'b)`).
/// * `types`: Tuple of type parameters with optional bounds (e.g., `(A, B: 'a)`).
/// * `doc_string`: Documentation string.
///
/// # Limitations
/// * **No `where` Clauses:** No support for `where` clauses.
#[macro_export]
macro_rules! make_type_apply {
	(
		$apply_alias_name:ident,
		$kind_trait_name:ident,
		$lifetimes:tt,
		$types:tt,
		$doc_string:literal
	) => {
		make_type_apply!(
			@impl $apply_alias_name,
			$kind_trait_name,
			$lifetimes,
			$types,
			$doc_string
		);
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		(),
		(),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub type $apply_alias_name<Brand> = <Brand as $kind_trait_name>::Output;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		(),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub type $apply_alias_name<$($lifetimes),*, Brand> = <Brand as $kind_trait_name>::Output<$($lifetimes),*>;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		(),
		($($types:ident $(: $($type_bounds:tt)+)?),+),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub type $apply_alias_name<Brand $(, $types $(: $($type_bounds)+)?)*> = <Brand as $kind_trait_name>::Output<$($types),*>;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		($($types:ident $(: $($type_bounds:tt)+)?),+),
		$doc_string:literal
	) => {
		#[doc = $doc_string]
		#[allow(non_camel_case_types)]
		pub type $apply_alias_name<$($lifetimes),*, Brand $(, $types $(: $($type_bounds)+)?)*> = <Brand as $kind_trait_name>::Output<$($lifetimes),* $(, $types)*>;
	};
}
