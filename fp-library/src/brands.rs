//! Representations of higher-kinded types.

use crate::hkt::{Apply, Kind};

pub use super::types::{
	option::OptionBrand,
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	solo::SoloBrand,
};

/// Trait containing functions to convert between the concrete type and the
/// corresponding instantiation of [`Apply`](../hkt/apply/type.Apply.html).
pub trait Brand<Concrete, A>
where
	Self: Kind<A>,
{
	fn inject(a: Concrete) -> Apply<Self, A>;
	fn project(a: Apply<Self, A>) -> Concrete;
}
