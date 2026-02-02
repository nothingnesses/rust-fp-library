//! `Kind` traits represent the arity of a kind.
//!
//! These traits are implemented by [`Brand` types][crate::brands],
//! which represent higher-kinded (unapplied/partially-applied) forms
//! (type constructors) of [types][crate::types].
//!
//! This is an implementation of the type-level defunctionalisation technique
//! to simulate higher-kinded types, based on Yallop and White's [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf).
//!
//! # `Kind` Traits
//!
//! Traits representing type-level application to simulate higher-kinded types.
//!
//! The naming convention used by these traits is `Kind_{hash}` where `{hash}` is a
//! deterministic 64-bit hash of the canonical signature.
//!
//! The canonical signature includes:
//! * Number of lifetimes and types.
//! * Type bounds (with full path preservation and generic arguments).
//! * Output bounds on the associated types.
//!
//! This naming scheme ensures that semantically equivalent signatures always map to the
//! same `Kind` trait, regardless of parameter names or formatting.
//!
//! ## Examples
//!
//! * `Kind_ad6c20556a82a1f0`: Signature `type Of<A>;`.
//! * `Kind_140eb1e35dc7afb3`: Signature `type Of<'a, A, B>;`.
//! * `Kind_cdc7cd43dac7585f`: Signature `type Of<'a, A: 'a>: 'a;`.
//!
//! As an example of how to use these traits, the trait [`Kind_ad6c20556a82a1f0`] would be
//! implemented by a [`Brand`][crate::brands] representing type constructors
//! with a single type parameter (e.g., `Foo<A>`). A type `Foo<A>` would have a
//! higher-kinded representation `FooBrand` which implements [`Kind_ad6c20556a82a1f0`].

use fp_macros::def_kind;

def_kind! {
	/// The applied type.
	type Of<A>;
}

def_kind! {
	/// The applied type.
	type Of<A, B>;
}

def_kind! {
	/// The applied type.
	type Of<'a>;
}

def_kind! {
	/// The applied type.
	type Of<'a, A>;
}

def_kind! {
	/// The applied type.
	type Of<'a, A, B>;
}

def_kind! {
	/// The applied type.
	type Of<'a, A: 'a>: 'a;
}

def_kind! {
	/// The applied type.
	type Of<'a, A: 'a, B: 'a>: 'a;
}
