use crate::hkt::Apply1L1T;
use super::{clonable_fn::{ApplyClonableFn, ClonableFn}, functor::Functor, lift::Lift};

/// A type class for types that support function application within a context.
///
/// `Semiapplicative` provides the ability to apply functions that are themselves
/// wrapped in a context to values that are also wrapped in a context.
///
/// # Laws
///
/// `Semiapplicative` instances must satisfy the following law:
/// * Composition: `apply(apply(f, g), x) = apply(f, apply(g, x))`.
pub trait Semiapplicative: Lift + Functor {
    /// Applies a function within a context to a value within a context.
    ///
    /// **Important**: This operation requires type erasure for heterogeneous functions.
    /// When a container (like `Vec`) holds multiple different closures, they must be
    /// type-erased via `Rc<dyn Fn>` or `Arc<dyn Fn>` because each Rust closure is a
    /// distinct anonymous type.
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply1L1T<'a, Self, A>
    ) -> Apply1L1T<'a, Self, B>;
}

/// Applies a function within a context to a value within a context.
///
/// Free function version that dispatches to [the type class' associated function][`Semiapplicative::apply`].
pub fn apply<'a, Brand: Semiapplicative, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
    ff: Apply1L1T<'a, Brand, ApplyClonableFn<'a, FnBrand, A, B>>,
    fa: Apply1L1T<'a, Brand, A>
) -> Apply1L1T<'a, Brand, B> {
    Brand::apply::<A, B, FnBrand>(ff, fa)
}
