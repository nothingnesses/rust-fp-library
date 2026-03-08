//! Constants used throughout the crate.

/// Known type names used throughout the macro system
pub mod types {
	/// The `Self` type keyword
	pub const SELF: &str = "Self";
	/// The `PhantomData` type from std::marker
	pub const PHANTOM_DATA: &str = "PhantomData";
	/// Rust's Box smart pointer
	pub const BOX: &str = "Box";
	/// Rust's Arc smart pointer
	pub const ARC: &str = "Arc";
	/// Rust's Rc smart pointer
	pub const RC: &str = "Rc";
	/// List of supported smart pointers
	pub const SMART_POINTERS: &[&str] = &[BOX, ARC, RC];
}

/// Known trait names used throughout the macro system
pub mod traits {
	/// Rust's Fn trait
	pub const FN: &str = "Fn";
	/// Rust's FnMut trait
	pub const FN_MUT: &str = "FnMut";
	/// Rust's FnOnce trait
	pub const FN_ONCE: &str = "FnOnce";
	/// List of supported Fn traits
	pub const FN_TRAITS: &[&str] = &[FN, FN_MUT, FN_ONCE];

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

/// Known brand names used throughout the macro system
pub mod brands {
	/// Brand for sendable and cloneable functions
	pub const SEND_CLONEABLE_FN: &str = "SendCloneableFn";
	/// Brand for cloneable functions
	pub const CLONEABLE_FN: &str = "CloneableFn";
	/// Brand for general functions
	pub const FUNCTION: &str = "Function";
	/// List of supported function brand traits
	pub const FN_BRANDS: &[&str] = &[SEND_CLONEABLE_FN, CLONEABLE_FN, FUNCTION];
}

/// Known macro names
pub mod macros {
	/// The `Apply` macro/trait
	pub const APPLY_MACRO: &str = "Apply";
	/// The `Kind` macro/trait
	pub const KIND_MACRO: &str = "Kind";
	/// The `trait_kind!` macro
	pub const TRAIT_KIND_MACRO: &str = "trait_kind";
	/// The `impl_kind!` macro
	pub const IMPL_KIND_MACRO: &str = "impl_kind";
	/// Assertion macros that doc examples must invoke at least once.
	///
	/// Each entry is the macro name including its trailing `!` so a simple
	/// substring search on the example string is sufficient.
	pub const ASSERTION_MACROS: &[&str] = &[
		"assert!",
		"assert_eq!",
		"assert_ne!",
		"debug_assert!",
		"debug_assert_eq!",
		"debug_assert_ne!",
		"assert_matches!",
	];
}

/// Markers and suffixes used for internal analysis
pub mod markers {
	/// Internal marker for function brand types
	pub const FN_BRAND_MARKER: &str = "fn_brand_marker";
	/// Common suffix for brand types
	pub const BRAND_SUFFIX: &str = "Brand";
}

/// Known attribute names used by the documentation macros
pub mod attributes {
	/// Attribute to specify the default associated type for a type
	pub const DOCUMENT_DEFAULT: &str = "document_default";
	/// Attribute to specify which associated type to use for documentation
	pub const DOCUMENT_USE: &str = "document_use";
	/// Attribute for Hindley-Milner signature generation
	pub const DOCUMENT_SIGNATURE: &str = "document_signature";
	/// Attribute for type parameter documentation
	pub const DOCUMENT_TYPE_PARAMETERS: &str = "document_type_parameters";
	/// Attribute for function parameter documentation
	pub const DOCUMENT_PARAMETERS: &str = "document_parameters";
	/// Attribute for function return documentation
	pub const DOCUMENT_RETURNS: &str = "document_returns";
	/// Attribute for function examples documentation
	pub const DOCUMENT_EXAMPLES: &str = "document_examples";
	/// Attribute for struct field documentation
	pub const DOCUMENT_FIELDS: &str = "document_fields";
	/// Attribute for module documentation
	pub const DOCUMENT_MODULE: &str = "document_module";
	/// List of documentation-specific attributes
	pub const DOCUMENT_SPECIFIC_ATTRS: &[&str] = &[
		DOCUMENT_DEFAULT,
		DOCUMENT_USE,
		DOCUMENT_SIGNATURE,
		DOCUMENT_TYPE_PARAMETERS,
		DOCUMENT_PARAMETERS,
		DOCUMENT_RETURNS,
		DOCUMENT_EXAMPLES,
		DOCUMENT_FIELDS,
		DOCUMENT_MODULE,
	];
	/// The required order for documentation attributes on a method or impl item.
	/// Any subset of these attributes must appear in this order.
	pub const DOCUMENT_ATTR_ORDER: &[&str] = &[
		DOCUMENT_SIGNATURE,
		DOCUMENT_TYPE_PARAMETERS,
		DOCUMENT_PARAMETERS,
		DOCUMENT_RETURNS,
		DOCUMENT_EXAMPLES,
	];
}

/// Configuration file and environment variable names
pub mod configuration {
	/// Environment variable for Cargo manifest directory
	pub const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
	/// Cargo manifest filename
	pub const CARGO_TOML: &str = "Cargo.toml";
	/// Configuration section name in Cargo.toml metadata
	pub const CONFIG_SECTION: &str = "document_signature";
}

/// Constants related to the re-export codegen
pub mod re_export {
	/// The 'mod.rs' filename stem
	pub const MOD_FILE_STEM: &str = "mod";
	/// Rust file extension
	pub const RS_EXTENSION: &str = "rs";
	/// The 'crate' keyword for path generation
	pub const CRATE_KEYWORD: &str = "crate";
	/// The 'src' directory name
	pub const SRC_DIR: &str = "src";
}

/// Constants related to documentation parsing and generation
pub mod documentation {
	/// Language tags that indicate Rust code blocks (validated for assertions).
	pub const RUST_CODE_TAGS: &[&str] = &["", "rust", "no_run", "rust,no_run"];
}
