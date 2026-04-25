//! POC for the effect-row canonicalisation hybrid (workaround 1 +
//! workaround 3 from port-plan section 4.1).
//!
//! Re-exports the `effects!` and `effects_coyo!` macros and frunk's
//! coproduct machinery for use in tests.

pub use {
	effect_row_macros::{
		effects,
		effects_coyo,
	},
	frunk_core::coproduct::{
		CNil,
		Coproduct,
		CoproductSubsetter,
	},
};

/// Stub Coyoneda for the POC.
///
/// Models `Coyoneda<F, A> = exists B. (F<B>, B -> A)` enough to
/// demonstrate the macro integration story for port-plan section 4.2's
/// static option (each row variant wrapped in `Coyoneda<E>` becomes a
/// `Functor` regardless of `E`'s own shape). The intermediate type `B`
/// is erased via `Box<dyn Any>`, mirroring how fp-library's real
/// `Coyoneda` handles the existential on stable Rust.
///
/// This stub does NOT implement the full Functor laws; it implements
/// the SHAPE that satisfies them. The point is to demonstrate that:
///   1. `Coyoneda<F, A>` is constructable for ANY `F` (no Functor
///      bound on `F`),
///   2. `map` composes a new `Coyoneda<F, B>` without re-running `F`,
///   3. The resulting type plugs into the row machinery cleanly.
pub mod coyoneda {
	use std::{
		any::Any,
		marker::PhantomData,
	};

	pub struct Coyoneda<F, A> {
		// `fb` holds an `F` containing some intermediate type `B`,
		// stored erased as `Box<dyn Any>`. This is the "exists B"
		// part of the existential.
		pub fb: F,
		// The function `B -> A`, also erased so we can store it
		// without naming `B` in the type.
		pub map: Box<dyn FnOnce(Box<dyn Any>) -> A>,
		_marker: PhantomData<A>,
	}

	impl<F, A: 'static> Coyoneda<F, A> {
		/// Lift `F` into `Coyoneda<F, A>` using an identity map.
		///
		/// The caller must provide a closure that converts the erased
		/// intermediate back to `A`. In the simple case where `F`
		/// already produces `A`, this closure is `|x: Box<dyn Any>|
		/// *x.downcast::<A>().expect("type-correct lift")`.
		pub fn lift(
			fb: F,
			decode: impl FnOnce(Box<dyn Any>) -> A + 'static,
		) -> Self {
			Self {
				fb,
				map: Box::new(decode),
				_marker: PhantomData,
			}
		}

		/// `map` composes the function and produces a new
		/// `Coyoneda<F, B>` without touching `fb`. This is the key
		/// property: ANY `F` becomes a `Functor` via `Coyoneda<F>`
		/// because `map` operates on the lifted function, not on `F`.
		pub fn map<B: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> Coyoneda<F, B> {
			let prev = self.map;
			Coyoneda {
				fb: self.fb,
				map: Box::new(move |any| f(prev(any))),
				_marker: PhantomData,
			}
		}

		/// Lower the Coyoneda back to its original `F<A>` shape by
		/// running the composed map over the stored intermediate.
		///
		/// The caller supplies a function that extracts the erased
		/// intermediate from `F`. For a simple wrapper, this is
		/// `|fb| Box::new(fb) as Box<dyn Any>`.
		pub fn lower(
			self,
			extract: impl FnOnce(F) -> Box<dyn Any>,
		) -> A {
			(self.map)(extract(self.fb))
		}
	}
}

/// Minimal `Functor` trait for the POC.
///
/// `Coyoneda<F, A>` implements this for any `F` without requiring `F`
/// to implement `Functor` itself. That's the static-via-Coyoneda claim
/// from section 4.2 reduced to one trait impl.
pub trait Functor<A> {
	type Output<B>;
	fn fmap<B, Func>(
		self,
		f: Func,
	) -> Self::Output<B>
	where
		Func: FnOnce(A) -> B + 'static,
		B: 'static;
}

impl<F: 'static, A: 'static> Functor<A> for coyoneda::Coyoneda<F, A> {
	type Output<B> = coyoneda::Coyoneda<F, B>;

	fn fmap<B, Func>(
		self,
		f: Func,
	) -> coyoneda::Coyoneda<F, B>
	where
		Func: FnOnce(A) -> B + 'static,
		B: 'static, {
		self.map(f)
	}
}

// Recursive Functor for Coproduct<H, T>: dispatches `fmap` to the
// active variant. This is the actual mechanism by which
// `VariantF<R>` implements `Functor` under the static option in
// port-plan section 4.2.
impl<H, T, A> Functor<A> for Coproduct<H, T>
where
	H: Functor<A> + 'static,
	T: Functor<A> + 'static,
	A: 'static,
{
	type Output<B> = Coproduct<H::Output<B>, T::Output<B>>;

	fn fmap<B, Func>(
		self,
		f: Func,
	) -> Self::Output<B>
	where
		Func: FnOnce(A) -> B + 'static,
		B: 'static, {
		match self {
			Coproduct::Inl(h) => Coproduct::Inl(h.fmap(f)),
			Coproduct::Inr(t) => Coproduct::Inr(t.fmap(f)),
		}
	}
}

impl<A: 'static> Functor<A> for CNil {
	type Output<B> = CNil;

	fn fmap<B, Func>(
		self,
		_f: Func,
	) -> CNil
	where
		Func: FnOnce(A) -> B + 'static,
		B: 'static, {
		match self {}
	}
}
