//! # Self Resolution Algorithm
//!
//! This module implements resolution of `Self` references in type signatures and documentation
//! generation. The system resolves `Self` to concrete types through a four-level hierarchical
//! precedence system.
//!
//! ## Overview
//!
//! When generating HM-style documentation with `#[document_signature]`, the macro needs to resolve
//! references to `Self` and its associated types (e.g., `Self::Map`). The resolution system
//! follows a four-level hierarchy to determine which concrete type to use.
//!
//! ## Resolution Order (Highest to Lowest Precedence)
//!
//! The algorithm tries each level in order until a resolution is found:
//!
//! ### 1. Method-Level Override: `#[document_use = "AssocName"]`
//!
//! The highest precedence is given to explicit annotations on individual methods.
//! This allows per-method customization of how `Self` projections are resolved.
//!
//! **Example:**
//! ```rust,ignore
//! impl MyBrand {
//!     #[document_use = "Of"]  // ← Method-level override
//!     #[document_signature]
//!     fn map<A, B>(self, f: impl Fn(A) -> B) -> Apply!(<Self as Kind>::Of<B>) {
//!         // When resolving <Self as Kind>::Of, use "Of" association
//!         // from the impl_kind! projection for MyBrand
//!     }
//! }
//! ```
//!
//! In this example, when resolving `<Self as Kind>::Of<B>`, the system looks up the
//! `Of` associated type defined for `MyBrand` in the `impl_kind!` macro.
//!
//! ### 2. Impl-Level Override: `#[document_use = "AssocName"]`
//!
//! The second level is an annotation on the entire impl block. This applies
//! to all methods in the block that don't have their own method-level override.
//!
//! **Example:**
//! ```rust,ignore
//! #[document_use = "Of"]  // ← Impl-level override
//! impl Functor for MyBrand {
//!     #[document_signature]
//!     fn map<A, B>(self, f: impl Fn(A) -> B) -> Apply!(<Self as Kind>::Of<B>) {
//!         // All methods in this impl use "Of" unless overridden
//!     }
//!
//!     #[document_signature]
//!     fn fmap<A, B>(self, f: impl Fn(A) -> B) -> Apply!(<Self as Kind>::Of<B>) {
//!         // Also uses "Of" from impl-level
//!     }
//! }
//! ```
//!
//! ### 3. (Type, Trait)-Scoped Default: `#[document_default]` in impl
//!
//! The third level is a scoped default specific to a (Type, Trait) pair.
//! This is set using `#[document_default]` on an associated type declaration
//! within a trait implementation.
//!
//! **Example:**
//! ```rust,ignore
//! impl Functor for MyBrand {
//!     #[document_default]  // ← Scoped default for (MyBrand, Functor)
//!     type Map = MyType<T>;
//!     type Item = OtherType<T>;
//!
//!     #[document_signature]
//!     fn map<A, B>(self, f: impl Fn(A) -> B) -> Self::Map {
//!         // Uses the Map associated type as default
//!     }
//! }
//! ```
//!
//! This default only applies to `MyBrand` when implementing `Functor`,
//! allowing different defaults for different trait implementations.
//!
//! ### 4. Module-Level Default: `#[document_default]` in `impl_kind!`
//!
//! The lowest precedence is given to module-wide defaults declared using
//! the `impl_kind!` macro. This sets a fallback for any type that doesn't
//! have a more specific override.
//!
//! **Example:**
//! ```rust,ignore
//! impl_kind! {
//!     for MyBrand {
//!         #[document_default]  // ← Module-level default for MyBrand
//!         type Of<T> = MyType<T>;
//!         type Other<T> = OtherType<T>;
//!     }
//! }
//!
//! // Later in an impl without explicit document_use:
//! impl Functor for MyBrand {
//!     #[document_signature]
//!     fn map<A, B>(self, f: impl Fn(A) -> B) -> Apply!(<Self as Kind>::Of<B>) {
//!         // Falls back to module-level default: Of -> MyType<B>
//!     }
//! }
//! ```
//!
//! ## Resolution Examples
//!
//! ### Example 1: Module-Level Default
//!
//! The simplest case uses module-level defaults from `impl_kind!`:
//!
//! ```rust,ignore
//! impl_kind! {
//!     for MyBrand {
//!         #[document_default]
//!         type Of<T> = MyType<T>;
//!     }
//! }
//!
//! impl MyBrand {
//!     #[document_signature]
//!     fn create<A>(value: A) -> Apply!(<Self as Kind>::Of<A>) {
//!         // Resolves to: MyType<A>
//!     }
//! }
//! ```
//!
//! ### Example 2: Impl-Level Override
//!
//! Override the default for all methods in an impl block:
//!
//! ```rust,ignore
//! impl_kind! {
//!     for MyBrand {
//!         #[document_default]
//!         type Of<T> = MyType<T>;
//!         type Other<T> = OtherType<T>;
//!     }
//! }
//!
//! #[document_use = "Other"]  // ← Override for this entire impl
//! impl Monad for MyBrand {
//!     #[document_signature]
//!     fn bind<A, B>(self, f: impl Fn(A) -> B) -> Apply!(<Self as Kind>::Other<B>) {
//!         // Resolves to: OtherType<B>
//!     }
//! }
//! ```
//!
//! ### Example 3: Method-Level Override
//!
//! Override the resolution for a single method:
//!
//! ```rust,ignore
//! impl Applicative for MyBrand {
//!     #[document_use = "Other"]  // ← Method-level override
//!     #[document_signature]
//!     fn apply<A, B>(self, f: impl Fn(A) -> B) -> Apply!(<Self as Kind>::Other<B>) {
//!         // Resolves to: OtherType<B>
//!     }
//!
//!     #[document_signature]
//!     fn pure<A>(value: A) -> Apply!(<Self as Kind>::Of<A>) {
//!         // Uses module-level default: MyType<A>
//!     }
//! }
//! ```
//!
//! ### Example 4: Qualified Self (e.g., `<Self as Trait>::Assoc`)
//!
//! Qualified self references explicitly specify which trait's associated type to use:
//!
//! ```rust,ignore
//! impl MyTrait for MyBrand {
//!     #[document_signature]
//!     fn method(self) -> Apply!(<Self as Kind>::Of<i32>) {
//!         // System extracts "Kind" trait and "Of" association
//!         // Looks up (MyBrand, Kind, Of) in projections map
//!         // Falls back through the four-level hierarchy
//!     }
//! }
//! ```
//!
//! The system extracts the trait qualification and uses the (Type, Trait)-scoped
//! resolution logic.
//!
//! ### Example 5: Bare Self
//!
//! When `Self` appears without qualification or associated type:
//!
//! ```rust,ignore
//! impl SomeTrait for MyBrand {
//!     #[document_signature]
//!     fn returns_self(self) -> Self {
//!         // Resolves to MyBrand (the concrete self type)
//!         // If impl has generics like impl<A> SomeTrait for MyType<A>,
//!         // it resolves to MyType<A> with parameters preserved
//!     }
//! }
//! ```
//!
//! The system preserves the concrete type with its generic parameters.
//!
//! ## Implementation Details
//!
//! The resolution algorithm is implemented in [`SelfSubstitutor`], which uses
//! the visitor pattern to traverse type syntax trees and replace `Self` references.
//!
//! **Key methods:**
//! - [`resolve_default_assoc_name`](SelfSubstitutor::resolve_default_assoc_name) - Implements the four-level hierarchy
//! - Visitor methods handle different type patterns (paths, qualified paths, etc.)
//!
//! **Resolution flow:**
//! 1. Visitor encounters a `Self` reference
//! 2. If it's a projection (`Self::Assoc` or `<Self as Trait>::Assoc`), extract association name
//! 3. Consult the four-level hierarchy to find the concrete type
//! 4. Look up the concrete type in the projections map
//! 5. Substitute `Self` with the resolved type
//!
//! ## Error Handling
//!
//! When resolution fails (e.g., no matching associated type found), the system
//! generates detailed error messages showing:
//! - The projection that failed to resolve
//! - Available associated types in the current context
//! - Suggestions for fixing the issue
//!
//! Errors are collected in [`SelfSubstitutor::errors`] and can be reported
//! to the user with proper span information.

