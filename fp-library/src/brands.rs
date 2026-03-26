//! Brands represent higher-kinded (unapplied/partially-applied) forms of
//! [types][crate::types], as opposed to concrete types, which are
//! fully-applied.
//!
//! For example, [`VecBrand`] represents the higher-kinded type [`Vec`], whereas
//! `Vec A`/`Vec<A>` is the concrete type where `Vec` has been applied to some
//! generic type `A`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = map::<OptionBrand, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

use {
	crate::{
		classes::RefCountedPointer,
		types::{
			ArcLazyConfig,
			LazyConfig,
			RcLazyConfig,
			TryLazyConfig,
		},
	},
	std::marker::PhantomData,
};

pub mod optics;

/// Brand for [`Arc`](std::sync::Arc) atomic reference-counted pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcBrand;

/// Brand for [atomically reference-counted][std::sync::Arc]
/// [closures][Fn] (`Arc<dyn Fn(A) -> B>`).
///
/// This type alias provides a way to construct and type-check [`Arc`](std::sync::Arc)-wrapped
/// closures in a generic context.
pub type ArcFnBrand = FnBrand<ArcBrand>;

/// An adapter that partially applies a `Bifunctor` to its first argument, creating a `Functor` over the second argument.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::functor::map,
/// };
///
/// let x = Result::<i32, i32>::Ok(5);
/// let y = map::<BifunctorFirstAppliedBrand<ResultBrand, i32>, _, _>(|s| s * 2, x);
/// assert_eq!(y, Ok(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BifunctorFirstAppliedBrand<Brand, A>(PhantomData<(Brand, A)>);

/// An adapter that partially applies a `Bifunctor` to its second argument, creating a `Functor` over the first argument.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::functor::map,
/// };
///
/// let x = Result::<i32, i32>::Err(5);
/// let y = map::<BifunctorSecondAppliedBrand<ResultBrand, i32>, _, _>(|e| e * 2, x);
/// assert_eq!(y, Err(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BifunctorSecondAppliedBrand<Brand, B>(PhantomData<(Brand, B)>);

/// Brand for [`CatList`](crate::types::CatList).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatListBrand;

/// Brand for the [`Const`](crate::types::const_val::Const) functor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstBrand<R>(PhantomData<R>);

/// Generic function brand parameterized by reference-counted pointer choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FnBrand<PtrBrand: RefCountedPointer>(PhantomData<PtrBrand>);

/// Brand for [`Identity`](crate::types::Identity).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityBrand;

/// Brand for [`Lazy`](crate::types::Lazy).
///
/// # Type Parameters
///
/// - `Config`: The memoization strategy, implementing [`LazyConfig`]. Use
///   [`RcLazyConfig`] for single-threaded contexts
///   or [`ArcLazyConfig`] for thread-safe contexts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LazyBrand<Config: LazyConfig>(PhantomData<Config>);

/// Brand for single-threaded [`RcLazy`](crate::types::RcLazy).
pub type RcLazyBrand = LazyBrand<RcLazyConfig>;

/// Brand for thread-safe [`ArcLazy`](crate::types::ArcLazy).
pub type ArcLazyBrand = LazyBrand<ArcLazyConfig>;

/// Brand for [`Option`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionBrand;

/// Brand for [`Pair`](crate::types::Pair).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairBrand;

/// Brand for the partially-applied form of [`Pair`](crate::types::Pair) with the first value applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairFirstAppliedBrand<First>(PhantomData<First>);

/// Brand for the partially-applied form of [`Pair`](crate::types::Pair) with the second value applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairSecondAppliedBrand<Second>(PhantomData<Second>);

/// An adapter that partially applies a `Profunctor` to its first argument, creating a `Functor`.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::functor::map,
/// };
///
/// let f = |x: i32| x + 1;
/// let g = map::<ProfunctorFirstAppliedBrand<RcFnBrand, i32>, _, _>(
/// 	|y: i32| y * 2,
/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
/// );
/// assert_eq!(g(10), 22); // (10 + 1) * 2 = 22
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfunctorFirstAppliedBrand<Brand, A>(PhantomData<(Brand, A)>);

/// An adapter that partially applies a `Profunctor` to its second argument, creating a `Contravariant` functor.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::contravariant::contramap,
/// };
///
/// let f = |x: i32| x > 5;
/// let is_long_int = contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, bool>, _, _>(
/// 	|s: String| s.len() as i32,
/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> bool>,
/// );
/// assert_eq!(is_long_int("123456".to_string()), true);
/// assert_eq!(is_long_int("123".to_string()), false);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfunctorSecondAppliedBrand<Brand, B>(PhantomData<(Brand, B)>);

/// Brand for [`Rc`](`std::rc::Rc`) reference-counted pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcBrand;

/// Brand for [reference-counted][std::rc::Rc] [closures][Fn]
/// (`Rc<dyn Fn(A) -> B>`).
///
/// This type alias provides a way to construct and type-check [`Rc`](`std::rc::Rc`)-wrapped
/// closures in a generic context.
pub type RcFnBrand = FnBrand<RcBrand>;

/// Brand for [`Result`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultBrand;

