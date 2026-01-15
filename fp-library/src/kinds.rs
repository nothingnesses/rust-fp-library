//! Simulates higher-kinded types using type-level defunctionalisation based on Yallop
//! and White's [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf).
//!
//! `Kind` traits represent the arity of a kind.
//! These traits are implemented by [`Brand` types][crate::brands],
//! which represent higher-kinded (unapplied/partially-applied) forms
//! (type constructors) of [types][crate::types].
//!
//! # Kind Traits
//!
//! Traits representing type-level application to simulate higher-kinded types.
//!
//! The naming convention used by these traits is `Kind_{hash}` where `{hash}` is a
//! deterministic 64-bit hash of the canonical signature.
//!
//! The canonical signature includes:
//! * Number of lifetimes and types.
//! * Type bounds (with full path preservation and generic arguments).
//! * Output bounds on the associated `Of` type.
//!
//! This naming scheme ensures that semantically equivalent signatures always map to the
//! same Kind trait, regardless of parameter names or formatting.
//!
//! ## Examples
//!
//! * `Kind_bd4ddc17b95f4bc6`: 0 lifetimes, 1 type.
//! * `Kind_fcf9d56b89a0b8b9`: 1 lifetime, 2 types.
//! * `Kind_c3c3610c70409ee6`: 1 lifetime, 1 type. Type 0 is bounded by Lifetime 0 (`A: 'a`). Of is bounded by Lifetime 0 (`Of: 'a`).
//!
//! As an example of how to use these traits, the trait [`Kind_bd4ddc17b95f4bc6`] would be
//! implemented by a [`Brand`][crate::brands] representing type constructors
//! with 0 lifetimes and 1 generic type. A type `Foo<A>` would have a
//! higher-kinded representation `FooBrand` which implements [`Kind_bd4ddc17b95f4bc6`].

use fp_macros::def_kind;

def_kind!((), (A), ());

def_kind!((), (A, B), ());

def_kind!(
	('a),
	(),
	()
);

def_kind!(
	('a),
	(A, B),
	()
);

def_kind!(
	('a),
	(A: 'a),
	('a)
);
