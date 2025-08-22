//! Macros for generating higher-kinded type traits and implementations.
//!
//! These macros provide a systematic way to generate traits for different kind arities,
//! allowing the library to work with types of varying complexity from concrete types
//! (kind `*`) to type constructors (kinds `* -> *`, `* -> * -> *`, etc.).

/// Generates a [`KindN` trait][crate::hkt::kinds] of a specific arity and its corresponding blanket implementation.
///
/// This macro creates traits that represent type-level applications for different kind arities.
/// Each generated trait has an `Output` associated type that represents the concrete type
/// produced when the brand is applied to the appropriate type parameters.
///
/// # Parameters
///
/// - `$KindN`: The name of the trait to generate (e.g., `Kind0`, `Kind1`, `Kind2`).
/// - `$ApplyN`: The corresponding type alias name (e.g., `Apply0`, `Apply1`, `Apply2`).
/// - `$kind_string`: A string representation of the kind (e.g., `"*"`, `"* -> *"`, `"* -> * -> *"`).
/// - `$Generics`: A tuple of generic type parameters (e.g., `()`, `(A)`, `(A, B)`).
#[macro_export]
macro_rules! make_trait_kind {
	(
		$KindN:ident,
		$kind_string:literal,
		()
	) => {
		make_trait_kind!(@impl $KindN, $kind_string,);
	};
	(
		$KindN:ident,
		$kind_string:literal,
		($($Generics:ident),+)
	) => {
		make_trait_kind!(@impl $KindN, $kind_string, <$($Generics),+>);
	};
	(
		@impl $KindN:ident,
		$kind_string:literal,
		$($output_generics:tt)*
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub trait $KindN {
			type Output $($output_generics)*;
		}
	};
}

/// Generates an [`ApplyN` type alias][crate::hkt::apply] of a specific arity.
///
/// This macro creates type aliases that simplify the usage of kind traits by providing
/// a more convenient syntax for type applications. These aliases are used throughout
/// the library to make type signatures more readable.
///
/// # Parameters
///
/// - `$KindN`: The kind trait name (e.g., `Kind0`, `Kind1`, `Kind2`).
/// - `$ApplyN`: The name of the type alias to generate (e.g., `Apply0`, `Apply1`, `Apply2`).
/// - `$kind_string`: A string representation of the kind (e.g., `"*"`, `"* -> *"`, `"* -> * -> *"`).
/// - `$Generics`: A tuple of generic type parameters (e.g., `()`, `(A)`, `(A, B)`).
#[macro_export]
macro_rules! make_type_apply {
    (
        $KindN:ident,
        $ApplyN:ident,
        $kind_string:literal,
        ()
    ) => {
        make_type_apply!(@impl $KindN, $ApplyN, $kind_string, ());
    };
    (
        $KindN:ident,
        $ApplyN:ident,
        $kind_string:literal,
        ($($Generics:ident),+)
    ) => {
        make_type_apply!(@impl $KindN, $ApplyN, $kind_string, ($($Generics),+));
    };
    (
        @impl $KindN:ident,
        $ApplyN:ident,
        $kind_string:literal,
        ($($Generics:ident),*)
    ) => {
        #[doc = concat!(
            "Alias for [types][crate::types] of kind `",
            $kind_string,
            "`."
        )]
        pub type $ApplyN<Brand $(, $Generics)*> = <Brand as $KindN>::Output<$($Generics),*>;
    };
}
