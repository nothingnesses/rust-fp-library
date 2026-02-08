use syn::{Error, Result};

/// Utility for collecting and combining multiple errors.
/// Replaces the repeated pattern of error accumulation throughout the codebase.
pub struct ErrorCollector {
	errors: Vec<Error>,
}

impl ErrorCollector {
	pub fn new() -> Self {
		Self { errors: Vec::new() }
	}

	pub fn push(
		&mut self,
		error: Error,
	) {
		self.errors.push(error);
	}

	pub fn extend(
		&mut self,
		other_errors: Vec<Error>,
	) {
		self.errors.extend(other_errors);
	}

	pub fn finish(self) -> Result<()> {
		if self.errors.is_empty() { Ok(()) } else { Err(Self::combine_errors(self.errors)) }
	}

	fn combine_errors(mut errors: Vec<Error>) -> Error {
		let mut combined = errors.remove(0);
		for err in errors {
			combined.combine(err);
		}
		combined
	}
}
