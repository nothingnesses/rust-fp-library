//! Code generation for the [`define_scoped_row!`](crate::define_scoped_row)
//! item macro.
//!
//! The macro defines a concrete marker row around a canonical
//! `CoproductBrand<..., CNilBrand>` scoped-effect row. Bare `Self`
//! type placeholders inside the row body are replaced with the marker
//! row before canonical sorting, which lets recursive scoped
//! constructor brands refer back to the enclosing row without manually
//! writing the marker name in every occurrence.

use {
	crate::{
		effects::row_sort::sort_types_unique,
		hkt::{
			AssociatedTypes,
			generate_name,
		},
	},
	proc_macro2::{
		Span,
		TokenStream,
	},
	quote::quote,
	syn::{
		Ident,
		PathArguments,
		Token,
		Type,
		Visibility,
		bracketed,
		parse::{
			Parse,
			ParseStream,
		},
		punctuated::Punctuated,
		visit_mut::{
			self,
			VisitMut,
		},
	},
};

struct DefineScopedRowInput {
	vis: Visibility,
	ident: Ident,
	types: Vec<Type>,
}

impl Parse for DefineScopedRowInput {
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let vis: Visibility = input.parse()?;
		input.parse::<Token![struct]>()?;
		let ident: Ident = input.parse()?;
		if input.peek(Token![<]) {
			return Err(
				input.error("generic scoped rows are deferred; define a concrete row marker here")
			);
		}
		input.parse::<Token![;]>()?;

		let content;
		bracketed!(content in input);
		let types =
			Punctuated::<Type, Token![,]>::parse_terminated(&content)?.into_iter().collect();

		Ok(Self {
			vis,
			ident,
			types,
		})
	}
}

/// Worker for the public `define_scoped_row!` macro.
pub fn define_scoped_row_worker(input: TokenStream) -> syn::Result<TokenStream> {
	let DefineScopedRowInput {
		vis,
		ident,
		types,
	} = syn::parse2(input)?;
	let kind_trait = effect_kind_trait()?;
	let sorted = sort_types_unique(types.into_iter().map(|mut ty| {
		ReplaceBareSelf {
			replacement: &ident,
		}
		.visit_type_mut(&mut ty);
		ty
	}))?;
	let underlying_row = build_coproduct_row(sorted);

	Ok(quote! {
		#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		#vis struct #ident;

		impl ::fp_library::kinds::#kind_trait for #ident {
			type Of<'a, A: 'a> =
				<#underlying_row as ::fp_library::kinds::#kind_trait>::Of<'a, A>;
		}

		impl ::fp_library::classes::WrapDrop for #ident {
			fn drop<'a, X: 'a>(
				fa: <#ident as ::fp_library::kinds::#kind_trait>::Of<'a, X>,
			) -> ::core::option::Option<X> {
				<#underlying_row as ::fp_library::classes::WrapDrop>::drop(fa)
			}
		}

		impl ::fp_library::classes::Functor for #ident {
			fn map<'a, A: 'a, B: 'a>(
				func: impl Fn(A) -> B + 'a,
				fa: <#ident as ::fp_library::kinds::#kind_trait>::Of<'a, A>,
			) -> <#ident as ::fp_library::kinds::#kind_trait>::Of<'a, B> {
				<#underlying_row as ::fp_library::classes::Functor>::map(func, fa)
			}
		}

		impl ::fp_library::classes::SendFunctor for #ident {
			fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
				func: impl Fn(A) -> B + Send + Sync + 'a,
				fa: <#ident as ::fp_library::kinds::#kind_trait>::Of<'a, A>,
			) -> <#ident as ::fp_library::kinds::#kind_trait>::Of<'a, B> {
				<#underlying_row as ::fp_library::classes::SendFunctor>::send_map(func, fa)
			}
		}

		impl ::fp_library::classes::RefFunctor for #ident
		where
			#underlying_row: ::fp_library::classes::RefFunctor,
		{
			fn ref_map<'a, A: 'a, B: 'a>(
				func: impl Fn(&A) -> B + 'a,
				fa: &<#ident as ::fp_library::kinds::#kind_trait>::Of<'a, A>,
			) -> <#ident as ::fp_library::kinds::#kind_trait>::Of<'a, B> {
				<#underlying_row as ::fp_library::classes::RefFunctor>::ref_map(func, fa)
			}
		}
	})
}

