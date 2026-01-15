//! Procedural macros for the `fp-library` crate.
//!
//! This crate provides macros for generating and working with Higher-Kinded Type (HKT) traits.
//! It includes:
//! - `Kind!`: Generates the name of a Kind trait based on its signature.
//! - `def_kind!`: Defines a new Kind trait.

use apply::{ApplyInput, apply_impl};
use def_kind::def_kind_impl;
use generate::generate_name;
use impl_kind::{ImplKindInput, impl_kind_impl};
use parse::KindInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub(crate) mod apply;
pub(crate) mod canonicalize;
pub(crate) mod def_kind;
pub(crate) mod generate;
pub(crate) mod impl_kind;
pub(crate) mod parse;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;

/// Generates the name of a Kind trait based on its signature.
///
/// This macro takes three parenthesized groups representing the signature:
/// 1. **Lifetimes**: A comma-separated list of lifetimes (e.g., `('a, 'b)`).
/// 2. **Types**: A comma-separated list of types with optional bounds (e.g., `(T, U: Display)`).
/// 3. **Output Bounds**: A `+`-separated list of bounds on the output type (e.g., `(Display + Clone)`).
///
/// # Example
///
/// ```ignore
/// // Generates the name for a Kind with:
/// // - 1 lifetime ('a)
/// // - 1 type parameter (T) bounded by Display and Clone
/// // - Output type bounded by Debug
/// let name = Kind!(('a), (T: Display + Clone), (Debug));
/// ```
///
/// # Limitations
///
/// Due to Rust syntax restrictions, this macro cannot be used directly in positions where a
/// concrete path is expected by the parser, such as:
/// * Supertrait bounds: `trait MyTrait: Kind!(...) {}` (Invalid)
/// * Type aliases: `type MyKind = Kind!(...);` (Invalid)
/// * Trait aliases: `trait MyKind = Kind!(...);` (Invalid)
///
/// In these cases, you must use the generated name directly (e.g., `Kind_...`).
#[proc_macro]
#[allow(non_snake_case)]
pub fn Kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as KindInput);
	let name = generate_name(&input);
	quote!(#name).into()
}

/// Defines a new Kind trait.
///
/// This macro generates a trait definition for a Higher-Kinded Type signature.
/// It takes the same three arguments as `Kind!`:
/// 1. **Lifetimes**
/// 2. **Types**
/// 3. **Output Bounds**
///
/// The generated trait includes a single associated type `Of`.
///
/// # Example
///
/// ```ignore
/// // Defines a Kind trait for a signature with:
/// // - 1 lifetime ('a)
/// // - 1 type parameter (T) bounded by Display
/// // - Output type bounded by Debug
/// def_kind!(('a), (T: Display), (Debug));
/// ```
#[proc_macro]
pub fn def_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as KindInput);
	def_kind_impl(input).into()
}

/// Implements a Kind trait for a brand.
///
/// This macro simplifies the implementation of a generated Kind trait for a specific
/// brand type. It infers the correct Kind trait to implement based on the signature
/// of the associated type `Of`.
///
/// # Syntax
///
/// ```ignore
/// impl_kind! {
///     impl<GENERICS> for BrandType {
///         type Of<PARAMS> = ConcreteType;
///     }
/// }
/// ```
///
/// Or with where clause:
///
/// ```ignore
/// impl_kind! {
///     impl<E> for ResultBrand<E> where E: Debug {
///         type Of<A> = Result<A, E>;
///     }
/// }
/// ```
///
/// # Example
///
/// ```ignore
/// impl_kind! {
///     for OptionBrand {
///         type Of<A> = Option<A>;
///     }
/// }
/// ```
#[proc_macro]
pub fn impl_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ImplKindInput);
	impl_kind_impl(input).into()
}

/// Applies a brand to type arguments.
///
/// This macro projects a brand type to its concrete type using the appropriate
/// Kind trait. It supports named parameters for clarity.
///
/// # Parameters
///
/// * `brand`: The brand type (e.g., `OptionBrand`).
/// * `signature`: The signature of the Kind trait to use (lifetimes, types).
/// * `lifetimes`: (Optional) Lifetime arguments to apply.
/// * `types`: (Optional) Type arguments to apply.
///
/// # Example
///
/// ```ignore
/// // Applies MyBrand to lifetime 'static and type String.
/// // The signature specifies 1 lifetime and 1 type parameter.
/// type Concrete = Apply!(
///     brand: MyBrand,
///     signature: ('a, T),
///     lifetimes: ('static),
///     types: (String)
/// );
/// ```
#[proc_macro]
#[allow(non_snake_case)]
pub fn Apply(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ApplyInput);
	apply_impl(input).into()
}
