//! Higher-kinded representation of [types][crate::types].

pub use super::types::{
	option::OptionBrand,
	pair::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	solo::SoloBrand,
	vec::VecBrand,
};

use crate::hkt::Kind;

/// Contains functions to convert between the concrete type and the
/// corresponding instantiation of [`Apply`][crate::hkt::apply::Apply].
pub trait Brand<Concrete, Parameters>: Kind<Parameters> {
	fn inject(a: Concrete) -> Self::Output;
	fn project(a: Self::Output) -> Concrete;
}

pub use crate::macros::hkt::{Brand1, Brand2, Brand3, Brand4};
