//! Wrapper for endofunctions (functions `a -> a`) with [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances based on function composition.
//!
//! Used to treat function composition as a monoidal operation where [`append`](crate::functions::append) composes functions and [`empty`](crate::functions::empty) is the identity function.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{CloneableFn, Monoid, Semigroup},
			functions::identity,
		},
		fp_macros::{document_fields, document_parameters, document_type_parameters},
		std::{
			fmt::{self, Debug, Formatter},
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
		"The brand of the cloneable function wrapper.",
		"The input and output type of the function."
	)]
	///
	#[document_fields("The wrapped function.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let f = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(f.0(5), 10);
	/// ```
	pub struct Endofunction<FnBrand: CloneableFn, A>(
		pub <FnBrand as CloneableFn>::Of<A, A>,
	);

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<FnBrand: CloneableFn, A> Endofunction<FnBrand, A> {
		/// Creates a new `Endofunction`.
		///
		/// This function wraps a function `a -> a` in an `Endofunction` struct.
		#[document_signature]
		///
		#[document_parameters("The function to wrap.")]
		///
		/// ### Returns
		///
		/// A new `Endofunction`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(f.0(5), 10);
		/// ```
		pub fn new(f: <FnBrand as CloneableFn>::Of<A, A>) -> Self {
			Self(f)
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to clone.")]
	impl<FnBrand: CloneableFn, A> Clone for Endofunction<FnBrand, A> {
		#[document_signature]
		fn clone(&self) -> Self {
			Self::new(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to format.")]
	impl<FnBrand: CloneableFn, A> Debug for Endofunction<FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<A, A>: Debug,
	{
		#[document_signature]
		#[document_parameters("The formatter to use.")]
		fn fmt(
			&self,
			fmt: &mut Formatter<'_>,
		) -> fmt::Result {
			fmt.debug_tuple("Endofunction").field(&self.0).finish()
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<FnBrand: CloneableFn, A> Eq for Endofunction<FnBrand, A> where
		<FnBrand as CloneableFn>::Of<A, A>: Eq
	{
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to hash.")]
	impl<FnBrand: CloneableFn, A> Hash for Endofunction<FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<A, A>: Hash,
	{
		#[document_signature]
		#[document_type_parameters("The type of the hasher.")]
		#[document_parameters("The hasher state to update.")]
		fn hash<H: std::hash::Hasher>(
			&self,
			state: &mut H,
		) {
			self.0.hash(state);
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<FnBrand: CloneableFn, A> Ord for Endofunction<FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<A, A>: Ord,
	{
		#[document_signature]
		#[document_parameters("The other function to compare to.")]
		fn cmp(
			&self,
			other: &Self,
		) -> std::cmp::Ordering {
			self.0.cmp(&other.0)
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<FnBrand: CloneableFn, A> PartialEq for Endofunction<FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<A, A>: PartialEq,
	{
		#[document_signature]
		#[document_parameters("The other function to compare to.")]
		fn eq(
			&self,
			other: &Self,
		) -> bool {
			self.0 == other.0
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<FnBrand: CloneableFn, A> PartialOrd for Endofunction<FnBrand, A>
	where
		<FnBrand as CloneableFn>::Of<A, A>: PartialOrd,
	{
		#[document_signature]
		#[document_parameters("The other function to compare to.")]
		fn partial_cmp(
			&self,
			other: &Self,
		) -> Option<std::cmp::Ordering> {
			self.0.partial_cmp(&other.0)
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<FnBrand: CloneableFn, A> Semigroup for Endofunction<FnBrand, A> {
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
		/// ### Returns
		///
		/// The composed function `a . b`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
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
			Self::new(<FnBrand as CloneableFn>::new(move |x| f(g(x))))
		}
	}

	#[document_type_parameters(
		"The brand of the function (e.g., `RcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<FnBrand: CloneableFn, A> Monoid for Endofunction<FnBrand, A> {
		/// The identity element.
		///
		/// This method returns the identity endofunction, which wraps the identity function.
		#[document_signature]
		///
		/// ### Returns
		///
		/// The identity endofunction.
		///
		/// ### Examples
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
			Self::new(<FnBrand as CloneableFn>::new(identity))
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
			classes::{cloneable_fn::CloneableFn, monoid::empty, semigroup::append},
		},
		quickcheck_macros::quickcheck,
	};

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let g = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_mul(2)
		}));
		let h = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
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
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(val) == f.0(val)
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(val) == f.0(val)
	}
}
