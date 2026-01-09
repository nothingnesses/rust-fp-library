use crate::{
    hkt::{Apply0L1T, Kind0L1T},
    v2::classes::{once::ApplyOnce, once::Once},
};
use std::cell::OnceCell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceCellBrand;

impl Kind0L1T for OnceCellBrand {
    type Output<A> = OnceCell<A>;
}

impl Once for OnceCellBrand {
    type Output<A> = Apply0L1T<Self, A>;

    fn new<A>() -> ApplyOnce<Self, A> {
        OnceCell::new()
    }

    fn get<A>(a: &ApplyOnce<Self, A>) -> Option<&A> {
        OnceCell::get(a)
    }

    fn get_mut<A>(a: &mut ApplyOnce<Self, A>) -> Option<&mut A> {
        OnceCell::get_mut(a)
    }

    fn set<A>(
        a: &ApplyOnce<Self, A>,
        value: A,
    ) -> Result<(), A> {
        OnceCell::set(a, value)
    }

    fn get_or_init<A, B: FnOnce() -> A>(
        a: &ApplyOnce<Self, A>,
        f: B,
    ) -> &A {
        OnceCell::get_or_init(a, f)
    }

    fn into_inner<A>(a: ApplyOnce<Self, A>) -> Option<A> {
        OnceCell::into_inner(a)
    }

    fn take<A>(a: &mut ApplyOnce<Self, A>) -> Option<A> {
        OnceCell::take(a)
    }
}
