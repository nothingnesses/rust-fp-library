use crate::{
	apply::ApplyInput,
	doc_utils::{DocArg, GenericArgs, validate_doc_args},
	function_utils::load_config,
	hm_signature::generate_signature,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	Attribute, GenericParam, ImplItem, ItemImpl, Result, Signature, Type,
	parse::{Parse, ParseStream},
	parse_quote,
	visit_mut::{self, VisitMut},
};

pub struct DocumentImplInput {
	pub doc_type_params: Option<GenericArgs>,
}

impl Parse for DocumentImplInput {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.is_empty() {
			return Ok(DocumentImplInput { doc_type_params: None });
		}

		// Look for doc_type_params(...)
		let ident: syn::Ident = input.parse()?;
		if ident != "doc_type_params" {
			return Err(syn::Error::new(ident.span(), "Expected doc_type_params"));
		}

		let inner_content;
		syn::parenthesized!(inner_content in input);
		let args: GenericArgs = inner_content.parse()?;

		Ok(DocumentImplInput { doc_type_params: Some(args) })
	}
}

pub fn document_impl_impl(
	attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	let input = match syn::parse2::<DocumentImplInput>(attr) {
		Ok(i) => i,
		Err(e) => return e.to_compile_error(),
	};

	let mut item_impl = match syn::parse2::<ItemImpl>(item) {
		Ok(i) => i,
		Err(e) => return e.to_compile_error(),
	};

	if let Err(e) = process_document_impl(&input, &mut item_impl) {
		return e.to_compile_error();
	}

	quote!(#item_impl)
}

fn process_document_impl(
	input: &DocumentImplInput,
	item_impl: &mut ItemImpl,
) -> Result<()> {
	let trait_path = item_impl.trait_.as_ref().map(|(_, path, _)| path);
	let trait_name = trait_path.map(|p| p.segments.last().unwrap().ident.to_string());
	let self_ty = &*item_impl.self_ty;

	let config = load_config();

	for item in &mut item_impl.items {
		if let ImplItem::Fn(method) = item {
			// 1. Handle HM Signature
			if let Some(attr_pos) = find_attribute(&method.attrs, "hm_signature") {
				method.attrs.remove(attr_pos);

				let mut synthetic_sig = method.sig.clone();

				// Substitute Self
				let mut substitutor = SelfSubstitutor { self_ty };
				substitutor.visit_signature_mut(&mut synthetic_sig);

				// Merge generics
				merge_generics(&mut synthetic_sig, &item_impl.generics);

				// Add trait bound: SelfTy: Trait (only if it's a trait impl)
				if let Some(trait_path) = trait_path {
					let where_clause = synthetic_sig.generics.make_where_clause();
					where_clause.predicates.push(parse_quote!(#self_ty: #trait_path));
				}

				let signature_data =
					generate_signature(&synthetic_sig, trait_name.as_deref(), &config);
				let doc_comment = format!("`{}`", signature_data);
				let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
				method.attrs.insert(attr_pos, doc_attr);
			}

			// 2. Handle Doc Type Params
			if let Some(mut current_pos) = find_attribute(&method.attrs, "doc_type_params")
				&& let Some(impl_doc_args) = &input.doc_type_params
			{
				let impl_generics = &item_impl.generics;
				let targets: Vec<String> = impl_generics
					.params
					.iter()
					.map(|p| match p {
						GenericParam::Type(t) => t.ident.to_string(),
						GenericParam::Lifetime(l) => l.lifetime.ident.to_string(),
						GenericParam::Const(c) => c.ident.to_string(),
					})
					.collect();

				let entries: Vec<_> = impl_doc_args.entries.iter().collect();
				validate_doc_args(targets.len(), entries.len(), Span::call_site())?;

				for (name_from_target, entry) in targets.iter().zip(entries) {
					let (name, desc) = match entry {
						DocArg::Override(n, d) => (n.value(), d.value()),
						DocArg::Desc(d) => (name_from_target.clone(), d.value()),
					};

					let doc_comment = format!("* `{}`: {}", name, desc);
					let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
					method.attrs.insert(current_pos, doc_attr);
					current_pos += 1;
				}
			}
		}
	}

	Ok(())
}

fn find_attribute(
	attrs: &[Attribute],
	name: &str,
) -> Option<usize> {
	attrs.iter().position(|attr| attr.path().is_ident(name))
}

fn merge_generics(
	sig: &mut Signature,
	impl_generics: &syn::Generics,
) {
	// Lifetimes must come first
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

struct SelfSubstitutor<'a> {
	self_ty: &'a Type,
}

impl<'a> VisitMut for SelfSubstitutor<'a> {
	fn visit_type_mut(
		&mut self,
		i: &mut Type,
	) {
		if let Type::Path(tp) = i {
			if tp.path.is_ident("Self") {
				*i = self.self_ty.clone();
			} else {
				// Handle Self::Assoc
				if let Some(first) = tp.path.segments.first()
					&& first.ident == "Self"
				{
					let mut new_path = tp.path.clone();
					new_path.segments = new_path.segments.into_iter().skip(1).collect();
					let self_ty = self.self_ty;
					let segments = &new_path.segments;
					*i = parse_quote!(<#self_ty>::#segments);
				}
			}
		}
		visit_mut::visit_type_mut(self, i);
	}

	fn visit_type_macro_mut(
		&mut self,
		i: &mut syn::TypeMacro,
	) {
		if i.mac.path.is_ident("Apply")
			&& let Ok(mut apply_input) = syn::parse2::<ApplyInput>(i.mac.tokens.clone())
		{
			self.visit_type_mut(&mut apply_input.brand);
			for arg in apply_input.args.args.iter_mut() {
				if let syn::GenericArgument::Type(ty) = arg {
					self.visit_type_mut(ty);
				}
			}

			let brand = &apply_input.brand;
			let kind_input = &apply_input.kind_input;
			let assoc_name = &apply_input.assoc_name;
			let args = &apply_input.args;

			i.mac.tokens = quote! { <#brand as Kind!(#kind_input)>::#assoc_name #args };
		}
		visit_mut::visit_type_macro_mut(self, i);
	}

	fn visit_signature_mut(
		&mut self,
		i: &mut Signature,
	) {
		for input in &mut i.inputs {
			if let syn::FnArg::Receiver(r) = input {
				let self_ty = self.self_ty;
				let attrs = &r.attrs;
				if let Some(reference) = &r.reference {
					let lt = &reference.1;
					if r.mutability.is_some() {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt mut #self_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					} else {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt #self_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					}
				} else {
					let pat: syn::Pat = parse_quote!(self);
					let ty: syn::Type = parse_quote!(#self_ty);
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::doc_utils::get_doc;
	use syn::parse_quote;

	/// Tests the full orchestration of the `document_impl` macro on a typical impl block.
	/// - Verifies that `#[hm_signature]` is correctly replaced by a generated doc comment containing the trait name.
	/// - Verifies that `#[doc_type_params]` on the method triggers the insertion of trait-level parameter docs before the method-level docs.
	/// - Checks the exact content of the inserted documentation to ensure positional mapping works.
	#[test]
	fn test_document_impl_expansion() {
		let attr = quote! {
			doc_type_params("The element type")
		};
		let item = quote! {
			impl<T: Clone> MyTrait for MyType<T> {
				#[hm_signature]
				#[doc_type_params]
				fn foo(&self, x: T) -> T { x.clone() }
			}
		};

		let output = document_impl_impl(attr, item);
		let item_impl: ItemImpl = syn::parse2(output).unwrap();

		let method = match &item_impl.items[0] {
			ImplItem::Fn(m) => m,
			_ => panic!("Expected method"),
		};

		// Attributes should be:
		// 0. HM Signature doc
		// 1. Impl doc
		// 2. #[doc_type_params] marker
		assert_eq!(method.attrs.len(), 3);

		// Note: HM Signature doc is inserted at index 0 because it replaces hm_signature
		// Then Impl doc is inserted at index 1 because it's right before doc_type_params (which moved to 1)

		assert!(method.attrs[0].path().is_ident("doc"));
		let sig_doc = get_doc(&method.attrs[0]);
		assert!(sig_doc.contains("MyTrait"));

		assert!(method.attrs[1].path().is_ident("doc"));
		let impl_doc = get_doc(&method.attrs[1]);
		assert_eq!(impl_doc, "* `T`: The element type");

		assert!(method.attrs[2].path().is_ident("doc_type_params"));
	}

	/// Tests the positional mapping and insertion order of trait parameter documentation.
	/// - Verifies that multiple parameters are documented in the order they appear in `doc_type_params`.
	/// - Verifies that the trait parameter docs are inserted *before* the method's `#[doc_type_params]` marker, ensuring correct final documentation order.
	#[test]
	fn test_doc_order() {
		let attr = quote! {
			doc_type_params("T desc", "U desc")
		};
		let item = quote! {
			impl<T, U> Trait for Type {
				#[doc_type_params]
				fn foo() {}
			}
		};

		let output = document_impl_impl(attr, item);
		let item_impl: ItemImpl = syn::parse2(output).unwrap();
		let method = match &item_impl.items[0] {
			ImplItem::Fn(m) => m,
			_ => panic!("Expected method"),
		};

		// 0. T desc
		// 1. U desc
		// 2. #[doc_type_params]
		assert_eq!(get_doc(&method.attrs[0]), "* `T`: T desc");
		assert_eq!(get_doc(&method.attrs[1]), "* `U`: U desc");
		assert!(method.attrs[2].path().is_ident("doc_type_params"));
	}

	/// Tests the substitution of value-receiver `self` arguments.
	/// - Verifies that `self` is converted to a typed argument `self: Type` where `Type` is the concrete `impl` type.
	#[test]
	fn test_self_substitution() {
		let self_ty: Type = parse_quote!(MyType<T>);
		let mut substitutor = SelfSubstitutor { self_ty: &self_ty };

		let mut sig: Signature = parse_quote!(fn bar(self, x: Self) -> Self);
		substitutor.visit_signature_mut(&mut sig);

		let expected: Signature = parse_quote!(fn bar(self: MyType<T>, x: MyType<T>) -> MyType<T>);
		assert_eq!(quote!(#sig).to_string(), quote!(#expected).to_string());
	}

	/// Tests the substitution of reference-receiver `&self` arguments.
	/// - Verifies that `&self` is converted to `self: &Type`, preserving the reference.
	#[test]
	fn test_self_ref_substitution() {
		let self_ty: Type = parse_quote!(MyType<T>);
		let mut substitutor = SelfSubstitutor { self_ty: &self_ty };

		let mut sig: Signature = parse_quote!(fn bar(&self, x: &Self) -> &Self);
		substitutor.visit_signature_mut(&mut sig);

		let expected: Signature =
			parse_quote!(fn bar(self: &MyType<T>, x: &MyType<T>) -> &MyType<T>);
		assert_eq!(quote!(#sig).to_string(), quote!(#expected).to_string());
	}

	/// Tests the substitution of mutable reference-receiver `&mut self` arguments.
	/// - Verifies that `&mut self` is converted to `self: &mut Type`, preserving mutability.
	#[test]
	fn test_self_mut_ref_substitution() {
		let self_ty: Type = parse_quote!(MyType<T>);
		let mut substitutor = SelfSubstitutor { self_ty: &self_ty };

		let mut sig: Signature = parse_quote!(fn bar(&mut self, x: &mut Self) -> &mut Self);
		substitutor.visit_signature_mut(&mut sig);

		let expected: Signature =
			parse_quote!(fn bar(self: &mut MyType<T>, x: &mut MyType<T>) -> &mut MyType<T>);
		assert_eq!(quote!(#sig).to_string(), quote!(#expected).to_string());
	}

	/// Tests the substitution of associated types on `Self`.
	/// - Verifies that `Self::Item` is converted to `<ConcreteType>::Item`.
	/// - This ensures that the generated signature uses valid Qualified Path syntax, which is required for `syn` to parse the synthetic signature correctly.
	#[test]
	fn test_self_assoc_substitution() {
		let self_ty: Type = parse_quote!(MyType<T>);
		let mut substitutor = SelfSubstitutor { self_ty: &self_ty };

		let mut sig: Signature = parse_quote!(fn bar(x: Self::Item) -> <Self as Trait>::Item);
		substitutor.visit_signature_mut(&mut sig);

		let expected: Signature =
			parse_quote!(fn bar(x: <MyType<T>>::Item) -> <MyType<T> as Trait>::Item);
		assert_eq!(quote!(#sig).to_string(), quote!(#expected).to_string());
	}

	/// Tests the capture and integration of complex bounds from the `impl` block.
	/// - Verifies that bounds defined on the `impl` generics (e.g., `where T: Semigroup`) are correctly merged into the method signature's documentation.
	/// - Checks for the presence of specific bound names in the generated doc string.
	#[test]
	fn test_complex_impl_bounds() {
		let attr = quote! {};
		let item = quote! {
			impl<T> MyTrait for MyType<T> where T: Semigroup + Monoid, MyType<T>: Functor {
				#[hm_signature]
				fn foo(x: T) {}
			}
		};

		let output = document_impl_impl(attr, item);
		let item_impl: ItemImpl = syn::parse2(output).unwrap();
		let method = match &item_impl.items[0] {
			ImplItem::Fn(m) => m,
			_ => panic!("Expected method"),
		};

		let doc = get_doc(&method.attrs[0]);
		println!("COMPLEX DOC: {}", doc);
		// Should contain bounds from impl
		assert!(doc.contains("Semigroup"));
		assert!(doc.contains("Monoid"));
		assert!(doc.contains("Functor"));
	}

	#[test]
	fn test_self_substitution_in_apply() {
		let self_ty: Type = parse_quote!(MyBrand);
		let mut substitutor = SelfSubstitutor { self_ty: &self_ty };

		// Note: The macro invocation tokens must be valid for syn to parse ApplyInput inside our visitor
		let mut sig: Signature = parse_quote!(
			fn foo(x: Apply!(<Self as Kind!(type Of<T>;)>::Of<T>))
		);
		substitutor.visit_signature_mut(&mut sig);

		let expected: Signature = parse_quote!(
			fn foo(x: Apply!(<MyBrand as Kind!(type Of<T>;)>::Of<T>))
		);
		assert_eq!(quote!(#sig).to_string(), quote!(#expected).to_string());
	}

	/// Tests that `document_impl` works on inherent impls (impl Type {}), not just trait impls.
	/// - Verifies that `self` is correctly substituted with the concrete type.
	/// - Verifies that no trait bound is added.
	#[test]
	fn test_inherent_impl_expansion() {
		let attr = quote! {};
		let item = quote! {
			impl<A> CatList<A> {
				#[hm_signature]
				fn is_empty(&self) -> bool { true }
			}
		};

		let output = document_impl_impl(attr, item);
		let item_impl: ItemImpl = syn::parse2(output).unwrap();

		let method = match &item_impl.items[0] {
			ImplItem::Fn(m) => m,
			_ => panic!("Expected method"),
		};

		let doc = get_doc(&method.attrs[0]);
		// Expected: forall a. &CatList a -> bool
		assert_eq!(doc, "`forall a. &CatList a -> bool`");
	}
}
