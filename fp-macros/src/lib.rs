#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

//! Procedural macros for the [`fp-library`](https://docs.rs/fp-library/latest/fp_library/) crate.
//!
//! This crate provides macros for generating and working with Higher-Kinded Type (HKT) traits.

pub(crate) mod a_do; // Applicative do-notation
pub(crate) mod analysis; // Type and trait analysis
pub(crate) mod codegen; // Code generation (includes re-exports)
pub(crate) mod core; // Core infrastructure (config, error, result)
pub(crate) mod documentation; // Documentation generation macros
pub(crate) mod hkt; // Higher-Kinded Type macros
pub(crate) mod hm; // Hindley-Milner type conversion
pub(crate) mod m_do; // Monadic do-notation
pub(crate) mod resolution; // Type resolution
pub(crate) mod support; // Support utilities (attributes, syntax, validation, errors)

#[cfg(test)]
mod property_tests;

use {
	crate::core::ToCompileError,
	a_do::a_do_worker,
	codegen::{
		FunctionFormatter,
		ReExportInput,
		TraitFormatter,
		generate_re_exports_worker,
	},
	documentation::{
		document_examples_worker,
		document_module_worker,
		document_parameters_worker,
		document_returns_worker,
		document_signature_worker,
		document_type_parameters_worker,
	},
	hkt::{
		ApplyInput,
		AssociatedTypes,
		ImplKindInput,
		apply_worker,
		generate_name,
		impl_kind_worker,
		kind_attr_worker,
		trait_kind_worker,
	},
	m_do::{
		DoInput,
		m_do_worker,
	},
	proc_macro::TokenStream,
	quote::quote,
	syn::parse_macro_input,
};

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
/// For supertrait bounds, use the [`kind`] attribute macro instead:
/// ```ignore
/// #[kind(type Of<'a, A: 'a>: 'a;)]
/// pub trait Functor { ... }
/// ```
///
/// For other positions, you must use the generated name directly (e.g., `Kind_...`).
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