/// Brand for the partially-applied form of [`Result`] with the [`Err`] type applied.
///
/// This brand forms a [`crate::classes::Functor`] and [`crate::classes::Monad`] over the success ([`Ok`]) type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultErrAppliedBrand<E>(PhantomData<E>);

/// Brand for the partially-applied form of [`Result`] with the [`Ok`] type applied.
///
/// This brand forms a [`crate::classes::Functor`] and [`crate::classes::Monad`] over the error ([`Err`]) type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultOkAppliedBrand<T>(PhantomData<T>);

/// Brand for [`Step`](crate::types::Step).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepBrand;

/// Brand for the partially-applied form of [`Step`](crate::types::Step) with the [`Done`](crate::types::Step::Done) type applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepDoneAppliedBrand<B>(PhantomData<B>);

/// Brand for the partially-applied form of [`Step`](crate::types::Step) with the [`Loop`](crate::types::Step::Loop) type applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepLoopAppliedBrand<A>(PhantomData<A>);

/// Brand for [`SendThunk`](crate::types::SendThunk).
///
/// Thread-safe counterpart of [`ThunkBrand`]. The inner closure is `Send`,
/// enabling deferred computation across thread boundaries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendThunkBrand;

/// Brand for [`Thunk`](crate::types::Thunk).
///
/// Note: This is for `Thunk<'a, A>`, NOT for `Trampoline<A>`.
/// `Trampoline` cannot implement HKT traits due to its `'static` requirement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThunkBrand;

/// Brand for [`TryLazy`](crate::types::TryLazy).
///
/// # Type Parameters
///
/// - `E`: The error type for the fallible computation.
/// - `Config`: The memoization strategy, implementing [`TryLazyConfig`]. Use
///   [`RcLazyConfig`] for single-threaded contexts
///   or [`ArcLazyConfig`] for thread-safe contexts.
///
/// # `'static` bound on `E`
///
/// The type parameter `E` requires `'static` in all HKT trait implementations.
/// This is an inherent limitation of the Brand pattern's reliance on type erasure:
/// the `Kind` trait's associated type `Of<'a, A>` introduces its own lifetime `'a`,
/// so any type parameter baked into the brand must outlive all possible `'a`, which
/// effectively requires `'static`. This prevents use with borrowed error types in
/// HKT contexts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryLazyBrand<E, Config: TryLazyConfig>(PhantomData<(E, Config)>);

/// Brand for single-threaded [`RcTryLazy`](crate::types::RcTryLazy).
pub type RcTryLazyBrand<E> = TryLazyBrand<E, RcLazyConfig>;

/// Brand for thread-safe [`ArcTryLazy`](crate::types::ArcTryLazy).
pub type ArcTryLazyBrand<E> = TryLazyBrand<E, ArcLazyConfig>;

/// Brand for [`TrySendThunk`](crate::types::TrySendThunk) (Bifunctor),
/// enabling fallible deferred computation across thread boundaries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrySendThunkBrand;

/// Brand for [`TryThunk`](crate::types::TryThunk) (Bifunctor).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryThunkBrand;

/// Brand for [`TryThunk`](crate::types::TryThunk) with the error value applied (Functor over [`Ok`]).
///
/// # `'static` bound on `E`
///
/// The type parameter `E` requires `'static` in all HKT trait implementations.
/// This is an inherent limitation of the Brand pattern's reliance on type erasure:
/// the `Kind` trait's associated type `Of<'a, A>` introduces its own lifetime `'a`,
/// so any type parameter baked into the brand must outlive all possible `'a`, which
/// effectively requires `'static`. This prevents use with borrowed error types in
/// HKT contexts.
///
/// # Note
///
/// There is no `TrySendThunkErrAppliedBrand` counterpart. `SendThunk` (and by
/// extension `TrySendThunk`) cannot implement HKT traits like [`Functor`](crate::classes::Functor)
/// because the HKT trait signatures lack `Send` bounds on their closure parameters.
/// Without HKT support, partially-applied brands serve no purpose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryThunkErrAppliedBrand<E>(PhantomData<E>);

/// Brand for [`TryThunk`](crate::types::TryThunk) with the success value applied (Functor over [`Err`]).
///
/// # `'static` bound on `A`
///
/// The type parameter `A` requires `'static` in all HKT trait implementations.
/// This is an inherent limitation of the Brand pattern's reliance on type erasure:
/// the `Kind` trait's associated type `Of<'a, A>` introduces its own lifetime `'a`,
/// so any type parameter baked into the brand must outlive all possible `'a`, which
/// effectively requires `'static`. This prevents use with borrowed success types in
/// HKT contexts.
///
/// # Note
///
/// There is no `TrySendThunkOkAppliedBrand` counterpart. See
/// [`TryThunkErrAppliedBrand`] for the rationale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryThunkOkAppliedBrand<A>(PhantomData<A>);

/// Brand for `(A,)`, with A not applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple1Brand;

/// Brand for `(First, Second)`, with neither `First` nor `Second` applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple2Brand;

/// Brand for `(First, Second)`, with `First` applied (Functor over `Second`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple2FirstAppliedBrand<First>(PhantomData<First>);

/// Brand for `(First, Second)`, with `Second` applied (Functor over `First`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple2SecondAppliedBrand<Second>(PhantomData<Second>);

/// Brand for [`Vec`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecBrand;
