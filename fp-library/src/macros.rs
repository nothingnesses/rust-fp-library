//! Macros for generating higher-kinded type traits and implementations.
//!
//! These macros provide a systematic way to generate traits for different kind arities,
//! allowing the library to work with types of varying complexity from concrete types
//! (kind `*`) to type constructors (kinds `* -> *`, `* -> * -> *`, etc.).

/// Generates a [`Kind` trait][crate::hkt::kinds] of a specific arity.
///
/// This macro creates traits that represent type-level applications for different kind arities.
/// Each generated trait has an `Of` associated type that represents the concrete type
/// produced when the brand is applied to the appropriate type parameters.
///
/// # Parameters
/// * `kind_trait_name`: Trait name (e.g., `Kind_L0_T1`).
/// * `lifetimes`: Tuple of lifetime parameters (e.g., `('a, 'b)`).
/// * `types`: Tuple of type parameters with optional bounds (e.g., `(A, B: 'a)`).
/// * `output_bounds`: Tuple containing bounds for the `Of` associated type (e.g., `(: 'a)` or `()`).
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
			type Of $($output_bounds)*;
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
			type Of<$($lifetimes),*> $($output_bounds)*;
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
			type Of<$($types $(: $($type_bounds)+)?),*> $($output_bounds)*;
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
			type Of<$($lifetimes),*, $($types $(: $($type_bounds)+)?),*> $($output_bounds)*;
		}
	};
}

/// Applies a [`Brand`][crate::brands] to type parameters using a [`Kind` trait][crate::hkt::kinds] to obtain the concrete type.
///
/// This macro is the primary mechanism for resolving Higher-Kinded Types (HKT) in this library.
/// It projects the associated `Of` type from the `Kind` trait implementation for the given `Brand`.
///
/// # Parameters
///
/// * `$brand`: The brand type (e.g., `OptionBrand`, `VecBrand`).
/// * `$kind`: The kind trait corresponding to the brand's arity (e.g., `Kind_L0_T1`, `Kind_L1_T1_B0l0_Ol0`).
/// * `($($lifetimes:lifetime),*)`: A tuple of lifetime parameters. Use `()` if there are no lifetimes.
/// * `($($types:ty),*)`: A tuple of type parameters.
///
/// # Examples
///
/// ## 0 Lifetimes, 1 Type (Kind `* -> *`)
///
/// ```ignore
/// use fp_library::Apply;
/// use fp_library::brands::OptionBrand;
/// use fp_library::hkt::Kind_L0_T1;
///
/// // Equivalent to: Option<i32>
/// type OptInt = Apply!(OptionBrand, Kind_L0_T1, (), (i32));
/// ```
///
/// ## 1 Lifetime, 1 Type (Kind `' -> * -> *`)
///
/// ```ignore
/// use fp_library::Apply;
/// use fp_library::brands::IdentityBrand;
/// use fp_library::hkt::Kind_L1_T1_B0l0_Ol0;
///
/// // Equivalent to: Identity<'a, i32>
/// type IdInt<'a> = Apply!(IdentityBrand, Kind_L1_T1_B0l0_Ol0, ('a), (i32));
/// ```
#[macro_export]
macro_rules! Apply {
	// 0 lifetimes
	($brand:ty, $kind:ty, (), ($($types:ty),*)) => {
		<$brand as $kind>::Of<$($types),*>
	};

	// 1+ lifetimes
	($brand:ty, $kind:ty, ($($lifetimes:lifetime),+), ($($types:ty),*)) => {
		<$brand as $kind>::Of<$($lifetimes),+, $($types),*>
	};
}
