//! Type aliases.

use std::sync::Arc;

/// A type alias for a clonable, dynamically-dispatched function.
pub type ClonableFn<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;
