//! Traits representing type-level application.

use crate::make_trait_kind;

make_trait_kind!(Kind0, Apply0, "*", ());
make_trait_kind!(Kind1, Apply1, "* -> *", (A));
make_trait_kind!(Kind2, Apply2, "* -> * -> *", (A, B));
