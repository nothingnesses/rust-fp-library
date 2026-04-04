//! Wrapper for endofunctions (functions `a -> a`) with [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances based on function composition.
//!
//! Used to treat function composition as a monoidal operation where [`append`](crate::functions::append) composes functions and [`empty`](crate::functions::empty) is the identity function.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			functions::identity,
		},
		fp_macros::*,
		std::{
			fmt::{
				self,
				Debug,
				Formatter,
			},
			hash::Hash,
		},
	};

	/// A wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.
	///
	/// `Endofunction a` represents a function `a -> a`.
	///
	/// It exists to provide a monoid instance where:
	///
	/// * The binary operation [append][Semigroup::append] is [function composition][crate::functions::compose].
	/// * The identity element [empty][Monoid::empty] is the [identity function][crate::functions::identity].
	///
	/// The wrapped function can be accessed directly via the [`.0` field][Endofunction#structfield.0].
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the cloneable function wrapper.",
		"The input and output type of the function."
	)]
	///
	pub struct Endofunction<'a, FnBrand: LiftFn, A: 'a>(
		/// The wrapped function.
		pub <FnBrand as CloneableFn>::Of<'a, A, A>,
	);

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: LiftFn, A: 'a> Endofunction<'a, FnBrand, A> {
		/// Creates a new `Endofunction`.
		///
		/// This function wraps a function `a -> a` in an `Endofunction` struct.
		#[document_signature]
		///
		#[document_parameters("The function to wrap.")]
		///
		#[document_returns("A new `Endofunction`.")]
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
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(f.0(5), 10);
		/// ```
		pub fn new(f: <FnBrand as CloneableFn>::Of<'a, A, A>) -> Self {
			Self(f)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to clone.")]
	impl<'a, FnBrand: LiftFn, A: 'a> Clone for Endofunction<'a, FnBrand, A> {
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
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let cloned = f.clone();
		/// assert_eq!(cloned.0(5), 10);
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
	impl<'a, FnBrand: LiftFn, A: 'a> Debug for Endofunction<'a, FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<'a, A, A>: Debug,
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
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // Debug formatting is available when the inner function type implements Debug.
		/// // Verify the endofunction applies correctly:
		/// assert_eq!(f.0(5), 10);
		/// ```
		fn fmt(
			&self,
			fmt: &mut Formatter<'_>,
		) -> fmt::Result {
			fmt.debug_tuple("Endofunction").field(&self.0).finish()
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: LiftFn, A: 'a> Eq for Endofunction<'a, FnBrand, A> where
		<FnBrand as CloneableFn>::Of<'a, A, A>: Eq
	{
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to hash.")]
	impl<'a, FnBrand: LiftFn, A: 'a> Hash for Endofunction<'a, FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<'a, A, A>: Hash,
	{
		#[document_signature]
		#[document_type_parameters("The type of the hasher.")]
		#[document_parameters("The hasher state to update.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // Hash is available when the inner function type implements Hash.
		/// // Verify the endofunction applies correctly:
		/// assert_eq!(f.0(5), 10);
		/// ```
		fn hash<H: std::hash::Hasher>(
			&self,
			state: &mut H,
		) {
			self.0.hash(state);
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<'a, FnBrand: LiftFn, A: 'a> Ord for Endofunction<'a, FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<'a, A, A>: Ord,
	{
		#[document_signature]
		#[document_parameters("The other function to compare to.")]
		#[document_returns("The ordering of the values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // Ord is available when the inner function type implements Ord.
		/// // Both produce the same output for the same input:
		/// assert_eq!(f.0(5), g.0(5));
		/// ```
		fn cmp(
			&self,
			other: &Self,
		) -> std::cmp::Ordering {
			self.0.cmp(&other.0)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<'a, FnBrand: LiftFn, A: 'a> PartialEq for Endofunction<'a, FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<'a, A, A>: PartialEq,
	{
		#[document_signature]
		#[document_parameters("The other function to compare to.")]
		#[document_returns("True if the values are equal, false otherwise.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // PartialEq is available when the inner function type implements PartialEq.
		/// // Both produce the same output for the same input:
		/// assert_eq!(f.0(5), g.0(5));
		/// ```
		fn eq(
			&self,
			other: &Self,
		) -> bool {
			self.0 == other.0
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<'a, FnBrand: LiftFn, A: 'a> PartialOrd for Endofunction<'a, FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<'a, A, A>: PartialOrd,
	{
		#[document_signature]
		#[document_parameters("The other function to compare to.")]
		#[document_returns("An ordering if the values can be compared, none otherwise.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // PartialOrd is available when the inner function type implements PartialOrd.
		/// // Both produce the same output for the same input:
		/// assert_eq!(f.0(5), g.0(5));
		/// ```
		fn partial_cmp(
			&self,
			other: &Self,
		) -> Option<std::cmp::Ordering> {
			self.0.partial_cmp(&other.0)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: 'a + LiftFn, A: 'a> Semigroup for Endofunction<'a, FnBrand, A> {
		/// The result of combining the two values using the semigroup operation.
		///
		/// This method composes two endofunctions into a single endofunction.
		/// Note that `Endofunction` composition is reversed relative to standard function composition:
		/// `append(f, g)` results in `f . g` (read as "f after g"), meaning `g` is applied first, then `f`.
		#[document_signature]
		///
		#[document_parameters(
			"The second function to apply (the outer function).",
			"The first function to apply (the inner function)."
		)]
		///
		#[document_returns("The composed function `a . b`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endofunction::<RcFnBrand, _>::new(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
		///
		/// // f(g(x)) = (x + 1) * 2
		/// let h = append::<_>(f, g);
		/// assert_eq!(h.0(5), 12);
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			let f = a.0;
			let g = b.0;
			// Compose: f . g
			Self::new(<FnBrand as LiftFn>::new(move |x| f(g(x))))
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: 'a + LiftFn, A: 'a> Monoid for Endofunction<'a, FnBrand, A> {
		/// The identity element.
		///
		/// This method returns the identity endofunction, which wraps the identity function.
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
		/// let id = empty::<Endofunction<RcFnBrand, i32>>();
		/// assert_eq!(id.0(5), 5);
		/// ```
		fn empty() -> Self {
			Self::new(<FnBrand as LiftFn>::new(identity))
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::RcFnBrand,
			classes::*,
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as LiftFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let g = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as LiftFn>::new(|x: i32| {
			x.wrapping_mul(2)
		}));
		let h = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as LiftFn>::new(|x: i32| {
			x.wrapping_sub(3)
		}));

		let lhs = append(f.clone(), append(g.clone(), h.clone()));
		let rhs = append(append(f, g), h);

		lhs.0(val) == rhs.0(val)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as LiftFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(val) == f.0(val)
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as LiftFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(val) == f.0(val)
	}
}
