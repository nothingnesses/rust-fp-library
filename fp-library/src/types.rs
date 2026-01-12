//! Concrete data types, their corresponding implementations and type aliases.
//!
//! This module provides implementations of various functional programming
//! data structures and wrappers, including `Identity`, `Lazy`, and extensions
//! for standard library types like `Option` and `Result`.

pub mod arc_fn;
pub mod endofunction;
pub mod endomorphism;
pub mod identity;
pub mod lazy;
pub mod once_cell;
pub mod once_lock;
pub mod option;
pub mod pair;
pub mod rc_fn;
pub mod result;
pub mod string;
pub mod vec;

pub use endofunction::Endofunction;
pub use endomorphism::Endomorphism;
pub use identity::Identity;
pub use lazy::Lazy;
pub use pair::Pair;
