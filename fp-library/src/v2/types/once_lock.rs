use crate::{
    hkt::{Apply0L1T, Kind0L1T},
    v2::classes::{once::ApplyOnce, once::Once},
};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceLockBrand;

impl Kind0L1T for OnceLockBrand {
    type Output<A> = OnceLock<A>;
}

impl Once for OnceLockBrand {
    type Output<A> = Apply0L1T<Self, A>;

    fn new<A>() -> ApplyOnce<Self, A> {
        OnceLock::new()
    }

    fn get<A>(a: &ApplyOnce<Self, A>) -> Option<&A> {
        OnceLock::get(a)
    }

    fn get_mut<A>(a: &mut ApplyOnce<Self, A>) -> Option<&mut A> {
        OnceLock::get_mut(a)
    }

    fn set<A>(a: &ApplyOnce<Self, A>, value: A) -> Result<(), A> {
        OnceLock::set(a, value)
    }

    fn get_or_init<A, B: FnOnce() -> A>(a: &ApplyOnce<Self, A>, f: B) -> &A {
        OnceLock::get_or_init(a, f)
    }

    fn into_inner<A>(a: ApplyOnce<Self, A>) -> Option<A> {
        OnceLock::into_inner(a)
    }

    fn take<A>(a: &mut ApplyOnce<Self, A>) -> Option<A> {
        OnceLock::take(a)
    }
}