fn effect_kind_trait() -> syn::Result<Ident> {
	let input: AssociatedTypes = syn::parse_quote! {
		type Of<'a, T: 'a>: 'a;
	};
	generate_name(&input).map_err(|e| syn::Error::new(Span::call_site(), e.to_string()))
}

struct ReplaceBareSelf<'a> {
	replacement: &'a Ident,
}

impl VisitMut for ReplaceBareSelf<'_> {
	fn visit_type_mut(
		&mut self,
		ty: &mut Type,
	) {
		if is_bare_self_type(ty) {
			let replacement = self.replacement;
			*ty = syn::parse_quote!(#replacement);
			return;
		}

		visit_mut::visit_type_mut(self, ty);
	}
}

fn is_bare_self_type(ty: &Type) -> bool {
	let Type::Path(type_path) = ty else {
		return false;
	};
	if type_path.qself.is_some()
		|| type_path.path.leading_colon.is_some()
		|| type_path.path.segments.len() != 1
	{
		return false;
	}
	let Some(segment) = type_path.path.segments.first() else {
		return false;
	};
	segment.ident == "Self" && matches!(segment.arguments, PathArguments::None)
}

fn build_coproduct_row(sorted: Vec<Type>) -> TokenStream {
	let mut acc: TokenStream = quote! { ::fp_library::brands::CNilBrand };
	for ty in sorted.into_iter().rev() {
		acc = quote! {
			::fp_library::brands::CoproductBrand<#ty, #acc>
		};
	}
	acc
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use super::*;

	#[test]
	fn emits_marker_struct_and_trait_impls() {
		let out = define_scoped_row_worker(quote! {
			pub struct Row;
			[BoxCatchBrand<BoxBrand, Error>]
		})
		.expect("worker failed")
		.to_string();

		assert!(out.contains("pub struct Row"));
		assert!(out.contains("impl :: fp_library :: kinds :: Kind_"));
		assert!(out.contains("impl :: fp_library :: classes :: WrapDrop for Row"));
		assert!(out.contains("impl :: fp_library :: classes :: Functor for Row"));
		assert!(out.contains("impl :: fp_library :: classes :: SendFunctor for Row"));
		assert!(out.contains("impl :: fp_library :: classes :: RefFunctor for Row"));
	}

	#[test]
	fn replaces_bare_self_before_output() {
		let out = define_scoped_row_worker(quote! {
			struct Row;
			[BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, Self>, i32, i32>]
		})
		.expect("worker failed")
		.to_string();

		assert!(out.contains("NodeBrand < CNilBrand , Row >"));
		assert!(!out.contains("NodeBrand < CNilBrand , Self >"));
	}

	#[test]
	fn canonical_order_independent_of_input() {
		let a = define_scoped_row_worker(quote! {
			struct Row;
			[
				BoxCatchBrand<BoxBrand, Error>,
				BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, Self>, i32, i32>,
			]
		})
		.expect("worker failed")
		.to_string();
		let b = define_scoped_row_worker(quote! {
			struct Row;
			[
				BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, Self>, i32, i32>,
				BoxCatchBrand<BoxBrand, Error>,
			]
		})
		.expect("worker failed")
		.to_string();

		assert_eq!(a, b);
	}

	#[test]
	fn generic_rows_are_rejected_for_now() {
		let err = define_scoped_row_worker(quote! {
			struct Row<T>;
			[]
		})
		.expect_err("generic rows should be deferred");

		assert!(err.to_string().contains("generic scoped rows are deferred"));
	}

	#[test]
	fn duplicate_scoped_row_entries_are_rejected() {
		let err = define_scoped_row_worker(quote! {
			struct Row;
			[
				SpanBrand<Self>,
				(SpanBrand<Self>),
			]
		})
		.expect_err("duplicate scoped row entry should fail");

		assert!(err.to_string().contains("duplicate row entry"));
	}
}
