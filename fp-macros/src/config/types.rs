//! Configuration types for the macro system.

use crate::resolution::ProjectionKey;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// User-provided configuration loaded from Cargo.toml.
///
/// This struct contains only serializable, static configuration that can be safely
/// cached across macro invocations. It does not contain any syn types or transient state.
#[derive(Debug, Clone, Deserialize)]
pub struct UserConfig {
	/// Mapping from brand names to custom display names
	/// Example: "OptionBrand" -> "Option"
	#[serde(default)]
	pub brand_mappings: HashMap<String, String>,

	/// Alternative names for the Apply! macro (for backward compatibility)
	#[serde(default)]
	pub apply_macro_aliases: HashSet<String>,

	/// Traits to ignore in trait objects and bounds (e.g., Send, Sync, Debug)
	#[serde(default = "default_ignored_traits")]
	pub ignored_traits: HashSet<String>,
}

impl Default for UserConfig {
	fn default() -> Self {
		Self {
			brand_mappings: HashMap::new(),
			apply_macro_aliases: HashSet::new(),
			ignored_traits: default_ignored_traits(),
		}
	}
}

/// Complete configuration for macro processing.
///
/// This combines user configuration with runtime state gathered during macro expansion.
/// Contains both serializable config (from Cargo.toml) and non-serializable syn types.
#[derive(Debug, Default, Clone)]
pub struct Config {
	/// User-provided configuration (cacheable, serializable)
	pub user_config: UserConfig,

	/// Projection map: ProjectionKey -> (Generics, TargetType)
	/// Populated by scanning impl blocks and impl_kind! macros
	pub projections: HashMap<ProjectionKey, (syn::Generics, syn::Type)>,

	/// Module-level defaults: TypePath -> AssocName
	/// Used when no explicit #[doc(use = "...")] is provided
	pub module_defaults: HashMap<String, String>,

	/// (Type, Trait)-scoped defaults: (TypePath, TraitPath) -> AssocName
	/// More specific than module_defaults, used for trait impl contexts
	pub scoped_defaults: HashMap<(String, String), String>,

	/// Types that should be preserved as-is (not lowercased)
	/// Used for concrete types resolved from Self
	pub concrete_types: HashSet<String>,

	/// The name of the Self type in the current context
	pub self_type_name: Option<String>,
}

/// Accessor methods for backward compatibility
impl Config {
	/// Access brand_mappings through user_config
	pub fn brand_mappings(&self) -> &HashMap<String, String> {
		&self.user_config.brand_mappings
	}

	/// Access apply_macro_aliases through user_config
	pub fn apply_macro_aliases(&self) -> &HashSet<String> {
		&self.user_config.apply_macro_aliases
	}

	/// Access ignored_traits through user_config
	pub fn ignored_traits(&self) -> &HashSet<String> {
		&self.user_config.ignored_traits
	}
}

impl From<UserConfig> for Config {
	fn from(user_config: UserConfig) -> Self {
		Self {
			user_config,
			projections: HashMap::new(),
			module_defaults: HashMap::new(),
			scoped_defaults: HashMap::new(),
			concrete_types: HashSet::new(),
			self_type_name: None,
		}
	}
}

fn default_ignored_traits() -> HashSet<String> {
	[
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
	]
	.iter()
	.map(|s| s.to_string())
	.collect()
}
