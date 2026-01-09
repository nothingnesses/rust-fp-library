use crate::{
    brands::VecBrand,
    hkt::{Apply1L1T, Kind1L1T},
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

impl Kind1L1T for VecBrand {
    type Output<'a, A: 'a> = Vec<A>;
}

impl Functor for VecBrand {
    /// Maps a function over the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::functor::map;
    /// use fp_library::brands::VecBrand;
    ///
    /// assert_eq!(map::<VecBrand, _, _, _>(|x: i32| x * 2, vec![1, 2, 3]), vec![2, 4, 6]);
    /// ```
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> B,
    {
        fa.into_iter().map(f).collect()
    }
}

impl Lift for VecBrand {
    /// Lifts a binary function into the vector context (Cartesian product).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::lift2;
    /// use fp_library::brands::VecBrand;
    ///
    /// assert_eq!(
    ///     lift2::<VecBrand, _, _, _, _>(|x, y| x + y, vec![1, 2], vec![10, 20]),
    ///     vec![11, 21, 12, 22]
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
        fa.iter()
            .flat_map(|a| fb.iter().map(|b| f(a.clone(), b.clone())))
            .collect()
    }
}

impl Pointed for VecBrand {
    /// Wraps a value in a vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::pointed::pure;
    /// use fp_library::brands::VecBrand;
    ///
    /// assert_eq!(pure::<VecBrand, _>(5), vec![5]);
    /// ```
    fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
        vec![a]
    }
}

impl ApplyFirst for VecBrand {}
impl ApplySecond for VecBrand {}

impl Semiapplicative for VecBrand {
    /// Applies wrapped functions to wrapped values (Cartesian product).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semiapplicative::apply;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    /// use fp_library::brands::{VecBrand};
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use std::rc::Rc;
    ///
    /// let funcs = vec![
    ///     <RcFnBrand as ClonableFn>::new(|x: i32| x + 1),
    ///     <RcFnBrand as ClonableFn>::new(|x: i32| x * 2),
    /// ];
    /// assert_eq!(apply::<VecBrand, _, _, RcFnBrand>(funcs, vec![1, 2]), vec![2, 3, 2, 4]);
    /// ```
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply1L1T<'a, Self, A>,
    ) -> Apply1L1T<'a, Self, B> {
        ff.iter()
            .flat_map(|f| fa.iter().map(move |a| f(a.clone())))
            .collect()
    }
}

impl Semimonad for VecBrand {
    /// Chains vector computations (flat_map).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::bind;
    /// use fp_library::brands::VecBrand;
    ///
    /// assert_eq!(
    ///     bind::<VecBrand, _, _, _>(vec![1, 2], |x| vec![x, x * 2]),
    ///     vec![1, 2, 2, 4]
    /// );
    /// ```
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply1L1T<'a, Self, A>,
        f: F,
    ) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> Apply1L1T<'a, Self, B>,
    {
        ma.into_iter().flat_map(f).collect()
    }
}

impl Foldable for VecBrand {
    /// Folds the vector from the right.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_right;
    /// use fp_library::brands::VecBrand;
    ///
    /// assert_eq!(fold_right::<VecBrand, _, _, _>(|x: i32, acc| x + acc, 0, vec![1, 2, 3]), 6);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        fa.into_iter().rev().fold(init, |acc, x| f(x, acc))
    }

    /// Folds the vector from the left.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_left;
    /// use fp_library::brands::VecBrand;
    ///
    /// assert_eq!(fold_left::<VecBrand, _, _, _>(|acc, x: i32| acc + x, 0, vec![1, 2, 3]), 6);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        fa.into_iter().fold(init, f)
    }

    /// Maps the values to a monoid and combines them.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_map;
    /// use fp_library::brands::VecBrand;
    /// use fp_library::v2::types::string; // Import to bring Monoid impl for String into scope
    ///
    /// assert_eq!(
    ///     fold_map::<VecBrand, _, _, _>(|x: i32| x.to_string(), vec![1, 2, 3]),
    ///     "123".to_string()
    /// );
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M,
    {
        fa.into_iter().map(f).fold(M::empty(), |acc, x| M::append(acc, x))
    }
}

impl Traversable for VecBrand {
    /// Traverses the vector with an applicative function.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::traverse;
    /// use fp_library::brands::{OptionBrand, VecBrand};
    ///
    /// assert_eq!(
    ///     traverse::<VecBrand, OptionBrand, _, _, _>(|x| Some(x * 2), vec![1, 2, 3]),
    ///     Some(vec![2, 4, 6])
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
        ta.into_iter().fold(F::pure(Vec::new()), |acc, x| {
            F::lift2(|mut v, b| {
                v.push(b);
                v
            }, acc, f(x))
        })
    }

    /// Sequences a vector of applicative.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::sequence;
    /// use fp_library::brands::{OptionBrand, VecBrand};
    ///
    /// assert_eq!(
    ///     sequence::<VecBrand, OptionBrand, _>(vec![Some(1), Some(2)]),
    ///     Some(vec![1, 2])
    /// );
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>,
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
    where
        Apply1L1T<'a, F, A>: Clone,
        Apply1L1T<'a, Self, A>: Clone,
    {
        ta.into_iter().fold(F::pure(Vec::new()), |acc, x| {
            F::lift2(|mut v, a| {
                v.push(a);
                v
            }, acc, x)
        })
    }
}
