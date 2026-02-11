use crate::{
	analysis::{TraitCategory, analyze_generics, classify_trait, format_brand_name},
	core::{
		config::{Config, get_config},
		constants::attributes::DOCUMENT_SIGNATURE,
		{Error, Result},
	},
	hm::{HmAst, type_to_hm},
	support::{
		ast::RustAst, generate_documentation::insert_doc_comment, is_phantom_data,
		parsing::parse_empty_attributes,
	},
};
use proc_macro2::TokenStream;
use std::collections::{HashMap, HashSet};
use syn::{GenericParam, ReturnType, TypeParamBound, WherePredicate};

pub fn document_signature_worker(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> Result<TokenStream> {
	// Validate that no attributes are provided
	parse_empty_attributes(attr)?;

	// Parse the item
	let mut item = RustAst::parse(item_tokens).map_err(Error::Parse)?;

	// Get the function signature
	let sig = item.signature().ok_or_else(|| {
		Error::validation(
			proc_macro2::Span::call_site(),
			format!("{DOCUMENT_SIGNATURE} can only be used on functions or methods"),
		)
	})?;

	// Get configuration
	let cfg = get_config();

	// Generate the Hindley-Milner signature
	let signature = generate_signature(sig, &cfg);
	let doc_comment = format!("`{signature}`");

	// Insert the documentation comment
	insert_doc_comment(item.attributes(), doc_comment, proc_macro2::Span::call_site());

	Ok(quote::quote! {
		#item
	})
}

pub struct SignatureData {
	pub forall: Vec<String>,
	pub constraints: Vec<String>,
	pub params: Vec<HmAst>,
	pub return_type: HmAst,
}

impl std::fmt::Display for SignatureData {
	fn fmt(
		&self,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		let mut parts = Vec::new();

		if !self.forall.is_empty() {
			parts.push(format!("forall {}.", self.forall.join(" ")));
		}

		if !self.constraints.is_empty() {
			let s = if self.constraints.len() == 1 {
				self.constraints[0].clone()
			} else {
				format!("({})", self.constraints.join(", "))
			};
			parts.push(format!("{s} =>"));
		}

		let func_sig = if self.params.is_empty() {
			let func_type = HmAst::Arrow(Box::new(HmAst::Unit), Box::new(self.return_type.clone()));
			func_type.to_string()
		} else {
			let input_type = if self.params.len() == 1 {
				self.params[0].clone()
			} else {
				HmAst::Tuple(self.params.clone())
			};
			let func_type = HmAst::Arrow(Box::new(input_type), Box::new(self.return_type.clone()));
			func_type.to_string()
		};
		parts.push(func_sig);

		write!(f, "{}", parts.join(" "))
	}
}

/// Generates a Hindley-Milner type signature from a Rust function signature.
///
/// ### Parameters
///
/// * `sig` - The Rust function signature to convert
/// * `config` - Configuration for type resolution and formatting
///
/// ### Note on Self Resolution
///
/// Self type resolution is handled by [`document_module`](crate::documentation::document_module)
/// before calling this function. When used standalone, `Self` types remain as-is in the signature.
pub fn generate_signature(
	sig: &syn::Signature,
	config: &Config,
) -> SignatureData {
	let (generic_names, fn_bounds) = analyze_generics(sig, config);

	// Erase unsafe modifier
	let mut sig = sig.clone();
	sig.unsafety = None;

	let (forall, constraints) = format_generics(&sig.generics, &fn_bounds, &generic_names, config);

	let params = format_parameters(&sig, &fn_bounds, &generic_names, config);

	let ret = format_return_type(&sig.output, &fn_bounds, &generic_names, config);

	// Note: Self resolution is now handled by document_module before calling this function.
	// Concrete type names (like CatList) are already in the signature and appear in forall
	// through the regular generic parameter collection. Trait constraints are also
	// handled through the normal constraint collection process.

	SignatureData { forall, constraints, params, return_type: ret }
}

fn format_generics(
	generics: &syn::Generics,
	fn_bounds: &HashMap<String, HmAst>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> (Vec<String>, Vec<String>) {
	let mut type_vars = Vec::new();
	let mut constraints = Vec::new();

	for param in &generics.params {
		if let GenericParam::Type(type_param) = param {
			let name = type_param.ident.to_string();

			// Only include in forall if it's not a function type variable that we are expanding
			if !fn_bounds.contains_key(&name) {
				// Keep type parameters in original case (uppercase)
				type_vars.push(name.clone());
			}

			for bound in &type_param.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound
					&& let Some(constraint) =
						format_trait_bound(trait_bound, &HmAst::Variable(name.clone()), config)
				{
					constraints.push(constraint);
				}
			}
		}
	}

	if let Some(where_clause) = &generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(predicate_type) = predicate {
				let type_ty =
					type_to_hm(&predicate_type.bounded_ty, fn_bounds, generic_names, config);
				for bound in &predicate_type.bounds {
					if let TypeParamBound::Trait(trait_bound) = bound
						&& let Some(constraint) = format_trait_bound(trait_bound, &type_ty, config)
					{
						constraints.push(constraint);
					}
				}
			}
		}
	}

	(type_vars, constraints)
}

