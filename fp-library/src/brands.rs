//! Higher-kinded representations of [types][crate::types].
//!
//! Brands represent higher-kinded (unapplied/partially-applied) forms of
//! [types][crate::types], as opposed to concrete types, which are
//! fully-applied.
//!
//! For example, [`VecBrand`] represents the higher-kinded type `Vec`, whereas
//! `Vec A`/`Vec<A>` is the concrete type where `Vec` has been applied to some
//! generic type `A`.

pub use crate::types::{
	arc_fn::ArcFnBrand,
	endofunction::EndofunctionBrand,
	endomorphism::EndomorphismBrand,
	identity::IdentityBrand,
	lazy::LazyBrand,
	once_cell::OnceCellBrand,
	once_lock::OnceLockBrand,
	option::OptionBrand,
	pair::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	rc_fn::RcFnBrand,
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	vec::VecBrand,
};
