//! Traits representing type-level application.

use crate::make_trait_kind;

make_trait_kind!(Kind0, "*", ());
make_trait_kind!(Kind1, "* -> *", (A));
make_trait_kind!(Kind2, "* -> * -> *", (A, B));
