use crate::{
    brands::IdentityBrand,
    hkt::{Apply1L1T, Kind1L1T},
    types::Identity,
    v2::classes::{
        applicative::Applicative,
        apply_first::ApplyFirst,
        apply_second::ApplySecond,
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

impl Kind1L1T for IdentityBrand {
    type Output<'a, A: 'a> = Identity<A>;
}

impl Functor for IdentityBrand {
    /// Maps a function over the value in the identity.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::functor::map;
    /// use fp_library::brands::IdentityBrand;
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(map::<IdentityBrand, _, _, _>(|x: i32| x * 2, Identity(5)), Identity(10));
    /// ```
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> B,
    {
        Identity(f(fa.0))
    }
}

impl Lift for IdentityBrand {
    /// Lifts a binary function into the identity context.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::lift2;
    /// use fp_library::brands::IdentityBrand;
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(
    ///     lift2::<IdentityBrand, _, _, _, _>(|x: i32, y: i32| x + y, Identity(1), Identity(2)),
    ///     Identity(3)
    /// );
    /// ```
    fn lift2<'a, A: 'a, B: 'a, C: 'a, F: 'a>(
        f: F,
        fa: Apply1L1T<'a, Self, A>,
        fb: Apply1L1T<'a, Self, B>,
    ) -> Apply1L1T<'a, Self, C>
    where
        F: Fn(A, B) -> C,
        A: Clone,
        B: Clone,
    {
        Identity(f(fa.0, fb.0))
    }
}

impl Pointed for IdentityBrand {
    /// Wraps a value in an identity.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::pointed::pure;
    /// use fp_library::brands::IdentityBrand;
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(pure::<IdentityBrand, _>(5), Identity(5));
    /// ```
    fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
        Identity(a)
    }
}

impl ApplyFirst for IdentityBrand {}
impl ApplySecond for IdentityBrand {}

impl Semiapplicative for IdentityBrand {
    /// Applies a wrapped function to a wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semiapplicative::apply;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    /// use fp_library::brands::{IdentityBrand};
    /// use fp_library::types::Identity;
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use std::rc::Rc;
    ///
    /// let f = Identity(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
    /// assert_eq!(apply::<IdentityBrand, _, _, RcFnBrand>(f, Identity(5)), Identity(10));
    /// ```
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply1L1T<'a, Self, A>,
    ) -> Apply1L1T<'a, Self, B> {
        Identity(ff.0(fa.0))
    }
}

impl Semimonad for IdentityBrand {
    /// Chains identity computations.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::bind;
    /// use fp_library::brands::IdentityBrand;
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(
    ///     bind::<IdentityBrand, _, _, _>(Identity(5), |x| Identity(x * 2)),
    ///     Identity(10)
    /// );
    /// ```
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply1L1T<'a, Self, A>,
        f: F,
    ) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> Apply1L1T<'a, Self, B>,
    {
        f(ma.0)
    }
}

impl Foldable for IdentityBrand {
    /// Folds the identity from the right.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_right;
    /// use fp_library::brands::IdentityBrand;
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(fold_right::<IdentityBrand, _, _, _>(|x: i32, acc| x + acc, 0, Identity(5)), 5);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        f(fa.0, init)
    }

    /// Folds the identity from the left.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_left;
    /// use fp_library::brands::IdentityBrand;
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(fold_left::<IdentityBrand, _, _, _>(|acc, x: i32| acc + x, 0, Identity(5)), 5);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        f(init, fa.0)
    }

    /// Maps the value to a monoid and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_map;
    /// use fp_library::brands::IdentityBrand;
    /// use fp_library::types::Identity;
    /// use fp_library::v2::types::string; // Import to bring Monoid impl for String into scope
    ///
    /// assert_eq!(fold_map::<IdentityBrand, _, _, _>(|x: i32| x.to_string(), Identity(5)), "5".to_string());
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M,
    {
        f(fa.0)
    }
}

impl Traversable for IdentityBrand {
    /// Traverses the identity with an applicative function.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::traverse;
    /// use fp_library::brands::{IdentityBrand, OptionBrand};
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(
    ///     traverse::<IdentityBrand, OptionBrand, _, _, _>(|x| Some(x * 2), Identity(5)),
    ///     Some(Identity(10))
    /// );
    /// ```
    fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
        f: Func,
        ta: Apply1L1T<'a, Self, A>,
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, B>>
    where
        Func: Fn(A) -> Apply1L1T<'a, F, B>,
        Apply1L1T<'a, Self, B>: Clone,
    {
        F::map(|b| Identity(b), f(ta.0))
    }

    /// Sequences an identity of applicative.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::sequence;
    /// use fp_library::brands::{IdentityBrand, OptionBrand};
    /// use fp_library::types::Identity;
    ///
    /// assert_eq!(
    ///     sequence::<IdentityBrand, OptionBrand, _>(Identity(Some(5))),
    ///     Some(Identity(5))
    /// );
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>,
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
    where
        Apply1L1T<'a, F, A>: Clone,
        Apply1L1T<'a, Self, A>: Clone,
    {
        F::map(|a| Identity(a), ta.0)
    }
}
