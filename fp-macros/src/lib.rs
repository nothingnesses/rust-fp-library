//! Procedural macros for the [`fp-library`](https://docs.rs/fp-library/latest/fp_library/) crate.
//!
//! This crate provides macros for generating and working with Higher-Kinded Type (HKT) traits.

pub(crate) mod analysis; // Type and trait analysis
pub(crate) mod codegen; // Code generation (includes re-exports)
pub(crate) mod conversion; // Hindley-Milner type conversion
pub(crate) mod core; // Core infrastructure (config, error, result)
pub(crate) mod documentation; // Documentation generation macros
pub(crate) mod hkt; // Higher-Kinded Type macros
pub(crate) mod resolution; // Type resolution
pub(crate) mod support; // Support utilities (attributes, syntax, validation, errors)

#[cfg(test)]
mod property_tests;

use crate::core::ToCompileError;
use codegen::{FunctionFormatter, ReExportInput, TraitFormatter, generate_re_exports_worker};
use documentation::{
	document_module_worker, document_parameters_worker, document_signature_worker,
	document_type_parameters_worker,
};
use hkt::{
	ApplyInput, AssociatedTypes, ImplKindInput, apply_worker, generate_name, impl_kind_worker,
	trait_kind_worker,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Generates the name of a `Kind` trait based on its signature.
///
/// This macro takes a list of associated type definitions, similar to a trait definition.
///
/// ### Syntax
///
/// ```ignore
/// Kind!(
///     type AssocName<Params>: Bounds;
///     // ...
/// )
/// ```
///
/// ### Parameters
///
/// * `Associated Types`: A list of associated type definitions (e.g., `type Of<T>;`) that define the signature of the Kind.
///
/// ### Generates
///
/// The name of the generated `Kind` trait (e.g., `Kind_0123456789abcdef`).
/// The name is deterministic and based on a hash of the signature.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// let name = Kind!(type Of<T>;);
///
/// // Expanded code
/// let name = Kind_...; // e.g., Kind_a1b2c3d4e5f67890
/// ```
///
/// ```ignore
/// // Invocation
/// let name = Kind!(type Of<'a, T: Display>: Debug;);
///
/// // Expanded code
/// let name = Kind_...; // Unique hash based on signature
/// ```
///
/// ```ignore
/// // Invocation
/// let name = Kind!(
///     type Of<T>;
///     type SendOf<T>: Send;
/// );
///
/// // Expanded code
/// let name = Kind_...; // Unique hash based on signature
/// ```
///
/// ### Limitations
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
	let input = parse_macro_input!(input as AssociatedTypes);
	let name = match generate_name(&input) {
		Ok(name) => name,
		Err(e) => return e.to_compile_error().into(),
	};
	quote!(#name).into()
}

/// Defines a new `Kind` trait.
///
/// This macro generates a trait definition for a Higher-Kinded Type signature.
///
/// ### Syntax
///
/// ```ignore
/// trait_kind!(
///     type AssocName<Params>: Bounds;
///     // ...
/// )
/// ```
///
/// ### Parameters
///
/// * `Associated Types`: A list of associated type definitions (e.g., `type Of<T>;`) that define the signature of the Kind.
///
/// ### Generates
///
/// A public trait definition with a unique name derived from the signature (format: `Kind_{hash}`).
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// trait_kind!(type Of<T>;);
///
/// // Expanded code
/// pub trait Kind_... { // e.g., Kind_a1b2c3d4e5f67890
///     type Of<T>;
/// }
/// ```
///
/// ```ignore
/// // Invocation
/// trait_kind!(type Of<'a, T: Display>: Debug;);
///
/// // Expanded code
/// pub trait Kind_... {
///     type Of<'a, T: Display>: Debug;
/// }
/// ```
///
/// ```ignore
/// // Invocation
/// trait_kind!(
///     type Of<T>;
///     type SendOf<T>: Send;
/// );
///
/// // Expanded code
/// pub trait Kind_... {
///     type Of<T>;
///     type SendOf<T>: Send;
/// }
/// ```
#[proc_macro]
pub fn trait_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as AssociatedTypes);
	match trait_kind_worker(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Implements a `Kind` trait for a brand.
///
/// This macro simplifies the implementation of a generated `Kind` trait for a specific
/// brand type. It infers the correct `Kind` trait to implement based on the signature
/// of the associated types provided in the block.
///
/// The signature (names, parameters, and bounds) of the associated types must match
/// the definition used in [`trait_kind!`] or [`Kind!`] to ensure the correct trait is implemented.
///
/// ### Syntax
///
/// ```ignore
/// impl_kind! {
///     // Optional impl generics
///     impl<Generics> for BrandType
///     // Optional where clause
///     where Bounds
///     {
///         type AssocName<Params> = ConcreteType;
///         // ... more associated types
///     }
/// }
/// ```
///
/// ### Parameters
///
/// * `Generics`: Optional generic parameters for the implementation.
/// * `BrandType`: The brand type to implement the Kind for.
/// * `Bounds`: Optional where clause bounds.
/// * `Associated Types`: The associated type assignments (e.g., `type Of<A> = Option<A>;`).
///
/// ### Generates
///
/// An implementation of the appropriate `Kind` trait for the brand.
///
/// ### Attributes
///
/// Inside the `impl_kind!` block, you can use documentation-specific attributes on associated types:
///
/// * `#[document_default]`: Marks this associated type as the default for resolving bare `Self` in
///   the generated documentation for this brand within the module.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// impl_kind! {
///     for OptionBrand {
///         #[document_default]
///         type Of<A> = Option<A>;
///     }
/// }
///
/// // Expanded code
/// impl Kind_... for OptionBrand { // e.g., Kind_a1b2c3d4e5f67890
///     type Of<A> = Option<A>;
/// }
/// ```
///
/// ```ignore
/// // Invocation
/// impl_kind! {
///     impl<E> for ResultBrand<E> {
///         type Of<A> = Result<A, E>;
///     }
/// }
///
/// // Expanded code
/// impl<E> Kind_... for ResultBrand<E> {
///     type Of<A> = Result<A, E>;
/// }
/// ```
///
/// ```ignore
/// // Invocation
/// impl_kind! {
///     impl<E> for MyBrand<E> where E: Clone {
///         type Of<A> = MyType<A, E>;
///         type SendOf<A> = MySendType<A, E>;
///     }
/// }
///
/// // Expanded code
/// impl<E> Kind_... for MyBrand<E> where E: Clone {
///     type Of<A> = MyType<A, E>;
///     type SendOf<A> = MySendType<A, E>;
/// }
/// ```
///
/// ```ignore
/// // Invocation
/// // Corresponds to: trait_kind!(type Of<T: Display>;);
/// impl_kind! {
///     for DisplayBrand {
///         // Bounds here are used to infer the correct `Kind` trait name
///         type Of<T: Display> = DisplayType<T>;
///     }
/// }
///
/// // Expanded code
/// impl Kind_... for DisplayBrand {
///     type Of<T: Display> = DisplayType<T>;
/// }
/// ```
#[proc_macro]
pub fn impl_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ImplKindInput);
	match impl_kind_worker(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Applies a brand to type arguments.
///
/// This macro projects a brand type to its concrete type using the appropriate
/// `Kind` trait. It uses a syntax that mimics a fully qualified path, where the
/// `Kind` trait is specified by its signature.
///
/// ### Syntax
///
/// ```ignore
/// Apply!(<Brand as Kind!( KindSignature )>::AssocType<Args>)
/// ```
///
/// ### Parameters
///
/// * `Brand`: The brand type (e.g., `OptionBrand`).
/// * `KindSignature`: A list of associated type definitions defining the `Kind` trait schema.
/// * `AssocType`: The associated type to project (e.g., `Of`).
/// * `Args`: The concrete arguments to apply.
///
/// ### Generates
///
/// The concrete type resulting from applying the brand to the arguments.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// // Applies MyBrand to lifetime 'static and type String.
/// type Concrete = Apply!(<MyBrand as Kind!( type Of<'a, T>; )>::Of<'static, String>);
///
/// // Expanded code
/// type Concrete = <MyBrand as Kind_...>::Of<'static, String>;
/// ```
///
/// ```ignore
/// // Invocation
/// // Applies MyBrand to a generic type T with bounds.
/// type Concrete = Apply!(<MyBrand as Kind!( type Of<T: Clone>; )>::Of<T>);
///
/// // Expanded code
/// type Concrete = <MyBrand as Kind_...>::Of<T>;
/// ```
///
/// ```ignore
/// // Invocation
/// // Complex example with lifetimes, types, and output bounds.
/// type Concrete = Apply!(<MyBrand as Kind!( type Of<'a, T: Clone + Debug>: Display; )>::Of<'a, T>);
///
/// // Expanded code
/// type Concrete = <MyBrand as Kind_...>::Of<'a, T>;
/// ```
///
/// ```ignore
/// // Invocation
/// // Use a custom associated type for projection.
/// type Concrete = Apply!(<MyBrand as Kind!( type Of<T>; type SendOf<T>; )>::SendOf<T>);
///
/// // Expanded code
/// type Concrete = <MyBrand as Kind_...>::SendOf<T>;
/// ```
#[proc_macro]
#[allow(non_snake_case)]
pub fn Apply(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ApplyInput);
	match apply_worker(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Generates re-exports for all public free functions in a directory.
///
/// This macro scans the specified directory for Rust files, parses them to find public free functions,
/// and generates `pub use` statements for them. It supports aliasing to resolve name conflicts.
///
/// ### Syntax
///
/// ```ignore
/// generate_function_re_exports!("path/to/directory", {
///     original_name: aliased_name,
///     ...
/// })
/// ```
///
/// ### Parameters
///
/// * `path/to/directory`: The path to the directory containing the modules, relative to the crate root.
/// * `aliases`: A map of function names to their desired aliases.
///
/// ### Generates
///
/// `pub use` statements for each public function found in the directory.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// generate_function_re_exports!("src/classes", {
///     identity: category_identity,
///     new: fn_new,
/// });
///
/// // Expanded code
/// pub use src::classes::category::identity as category_identity;
/// pub use src::classes::function::new as fn_new;
/// // ... other re-exports
/// ```
#[proc_macro]
pub fn generate_function_re_exports(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ReExportInput);
	generate_re_exports_worker(&input, &FunctionFormatter).into()
}

/// Generates re-exports for all public traits in a directory.
///
/// This macro scans the specified directory for Rust files, parses them to find public traits,
/// and generates `pub use` statements for them.
///
/// ### Syntax
///
/// ```ignore
/// generate_trait_re_exports!("path/to/directory", {
///     original_name: aliased_name,
///     ...
/// })
/// ```
///
/// ### Parameters
///
/// * `path/to/directory`: The path to the directory containing the modules, relative to the crate root.
/// * `aliases`: A map of trait names to their desired aliases (optional).
///
/// ### Generates
///
/// `pub use` statements for each public trait found in the directory.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// generate_trait_re_exports!("src/classes", {});
///
/// // Expanded code
/// pub use src::classes::functor::Functor;
/// pub use src::classes::monad::Monad;
/// // ... other re-exports
/// ```
#[proc_macro]
pub fn generate_trait_re_exports(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ReExportInput);
	generate_re_exports_worker(&input, &TraitFormatter).into()
}

