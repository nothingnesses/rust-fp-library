//! Implementation of the `impl_kind!` macro.
//!
//! This module handles the parsing and expansion of the `impl_kind!` macro, which is used
//! to implement a generated `Kind` trait for a specific brand type.

use {
	super::{
		AssociatedType as AssociatedTypeInput,
		AssociatedTypeBase,
		AssociatedTypes,
		generate_inferable_brand_name,
		generate_name,
	},
	crate::{
		core::Result,
		support::{
			attributes,
			parsing::{
				parse_many,
				parse_non_empty,
			},
		},
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		GenericParam,
		Generics,
		Token,
		Type,
		WhereClause,
		braced,
		parse::{
			Parse,
			ParseStream,
		},
		visit::Visit,
	},
};

/// Input structure for the `impl_kind!` macro.
///
/// Parses syntax like:
/// ```ignore
/// impl<T> for MyBrand {
///     type Of<A> = MyType<A>;
///     type SendOf<B> = MySendType<B>;
/// }
/// ```
pub struct ImplKindInput {
	/// Attributes (including doc comments) for the impl block.
	pub attributes: Vec<syn::Attribute>,
	/// Generics for the impl block (e.g., `impl<T>`).
	pub impl_generics: Generics,
	/// The `for` keyword.
	pub _for_token: Token![for],
	/// The brand type being implemented (e.g., `MyBrand`).
	pub brand: Type,
	/// The brace token surrounding the associated type definitions.
	pub _brace_token: syn::token::Brace,
	/// The associated type definitions inside the braces.
	pub definitions: Vec<AssociatedType>,
}

/// Represents a single associated type definition inside `impl_kind!`.
///
/// Example: `type Of<A> = MyType<A>;`
pub struct AssociatedType {
	/// The common signature parts.
	pub signature: AssociatedTypeBase,
	/// The `=` token.
	pub _eq_token: Token![=],
	/// The concrete type being assigned (e.g., `MyType<A>`).
	pub target_type: Type,
	/// Optional where clause.
	pub where_clause: Option<WhereClause>,
	/// The semicolon.
	pub semi_token: Token![;],
}

impl Parse for ImplKindInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attributes = input.call(syn::Attribute::parse_outer)?;

		let mut impl_generics = if input.peek(Token![impl]) {
			input.parse::<Token![impl]>()?;
			input.parse::<Generics>()?
		} else {
			Generics::default()
		};

		let for_token: Token![for] = input.parse()?;
		let brand: Type = input.parse()?;

		// Parse where clause if present (comes after brand, before braces)
		if input.peek(Token![where]) {
			impl_generics.where_clause = Some(input.parse()?);
		}

		let content;
		let brace_token = braced!(content in input);

		let definitions = parse_many(&content)?;
		let definitions = parse_non_empty(
			definitions,
			"Kind implementation must have at least one associated type definition",
		)?;

		Ok(ImplKindInput {
			attributes,
			impl_generics,
			_for_token: for_token,
			brand,
			_brace_token: brace_token,
			definitions,
		})
	}
}

impl Parse for AssociatedType {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let signature =
			AssociatedTypeBase::parse_signature(input, |i| i.peek(Token![=]) || i.peek(Token![;]))?;

		let eq_token: Token![=] = input.parse()?;
		let target_type: Type = input.parse()?;

		let where_clause: Option<WhereClause> =
			if input.peek(Token![where]) { Some(input.parse()?) } else { None };

		let semi_token: Token![;] = input.parse()?;

		Ok(AssociatedType {
			signature,
			_eq_token: eq_token,
			target_type,
			where_clause,
			semi_token,
		})
	}
}

/// Checks whether the target type is a projection type that should not
/// receive InferableBrand impls.
///
/// A projection type is one that:
/// - Contains an `Apply!` macro invocation (detected as `syn::Type::Macro`)
/// - Contains a qualified path with `::` (detected as a multi-segment path
///   or a path with a `QSelf`)
///
/// Uses structural AST checks rather than string heuristics (Decision S).
fn is_projection_type(ty: &Type) -> bool {
	struct ProjectionChecker {
		found: bool,
	}

	impl<'ast> Visit<'ast> for ProjectionChecker {
		fn visit_type_macro(
			&mut self,
			_: &'ast syn::TypeMacro,
		) {
			self.found = true;
		}

		fn visit_type_path(
			&mut self,
			type_path: &'ast syn::TypePath,
		) {
			if type_path.path.segments.len() > 1 || type_path.qself.is_some() {
				self.found = true;
			}
			syn::visit::visit_type_path(self, type_path);
		}
	}

	let mut checker = ProjectionChecker {
		found: false,
	};
	checker.visit_type(ty);
	checker.found
}

