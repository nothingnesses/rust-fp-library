use crate::{
    hkt::{Apply1L2T, Kind1L2T},
    v2::classes::{
        category::Category,
        clonable_fn::{ApplyClonableFn, ClonableFn},
        function::{ApplyFunction, Function},
        semigroupoid::Semigroupoid,
    },
};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;

impl Kind1L2T for RcFnBrand {
    type Output<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
}

impl Function for RcFnBrand {
    type Output<'a, A, B> = Apply1L2T<'a, Self, A, B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyFunction<'a, Self, A, B> {
        Rc::new(f)
    }
}

impl ClonableFn for RcFnBrand {
    type Output<'a, A, B> = Apply1L2T<'a, Self, A, B>;

    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyClonableFn<'a, Self, A, B> {
        Rc::new(f)
    }
}

impl Semigroupoid for RcFnBrand {
    fn compose<'a, B: 'a, C: 'a, D: 'a>(
        f: Apply1L2T<'a, Self, C, D>,
        g: Apply1L2T<'a, Self, B, C>,
    ) -> Apply1L2T<'a, Self, B, D> {
        <Self as ClonableFn>::new(move |b| f(g(b)))
    }
}

impl Category for RcFnBrand {
    fn identity<'a, A>() -> Apply1L2T<'a, Self, A, A> {
        Rc::new(|a| a)
    }
}
