//! Type aliases.

use std::sync::Arc;

/// A clonable function type that wraps an `Arc` of a `Fn` trait object,
/// representing a dynamically-dispatched function.
///
/// This type alias provides a way to store heap-allocated functions that can be cloned
/// and shared across contexts. The lifetime `'a` ensures the function doesn't outlive
/// referenced data, while `A` and `B` represent the input and output types, respectively.
pub type ArcFn<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;