/// Checks whether a `InferableBrand` impl should be generated for this `impl_kind!`
/// invocation.
///
/// Returns `false` (skip generation) when:
/// - Multiple associated types are defined (ambiguous primary type)
/// - The target type is a projection (contains `Apply!` or `::`)
///
/// The `#[multi_brand]` attribute does NOT suppress InferableBrand generation.
/// Multi-brand types get InferableBrand impls (one per brand); the attribute
/// is a documentation marker only.
fn should_generate_slot(input: &ImplKindInput) -> bool {
	// Skip if multiple associated types (ambiguous primary type)
	if input.definitions.len() != 1 {
		return false;
	}

	// Skip if target type is a projection
	let Some(def) = input.definitions.first() else {
		return false;
	};
	if is_projection_type(&def.target_type) {
		return false;
	}

	true
}

/// Generates the implementation for the `impl_kind!` macro.
///
/// This function takes the parsed input, determines the correct `Kind` trait based on
/// the signature of the associated types, and generates the `impl` block.
///
/// By default, it also generates:
/// - A `InferableBrand_{hash}` impl with `type Marker = Val` (unless the target is a projection
///   type or multiple associated types are defined).
pub fn impl_kind_worker(input: ImplKindInput) -> Result<TokenStream> {
	let brand = &input.brand;
	let impl_generics = &input.impl_generics;

	// Convert to KindInput for name generation
	let kind_input = AssociatedTypes {
		associated_types: input
			.definitions
			.iter()
			.map(|def| AssociatedTypeInput {
				signature: def.signature.clone(),
				semi_token: def.semi_token,
			})
			.collect(),
	};
	let kind_trait_name = generate_name(&kind_input)?;

	let assoc_types_impl = input.definitions.iter().map(|def| {
		let ident = &def.signature.name;
		let generics = &def.signature.generics;
		let target = &def.target_type;
		let where_clause = &def.where_clause;
		// Filter out documentation-specific attributes to avoid "unused attribute" warnings
		let attrs = attributes::filter_doc_attributes(&def.signature.attributes);

		quote! {
			#(#attrs)*
			type #ident #generics = #target #where_clause;
		}
	});

	// Generate doc comment
	let doc_comment =
		format!("Generated implementation of `{kind_trait_name}` for `{}`.", quote!(#brand));

	let (impl_generics_impl, _, impl_generics_where) = impl_generics.split_for_impl();

	// Filter out #[multi_brand] from the attributes passed to the Kind impl
	let attrs: Vec<_> =
		input.attributes.iter().filter(|attr| !attr.path().is_ident("multi_brand")).collect();
	let has_doc = attrs.iter().any(|attr| attr.path().is_ident("doc"));
	let maybe_separator = if has_doc {
		quote! { #[doc = ""] }
	} else {
		quote! {}
	};

	let kind_impl = quote! {
		#[doc = #doc_comment]
		#maybe_separator
		#(#attrs)*
		impl #impl_generics_impl #kind_trait_name for #brand #impl_generics_where {
			#(#assoc_types_impl)*
		}
	};

	// Generate InferableBrand impl if applicable.
	// InferableBrand is generated for ALL brands (including multi-brand types).
	// The #[multi_brand] attribute does not suppress InferableBrand generation.
	// Each impl_kind! invocation produces at most one
	// InferableBrand impl; multiple brands for the same concrete type come from
	// multiple impl_kind! invocations.
	//
	// Marker-agreement invariant: all InferableBrand impls for a given Self type
	// must agree on the same Marker value. Since impl_kind! always
	// generates owned-type impls (not reference impls), Marker is always
	// Val. The &T blanket (generated by trait_kind!) handles Ref.
	let slot_impl = if should_generate_slot(&input)
		&& let Some(def) = input.definitions.first()
	{
		let inferable_brand_name = generate_inferable_brand_name(&kind_input)?;
		let target_type = &def.target_type;
		let assoc_generics = &def.signature.generics;

		// Extract lifetime and type params from the associated type generics
		let lifetime_names: Vec<_> =
			assoc_generics
				.params
				.iter()
				.filter_map(|p| {
					if let GenericParam::Lifetime(lt) = p { Some(&lt.lifetime) } else { None }
				})
				.collect();

		let type_idents: Vec<_> = assoc_generics
			.params
			.iter()
			.filter_map(|p| if let GenericParam::Type(tp) = p { Some(&tp.ident) } else { None })
			.collect();

		// InferableBrand impl generics: all assoc type generics + all impl generics.
		// The assoc type generics include lifetimes and type params with
		// their bounds (e.g., 'a, A: 'a). The impl generics include the
		// brand's generic params (e.g., E: 'static).
		let all_slot_params: Vec<_> =
			assoc_generics.params.iter().chain(impl_generics.params.iter()).collect();

		let slot_doc = format!(
			"Generated `{inferable_brand_name}` implementation for `{}` with brand `{}`.",
			quote!(#target_type),
			quote!(#brand),
		);

		quote! {
			#[doc = #slot_doc]
			#[expect(non_camel_case_types, reason = "Generated name uses hash suffix for uniqueness")]
			impl<#(#all_slot_params),*>
				#inferable_brand_name<#(#lifetime_names,)* #brand #(, #type_idents)*>
			for #target_type
			#impl_generics_where
			{
				type Marker = ::fp_library::dispatch::Val;
			}
		}
	} else {
		quote! {}
	};

	Ok(quote! {
		#kind_impl
		#slot_impl
	})
}

#[cfg(test)]
#[expect(
	clippy::indexing_slicing,
	clippy::expect_used,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use super::*;

	// ===========================================================================
	// impl_kind! Parsing and Generation Tests
	// ===========================================================================

	#[test]
	fn test_parse_impl_kind_simple() {
		let input = "for OptionBrand { type Of<A> = Option<A>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		assert_eq!(parsed.definitions.len(), 1);
		assert_eq!(parsed.definitions[0].signature.name.to_string(), "Of");
	}

	#[test]
	fn test_parse_impl_kind_multiple() {
		let input = "for MyBrand {
			type Of<A> = MyType<A>;
			type SendOf<B> = MySendType<B>;
		}";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		assert_eq!(parsed.definitions.len(), 2);
		assert_eq!(parsed.definitions[0].signature.name.to_string(), "Of");
		assert_eq!(parsed.definitions[1].signature.name.to_string(), "SendOf");
	}

	#[test]
	fn test_impl_kind_generation() {
		let input = "for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl Kind_"));
		assert!(output_str.contains("for OptionBrand"));
		assert!(output_str.contains("type Of < 'a , A : 'a > = Option < A >"));
	}

	// ===========================================================================
	// impl_kind! with generics Tests
	// ===========================================================================

	#[test]
	fn test_impl_kind_with_impl_generics() {
		let input = "impl<E> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
	}

	#[test]
	fn test_impl_kind_with_multiple_impl_generics() {
		let input = "impl<E: Clone, F: Send> for MyBrand<E, F> { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E : Clone , F : Send > Kind_"));
		assert!(output_str.contains("for MyBrand < E , F >"));
	}

	#[test]
	fn test_impl_kind_with_bounded_impl_generic() {
		let input = "impl<E: std::fmt::Debug> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E : std :: fmt :: Debug > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
	}

	// ===========================================================================
	// impl_kind! with where clauses Tests
	// ===========================================================================

	#[test]
	fn test_impl_kind_with_where_clause() {
		let input =
			"impl<E> for ResultBrand<E> where E: std::fmt::Debug { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
		assert!(output_str.contains("where E : std :: fmt :: Debug"));
	}

	#[test]
	fn test_impl_kind_with_multiple_where_bounds() {
		let input = "impl<E, F> for MyBrand<E, F> where E: Clone, F: Send { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E , F >"));
		assert!(output_str.contains("where E : Clone , F : Send"));
	}
}
