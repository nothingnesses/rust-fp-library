use std::process::Output;

use crate::{
	classes::{
		ClonableFn, Monoid, Semigroup, Semigroupoid, clonable_fn::ApplyFn, monoid::HktMonoid,
		semigroup::HktSemigroup,
	},
	functions::{append, compose, flip, identity},
	hkt::{Apply0L1T, Apply1L0T, Apply1L2T, Kind0L1T, Kind1L0T},
	types::{Endomorphism, endomorphism::EndomorphismHkt},
};

/// A type class for structures that can be folded to a single value.
///
/// A `Foldable` represents a structure that can be folded over to combine its elements
/// into a single result. This is useful for operations like summing values, collecting into a collection,
/// or applying monoidal operations.
///
/// A minimum implementation of `Foldable` requires the manual implementation of at least [`Foldable::fold_right`] or [`Foldable::fold_map`].
pub trait Foldable: Kind0L1T {
	// /// Maps values to a monoid and combines them.
	// ///
	// /// The default implementation of `fold_map` is implemented in terms of [`fold_right`], [`compose`], [`append`][crate::functions::append] and [`empty`][crate::functions::empty] where:
	// ///
	// /// `fold_map f = (fold_right ((compose append) f)) empty`
	// ///
	// /// # Type Signature
	// ///
	// /// `forall a. Foldable f, Monoid m => (a -> m) -> f a -> m`
	// ///
	// /// # Parameters
	// ///
	// /// * `f`: A function that converts from values into monoidal elements.
	// /// * `fa`: A foldable structure containing values of type `A`.
	// ///
	// /// # Returns
	// ///
	// /// Final monoid obtained from the folding operation.
	// fn fold_map<'a, ClonableFnBrand: 'a + ClonableFn, A, M: Clone>(
	// 	f: ApplyFn<'a, ClonableFnBrand, A, M>
	// ) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, M>
	// where
	// 	for<'b> Apply1L0T<'b, M>: Monoid<'b>,
	// 	for<'b> M: HktMonoid<Output<'b> = M>,
	// 	for<'b> ApplyFn<'a, ClonableFnBrand, M, M>:
	// 		HktMonoid<Output<'b> = ApplyFn<'a, ClonableFnBrand, M, M>>,
	// 	for<'b> ApplyFn<'a, ClonableFnBrand, M, M>: Monoid<'b>,
	// {
	// 	ClonableFnBrand::new(move |fa: Apply0L1T<Self, A>| {
	// 		let app =
	// 			ClonableFnBrand::new::<'a, M, _>(move |a| append::<'a, ClonableFnBrand, M>(a));
	// 		let compose_append = compose::<'a, ClonableFnBrand, A, M, _>(app);
	// 		let compose_append_f = compose_append(f.clone());
	// 		let compose_append_f =
	// 			ClonableFnBrand::new::<'a, A, _>(move |a: A| compose_append_f(a));
	// 		let fold_right_compose_append_f =
	// 			Self::fold_right::<'a, ClonableFnBrand, A, M>(compose_append_f);
	// 		let fold_right_compose_append_f_empty = fold_right_compose_append_f(Monoid::empty());
	// 		fold_right_compose_append_f_empty(fa)
	// 	})
	// }

	// /// Folds the structure by applying a function from right to left.
	// ///
	// /// The default implementation of `fold_right` is implemented in terms of [`fold_map`] using the [`Endomorphism` monoid][`crate::types::Endomorphism`] where:
	// ///
	// /// `((fold_right f) b) fa = ((fold_map f) fa) b`
	// ///
	// /// # Type Signature
	// ///
	// /// `forall a b. Foldable f => (a -> b -> b) -> b -> f a -> b`
	// ///
	// /// # Parameters
	// ///
	// /// * `f`: A curried binary function that takes in the next item in the structure, the current value of the accumulator and returns the next value of accumulator.
	// /// * `b`: Initial value of type `B`.
	// /// * `fa`: A foldable structure containing values of type `A`.
	// ///
	// /// # Returns
	// ///
	// /// Final value of type `B` obtained from the folding operation.
	// fn fold_right<'a, ClonableFnBrand: ClonableFn + 'a, A, B: 'a + Clone>(
	// 	f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	// ) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>>
	// where
	// 	for<'b> ApplyFn<'a, ClonableFnBrand, B, B>:
	// 		HktMonoid<Output<'b> = ApplyFn<'a, ClonableFnBrand, B, B>>,
	// 	for<'b> ApplyFn<'a, ClonableFnBrand, B, B>: Monoid<'b>,
	// {
	// 	ClonableFnBrand::new::<'a, B, _>(move |b: B| {
	// 		ClonableFnBrand::new::<'a, _, B>({
	// 			let f = f.clone();
	// 			move |fa: Apply0L1T<Self, A>| {
	// 				let fold_map_f = Self::fold_map::<
	// 					'a,
	// 					ClonableFnBrand,
	// 					A,
	// 					ApplyFn<'a, ClonableFnBrand, B, B>,
	// 				>(f.clone());
	// 				let fold_map_f_fa = fold_map_f(fa);
	// 				let fold_map_f_fa_b = fold_map_f_fa(b.clone());
	// 				fold_map_f_fa_b
	// 			}
	// 		})
	// 	})
	// }
}
