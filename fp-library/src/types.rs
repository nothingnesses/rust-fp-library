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
pub mod cat_list;
pub mod endofunction;
pub mod endomorphism;
pub mod fn_brand;
pub mod free;
pub mod identity;
pub mod lazy;
pub mod option;
pub mod pair;
pub mod rc_ptr;
pub mod result;
pub mod send_endofunction;
pub mod step;
pub mod string;
pub mod thunk;
pub mod trampoline;
pub mod try_lazy;
pub mod try_thunk;
pub mod try_trampoline;
pub mod vec;

pub use cat_list::CatList;
pub use endofunction::Endofunction;
pub use endomorphism::Endomorphism;
pub use free::Free;
pub use identity::Identity;
pub use lazy::{ArcLazy, ArcLazyConfig, Lazy, LazyConfig, RcLazy, RcLazyConfig};
pub use pair::Pair;
pub use send_endofunction::SendEndofunction;
pub use step::Step;
pub use thunk::Thunk;
pub use trampoline::Trampoline;
pub use try_lazy::{ArcTryLazy, RcTryLazy, TryLazy};
pub use try_thunk::TryThunk;
pub use try_trampoline::TryTrampoline;
