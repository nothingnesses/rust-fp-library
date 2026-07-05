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

/// The stable name of the `type Of<'a, A: 'a>: 'a` kind trait, the shape the
/// effects subsystem's brands implement, so hand-written bounds can spell
/// this alias instead of the generated `Kind_{hash}` name. Macro-emitted
/// code names the same trait through the generator that produces it, so the
/// two spellings cannot diverge.
pub use self::Kind_cdc7cd43dac7585f as LifetimeUnaryKind;
