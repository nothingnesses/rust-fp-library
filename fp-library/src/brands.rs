//! Higher-kinded representations of [types][crate::types].
//!
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

use crate::classes::{
	category::Category, cloneable_fn::CloneableFn, ref_counted_pointer::RefCountedPointer,
};
use std::marker::PhantomData;

/// Brand for [`std::sync::Arc`] atomic reference-counted pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcBrand;

/// Brand for [atomically reference-counted][std::sync::Arc]
/// [closures][Fn] (`Arc<dyn Fn(A) -> B>`).
///
/// This type alias provides a way to construct and type-check [`std::sync::Arc`]-wrapped
/// closures in a generic context.
pub type ArcFnBrand = FnBrand<ArcBrand>;

/// Brand for [`std::boxed::Box`] unique ownership pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoxBrand;

/// Brand for [`Endofunction`](crate::types::Endofunction).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndofunctionBrand<FnBrand: CloneableFn, A>(PhantomData<(FnBrand, A)>);

/// Generic function brand parameterized by reference-counted pointer choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FnBrand<PtrBrand: RefCountedPointer>(PhantomData<PtrBrand>);

/// Brand for [`Endomorphism`](crate::types::Endomorphism).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);

/// Brand for [`Identity`](crate::types::Identity).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityBrand;

/// Brand for [`Lazy`](crate::types::Lazy).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LazyBrand<Config>(PhantomData<Config>);

/// Brand for [`OnceCell`](std::cell::OnceCell).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceCellBrand;

/// Brand for [`OnceLock`](std::sync::OnceLock).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceLockBrand;

/// Brand for [`Option`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionBrand;

/// Brand for [`Pair`](crate::types::Pair).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairBrand;

/// Brand for the partially-applied form of [`Pair`](crate::types::Pair) with [the first value][crate::types::Pair] filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairWithFirstBrand<First>(First);

/// Brand for the partially-applied form of [`Pair`](crate::types::Pair) with [the second value][crate::types::Pair] filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairWithSecondBrand<Second>(Second);

/// Brand for [`std::rc::Rc`] reference-counted pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcBrand;

/// Brand for [reference-counted][std::rc::Rc] [closures][Fn]
/// (`Rc<dyn Fn(A) -> B>`).
///
/// This type alias provides a way to construct and type-check [`std::rc::Rc`]-wrapped
/// closures in a generic context.
pub type RcFnBrand = FnBrand<RcBrand>;

/// Brand for [`Result`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultBrand;

/// Brand for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultWithErrBrand<E>(E);

/// Brand for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultWithOkBrand<T>(T);

/// Brand for [`Vec`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecBrand;
