//! `Kind` and `InferableBrand` traits for higher-kinded type simulation.
//!
//! Each [`trait_kind!`](fp_macros::trait_kind) invocation generates a `Kind_{hash}` trait
//! (forward mapping: brand -> concrete type) and a matching `InferableBrand_{hash}` trait
//! (reverse mapping: concrete type -> brand). Both share a deterministic content hash
//! derived from the signature.
//!
//! For a full explanation of the HKT encoding and the hash naming convention,
//! see [Higher-Kinded Types][crate::docs::hkt]. For the trait shapes, impl
//! landscape, and Marker invariant, see
//! [Brand Inference][crate::docs::brand_inference].

use fp_macros::trait_kind;

trait_kind! {
	/// The applied type.
	type Of<A>;
}

trait_kind! {
	/// The applied type.
	type Of<'a>;
}

trait_kind! {
	/// The applied type.
	type Of<'a, A>;
}

trait_kind! {
	/// The applied type.
	type Of<'a, A: 'a>: 'a;
}

trait_kind! {
	/// The applied type.
	type Of<A, B>;
}

trait_kind! {
	/// The applied type.
	type Of<'a, A, B>;
}

trait_kind! {
	/// The applied type.
	type Of<'a, A, B>: 'a;
}

trait_kind! {
	/// The applied type.
	type Of<'a, A: 'a, B: 'a>: 'a;
}
