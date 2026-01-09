use crate::{
    brands::{PairWithFirstBrand, PairWithSecondBrand},
    hkt::Apply0L1T,
    types::Pair,
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
        semigroup::Semigroup,
        traversable::Traversable,
    },
};

// PairWithFirstBrand<First> (Functor over Second)

impl<First> Functor for PairWithFirstBrand<First> {
    /// Maps a function over the second value in the pair.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::functor::map;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(map::<PairWithFirstBrand<_>, _, _, _>(|x: i32| x * 2, Pair(1, 5)), Pair(1, 10));
    /// ```
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> B,
    {
        Pair(fa.0, f(fa.1))
    }
}

impl<First: Clone> Lift for PairWithFirstBrand<First>
where
    First: Semigroup,
{
    /// Lifts a binary function into the pair context (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::lift2;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     lift2::<PairWithFirstBrand<String>, _, _, _, _>(|x, y| x + y, Pair("a".to_string(), 1), Pair("b".to_string(), 2)),
    ///     Pair("ab".to_string(), 3)
    /// );
    /// ```
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
        Pair(Semigroup::append(fa.0, fb.0), f(fa.1, fb.1))
    }
}

impl<First: Clone> Pointed for PairWithFirstBrand<First>
where
    First: Monoid,
{
    /// Wraps a value in a pair (with empty first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::pointed::pure;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(pure::<PairWithFirstBrand<String>, _>(5), Pair("".to_string(), 5));
    /// ```
    fn pure<A>(a: A) -> Apply0L1T<Self, A> {
        Pair(Monoid::empty(), a)
    }
}

impl<First: Clone + Semigroup> ApplyFirst for PairWithFirstBrand<First> {}
impl<First: Clone + Semigroup> ApplySecond for PairWithFirstBrand<First> {}

impl<First: Clone> Semiapplicative for PairWithFirstBrand<First>
where
    First: Semigroup,
{
    /// Applies a wrapped function to a wrapped value (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semiapplicative::apply;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use fp_library::v2::types::string;
    /// use std::rc::Rc;
    ///
    /// let f = Pair("a".to_string(), <RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
    /// assert_eq!(apply::<PairWithFirstBrand<String>, _, _, RcFnBrand>(f, Pair("b".to_string(), 5)), Pair("ab".to_string(), 10));
    /// ```
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply0L1T<Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply0L1T<Self, A>,
    ) -> Apply0L1T<Self, B> {
        Pair(Semigroup::append(ff.0, fa.0), ff.1(fa.1))
    }
}

impl<First: Clone> Semimonad for PairWithFirstBrand<First>
where
    First: Semigroup,
{
    /// Chains pair computations (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::bind;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     bind::<PairWithFirstBrand<String>, _, _, _>(Pair("a".to_string(), 5), |x| Pair("b".to_string(), x * 2)),
    ///     Pair("ab".to_string(), 10)
    /// );
    /// ```
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply0L1T<Self, A>,
        f: F,
    ) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> Apply0L1T<Self, B>,
    {
        let Pair(first, second) = ma;
        let Pair(next_first, next_second) = f(second);
        Pair(Semigroup::append(first, next_first), next_second)
    }
}

impl<First> Foldable for PairWithFirstBrand<First> {
    /// Folds the pair from the right (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_right;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(fold_right::<PairWithFirstBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair((), 5)), 5);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        f(fa.1, init)
    }

    /// Folds the pair from the left (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_left;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(fold_left::<PairWithFirstBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair((), 5)), 5);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        f(init, fa.1)
    }

    /// Maps the value to a monoid and returns it (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_map;
    /// use fp_library::brands::PairWithFirstBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     fold_map::<PairWithFirstBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair((), 5)),
    ///     "5".to_string()
    /// );
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M,
    {
        f(fa.1)
    }
}

impl<First: Clone> Traversable for PairWithFirstBrand<First> {
    /// Traverses the pair with an applicative function (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::traverse;
    /// use fp_library::brands::{PairWithFirstBrand, OptionBrand};
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(
    ///     traverse::<PairWithFirstBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Pair((), 5)),
    ///     Some(Pair((), 10))
    /// );
    /// ```
    fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
        f: Func,
        ta: Apply0L1T<Self, A>,
    ) -> Apply0L1T<F, Apply0L1T<Self, B>>
    where
        Func: Fn(A) -> Apply0L1T<F, B>,
        Apply0L1T<Self, B>: Clone,
    {
        let Pair(first, second) = ta;
        F::map(move |b| Pair(first.clone(), b), f(second))
    }

    /// Sequences a pair of applicative (over second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::sequence;
    /// use fp_library::brands::{PairWithFirstBrand, OptionBrand};
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(
    ///     sequence::<PairWithFirstBrand<()>, OptionBrand, _>(Pair((), Some(5))),
    ///     Some(Pair((), 5))
    /// );
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply0L1T<Self, Apply0L1T<F, A>>,
    ) -> Apply0L1T<F, Apply0L1T<Self, A>>
    where
        Apply0L1T<F, A>: Clone,
        Apply0L1T<Self, A>: Clone,
    {
        let Pair(first, second) = ta;
        F::map(move |a| Pair(first.clone(), a), second)
    }
}

