use crate::{
	doc_utils::{GenericItem, insert_doc_comment},
	function_utils::{
		Config, TraitCategory, analyze_generics, classify_trait, format_brand_name,
		is_phantom_data, load_config, type_to_hm,
	},
	hm_ast::HMType,
};
use std::collections::{HashMap, HashSet};
use syn::{GenericParam, ReturnType, TypeParamBound, WherePredicate};

pub fn hm_signature_impl(
	attr: proc_macro2::TokenStream,
	item_tokens: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
	// If we are inside document_module, this attribute might be processed twice.
	// But hm_signature_impl is a standalone macro.
	let mut item = match GenericItem::parse(item_tokens) {
		Ok(i) => i,
		Err(e) => return e.to_compile_error(),
	};

	let sig = match item.sig() {
		Some(s) => s,
		None => {
			return syn::Error::new(
				proc_macro2::Span::call_site(),
				"hm_signature can only be used on functions or methods",
			)
			.to_compile_error();
		}
	};

	if !attr.is_empty() {
		return syn::Error::new(
			proc_macro2::Span::call_site(),
			"hm_signature does not accept arguments",
		)
		.to_compile_error();
	}

	let config = load_config();
	let signature = generate_signature(sig, None, &config);
	let doc_comment = format!("`{}`", signature);

	insert_doc_comment(item.attrs(), doc_comment, proc_macro2::Span::call_site());

	quote::quote! {
		#item
	}
}

pub struct SignatureData {
	pub forall: Vec<String>,
	pub constraints: Vec<String>,
	pub params: Vec<HMType>,
	pub return_type: HMType,
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
			parts.push(format!("{} =>", s));
		}

		let func_sig = if self.params.is_empty() {
			let func_type =
				HMType::Arrow(Box::new(HMType::Unit), Box::new(self.return_type.clone()));
			format!("{}", func_type)
		} else {
			let input_type = if self.params.len() == 1 {
				self.params[0].clone()
			} else {
				HMType::Tuple(self.params.clone())
			};
			let func_type = HMType::Arrow(Box::new(input_type), Box::new(self.return_type.clone()));
			format!("{}", func_type)
		};
		parts.push(func_sig);

		write!(f, "{}", parts.join(" "))
	}
}

pub fn generate_signature(
	sig: &syn::Signature,
	trait_context: Option<&str>,
	config: &Config,
) -> SignatureData {
	let (generic_names, fn_bounds) = analyze_generics(sig, config);

	// Erase unsafe modifier
	let mut sig = sig.clone();
	sig.unsafety = None;

	let (mut forall, mut constraints) =
		format_generics(&sig.generics, &fn_bounds, &generic_names, config);
	let params = format_parameters(&sig, &fn_bounds, &generic_names, config);
	let ret = format_return_type(&sig.output, &fn_bounds, &generic_names, config);

	let uses_self = params.iter().any(hm_type_uses_self) || hm_type_uses_self(&ret);

	if uses_self {
		forall.insert(0, "self".to_string());

		if let Some(trait_name) = trait_context {
			constraints.insert(0, format!("{} self", trait_name));
		}
	}

	SignatureData { forall, constraints, params, return_type: ret }
}

fn hm_type_uses_self(ty: &HMType) -> bool {
	match ty {
		HMType::Variable(name) => name == "self",
		HMType::Constructor(name, args) => name == "self" || args.iter().any(hm_type_uses_self),
		HMType::Arrow(a, b) => hm_type_uses_self(a) || hm_type_uses_self(b),
		HMType::Tuple(args) => args.iter().any(hm_type_uses_self),
		HMType::List(inner) => hm_type_uses_self(inner),
		HMType::Reference(inner) => hm_type_uses_self(inner),
		HMType::MutableReference(inner) => hm_type_uses_self(inner),
		HMType::TraitObject(inner) => hm_type_uses_self(inner),
		HMType::Unit => false,
	}
}

fn format_generics(
	generics: &syn::Generics,
	fn_bounds: &HashMap<String, HMType>,
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
				type_vars.push(name.to_lowercase());
			}

			for bound in &type_param.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound
					&& let Some(constraint) = format_trait_bound(
						trait_bound,
						&HMType::Variable(name.to_lowercase()),
						config,
					) {
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
	type_var: &HMType,
	config: &Config,
) -> Option<String> {
	let trait_name = bound.path.segments.last().unwrap().ident.to_string();

	match classify_trait(&trait_name, config) {
		TraitCategory::FnTrait | TraitCategory::FnBrand => None,
		TraitCategory::Other(name) => {
			if config.ignored_traits.contains(&name) {
				None
			} else {
				let name = format_brand_name(&name, config);
				Some(format!("{} {}", name, type_var))
			}
		}
		_ => None,
	}
}

