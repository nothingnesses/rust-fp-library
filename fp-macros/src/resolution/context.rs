use super::resolver::{normalize_type, type_uses_self_assoc};
use crate::{
	core::{config::Config, constants::known_attrs,error_handling::ErrorCollector},
	hkt::ImplKindInput,
	resolution::ProjectionKey,
	support::attributes::has_attr,
};
use quote::ToTokens;
use syn::{Error, ImplItem, Item, Result, spanned::Spanned};

/// Extract context from items (projections, defaults, etc.)
pub fn extract_context(
	items: &[Item],
	config: &mut Config,
) -> Result<()> {
	let mut errors = ErrorCollector::new();

	// Track defaults per (Type, Trait) to detect conflicts across split impl blocks
	let mut scoped_defaults_tracker: std::collections::HashMap<
		(String, String),
		Vec<(String, proc_macro2::Span)>,
	> = std::collections::HashMap::new();

	for item in items {
		match item {
			Item::Macro(m) if m.mac.path.is_ident("impl_kind") => {
				// Check if this macro has cfg attributes - skip conflict detection if present
				// since cfg evaluation happens after macro expansion
				let has_cfg = m.attrs.iter().any(|attr| attr.path().is_ident("cfg"));

				if let Ok(impl_kind) = m.mac.parse_body::<ImplKindInput>() {
					let brand_path = impl_kind.brand.to_token_stream().to_string();
					for def in &impl_kind.definitions {
						let assoc_name = def.ident.to_string();

						// Check for circular references (Self:: forbidden in RHS)
						if type_uses_self_assoc(&def.target_type) {
							errors.push(Error::new(
								def.target_type.span(),
								"Self:: reference forbidden in associated type definition",
							));
						}

						// Check for collisions in impl_kind! only if not cfg-gated
						// We use normalized types to detect semantic collisions even if generic names differ
						let key = ProjectionKey::new(&brand_path, &assoc_name);
						if !has_cfg {
							let normalized_target =
								normalize_type(def.target_type.clone(), &def.generics);
							if let Some((prev_generics, prev_type)) = config.projections.get(&key) {
								let prev_normalized =
									normalize_type(prev_type.clone(), prev_generics);
								// Use direct structural comparison instead of string-based comparison.
								// syn::Type implements PartialEq which compares the AST structure,
								// making this reliable and efficient. Since both types are normalized
								// (generics replaced with T0, T1, etc.), this comparison is semantically correct.
								if prev_normalized != normalized_target {
									errors.push(Error::new(
										def.ident.span(),
										format!(
											"Conflicting implementation for {}: already defined with different type",
											assoc_name
										),
									));
								}
							}
						}

						config
							.projections
							.insert(key, (def.generics.clone(), def.target_type.clone()));

						if has_attr(&def.attrs, known_attrs::DOC_DEFAULT)
							&& let Some(prev) = config
								.module_defaults
								.insert(brand_path.clone(), assoc_name.clone())
						{
							errors.push(Error::new(
								def.ident.span(),
								format!(
									"Conflicting module default for {}: {} and {}",
									brand_path, prev, assoc_name
								),
							));
						}
					}
				}
			}
			Item::Impl(item_impl) => {
				let self_ty_path = item_impl.self_ty.to_token_stream().to_string();
				let trait_path = item_impl
					.trait_
					.as_ref()
					.map(|(_, path, _)| path.to_token_stream().to_string());

				// Split impl block merging: merge associated types across multiple impl blocks
				for item in &item_impl.items {
					if let ImplItem::Type(assoc_type) = item {
						let assoc_name = assoc_type.ident.to_string();

						// Store projection (multiple impl blocks can define same assoc type)
						let key = if let Some(ref t_path) = trait_path {
							ProjectionKey::scoped(&self_ty_path, t_path, &assoc_name)
						} else {
							ProjectionKey::new(&self_ty_path, &assoc_name)
						};
						config
							.projections
							.insert(key, (assoc_type.generics.clone(), assoc_type.ty.clone()));

						// Track doc_default across split impl blocks
						if has_attr(&assoc_type.attrs, known_attrs::DOC_DEFAULT)
							&& let Some(t_path) = &trait_path
						{
							let key = (self_ty_path.clone(), t_path.clone());
							scoped_defaults_tracker
								.entry(key)
								.or_default()
								.push((assoc_name.clone(), assoc_type.ident.span()));
						}
					}
				}
			}
			_ => {}
		}
	}

	// Check for conflicting defaults across split impl blocks
	for ((self_ty, trait_path), defaults) in scoped_defaults_tracker {
		if defaults.len() > 1 {
			let names: Vec<_> = defaults.iter().map(|(n, _)| n.as_str()).collect();
			for (_name, span) in &defaults {
				errors.push(Error::new(
					*span,
					format!(
						"Multiple #[doc_default] annotations for ({}, {}): {}",
						self_ty,
						trait_path,
						names.join(", ")
					),
				));
			}
		} else if let Some((name, _)) = defaults.first() {
			config.scoped_defaults.insert((self_ty, trait_path), name.clone());
		}
	}

	errors.finish()
}
