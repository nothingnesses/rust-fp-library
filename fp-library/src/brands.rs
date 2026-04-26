//! Brands represent higher-kinded (unapplied/partially-applied) forms of
//! [types][crate::types], as opposed to concrete types, which are
//! fully-applied.
//!
//! For example, [`VecBrand`] represents the higher-kinded type [`Vec`], whereas
//! `Vec A`/`Vec<A>` is the concrete type where `Vec` has been applied to some
//! generic type `A`.
//!
//! For how brands encode HKTs, see [Higher-Kinded Types][crate::docs::hkt].
//! For the brand inference system, see [Brand Inference][crate::docs::brand_inference].
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! let x = Some(5);
//! let y = map::<OptionBrand, _, _, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

use {
	crate::{
		classes::{
			LazyConfig,
			RefCountedPointer,
		},
		types::{
			ArcLazyConfig,
			RcLazyConfig,
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

/// Brand for thread-safe [`ArcLazy`](crate::types::ArcLazy).
pub type ArcLazyBrand = LazyBrand<ArcLazyConfig>;

/// Brand for thread-safe [`ArcTryLazy`](crate::types::ArcTryLazy).
pub type ArcTryLazyBrand<E> = TryLazyBrand<E, ArcLazyConfig>;

/// An adapter that partially applies a [`Bifunctor`](crate::classes::Bifunctor) to its first argument, creating a [`Functor`](crate::classes::Functor) over the second argument.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::explicit::*,
/// };
///
/// let x = Result::<i32, i32>::Ok(5);
/// let y = map::<BifunctorFirstAppliedBrand<ResultBrand, i32>, _, _, _, _>(|s| s * 2, x);
/// assert_eq!(y, Ok(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BifunctorFirstAppliedBrand<Brand, A>(PhantomData<(Brand, A)>);

/// An adapter that partially applies a [`Bifunctor`](crate::classes::Bifunctor) to its second argument, creating a [`Functor`](crate::classes::Functor) over the first argument.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::explicit::*,
/// };
///
/// let x = Result::<i32, i32>::Err(5);
/// let y = map::<BifunctorSecondAppliedBrand<ResultBrand, i32>, _, _, _, _>(|e| e * 2, x);
/// assert_eq!(y, Err(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BifunctorSecondAppliedBrand<Brand, B>(PhantomData<(Brand, B)>);

/// Brand for [`Box`] owned heap-allocated pointer.
///
/// `BoxBrand` implements [`Pointer`](crate::classes::Pointer) and
/// [`ToDynFn`](crate::classes::ToDynFn) but not
/// [`RefCountedPointer`] (since `Box<dyn Fn>` is not `Clone`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoxBrand;

/// Brand for [`CatList`](crate::types::CatList).
///
/// `CatList` is the catenable list that serves as the backbone of
/// [`Free`](crate::types::Free) monad evaluation, providing O(1) append
/// and amortized O(1) uncons for the "Reflection without Remorse" technique
/// that makes `Free` stack-safe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatListBrand;

/// Brand for the [`Const`](crate::types::const_val::Const) functor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstBrand<R>(PhantomData<R>);

/// Brand for [`ControlFlow`](core::ops::ControlFlow).
///
/// The type parameters are swapped relative to `ControlFlow<B, C>` so that
/// the first HKT parameter is the continue (loop/state) value and the second
/// is the break (done/result) value, matching `tail_rec_m` conventions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlFlowBrand;

/// Brand for the partially-applied form of [`ControlFlow`](core::ops::ControlFlow) with the [`Break`](core::ops::ControlFlow::Break) type applied.
///
/// Fixes the `Break` (result) type, yielding a `Functor` over the `Continue`
/// (continuation) type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlFlowBreakAppliedBrand<B>(PhantomData<B>);

/// Brand for the partially-applied form of [`ControlFlow`](core::ops::ControlFlow) with the [`Continue`](core::ops::ControlFlow::Continue) type applied.
///
/// Fixes the `Continue` (continuation) type, yielding a `Functor` over the `Break`
/// (result) type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlFlowContinueAppliedBrand<C>(PhantomData<C>);

/// Brand for [`ArcCoyoneda`](crate::types::ArcCoyoneda), the thread-safe
/// reference-counted free functor.
///
/// Like [`CoyonedaBrand`], but the underlying `ArcCoyoneda` is `Clone`, `Send`,
/// and `Sync`, enabling additional type class instances.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcCoyonedaBrand<F>(PhantomData<F>);

/// Brand for [`ArcFreeExplicit`](crate::types::ArcFreeExplicit), the
/// thread-safe multi-shot naive recursive Free monad supporting non-`'static`
/// payloads.
///
/// Like [`RcFreeExplicitBrand`], the underlying type keeps the functor
/// structure as a concrete recursive enum (no `dyn Any` erasure), so `A: 'a`
/// is admitted at the cost of O(N) [`bind`](crate::types::ArcFreeExplicit::bind)
/// on left-associated chains. The outer [`Arc`](std::sync::Arc) wrapper plus
/// [`Arc<dyn Fn + Send + Sync>`](std::sync::Arc) continuations provide
/// unconditional O(1) [`Clone`] and [`Send`] + [`Sync`] participation,
/// matching [`ArcFree`](crate::types::ArcFree)'s thread-safety pattern.
///
/// `F` must be `'static` because the [`Kind`](crate::kinds) trait's associated
/// type `Of<'a, A>` introduces its own lifetime `'a`, so type parameters baked
/// into the brand must outlive all possible `'a`. In practice this is not a
/// restriction because all brands in the library are zero-sized marker types,
/// which are inherently `'static`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcFreeExplicitBrand<F>(PhantomData<F>);

/// Brand for [`Coyoneda`](crate::types::Coyoneda), the free functor.
///
/// `CoyonedaBrand<F>` is a [`Functor`](crate::classes::Functor) for any type constructor
/// `F` with the appropriate [`Kind`](crate::kinds) signature, even if `F` itself is not
/// a `Functor`. The `Functor` constraint on `F` is only required when
/// [`lower`](crate::types::Coyoneda::lower)ing back to `F`.
///
/// `F` must be `'static` because the [`Kind`](crate::kinds) trait's associated type
/// `Of<'a, A>` introduces its own lifetime `'a`, so type parameters baked into the
/// brand must outlive all possible `'a`. In practice this is not a restriction because
/// all brands in the library are zero-sized marker types, which are inherently `'static`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoyonedaBrand<F>(PhantomData<F>);

/// Brand for [`BoxedCoyonedaExplicit`](crate::types::BoxedCoyonedaExplicit),
/// the boxed variant of [`CoyonedaExplicit`](crate::types::CoyonedaExplicit).
///
/// Unlike [`CoyonedaBrand`], which hides the intermediate type `B` behind a
/// trait object (producing k calls to `F::map` at lower time), this brand
/// exposes `B` as a type parameter, enabling single-pass fusion (one `F::map`
/// at lower time regardless of how many maps were chained). The trade-off is
/// that `B` is fixed for a given brand instance, which prevents implementing
/// `Pointed`, `Semiapplicative`, or `Semimonad`.
///
/// Implements [`Functor`](crate::classes::Functor) (without requiring
/// `F: Functor`) and [`Foldable`](crate::classes::Foldable) (without requiring
/// `F: Functor`, only `F: Foldable`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoyonedaExplicitBrand<F, B>(PhantomData<(F, B)>);

/// Generic function brand parameterized by reference-counted pointer choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FnBrand<PtrBrand: RefCountedPointer>(PhantomData<PtrBrand>);

/// Brand for [`FreeExplicit`](crate::types::FreeExplicit), the naive recursive
/// Free monad supporting non-`'static` payloads.
///
/// Unlike the existing [`Free`](crate::types::Free), which cannot be a brand
/// because its `Box<dyn Any>` continuation queue forces `A: 'static`,
/// `FreeExplicit` keeps the functor structure as a concrete recursive enum
/// and so satisfies the [`Kind`](crate::kinds) signature. The trade-off is
/// O(N) [`bind`](crate::types::FreeExplicit::bind) on left-associated chains.
///
/// `F` must be `'static` because the [`Kind`](crate::kinds) trait's associated
/// type `Of<'a, A>` introduces its own lifetime `'a`, so type parameters baked
/// into the brand must outlive all possible `'a`. In practice this is not a
/// restriction because all brands in the library are zero-sized marker types,
/// which are inherently `'static`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FreeExplicitBrand<F>(PhantomData<F>);

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

/// An adapter that partially applies a [`Profunctor`](crate::classes::Profunctor) to its first argument, creating a [`Functor`](crate::classes::Functor).
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::explicit::*,
/// };
///
/// let f = |x: i32| x + 1;
/// let g = map::<ProfunctorFirstAppliedBrand<RcFnBrand, i32>, _, _, _, _>(
/// 	|y: i32| y * 2,
/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
/// );
/// assert_eq!(g(10), 22); // (10 + 1) * 2 = 22
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfunctorFirstAppliedBrand<Brand, A>(PhantomData<(Brand, A)>);

/// An adapter that partially applies a [`Profunctor`](crate::classes::Profunctor) to its second argument, creating a [`Contravariant`](crate::classes::Contravariant) functor.
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

/// Brand for [`RcCoyoneda`](crate::types::RcCoyoneda), the reference-counted
/// free functor with [`Clone`] support.
///
/// Like [`CoyonedaBrand`], but the underlying `RcCoyoneda` is `Clone`, enabling
/// additional type class instances such as [`Semiapplicative`](crate::classes::Semiapplicative).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcCoyonedaBrand<F>(PhantomData<F>);

/// Brand for [reference-counted][std::rc::Rc] [closures][Fn]
/// (`Rc<dyn Fn(A) -> B>`).
///
/// This type alias provides a way to construct and type-check [`Rc`](`std::rc::Rc`)-wrapped
/// closures in a generic context.
pub type RcFnBrand = FnBrand<RcBrand>;

/// Brand for [`RcFreeExplicit`](crate::types::RcFreeExplicit), the multi-shot
/// reference-counted naive recursive Free monad supporting non-`'static`
/// payloads.
///
/// Like [`FreeExplicitBrand`], the underlying type keeps the functor structure
/// as a concrete recursive enum (no `dyn Any` erasure), so `A: 'a` is admitted
/// at the cost of O(N) [`bind`](crate::types::RcFreeExplicit::bind) on
/// left-associated chains. The outer [`Rc`](std::rc::Rc) wrapper plus
/// [`Rc<dyn Fn>`](std::rc::Rc) continuations provide unconditional O(1)
/// [`Clone`] and multi-shot semantics, matching
/// [`RcFree`](crate::types::RcFree)'s cloning pattern.
///
/// `F` must be `'static` because the [`Kind`](crate::kinds) trait's associated
/// type `Of<'a, A>` introduces its own lifetime `'a`, so type parameters baked
/// into the brand must outlive all possible `'a`. In practice this is not a
/// restriction because all brands in the library are zero-sized marker types,
/// which are inherently `'static`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFreeExplicitBrand<F>(PhantomData<F>);

/// Brand for single-threaded [`RcLazy`](crate::types::RcLazy).
pub type RcLazyBrand = LazyBrand<RcLazyConfig>;

/// Brand for single-threaded [`RcTryLazy`](crate::types::RcTryLazy).
pub type RcTryLazyBrand<E> = TryLazyBrand<E, RcLazyConfig>;

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

/// Brand for [`SendThunk`](crate::types::SendThunk).
///
/// Thread-safe counterpart of [`ThunkBrand`]. The inner closure is `Send`,
/// enabling deferred computation across thread boundaries.
///
/// # HKT limitations
///
/// `SendThunkBrand` does **not** implement [`Functor`](crate::classes::Functor),
/// [`Monad`](crate::classes::Monad), or any other HKT type-class traits.
/// Those traits accept closure parameters as `impl Fn`/`impl FnOnce` without a
/// `Send` bound, so there is no way to guarantee that the closures passed to
/// `map`, `bind`, etc. are safe to store inside a `Send` thunk. Use
/// [`ThunkBrand`] when HKT polymorphism is needed, or work with `SendThunk`
/// directly through its inherent methods.
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
/// - `Config`: The memoization strategy, implementing [`LazyConfig`]. Use
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
pub struct TryLazyBrand<E, Config: LazyConfig>(PhantomData<(E, Config)>);

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