// PairWithSecondBrand<Second> (Functor over First)

impl<Second> Functor for PairWithSecondBrand<Second> {
    /// Maps a function over the first value in the pair.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::functor::map;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(map::<PairWithSecondBrand<_>, _, _, _>(|x: i32| x * 2, Pair(5, 1)), Pair(10, 1));
    /// ```
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> B,
    {
        Pair(f(fa.0), fa.1)
    }
}

impl<Second: Clone> Lift for PairWithSecondBrand<Second>
where
    Second: Semigroup,
{
    /// Lifts a binary function into the pair context (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::lift2;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     lift2::<PairWithSecondBrand<String>, _, _, _, _>(|x, y| x + y, Pair(1, "a".to_string()), Pair(2, "b".to_string())),
    ///     Pair(3, "ab".to_string())
    /// );
    /// ```
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
        Pair(f(fa.0, fb.0), Semigroup::append(fa.1, fb.1))
    }
}

impl<Second: Clone> Pointed for PairWithSecondBrand<Second>
where
    Second: Monoid,
{
    /// Wraps a value in a pair (with empty second).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::pointed::pure;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(pure::<PairWithSecondBrand<String>, _>(5), Pair(5, "".to_string()));
    /// ```
    fn pure<A>(a: A) -> Apply0L1T<Self, A> {
        Pair(a, Monoid::empty())
    }
}

impl<Second: Clone + Semigroup> ApplyFirst for PairWithSecondBrand<Second> {}
impl<Second: Clone + Semigroup> ApplySecond for PairWithSecondBrand<Second> {}

impl<Second: Clone> Semiapplicative for PairWithSecondBrand<Second>
where
    Second: Semigroup,
{
    /// Applies a wrapped function to a wrapped value (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semiapplicative::apply;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use fp_library::v2::types::string;
    /// use std::rc::Rc;
    ///
    /// let f = Pair(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2), "a".to_string());
    /// assert_eq!(apply::<PairWithSecondBrand<String>, _, _, RcFnBrand>(f, Pair(5, "b".to_string())), Pair(10, "ab".to_string()));
    /// ```
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply0L1T<Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply0L1T<Self, A>,
    ) -> Apply0L1T<Self, B> {
        Pair(ff.0(fa.0), Semigroup::append(ff.1, fa.1))
    }
}

impl<Second: Clone> Semimonad for PairWithSecondBrand<Second>
where
    Second: Semigroup,
{
    /// Chains pair computations (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::bind;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     bind::<PairWithSecondBrand<String>, _, _, _>(Pair(5, "a".to_string()), |x| Pair(x * 2, "b".to_string())),
    ///     Pair(10, "ab".to_string())
    /// );
    /// ```
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply0L1T<Self, A>,
        f: F,
    ) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> Apply0L1T<Self, B>,
    {
        let Pair(first, second) = ma;
        let Pair(next_first, next_second) = f(first);
        Pair(next_first, Semigroup::append(second, next_second))
    }
}

impl<Second> Foldable for PairWithSecondBrand<Second> {
    /// Folds the pair from the right (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_right;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(fold_right::<PairWithSecondBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair(5, ())), 5);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        f(fa.0, init)
    }

    /// Folds the pair from the left (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_left;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(fold_left::<PairWithSecondBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair(5, ())), 5);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        f(init, fa.0)
    }

    /// Maps the value to a monoid and returns it (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_map;
    /// use fp_library::brands::PairWithSecondBrand;
    /// use fp_library::types::Pair;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     fold_map::<PairWithSecondBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair(5, ())),
    ///     "5".to_string()
    /// );
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M,
    {
        f(fa.0)
    }
}

impl<Second: Clone> Traversable for PairWithSecondBrand<Second> {
    /// Traverses the pair with an applicative function (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::traverse;
    /// use fp_library::brands::{PairWithSecondBrand, OptionBrand};
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(
    ///     traverse::<PairWithSecondBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Pair(5, ())),
    ///     Some(Pair(10, ()))
    /// );
    /// ```
    fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
        f: Func,
        ta: Apply0L1T<Self, A>,
    ) -> Apply0L1T<F, Apply0L1T<Self, B>>
    where
        Func: Fn(A) -> Apply0L1T<F, B>,
        Apply0L1T<Self, B>: Clone,
    {
        let Pair(first, second) = ta;
        F::map(move |b| Pair(b, second.clone()), f(first))
    }

    /// Sequences a pair of applicative (over first).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::sequence;
    /// use fp_library::brands::{PairWithSecondBrand, OptionBrand};
    /// use fp_library::types::Pair;
    ///
    /// assert_eq!(
    ///     sequence::<PairWithSecondBrand<()>, OptionBrand, _>(Pair(Some(5), ())),
    ///     Some(Pair(5, ()))
    /// );
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply0L1T<Self, Apply0L1T<F, A>>,
    ) -> Apply0L1T<F, Apply0L1T<Self, A>>
    where
        Apply0L1T<F, A>: Clone,
        Apply0L1T<Self, A>: Clone,
    {
        let Pair(first, second) = ta;
        F::map(move |a| Pair(a, second.clone()), first)
    }
}
