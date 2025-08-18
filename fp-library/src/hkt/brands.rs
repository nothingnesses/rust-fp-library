//! Traits containing functions to convert between the concrete type and the
//! corresponding instantiation of [`Apply`].

use crate::{
	hkt::{Apply0, Apply1, Apply2, Kind0, Kind1, Kind2}, make_trait_brand
};

make_trait_brand!(Brand0, Kind0, Apply0, "*", ());
make_trait_brand!(Brand1, Kind1, Apply1, "* -> *", (A));
make_trait_brand!(Brand2, Kind2, Apply2, "* -> * -> *", (A, B));
