//! Utility functions for analyzing methods and impl blocks.
//!
//! This module provides helper functions for common operations on methods and impl blocks,
//! such as detecting receiver parameters.

use syn::{FnArg, ImplItem};

/// Check if a method has a receiver parameter (self, &self, &mut self, etc.)
pub fn has_receiver(method: &syn::ImplItemFn) -> bool {
	method.sig.inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)))
}

/// Check if a method has non-receiver parameters
pub fn has_non_receiver_parameters(method: &syn::ImplItemFn) -> bool {
	method.sig.inputs.iter().any(|arg| matches!(arg, FnArg::Typed(_)))
}

/// Check if an impl block contains any methods with receiver parameters
pub fn impl_has_receiver_methods(item_impl: &syn::ItemImpl) -> bool {
	item_impl
		.items
		.iter()
		.any(|item| if let ImplItem::Fn(method) = item { has_receiver(method) } else { false })
}