fn format_parameters(
	sig: &syn::Signature,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> Vec<HMType> {
	let mut params = Vec::new();
	for input in &sig.inputs {
		match input {
			syn::FnArg::Receiver(receiver) => {
				let self_ty = HMType::Variable("self".to_string());
				if receiver.reference.is_some() {
					if receiver.mutability.is_some() {
						params.push(HMType::MutableReference(Box::new(self_ty)));
					} else {
						params.push(HMType::Reference(Box::new(self_ty)));
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
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	match output {
		ReturnType::Default => HMType::Unit, // Unit type
		ReturnType::Type(_, ty) => type_to_hm(ty, fn_bounds, generic_names, config),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use syn::{ItemFn, parse_quote};

	#[test]
	fn test_simple_signature() {
		let input: ItemFn = parse_quote! {
			fn identity<A>(x: A) -> A { x }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a. a -> a");
	}

	#[test]
	fn test_impl_fn() {
		let input: ItemFn = parse_quote! {
			fn map<A, B>(f: impl Fn(A) -> B, x: A) -> B { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a b. (a -> b, a) -> b");
	}

	#[test]
	fn test_associated_type() {
		let input: ItemFn = parse_quote! {
			fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B> { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall f a b. Functor f => (a -> b, f a) -> f b");
	}

	#[test]
	fn test_apply_macro() {
		let input: ItemFn = parse_quote! {
			fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: Apply!(<F as Kind!(type Of<'a, T>: 'a;)>::Of<'a, A>)) -> Apply!(<F as Kind!(type Of<'a, T>: 'a;)>::Of<'a, B>) { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall f a b. Functor f => (a -> b, f a) -> f b");
	}

	#[test]
	fn test_brand_name() {
		let input: ItemFn = parse_quote! {
			fn map<A, B>(x: OptionBrand<A>) -> OptionBrand<B> { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a b. Option a -> Option b");
	}

	#[test]
	fn test_where_clause() {
		let input: ItemFn = parse_quote! {
			fn map<F, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B>
			where F: Functor
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall f a b. Functor f => (a -> b, f a) -> f b");
	}

	#[test]
	fn test_fn_bound_in_where() {
		let input: ItemFn = parse_quote! {
			fn map<Func, A, B>(f: Func, x: A) -> B
			where Func: Fn(A) -> B
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a b. (a -> b, a) -> b");
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
		let sig =
			generate_signature(&input.sig, Some("Witherable"), &Config::default()).to_string();
		assert_eq!(
			sig,
			"forall self m a o e. (Witherable self, Applicative m) => (a -> m (Result o e), self a) -> m (Pair (self o) (self e))"
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

		// Use helper from doc_utils, need to import it or use full path
		// Since we are in a test module inside the crate, we can access crate::doc_utils
		use crate::doc_utils::get_doc;

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
		let sig =
			generate_signature(&input.sig, Some("ParFoldable"), &Config::default()).to_string();
		// Expected: forall self a b. ParFoldable self => ((a, b) -> b, b, self a) -> b
		assert_eq!(sig, "forall self a b. ParFoldable self => ((a, b) -> b, b, self a) -> b");
	}

	#[test]
	fn test_smart_pointers() {
		let input: ItemFn = parse_quote! {
			fn foo(x: Box<i32>, y: Arc<String>, z: Rc<Vec<f64>>) -> Box<u32> { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "(i32, String, Vec f64) -> u32");
	}

	#[test]
	fn test_arrays_and_slices() {
		let input: ItemFn = parse_quote! {
			fn foo(x: [i32; 5], y: &[String]) -> &[u32] { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "([i32], &[String]) -> &[u32]");
	}

	#[test]
	fn test_trait_objects() {
		let input: ItemFn = parse_quote! {
			fn foo(x: &dyn Fn(i32) -> i32, y: Box<dyn Iterator<Item = String>>) -> i32 { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "(&dyn (i32 -> i32), dyn (Iterator String)) -> i32");
	}

	#[test]
	fn test_bare_fn() {
		let input: ItemFn = parse_quote! {
			fn foo(x: fn(i32, i32) -> i32) -> i32 { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "((i32, i32) -> i32) -> i32");
	}

	#[test]
	fn test_config_mapping() {
		let mut config = Config::default();
		config.brand_mappings.insert("CustomBrand".to_string(), "Custom".to_string());

		let input: ItemFn = parse_quote! {
			fn foo(x: CustomBrand<i32>) -> CustomBrand<u32> { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &config).to_string();
		assert_eq!(sig, "Custom i32 -> Custom u32");
	}

	#[test]
	fn test_impl_iterator() {
		let input: ItemFn = parse_quote! {
			fn foo(x: impl Iterator<Item = String>) -> i32 { 0 }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "Iterator String -> i32");
	}

	#[test]
	fn test_trait_object_multi_bound() {
		let input: ItemFn = parse_quote! {
			fn foo(x: Box<dyn Iterator<Item = i32> + Send>) -> i32 { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		// Send is ignored by default config
		assert_eq!(sig, "dyn (Iterator i32) -> i32");
	}

	#[test]
	fn test_phantom_data_omission() {
		let input: ItemFn = parse_quote! {
			fn foo<A>(x: A, p: std::marker::PhantomData<A>) -> A { x }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a. a -> a");
	}

	#[test]
	fn test_phantom_data_tuple_omission() {
		let input: ItemFn = parse_quote! {
			fn foo<A>(x: (A, std::marker::PhantomData<A>)) -> A { x.0 }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a. a -> a");
	}

	#[test]
	fn test_phantom_data_in_generic() {
		let input: ItemFn = parse_quote! {
			fn foo<A>(x: Vec<std::marker::PhantomData<A>>) { }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		// Vec expects an arg, so Vec () is appropriate if PhantomData maps to ()
		assert_eq!(sig, "forall a. Vec () -> ()");
	}

	#[test]
	fn test_lifetimes_and_const_generics() {
		let input: ItemFn = parse_quote! {
			fn foo<'a, const N: usize, A: 'a>(x: &'a [A; N]) -> A { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a. &[a] -> a");
	}

	#[test]
	fn test_multiple_constraints() {
		let input: ItemFn = parse_quote! {
			fn foo<F, A>(fa: F::Of<A>)
			where F: Functor + Foldable, A: Clone
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall f a. (Functor f, Foldable f) => f a -> ()");
	}

	#[test]
	fn test_forall_order() {
		let input: ItemFn = parse_quote! {
			fn foo<B, A, C>(a: A, b: B, c: C) { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall b a c. (a, b, c) -> ()");
	}

	#[test]
	fn test_bifunctor_apply() {
		let input: ItemFn = parse_quote! {
			fn bimap<P, A, B, C, D>(f: impl Fn(A) -> B, g: impl Fn(C) -> D, pab: Apply!(<P as Kind!(type Of<A, B>;)>::Of<A, C>)) -> Apply!(<P as Kind!(type Of<A, B>;)>::Of<B, D>)
			where P: Bifunctor
			{ todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall p a b c d. Bifunctor p => (a -> b, c -> d, p a c) -> p b d");
	}

	#[test]
	fn test_multi_letter_generic() {
		let input: ItemFn = parse_quote! {
			fn foo<Input, Output>(x: Input) -> Output { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall input output. input -> output");
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
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a b c. ((a, b) -> c) -> (b, a) -> c");
	}

	#[test]
	fn test_self_receiver_by_value() {
		let input: ItemFn = parse_quote! {
			fn is_empty(self) -> bool { true }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall self. self -> bool");
	}

	#[test]
	fn test_self_receiver_by_reference() {
		let input: ItemFn = parse_quote! {
			fn is_empty(&self) -> bool { true }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall self. &self -> bool");
	}

	#[test]
	fn test_self_receiver_by_mutable_reference() {
		let input: ItemFn = parse_quote! {
			fn is_empty(&mut self) -> bool { true }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall self. &mut self -> bool");
	}

	#[test]
	fn test_zero_argument_function() {
		let input: ItemFn = parse_quote! {
			fn empty<A>() -> CatList<A> { todo!() }
		};
		let sig = generate_signature(&input.sig, None, &Config::default()).to_string();
		assert_eq!(sig, "forall a. () -> CatList a");
	}
}
