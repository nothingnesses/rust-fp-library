//! Configuration subsystem for the macro system.
//!
//! This module handles:
//! - Loading configuration from Cargo.toml
//! - Configuration types and validation
//! - Runtime configuration state

use crate::{resolution::ProjectionKey,core::constants::{config_names, default_traits}};
use serde::Deserialize;
use std::{
	collections::{HashMap, HashSet},
	sync::LazyLock,
};

// ==================== Configuration Types ====================

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
	default_traits::DEFAULT_IGNORED_TRAITS
		.iter()
		.map(|s| s.to_string())
		.collect()
}

// ==================== Configuration Loading ====================

#[derive(Debug, Deserialize)]
struct CargoMetadata {
	hm_signature: Option<UserConfig>,
}

#[derive(Debug, Deserialize)]
struct CargoManifest {
	package: Option<PackageMetadata>,
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
	metadata: Option<CargoMetadata>,
}

/// Static cache for user configuration loaded from Cargo.toml.
///
/// This LazyLock ensures the configuration is loaded from disk only once per compilation,
/// significantly improving performance in large codebases where macros are invoked
/// hundreds or thousands of times.
///
/// When configuration loading fails, diagnostic warnings are emitted to stderr to help
/// users understand why their settings aren't being applied.
static USER_CONFIG_CACHE: LazyLock<UserConfig> = LazyLock::new(|| {
	match load_user_config_impl() {
		Ok(config) => config,
		Err(ConfigLoadError::NotFound) => {
			// Silently use defaults when no config is present - this is expected
			UserConfig::default()
		}
		Err(e) => {
			// Emit warning for actual errors so users know their config is being ignored
			eprintln!(
				"warning: Failed to load [package.metadata.{}] configuration: {}",
				config_names::CONFIG_SECTION,
				e
			);
			eprintln!("         Using default configuration instead.");
			eprintln!(
				"         Check your Cargo.toml for syntax errors in the [package.metadata.{}] section.",
				config_names::CONFIG_SECTION
			);
			UserConfig::default()
		}
	}
});

/// Errors that can occur when loading user configuration.
#[derive(Debug)]
enum ConfigLoadError {
	/// No configuration file or section found (expected, not an error)
	NotFound,
	/// IO error reading Cargo.toml
	IoError(std::io::Error),
	/// TOML parsing error
	TomlError(toml::de::Error),
	/// Configuration structure is invalid
	InvalidStructure(String),
}

impl std::fmt::Display for ConfigLoadError {
	fn fmt(
		&self,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		match self {
			ConfigLoadError::NotFound => write!(f, "configuration not found"),
			ConfigLoadError::IoError(e) => write!(f, "failed to read Cargo.toml: {}", e),
			ConfigLoadError::TomlError(e) => write!(f, "invalid TOML syntax: {}", e),
			ConfigLoadError::InvalidStructure(msg) => {
				write!(f, "invalid configuration structure: {}", msg)
			}
		}
	}
}

/// Implementation of configuration loading with proper error reporting.
///
/// This function attempts to load configuration from Cargo.toml and returns
/// detailed errors for any failures, allowing the caller to decide how to handle them.
fn load_user_config_impl() -> Result<UserConfig, ConfigLoadError> {
	let manifest_dir = std::env::var(config_names::CARGO_MANIFEST_DIR).unwrap_or_else(|_| ".".to_string());
	let manifest_path = std::path::Path::new(&manifest_dir).join(config_names::CARGO_TOML);

	// Read the file
	let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
		// Distinguish between "file not found" and other IO errors
		if e.kind() == std::io::ErrorKind::NotFound {
			ConfigLoadError::NotFound
		} else {
			ConfigLoadError::IoError(e)
		}
	})?;

	// Parse TOML
	let manifest: CargoManifest = toml::from_str(&content).map_err(ConfigLoadError::TomlError)?;

	// Navigate the structure
	let package = manifest.package.ok_or_else(|| {
		ConfigLoadError::InvalidStructure("missing [package] section".to_string())
	})?;

	let metadata = package.metadata.ok_or(ConfigLoadError::NotFound)?; // No metadata is expected, not an error

	let user_config = metadata.hm_signature.ok_or(ConfigLoadError::NotFound)?; // No hm_signature section is expected, not an error

	Ok(user_config)
}

/// Loads user configuration from Cargo.toml (cached).
///
/// This reads the `[package.metadata.hm_signature]` section and returns a cached UserConfig.
/// The configuration is loaded only once per compilation, improving performance.
/// The returned UserConfig can then be converted to a full Config using `Config::from()`.
///
/// Note: The configuration section name is defined in [`config_names::CONFIG_SECTION`].
pub fn load_user_config() -> UserConfig {
	USER_CONFIG_CACHE.clone()
}

/// Loads a complete Config by reading Cargo.toml (cached) and initializing runtime state.
///
/// This is a convenience function that loads the cached user config and converts it to Config.
/// The user configuration is loaded from disk only once per compilation.
pub fn load_config() -> Config {
	load_user_config().into()
}

/// Gets the cached configuration.
///
/// This is a convenience alias for `load_config()` that emphasizes the caching behavior.
/// The configuration is loaded from Cargo.toml only once per compilation and then cached.
///
/// # Examples
///
/// ```ignore
/// use fp_macros::core::config::get_config;
///
/// let config = get_config();
/// // Use config...
/// ```
#[inline]
pub fn get_config() -> Config {
	load_config()
}