fn format_trait_bound(
	bound: &syn::TraitBound,
	type_var: &HmAst,
	config: &Config,
) -> Option<String> {
	// Safely get the last segment of the trait path
	let segment = bound.path.segments.last()?;
	let trait_name = segment.ident.to_string();

	match classify_trait(&trait_name, config) {
		TraitCategory::FnTrait | TraitCategory::FnBrand => None,
		TraitCategory::Other(name) => {
			if config.ignored_traits().contains(&name) {
				None
			} else {
				let name = format_brand_name(&name, config);
				Some(format!("{name} {type_var}"))
			}
		}
		_ => None,
	}
}

fn format_parameters(
	sig: &syn::Signature,
	fn_bounds: &HashMap<String, HmAst>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> Vec<HmAst> {
	let mut params = Vec::new();
	for input in &sig.inputs {
		match input {
			syn::FnArg::Receiver(receiver) => {
				let self_ty = HmAst::Variable("self".to_string());
				if receiver.reference.is_some() {
					if receiver.mutability.is_some() {
						params.push(HmAst::MutableReference(Box::new(self_ty)));
					} else {
						params.push(HmAst::Reference(Box::new(self_ty)));
					}
				} else {
					params.push(self_ty);
				}
			}
			syn::FnArg::Typed(pat_type) => {
				if !is_phantom_data(&pat_type.ty) {
					params.push(type_to_hm(&pat_type.ty, fn_bounds, generic_names, config));
				}
			}
		}
	}
	params
}

fn format_return_type(
	output: &ReturnType,
	fn_bounds: &HashMap<String, HmAst>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HmAst {
	match output {
		ReturnType::Default => HmAst::Unit, // Unit type
		ReturnType::Type(_, ty) => type_to_hm(ty, fn_bounds, generic_names, config),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::support::generate_documentation::get_doc;
	use syn::{ItemFn, parse_quote};

	#[test]
	fn test_simple_signature() {
		let input: ItemFn = parse_quote! {
			fn identity<A>(x: A) -> A { x }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A. A -> A");
	}

	#[test]
	fn test_impl_fn() {
		let input: ItemFn = parse_quote! {
			fn map<A, B>(f: impl Fn(A) -> B, x: A) -> B { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A B. (A -> B, A) -> B");
	}

	#[test]
	fn test_associated_type() {
		let input: ItemFn = parse_quote! {
			fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B> { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall F A B. Functor F => (A -> B, F A) -> F B");
	}

	#[test]
	fn test_apply_macro() {
		let input: ItemFn = parse_quote! {
			fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: Apply!(<F as Kind!(type Of<'a, T>: 'a;)>::Of<'a, A>)) -> Apply!(<F as Kind!(type Of<'a, T>: 'a;)>::Of<'a, B>) { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall F A B. Functor F => (A -> B, F A) -> F B");
	}

	#[test]
	fn test_brand_name() {
		let input: ItemFn = parse_quote! {
			fn map<A, B>(x: OptionBrand<A>) -> OptionBrand<B> { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A B. Option A -> Option B");
	}

	#[test]
	fn test_where_clause() {
		let input: ItemFn = parse_quote! {
			fn map<F, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B>
			where F: Functor
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall F A B. Functor F => (A -> B, F A) -> F B");
	}

	#[test]
	fn test_fn_bound_in_where() {
		let input: ItemFn = parse_quote! {
			fn map<Func, A, B>(f: Func, x: A) -> B
			where Func: Fn(A) -> B
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A B. (A -> B, A) -> B");
	}

	#[test]
	fn test_wilt_signature() {
		let input: ItemFn = parse_quote! {
			fn wilt<'a, M: Applicative, A: 'a + Clone, O: 'a + Clone, E: 'a + Clone, Func>(
				func: Func,
				ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Pair<
					Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
					Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
				>,
			>)
			where
				Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
				Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
				Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
			{
				todo!()
			}
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		// Note: Self should be resolved by document_module before calling this.
		// In standalone usage, Self stays as-is.
		assert_eq!(
			sig,
			"forall M A O E. Applicative M => (A -> M (Result O E), Self A) -> M (Pair (Self O) (Self E))"
		);
	}

	#[test]
	fn test_placement_logic() {
		// 1: empty
		// 2: First
		// 3: empty (where macro was)
		// 4: Second
		let code = "\n#[doc = \"First\"]\n\n#[doc = \"Second\"]\nfn foo() {}";
		let mut input_fn: ItemFn = syn::parse_str(code).unwrap();

		// Create span at line 3
		let span_source = "\n\nstruct S;";
		let ts: proc_macro2::TokenStream = span_source.parse().unwrap();
		let macro_span = ts.into_iter().next().unwrap().span();

		insert_doc_comment(&mut input_fn.attrs, "Signature".to_string(), macro_span);

		assert_eq!(input_fn.attrs.len(), 3);

		assert_eq!(get_doc(&input_fn.attrs[0]), "First");
		assert_eq!(get_doc(&input_fn.attrs[1]), "Signature");
		assert_eq!(get_doc(&input_fn.attrs[2]), "Second");
	}

	#[test]
	fn test_par_fold_right() {
		let input: ItemFn = parse_quote! {
			fn par_fold_right<'a, FnBrand, A, B>(
				func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
				init: B,
				fa: <Self as Kind_cdc7cd43dac7585f>::Of<'a, A>,
			) -> B
			where
				A: 'a + Clone + Send + Sync,
				B: Send + Sync + 'a,
				FnBrand: 'a + SendCloneableFn,
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		// Note: Self should be resolved by document_module, in standalone it stays as Self
		assert_eq!(sig, "forall A B. ((A, B) -> B, B, Self A) -> B");
	}

	#[test]
	fn test_smart_pointers() {
		let input: ItemFn = parse_quote! {
			fn foo(x: Box<i32>, y: Arc<String>, z: Rc<Vec<f64>>) -> Box<u32> { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "(i32, String, Vec f64) -> u32");
	}

	#[test]
	fn test_arrays_and_slices() {
		let input: ItemFn = parse_quote! {
			fn foo(x: [i32; 5], y: &[String]) -> &[u32] { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "([i32], &[String]) -> &[u32]");
	}

	#[test]
	fn test_trait_objects() {
		let input: ItemFn = parse_quote! {
			fn foo(x: &dyn Fn(i32) -> i32, y: Box<dyn Iterator<Item = String>>) -> i32 { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "(&dyn (i32 -> i32), dyn (Iterator String)) -> i32");
	}

	#[test]
	fn test_bare_fn() {
		let input: ItemFn = parse_quote! {
			fn foo(x: fn(i32, i32) -> i32) -> i32 { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "((i32, i32) -> i32) -> i32");
	}

	#[test]
	fn test_config_mapping() {
		let mut config = Config::default();
		config.user_config.brand_mappings.insert("CustomBrand".to_string(), "Custom".to_string());

		let input: ItemFn = parse_quote! {
			fn foo(x: CustomBrand<i32>) -> CustomBrand<u32> { todo!() }
		};
		let sig = generate_signature(&input.sig, &config).to_string();
		assert_eq!(sig, "Custom i32 -> Custom u32");
	}

	#[test]
	fn test_impl_iterator() {
		let input: ItemFn = parse_quote! {
			fn foo(x: impl Iterator<Item = String>) -> i32 { 0 }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "Iterator String -> i32");
	}

	#[test]
	fn test_trait_object_multi_bound() {
		let input: ItemFn = parse_quote! {
			fn foo(x: Box<dyn Iterator<Item = i32> + Send>) -> i32 { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		// Send is ignored by default config
		assert_eq!(sig, "dyn (Iterator i32) -> i32");
	}

	#[test]
	fn test_phantom_data_omission() {
		let input: ItemFn = parse_quote! {
			fn foo<A>(x: A, p: std::marker::PhantomData<A>) -> A { x }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A. A -> A");
	}

	#[test]
	fn test_phantom_data_tuple_omission() {
		let input: ItemFn = parse_quote! {
			fn foo<A>(x: (A, std::marker::PhantomData<A>)) -> A { x.0 }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A. A -> A");
	}

	#[test]
	fn test_phantom_data_in_generic() {
		let input: ItemFn = parse_quote! {
			fn foo<A>(x: Vec<std::marker::PhantomData<A>>) { }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		// Vec expects an arg, so Vec () is appropriate if PhantomData maps to ()
		assert_eq!(sig, "forall A. Vec () -> ()");
	}

	#[test]
	fn test_lifetimes_and_const_generics() {
		let input: ItemFn = parse_quote! {
			fn foo<'a, const N: usize, A: 'a>(x: &'a [A; N]) -> A { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A. &[A] -> A");
	}

	#[test]
	fn test_multiple_constraints() {
		let input: ItemFn = parse_quote! {
			fn foo<F, A>(fa: F::Of<A>)
			where F: Functor + Foldable, A: Clone
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall F A. (Functor F, Foldable F) => F A -> ()");
	}

	#[test]
	fn test_forall_order() {
		let input: ItemFn = parse_quote! {
			fn foo<B, A, C>(a: A, b: B, c: C) { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall B A C. (A, B, C) -> ()");
	}

	#[test]
	fn test_bifunctor_apply() {
		let input: ItemFn = parse_quote! {
			fn bimap<P, A, B, C, D>(f: impl Fn(A) -> B, g: impl Fn(C) -> D, pab: Apply!(<P as Kind!(type Of<A, B>;)>::Of<A, C>)) -> Apply!(<P as Kind!(type Of<A, B>;)>::Of<B, D>)
			where P: Bifunctor
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall P A B C D. Bifunctor P => (A -> B, C -> D, P A C) -> P B D");
	}

	#[test]
	fn test_multi_letter_generic() {
		let input: ItemFn = parse_quote! {
			fn foo<Input, Output>(x: Input) -> Output { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall Input Output. Input -> Output");
	}

	#[test]
	fn test_flip_signature() {
		let input: ItemFn = parse_quote! {
			pub fn flip<A, B, C, F>(f: F) -> impl Fn(B, A) -> C
			where
				F: Fn(A, B) -> C,
			{
				move |b, a| f(a, b)
			}
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A B C. ((A, B) -> C) -> (B, A) -> C");
	}

	#[test]
	fn test_self_receiver_by_value() {
		let input: ItemFn = parse_quote! {
			fn is_empty(self) -> bool { true }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		// Note: self receiver stays lowercase, document_module resolves to concrete type
		assert_eq!(sig, "self -> bool");
	}

	#[test]
	fn test_self_receiver_by_reference() {
		let input: ItemFn = parse_quote! {
			fn is_empty(&self) -> bool { true }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		// Note: self receiver stays lowercase, document_module resolves to concrete type
		assert_eq!(sig, "&self -> bool");
	}

	#[test]
	fn test_self_receiver_by_mutable_reference() {
		let input: ItemFn = parse_quote! {
			fn is_empty(&mut self) -> bool { true }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		// Note: self receiver stays lowercase, document_module resolves to concrete type
		assert_eq!(sig, "&mut self -> bool");
	}

	#[test]
	fn test_zero_argument_function() {
		let input: ItemFn = parse_quote! {
			fn empty<A>() -> CatList<A> { todo!() }
		};
		let sig = generate_signature(&input.sig, &Config::default()).to_string();
		assert_eq!(sig, "forall A. () -> CatList A");
	}
}
