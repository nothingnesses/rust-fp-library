//! Concrete data types, their corresponding implementations and type aliases.
//!
//! This module provides implementations of various functional programming
//! data structures and wrappers, including `Identity`, `Memo`, and extensions
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
pub mod cat_queue;
pub mod endofunction;
pub mod endomorphism;
pub mod eval;
pub mod fn_brand;
pub mod free;
pub mod identity;
pub mod memo;
pub mod option;
pub mod pair;
pub mod rc_ptr;
pub mod result;
pub mod send_endofunction;
pub mod step;
pub mod string;
pub mod task;
pub mod thunk;
pub mod try_eval;
pub mod try_memo;
pub mod try_task;
pub mod vec;

pub use cat_list::CatList;
pub use cat_queue::CatQueue;
pub use endofunction::Endofunction;
pub use endomorphism::Endomorphism;
pub use eval::Eval;
pub use free::Free;
pub use identity::Identity;
pub use memo::{ArcMemo, ArcMemoConfig, Memo, MemoConfig, RcMemo, RcMemoConfig};
pub use pair::Pair;
pub use send_endofunction::SendEndofunction;
pub use step::Step;
pub use task::Task;
pub use thunk::Thunk;
pub use try_eval::TryEval;
pub use try_memo::{ArcTryMemo, RcTryMemo, TryMemo};
pub use try_task::TryTask;
