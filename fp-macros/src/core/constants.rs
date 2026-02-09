//! Constants used throughout the crate.

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

/// Default traits to ignore in trait objects and bounds
pub mod default_traits {
	/// Default list of traits to ignore in trait objects and bounds.
	/// These are common marker traits that don't affect the functional signature.
	pub const DEFAULT_IGNORED_TRAITS: &[&str] = &[
		"Clone",
		"Copy",
		"Debug",
		"Display",
		"PartialEq",
		"Eq",
		"PartialOrd",
		"Ord",
		"Hash",
		"Default",
		"Send",
		"Sync",
		"Sized",
		"Unpin",
	];
}

/// Configuration file and environment variable names
pub mod config_names {
	/// Environment variable for Cargo manifest directory
	pub const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
	/// Cargo manifest filename
	pub const CARGO_TOML: &str = "Cargo.toml";
	/// Configuration section name in Cargo.toml metadata
	pub const CONFIG_SECTION: &str = "hm_signature";
}
