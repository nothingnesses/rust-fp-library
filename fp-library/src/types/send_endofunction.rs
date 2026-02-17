//! Thread-safe wrapper for endofunctions with [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances.
//!
//! The `Send + Sync` counterpart to [`Endofunction`](crate::types::Endofunction), wrapping functions that can be safely shared across threads.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{Monoid, Semigroup, SendCloneableFn},
			functions::identity,
		},
		fp_macros::{document_fields, document_parameters, document_type_parameters},
		std::{
			fmt::{self, Debug, Formatter},
			hash::Hash,
		},
	};

	/// A thread-safe wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.
	///
	/// `SendEndofunction a` represents a function `a -> a` that is `Send + Sync`.
	///
	/// It exists to provide a monoid instance where:
	///
	/// * The binary operation [append][Semigroup::append] is [function composition][crate::functions::compose].
	/// * The identity element [empty][Monoid::empty] is the [identity function][crate::functions::identity].
	///
	/// The wrapped function can be accessed directly via the [`.0` field][SendEndofunction#structfield.0].
	#[document_type_parameters(
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	///
	#[document_fields("The wrapped thread-safe function.")]
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
	/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(
	/// 	|x: i32| x * 2,
	/// ));
	/// assert_eq!(f.0(5), 10);
	/// ```
	pub struct SendEndofunction<FnBrand: SendCloneableFn, A>(
		pub <FnBrand as SendCloneableFn>::SendOf<A, A>,
	);

	#[document_type_parameters(
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	impl<FnBrand: SendCloneableFn, A> SendEndofunction<FnBrand, A> {
		/// Creates a new `SendEndofunction`.
		///
		/// This function wraps a thread-safe function `a -> a` in a `SendEndofunction` struct.
		#[document_signature]
		///
		#[document_parameters("The function to wrap.")]
		///
		/// ### Returns
		///
		/// A new `SendEndofunction`.
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
		/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(
		/// 	|x: i32| x * 2,
		/// ));
		/// assert_eq!(f.0(5), 10);
		/// ```
		pub fn new(f: <FnBrand as SendCloneableFn>::SendOf<A, A>) -> Self {
			Self(f)
		}
	}

	#[document_type_parameters(
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to clone.")]
	impl<FnBrand: SendCloneableFn, A> Clone for SendEndofunction<FnBrand, A> {
		#[document_signature]
		fn clone(&self) -> Self {
			Self::new(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to format.")]
	impl<FnBrand: SendCloneableFn, A> Debug for SendEndofunction<FnBrand, A>
	where
		<FnBrand as SendCloneableFn>::SendOf<A, A>: Debug,
	{
		#[document_signature]
		#[document_parameters("The formatter to use.")]
		fn fmt(
			&self,
			fmt: &mut Formatter<'_>,
		) -> fmt::Result {
			fmt.debug_tuple("SendEndofunction").field(&self.0).finish()
		}
	}

	#[document_type_parameters(
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	impl<FnBrand: SendCloneableFn, A> Eq for SendEndofunction<FnBrand, A> where
		<FnBrand as SendCloneableFn>::SendOf<A, A>: Eq
	{
	}

	#[document_type_parameters(
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to hash.")]
	impl<FnBrand: SendCloneableFn, A> Hash for SendEndofunction<FnBrand, A>
	where
		<FnBrand as SendCloneableFn>::SendOf<A, A>: Hash,
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
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<FnBrand: SendCloneableFn, A> Ord for SendEndofunction<FnBrand, A>
	where
		<FnBrand as SendCloneableFn>::SendOf<A, A>: Ord,
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
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<FnBrand: SendCloneableFn, A> PartialEq for SendEndofunction<FnBrand, A>
	where
		<FnBrand as SendCloneableFn>::SendOf<A, A>: PartialEq,
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
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to compare.")]
	impl<FnBrand: SendCloneableFn, A> PartialOrd for SendEndofunction<FnBrand, A>
	where
		<FnBrand as SendCloneableFn>::SendOf<A, A>: PartialOrd,
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
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	impl<FnBrand: SendCloneableFn, A: Send + Sync> Semigroup
		for SendEndofunction<FnBrand, A>
	{
		/// The result of combining the two values using the semigroup operation.
		///
		/// This method combines two endofunctions into a single endofunction.
		/// Note that `SendEndofunction` composition is reversed relative to standard function composition:
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
		/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(
		/// 	|x: i32| x * 2,
		/// ));
		/// let g = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(
		/// 	|x: i32| x + 1,
		/// ));
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
			Self::new(<FnBrand as SendCloneableFn>::send_cloneable_fn_new(move |x| f(g(x))))
		}
	}

	#[document_type_parameters(
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	impl<FnBrand: SendCloneableFn, A: Send + Sync> Monoid
		for SendEndofunction<FnBrand, A>
	{
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
		/// let id = empty::<SendEndofunction<ArcFnBrand, i32>>();
		/// assert_eq!(id.0(5), 5);
		/// ```
		fn empty() -> Self {
			Self::new(<FnBrand as SendCloneableFn>::send_cloneable_fn_new(identity))
		}
	}
}

pub use inner::*;
