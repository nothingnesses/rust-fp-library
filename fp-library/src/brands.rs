//! Higher-kinded representation of [types][crate::types].

pub use crate::types::{
	arc_fn::ArcFnBrand,
	option::OptionBrand,
	pair::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	rc_fn::RcFnBrand,
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	solo::SoloBrand,
	vec::VecBrand,
};
