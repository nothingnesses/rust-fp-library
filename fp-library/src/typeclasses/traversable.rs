use crate::{
	hkt::{Apply, Kind},
	typeclasses::{Applicative, Foldable, Functor},
};

pub trait Traversable: Functor + Foldable {
	/// # Type Signature
	///
	/// `forall t f a b. Traversable t, Applicative f => (a -> f b) -> t a -> f (t b)`
	fn traverse<F, A, B>(
		f: impl Fn(A) -> Apply<F, (B,)>
	) -> impl (Fn(Apply<Self, (A,)>) -> Apply<F, (Apply<Self, (B,)>,)>)
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Kind<(B,)> + Kind<(Apply<Self, (B,)>,)> + Applicative;
}
