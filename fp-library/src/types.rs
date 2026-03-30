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

pub mod additive;
pub mod arc_ptr;
pub mod cat_list;
pub mod conjunctive;
pub mod const_val;
pub mod control_flow;
pub mod coyoneda;
pub mod disjunctive;
pub mod dual;
pub mod endofunction;
pub mod endomorphism;
pub mod first;
pub mod fn_brand;
pub mod free;
pub mod identity;
pub mod last;
pub mod lazy;
pub mod multiplicative;
pub mod optics;
pub mod option;
pub mod pair;
pub mod rc_ptr;
pub mod result;
pub mod send_thunk;
pub mod string;
pub mod thunk;
pub mod trampoline;
pub mod try_lazy;
pub mod try_send_thunk;
pub mod try_thunk;
pub mod try_trampoline;
pub mod tuple_1;
pub mod tuple_2;
pub mod vec;

pub use {
	additive::Additive,
	cat_list::CatList,
	conjunctive::Conjunctive,
	coyoneda::Coyoneda,
	disjunctive::Disjunctive,
	dual::Dual,
	endofunction::Endofunction,
	endomorphism::Endomorphism,
	first::First,
	free::{
		Free,
		FreeStep,
	},
	identity::Identity,
	last::Last,
	lazy::{
		ArcLazy,
		ArcLazyConfig,
		Lazy,
		RcLazy,
		RcLazyConfig,
	},
	multiplicative::Multiplicative,
	optics::{
		Composed,
		Lens,
		LensPrime,
	},
	pair::Pair,
	send_thunk::SendThunk,
	thunk::Thunk,
	trampoline::Trampoline,
	try_lazy::{
		ArcTryLazy,
		RcTryLazy,
		TryLazy,
	},
	try_send_thunk::TrySendThunk,
	try_thunk::TryThunk,
	try_trampoline::TryTrampoline,
};
