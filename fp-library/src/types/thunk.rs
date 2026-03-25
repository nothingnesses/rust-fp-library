//! Deferred, non-memoized computation with higher-kinded type support.
//!
//! Builds computation chains without stack safety guarantees but supports borrowing and lifetime polymorphism. Does not cache results; if you need the same computation's result more than once, wrap it in [`Lazy`](crate::types::Lazy). For stack-safe alternatives, use [`Trampoline`](crate::types::Trampoline).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::ThunkBrand,
			classes::{
				ApplyFirst,
				ApplySecond,
				CloneableFn,
				Deferrable,
				Evaluable,
				Foldable,
				FoldableWithIndex,
				Functor,
				FunctorWithIndex,
				Lift,
				MonadRec,
				Monoid,
				Pointed,
				Semiapplicative,
				Semigroup,
				Semimonad,
				WithIndex,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcLazyConfig,
				Lazy,
				LazyConfig,
				RcLazyConfig,
				Step,
				Trampoline,
			},
		},
		fp_macros::*,
		std::fmt,
	};

	/// A deferred computation that produces a value of type `A`.
	///
	/// `Thunk` is NOT memoized and does not cache results. Since [`evaluate`](Thunk::evaluate) takes
	/// `self` by value, a `Thunk` can only be evaluated once. If you need the result more than once,
	/// wrap it in [`Lazy`](crate::types::Lazy) via [`into_rc_lazy`](Thunk::into_rc_lazy).
	///
	/// Unlike [`Trampoline`](crate::types::Trampoline), `Thunk` does NOT require `'static` and CAN implement
	/// HKT traits like [`Functor`], [`Semimonad`], etc.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is [`ThunkBrand`](crate::brands::ThunkBrand),
	/// which is fully polymorphic over the result type.
	///
	/// ### Trade-offs vs `Trampoline`
	///
	/// | Aspect         | `Thunk<'a, A>`              | `Trampoline<A>`              |
	/// |----------------|-----------------------------|------------------------------ |
	/// | HKT compatible | ✅ Yes                      | ❌ No (requires `'static`)   |
	/// | Stack-safe     | ⚠️ Partial (tail_rec_m only) | ✅ Yes (unlimited)           |
	/// | Lifetime       | `'a` (can borrow)           | `'static` only               |
	/// | Thread safety  | Not `Send`                  | Not `Send` (`A: 'static`)    |
	/// | Use case       | Glue code, composition      | Deep recursion, pipelines    |
	///
	/// ### Algebraic Properties
	///
	/// `Thunk` is a proper Monad:
	/// - `pure(a).evaluate() == a` (left identity).
	/// - `thunk.bind(pure) == thunk` (right identity).
	/// - `thunk.bind(f).bind(g) == thunk.bind(|a| f(a).bind(g))` (associativity).
	///
	/// ### Stack Safety
	///
	/// `Thunk::bind` chains are **not** stack-safe. Each nested [`bind`](Thunk::bind) adds a
	/// frame to the call stack, so sufficiently deep chains will cause a stack overflow.
	///
	/// For stack-safe recursion within `Thunk`, use [`tail_rec_m`](crate::functions::tail_rec_m), which
	/// uses an internal loop to avoid growing the stack.
	///
	/// For unlimited stack safety on all operations (including `bind` chains of arbitrary
	/// depth), convert to [`Trampoline`](crate::types::Trampoline) instead, which is built
	/// on the [`Free`](crate::types::Free) monad and guarantees O(1) stack usage.
	///
	/// ### Limitations
	///
	/// **Cannot implement `Traversable`**: The [`Traversable`](crate::classes::Traversable) trait
	/// requires `Self::Of<'a, B>: Clone` (i.e., `Thunk<'a, B>: Clone`) in both `traverse` and
	/// `sequence`. `Thunk` wraps `Box<dyn FnOnce() -> A>`, which cannot implement `Clone`
	/// because `FnOnce` closures are consumed on invocation and `Box<dyn FnOnce>` does not
	/// support cloning. Since the trait bounds on `Traversable` are fixed, there is no way
	/// to implement the trait for `Thunk` without changing its internal representation.
	/// This is an intentional trade-off: `Thunk` prioritizes zero-overhead deferred execution
	/// and lifetime flexibility over structural cloning.
	///
	/// Implemented typeclasses:
	/// - ✅ [`Functor`], [`Foldable`], [`Semimonad`]/Monad, [`Semiapplicative`]/Applicative
	/// - ❌ [`Traversable`](crate::classes::Traversable) (requires `Clone`)
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	///
	pub struct Thunk<'a, A>(
		/// The closure that performs the computation.
		Box<dyn FnOnce() -> A + 'a>,
	);

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	#[document_parameters("The thunk instance.")]
	impl<'a, A: 'a> Thunk<'a, A> {
		/// Creates a new `Thunk` from a thunk.
		#[document_signature]
		///
		#[document_parameters("The thunk to wrap.")]
		///
		#[document_returns("A new `Thunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = Thunk::new(|| 42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn new(f: impl FnOnce() -> A + 'a) -> Self {
			Thunk(Box::new(f))
		}

		/// Returns a pure value (already computed).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `Thunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self
		where
			A: 'a, {
			Thunk::new(move || a)
		}

		/// Defers a computation that returns a Thunk.
		#[document_signature]
		///
		#[document_parameters("The thunk that returns a `Thunk`.")]
		///
		#[document_returns("A new `Thunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let thunk = Thunk::defer(|| pure::<ThunkBrand, _>(42));
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn defer(f: impl FnOnce() -> Thunk<'a, A> + 'a) -> Self {
			Thunk::new(move || f().evaluate())
		}

		/// Monadic bind: chains computations.
		///
		/// Note: Each `bind` adds to the call stack. For deep recursion,
		/// use [`Trampoline`](crate::types::Trampoline) instead.
		///
		/// This inherent method accepts [`FnOnce`] for maximum flexibility. The HKT-level
		/// [`Semimonad::bind`](crate::classes::Semimonad::bind) requires [`Fn`] instead,
		/// because some types (such as `Vec`) need to call the function multiple times.
		/// Prefer this inherent method when you do not need HKT generality.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `Thunk` instance representing the chained computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(21).bind(|x| pure::<ThunkBrand, _>(x * 2));
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn bind<B: 'a>(
			self,
			f: impl FnOnce(A) -> Thunk<'a, B> + 'a,
		) -> Thunk<'a, B> {
			Thunk::new(move || {
				let a = (self.0)();
				let thunk_b = f(a);
				(thunk_b.0)()
			})
		}

		/// Functor map: transforms the result.
		///
		/// This inherent method accepts `FnOnce`, which is more permissive than the
		/// HKT [`Functor::map`] free function. The HKT version requires `Fn` because
		/// the trait signature must support containers with multiple elements (e.g., `Vec`).
		/// Since `Thunk` contains exactly one value, `FnOnce` suffices here. Prefer
		/// this method when you do not need HKT polymorphism and want to pass a
		/// non-reusable closure.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the transformation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `Thunk` instance with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(21).map(|x| x * 2);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn map<B: 'a>(
			self,
			f: impl FnOnce(A) -> B + 'a,
		) -> Thunk<'a, B> {
			Thunk::new(move || f((self.0)()))
		}

		/// Forces evaluation and returns the result.
		#[document_signature]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn evaluate(self) -> A {
			(self.0)()
		}

		/// Converts this `Thunk` into a memoized [`Lazy`](crate::types::Lazy) value.
		///
		/// The computation will be evaluated at most once; subsequent accesses
		/// return the cached result.
		#[document_signature]
		///
		#[document_returns("A memoized `Lazy` value that evaluates this thunk on first access.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = Thunk::new(|| 42);
		/// let lazy = thunk.into_rc_lazy();
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		#[inline]
		pub fn into_rc_lazy(self) -> Lazy<'a, A, RcLazyConfig> {
			Lazy::from(self)
		}

		/// Evaluates this `Thunk` and wraps the result in a thread-safe [`ArcLazy`](crate::types::Lazy).
		///
		/// The thunk is evaluated eagerly because its inner closure is not
		/// `Send`. The result is stored in an `ArcLazy` for thread-safe sharing.
		#[document_signature]
		///
		#[document_returns("A thread-safe `ArcLazy` containing the eagerly evaluated result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = Thunk::new(|| 42);
		/// let lazy = thunk.into_arc_lazy();
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		#[inline]
		pub fn into_arc_lazy(self) -> Lazy<'a, A, ArcLazyConfig>
		where
			A: Send + Sync + 'a, {
			let val = self.evaluate();
			Lazy::<'a, A, ArcLazyConfig>::new(move || val)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation.",
		"The memoization configuration."
	)]
	impl<'a, A, Config> From<Lazy<'a, A, Config>> for Thunk<'a, A>
	where
		A: Clone + 'a,
		Config: LazyConfig,
	{
		/// Converts a [`Lazy`] value into a [`Thunk`] by cloning the memoized value.
		///
		/// This conversion clones the cached value on each evaluation.
		/// The cost depends on the [`Clone`] implementation of `A`.
		#[document_signature]
		#[document_parameters("The lazy value to convert.")]
		#[document_returns("A thunk that evaluates the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// let thunk = Thunk::from(lazy);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		fn from(lazy: Lazy<'a, A, Config>) -> Self {
			Thunk::new(move || lazy.evaluate().clone())
		}
	}

	#[document_type_parameters("The type of the value produced by the computation.")]
	impl<A: 'static> From<crate::types::Trampoline<A>> for Thunk<'static, A> {
		#[document_signature]
		#[document_parameters("The trampoline to convert.")]
		#[document_returns("A thunk that evaluates the trampoline.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let task = Trampoline::pure(42);
		/// let thunk = Thunk::from(task);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		fn from(trampoline: crate::types::Trampoline<A>) -> Self {
			Thunk::new(move || trampoline.evaluate())
		}
	}

	#[document_type_parameters("The type of the value produced by the computation.")]
	impl<A: 'static + Send> From<Thunk<'static, A>> for Trampoline<A> {
		/// Converts a `'static` `Thunk` into a `Trampoline`.
		///
		/// This lifts a non-stack-safe `Thunk` into the stack-safe `Trampoline`
		/// execution model. The resulting `Trampoline` evaluates the thunk when run.
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A trampoline that evaluates the thunk.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = Thunk::new(|| 42);
		/// let trampoline = Trampoline::from(thunk);
		/// assert_eq!(trampoline.evaluate(), 42);
		/// ```
		fn from(thunk: Thunk<'static, A>) -> Self {
			Trampoline::new(move || thunk.evaluate())
		}
	}

	impl_kind! {
		for ThunkBrand {
			type Of<'a, A: 'a>: 'a = Thunk<'a, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: 'a> Deferrable<'a> for Thunk<'a, A> {
		/// Creates a `Thunk` from a computation that produces it.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the thunk.")]
		///
		#[document_returns("The deferred thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Deferrable,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let task: Thunk<i32> = Deferrable::defer(|| Thunk::pure(42));
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			Thunk::defer(f)
		}
	}

	impl Functor for ThunkBrand {
		/// Maps a function over the result of a `Thunk` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value inside the `Thunk`.",
			"The type of the result of the transformation."
		)]
		///
		#[document_parameters(
			"The function to apply to the result of the computation.",
			"The `Thunk` instance."
		)]
		///
		#[document_returns("A new `Thunk` instance with the transformed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(10);
		/// let mapped = map::<ThunkBrand, _, _>(|x| x * 2, thunk);
		/// assert_eq!(mapped.evaluate(), 20);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	impl Pointed for ThunkBrand {
		/// Wraps a value in a `Thunk` context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `Thunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let thunk: Thunk<i32> = pure::<ThunkBrand, _>(42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Thunk::pure(a)
		}
	}

	impl Lift for ThunkBrand {
		/// Lifts a binary function into the `Thunk` context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first `Thunk`.",
			"The second `Thunk`."
		)]
		///
		#[document_returns(
			"A new `Thunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let eval1 = pure::<ThunkBrand, _>(10);
		/// let eval2 = pure::<ThunkBrand, _>(20);
		/// let result = lift2::<ThunkBrand, _, _, _>(|a, b| a + b, eval1, eval2);
		/// assert_eq!(result.evaluate(), 30);
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			fa.bind(move |a| fb.map(move |b| func(a, b)))
		}
	}

	impl ApplyFirst for ThunkBrand {}
	impl ApplySecond for ThunkBrand {}

	impl Semiapplicative for ThunkBrand {
		/// Applies a function wrapped in `Thunk` to a value wrapped in `Thunk`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function wrapper.",
			"The type of the input.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The `Thunk` containing the function.",
			"The `Thunk` containing the value."
		)]
		///
		#[document_returns(
			"A new `Thunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let func = pure::<ThunkBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let val = pure::<ThunkBrand, _>(21);
		/// let result = apply::<RcFnBrand, ThunkBrand, _, _>(func, val);
		/// assert_eq!(result.evaluate(), 42);
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ff.bind(move |f| {
				fa.map(
					#[allow(clippy::redundant_closure)] // Required for move semantics
					move |a| f(a),
				)
			})
		}
	}

	impl Semimonad for ThunkBrand {
		/// Chains `Thunk` computations.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the result of the first computation.",
			"The type of the result of the new computation."
		)]
		///
		#[document_parameters(
			"The first `Thunk`.",
			"The function to apply to the result of the computation."
		)]
		///
		#[document_returns("A new `Thunk` instance representing the chained computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(10);
		/// let result = bind::<ThunkBrand, _, _>(thunk, |x| pure::<ThunkBrand, _>(x * 2));
		/// assert_eq!(result.evaluate(), 20);
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	impl MonadRec for ThunkBrand {
		/// Performs tail-recursive monadic computation.
		///
		/// The step function `f` should return shallow thunks (ideally [`Thunk::pure`]
		/// or a single-level [`Thunk::new`]). If `f` builds deep [`bind`](Thunk::bind)
		/// chains inside the returned thunk, the internal [`evaluate`](Thunk::evaluate)
		/// call can still overflow the stack.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = tail_rec_m::<ThunkBrand, _, _>(
		/// 	|x| pure::<ThunkBrand, _>(if x < 1000 { Step::Loop(x + 1) } else { Step::Done(x) }),
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), 1000);
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
			+ Clone
			+ 'a,
			a: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Thunk::new(move || {
				let mut current = a;
				loop {
					match f(current).evaluate() {
						Step::Loop(next) => current = next,
						Step::Done(res) => break res,
					}
				}
			})
		}
	}

	impl Evaluable for ThunkBrand {
		/// Runs the eval, producing the inner value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value inside the thunk."
		)]
		///
		#[document_parameters("The eval to run.")]
		///
		#[document_returns("The result of running the thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let thunk = Thunk::new(|| 42);
		/// assert_eq!(evaluate::<ThunkBrand, _>(thunk), 42);
		/// ```
		fn evaluate<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			fa.evaluate()
		}
	}

	impl Foldable for ThunkBrand {
		/// Folds the `Thunk` from the right.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The `Thunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(10);
		/// let result = fold_right::<RcFnBrand, ThunkBrand, _, _>(|a, b| a + b, 5, thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(fa.evaluate(), initial)
		}

		/// Folds the `Thunk` from the left.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The `Thunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(10);
		/// let result = fold_left::<RcFnBrand, ThunkBrand, _, _>(|b, a| b + a, 5, thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(initial, fa.evaluate())
		}

		/// Maps the value to a monoid and returns it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The Thunk to fold.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let thunk = pure::<ThunkBrand, _>(10);
		/// let result = fold_map::<RcFnBrand, ThunkBrand, _, _>(|a| a.to_string(), thunk);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			func(fa.evaluate())
		}
	}

	impl WithIndex for ThunkBrand {
		type Index = ();
	}

	impl FunctorWithIndex for ThunkBrand {
		/// Maps a function over the value in the thunk, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value inside the thunk.",
			"The type of the result of applying the function."
		)]
		#[document_parameters(
			"The function to apply to the value and its index.",
			"The thunk to map over."
		)]
		#[document_returns("A new thunk containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ThunkBrand,
		/// 	classes::functor_with_index::FunctorWithIndex,
		/// };
		///
		/// let thunk = fp_library::types::Thunk::pure(5);
		/// let result = <ThunkBrand as FunctorWithIndex>::map_with_index(|_, x| x * 2, thunk);
		/// assert_eq!(result.evaluate(), 10);
		/// ```
		fn map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn((), A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(move |a| f((), a))
		}
	}

	impl FoldableWithIndex for ThunkBrand {
		/// Folds the thunk using a monoid, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value inside the thunk.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to the value and its index.",
			"The thunk to fold."
		)]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ThunkBrand,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// };
		///
		/// let thunk = fp_library::types::Thunk::pure(5);
		/// let result =
		/// 	<ThunkBrand as FoldableWithIndex>::fold_map_with_index(|_, x: i32| x.to_string(), thunk);
		/// assert_eq!(result, "5");
		/// ```
		fn fold_map_with_index<'a, A: 'a, R: Monoid>(
			f: impl Fn((), A) -> R + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R {
			f((), fa.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Semigroup + 'a> Semigroup for Thunk<'a, A> {
		/// Combines two `Thunk`s by combining their results.
		#[document_signature]
		///
		#[document_parameters("The first `Thunk`.", "The second `Thunk`.")]
		///
		#[document_returns("A new `Thunk` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let t1 = pure::<ThunkBrand, _>("Hello".to_string());
		/// let t2 = pure::<ThunkBrand, _>(" World".to_string());
		/// let t3 = append::<_>(t1, t2);
		/// assert_eq!(t3.evaluate(), "Hello World");
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Thunk::new(move || Semigroup::append(a.evaluate(), b.evaluate()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Monoid + 'a> Monoid for Thunk<'a, A> {
		/// Returns the identity `Thunk`.
		#[document_signature]
		///
		#[document_returns("A `Thunk` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let t: Thunk<String> = Thunk::empty();
		/// assert_eq!(t.evaluate(), "");
		/// ```
		fn empty() -> Self {
			Thunk::new(|| Monoid::empty())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	#[document_parameters("The thunk to format.")]
	impl<'a, A> fmt::Debug for Thunk<'a, A> {
		/// Formats the thunk without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = Thunk::pure(42);
		/// assert_eq!(format!("{:?}", thunk), "Thunk(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("Thunk(<unevaluated>)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::*,
			classes::{
				monoid::empty,
				semigroup::append,
			},
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	/// Tests basic execution of Thunk.
	///
	/// Verifies that `Thunk::new` creates a computation that can be run to produce the expected value.
	#[test]
	fn test_basic_execution() {
		let thunk = Thunk::new(|| 42);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `Thunk::pure`.
	///
	/// Verifies that `Thunk::pure` creates a computation that returns the provided value.
	#[test]
	fn test_pure() {
		let thunk = Thunk::pure(42);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests borrowing in Thunk.
	///
	/// Verifies that `Thunk` can capture references to values on the stack.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let thunk = Thunk::new(|| &x);
		assert_eq!(thunk.evaluate(), &42);
	}

	/// Tests `Thunk::map`.
	///
	/// Verifies that `map` transforms the result of the computation.
	#[test]
	fn test_map() {
		let thunk = Thunk::pure(21).map(|x| x * 2);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `Thunk::bind`.
	///
	/// Verifies that `bind` chains computations correctly.
	#[test]
	fn test_bind() {
		let thunk = Thunk::pure(21).bind(|x| Thunk::pure(x * 2));
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `Thunk::defer`.
	///
	/// Verifies that `defer` allows creating a `Thunk` from a thunk that returns a `Thunk`.
	#[test]
	fn test_defer() {
		let thunk = Thunk::defer(|| Thunk::pure(42));
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `From<Lazy>`.
	#[test]
	fn test_thunk_from_memo() {
		use crate::types::RcLazy;
		let memo = RcLazy::new(|| 42);
		let thunk = Thunk::from(memo);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests the `Semigroup` implementation for `Thunk`.
	///
	/// Verifies that `append` correctly combines two thunks.
	#[test]
	fn test_thunk_semigroup() {
		use crate::{
			brands::*,
			classes::semigroup::append,
			functions::*,
		};
		let t1 = pure::<ThunkBrand, _>("Hello".to_string());
		let t2 = pure::<ThunkBrand, _>(" World".to_string());
		let t3 = append(t1, t2);
		assert_eq!(t3.evaluate(), "Hello World");
	}

	/// Tests the `Monoid` implementation for `Thunk`.
	///
	/// Verifies that `empty` returns the identity element.
	#[test]
	fn test_thunk_monoid() {
		use crate::classes::monoid::empty;
		let t: Thunk<String> = empty();
		assert_eq!(t.evaluate(), "");
	}

	/// Tests `From<Trampoline>` for `Thunk`.
	///
	/// Verifies that converting a `Trampoline` to a `Thunk` preserves the computed value.
	#[test]
	fn test_thunk_from_trampoline() {
		use crate::types::Trampoline;

		let task = Trampoline::pure(42);
		let thunk = Thunk::from(task);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests roundtrip `Trampoline` -> `Thunk` -> evaluate with a lazy computation.
	///
	/// Verifies that a lazy trampoline is correctly evaluated when converted to a thunk.
	#[test]
	fn test_thunk_from_trampoline_lazy() {
		use crate::types::Trampoline;

		let task = Trampoline::new(|| 21 * 2);
		let thunk = Thunk::from(task);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `From<Thunk<'static, A>> for Trampoline<A>`.
	///
	/// Verifies that a `'static` `Thunk` can be converted to a `Trampoline`.
	#[test]
	fn test_thunk_to_trampoline() {
		use crate::types::Trampoline;
		let thunk = Thunk::new(|| 42);
		let trampoline = Trampoline::from(thunk);
		assert_eq!(trampoline.evaluate(), 42);
	}

	/// Tests `From<Thunk<'static, A>> for Trampoline<A>` with chained computation.
	///
	/// Verifies that conversion preserves the deferred computation.
	#[test]
	fn test_thunk_to_trampoline_chained() {
		use crate::types::Trampoline;
		let thunk = Thunk::pure(10).map(|x| x * 3).bind(|x| Thunk::pure(x + 12));
		let trampoline = Trampoline::from(thunk);
		assert_eq!(trampoline.evaluate(), 42);
	}

	// QuickCheck Law Tests

	// Functor Laws

	/// Functor identity: `map(identity, fa) == fa`.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		map::<ThunkBrand, _, _>(identity, pure::<ThunkBrand, _>(x)).evaluate() == x
	}

	/// Functor composition: `map(f . g, fa) == map(f, map(g, fa))`.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		let f = |a: i32| a.wrapping_add(1);
		let g = |a: i32| a.wrapping_mul(2);
		let lhs = map::<ThunkBrand, _, _>(move |a| f(g(a)), pure::<ThunkBrand, _>(x)).evaluate();
		let rhs = map::<ThunkBrand, _, _>(f, map::<ThunkBrand, _, _>(g, pure::<ThunkBrand, _>(x)))
			.evaluate();
		lhs == rhs
	}

	// Monad Laws

	/// Monad left identity: `pure(a).bind(f) == f(a)`.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| pure::<ThunkBrand, _>(x.wrapping_mul(2));
		let lhs = bind::<ThunkBrand, _, _>(pure::<ThunkBrand, _>(a), f).evaluate();
		let rhs = f(a).evaluate();
		lhs == rhs
	}

	/// Monad right identity: `m.bind(pure) == m`.
	#[quickcheck]
	fn monad_right_identity(x: i32) -> bool {
		let lhs =
			bind::<ThunkBrand, _, _>(pure::<ThunkBrand, _>(x), pure::<ThunkBrand, _>).evaluate();
		lhs == x
	}

	/// Monad associativity: `m.bind(f).bind(g) == m.bind(|a| f(a).bind(g))`.
	#[quickcheck]
	fn monad_associativity(x: i32) -> bool {
		let f = |a: i32| pure::<ThunkBrand, _>(a.wrapping_add(1));
		let g = |a: i32| pure::<ThunkBrand, _>(a.wrapping_mul(3));
		let m = pure::<ThunkBrand, _>(x);
		let m2 = pure::<ThunkBrand, _>(x);
		let lhs = bind::<ThunkBrand, _, _>(bind::<ThunkBrand, _, _>(m, f), g).evaluate();
		let rhs =
			bind::<ThunkBrand, _, _>(m2, move |a| bind::<ThunkBrand, _, _>(f(a), g)).evaluate();
		lhs == rhs
	}

	// Semigroup Laws

	/// Semigroup associativity: `append(append(a, b), c) == append(a, append(b, c))`.
	#[quickcheck]
	fn semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		let ta = pure::<ThunkBrand, _>(a.clone());
		let tb = pure::<ThunkBrand, _>(b.clone());
		let tc = pure::<ThunkBrand, _>(c.clone());
		let ta2 = pure::<ThunkBrand, _>(a);
		let tb2 = pure::<ThunkBrand, _>(b);
		let tc2 = pure::<ThunkBrand, _>(c);
		let lhs = append(append(ta, tb), tc).evaluate();
		let rhs = append(ta2, append(tb2, tc2)).evaluate();
		lhs == rhs
	}

	// Monoid Laws

	/// Monoid left identity: `append(empty(), a) == a`.
	#[quickcheck]
	fn monoid_left_identity(x: String) -> bool {
		let a = pure::<ThunkBrand, _>(x.clone());
		let lhs: Thunk<String> = append(empty(), a);
		lhs.evaluate() == x
	}

	/// Monoid right identity: `append(a, empty()) == a`.
	#[quickcheck]
	fn monoid_right_identity(x: String) -> bool {
		let a = pure::<ThunkBrand, _>(x.clone());
		let rhs: Thunk<String> = append(a, empty());
		rhs.evaluate() == x
	}

	// 7.1: HKT-level trait tests

	/// Tests `Foldable` for `ThunkBrand` via the free function `fold_right`.
	#[test]
	fn test_foldable_via_brand() {
		let thunk = pure::<ThunkBrand, _>(10);
		let result = fold_right::<RcFnBrand, ThunkBrand, _, _>(|x, acc| x + acc, 5, thunk);
		assert_eq!(result, 15);
	}

	/// Tests `Lift::lift2` for `ThunkBrand` via the free function.
	#[test]
	fn test_lift2_via_brand() {
		let t1 = pure::<ThunkBrand, _>(10);
		let t2 = pure::<ThunkBrand, _>(20);
		let result = lift2::<ThunkBrand, _, _, _>(|a, b| a + b, t1, t2);
		assert_eq!(result.evaluate(), 30);
	}

	/// Tests `Semiapplicative::apply` for `ThunkBrand` via the free function.
	#[test]
	fn test_apply_via_brand() {
		let func = pure::<ThunkBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		let val = pure::<ThunkBrand, _>(21);
		let result = apply::<RcFnBrand, ThunkBrand, _, _>(func, val);
		assert_eq!(result.evaluate(), 42);
	}

	/// Tests `Evaluable::evaluate` for `ThunkBrand` via the free function.
	#[test]
	fn test_evaluable_via_brand() {
		let thunk = pure::<ThunkBrand, _>(42);
		let result = evaluate::<ThunkBrand, _>(thunk);
		assert_eq!(result, 42);
	}

	// 7.2: into_rc_lazy and into_arc_lazy tests

	/// Tests that `Thunk::into_rc_lazy` caches the result and does not re-run the closure.
	#[test]
	fn test_memoize_caching() {
		use std::cell::Cell;

		let counter = Cell::new(0usize);
		let thunk = Thunk::new(|| {
			counter.set(counter.get() + 1);
			42
		});
		let lazy = thunk.into_rc_lazy();

		assert_eq!(counter.get(), 0);
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(counter.get(), 1);
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(counter.get(), 1);
	}

	/// Tests that `Thunk::into_arc_lazy` caches the result and does not re-run the closure.
	#[test]
	fn test_memoize_arc_caching() {
		use std::sync::atomic::{
			AtomicUsize,
			Ordering,
		};

		let counter = AtomicUsize::new(0);
		let thunk = Thunk::new(|| {
			counter.fetch_add(1, Ordering::SeqCst);
			42
		});
		let lazy = thunk.into_arc_lazy();

		// into_arc_lazy evaluates eagerly because Thunk is !Send,
		// so the counter should already be 1.
		assert_eq!(counter.load(Ordering::SeqCst), 1);
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	/// Tests `MonadRec::tail_rec_m` stack safety with a large iteration count.
	///
	/// Verifies that `tail_rec_m` does not overflow the stack even with 100,000+ iterations,
	/// because it uses an iterative loop internally rather than recursive calls.
	#[test]
	fn test_tail_rec_m_stack_safety() {
		use crate::{
			brands::ThunkBrand,
			classes::monad_rec::tail_rec_m,
			functions::pure,
			types::Step,
		};

		let iterations: i64 = 200_000;
		let result = tail_rec_m::<ThunkBrand, _, _>(
			|acc| {
				pure::<ThunkBrand, _>(
					if acc < iterations { Step::Loop(acc + 1) } else { Step::Done(acc) },
				)
			},
			0i64,
		);
		assert_eq!(result.evaluate(), iterations);
	}

	/// Tests `FunctorWithIndex` for `ThunkBrand` via the HKT trait method.
	///
	/// Verifies that `map_with_index` provides the unit index `()` and transforms the value.
	#[test]
	fn test_functor_with_index() {
		use crate::{
			brands::ThunkBrand,
			classes::functor_with_index::FunctorWithIndex,
			functions::pure,
		};

		let thunk = pure::<ThunkBrand, _>(21);
		let result = ThunkBrand::map_with_index(|(), x| x * 2, thunk);
		assert_eq!(result.evaluate(), 42);
	}

	/// Tests `FunctorWithIndex` identity law for `ThunkBrand`.
	///
	/// Verifies that `map_with_index(|_, a| a, fa)` is equivalent to `fa`.
	#[test]
	fn test_functor_with_index_identity() {
		use crate::{
			brands::ThunkBrand,
			classes::functor_with_index::FunctorWithIndex,
			functions::pure,
		};

		let thunk = pure::<ThunkBrand, _>(42);
		let result = ThunkBrand::map_with_index(|_, a: i32| a, thunk);
		assert_eq!(result.evaluate(), 42);
	}

	/// Tests `FunctorWithIndex` compatibility with `Functor` for `ThunkBrand`.
	///
	/// Verifies that `map(f, fa) == map_with_index(|_, a| f(a), fa)`.
	#[test]
	fn test_functor_with_index_compat_with_functor() {
		use crate::{
			brands::ThunkBrand,
			classes::functor_with_index::FunctorWithIndex,
			functions::{
				map,
				pure,
			},
		};

		let f = |a: i32| a * 3 + 1;
		let thunk1 = pure::<ThunkBrand, _>(10);
		let thunk2 = pure::<ThunkBrand, _>(10);
		let via_map = map::<ThunkBrand, _, _>(f, thunk1).evaluate();
		let via_map_with_index = ThunkBrand::map_with_index(|_, a| f(a), thunk2).evaluate();
		assert_eq!(via_map, via_map_with_index);
	}

	/// Tests `FoldableWithIndex` for `ThunkBrand` via the HKT trait method.
	///
	/// Verifies that `fold_map_with_index` provides the unit index `()` and folds the value.
	#[test]
	fn test_foldable_with_index() {
		use crate::{
			brands::ThunkBrand,
			classes::foldable_with_index::FoldableWithIndex,
			functions::pure,
		};

		let thunk = pure::<ThunkBrand, _>(42);
		let result: String = ThunkBrand::fold_map_with_index(|(), a: i32| a.to_string(), thunk);
		assert_eq!(result, "42");
	}

	/// Tests `FoldableWithIndex` compatibility with `Foldable` for `ThunkBrand`.
	///
	/// Verifies that `fold_map(f, fa) == fold_map_with_index(|_, a| f(a), fa)`.
	#[test]
	fn test_foldable_with_index_compat_with_foldable() {
		use crate::{
			brands::*,
			classes::foldable_with_index::FoldableWithIndex,
			functions::{
				fold_map,
				pure,
			},
		};

		let f = |a: i32| a.to_string();
		let thunk1 = pure::<ThunkBrand, _>(99);
		let thunk2 = pure::<ThunkBrand, _>(99);
		let via_fold_map = fold_map::<RcFnBrand, ThunkBrand, _, _>(f, thunk1);
		let via_fold_map_with_index: String = ThunkBrand::fold_map_with_index(|_, a| f(a), thunk2);
		assert_eq!(via_fold_map, via_fold_map_with_index);
	}
}
