//! Error handling infrastructure for the macro system.

/// Known type names used throughout the macro system
pub mod known_types {
	/// The `Self` type keyword
	pub const SELF: &str = "Self";
	/// The `PhantomData` type from std::marker
	pub const PHANTOM_DATA: &str = "PhantomData";
	/// The `Apply` macro/trait
	pub const APPLY_MACRO: &str = "Apply";
	/// Internal marker for function brand types
	pub const FN_BRAND_MARKER: &str = "fn_brand_marker";
}

/// Known attribute names used by the documentation macros
pub mod known_attrs {
	/// Attribute to specify the default associated type for a type
	pub const DOC_DEFAULT: &str = "doc_default";
	/// Attribute to specify which associated type to use for documentation
	pub const DOC_USE: &str = "doc_use";
	/// Attribute for Hindley-Milner signature generation
	#[allow(dead_code)] // Part of public API, not all constants used yet
	pub const HM_SIGNATURE: &str = "hm_signature";
	/// Attribute for type parameter documentation
	#[allow(dead_code)] // Part of public API, not all constants used yet
	pub const DOC_TYPE_PARAMS: &str = "doc_type_params";
	/// Attribute for function parameter documentation
	#[allow(dead_code)] // Part of public API, not all constants used yet
	pub const DOC_PARAMS: &str = "doc_params";
}
