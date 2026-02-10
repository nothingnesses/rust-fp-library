//! Type-safe impl block keys for metadata storage.
//!
//! This module provides a newtype wrapper for uniquely identifying impl blocks,
//! used for storing impl-level metadata like type parameter documentation.

use std::hash::{Hash, Hasher};

/// Type-safe key for impl block identification.
///
/// Used to uniquely identify an impl block for storing impl-level metadata
/// like type parameter documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplKey {
	type_path: String,
	trait_path: Option<String>,
}

impl ImplKey {
	/// Create a new impl key for an inherent impl.
	///
	/// # Example
	/// ```ignore
	/// let key = ImplKey::new("Free<F, A>");
	/// // Represents: impl Free<F, A> { ... }
	/// ```
	pub fn new(type_path: impl Into<String>) -> Self {
		Self { type_path: type_path.into(), trait_path: None }
	}

	/// Create a new impl key for a trait impl.
	///
	/// # Example
	/// ```ignore
	/// let key = ImplKey::with_trait("Free<ThunkBrand, A>", "Deferrable");
	/// // Represents: impl Deferrable for Free<ThunkBrand, A> { ... }
	/// ```
	pub fn with_trait(
		type_path: impl Into<String>,
		trait_path: impl Into<String>,
	) -> Self {
		Self { type_path: type_path.into(), trait_path: Some(trait_path.into()) }
	}

	/// Create an impl key from type and optional trait paths.
	///
	/// This is a convenience method that dispatches to either `new` or `with_trait`
	/// based on whether a trait path is provided.
	///
	/// # Example
	/// ```ignore
	/// // Inherent impl
	/// let key = ImplKey::from_paths("Free<F, A>", None);
	/// // Equivalent to: ImplKey::new("Free<F, A>")
	///
	/// // Trait impl
	/// let key = ImplKey::from_paths("Free<F, A>", Some("Functor"));
	/// // Equivalent to: ImplKey::with_trait("Free<F, A>", "Functor")
	/// ```
	pub fn from_paths(
		type_path: impl Into<String>,
		trait_path: Option<impl Into<String>>,
	) -> Self {
		match trait_path {
			Some(t) => Self::with_trait(type_path, t),
			None => Self::new(type_path),
		}
	}

	/// Get the type path component.
	pub fn type_path(&self) -> &str {
		&self.type_path
	}

	/// Get the trait path component, if any.
	pub fn trait_path(&self) -> Option<&str> {
		self.trait_path.as_deref()
	}

	/// Check if this is an inherent impl key (no trait).
	pub fn is_inherent(&self) -> bool {
		self.trait_path.is_none()
	}

	/// Check if this is a trait impl key.
	pub fn is_trait_impl(&self) -> bool {
		self.trait_path.is_some()
	}
}

impl Hash for ImplKey {
	fn hash<H: Hasher>(
		&self,
		state: &mut H,
	) {
		self.type_path.hash(state);
		self.trait_path.hash(state);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_inherent() {
		let key = ImplKey::new("Free<F, A>");
		assert_eq!(key.type_path(), "Free<F, A>");
		assert_eq!(key.trait_path(), None);
		assert!(key.is_inherent());
		assert!(!key.is_trait_impl());
	}

	#[test]
	fn test_with_trait() {
		let key = ImplKey::with_trait("Free<ThunkBrand, A>", "Deferrable");
		assert_eq!(key.type_path(), "Free<ThunkBrand, A>");
		assert_eq!(key.trait_path(), Some("Deferrable"));
		assert!(!key.is_inherent());
		assert!(key.is_trait_impl());
	}

	#[test]
	fn test_equality() {
		let key1 = ImplKey::with_trait("Free<F, A>", "Functor");
		let key2 = ImplKey::with_trait("Free<F, A>", "Functor");
		let key3 = ImplKey::new("Free<F, A>");

		assert_eq!(key1, key2);
		assert_ne!(key1, key3);
	}

	#[test]
	fn test_hash_consistency() {
		use std::collections::HashSet;

		let key1 = ImplKey::with_trait("Free<F, A>", "Functor");
		let key2 = ImplKey::with_trait("Free<F, A>", "Functor");

		let mut set = HashSet::new();
		set.insert(key1.clone());
		assert!(set.contains(&key2));
	}

	#[test]
	fn test_from_paths_with_trait() {
		let key = ImplKey::from_paths("Free<F, A>", Some("Functor"));
		assert_eq!(key.type_path(), "Free<F, A>");
		assert_eq!(key.trait_path(), Some("Functor"));
		assert!(key.is_trait_impl());
		assert!(!key.is_inherent());
	}

	#[test]
	fn test_from_paths_without_trait() {
		let key = ImplKey::from_paths("Free<F, A>", None::<&str>);
		assert_eq!(key.type_path(), "Free<F, A>");
		assert_eq!(key.trait_path(), None);
		assert!(key.is_inherent());
		assert!(!key.is_trait_impl());
	}

	#[test]
	fn test_from_paths_equivalence() {
		let key1 = ImplKey::from_paths("MyType", Some("MyTrait"));
		let key2 = ImplKey::with_trait("MyType", "MyTrait");
		assert_eq!(key1, key2);

		let key3 = ImplKey::from_paths("MyType", None::<&str>);
		let key4 = ImplKey::new("MyType");
		assert_eq!(key3, key4);
	}
}
