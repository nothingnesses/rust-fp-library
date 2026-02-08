//! Configuration loading from Cargo.toml.

use super::types::{Config, UserConfig};
use serde::Deserialize;
use std::sync::LazyLock;

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
				"warning: Failed to load [package.metadata.hm_signature] configuration: {}",
				e
			);
			eprintln!("         Using default configuration instead.");
			eprintln!(
				"         Check your Cargo.toml for syntax errors in the [package.metadata.hm_signature] section."
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
	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
	let manifest_path = std::path::Path::new(&manifest_dir).join("Cargo.toml");

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
