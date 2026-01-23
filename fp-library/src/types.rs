//! Concrete data types, their corresponding implementations and type aliases.
//!
//! This module provides implementations of various functional programming
//! data structures and wrappers, including `Identity`, `Lazy`, and extensions
//! for standard library types like `Option` and `Result`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::Identity;
//!
//! let x = Identity(5);
//! assert_eq!(x.0, 5);
//! ```

pub mod arc_ptr;
pub mod endofunction;
pub mod endomorphism;
pub mod fn_brand;
pub mod identity;
pub mod lazy;
pub mod once_cell;
pub mod once_lock;
pub mod option;
pub mod pair;
pub mod rc_ptr;
pub mod result;
pub mod send_endofunction;
pub mod string;
pub mod vec;

pub use endofunction::Endofunction;
pub use endomorphism::Endomorphism;
pub use identity::Identity;
pub use lazy::{ArcLazy, ArcLazyConfig, Lazy, LazyConfig, LazyError, RcLazy, RcLazyConfig};
pub use pair::Pair;
pub use send_endofunction::SendEndofunction;
