use crate::hkt::{Apply0L1T, Kind0L1T};
use super::lift::Lift;

/// A type class for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
pub trait ApplySecond: Lift {
    /// Combines two contexts, keeping the value from the second context.
    fn apply_second<'a, A: 'a + Clone, B: 'a + Clone>(
        fa: Apply0L1T<Self, A>,
        fb: Apply0L1T<Self, B>
    ) -> Apply0L1T<Self, B> {
        Self::lift2(|_, b| b, fa, fb)
    }
}

/// Combines two contexts, keeping the value from the second context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplySecond::apply_second`].
pub fn apply_second<'a, Brand: ApplySecond, A: 'a + Clone, B: 'a + Clone>(
    fa: Apply0L1T<Brand, A>,
    fb: Apply0L1T<Brand, B>
) -> Apply0L1T<Brand, B> {
    Brand::apply_second(fa, fb)
}
