use crate::hkt::Apply1L1T;
use super::lift::Lift;

/// A type class for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
pub trait ApplySecond: Lift {
    /// Combines two contexts, keeping the value from the second context.
    fn apply_second<'a, A: 'a + Clone, B: 'a + Clone>(
        fa: Apply1L1T<'a, Self, A>,
        fb: Apply1L1T<'a, Self, B>
    ) -> Apply1L1T<'a, Self, B> {
        Self::lift2(|_, b| b, fa, fb)
    }
}

/// Combines two contexts, keeping the value from the second context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplySecond::apply_second`].
pub fn apply_second<'a, Brand: ApplySecond, A: 'a + Clone, B: 'a + Clone>(
    fa: Apply1L1T<'a, Brand, A>,
    fb: Apply1L1T<'a, Brand, B>
) -> Apply1L1T<'a, Brand, B> {
    Brand::apply_second(fa, fb)
}
