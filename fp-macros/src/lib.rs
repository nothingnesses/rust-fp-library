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
#[expect(
	clippy::unwrap_used,
	clippy::indexing_slicing,
	reason = "Tests use panicking operations for brevity and clarity"
)]
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
		doc_include_worker,
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
		generate_inferable_brand_name,
		generate_name,
		generate_slot_name,
		impl_kind_worker,
		kind_attr_worker,
		resolve_inferable_brand,
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
#[expect(non_snake_case, reason = "Matches the PascalCase type-level concept it represents")]
pub fn Kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as AssociatedTypes);
	let name = match generate_name(&input) {
		Ok(name) => name,
		Err(e) => return e.to_compile_error().into(),
	};
	quote!(#name).into()
}

/// Generates the name of an `InferableBrand` trait based on its signature.
///
/// This macro is analogous to [`Kind!`] but produces `InferableBrand_{hash}`
/// identifiers instead of `Kind_{hash}`. Both macros use the same content
/// hash, so a `Kind` trait and its corresponding `InferableBrand` trait
/// always share the same hash suffix.
///
/// ### Syntax
///
/// ```ignore
/// InferableBrand!(
///     type AssocName<Params>: Bounds;
///     // ...
/// )
/// ```
///
/// * `Associated Types`: A list of associated type definitions (e.g., `type Of<T>;`) that define the signature of the InferableBrand.
///
/// ### Generates
///
/// The name of the generated `InferableBrand` trait (e.g., `InferableBrand_0123456789abcdef`).
/// The name is deterministic and based on the same hash as the corresponding `Kind` trait.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// let name = InferableBrand!(type Of<'a, A: 'a>: 'a;);
///
/// // Expanded code
/// let name = InferableBrand_...; // e.g., InferableBrand_cdc7cd43dac7585f
/// ```
///
/// ```ignore
/// // Inside Apply! (the primary use case)
/// Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
/// ```
///
/// ### Limitations
///
/// Due to Rust syntax restrictions, this macro cannot be used directly in positions where a
/// concrete path is expected by the parser, such as:
/// * Trait bounds: `FA: InferableBrand!(...) {}` (Invalid)
/// * Qualified paths: `<FA as InferableBrand!(...)>::Brand` (Invalid)
///
/// In these positions, use the generated name directly (e.g., `InferableBrand_cdc7cd43dac7585f`).
/// Inside `Apply!()`, the macro is supported and resolved automatically via preprocessing.
#[proc_macro]
#[expect(non_snake_case, reason = "Matches the PascalCase type-level concept it represents")]
pub fn InferableBrand(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as AssociatedTypes);
	let name = match generate_inferable_brand_name(&input) {
		Ok(name) => name,
		Err(e) => return e.to_compile_error().into(),
	};
	quote!(#name).into()
}

