use crate::{
    brands::OptionBrand,
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

impl Kind1L1T for OptionBrand {
    type Output<'a, A: 'a> = Option<A>;
}

impl Functor for OptionBrand {
    /// Maps a function over the value in the option.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Functor Option => (a -> b, Option a) -> Option b`
    ///
    /// # Parameters
    ///
    /// * `f`: The function to apply to the value.
    /// * `fa`: The option to map over.
    ///
    /// # Returns
    ///
    /// A new option containing the result of applying the function, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::functor::map;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5)), Some(10));
    /// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, None), None);
    /// ```
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> B,
    {
        fa.map(f)
    }
}

impl Lift for OptionBrand {
    /// Lifts a binary function into the option context.
    ///
    /// # Type Signature
    ///
    /// `forall a b c. Lift Option => ((a, b) -> c, Option a, Option b) -> Option c`
    ///
    /// # Parameters
    ///
    /// * `f`: The binary function to apply.
    /// * `fa`: The first option.
    /// * `fb`: The second option.
    ///
    /// # Returns
    ///
    /// `Some(f(a, b))` if both options are `Some`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::lift2;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), Some(2)), Some(3));
    /// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), None), None);
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
        fa.zip(fb).map(|(a, b)| f(a, b))
    }
}

impl Pointed for OptionBrand {
    /// Wraps a value in an option.
    ///
    /// # Type Signature
    ///
    /// `forall a. Pointed Option => a -> Option a`
    ///
    /// # Parameters
    ///
    /// * `a`: The value to wrap.
    ///
    /// # Returns
    ///
    /// `Some(a)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::pointed::pure;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(pure::<OptionBrand, _>(5), Some(5));
    /// ```
    fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
        Some(a)
    }
}

impl ApplyFirst for OptionBrand {}
impl ApplySecond for OptionBrand {}

impl Semiapplicative for OptionBrand {
    /// Applies a wrapped function to a wrapped value.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Semiapplicative Option => (Option (a -> b), Option a) -> Option b`
    ///
    /// # Parameters
    ///
    /// * `ff`: The option containing the function.
    /// * `fa`: The option containing the value.
    ///
    /// # Returns
    ///
    /// `Some(f(a))` if both are `Some`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semiapplicative::apply;
    /// use fp_library::v2::classes::clonable_fn::ClonableFn;
    /// use fp_library::brands::{OptionBrand};
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    /// use std::rc::Rc;
    ///
    /// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
    /// assert_eq!(apply::<OptionBrand, _, _, RcFnBrand>(f, Some(5)), Some(10));
    /// ```
    fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
        ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
        fa: Apply1L1T<'a, Self, A>,
    ) -> Apply1L1T<'a, Self, B> {
        match (ff, fa) {
            (Some(f), Some(a)) => Some(f(a)),
            _ => None,
        }
    }
}

impl Semimonad for OptionBrand {
    /// Chains option computations.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Semimonad Option => (Option a, a -> Option b) -> Option b`
    ///
    /// # Parameters
    ///
    /// * `ma`: The first option.
    /// * `f`: The function to apply to the value inside the option.
    ///
    /// # Returns
    ///
    /// The result of applying `f` to the value if `ma` is `Some`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::bind;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(bind::<OptionBrand, _, _, _>(Some(5), |x| Some(x * 2)), Some(10));
    /// assert_eq!(bind::<OptionBrand, _, _, _>(None, |x: i32| Some(x * 2)), None);
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

impl Foldable for OptionBrand {
    /// Folds the option from the right.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Foldable Option => ((a, b) -> b, b, Option a) -> b`
    ///
    /// # Parameters
    ///
    /// * `f`: The folding function.
    /// * `init`: The initial value.
    /// * `fa`: The option to fold.
    ///
    /// # Returns
    ///
    /// `f(a, init)` if `fa` is `Some(a)`, otherwise `init`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_right;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(fold_right::<OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, Some(5)), 5);
    /// assert_eq!(fold_right::<OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, None), 0);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(A, B) -> B,
    {
        match fa {
            Some(a) => f(a, init),
            None => init,
        }
    }

    /// Folds the option from the left.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Foldable Option => ((b, a) -> b, b, Option a) -> b`
    ///
    /// # Parameters
    ///
    /// * `f`: The folding function.
    /// * `init`: The initial value.
    /// * `fa`: The option to fold.
    ///
    /// # Returns
    ///
    /// `f(init, a)` if `fa` is `Some(a)`, otherwise `init`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_left;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(fold_left::<OptionBrand, _, _, _>(|acc, x: i32| acc + x, 0, Some(5)), 5);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(B, A) -> B,
    {
        match fa {
            Some(a) => f(init, a),
            None => init,
        }
    }

    /// Maps the value to a monoid and returns it, or returns empty.
    ///
    /// # Type Signature
    ///
    /// `forall a m. (Foldable Option, Monoid m) => ((a) -> m, Option a) -> m`
    ///
    /// # Parameters
    ///
    /// * `f`: The mapping function.
    /// * `fa`: The option to fold.
    ///
    /// # Returns
    ///
    /// `f(a)` if `fa` is `Some(a)`, otherwise `M::empty()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::fold_map;
    /// use fp_library::brands::OptionBrand;
    /// use fp_library::v2::types::string; // Import to bring Monoid impl for String into scope
    ///
    /// assert_eq!(fold_map::<OptionBrand, _, _, _>(|x: i32| x.to_string(), Some(5)), "5".to_string());
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> M
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
    /// Traverses the option with an applicative function.
    ///
    /// # Type Signature
    ///
    /// `forall a b f. (Traversable Option, Applicative f) => (a -> f b, Option a) -> f (Option b)`
    ///
    /// # Parameters
    ///
    /// * `f`: The function to apply.
    /// * `ta`: The option to traverse.
    ///
    /// # Returns
    ///
    /// The option wrapped in the applicative context.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::traverse;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(traverse::<OptionBrand, OptionBrand, _, _, _>(|x| Some(x * 2), Some(5)), Some(Some(10)));
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
            Some(a) => F::map(|b| Some(b), f(a)),
            None => F::pure(None),
        }
    }

    /// Sequences an option of applicative.
    ///
    /// # Type Signature
    ///
    /// `forall a f. (Traversable Option, Applicative f) => (Option (f a)) -> f (Option a)`
    ///
    /// # Parameters
    ///
    /// * `ta`: The option containing the applicative value.
    ///
    /// # Returns
    ///
    /// The option wrapped in the applicative context.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::traversable::sequence;
    /// use fp_library::brands::OptionBrand;
    ///
    /// assert_eq!(sequence::<OptionBrand, OptionBrand, _>(Some(Some(5))), Some(Some(5)));
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>,
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
    where
        Apply1L1T<'a, F, A>: Clone,
        Apply1L1T<'a, Self, A>: Clone,
    {
        match ta {
            Some(fa) => F::map(|a| Some(a), fa),
            None => F::pure(None),
        }
    }
}