/// Generates a Hindley-Milner style type signature for a function.
///
/// This macro analyzes the function signature and generates a documentation comment
/// containing the corresponding Hindley-Milner type signature.
///
/// When used within a module annotated with [`#[document_module]`](macro@document_module),
/// it automatically resolves `Self` and associated types based on the module's projection map.
///
/// ### Syntax
///
/// ```ignore
/// #[document_signature]
/// pub fn function_name<Generics>(params) -> ReturnType { ... }
/// ```
///
/// When applying this macro to a method inside a trait, you can provide the trait name
/// as an argument to correctly generate the `Trait self` constraint.
///
/// ### Generates
///
/// A documentation comment with the generated signature, prepended to the function definition.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// #[document_signature]
/// pub fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B> { ... }
///
/// // Expanded code
/// /// `forall f a b. Functor f => (a -> b, f a) -> f b`
/// pub fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B> { ... }
/// ```
///
/// ```ignore
/// // Invocation
/// #[document_signature]
/// pub fn foo(x: impl Iterator<Item = String>) -> i32 { ... }
///
/// // Expanded code
/// /// `iterator -> i32`
/// pub fn foo(x: impl Iterator<Item = String>) -> i32 { ... }
/// ```
///
/// ```ignore
/// // Invocation
/// trait Functor {
///     #[document_signature]
///     fn map<A, B>(f: impl Fn(A) -> B, fa: Self::Of<A>) -> Self::Of<B>;
/// }
///
/// // Expanded code
/// trait Functor {
///     /// `forall self a b. Functor self => (a -> b, self a) -> self b`
///     fn map<A, B>(f: impl Fn(A) -> B, fa: Self::Of<A>) -> Self::Of<B>;
/// }
/// ```
///
/// ### Configuration
///
/// This macro can be configured via `Cargo.toml` under `[package.metadata.document_signature]`.
///
/// * `brand_mappings`: A map of brand struct names to their display names in the signature.
/// * `apply_macro_aliases`: A list of macro names that should be treated as `Apply!`.
/// * `ignored_traits`: A list of traits to ignore in the signature constraints.
///
/// Example:
/// ```toml
/// [package.metadata.document_signature]
/// brand_mappings = { "OptionBrand" = "Option", "VecBrand" = "Vec" }
/// apply_macro_aliases = ["MyApply"]
/// ignored_traits = ["Clone", "Debug"]
/// ```
#[proc_macro_attribute]
pub fn document_signature(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	match document_signature_worker(attr.into(), item.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Generates documentation for a function's type parameters.
///
/// This macro analyzes the function signature and generates a documentation comment
/// list based on the provided descriptions.
///
/// When used within a module annotated with [`#[document_module]`](macro@document_module),
/// it benefits from automatic `Self` resolution and is applied as part of the module-level
/// documentation pass.
///
/// ### Syntax
///
/// ```ignore
/// #[document_type_parameters(
///     "Description for first parameter",
///     ("OverriddenName", "Description for second parameter"),
///     ...
/// )]
/// pub fn function_name<Generics>(params) -> ReturnType { ... }
/// ```
///
/// ### Parameters
///
/// * `Descriptions`: A comma-separated list. Each entry can be either a string literal
///   or a tuple of two string literals `(Name, Description)`.
///
/// ### Generates
///
/// A list of documentation comments, one for each generic parameter, prepended to the
/// function definition.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// #[document_type_parameters(
///     "The type of the elements.",
///     ("E", "The error type.")
/// )]
/// pub fn map<T, ERR>(...) { ... }
///
/// // Expanded code
/// /// * `T`: The type of the elements.
/// /// * `E`: The error type.
/// pub fn map<T, ERR>(...) { ... }
/// ```
///
/// ### Constraints
///
/// * The number of arguments must exactly match the number of generic parameters
///   (including lifetimes, types, and const generics) in the function signature.
#[proc_macro_attribute]
pub fn document_type_parameters(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	match document_type_parameters_worker(attr.into(), item.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Generates documentation for a function's parameters.
///
/// This macro analyzes the function signature and generates a documentation comment
/// list based on the provided descriptions. It also handles curried return types.
///
/// ### Syntax
///
/// ```ignore
/// #[document_parameters(
///     "Description for first parameter",
///     ("overridden_name", "Description for second parameter"),
///     ...
/// )]
/// pub fn function_name(params) -> impl Fn(...) { ... }
/// ```
///
/// ### Parameters
///
/// * `Descriptions`: A comma-separated list. Each entry can be either a string literal
///   or a tuple of two string literals `(Name, Description)`.
///
/// ### Generates
///
/// A list of documentation comments, one for each parameter, prepended to the
/// function definition.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// #[document_parameters(
///     "The first input value.",
///     ("y", "The second input value.")
/// )]
/// pub fn foo(x: i32) -> impl Fn(i32) -> i32 { ... }
///
/// // Expanded code
/// /// * `x`: The first input value.
/// /// * `y`: The second input value.
/// pub fn foo(x: i32) -> impl Fn(i32) -> i32 { ... }
/// ```
///
/// ### Constraints
///
/// * The number of arguments must exactly match the number of function parameters
///   (excluding `self` but including parameters from curried return types).
///
/// ### Configuration
///
/// This macro can be configured via `Cargo.toml` under `[package.metadata.document_signature]`.
///
/// * `apply_macro_aliases`: A list of macro names that should be treated as `Apply!` for curried parameter extraction.
///
/// Example:
/// ```toml
/// [package.metadata.document_signature]
/// apply_macro_aliases = ["MyApply"]
/// ```
#[proc_macro_attribute]
pub fn document_parameters(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	match document_parameters_worker(attr.into(), item.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Orchestrates documentation generation for an entire module.
///
/// This macro provides a centralized way to handle documentation for Higher-Kinded Type (HKT)
/// implementations. It performs a two-pass analysis of the module:
///
/// 1. **Context Extraction**: It scans for `impl_kind!` invocations and standard `impl` blocks
///    to build a comprehensive mapping of associated types (a "projection map").
/// 2. **Documentation Generation**: It processes all methods annotated with [`#[document_signature]`](macro@document_signature)
///    or [`#[document_type_parameters]`](macro@document_type_parameters), resolving `Self` and associated types
///    using the collected context.
///
/// ### Syntax
///
/// Due to inner macro attributes being unstable, use the following wrapper pattern:
///
/// ```ignore
/// #[fp_macros::document_module]
/// mod inner {
///     // ... module content ...
/// }
/// pub use inner::*;
/// ```
///
/// ### Generates
///
/// In-place replacement of [`#[document_signature]`](macro@document_signature) and
/// [`#[document_type_parameters]`](macro@document_type_parameters) attributes with generated documentation
/// comments. It also resolves `Self` and `Self::AssocType` references to their concrete
/// types based on the module's projection map.
///
/// ### Attributes
///
/// The macro supports several documentation-specific attributes for configuration:
///
/// * `#[document_default]`: (Used inside `impl` or `impl_kind!`) Marks an associated type as the
///   default to use when resolving bare `Self` references.
/// * `#[document_use = "AssocName"]`: (Used on `impl` or `fn`) Explicitly specifies which
///   associated type definition to use for resolution within that scope.
///
/// ### Hierarchical Configuration
///
/// When resolving the concrete type of `Self`, the macro follows this precedence:
///
/// 1. **Method Override**: `#[document_use = "AssocName"]` on the method.
/// 2. **Impl Block Override**: `#[document_use = "AssocName"]` on the `impl` block.
/// 3. **(Type, Trait)-Scoped Default**: `#[document_default]` on the associated type definition
///    in a trait `impl` block.
/// 4. **Module Default**: `#[document_default]` on the associated type definition in `impl_kind!`.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// #[fp_macros::document_module]
/// mod inner {
///     use super::*;
///
///     impl_kind! {
///         for MyBrand {
///             #[document_default]
///             type Of<'a, T: 'a>: 'a = MyType<T>;
///         }
///     }
///
///     impl Functor for MyBrand {
///         #[document_signature]
///         fn map<'a, A: 'a, B: 'a, Func>(
///             f: Func,
///             fa: Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
///         ) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
///         where
///             Func: Fn(A) -> B + 'a
///         {
///             todo!()
///         }
///     }
/// }
/// pub use inner::*;
///
/// // Expanded code
/// mod inner {
///     use super::*;
///
///     // ... generated Kind implementations ...
///
///     impl Functor for MyBrand {
///         /// `forall a b. (a -> b, MyType a) -> MyType b`
///         fn map<'a, A: 'a, B: 'a, Func>(
///             f: Func,
///             fa: Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
///         ) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
///         where
///             Func: Fn(A) -> B + 'a
///         {
///             todo!()
///         }
///     }
/// }
/// pub use inner::*;
/// ```
#[proc_macro_attribute]
pub fn document_module(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	match document_module_worker(attr.into(), item.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}
