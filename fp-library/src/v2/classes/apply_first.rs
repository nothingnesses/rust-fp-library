use crate::hkt::Apply1L1T;
use super::lift::Lift;

/// A type class for types that support combining two contexts, keeping the first value.
///
/// `ApplyFirst` provides the ability to sequence two computations but discard
/// the result of the second computation, keeping only the result of the first.
pub trait ApplyFirst: Lift {
    /// Combines two contexts, keeping the value from the first context.
    fn apply_first<'a, A: 'a + Clone, B: 'a + Clone>(
        fa: Apply1L1T<'a, Self, A>,
        fb: Apply1L1T<'a, Self, B>
    ) -> Apply1L1T<'a, Self, A> {
        Self::lift2(|a, _| a, fa, fb)
    }
}

/// Combines two contexts, keeping the value from the first context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplyFirst::apply_first`].
pub fn apply_first<'a, Brand: ApplyFirst, A: 'a + Clone, B: 'a + Clone>(
    fa: Apply1L1T<'a, Brand, A>,
    fb: Apply1L1T<'a, Brand, B>
) -> Apply1L1T<'a, Brand, A> {
    Brand::apply_first(fa, fb)
}
