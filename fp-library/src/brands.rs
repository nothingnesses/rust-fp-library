//! Higher-kinded representation of [types][crate::types].

pub use crate::types::{
	endomorphism::EndomorphismBrand,
	option::OptionBrand,
	pair::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	solo::SoloBrand,
	string::StringBrand,
	vec::{ConcreteVecBrand, VecBrand},
};