use {
	crate::{
		analysis::{
			format_brand_name,
			get_type_parameters,
		},
		core::{
			config::Config,
			constants::{
				macros,
				types,
			},
			error_handling::ErrorCollector,
		},
		hkt::ApplyInput,
		resolution::ProjectionKey,
	},
	quote::quote,
	std::collections::HashMap,
	syn::{
		Error,
		GenericParam,
		Signature,
		parse_quote,
		spanned::Spanned,
		visit_mut::{
			self,
			VisitMut,
		},
	},
};

/// Extract the concrete type name from a Type for use in HM signatures
pub fn get_concrete_type_name(
	ty: &syn::Type,
	config: &Config,
) -> Option<String> {
	match ty {
		syn::Type::Path(type_path) => {
			if let Some(segment) = type_path.path.segments.first() {
				let name = segment.ident.to_string();
				// Apply brand name formatting
				Some(format_brand_name(&name, config))
			} else {
				None
			}
		}
		_ => None,
	}
}

/// Extract base type name and generic parameter names from impl self type
/// For `impl<A> CatList<A>`, returns ("CatList", ["A"])
pub fn get_self_type_info(
	self_ty: &syn::Type,
	impl_generics: &syn::Generics,
) -> (Option<String>, Vec<String>) {
	let base_name = match self_ty {
		syn::Type::Path(type_path) =>
			type_path.path.segments.last().map(|seg| seg.ident.to_string()),
		_ => None,
	};

	let generic_names = get_type_parameters(impl_generics);

	(base_name, generic_names)
}