/// Defines a new `Kind` trait and its corresponding `InferableBrand` trait.
///
/// This macro generates a trait definition for a Higher-Kinded Type signature,
/// along with an `InferableBrand` trait that enables automatic Brand inference
/// in dispatch functions (see [`crate::dispatch`](https://docs.rs/fp-library/latest/fp_library/dispatch/)).
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
/// Two public trait definitions with unique names derived from the signature:
///
/// 1. `Kind_{hash}`: The HKT trait with the specified associated types.
/// 2. `InferableBrand_{hash}`: A reverse-mapping trait for brand inference,
///    with a blanket `impl for &T`. Both share the same content hash.
///
/// ### Examples
///
/// ```ignore
/// // Invocation
/// trait_kind!(type Of<T>;);
///
/// // Expanded code
/// pub trait Kind_a1b2... {
///     type Of<T>;
/// }
/// pub trait InferableBrand_a1b2... {
///     type Brand: Kind_a1b2...;
/// }
/// impl<T: InferableBrand_a1b2... + ?Sized> InferableBrand_a1b2... for &T {
///     type Brand = T::Brand;
/// }
/// ```
///
/// ```ignore
/// // Invocation
/// trait_kind!(type Of<'a, T: Display>: Debug;);
///
/// // Expanded code (same pattern: Kind trait + InferableBrand trait + blanket ref impl)
/// pub trait Kind_cdef... {
///     type Of<'a, T: Display>: Debug;
/// }
/// pub trait InferableBrand_cdef... {
///     type Brand: Kind_cdef...;
/// }
/// impl<T: InferableBrand_cdef... + ?Sized> InferableBrand_cdef... for &T { ... }
/// ```
#[proc_macro]
pub fn trait_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as AssociatedTypes);
	match trait_kind_worker(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Implements a `Kind` trait and its `InferableBrand` trait for a brand.
///
/// This macro simplifies the implementation of a generated `Kind` trait for a specific
/// brand type, and also generates the `InferableBrand` impl that enables automatic Brand
/// inference in dispatch functions. It infers the correct `Kind` trait to implement based
/// on the signature of the associated types provided in the block.
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
/// 1. An implementation of the appropriate `Kind_{hash}` trait for the brand.
/// 2. A `Slot_{hash}` impl for the target type with `type Marker = Val`,
///    enabling closure-directed brand inference for both single-brand and
///    multi-brand types.
/// 3. An `InferableBrand_{hash}` impl for the target type, mapping it back to
///    the brand. This enables brand inference for free functions like `map`,
///    `bind`, etc.
///
/// The `InferableBrand` impl is suppressed when:
/// - `#[multi_brand]` is present (for types with multiple brands).
/// - The target type is a projection (contains `Apply!` or `::`).
/// - Multiple associated types are defined.
///
/// The `Slot` impl is generated for ALL brands (including multi-brand types).
/// The `#[multi_brand]` attribute does not suppress Slot generation.
/// Projection types and multiple-associated-type definitions do suppress it.
///
/// ### Attributes
///
/// Inside the `impl_kind!` block, you can use these attributes:
///
/// * `#[multi_brand]`: Marks this brand as sharing its target type with other
///   brands. Suppresses `InferableBrand` impl generation (since the brand is
///   not unique). Does NOT suppress `Slot` impl generation.
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
/// // Expanded code (Kind impl + InferableBrand impl)
/// impl Kind_a1b2... for OptionBrand {
///     type Of<A> = Option<A>;
/// }
/// impl<A> InferableBrand_a1b2... for Option<A> {
///     type Brand = OptionBrand;
/// }
/// ```
///
/// ```ignore
/// // Invocation with impl generics
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
/// impl<A, E> InferableBrand_... for Result<A, E> {
///     type Brand = ResultBrand<E>;
/// }
/// ```
///
/// ```ignore
/// // Multi-brand type: InferableBrand suppressed, Slot still generated
/// impl_kind! {
///     #[multi_brand]
///     impl<E> for ResultErrAppliedBrand<E> {
///         type Of<'a, A: 'a>: 'a = Result<A, E>;
///     }
/// }
///
/// // Expanded code (Kind impl + Slot impl, no InferableBrand)
/// impl<E> Kind_... for ResultErrAppliedBrand<E> {
///     type Of<'a, A: 'a>: 'a = Result<A, E>;
/// }
/// impl<'a, A: 'a, E> Slot_...<'a, ResultErrAppliedBrand<E>, A>
///     for Result<A, E>
/// {
///     type Marker = Val;
/// }
/// ```
///
/// ```ignore
/// // Multiple associated types: InferableBrand skipped automatically
/// impl_kind! {
///     impl<E> for MyBrand<E> where E: Clone {
///         type Of<A> = MyType<A, E>;
///         type SendOf<A> = MySendType<A, E>;
///     }
/// }
///
/// // Expanded code (only Kind impl)
/// impl<E> Kind_... for MyBrand<E> where E: Clone {
///     type Of<A> = MyType<A, E>;
///     type SendOf<A> = MySendType<A, E>;
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
/// `InferableBrand!(SIG)` invocations within the brand position are resolved
/// automatically via preprocessing, enabling readable signatures like:
/// `Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!(...)>::Of<'a, B>)`
///
/// ### Syntax
///
/// ```ignore
/// Apply!(<Brand as Kind!( KindSignature )>::AssocType<Args>)
/// ```
///
/// * `Brand`: The brand type (e.g., `OptionBrand`). May contain `InferableBrand!(SIG)` invocations.
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
///
/// ```ignore
/// // InferableBrand! in the brand position (resolved via preprocessing)
/// Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
///
/// // Expanded code (InferableBrand! resolved to InferableBrand_cdc7...)
/// <FA as InferableBrand_cdc7...>::Brand as Kind_cdc7...>::Of<'a, B>
/// ```
#[proc_macro]
#[expect(non_snake_case, reason = "Matches the PascalCase type-level concept it represents")]
pub fn Apply(input: TokenStream) -> TokenStream {
	// Resolve any InferableBrand!(SIG) invocations before parsing.
	let preprocessed: proc_macro2::TokenStream = input.into();
	let preprocessed = match resolve_inferable_brand(preprocessed) {
		Ok(ts) => ts,
		Err(e) => return e.to_compile_error().into(),
	};
	let input = match syn::parse2::<ApplyInput>(preprocessed) {
		Ok(input) => input,
		Err(e) => return e.to_compile_error().into(),
	};
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
/// and generates `pub use` statements for them. It supports aliasing to resolve name conflicts
/// and exclusions to suppress specific functions from being re-exported.
///
/// ### Syntax
///
/// ```ignore
/// generate_function_re_exports!("path/to/directory", {
///     "module::name": aliased_name,
///     ...
/// }, exclude {
///     "module::name",
///     ...
/// })
/// ```
///
/// * `path/to/directory`: The path to the directory containing the modules, relative to the crate root.
/// * `aliases` (optional): A map of function names to their desired aliases. Keys can be
///   qualified (`"module::function"`) or unqualified (`"function"`). When aliased, the function
///   is exported under the alias name only.
/// * `exclude` (optional): A set of function names to suppress entirely. Keys use the same
///   qualified/unqualified format as aliases. Excluded functions are not re-exported at all,
///   but remain available in their original modules.
///
/// ### Generates
///
/// `pub use` statements for each public function found in the directory, except those
/// listed in the `exclude` block.
///
/// ### Examples
///
/// ```ignore
/// generate_function_re_exports!("src/classes", {
///     "category::identity": category_identity,
///     "filterable::filter": filterable_filter,
/// }, exclude {
///     "ref_filterable::ref_filter",
///     "ref_filterable::ref_filter_map",
/// });
///
/// // Expanded: re-exports all public functions except ref_filter and ref_filter_map.
/// // category::identity is exported as category_identity.
/// // filterable::filter is exported as filterable_filter.
/// ```
#[proc_macro]
pub fn generate_function_re_exports(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as ReExportInput);
	generate_re_exports_worker(&input, &FunctionFormatter).into()
}

/// Generates re-exports for all public traits in a directory.
///
/// This macro scans the specified directory for Rust files, parses them to find public traits,
/// and generates `pub use` statements for them. Supports the same aliasing and exclusion
/// syntax as [`generate_function_re_exports!`].
///
/// ### Syntax
///
/// ```ignore
/// generate_trait_re_exports!("path/to/directory", {
///     "module::TraitName": AliasedName,
///     ...
/// }, exclude {
///     "module::TraitName",
///     ...
/// })
/// ```
///
/// * `path/to/directory`: The path to the directory containing the modules, relative to the crate root.
/// * `aliases` (optional): A map of trait names to their desired aliases.
/// * `exclude` (optional): A set of trait names to suppress from re-export.
///
/// ### Generates
///
/// `pub use` statements for each public trait found in the directory, except those
/// listed in the `exclude` block.
///
/// ### Examples
///
/// ```ignore
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
/// // Auto-generate from the function signature
/// #[document_signature]
/// pub fn function_name<Generics>(params) -> ReturnType { ... }
///
/// // Manual override with an explicit signature string
/// #[document_signature("forall A B. (A -> B) -> A -> B")]
/// pub fn function_name<Generics>(params) -> ReturnType { ... }
/// ```
///
/// When a string argument is provided, it is emitted directly as the
/// signature without any analysis. This is useful for functions whose
/// signatures cannot be inferred automatically.
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
/// Manual override:
///
/// ```ignore
/// // Invocation
/// #[document_signature("forall F A B. Contravariant F => (B -> A, F A) -> F B")]
/// pub fn contramap<FA, A, B>(f: impl Fn(B) -> A, fa: FA) -> FA::Of<B> { ... }
///
/// // Expanded code
/// /// ### Type Signature
/// /// `forall F A B. Contravariant F => (B -> A, F A) -> F B`
/// pub fn contramap<FA, A, B>(f: impl Fn(B) -> A, fa: FA) -> FA::Of<B> { ... }
/// ```
///
/// ### Dispatch-aware generation
///
/// When used inside a module annotated with
/// [`#[document_module]`](macro@document_module), this macro benefits
/// from dispatch trait analysis. If the function references a dispatch
/// trait (via `impl *Dispatch<...>` or a where-clause bound), the
/// macro builds a synthetic signature that replaces dispatch machinery
/// with semantic equivalents (branded types, closure arrows, type
/// class constraints). This produces cleaner signatures like
/// `forall Brand A B. Functor Brand => (A -> B, Brand A) -> Brand B`
/// instead of the raw Rust signature with `InferableBrand` and
/// `Kind_*` bounds.
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
/// Haskell/PureScript `do` notation. Supports both explicit-brand and
/// inferred-brand modes.
///
/// ### Syntax
///
/// ```ignore
/// // Explicit mode: brand specified, pure() rewritten automatically
/// m_do!(Brand {
///     x <- expr;            // Bind: extract value from monadic computation
///     y: Type <- expr;      // Typed bind: with explicit type annotation
///     _ <- expr;            // Discard bind: sequence, discarding the result
///     expr;                 // Sequence: discard result (shorthand for `_ <- expr;`)
///     let z = expr;         // Let binding: pure, not monadic
///     let w: Type = expr;   // Typed let binding
///     pure(z)               // pure() rewritten to pure::<Brand, _>(z)
/// })
///
/// // Inferred mode: brand inferred from container types
/// m_do!({
///     x <- Some(5);         // Brand inferred from Some(5) via InferableBrand
///     y <- Some(x + 1);
///     Some(x + y)           // Write concrete constructor (pure() not available)
/// })
///
/// // By-reference modes:
/// m_do!(ref Brand { ... })  // Explicit, ref dispatch
/// m_do!(ref { ... })        // Inferred, ref dispatch
/// ```
///
/// * `Brand` (optional): The monad brand type. When omitted, the brand is inferred
///   from container types via `InferableBrand`.
/// * `ref` (optional): Enables by-reference dispatch. Closures receive `&A`
///   instead of `A`, routing through `RefSemimonad::ref_bind`. Typed binds
///   use the type as-is (include `&` in the type annotation). Untyped binds
///   get `: &_` added automatically.
/// * In explicit mode, bare `pure(args)` calls are rewritten to `pure::<Brand, _>(args)`
///   (or `ref_pure::<Brand, _>(&args)` in ref mode).
/// * In inferred mode, bare `pure(args)` calls emit a `compile_error!` because
///   `pure` has no container argument to infer the brand from. Write concrete
///   constructors instead (e.g., `Some(x)` instead of `pure(x)`).
///
/// ### Statement Forms
///
/// | Syntax | Explicit expansion | Inferred expansion |
/// |--------|--------------------|--------------------|
/// | `x <- expr;` | `explicit::bind::<Brand, _, _, _, _>(expr, move \|x\| { ... })` | `bind(expr, move \|x\| { ... })` |
/// | `x: Type <- expr;` | Same with `\|x: Type\|` | Same with `\|x: Type\|` |
/// | `expr;` | `explicit::bind::<Brand, _, _, _, _>(expr, move \|_\| { ... })` | `bind(expr, move \|_\| { ... })` |
/// | `let x = expr;` | `{ let x = expr; ... }` | `{ let x = expr; ... }` |
/// | `expr` (final) | Emitted as-is | Emitted as-is |
///
/// ### Examples
///
/// ```ignore
/// // Inferred mode (primary API for single-brand types)
/// let result = m_do!({
///     x <- Some(5);
///     y <- Some(x + 1);
///     let z = x * y;
///     Some(z)
/// });
/// assert_eq!(result, Some(30));
///
/// // Expands to:
/// let result = bind(Some(5), move |x| {
///     bind(Some(x + 1), move |y| {
///         let z = x * y;
///         Some(z)
///     })
/// });
/// ```
///
/// ```ignore
/// // Explicit mode (for ambiguous types or to use pure())
/// let result = m_do!(VecBrand {
///     x <- vec![1, 2];
///     y <- vec![10, 20];
///     pure(x + y)
/// });
/// assert_eq!(result, vec![11, 21, 12, 22]);
///
/// // Expands to:
/// let result = explicit::bind::<VecBrand, _, _, _, _>(vec![1, 2], move |x| {
///     explicit::bind::<VecBrand, _, _, _, _>(vec![10, 20], move |y| {
///         pure::<VecBrand, _>(x + y)
///     })
/// });
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
/// Desugars flat applicative syntax into `pure` / `map` / `lift2`-`lift5`
/// calls, matching PureScript `ado` notation. Unlike [`m_do!`], bindings are
/// independent: later bind expressions cannot reference earlier bound variables.
/// Supports both explicit-brand and inferred-brand modes.
///
/// ### Syntax
///
/// ```ignore
/// // Explicit mode
/// a_do!(Brand {
///     x <- expr;            // Bind: independent applicative computation
///     y: Type <- expr;      // Typed bind: with explicit type annotation
///     _ <- expr;            // Discard bind: compute for effect
///     expr;                 // Sequence: shorthand for `_ <- expr;`
///     let z = expr;         // Let binding: placed inside the combining closure
///     expr                  // Final expression: the combining body
/// })
///
/// // Inferred mode (brand inferred from container types)
/// a_do!({
///     x <- Some(3);
///     y <- Some(4);
///     x + y
/// })
///
/// // By-reference modes:
/// a_do!(ref Brand { ... })  // Explicit, ref dispatch
/// a_do!(ref { ... })        // Inferred, ref dispatch
/// ```
///
/// * `Brand` (optional): The applicative brand type. When omitted, the brand is
///   inferred from container types via `InferableBrand`.
/// * `ref` (optional): Enables by-reference dispatch. The combining closure
///   receives references (`&A`, `&B`, etc.) via `RefLift::ref_lift2`. Typed
///   binds use the type as-is (include `&`). Untyped binds get `: &_`.
/// * Bind expressions are evaluated independently (applicative, not monadic).
/// * `let` bindings before any `<-` are hoisted outside the combinator call.
/// * `let` bindings after a `<-` are placed inside the combining closure.
/// * In explicit mode, bare `pure(args)` calls are rewritten to `pure::<Brand, _>(args)`.
/// * In inferred mode, bare `pure(args)` calls emit a `compile_error!`.
/// * In inferred mode with 0 binds, a `compile_error!` is emitted because
///   `pure()` requires a brand. Write the concrete constructor directly.
///
/// ### Desugaring
///
/// | Binds | Explicit expansion | Inferred expansion |
/// |-------|--------------------|--------------------|
/// | 0 | `pure::<Brand, _>(final_expr)` | `compile_error!` |
/// | 1 | `explicit::map::<Brand, _, _, _, _>(\|x\| body, expr)` | `map(\|x\| body, expr)` |
/// | N (2-5) | `explicit::liftN::<Brand, ...>(\|x, y, ...\| body, ...)` | `liftN(\|x, y, ...\| body, ...)` |
///
/// ### Examples
///
/// ```ignore
/// use fp_library::functions::*;
/// use fp_macros::a_do;
///
/// // Inferred mode: two independent computations combined with lift2
/// let result = a_do!({
///     x <- Some(3);
///     y <- Some(4);
///     x + y
/// });
/// assert_eq!(result, Some(7));
///
/// // Expands to:
/// let result = lift2(|x, y| x + y, Some(3), Some(4));
///
/// // Inferred mode: single bind uses map
/// let result = a_do!({ x <- Some(5); x * 2 });
/// assert_eq!(result, Some(10));
///
/// // Expands to:
/// let result = map(|x| x * 2, Some(5));
/// ```
///
/// ```ignore
/// // Explicit mode: zero-bind block uses pure (requires brand)
/// let result: Option<i32> = a_do!(OptionBrand { 42 });
/// assert_eq!(result, Some(42));
///
/// // Expands to:
/// let result: Option<i32> = pure::<OptionBrand, _>(42);
///
/// // Explicit mode: single bind
/// let result = a_do!(OptionBrand { x <- Some(5); x * 2 });
///
/// // Expands to:
/// let result = explicit::map::<OptionBrand, _, _, _, _>(|x| x * 2, Some(5));
/// ```
#[proc_macro]
pub fn a_do(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DoInput);
	match a_do_worker(input) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Includes a markdown file with relative `.md` links rewritten to rustdoc intra-doc links.
///
/// This macro reads a markdown file at compile time (relative to `CARGO_MANIFEST_DIR`)
/// and rewrites same-directory `.md` links to point at `crate::docs::module_name`
/// submodules, making cross-document links work in rendered rustdoc output.
///
/// ### Syntax
///
/// ```ignore
/// #![doc = doc_include!("docs/hkt.md")]
/// ```
///
/// ### Link Rewriting
///
/// - `[text](./foo-bar.md)` becomes `[text][crate::docs::foo_bar]`
/// - `[text](foo-bar.md)` becomes `[text][crate::docs::foo_bar]`
/// - Links with path separators (`../`, subdirectories) are left unchanged.
/// - Non-`.md` links are left unchanged.
#[proc_macro]
pub fn doc_include(input: TokenStream) -> TokenStream {
	match doc_include_worker(input.into()) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}
