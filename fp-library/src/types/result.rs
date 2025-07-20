use crate::hkt::Kind2;

pub mod result_with_err;
pub mod result_with_ok;

pub use result_with_err::*;
pub use result_with_ok::*;

pub struct ResultBrand;

impl<A, B> Kind2<A, B> for ResultBrand {
	type Output = Result<B, A>;
}
