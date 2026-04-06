//! Wrapper for by-reference endofunctions (functions `&a -> a`) with [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances based on function composition.
//!
//! Used by [`RefFoldable`](crate::classes::RefFoldable)'s default implementations
//! to derive `ref_fold_map` from `ref_fold_right` (and vice versa) via monoidal
//! composition, mirroring how [`Endofunction`](crate::types::Endofunction) is used
//! by [`Foldable`](crate::classes::Foldable).

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::{
			dispatch::Ref,
			*,
		},
		fp_macros::*,
		std::fmt::{
			self,
			Debug,
			Formatter,
		},
	};

	/// A wrapper for by-reference endofunctions (functions `&a -> a`) that enables monoidal operations.
	///
	/// `RefEndofunction a` represents a function `&a -> a`.
	///
	/// It exists to provide a monoid instance where:
	///
	/// * The binary operation [append][Semigroup::append] is function composition: `|x: &A| f(&g(x))`.
	/// * The identity element [empty][Monoid::empty] is `|x: &A| x.clone()` (requires `A: Clone`).
	///
	/// The wrapped function can be accessed directly via the [`.0` field][RefEndofunction#structfield.0].
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the cloneable function wrapper.",
		"The input and output type of the function."
	)]
	///
	pub struct RefEndofunction<'a, FnBrand: LiftRefFn, A: 'a>(
		/// The wrapped function.
		pub <FnBrand as CloneFn<Ref>>::Of<'a, A, A>,
	);

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: LiftRefFn, A: 'a> RefEndofunction<'a, FnBrand, A> {
		/// Creates a new `RefEndofunction`.
		///
		/// This function wraps a function `&a -> a` in a `RefEndofunction` struct.
		#[document_signature]
		///
		#[document_parameters("The function to wrap.")]
		///
		#[document_returns("A new `RefEndofunction`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f =
		/// 	RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| *x * 2));
		/// assert_eq!(f.0(&5), 10);
		/// ```
		pub fn new(f: <FnBrand as CloneFn<Ref>>::Of<'a, A, A>) -> Self {
			Self(f)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to clone.")]
	impl<'a, FnBrand: LiftRefFn, A: 'a> Clone for RefEndofunction<'a, FnBrand, A> {
		#[document_signature]
		#[document_returns("The cloned endofunction.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f =
		/// 	RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| *x * 2));
		/// let cloned = f.clone();
		/// assert_eq!(cloned.0(&5), 10);
		/// ```
		fn clone(&self) -> Self {
			Self::new(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to format.")]
	impl<'a, FnBrand: LiftRefFn, A: 'a> Debug for RefEndofunction<'a, FnBrand, A>
	where
		<FnBrand as CloneFn<Ref>>::Of<'a, A, A>: Debug,
	{
		#[document_signature]
		#[document_parameters("The formatter to use.")]
		#[document_returns("The result of the formatting operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f =
		/// 	RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| *x * 2));
		/// assert_eq!(f.0(&5), 10);
		/// ```
		fn fmt(
			&self,
			fmt: &mut Formatter<'_>,
		) -> fmt::Result {
			fmt.debug_tuple("RefEndofunction").field(&self.0).finish()
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: 'a + LiftRefFn, A: 'a> Semigroup for RefEndofunction<'a, FnBrand, A> {
		/// The result of combining the two values using the semigroup operation.
		///
		/// This method composes two by-reference endofunctions into a single endofunction.
		/// `append(f, g)` results in `|x: &A| f(&g(x))` (g is applied first, then f).
		#[document_signature]
		///
		#[document_parameters(
			"The second function to apply (the outer function).",
			"The first function to apply (the inner function)."
		)]
		///
		#[document_returns("The composed function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f =
		/// 	RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| *x * 2));
		/// let g =
		/// 	RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| *x + 1));
		///
		/// // f(g(x)) = (x + 1) * 2
		/// let h = append::<_>(f, g);
		/// assert_eq!(h.0(&5), 12);
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			let f = a.0;
			let g = b.0;
			// Compose: |x: &A| f(&g(x))
			Self::new(<FnBrand as LiftRefFn>::new(move |x: &A| f(&g(x))))
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: 'a + LiftRefFn, A: Clone + 'a> Monoid for RefEndofunction<'a, FnBrand, A> {
		/// The identity element.
		///
		/// This method returns the identity endofunction, which clones its input.
		/// Requires `A: Clone` since the identity for `&A -> A` must produce an owned value.
		#[document_signature]
		///
		#[document_returns("The identity endofunction.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let id = empty::<RefEndofunction<RcFnBrand, i32>>();
		/// assert_eq!(id.0(&5), 5);
		/// ```
		fn empty() -> Self {
			Self::new(<FnBrand as LiftRefFn>::new(|x: &A| x.clone()))
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::*,
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(val: i32) -> bool {
		let f =
			RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| {
				x.wrapping_add(1)
			}));
		let g =
			RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| {
				x.wrapping_mul(2)
			}));
		let h =
			RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| {
				x.wrapping_sub(3)
			}));

		let lhs = append(f.clone(), append(g.clone(), h.clone()));
		let rhs = append(append(f, g), h);

		lhs.0(&val) == rhs.0(&val)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(val: i32) -> bool {
		let f =
			RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| {
				x.wrapping_add(1)
			}));
		let id = empty::<RefEndofunction<RcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(&val) == f.0(&val)
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f =
			RefEndofunction::<RcFnBrand, _>::new(lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| {
				x.wrapping_add(1)
			}));
		let id = empty::<RefEndofunction<RcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(&val) == f.0(&val)
	}
}
