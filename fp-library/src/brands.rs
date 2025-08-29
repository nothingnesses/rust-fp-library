//! Higher-kinded representations of [types][crate::types].

pub use crate::types::{
	arc_fn::ArcFnBrand,
	identity::IdentityBrand,
	option::OptionBrand,
	pair::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	rc_fn::RcFnBrand,
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	vec::VecBrand,
};