/// Build a parameterized type from a base name and generic parameters
/// For ("CatList", ["A"]), returns `CatList<A>`
pub fn build_parameterized_type(
	base_name: &str,
	generic_params: &[String],
) -> syn::Type {
	let base_ident = syn::Ident::new(base_name, proc_macro2::Span::call_site());
	if generic_params.is_empty() {
		parse_quote!(#base_ident)
	} else {
		let params: Vec<syn::Ident> = generic_params
			.iter()
			.map(|p| syn::Ident::new(p, proc_macro2::Span::call_site()))
			.collect();
		parse_quote!(#base_ident<#(#params),*>)
	}
}

/// Merges generic parameters from an impl block into a function signature.
///
/// This function combines generic parameters from both the impl block and the function signature,
/// ensuring proper ordering: lifetimes first, then types, then const parameters. It also merges
/// where clauses from the impl block into the function's where clause.
///
/// ### Parameters
/// * `sig` - The function signature to modify (generic parameters will be added/merged)
/// * `impl_generics` - The generic parameters from the impl block
///
/// ### Example
/// Given `impl<T> Foo<T>` and `fn bar<U>()`, the result will be `fn bar<T, U>()`.
pub fn merge_generics(
	sig: &mut Signature,
	impl_generics: &syn::Generics,
) {
	let mut new_params = syn::punctuated::Punctuated::<GenericParam, syn::token::Comma>::new();
	for p in impl_generics.params.iter().chain(sig.generics.params.iter()) {
		if let GenericParam::Lifetime(_) = p {
			new_params.push(p.clone());
		}
	}
	for p in impl_generics.params.iter().chain(sig.generics.params.iter()) {
		if let GenericParam::Type(_) = p {
			new_params.push(p.clone());
		}
	}
	for p in impl_generics.params.iter().chain(sig.generics.params.iter()) {
		if let GenericParam::Const(_) = p {
			new_params.push(p.clone());
		}
	}
	sig.generics.params = new_params;

	if let Some(impl_where) = &impl_generics.where_clause {
		let where_clause = sig.generics.make_where_clause();
		for pred in &impl_where.predicates {
			where_clause.predicates.push(pred.clone());
		}
	}
}

pub struct SelfSubstitutor<'a> {
	self_ty: &'a syn::Type,
	self_ty_path: &'a str,
	trait_path: Option<&'a str>,
	document_use: Option<&'a str>,
	signature_hash: Option<u64>,
	config: &'a Config,
	pub errors: ErrorCollector,
	/// The base type name (e.g., "CatList") extracted from self_ty
	base_type_name: Option<String>,
	/// Generic parameter names from the impl block (e.g., ["A"])
	impl_generic_params: Vec<String>,
}

impl<'a> SelfSubstitutor<'a> {
	pub fn new(
		self_ty: &'a syn::Type,
		self_ty_path: &'a str,
		trait_path: Option<&'a str>,
		document_use: Option<&'a str>,
		config: &'a Config,
		base_type_name: Option<String>,
		impl_generic_params: Vec<String>,
	) -> Self {
		Self {
			self_ty,
			self_ty_path,
			trait_path,
			document_use,
			signature_hash: None,
			config,
			errors: ErrorCollector::new(),
			base_type_name,
			impl_generic_params,
		}
	}

	/// Resolve the default associated type name for bare `Self`.
	/// Tries document_use, then scoped_defaults, then module_defaults.
	fn resolve_default_assoc_name(&self) -> Option<String> {
		self.document_use
			.map(|s| s.to_string())
			.or_else(|| {
				self.trait_path.and_then(|tp| {
					self.config
						.scoped_defaults
						.get(&(self.self_ty_path.to_string(), tp.to_string()))
						.cloned()
				})
			})
			.or_else(|| self.config.module_defaults.get(self.self_ty_path).cloned())
	}

	/// Look up a projection in the config by associated type name.
	/// Tries trait-specific projection first, then falls back to module-level.
	fn lookup_projection(
		&self,
		assoc_name: &str,
	) -> Option<&(syn::Generics, syn::Type)> {
		// Try (Type, Trait, AssocName) scoped lookup first
		let scoped_key = self
			.trait_path
			.map(|trait_path| ProjectionKey::scoped(self.self_ty_path, trait_path, assoc_name));

		if let Some(key) = scoped_key
			&& let Some(result) = self.config.projections.get(&key)
		{
			return Some(result);
		}

		// Fall back to module-level (Type, AssocName) lookup
		let mut module_key = ProjectionKey::new(self.self_ty_path, assoc_name);
		if let Some(hash) = self.signature_hash {
			module_key = module_key.with_signature_hash(hash);
		}

		if let Some(result) = self.config.projections.get(&module_key) {
			return Some(result);
		}

		// If we had a hash and didn't find it, try without hash as fallback for legacy or non-hashed entries
		if self.signature_hash.is_some() {
			let module_key_no_hash = ProjectionKey::new(self.self_ty_path, assoc_name);
			if let Some(result) = self.config.projections.get(&module_key_no_hash) {
				#[cfg(debug_assertions)]
				{
					eprintln!(
						"Warning: Signature hash lookup failed for {}.{assoc_name}, falling back to legacy lookup",
						self.self_ty_path
					);
				}
				return Some(result);
			}
		}

		None
	}

	/// Build a fallback type using the base type name and impl generic parameters.
	fn build_fallback_type(&self) -> syn::Type {
		if let Some(base_name) = &self.base_type_name {
			build_parameterized_type(base_name, &self.impl_generic_params)
		} else {
			self.self_ty.clone()
		}
	}

	/// Resolve bare `Self` to a concrete type.
	fn resolve_bare_self(
		&mut self,
		tp: &syn::TypePath,
	) -> syn::Type {
		if let Some(assoc_name) = self.resolve_default_assoc_name() {
			if let Some((_generics, target)) = self.lookup_projection(&assoc_name) {
				target.clone()
			} else {
				// Fallback: use parameterized concrete type if available
				self.build_fallback_type()
			}
		} else {
			// No default found
			if self.base_type_name.is_some() {
				self.build_fallback_type()
			} else {
				// Report error with available types
				self.errors.push(create_missing_default_error(
					tp.span(),
					self.self_ty_path,
					self.trait_path,
					self.config,
				));
				self.self_ty.clone()
			}
		}
	}

	/// Resolve `Self::AssocType<Args>` to a concrete type.
	fn resolve_self_assoc_type(
		&mut self,
		tp: &syn::TypePath,
		segment: &syn::PathSegment,
	) -> syn::Type {
		let assoc_name = segment.ident.to_string();
		if let Some((generics, target)) = self.lookup_projection(&assoc_name) {
			if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
				substitute_generics(target.clone(), generics, &args.args)
			} else {
				target.clone()
			}
		} else {
			// Report error with available types
			self.errors.push(create_missing_assoc_type_error(
				tp.span(),
				self.self_ty_path,
				&assoc_name,
				self.trait_path,
				self.config,
			));
			// Fallback to qualified path
			let self_ty = self.self_ty;
			let mut new_path = tp.path.clone();
			new_path.segments = new_path.segments.into_iter().skip(1).collect();
			let segments = &new_path.segments;
			parse_quote!(<#self_ty>::#segments)
		}
	}
}

impl<'a> VisitMut for SelfSubstitutor<'a> {
	fn visit_type_mut(
		&mut self,
		i: &mut syn::Type,
	) {
		if let syn::Type::Path(tp) = i {
			if tp.path.is_ident(types::SELF) {
				// Resolve bare Self
				*i = self.resolve_bare_self(tp);
			} else if let Some(first) = tp.path.segments.first()
				&& first.ident == types::SELF
				&& tp.path.segments.len() > 1
			{
				// Resolve Self::AssocType<Args>
				// SAFETY: segments.len() > 1 checked above
				#[allow(clippy::indexing_slicing)]
				let segment = &tp.path.segments[1];
				*i = self.resolve_self_assoc_type(tp, segment);
			}
		}
		visit_mut::visit_type_mut(self, i);
	}

	fn visit_type_macro_mut(
		&mut self,
		i: &mut syn::TypeMacro,
	) {
		if i.mac.path.is_ident(macros::APPLY_MACRO)
			&& let Ok(mut apply_input) = syn::parse2::<ApplyInput>(i.mac.tokens.clone())
		{
			// Temporarily store signature hash for lookups within this Apply!
			let prev_hash = self.signature_hash;
			// We need the hash of the target associated type within the Apply! macro.
			// Apply! input contains kind_input (AssociatedTypes) and assoc_name.
			let current_hash = match apply_input
				.kind_input
				.associated_types
				.iter()
				.find(|a| a.signature.name == apply_input.assoc_name)
			{
				Some(a) => match crate::hkt::canonicalizer::hash_assoc_signature(&a.signature) {
					Ok(h) => Some(h),
					Err(e) => {
						self.errors.push(syn::Error::new(
							a.signature.name.span(),
							format!("Failed to compute signature hash: {e}"),
						));
						None
					}
				},
				None => None,
			};
			self.signature_hash = current_hash;

			self.visit_type_mut(&mut apply_input.brand);
			for arg in apply_input.args.args.iter_mut() {
				if let syn::GenericArgument::Type(ty) = arg {
					self.visit_type_mut(ty);
				}
			}

			// If the brand resolved to a target type with generics, and Apply! has arguments,
			// we should perform substitution if possible.
			// However, document_module's SelfSubstitutor currently only replaces the type.
			// The HM signature generator will handle the projection if it remains a path.
			// But if we already replaced it with a concrete type (e.g. CatList),
			// we might have CatList instead of CatList<A>.

			// Let's check if brand was substituted.
			// We can use the same logic as visit_type_mut for Self segments.

			let brand = &apply_input.brand;
			let kind_input = &apply_input.kind_input;
			let assoc_name = &apply_input.assoc_name;
			let args = &apply_input.args;

			i.mac.tokens = quote! { <#brand as Kind!(#kind_input)>::#assoc_name #args };

			self.signature_hash = prev_hash;
		}
		visit_mut::visit_type_macro_mut(self, i);
	}

	fn visit_signature_mut(
		&mut self,
		i: &mut Signature,
	) {
		for input in &mut i.inputs {
			if let syn::FnArg::Receiver(r) = input {
				// Build the concrete parameterized type for the receiver
				let concrete_ty = if let Some(base_name) = &self.base_type_name {
					build_parameterized_type(base_name, &self.impl_generic_params)
				} else {
					self.self_ty.clone()
				};

				let attrs = &r.attrs;
				if let Some(reference) = &r.reference {
					let lt = &reference.1;
					if r.mutability.is_some() {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt mut #concrete_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					} else {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt #concrete_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					}
				} else {
					let pat: syn::Pat = parse_quote!(self);
					let ty: syn::Type = parse_quote!(#concrete_ty);
					*input = syn::FnArg::Typed(syn::PatType {
						attrs: attrs.clone(),
						pat: Box::new(pat),
						colon_token: Default::default(),
						ty: Box::new(ty),
					});
				}
			}
		}
		visit_mut::visit_signature_mut(self, i);
	}
}

pub fn type_uses_self_assoc(ty: &syn::Type) -> bool {
	struct SelfAssocVisitor {
		found: bool,
	}
	impl syn::visit::Visit<'_> for SelfAssocVisitor {
		fn visit_type_path(
			&mut self,
			i: &syn::TypePath,
		) {
			if let Some(first) = i.path.segments.first()
				&& first.ident == types::SELF
				&& i.path.segments.len() > 1
			{
				self.found = true;
			}
			syn::visit::visit_type_path(self, i);
		}
	}
	let mut visitor = SelfAssocVisitor {
		found: false,
	};
	syn::visit::visit_type(&mut visitor, ty);
	visitor.found
}

/// Substitutes generic parameters in a type with concrete type arguments.
///
/// This function takes a type that uses generic parameters and replaces those parameters
/// with concrete types from the provided arguments. It handles both type parameters and
/// const parameters.
///
/// ### Parameters
/// * `ty` - The type to transform (may contain generic parameter references)
/// * `generics` - The generic parameter definitions (e.g., from `<T, U>`)
/// * `args` - The concrete arguments to substitute (e.g., from `<String, i32>`)
///
/// ### Returns
/// The type with all generic parameters replaced by their concrete arguments.
///
/// ### Example
/// Given `Vec<T>` with generics `<T>` and args `<String>`, returns `Vec<String>`.
pub(crate) fn substitute_generics(
	mut ty: syn::Type,
	generics: &syn::Generics,
	args: &syn::punctuated::Punctuated<syn::GenericArgument, syn::token::Comma>,
) -> syn::Type {
	let mut mapping = HashMap::new();
	let mut const_mapping = HashMap::new();

	for (param, arg) in generics.params.iter().zip(args.iter()) {
		match (param, arg) {
			(syn::GenericParam::Type(tp), syn::GenericArgument::Type(at)) => {
				mapping.insert(tp.ident.to_string(), at.clone());
			}
			(syn::GenericParam::Const(cp), syn::GenericArgument::Const(ca)) => {
				const_mapping.insert(cp.ident.to_string(), ca.clone());
			}
			(syn::GenericParam::Const(cp), syn::GenericArgument::Type(syn::Type::Path(tp)))
				if tp.path.get_ident().is_some() =>
			{
				// Sometimes const generics are passed as types in early parsing phases or macros
				if let Some(ident) = tp.path.get_ident() {
					const_mapping.insert(cp.ident.to_string(), syn::parse_quote!(#ident));
				}
			}
			_ => {}
		}
	}

	struct SubstitutionVisitor<'a> {
		mapping: &'a HashMap<String, syn::Type>,
		const_mapping: &'a HashMap<String, syn::Expr>,
	}
	impl VisitMut for SubstitutionVisitor<'_> {
		fn visit_type_mut(
			&mut self,
			i: &mut syn::Type,
		) {
			if let syn::Type::Path(tp) = i
				&& let Some(ident) = tp.path.get_ident()
				&& let Some(target) = self.mapping.get(&ident.to_string())
			{
				*i = target.clone();
				return;
			}
			visit_mut::visit_type_mut(self, i);
		}

		fn visit_expr_mut(
			&mut self,
			i: &mut syn::Expr,
		) {
			if let syn::Expr::Path(ep) = i
				&& let Some(ident) = ep.path.get_ident()
				&& let Some(target) = self.const_mapping.get(&ident.to_string())
			{
				*i = target.clone();
				return;
			}
			visit_mut::visit_expr_mut(self, i);
		}
	}

	let mut visitor = SubstitutionVisitor {
		mapping: &mapping,
		const_mapping: &const_mapping,
	};
	visitor.visit_type_mut(&mut ty);
	ty
}

/// Normalizes a type by replacing generic parameter names with canonical names.
///
/// This function is used for comparing types structurally by converting generic parameter
/// names to a canonical form (T0, T1, T2, etc.). This allows detecting if two types are
/// semantically equivalent even if they use different parameter names.
///
/// ### Parameters
/// * `ty` - The type to normalize
/// * `generics` - The generic parameter definitions that define which names to normalize
///
/// ### Returns
/// A normalized type where generic parameters are renamed to T0, T1, T2, etc.
///
/// ### Example
/// Given `Vec<A>` with generics `<A>`, returns `Vec<T0>`.
/// Given `HashMap<K, V>` with generics `<K, V>`, returns `HashMap<T0, T1>`.
///
/// This enables comparing `Vec<A>` and `Vec<B>` as structurally equivalent.
pub fn normalize_type(
	mut ty: syn::Type,
	generics: &syn::Generics,
) -> syn::Type {
	let mut mapping = HashMap::new();
	let mut type_idx = 0;
	for param in &generics.params {
		if let syn::GenericParam::Type(tp) = param {
			let ident = quote::format_ident!("T{type_idx}");
			mapping.insert(tp.ident.to_string(), parse_quote!(#ident));
			type_idx += 1;
		}
	}

	struct NormalizationVisitor<'a> {
		mapping: &'a HashMap<String, syn::Type>,
	}
	impl VisitMut for NormalizationVisitor<'_> {
		fn visit_type_mut(
			&mut self,
			i: &mut syn::Type,
		) {
			if let syn::Type::Path(tp) = i
				&& let Some(ident) = tp.path.get_ident()
				&& let Some(target) = self.mapping.get(&ident.to_string())
			{
				*i = target.clone();
				return;
			}
			visit_mut::visit_type_mut(self, i);
		}
	}

	let mut visitor = NormalizationVisitor {
		mapping: &mapping,
	};
	visitor.visit_type_mut(&mut ty);
	ty
}

fn get_available_types_for_brand(
	config: &Config,
	self_ty_path: &str,
	trait_path: Option<&str>,
) -> (Vec<String>, Vec<String>) {
	let mut in_this_impl = Vec::new();
	let mut in_other_traits = Vec::new();

	for key in config.projections.keys() {
		if key.type_path() == self_ty_path {
			match (key.trait_path(), trait_path) {
				(Some(t), Some(current)) if t == current => {
					in_this_impl.push(key.assoc_name().to_string());
				}
				(Some(_), _) | (None, _) => {
					in_other_traits.push(key.assoc_name().to_string());
				}
			}
		}
	}

	in_this_impl.sort();
	in_this_impl.dedup();
	in_other_traits.sort();
	in_other_traits.dedup();

	(in_this_impl, in_other_traits)
}

fn create_missing_default_error(
	span: proc_macro2::Span,
	self_ty_path: &str,
	trait_path: Option<&str>,
	config: &Config,
) -> Error {
	let (in_this_impl, in_other_traits) =
		get_available_types_for_brand(config, self_ty_path, trait_path);

	let mut message =
		format!("Cannot resolve bare `Self` for type `{self_ty_path}` - no default specified");

	if !in_this_impl.is_empty() {
		message.push_str(&format!(
			r#"
  = note: Available in this impl: {}"#,
			in_this_impl.join(", ")
		));
	}

	if !in_other_traits.is_empty() {
		message.push_str(&format!(
			r#"
  = note: Available in other traits: {}"#,
			in_other_traits.join(", ")
		));
	}

	message.push_str(
		r#"
  = help: Mark one as default with #[document_default], or use explicit #[document_use = "AssocName"]"#,
	);

	Error::new(span, message)
}

fn create_missing_assoc_type_error(
	span: proc_macro2::Span,
	self_ty_path: &str,
	assoc_name: &str,
	trait_path: Option<&str>,
	config: &Config,
) -> Error {
	let (in_this_impl, in_other_traits) =
		get_available_types_for_brand(config, self_ty_path, trait_path);

	let mut message = format!("Cannot resolve `Self::{assoc_name}` for type `{self_ty_path}`");

	let all_available: Vec<String> =
		in_this_impl.iter().chain(in_other_traits.iter()).cloned().collect();

	if !all_available.is_empty() {
		message.push_str(&format!(
			r#"
  = note: Available associated types: {}"#,
			all_available.join(", ")
		));
	} else {
		message.push_str(
			r#"
  = note: No associated types found for this type"#,
		);
	}

	message.push_str(&format!(
		r#"
  = help: Add an associated type definition:
    impl_kind! {{{{
        for {self_ty_path} {{{{
            type {assoc_name}<T> = YourType<T>;
        }}}}
    }}}}"#,
	));

	Error::new(span, message)
}
