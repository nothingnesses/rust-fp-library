//! Utility functions for analyzing methods, traits, and impl blocks.
//!
//! This module provides helper functions for common operations on method signatures,
//! impl blocks, and trait definitions, such as detecting receiver parameters.

use syn::{
	FnArg,
	ImplItem,
	TraitItem,
};

/// Check if a signature has a receiver parameter (self, &self, &mut self, etc.)
pub fn sig_has_receiver(sig: &syn::Signature) -> bool {
	sig.inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)))
}

/// Check if a signature has non-receiver parameters
pub fn sig_has_non_receiver_parameters(sig: &syn::Signature) -> bool {
	sig.inputs.iter().any(|arg| matches!(arg, FnArg::Typed(_)))
}

/// Check if a method has a receiver parameter (self, &self, &mut self, etc.)
pub fn has_receiver(method: &syn::ImplItemFn) -> bool {
	sig_has_receiver(&method.sig)
}

/// Check if an impl block contains any methods with receiver parameters
pub fn impl_has_receiver_methods(item_impl: &syn::ItemImpl) -> bool {
	item_impl
		.items
		.iter()
		.any(|item| if let ImplItem::Fn(method) = item { has_receiver(method) } else { false })
}

/// Check if a trait definition contains any methods with receiver parameters
pub fn trait_has_receiver_methods(item_trait: &syn::ItemTrait) -> bool {
	item_trait
		.items
		.iter()
		.any(|item| matches!(item, TraitItem::Fn(method) if sig_has_receiver(&method.sig)))
}
