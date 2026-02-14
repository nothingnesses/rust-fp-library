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
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

use crate::classes::RefCountedPointer;
use std::marker::PhantomData;

/// Brand for [`Arc`](std::sync::Arc) atomic reference-counted pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcBrand;

/// Brand for [atomically reference-counted][std::sync::Arc]
/// [closures][Fn] (`Arc<dyn Fn(A) -> B>`).
///
/// This type alias provides a way to construct and type-check [`Arc`](std::sync::Arc)-wrapped
/// closures in a generic context.
pub type ArcFnBrand = FnBrand<ArcBrand>;

/// Brand for [`CatList`](crate::types::CatList).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatListBrand;

/// Brand for [`Thunk`](crate::types::Thunk).
///
/// Note: This is for `Thunk<'a, A>`, NOT for `Trampoline<A>`.
/// `Trampoline` cannot implement HKT traits due to its `'static` requirement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThunkBrand;

/// Generic function brand parameterized by reference-counted pointer choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FnBrand<PtrBrand: RefCountedPointer>(PhantomData<PtrBrand>);

/// Brand for [`Identity`](crate::types::Identity).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityBrand;

/// Brand for [`Lazy`](crate::types::Lazy).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LazyBrand<Config>(PhantomData<Config>);

/// Brand for [`Option`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionBrand;

/// Brand for [`Pair`](crate::types::Pair).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairBrand;

/// Brand for the partially-applied form of [`Pair`](crate::types::Pair) with the first value fixed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairWithFirstBrand<First>(PhantomData<First>);

/// Brand for the partially-applied form of [`Pair`](crate::types::Pair) with the second value fixed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairWithSecondBrand<Second>(PhantomData<Second>);

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

/// Brand for the partially-applied form of [`Result`] with the [`Err`] type fixed.
///
/// This brand forms a [`crate::classes::Functor`] and [`crate::classes::Monad`] over the success ([`Ok`]) type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultWithErrBrand<E>(PhantomData<E>);

/// Brand for the partially-applied form of [`Result`] with the [`Ok`] type fixed.
///
/// This brand forms a [`crate::classes::Functor`] and [`crate::classes::Monad`] over the error ([`Err`]) type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultWithOkBrand<T>(PhantomData<T>);

/// Brand for [`Step`](crate::types::Step).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepBrand;

/// Brand for the partially-applied form of [`Step`](crate::types::Step) with the [`Loop`](crate::types::Step::Loop) type fixed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepWithLoopBrand<A>(PhantomData<A>);

/// Brand for the partially-applied form of [`Step`](crate::types::Step) with the [`Done`](crate::types::Step::Done) type fixed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StepWithDoneBrand<B>(PhantomData<B>);

/// Brand for [`TryLazy`](crate::types::TryLazy).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryLazyBrand<E, Config>(PhantomData<(E, Config)>);

/// Brand for [`TryThunk`](crate::types::TryThunk) (Bifunctor).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryThunkBrand;

/// Brand for [`TryThunk`](crate::types::TryThunk) with the error value fixed (Functor over [`Ok`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryThunkWithErrBrand<E>(PhantomData<E>);

/// Brand for [`TryThunk`](crate::types::TryThunk) with the success value fixed (Functor over [`Err`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryThunkWithOkBrand<A>(PhantomData<A>);

/// Brand for `(A,)`, with A not filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple1Brand;

/// Brand for `(First, Second)`, with neither `First` nor `Second` fixed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple2Brand;

/// Brand for `(First, Second)`, with `First` fixed (Functor over `Second`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple2WithFirstBrand<First>(PhantomData<First>);

/// Brand for `(First, Second)`, with `Second` fixed (Functor over `First`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple2WithSecondBrand<Second>(PhantomData<Second>);

/// Brand for [`Vec`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecBrand;
