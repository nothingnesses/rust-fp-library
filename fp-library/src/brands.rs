//! Higher-kinded representation of [types][crate::types].

pub use crate::types::{
	option::OptionBrand,
	pair::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	solo::SoloBrand,
	string::StringBrand,
	string_hkt::StringBrandHKT,
	vec::VecBrand,
};
