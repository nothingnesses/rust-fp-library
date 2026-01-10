use crate::{
    brands::{ResultWithErrBrand, ResultWithOkBrand},
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

use crate::hkt::Kind0L2T;

pub struct ResultBrand;

impl Kind0L2T for ResultBrand {
    type Output<A, B> = Result<B, A>;
}

// ResultWithErrBrand<E> (Functor over T)

impl<E: 'static> Kind1L1T for ResultWithErrBrand<E> {
    type Output<'a, A: 'a> = Result<A, E>;
}

impl<E: 'static> Functor for ResultWithErrBrand<E> {
    /// Maps a function over the value in the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::functor::map;
    /// use fp_library::brands::ResultWithErrBrand;
    ///
    /// assert_eq!(map::<ResultWithErrBrand<()>, _, _, _>(|x: i32| x * 2, Ok(5)), Ok(10));
    /// assert_eq!(map::<ResultWithErrBrand<i32>, _, _, _>(|x: i32| x * 2, Err(1)), Err(1));
    /// ```
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> B,
    {
        fa.map(f)
    }
}

impl<E: Clone + 'static> Lift for ResultWithErrBrand<E> {
    /// Lifts a binary function into the result context.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::lift2;
    /// use fp_library::brands::ResultWithErrBrand;
    ///
    /// assert_eq!(
    ///     lift2::<ResultWithErrBrand<()>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
    ///     Ok(3)
    /// );
    /// assert_eq!(
    ///     lift2::<ResultWithErrBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
    ///     Err(2)
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
        match (fa, fb) {
            (Ok(a), Ok(b)) => Ok(f(a, b)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl<E: 'static> Pointed for ResultWithErrBrand<E> {
    /// Wraps a value in a result.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::pointed::pure;
    /// use fp_library::brands::ResultWithErrBrand;
    ///
    /// assert_eq!(pure::<ResultWithErrBrand<()>, _>(5), Ok(5));
    /// ```
    fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
        Ok(a)
    }
}

impl<E: Clone + 'static> ApplyFirst for ResultWithErrBrand<E> {}
impl<E: Clone + 'static> ApplySecond for ResultWithErrBrand<E> {}

impl<E: Clone + 'static> Semiapplicative for ResultWithErrBrand<E> {
    /// Applies a wrapped function to a wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semiapplicative::apply;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    /// use fp_library::brands::ResultWithErrBrand;
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use std::rc::Rc;
    ///
    /// let f = Ok(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
    /// assert_eq!(apply::<ResultWithErrBrand<()>, _, _, RcFnBrand>(f, Ok(5)), Ok(10));
    /// ```
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply1L1T<'a, Self, A>,
    ) -> Apply1L1T<'a, Self, B> {
        match (ff, fa) {
            (Ok(f), Ok(a)) => Ok(f(a)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl<E: Clone + 'static> Semimonad for ResultWithErrBrand<E> {
    /// Chains result computations.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::bind;
    /// use fp_library::brands::ResultWithErrBrand;
    ///
    /// assert_eq!(
    ///     bind::<ResultWithErrBrand<()>, _, _, _>(Ok(5), |x| Ok(x * 2)),
    ///     Ok(10)
    /// );
    /// ```
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply1L1T<'a, Self, A>,
        f: F,
    ) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> Apply1L1T<'a, Self, B>,
    {
        ma.and_then(f)
    }
}

impl<E: 'static> Foldable for ResultWithErrBrand<E> {
    /// Folds the result from the right.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_right;
    /// use fp_library::brands::ResultWithErrBrand;
    ///
    /// assert_eq!(fold_right::<ResultWithErrBrand<()>, _, _, _>(|x, acc| x + acc, 0, Ok(5)), 5);
    /// assert_eq!(fold_right::<ResultWithErrBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)), 0);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        match fa {
            Ok(a) => f(a, init),
            Err(_) => init,
        }
    }

    /// Folds the result from the left.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_left;
    /// use fp_library::brands::ResultWithErrBrand;
    ///
    /// assert_eq!(fold_left::<ResultWithErrBrand<()>, _, _, _>(|acc, x| acc + x, 0, Ok(5)), 5);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        match fa {
            Ok(a) => f(init, a),
            Err(_) => init,
        }
    }

    /// Maps the value to a monoid and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_map;
    /// use fp_library::brands::ResultWithErrBrand;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     fold_map::<ResultWithErrBrand<()>, _, _, _>(|x: i32| x.to_string(), Ok(5)),
    ///     "5".to_string()
    /// );
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M,
    {
        match fa {
            Ok(a) => f(a),
            Err(_) => M::empty(),
        }
    }
}

impl<E: Clone + 'static> Traversable for ResultWithErrBrand<E> {
    /// Traverses the result with an applicative function.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::traverse;
    /// use fp_library::brands::{ResultWithErrBrand, OptionBrand};
    ///
    /// assert_eq!(
    ///     traverse::<ResultWithErrBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Ok(5)),
    ///     Some(Ok(10))
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
        match ta {
            Ok(a) => F::map(|b| Ok(b), f(a)),
            Err(e) => F::pure(Err(e)),
        }
    }

    /// Sequences a result of applicative.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::sequence;
    /// use fp_library::brands::{ResultWithErrBrand, OptionBrand};
    ///
    /// assert_eq!(
    ///     sequence::<ResultWithErrBrand<()>, OptionBrand, _>(Ok(Some(5))),
    ///     Some(Ok(5))
    /// );
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>,
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
    where
        Apply1L1T<'a, F, A>: Clone,
        Apply1L1T<'a, Self, A>: Clone,
    {
        match ta {
            Ok(fa) => F::map(|a| Ok(a), fa),
            Err(e) => F::pure(Err(e)),
        }
    }
}

