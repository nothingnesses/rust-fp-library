use crate::{
	classes::{ClonableFn, Monoid, clonable_fn::ApplyFn},
	functions::{compose, flip, identity},
	hkt::{Apply0L1T, Kind0L1T},
	types::Endomorphism,
};

/// A type class for structures that can be folded to a single value.
///
/// A `Foldable` represents a structure that can be folded over to combine its elements
/// into a single result. This is useful for operations like summing values, collecting into a collection,
/// or applying monoidal operations.
///
/// A minimum implementation of `Foldable` requires the manual implementation of at least [`Foldable::fold_right`] or [`Foldable::fold_map`].
pub trait Foldable: Kind0L1T {
	/// Folds the structure by applying a function from left to right.
	///
	/// The default implementation of `fold_left` is implemented in terms of [`fold_right`], [`flip`], [`compose`] and [`identity`] where:
	///
	/// `((fold_left f) b) fa = (((fold_right (((compose (flip compose)) (flip f)))) identity) fa) b`
	///
	/// # Type Signature
	///
	/// `forall f a b. Foldable f => (b -> a -> b) -> b -> f a -> b`
	///
	/// # Parameters
	///
	/// * `f`: A curried binary function that takes in the current value of the accumulator, the next item in the structure and returns the next value of accumulator.
	/// * `b`: Initial value of type `B`.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final value of type `B` obtained from the folding operation.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::fold_left};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_left::<RcFnBrand, VecBrand, _, _>(Rc::new(|carry| Rc::new(move |item| carry * 2 + item)))(0)(vec![1, 2, 3]),
	///     11
	/// );
	/// ```
	fn fold_left<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			ClonableFnBrand::new({
				let f = f.clone();
				move |fa| {
					(((Self::fold_right::<ClonableFnBrand, _, _>(compose::<
						ClonableFnBrand,
						_,
						_,
						_,
					>(flip::<
						ClonableFnBrand,
						_,
						_,
						_,
					>(
						ClonableFnBrand::new(compose::<ClonableFnBrand, _, _, _>),
					))(flip::<
						ClonableFnBrand,
						_,
						_,
						_,
					>(f.clone()))))(ClonableFnBrand::new(identity)))(fa))(b.to_owned())
				}
			})
		})
	}

	/// Maps values to a monoid and combines them.
	///
	/// The default implementation of `fold_map` is implemented in terms of [`fold_right`], [`compose`], [`append`][crate::functions::append] and [`empty`][crate::functions::empty] where:
	///
	/// `fold_map f = (fold_right ((compose append) f)) empty`
	///
	/// # Type Signature
	///
	/// `forall f a m. Foldable f, Monoid m => (a -> m) -> f a -> m`
	///
	/// # Parameters
	///
	/// * `f`: A function that converts from values into monoidal elements.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final monoid obtained from the folding operation.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::{fold_map, identity}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, VecBrand, _, String>(Rc::new(identity))(vec![
	///         "Hello, ".to_string(),
	///         "World!".to_string()
	///     ]),
	///     "Hello, World!"
	/// );
	/// ```
	fn fold_map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, M: 'a + Monoid + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, M>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, M> {
		ClonableFnBrand::new(move |fa| {
			let f = f.clone();
			((Self::fold_right::<ClonableFnBrand, _, _>(ClonableFnBrand::new(move |a: A| {
				let f = f.clone();
				compose::<ClonableFnBrand, _, _, _>(ClonableFnBrand::new(
					M::append::<ClonableFnBrand>,
				))(f)(a.to_owned())
			})))(M::empty()))(fa)
		})
	}

	/// Folds the structure by applying a function from right to left.
	///
	/// The default implementation of `fold_right` is implemented in terms of [`fold_map`] using the [`Endomorphism` monoid][`crate::types::Endomorphism`] where:
	///
	/// `((fold_right f) b) fa = ((fold_map f) fa) b`
	///
	/// # Type Signature
	///
	/// `forall f a b. Foldable f => (a -> b -> b) -> b -> f a -> b`
	///
	/// # Parameters
	///
	/// * `f`: A curried binary function that takes in the next item in the structure, the current value of the accumulator and returns the next value of accumulator.
	/// * `b`: Initial value of type `B`.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final value of type `B` obtained from the folding operation.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::fold_right};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, VecBrand, _, _>(Rc::new(|item| Rc::new(move |carry| carry * 2 + item)))(0)(vec![1, 2, 3]),
	///     17
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			ClonableFnBrand::new({
				let f = f.clone();
				move |fa| {
					((Self::fold_map::<ClonableFnBrand, A, Endomorphism<'a, ClonableFnBrand, B>>(
						ClonableFnBrand::new({
							let f = f.clone();
							move |a| Endomorphism(f(a))
						}),
					)(fa))
					.0)(b.to_owned())
				}
			})
		})
	}
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_left`].
///
/// The default implementation of `fold_left` is implemented in terms of [`fold_right`], [`flip`], [`compose`] and [`identity`] where:
///
/// `((fold_left f) b) fa = (((fold_right (((compose (flip compose)) (flip f)))) identity) fa) b`
///
/// # Type Signature
///
/// `forall f a b. Foldable f => (b -> a -> b) -> b -> f a -> b`
///
/// # Parameters
///
/// * `f`: A curried binary function that takes in the current value of the accumulator, the next item in the structure and returns the next value of accumulator.
/// * `b`: Initial value of type `B`.
/// * `fa`: A foldable structure containing values of type `A`.
///
/// # Returns
///
/// Final value of type `B` obtained from the folding operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::fold_left};
/// use std::rc::Rc;
///
/// assert_eq!(
///     fold_left::<RcFnBrand, VecBrand, _, _>(Rc::new(|carry| Rc::new(move |item| carry * 2 + item)))(0)(vec![1, 2, 3]),
///     11
/// );
/// ```
pub fn fold_left<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: Foldable,
	A: 'a + Clone,
	B: 'a + Clone,
