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
	std::collections::HashSet,
	syn::{
		GenericParam,
		Generics,
		Token,
		Type,
		TypeParamBound,
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

/// Collects all identifier and lifetime names referenced in a type expression.
struct TypeIdentCollector {
	idents: HashSet<String>,
	lifetimes: HashSet<String>,
}

impl TypeIdentCollector {
	fn new() -> Self {
		Self {
			idents: HashSet::new(),
			lifetimes: HashSet::new(),
		}
	}

	fn collect(ty: &Type) -> Self {
		let mut collector = Self::new();
		collector.visit_type(ty);
		collector
	}
}

impl<'ast> Visit<'ast> for TypeIdentCollector {
	fn visit_path(
		&mut self,
		path: &'ast syn::Path,
	) {
		for segment in &path.segments {
			self.idents.insert(segment.ident.to_string());
			syn::visit::visit_path_segment(self, segment);
		}
	}

	fn visit_lifetime(
		&mut self,
		lifetime: &'ast syn::Lifetime,
	) {
		self.lifetimes.insert(lifetime.ident.to_string());
	}

	fn visit_type_tuple(
		&mut self,
		tuple: &'ast syn::TypeTuple,
	) {
		for elem in &tuple.elems {
			self.visit_type(elem);
		}
	}
}

/// Checks whether an `InferableBrand` impl should be generated for this
/// `impl_kind!` invocation.
///
/// Returns `false` (skip generation) when:
/// - `#[no_inferable_brand]` attribute is present
/// - Multiple associated types are defined (ambiguous primary type)
/// - The target type is a projection (contains `Apply!` or `::`)
fn should_generate_inferable_brand(input: &ImplKindInput) -> bool {
	// Check for #[no_inferable_brand] attribute
	if input.attributes.iter().any(|attr| attr.path().is_ident("no_inferable_brand")) {
		return false;
	}

	// Skip if multiple associated types (ambiguous primary type)
	if input.definitions.len() != 1 {
		return false;
	}

	// Skip if target type is a projection (contains Apply! or ::)
	let Some(def) = input.definitions.first() else {
		return false;
	};
	let target = &def.target_type;
	let target_str = quote!(#target).to_string();
	if target_str.contains("::") || target_str.contains("Apply") {
		return false;
	}

	true
}

/// Builds the generics for an `InferableBrand` impl by collecting only the
/// generic parameters that appear in the target type, with appropriate bounds.
fn build_inferable_brand_generics(
	target_type: &Type,
	assoc_generics: &Generics,
	impl_generics: &Generics,
) -> Generics {
	let collector = TypeIdentCollector::collect(target_type);

	// Collect lifetimes from the associated type's output bounds that appear
	// in the target type. These are used to add lifetime bounds on impl params.
	let output_lifetimes_in_target: HashSet<String> = assoc_generics
		.params
		.iter()
		.filter_map(|p| {
			if let GenericParam::Lifetime(lt) = p {
				let name = lt.lifetime.ident.to_string();
				if collector.lifetimes.contains(&name) { Some(name) } else { None }
			} else {
				None
			}
		})
		.collect();

	let mut params = syn::punctuated::Punctuated::new();

	// Add lifetimes from assoc generics that appear in target type
	for param in &assoc_generics.params {
		if let GenericParam::Lifetime(lt) = param
			&& collector.lifetimes.contains(&lt.lifetime.ident.to_string())
		{
			params.push(param.clone());
		}
	}

	// Add type params from assoc generics that appear in target type,
	// stripping lifetime bounds that reference lifetimes not in the target
	for param in &assoc_generics.params {
		if let GenericParam::Type(ty) = param
			&& collector.idents.contains(&ty.ident.to_string())
		{
			let mut ty = ty.clone();
			ty.bounds = ty
				.bounds
				.into_iter()
				.filter(|bound| {
					if let TypeParamBound::Lifetime(lt) = bound {
						collector.lifetimes.contains(&lt.ident.to_string())
					} else {
						true
					}
				})
				.collect();
			params.push(GenericParam::Type(ty));
		}
	}

	// Add lifetimes from impl generics that appear in target type
	for param in &impl_generics.params {
		if let GenericParam::Lifetime(lt) = param
			&& collector.lifetimes.contains(&lt.lifetime.ident.to_string())
		{
			params.push(param.clone());
		}
	}

	// Add type params from impl generics that appear in target type,
	// with additional lifetime bounds from output lifetimes
	for param in &impl_generics.params {
		if let GenericParam::Type(ty) = param
			&& collector.idents.contains(&ty.ident.to_string())
		{
			let mut ty = ty.clone();
			// Add lifetime bounds for output lifetimes that appear in target
			for lt_name in &output_lifetimes_in_target {
				let lt = syn::Lifetime::new(&format!("'{lt_name}"), proc_macro2::Span::call_site());
				ty.bounds.push(TypeParamBound::Lifetime(lt));
			}
			params.push(GenericParam::Type(ty));
		}
	}

	let has_params = !params.is_empty();
	Generics {
		lt_token: if has_params { Some(Default::default()) } else { None },
		params,
		gt_token: if has_params { Some(Default::default()) } else { None },
		where_clause: None,
	}
}

/// Generates the implementation for the `impl_kind!` macro.
///
/// This function takes the parsed input, determines the correct `Kind` trait based on
/// the signature of the associated types, and generates the `impl` block.
///
/// By default, it also generates a corresponding `InferableBrand_{hash}` impl for the
/// target type, enabling brand inference. This is suppressed when:
/// - `#[no_inferable_brand]` is present
/// - The target type is a projection (contains `Apply!` or `::`)
/// - Multiple associated types are defined
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

	// Filter out #[no_inferable_brand] from the attributes passed to the Kind impl
	let attrs: Vec<_> = input
		.attributes
		.iter()
		.filter(|attr| !attr.path().is_ident("no_inferable_brand"))
		.collect();
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

	// Generate InferableBrand impl if applicable
	let ib_impl = if should_generate_inferable_brand(&input)
		&& let Some(def) = input.definitions.first()
	{
		let ib_trait_name = generate_inferable_brand_name(&kind_input)?;
		let target_type = &def.target_type;
		let ib_generics =
			build_inferable_brand_generics(target_type, &def.signature.generics, impl_generics);
		let (ib_impl_generics, ..) = ib_generics.split_for_impl();

		let ib_doc = format!(
			"Generated `{ib_trait_name}` implementation mapping `{}` back to `{}`.",
			quote!(#target_type),
			quote!(#brand),
		);

		quote! {
			#[doc = #ib_doc]
			impl #ib_impl_generics #ib_trait_name for #target_type {
				type Brand = #brand;
			}
		}
	} else {
		quote! {}
	};

	Ok(quote! {
		#kind_impl
		#ib_impl
	})
}

#[cfg(test)]
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
