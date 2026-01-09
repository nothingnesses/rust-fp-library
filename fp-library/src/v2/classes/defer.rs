use super::clonable_fn::{ApplyClonableFn, ClonableFn};

/// A type class for types that can be constructed lazily.
pub trait Defer<'a> {
    /// Creates a value from a computation that produces the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::defer::Defer;
    /// use fp_library::v2::types::lazy::Lazy;
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use fp_library::v2::types::once_cell::OnceCellBrand;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    ///
    /// let lazy = Lazy::<OnceCellBrand, RcFnBrand, _>::defer::<RcFnBrand>(
    ///     <RcFnBrand as ClonableFn>::new(|_| Lazy::new(<RcFnBrand as ClonableFn>::new(|_| 42)))
    /// );
    /// assert_eq!(Lazy::force(lazy), 42);
    /// ```
    fn defer<ClonableFnBrand: 'a + ClonableFn>(
        f: ApplyClonableFn<'a, ClonableFnBrand, (), Self>
    ) -> Self
    where
        Self: Sized;
}