/// Adds a `Kind` supertrait bound to a trait definition.
///
/// This attribute macro parses a Kind signature and adds the corresponding
/// `Kind_` trait as a supertrait bound, avoiding the need to reference
/// hash-based trait names directly.
///
/// ### Syntax
///
/// ```ignore
/// #[kind(type AssocName<Params>: Bounds;)]
/// pub trait MyTrait {
///     // ...
/// }
/// ```
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// #[kind(type Of<'a, A: 'a>: 'a;)]
/// pub trait Functor {
///     fn map<'a, A: 'a, B: 'a>(
///         f: impl Fn(A) -> B + 'a,
///         fa: Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
///     ) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
/// }
///
/// // Expanded code
/// pub trait Functor: Kind_cdc7cd43dac7585f {
///     // body unchanged
/// }
/// ```
///
/// ```ignore
/// // Works with existing supertraits
/// #[kind(type Of<'a, A: 'a>: 'a;)]
/// pub trait Monad: Applicative {
///     // Kind_ bound is appended: Monad: Applicative + Kind_...
/// }
/// ```
#[proc_macro_attribute]
pub fn kind(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	let attr = parse_macro_input!(attr as AssociatedTypes);
	let item = parse_macro_input!(item as syn::ItemTrait);
	match kind_attr_worker(attr, item) {
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
/// /// ### Type Signature
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
/// /// ### Type Signature
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
///     /// ### Type Signature
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

/// Generates documentation for type parameters.
///
/// This macro analyzes the item's signature (function, struct, enum, impl block, etc.)
/// and generates a documentation comment list based on the provided descriptions.
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
/// It can also be used on other items like `impl` blocks:
///
/// ```ignore
/// #[document_type_parameters("Description for T")]
/// impl<T> MyType<T> { ... }
/// ```
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
/// /// ### Type Parameters
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
/// It can also be used on `impl` blocks to provide a common description for the receiver (`self`)
/// parameter of methods within the block.
///
/// ### Syntax
///
/// For functions:
/// ```ignore
/// #[document_parameters(
///     "Description for first parameter",
///     ("overridden_name", "Description for second parameter"),
///     ...
/// )]
/// pub fn function_name(params) -> impl Fn(...) { ... }
/// ```
///
/// For `impl` blocks:
/// ```ignore
/// #[document_parameters("Description for receiver")]
/// impl MyType {
///     #[document_parameters]
///     pub fn method_with_receiver(&self) { ... }
///
///     #[document_parameters("Description for arg")]
///     pub fn method_with_args(&self, arg: i32) { ... }
/// }
/// ```
///
/// * `Descriptions`: A comma-separated list. Each entry can be either a string literal
///   or a tuple of two string literals `(Name, Description)`.
/// * For `impl` blocks: Exactly one string literal describing the receiver parameter.
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
/// /// ### Parameters
/// /// * `x`: The first input value.
/// /// * `y`: The second input value.
/// pub fn foo(x: i32) -> impl Fn(i32) -> i32 { ... }
/// ```
///
/// ```ignore
/// // Invocation on impl block
/// #[document_parameters("The list instance")]
/// impl<A> MyList<A> {
///     #[document_parameters("The element to push")]
///     pub fn push(&mut self, item: A) { ... }
/// }
///
/// // Expanded code
/// impl<A> MyList<A> {
///     /// ### Parameters
///     /// * `&mut self`: The list instance
///     /// * `item`: The element to push
///     pub fn push(&mut self, item: A) { ... }
/// }
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

/// Generates documentation for the return value of a function.
///
/// This macro adds a "Returns" section to the function's documentation.
///
/// ### Syntax
///
/// ```ignore
/// #[document_returns("Description of the return value.")]
/// pub fn foo() -> i32 { ... }
/// ```
///
/// ### Generates
///
/// A documentation comment describing the return value.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// #[document_returns("The sum of x and y.")]
/// pub fn add(x: i32, y: i32) -> i32 { ... }
///
/// // Expanded code
/// /// ### Returns
/// /// The sum of x and y.
/// pub fn add(x: i32, y: i32) -> i32 { ... }
/// ```
#[proc_macro_attribute]
pub fn document_returns(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	match document_returns_worker(attr.into(), item.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Inserts a `### Examples` heading and validates doc comment code blocks.
///
/// This attribute macro expands in-place to a `### Examples` heading. Example
/// code is written as regular doc comments using fenced code blocks after the
/// attribute. Every Rust code block must contain at least one assertion macro
/// invocation (e.g., `assert_eq!`, `assert!`).
///
/// ### Syntax
///
/// ```ignore
/// #[document_examples]
/// ///
/// /// ```
/// /// let result = add(1, 2);
/// /// assert_eq!(result, 3);
/// /// ```
/// pub fn add(x: i32, y: i32) -> i32 { ... }
/// ```
///
/// ### Generates
///
/// A `### Examples` heading is inserted at the attribute's position. The code
/// blocks in the doc comments are validated but not modified.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// #[document_examples]
/// ///
/// /// ```
/// /// let x = my_fn(1, 2);
/// /// assert_eq!(x, 3);
/// /// ```
/// pub fn my_fn(a: i32, b: i32) -> i32 { a + b }
///
/// // Expanded code
/// /// ### Examples
/// ///
/// /// ```
/// /// let x = my_fn(1, 2);
/// /// assert_eq!(x, 3);
/// /// ```
/// pub fn my_fn(a: i32, b: i32) -> i32 { a + b }
/// ```
///
/// ### Errors
///
/// * Arguments are provided to the attribute.
/// * No Rust code block is found in the doc comments.
/// * A Rust code block does not contain an assertion macro invocation.
/// * The attribute is applied more than once to the same function.
#[proc_macro_attribute]
pub fn document_examples(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	match document_examples_worker(attr.into(), item.into()) {
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
/// 3. **Validation** (Optional): Checks that impl blocks and methods have appropriate documentation
///    attributes and emits compile-time warnings for missing documentation.
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
/// To disable validation warnings:
///
/// ```ignore
/// #[fp_macros::document_module(no_validation)]
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
/// ### Validation
///
/// By default, `document_module` validates that impl blocks and methods have appropriate
/// documentation attributes and emits compile-time warnings for missing documentation.
///
/// To disable validation, use `#[document_module(no_validation)]`.
///
/// #### Validation Rules
///
/// An impl block or trait definition should have:
/// * `#[document_type_parameters]` if it has type parameters
/// * `#[document_parameters]` if it contains methods with receiver parameters (self, &self, &mut self)
///
/// A method should have:
/// * `#[document_signature]` - always recommended for documenting the Hindley-Milner signature
/// * `#[document_type_parameters]` if it has type parameters
/// * `#[document_parameters]` if it has non-receiver parameters
/// * `#[document_returns]` if it has a return type
/// * `#[document_examples]` - always recommended
///
/// A free function should have:
/// * `#[document_examples]` - always recommended
///
/// Documentation attributes must not be duplicated and must appear in canonical order:
/// `#[document_signature]` -> `#[document_type_parameters]` -> `#[document_parameters]` ->
/// `#[document_returns]` -> `#[document_examples]`.
///
/// Additionally, a lint warns when a named generic type parameter could be replaced with
/// `impl Trait` (i.e., it has trait bounds, appears in exactly one parameter position, does
/// not appear in the return type, and is not cross-referenced by other type parameters).
/// This lint skips trait implementations. Suppress it on individual functions or methods
/// with `#[allow_named_generics]`.
///
/// #### Examples of Validation
///
/// ```ignore
/// // This will emit warnings:
/// #[fp_macros::document_module]
/// mod inner {
///     pub struct MyType;
///
///     // WARNING: Impl block contains methods with receiver parameters
///     // but no #[document_parameters] attribute
///     impl MyType {
///         // WARNING: Method should have #[document_signature] attribute
///         // WARNING: Method has parameters but no #[document_parameters] attribute
///         // WARNING: Method has a return type but no #[document_returns] attribute
///         // WARNING: Method should have a #[document_examples] attribute
///         pub fn process(&self, x: i32) -> i32 { x }
///     }
/// }
/// ```
///
/// ```ignore
/// // Properly documented (no warnings):
/// #[fp_macros::document_module]
/// mod inner {
///     pub struct MyType;
///
///     #[document_parameters("The MyType instance")]
///     impl MyType {
///         #[document_signature]
///         #[document_parameters("The input value")]
///         #[document_returns("The input value unchanged.")]
///         #[document_examples]
///         ///
///         /// ```
///         /// # use my_crate::MyType;
///         /// let t = MyType;
///         /// assert_eq!(t.process(42), 42);
///         /// ```
///         pub fn process(&self, x: i32) -> i32 { x }
///     }
/// }
/// ```
///
/// ```ignore
/// // Disable validation to suppress warnings:
/// #[fp_macros::document_module(no_validation)]
/// mod inner {
///     // ... undocumented code won't produce warnings ...
/// }
/// ```
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
///         /// ### Type Signature
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

/// Monadic do-notation.
///
/// Desugars flat monadic syntax into nested `bind` calls, matching
/// Haskell/PureScript `do` notation.
///
/// ### Syntax
///
/// ```ignore
/// m_do!(Brand {
///     x <- expr;            // Bind: extract value from monadic computation
///     y: Type <- expr;      // Typed bind: with explicit type annotation
///     _ <- expr;            // Discard bind: sequence, discarding the result
///     expr;                 // Sequence: discard result (shorthand for `_ <- expr;`)
///     let z = expr;         // Let binding: pure, not monadic
///     let w: Type = expr;   // Typed let binding
///     expr                  // Final expression: no semicolon, returned as-is
/// })
///
/// // By-reference mode (for memoized types like Lazy):
/// m_do!(ref Brand {
///     a: &i32 <- lazy_expr; // Closure receives &i32 via RefSemimonad
///     pure(*a * 2)          // pure is rewritten to ref_pure
/// })
/// ```
///
/// * `Brand`: The monad brand type (e.g., `OptionBrand`, `VecBrand`).
/// * `ref` (optional): Enables by-reference dispatch. Closures receive `&A`
///   instead of `A`, routing through `RefSemimonad::ref_bind`. Typed binds
///   use the type as-is (include `&` in the type annotation). Untyped binds
///   get `: &_` added automatically. `pure(x)` is rewritten to `ref_pure(&x)`.
/// * `x <- expr;`: Binds the result of a monadic expression to a pattern.
/// * `let z = expr;`: A pure let binding (not monadic).
/// * `expr;`: Sequences a monadic expression, discarding the result.
/// * `expr` (final): The return expression, emitted as-is.
///
/// Bare `pure(args)` calls are automatically rewritten to `pure::<Brand, _>(args)`
/// (or `ref_pure::<Brand, _>(&args)` in ref mode).
///
/// ### Statement Forms
///
/// | Syntax | Expansion |
/// |--------|-----------|
/// | `x <- expr;` | `bind::<Brand, _, _>(expr, move \|x\| { … })` |
/// | `x: Type <- expr;` | `bind::<Brand, _, _>(expr, move \|x: Type\| { … })` |
/// | `_ <- expr;` | `bind::<Brand, _, _>(expr, move \|_\| { … })` |
/// | `expr;` | `bind::<Brand, _, _>(expr, move \|_\| { … })` |
/// | `let x = expr;` | `{ let x = expr; … }` |
/// | `expr` (final) | Emitted as-is |
///
/// ### Generates
///
/// Nested `bind` calls equivalent to hand-written monadic code.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// use fp_library::{brands::*, functions::*};
/// use fp_macros::m_do;
///
/// let result = m_do!(OptionBrand {
///     x <- Some(5);
///     y <- Some(x + 1);
///     let z = x * y;
///     pure(z)
/// });
/// assert_eq!(result, Some(30));
///
/// // Expanded code
/// let result = bind::<OptionBrand, _, _>(Some(5), move |x| {
///     bind::<OptionBrand, _, _>(Some(x + 1), move |y| {
///         let z = x * y;
///         pure::<OptionBrand, _>(z)
///     })
/// });
/// ```
///
/// ```ignore
/// // Invocation
/// // Works with any monad brand
/// let result = m_do!(VecBrand {
///     x <- vec![1, 2];
///     y <- vec![10, 20];
///     pure(x + y)
/// });
/// assert_eq!(result, vec![11, 21, 12, 22]);
/// ```
///
/// ### Ref mode: multi-bind limitation
///
/// In ref mode, each bind generates a `move` closure that receives `&A`.
/// Inner closures cannot capture references from outer binds because the
/// reference lifetime is scoped to the outer closure. Attempting to use a
/// ref-bound variable in a later bind produces a lifetime error.
///
/// **Workaround:** use `let` bindings to dereference or clone values so
/// they become owned and can be captured by later closures:
///
/// ```ignore
/// m_do!(ref LazyBrand {
///     x: &i32 <- lazy_a;
///     let x_val = *x;          // dereference into owned value
///     y: &i32 <- lazy_b;
///     pure(x_val + *y)         // x_val is owned, safe to use here
/// })
/// ```
///
/// When all binds are independent (no bind uses the result of another),
/// prefer [`a_do!`] instead. Applicative do-notation evaluates all
/// expressions independently, so there is no closure nesting and no
/// capture issue.
#[proc_macro]
pub fn m_do(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DoInput);
	match m_do_worker(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Applicative do-notation.
///
/// Desugars flat applicative syntax into `pure` / `map` / `lift2`–`lift5`
/// calls, matching PureScript `ado` notation. Unlike [`m_do!`], bindings are
/// independent: later bind expressions cannot reference earlier bound variables.
///
/// ### Syntax
///
/// ```ignore
/// a_do!(Brand {
///     x <- expr;            // Bind: independent applicative computation
///     y: Type <- expr;      // Typed bind: with explicit type annotation
///     _ <- expr;            // Discard bind: compute for effect
///     expr;                 // Sequence: shorthand for `_ <- expr;`
///     let z = expr;         // Let binding: placed inside the combining closure
///     let w: Type = expr;   // Typed let binding
///     expr                  // Final expression: the combining body
/// })
///
/// // By-reference mode (for memoized types like Lazy):
/// a_do!(ref Brand {
///     a: &i32 <- lazy_a;    // Closure receives &i32 via RefLift
///     b: &i32 <- lazy_b;
///     *a + *b
/// })
/// ```
///
/// * `Brand`: The applicative brand type (e.g., `OptionBrand`, `VecBrand`).
/// * `ref` (optional): Enables by-reference dispatch. The combining closure
///   receives references (`&A`, `&B`, etc.) via `RefLift::ref_lift2`. Typed
///   binds use the type as-is (include `&`). Untyped binds get `: &_`.
///   Zero-bind blocks use `ref_pure` instead of `pure`.
/// * Bind expressions are evaluated independently (applicative, not monadic).
/// * `let` bindings before any `<-` are hoisted outside the combinator call.
/// * `let` bindings after a `<-` are placed inside the combining closure.
/// * Bare `pure(args)` calls in bind expressions are rewritten to `pure::<Brand, _>(args)`
///   (or `ref_pure::<Brand, _>(&args)` in ref mode).
///
/// ### Desugaring
///
/// | Binds | Expansion |
/// |-------|-----------|
/// | 0 | `pure::<Brand, _>(final_expr)` |
/// | 1 | `map::<Brand, _, _>(\|x\| body, expr)` |
/// | N (2–5) | `liftN::<Brand, _, …>(\|x, y, …\| body, expr1, expr2, …)` |
///
/// ### Examples
///
/// ```ignore
/// use fp_library::{brands::*, functions::*};
/// use fp_macros::a_do;
///
/// // Two independent computations combined with lift2
/// let result = a_do!(OptionBrand {
///     x <- Some(3);
///     y <- Some(4);
///     x + y
/// });
/// assert_eq!(result, Some(7));
///
/// // Expands to:
/// let result = lift2::<OptionBrand, _, _, _>(|x, y| x + y, Some(3), Some(4));
///
/// // Single bind uses map
/// let result = a_do!(OptionBrand { x <- Some(5); x * 2 });
/// assert_eq!(result, Some(10));
///
/// // No binds uses pure
/// let result: Option<i32> = a_do!(OptionBrand { 42 });
/// assert_eq!(result, Some(42));
/// ```
#[proc_macro]
pub fn a_do(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DoInput);
	match a_do_worker(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}
