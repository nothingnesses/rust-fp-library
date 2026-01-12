//! Higher-kinded representations of [types][crate::types].
//!
//! Brands represent higher-kinded (unapplied/partially-applied) forms of
//! [types][crate::types], as opposed to concrete types, which are
//! fully-applied.
//!
//! For example, [`VecBrand`] represents the higher-kinded type `Vec`, whereas
//! `Vec A`/`Vec<A>` is the concrete type where `Vec` has been applied to some
//! generic type `A`.

use crate::classes::{category::Category, clonable_fn::ClonableFn, once::Once};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcFnBrand;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndofunctionBrand<ClonableFnBrand: ClonableFn, A>(PhantomData<(ClonableFnBrand, A)>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);

pub struct IdentityBrand;

/// Brand for the `Lazy` type constructor.
pub struct LazyBrand<OnceBrand: Once, ClonableFnBrand: ClonableFn>(
	PhantomData<(OnceBrand, ClonableFnBrand)>,
);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceCellBrand;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceLockBrand;

pub struct OptionBrand;

pub struct PairBrand;

/// [Brand][crate::brands] for the partially-applied form of [`crate::types::Pair`] with [the first value][crate::types::Pair] filled in.
pub struct PairWithFirstBrand<First>(First);

/// [Brand][crate::brands] for the partially-applied form of [`crate::types::Pair`] with [the second value][crate::types::Pair] filled in.
pub struct PairWithSecondBrand<Second>(Second);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;

/// [Brand][crate::brands] for [`Result`].
pub struct ResultBrand;

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.
pub struct ResultWithErrBrand<E>(E);

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.
pub struct ResultWithOkBrand<T>(T);

pub struct VecBrand;