// ResultWithOkBrand<T> (Functor over E)

impl<T: 'static> Kind1L1T for ResultWithOkBrand<T> {
    type Output<'a, A: 'a> = Result<T, A>;
}

impl<T: 'static> Functor for ResultWithOkBrand<T> {
    /// Maps a function over the error value in the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::functor::map;
    /// use fp_library::brands::ResultWithOkBrand;
    ///
    /// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Err(5)), Err(10));
    /// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Ok(1)), Ok(1));
    /// ```
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> B,
    {
        match fa {
            Ok(t) => Ok(t),
            Err(e) => Err(f(e)),
        }
    }
}

impl<T: Clone + 'static> Lift for ResultWithOkBrand<T> {
    /// Lifts a binary function into the result context (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::lift2;
    /// use fp_library::brands::ResultWithOkBrand;
    ///
    /// assert_eq!(
    ///     lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
    ///     Err(3)
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
        match (fa, fb) {
            (Err(a), Err(b)) => Err(f(a, b)),
            (Ok(t), _) => Ok(t),
            (_, Ok(t)) => Ok(t),
        }
    }
}

impl<T: 'static> Pointed for ResultWithOkBrand<T> {
    /// Wraps a value in a result (as error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::pointed::pure;
    /// use fp_library::brands::ResultWithOkBrand;
    ///
    /// assert_eq!(pure::<ResultWithOkBrand<()>, _>(5), Err(5));
    /// ```
    fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
        Err(a)
    }
}

impl<T: Clone + 'static> ApplyFirst for ResultWithOkBrand<T> {}
impl<T: Clone + 'static> ApplySecond for ResultWithOkBrand<T> {}

impl<T: Clone + 'static> Semiapplicative for ResultWithOkBrand<T> {
    /// Applies a wrapped function to a wrapped value (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semiapplicative::apply;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    /// use fp_library::brands::ResultWithOkBrand;
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use std::rc::Rc;
    ///
    /// let f = Err(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
    /// assert_eq!(apply::<ResultWithOkBrand<()>, _, _, RcFnBrand>(f, Err(5)), Err(10));
    /// ```
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply1L1T<'a, Self, A>,
    ) -> Apply1L1T<'a, Self, B> {
        match (ff, fa) {
            (Err(f), Err(a)) => Err(f(a)),
            (Ok(t), _) => Ok(t),
            (_, Ok(t)) => Ok(t),
        }
    }
}

impl<T: Clone + 'static> Semimonad for ResultWithOkBrand<T> {
    /// Chains result computations (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::bind;
    /// use fp_library::brands::ResultWithOkBrand;
    ///
    /// assert_eq!(
    ///     bind::<ResultWithOkBrand<()>, _, _, _>(Err(5), |x| Err(x * 2)),
    ///     Err(10)
    /// );
    /// ```
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply1L1T<'a, Self, A>,
        f: F,
    ) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> Apply1L1T<'a, Self, B>,
    {
        match ma {
            Ok(t) => Ok(t),
            Err(e) => f(e),
        }
    }
}

impl<T: 'static> Foldable for ResultWithOkBrand<T> {
    /// Folds the result from the right (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_right;
    /// use fp_library::brands::ResultWithOkBrand;
    ///
    /// assert_eq!(fold_right::<ResultWithOkBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)), 1);
    /// assert_eq!(fold_right::<ResultWithOkBrand<()>, _, _, _>(|x: i32, acc| x + acc, 0, Ok(())), 0);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        match fa {
            Err(e) => f(e, init),
            Ok(_) => init,
        }
    }

    /// Folds the result from the left (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_left;
    /// use fp_library::brands::ResultWithOkBrand;
    ///
    /// assert_eq!(fold_left::<ResultWithOkBrand<()>, _, _, _>(|acc, x| acc + x, 0, Err(5)), 5);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        match fa {
            Err(e) => f(init, e),
            Ok(_) => init,
        }
    }

    /// Maps the value to a monoid and returns it (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_map;
    /// use fp_library::brands::ResultWithOkBrand;
    /// use fp_library::v2::types::string;
    ///
    /// assert_eq!(
    ///     fold_map::<ResultWithOkBrand<()>, _, _, _>(|x: i32| x.to_string(), Err(5)),
    ///     "5".to_string()
    /// );
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M,
    {
        match fa {
            Err(e) => f(e),
            Ok(_) => M::empty(),
        }
    }
}

impl<T: Clone + 'static> Traversable for ResultWithOkBrand<T> {
    /// Traverses the result with an applicative function (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::traverse;
    /// use fp_library::brands::{ResultWithOkBrand, OptionBrand};
    ///
    /// assert_eq!(
    ///     traverse::<ResultWithOkBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Err(5)),
    ///     Some(Err(10))
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
        match ta {
            Err(e) => F::map(|b| Err(b), f(e)),
            Ok(t) => F::pure(Ok(t)),
        }
    }

    /// Sequences a result of applicative (over error).
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::sequence;
    /// use fp_library::brands::{ResultWithOkBrand, OptionBrand};
    ///
    /// assert_eq!(
    ///     sequence::<ResultWithOkBrand<()>, OptionBrand, _>(Err(Some(5))),
    ///     Some(Err(5))
    /// );
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>,
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
    where
        Apply1L1T<'a, F, A>: Clone,
        Apply1L1T<'a, Self, A>: Clone,
    {
        match ta {
            Err(fe) => F::map(|e| Err(e), fe),
            Ok(t) => F::pure(Ok(t)),
        }
    }
}