>(
	f: ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, A, B>>
) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, A>, B>> {
	Brand::fold_left::<ClonableFnBrand, _, _>(f)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_map`].
///
/// The default implementation of `fold_map` is implemented in terms of [`fold_right`], [`compose`], [`append`][crate::functions::append] and [`empty`][crate::functions::empty] where:
///
/// `fold_map f = (fold_right ((compose append) f)) empty`
///
/// # Type Signature
///
/// `forall f a m. Foldable f, Monoid m => (a -> m) -> f a -> m`
///
/// # Parameters
///
/// * `f`: A function that converts from values into monoidal elements.
/// * `fa`: A foldable structure containing values of type `A`.
///
/// # Returns
///
/// Final monoid obtained from the folding operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::{fold_map, identity}};
/// use std::rc::Rc;
///
/// assert_eq!(
///     fold_map::<RcFnBrand, VecBrand, _, String>(Rc::new(identity))(vec![
///         "Hello, ".to_string(),
///         "World!".to_string()
///     ]),
///     "Hello, World!"
/// );
/// ```
pub fn fold_map<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: Foldable,
	A: 'a + Clone,
	M: 'a + Monoid + Clone,
>(
	f: ApplyFn<'a, ClonableFnBrand, A, M>
) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, A>, M> {
	Brand::fold_map::<ClonableFnBrand, _, M>(f)
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_right`].
///
/// The default implementation of `fold_right` is implemented in terms of [`fold_map`] using the [`Endomorphism` monoid][`crate::types::Endomorphism`] where:
///
/// `((fold_right f) b) fa = ((fold_map f) fa) b`
///
/// # Type Signature
///
/// `forall f a b. Foldable f => (a -> b -> b) -> b -> f a -> b`
///
/// # Parameters
///
/// * `f`: A curried binary function that takes in the next item in the structure, the current value of the accumulator and returns the next value of accumulator.
/// * `b`: Initial value of type `B`.
/// * `fa`: A foldable structure containing values of type `A`.
///
/// # Returns
///
/// Final value of type `B` obtained from the folding operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::fold_right};
/// use std::rc::Rc;
///
/// assert_eq!(
///     fold_right::<RcFnBrand, VecBrand, _, _>(Rc::new(|item| Rc::new(move |carry| carry * 2 + item)))(0)(vec![1, 2, 3]),
///     17
/// );
/// ```
pub fn fold_right<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: Foldable,
	A: 'a + Clone,
	B: 'a + Clone,
>(
	f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, A>, B>> {
	Brand::fold_right::<ClonableFnBrand, _, _>(f)
}
