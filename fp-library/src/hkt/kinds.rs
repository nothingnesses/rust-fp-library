//! Traits representing type-level application to simulate higher-kinded types.
//!
//! The naming convention used by these traits is `KindNLMT` where `N`
//! represents the number of lifetimes and `M` represents the number of
//! generic types.
//!
//! As an example of how to use these traits, the trait [`Kind0L1T`] would be
//! implemented by a [`Brand`][crate::brands] representing type constructors
//! with 0 lifetimes and 1 generic type. A type `Foo<A>` would have a
//! higher-kinded representation `FooBrand` which implements [`Kind0L1T`].

use crate::make_trait_kind;

make_trait_kind!(Kind0L1T, (), (A), "* -> *");

make_trait_kind!(Kind0L2T, (), (A, B), "* -> * -> *");

make_trait_kind!(
  Kind1L0T,
  ('a),
  (),
  "' -> *"
);

make_trait_kind!(
	Kind1L2T,
	('a),
	(A, B),
	"' -> * -> * -> *"
);

/// Trait for [brands][crate::brands] of [types][crate::types] of kind `' -> * -> *`.
///
/// This trait represents a type constructor that takes one lifetime and one type argument.
/// The type argument `A` is bounded by the lifetime `'a`, ensuring that the resulting type
/// can validly hold values of type `A` for the duration of `'a`.
/// The output type itself is also bounded by `'a`.
pub trait Kind1L1T {
	type Output<'a, A: 'a>: 'a;
}
