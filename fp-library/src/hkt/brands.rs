//! Traits containing functions to convert between the concrete type and the
//! corresponding instantiation of [`Apply`].

use crate::{
	hkt::{apply::Apply, kinds::Kind},
	make_trait_brand,
};

/// Contains functions to convert between the concrete type and the
/// corresponding instantiation of [`Apply`].
pub trait Brand<Concrete, Parameters>: Kind<Parameters> {
	fn inject(a: Concrete) -> Self::Output;
	fn project(a: Self::Output) -> Concrete;
}

make_trait_brand!(Brand1, "* -> *", (A));
make_trait_brand!(Brand2, "* -> * -> *", (A, B));
