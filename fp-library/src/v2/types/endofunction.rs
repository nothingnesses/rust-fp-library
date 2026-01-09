//! Implementations for [`Endofunction`], a wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.

use crate::v2::classes::{
    clonable_fn::{ApplyClonableFn, ClonableFn},
    monoid::Monoid,
    semigroup::Semigroup,
};
use crate::v2::functions::identity;
use std::fmt::{self, Debug, Formatter};
use std::hash::Hash;

/// A wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.
///
/// `Endofunction a` represents a function `a -> a`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [function composition][crate::v2::functions::compose].
/// * The identity element [empty][Monoid::empty] is the [identity function][crate::v2::functions::identity].
///
/// The wrapped function can be accessed directly via the [`.0` field][Endofunction#structfield.0].
pub struct Endofunction<'a, CFB: ClonableFn, A>(pub ApplyClonableFn<'a, CFB, A, A>);

impl<'a, CFB: ClonableFn, A> Endofunction<'a, CFB, A> {
    pub fn new(f: ApplyClonableFn<'a, CFB, A, A>) -> Self {
        Self(f)
    }
}

impl<'a, CFB: ClonableFn, A> Clone for Endofunction<'a, CFB, A> {
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<'a, CFB: ClonableFn, A> Debug for Endofunction<'a, CFB, A>
where
    ApplyClonableFn<'a, CFB, A, A>: Debug,
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("Endofunction").field(&self.0).finish()
    }
}

impl<'a, CFB: ClonableFn, A> Eq for Endofunction<'a, CFB, A> where ApplyClonableFn<'a, CFB, A, A>: Eq {}

impl<'a, CFB: ClonableFn, A> Hash for Endofunction<'a, CFB, A>
where
    ApplyClonableFn<'a, CFB, A, A>: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a, CFB: ClonableFn, A> Ord for Endofunction<'a, CFB, A>
where
    ApplyClonableFn<'a, CFB, A, A>: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<'a, CFB: ClonableFn, A> PartialEq for Endofunction<'a, CFB, A>
where
    ApplyClonableFn<'a, CFB, A, A>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, CFB: ClonableFn, A> PartialOrd for Endofunction<'a, CFB, A>
where
    ApplyClonableFn<'a, CFB, A, A>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a, CFB: 'a + ClonableFn, A: 'a> Semigroup for Endofunction<'a, CFB, A> {
    fn append(a: Self, b: Self) -> Self {
        let f = a.0;
        let g = b.0;
        // Compose: f . g
        Self::new(<CFB as ClonableFn>::new(move |x| f(g(x))))
    }
}

impl<'a, CFB: 'a + ClonableFn, A: 'a> Monoid for Endofunction<'a, CFB, A> {
    fn empty() -> Self {
        Self::new(<CFB as ClonableFn>::new(identity))
    }
}
