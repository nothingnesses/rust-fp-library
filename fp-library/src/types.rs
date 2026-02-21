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
pub mod optics;
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
pub mod tuple_1;
pub mod tuple_2;
pub mod vec;

pub use {
	cat_list::CatList,
	endofunction::Endofunction,
	endomorphism::Endomorphism,
	free::Free,
	identity::Identity,
	lazy::{
		ArcLazy,
		ArcLazyConfig,
		Lazy,
		LazyConfig,
		RcLazy,
		RcLazyConfig,
	},
	optics::{
		Composed,
		Lens,
		LensPrime,
	},
	pair::Pair,
	send_endofunction::SendEndofunction,
	step::Step,
	thunk::Thunk,
	trampoline::Trampoline,
	try_lazy::{
		ArcTryLazy,
		RcTryLazy,
		TryLazy,
	},
	try_thunk::TryThunk,
	try_trampoline::TryTrampoline,
};
