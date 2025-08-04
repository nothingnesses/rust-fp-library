//! Implementations for [`Result`].

pub mod result_with_err;
pub mod result_with_ok;

use crate::hkt::{Apply, Brand2, Kind2};
pub use result_with_err::*;
pub use result_with_ok::*;

/// [Brand][crate::brands] for [`Result`].
pub struct ResultBrand;

impl<A, B> Kind2<A, B> for ResultBrand {
	type Output = Result<B, A>;
}

impl<A, B> Brand2<Result<B, A>, A, B> for ResultBrand {
	fn inject(a: Result<B, A>) -> Apply<Self, (A, B)> {
		a
	}

	fn project(a: Apply<Self, (A, B)>) -> Result<B, A> {
		a
	}
}
