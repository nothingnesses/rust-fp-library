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

/// [Brand][crate::brands] for [atomically reference-counted][std::sync::Arc]
/// [closures][Fn] (`Arc<dyn Fn(A) -> B>`).
///
/// This struct implements [`ClonableFn`] to provide a way to construct and
/// type-check [`std::sync::Arc`]-wrapped closures in a generic context. The lifetime `'a`
/// ensures the closure doesn't outlive referenced data, while `A` and `B`
/// represent input and output types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcFnBrand;

/// [Brand][crate::brands] for [`crate::types::Endofunction`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndofunctionBrand<ClonableFnBrand: ClonableFn, A>(PhantomData<(ClonableFnBrand, A)>);

/// [Brand][crate::brands] for [`crate::types::Endomorphism`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);

/// [Brand][crate::brands] for [`crate::types::Identity`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityBrand;

/// [Brand][crate::brands] for [`crate::types::Lazy`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LazyBrand<OnceBrand: Once, ClonableFnBrand: ClonableFn>(
	PhantomData<(OnceBrand, ClonableFnBrand)>,
);

/// [Brand][crate::brands] for [`std::cell::OnceCell`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceCellBrand;

/// [Brand][crate::brands] for [`std::sync::OnceLock`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceLockBrand;

/// [Brand][crate::brands] for [`Option`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionBrand;

/// [Brand][crate::brands] for [`crate::types::Pair`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairBrand;

/// [Brand][crate::brands] for the partially-applied form of [`crate::types::Pair`] with [the first value][crate::types::Pair] filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairWithFirstBrand<First>(First);

/// [Brand][crate::brands] for the partially-applied form of [`crate::types::Pair`] with [the second value][crate::types::Pair] filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PairWithSecondBrand<Second>(Second);

/// [Brand][crate::brands] for [reference-counted][std::rc::Rc] [closures][Fn]
/// (`Rc<dyn Fn(A) -> B>`).
///
/// This struct implements [`ClonableFn`] to provide a way to construct and
/// type-check [`std::rc::Rc`]-wrapped closures in a generic context. The lifetime `'a`
/// ensures the closure doesn't outlive referenced data, while `A` and `B`
/// represent input and output types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;

/// [Brand][crate::brands] for [`Result`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultBrand;

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultWithErrBrand<E>(E);

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResultWithOkBrand<T>(T);

/// [Brand][crate::brands] for [`Vec`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VecBrand;
