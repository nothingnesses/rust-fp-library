//! A type class for types that have a canonical index associated with their structure.

#[fp_macros::document_module]
mod inner {
	/// A type class for types that have a canonical index associated with their structure.
	///
	/// `WithIndex` is a supertype for the `WithIndex` family of type classes
	/// ([`FunctorWithIndex`](crate::classes::FunctorWithIndex),
	/// [`FoldableWithIndex`](crate::classes::FoldableWithIndex),
	/// [`TraversableWithIndex`](crate::classes::TraversableWithIndex), etc.).
	/// It provides the associated [`Index`](WithIndex::Index) type that is uniquely determined
	/// by the implementing brand, encoding the functional dependency `f -> i` from PureScript's
	/// [`FunctorWithIndex`](https://pursuit.purescript.org/packages/purescript-foldable-traversable/docs/Data.FunctorWithIndex).
	///
	/// Because a brand can only implement `WithIndex` once, the index type is shared
	/// across the entire `WithIndex` hierarchy, preventing inconsistent index types between
	/// `FunctorWithIndex`, `FoldableWithIndex`, and `TraversableWithIndex` for the same brand.
	pub trait WithIndex {
		/// The index type for this structure.
		///
		/// Must be `Clone` to support default implementations of fold/traverse
		/// operations that compose closures over the index.
		type Index: Clone;
	}
}

pub use inner::*;
