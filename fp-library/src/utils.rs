//! Internal utility functions shared across the crate.

use std::any::Any;

/// Converts a panic payload into a human-readable `String`.
///
/// Handles the common cases where the payload is a `&str` or `String`,
/// and falls back to `"Unknown panic"` for other types.
pub(crate) fn panic_payload_to_string(payload: Box<dyn Any + Send>) -> String {
	if let Some(s) = payload.downcast_ref::<&str>() {
		s.to_string()
	} else if let Some(s) = payload.downcast_ref::<String>() {
		s.clone()
	} else {
		"Unknown panic".to_string()
	}
}
