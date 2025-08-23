//! Implementations for [`Result`].

pub mod result_with_err;
pub mod result_with_ok;

use crate::hkt::Kind0L2T;
pub use result_with_err::*;
pub use result_with_ok::*;

/// [Brand][crate::brands] for [`Result`].
pub struct ResultBrand;

impl Kind0L2T for ResultBrand {
	type Output<A, B> = Result<B, A>;
}
