//! Parsing and emission for the `define_row!` macro.
//!
//! A row over effects that own sub-programs must be a nominal type: the
//! effect brands carry the row as a type parameter, so a self-referencing
//! row type alias is a definition cycle, while the same self-reference
//! through a nominal brand's kind projection is lazy and legal. This macro
//! emits that nominal shape from a member list:
//!
//! - the row brand (a unit struct named by the invocation),
//! - its kind projection to the `Coyoneda`-wrapped `CoproductBrand` chain
//!   over the members (each member is written as a bare effect brand and
//!   wrapped in `CoyonedaBrand` by the emission; members may reference the
//!   row name being defined),
//! - delegating `Functor` and `WrapDrop` impls that forward to the chain.
//!
//! Members are kept in declared order: dispatch over the row is brand-keyed
//! (position-independent), so no canonical sorting is performed and
//! appending a member never disturbs existing positions.

use {
	crate::hkt::{
		AssociatedTypes,
		ImplKindInput,
		generate_name,
		impl_kind_worker,
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Attribute,
		Ident,
		Path,
		Token,
		Type,
		Visibility,
		braced,
		parse::{
			Parse,
			ParseStream,
		},
		spanned::Spanned,
	},
};

syn::custom_keyword!(row);

/// The parsed `define_row!` input.
pub struct RowSpec {
	docs: Vec<Attribute>,
	crate_path: Path,
	vis: Visibility,
	name: Ident,
	members: Vec<Type>,
}

impl Parse for RowSpec {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let mut docs = Vec::new();
		let mut crate_path: Option<Path> = None;
		for attr in attrs {
			if attr.path().is_ident("doc") {
				docs.push(attr);
			} else if attr.path().is_ident("crate_path") {
				if crate_path.is_some() {
					return Err(syn::Error::new(attr.span(), "duplicate `#[crate_path(...)]`"));
				}
				crate_path = Some(attr.parse_args()?);
			} else {
				return Err(syn::Error::new(
					attr.span(),
					"unrecognised attribute in `define_row!`",
				));
			}
		}
		let crate_path = match crate_path {
			Some(path) => path,
			None => syn::parse_quote!(::fp_library),
		};
		let vis: Visibility = input.parse()?;
		input.parse::<row>()?;
		let name: Ident = input.parse()?;
		if docs.is_empty() {
			return Err(syn::Error::new(
				name.span(),
				"every row must carry a doc comment; it is emitted onto the row brand",
			));
		}
		let body;
		braced!(body in input);
		let members: Vec<Type> =
			body.parse_terminated(Type::parse, Token![,])?.into_iter().collect();
		if members.is_empty() {
			return Err(syn::Error::new(
				name.span(),
				"a row must declare at least one member effect brand",
			));
		}
		Ok(RowSpec {
			docs,
			crate_path,
			vis,
			name,
			members,
		})
	}
}

/// Emits the nominal row for a parsed spec.
pub fn define_row_worker(spec: RowSpec) -> syn::Result<TokenStream> {
	let RowSpec {
		docs,
		crate_path: cp,
		vis,
		name,
		members,
	} = spec;

	// The kind trait every projection references, named through the same
	// generator that produces it.
	let kind_signature: AssociatedTypes = syn::parse_quote!(
		type Of<'a, T: 'a>: 'a;
	);
	let kind_trait = generate_name(&kind_signature)?;

	// The Coyoneda-wrapped CoproductBrand chain over the members, in
	// declared order.
	let mut chain = quote!(#cp::brands::CNilBrand);
	for member in members.iter().rev() {
		chain = quote!(#cp::brands::CoproductBrand<#cp::brands::CoyonedaBrand<#member>, #chain>);
	}

	// The derived documentation appended to the row brand.
	let member_list = members
		.iter()
		.map(|member| format!("`{}`", quote!(#member)))
		.collect::<Vec<_>>()
		.join(", ");
	let members_doc = format!(" Row members, in declared order: {member_list}.");

	// The kind projection, emitted through the same worker `impl_kind!` uses.
	let impl_kind_input: ImplKindInput = syn::parse2(quote! {
		impl for #name {
			type Of<'a, A: 'a>: 'a = <#chain as #cp::kinds::#kind_trait>::Of<'a, A>;
		}
	})?;
	// The worker emits the kind traits unqualified (its hand-written call
	// sites glob-import `kinds`); scoping the glob inside an anonymous const
	// keeps the emission self-contained at any invocation site.
	let kind_impl = impl_kind_worker(impl_kind_input)?;
	let kind_impl = quote! {
		const _: () = {
			use #cp::kinds::*;
			#kind_impl
		};
	};

	Ok(quote! {
		#(#docs)*
		#[doc = ""]
		#[doc = #members_doc]
		#vis struct #name;

		#kind_impl

		impl #cp::classes::Functor for #name {
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: <Self as #cp::kinds::#kind_trait>::Of<'a, A>,
			) -> <Self as #cp::kinds::#kind_trait>::Of<'a, B> {
				<#chain as #cp::classes::Functor>::map(f, fa)
			}
		}

		impl #cp::classes::WrapDrop for #name {
			fn drop<'a, X: 'a>(
				fa: <Self as #cp::kinds::#kind_trait>::Of<'a, X>
			) -> Option<X> {
				<#chain as #cp::classes::WrapDrop>::drop(fa)
			}
		}
	})
}
