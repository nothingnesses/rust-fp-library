//! Type-safe projection keys for resolution.
//!
//! This module provides a newtype wrapper around projection keys to prevent
//! tuple ordering errors and improve API clarity.

use std::hash::{Hash, Hasher};

/// Type-safe key for projection resolution.
///
/// Replaces the tuple-based `(String, Option<String>, String)` with a
/// clear, type-safe API that prevents ordering errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionKey {
    type_path: String,
    trait_path: Option<String>,
    assoc_name: String,
}

impl ProjectionKey {
    /// Create a new module-level projection key (no trait qualification).
    ///
    /// # Example
    /// ```ignore
    /// let key = ProjectionKey::new("MyBrand", "Of");
    /// // Represents: (MyBrand, None, Of) - module-level default
    /// ```
    pub fn new(type_path: impl Into<String>, assoc_name: impl Into<String>) -> Self {
        Self {
            type_path: type_path.into(),
            trait_path: None,
            assoc_name: assoc_name.into(),
        }
    }

    /// Create a scoped projection key (Type, Trait, AssocName).
    ///
    /// # Example
    /// ```ignore
    /// let key = ProjectionKey::scoped("MyBrand", "Functor", "Map");
    /// // Represents: (MyBrand, Some(Functor), Map) - trait-scoped
    /// ```
    pub fn scoped(
        type_path: impl Into<String>,
        trait_path: impl Into<String>,
        assoc_name: impl Into<String>,
    ) -> Self {
        Self {
            type_path: type_path.into(),
            trait_path: Some(trait_path.into()),
            assoc_name: assoc_name.into(),
        }
    }

    /// Set the trait path for this key.
    ///
    /// Converts a module-level key to a scoped key.
    pub fn with_trait(mut self, trait_path: impl Into<String>) -> Self {
        self.trait_path = Some(trait_path.into());
        self
    }

    /// Remove trait qualification, making this a module-level key.
    pub fn module_level(mut self) -> Self {
        self.trait_path = None;
        self
    }

    /// Get the type path component.
    pub fn type_path(&self) -> &str {
        &self.type_path
    }

    /// Get the trait path component, if any.
    pub fn trait_path(&self) -> Option<&str> {
        self.trait_path.as_deref()
    }

    /// Get the associated type name component.
    pub fn assoc_name(&self) -> &str {
        &self.assoc_name
    }

    /// Check if this is a module-level key (no trait qualification).
    pub fn is_module_level(&self) -> bool {
        self.trait_path.is_none()
    }

    /// Check if this is a scoped key (has trait qualification).
    pub fn is_scoped(&self) -> bool {
        self.trait_path.is_some()
    }

    /// Convert to tuple representation for backward compatibility.
    ///
    /// Returns: `(type_path, trait_path, assoc_name)`
    pub fn to_tuple(&self) -> (String, Option<String>, String) {
        (
            self.type_path.clone(),
            self.trait_path.clone(),
            self.assoc_name.clone(),
        )
    }

    /// Create from tuple representation for backward compatibility.
    ///
    /// # Arguments
    /// * `tuple` - `(type_path, trait_path, assoc_name)`
    pub fn from_tuple(tuple: (String, Option<String>, String)) -> Self {
        Self {
            type_path: tuple.0,
            trait_path: tuple.1,
            assoc_name: tuple.2,
        }
    }
}

impl Hash for ProjectionKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_path.hash(state);
        self.trait_path.hash(state);
        self.assoc_name.hash(state);
    }
}

impl From<(String, Option<String>, String)> for ProjectionKey {
    fn from(tuple: (String, Option<String>, String)) -> Self {
        Self::from_tuple(tuple)
    }
}

impl From<ProjectionKey> for (String, Option<String>, String) {
    fn from(key: ProjectionKey) -> Self {
        key.to_tuple()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_module_level() {
        let key = ProjectionKey::new("MyBrand", "Of");
        assert_eq!(key.type_path(), "MyBrand");
        assert_eq!(key.trait_path(), None);
        assert_eq!(key.assoc_name(), "Of");
        assert!(key.is_module_level());
        assert!(!key.is_scoped());
    }

    #[test]
    fn test_scoped() {
        let key = ProjectionKey::scoped("MyBrand", "Functor", "Map");
        assert_eq!(key.type_path(), "MyBrand");
        assert_eq!(key.trait_path(), Some("Functor"));
        assert_eq!(key.assoc_name(), "Map");
        assert!(!key.is_module_level());
        assert!(key.is_scoped());
    }

    #[test]
    fn test_with_trait() {
        let key = ProjectionKey::new("MyBrand", "Of").with_trait("Functor");
        assert_eq!(key.type_path(), "MyBrand");
        assert_eq!(key.trait_path(), Some("Functor"));
        assert_eq!(key.assoc_name(), "Of");
    }

    #[test]
    fn test_module_level_conversion() {
        let key = ProjectionKey::scoped("MyBrand", "Functor", "Map").module_level();
        assert_eq!(key.trait_path(), None);
        assert!(key.is_module_level());
    }

    #[test]
    fn test_tuple_conversion() {
        let original = ProjectionKey::scoped("MyBrand", "Functor", "Map");
        let tuple = original.to_tuple();
        let restored = ProjectionKey::from_tuple(tuple);
        assert_eq!(original, restored);
    }

    #[test]
    fn test_from_into_tuple() {
        let tuple = ("MyBrand".to_string(), Some("Functor".to_string()), "Map".to_string());
        let key: ProjectionKey = tuple.clone().into();
        let restored: (String, Option<String>, String) = key.into();
        assert_eq!(tuple, restored);
    }

    #[test]
    fn test_equality() {
        let key1 = ProjectionKey::scoped("MyBrand", "Functor", "Map");
        let key2 = ProjectionKey::scoped("MyBrand", "Functor", "Map");
        let key3 = ProjectionKey::new("MyBrand", "Map");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashSet;

        let key1 = ProjectionKey::scoped("MyBrand", "Functor", "Map");
        let key2 = ProjectionKey::scoped("MyBrand", "Functor", "Map");

        let mut set = HashSet::new();
        set.insert(key1.clone());
        assert!(set.contains(&key2));
    }
}
