use crate::{
    brands::OptionBrand,
    hkt::Apply0L1T,
    v2::classes::{
        applicative::Applicative,
        clonable_fn::{ApplyClonableFn, ClonableFn},
        foldable::Foldable,
        functor::Functor,
        lift::Lift,
        monoid::Monoid,
        pointed::Pointed,
        semiapplicative::Semiapplicative,
        semimonad::Semimonad,
        traversable::Traversable,
    },
};

impl Functor for OptionBrand {
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> B,
    {
        fa.map(f)
    }
}

impl Lift for OptionBrand {
    fn lift2<'a, A: 'a, B: 'a, C: 'a, F: 'a>(
        f: F,
        fa: Apply0L1T<Self, A>,
        fb: Apply0L1T<Self, B>,
    ) -> Apply0L1T<Self, C>
    where
        F: Fn(A, B) -> C,
        A: Clone,
        B: Clone,
    {
        fa.zip(fb).map(|(a, b)| f(a, b))
    }
}

impl Pointed for OptionBrand {
    fn pure<A>(a: A) -> Apply0L1T<Self, A> {
        Some(a)
    }
}

impl Semiapplicative for OptionBrand {
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply0L1T<Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply0L1T<Self, A>,
    ) -> Apply0L1T<Self, B> {
        match (ff, fa) {
            (Some(f), Some(a)) => Some(f(a)),
            _ => None,
        }
    }
}

impl Semimonad for OptionBrand {
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply0L1T<Self, A>,
        f: F,
    ) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> Apply0L1T<Self, B>,
    {
        ma.and_then(f)
    }
}

impl Foldable for OptionBrand {
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        match fa {
            Some(a) => f(a, init),
            None => init,
        }
    }

    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        match fa {
            Some(a) => f(init, a),
            None => init,
        }
    }

    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M,
    {
        match fa {
            Some(a) => f(a),
            None => M::empty(),
        }
    }
}

impl Traversable for OptionBrand {
    fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
        f: Func,
        ta: Apply0L1T<Self, A>,
    ) -> Apply0L1T<F, Apply0L1T<Self, B>>
    where
        Func: Fn(A) -> Apply0L1T<F, B>,
        Apply0L1T<Self, B>: Clone,
    {
        match ta {
            Some(a) => F::map(|b| Some(b), f(a)),
            None => F::pure(None),
        }
    }

    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply0L1T<Self, Apply0L1T<F, A>>,
    ) -> Apply0L1T<F, Apply0L1T<Self, A>>
    where
        Apply0L1T<F, A>: Clone,
        Apply0L1T<Self, A>: Clone,
    {
        match ta {
            Some(fa) => F::map(|a| Some(a), fa),
            None => F::pure(None),
        }
    }
}
