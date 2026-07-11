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
//!
//! Under the opt-in `#[handlers]` attribute the row also gets its one-pass
//! handler surface, composed from the per-effect pieces `define_effect!`
//! emits (the row macro cannot see the effects' operation inventories, so
//! everything crosses the seam by path-resolved projection): the handler
//! struct (one arm-bundle field per cell at the member's `Arms`
//! projection), the row abort enum (one variant per cell, injected by the
//! loop passing each cell's variant constructor to its dispatch), and the
//! `RowHandler` loop (brand-keyed `uninject` dispatch to the members'
//! dispatch functions).
//! The surface is opt-in because it requires every member to carry the
//! emitted `HandlerPieces` impl, which a hand-written cell may lack.

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
	handlers: bool,
	vis: Visibility,
	name: Ident,
	members: Vec<Type>,
}

impl Parse for RowSpec {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let mut docs = Vec::new();
		let mut crate_path: Option<Path> = None;
		let mut handlers = false;
		for attr in attrs {
			if attr.path().is_ident("doc") {
				docs.push(attr);
			} else if attr.path().is_ident("crate_path") {
				if crate_path.is_some() {
					return Err(syn::Error::new(attr.span(), "duplicate `#[crate_path(...)]`"));
				}
				crate_path = Some(attr.parse_args()?);
			} else if attr.path().is_ident("handlers") {
				if handlers {
					return Err(syn::Error::new(attr.span(), "duplicate `#[handlers]`"));
				}
				attr.meta.require_path_only()?;
				handlers = true;
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
			handlers,
			vis,
			name,
			members,
		})
	}
}

/// Derives the stable stem a handler field and abort variant are named
/// after: the member's last path segment with any `Brand` suffix stripped.
fn member_stem(member: &Type) -> syn::Result<Ident> {
	let Type::Path(path) = member else {
		return Err(syn::Error::new(
			member.span(),
			"a `#[handlers]` row member must be written as a brand type path so a handler field name can be derived from it",
		));
	};
	let segment = path.path.segments.last().ok_or_else(|| {
		syn::Error::new(member.span(), "a `#[handlers]` row member path must not be empty")
	})?;
	let text = segment.ident.to_string();
	let stem = text.strip_suffix("Brand").unwrap_or(&text);
	if stem.is_empty() {
		return Err(syn::Error::new(
			segment.ident.span(),
			"a `#[handlers]` row member must carry a name besides the `Brand` suffix",
		));
	}
	Ok(Ident::new(stem, segment.ident.span()))
}

/// Converts an UpperCamelCase stem into the snake_case handler field name.
fn snake_case(stem: &Ident) -> Ident {
	let mut out = String::new();
	for (index, ch) in stem.to_string().chars().enumerate() {
		if ch.is_uppercase() {
			if index != 0 {
				out.push('_');
			}
			out.extend(ch.to_lowercase());
		} else {
			out.push(ch);
		}
	}
	Ident::new(&out, stem.span())
}

/// Emits the nominal row for a parsed spec.
pub fn define_row_worker(spec: RowSpec) -> syn::Result<TokenStream> {
	let RowSpec {
		docs,
		crate_path: cp,
		handlers,
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

	// ---- The handler surface (opt-in): the one-pass loop over the row ----
	// Emitted only under `#[handlers]`, because it requires every member to
	// carry the `HandlerPieces` impl `define_effect!` emits, which a
	// hand-written cell may lack.
	let handlers_emission: Option<TokenStream> = if handlers {
		let stems: Vec<Ident> = members.iter().map(member_stem).collect::<syn::Result<_>>()?;
		let mut seen: Vec<&Ident> = Vec::new();
		for (member, stem) in members.iter().zip(&stems) {
			if seen.contains(&stem) {
				return Err(syn::Error::new(
					member.span(),
					format!(
						"row members derive the duplicate handler name `{stem}`; a `#[handlers]` row cannot hold two cells of the same effect until labelled (tagged) effects exist",
					),
				));
			}
			seen.push(stem);
		}
		let fields: Vec<Ident> = stems.iter().map(snake_case).collect();
		let handlers_name = quote::format_ident!("{}Handlers", name);
		let abort_name = quote::format_ident!("{}Abort", name);
		let handle_path = quote!(#cp::types::effects::handle);
		let field_docs: Vec<String> = members
			.iter()
			.map(|member| format!(" The `{}` cell's arms.", quote!(#member)))
			.collect();
		let variant_docs: Vec<String> = members
			.iter()
			.map(|member| format!(" The `{}` cell's abort.", quote!(#member)))
			.collect();
		let handlers_doc = format!(
			" The handler list for [`{name}`]: one arm-bundle field per cell, over handler state borrowed for `'h`. Construction is a named-field struct literal, so it is order-insensitive and a misspelled cell name is a compile error.",
		);
		let abort_doc = format!(
			" The abort union for [`{name}`]: one variant per cell, carrying that cell's abort payloads. A variant whose cell aborts on no operation is uninhabited and can only be matched absurdly.",
		);
		let steps: Vec<TokenStream> = members
			.iter()
			.zip(fields.iter().zip(&stems))
			.map(|(member, (field, stem))| {
				quote! {
					let selected: ::core::result::Result<
						#cp::types::Coyoneda<'static, #member, #cp::types::Free<#name, T>>,
						_,
					> = layer.uninject();
					let layer = match selected {
						Ok(coyo) => {
							program = <#member as #handle_path::HandlerPieces<
								#name,
								#abort_name,
							>>::dispatch(coyo.lower(), &self.#field, self, #abort_name::#stem)?;
							continue;
						}
						Err(rest) => rest,
					};
				}
			})
			.collect();
		Some(quote! {
			#[doc = #handlers_doc]
			#vis struct #handlers_name<'h> {
				#(
					#[doc = #field_docs]
					#vis #fields: <#members as #handle_path::HandlerPieces<#name, #abort_name>>::Arms<'h>,
				)*
			}

			#[doc = #abort_doc]
			#vis enum #abort_name {
				#(
					#[doc = #variant_docs]
					#stems(<#members as #handle_path::EffectAbort>::Abort),
				)*
			}

			impl<'h> #handle_path::RowHandler<#name, #abort_name> for #handlers_name<'h> {
				fn handle<T: 'static>(
					&self,
					program: #cp::types::Free<#name, T>,
				) -> ::core::result::Result<T, #abort_name> {
					let mut program = program;
					loop {
						let layer = match program.resume() {
							Ok(value) => return Ok(value),
							Err(layer) => layer,
						};
						#(#steps)*
						match layer {}
					}
				}
			}
		})
	} else {
		None
	};

	Ok(quote! {
		#(#docs)*
		#[doc = ""]
		#[doc = #members_doc]
		#vis struct #name;

		#handlers_emission

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
