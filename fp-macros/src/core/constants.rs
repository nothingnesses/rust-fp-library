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
	/// Rust's Box smart pointer
	pub const BOX: &str = "Box";
	/// Rust's Arc smart pointer
	pub const ARC: &str = "Arc";
	/// Rust's Rc smart pointer
	pub const RC: &str = "Rc";
	/// Rust's Fn trait
	pub const FN: &str = "Fn";
	/// Rust's FnMut trait
	pub const FN_MUT: &str = "FnMut";
	/// Rust's FnOnce trait
	pub const FN_ONCE: &str = "FnOnce";
	/// List of supported smart pointers
	pub const SMART_POINTERS: &[&str] = &[BOX, ARC, RC];
	/// List of supported Fn traits
	pub const FN_TRAITS: &[&str] = &[FN, FN_MUT, FN_ONCE];
}

/// Known attribute names used by the documentation macros
pub mod known_attrs {
	/// Attribute to specify the default associated type for a type
	pub const DOCUMENT_DEFAULT: &str = "document_default";
	/// Attribute to specify which associated type to use for documentation
	pub const DOCUMENT_USE: &str = "document_use";
	/// Attribute for Hindley-Milner signature generation
	pub const DOCUMENT_SIGNATURE: &str = "document_signature";
	/// Attribute for type parameter documentation
	pub const DOCUMENT_TYPE_PARAMETERS: &str = "document_type_parameters";
	/// Attribute for function parameter documentation
	#[allow(dead_code)]
	pub const DOCUMENT_PARAMETERS: &str = "document_parameters";
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
	pub const CONFIG_SECTION: &str = "document_signature";
}
