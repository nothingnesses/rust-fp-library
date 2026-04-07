//! Thread-safe wrapper for endofunctions (functions `a -> a`) with [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances based on function composition.
//!
//! Used to treat function composition as a monoidal operation in thread-safe contexts where [`append`](crate::functions::append) composes functions and [`empty`](crate::functions::empty) is the identity function.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				send_clone_fn::SendLiftFn,
				*,
			},
			functions::identity,
		},
		fp_macros::*,
		std::fmt::{
			self,
			Debug,
			Formatter,
		},
	};

	/// A thread-safe wrapper for endofunctions that enables monoidal operations.
	///
	/// `SendEndofunction a` represents a function `a -> a` wrapped in an `Arc<dyn Fn>`.
	///
	/// It exists to provide a monoid instance where:
	///
	/// * The binary operation [append][Semigroup::append] is function composition.
	/// * The identity element [empty][Monoid::empty] is the identity function.
	///
	/// This is the `Send + Sync` counterpart of [`Endofunction`](crate::types::Endofunction).
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the thread-safe cloneable function wrapper.",
		"The input and output type of the function."
	)]
	///
	pub struct SendEndofunction<'a, FnBrand: SendLiftFn, A: 'a>(
		/// The wrapped function.
		pub <FnBrand as SendCloneFn>::Of<'a, A, A>,
	);

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `ArcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: SendLiftFn, A: 'a> SendEndofunction<'a, FnBrand, A> {
		/// Creates a new `SendEndofunction`.
		#[document_signature]
		///
		#[document_parameters("The function to wrap.")]
		///
		#[document_returns("A new `SendEndofunction`.")]
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
		/// 	SendEndofunction::<ArcFnBrand, _>::new(send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| {
		/// 		x * 2
		/// 	}));
		/// assert_eq!(f.0(5), 10);
		/// ```
		pub fn new(f: <FnBrand as SendCloneFn>::Of<'a, A, A>) -> Self {
			Self(f)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `ArcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to clone.")]
	impl<'a, FnBrand: SendLiftFn, A: 'a> Clone for SendEndofunction<'a, FnBrand, A> {
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
		/// 	SendEndofunction::<ArcFnBrand, _>::new(send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| {
		/// 		x * 2
		/// 	}));
		/// let cloned = f.clone();
		/// assert_eq!(cloned.0(5), 10);
		/// ```
		fn clone(&self) -> Self {
			Self::new(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `ArcFnBrand`).",
		"The input and output type of the function."
	)]
	#[document_parameters("The function to format.")]
	impl<'a, FnBrand: SendLiftFn, A: 'a> Debug for SendEndofunction<'a, FnBrand, A>
	where
		<FnBrand as SendCloneFn>::Of<'a, A, A>: Debug,
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
		/// 	SendEndofunction::<ArcFnBrand, _>::new(send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| {
		/// 		x * 2
		/// 	}));
		/// assert_eq!(f.0(5), 10);
		/// ```
		fn fmt(
			&self,
			fmt: &mut Formatter<'_>,
		) -> fmt::Result {
			fmt.debug_tuple("SendEndofunction").field(&self.0).finish()
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `ArcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: 'a + SendLiftFn, A: 'a + Send + Sync> Semigroup
		for SendEndofunction<'a, FnBrand, A>
	{
		/// Composes two endofunctions.
		///
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
		/// let f =
		/// 	SendEndofunction::<ArcFnBrand, _>::new(send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| {
		/// 		x * 2
		/// 	}));
		/// let g =
		/// 	SendEndofunction::<ArcFnBrand, _>::new(send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| {
		/// 		x + 1
		/// 	}));
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
			Self::new(<FnBrand as SendLiftFn>::new(move |x| f(g(x))))
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the function (e.g., `ArcFnBrand`).",
		"The input and output type of the function."
	)]
	impl<'a, FnBrand: 'a + SendLiftFn, A: 'a + Send + Sync> Monoid
		for SendEndofunction<'a, FnBrand, A>
	{
		/// Returns the identity endofunction.
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
		/// let id = empty::<SendEndofunction<ArcFnBrand, i32>>();
		/// assert_eq!(id.0(5), 5);
		/// ```
		fn empty() -> Self {
			Self::new(<FnBrand as SendLiftFn>::new(identity))
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::ArcFnBrand,
			classes::{
				send_clone_fn::SendLiftFn,
				*,
			},
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	#[quickcheck]
	fn semigroup_associativity(val: i32) -> bool {
		let f =
			SendEndofunction::<ArcFnBrand, _>::new(<ArcFnBrand as SendLiftFn>::new(|x: i32| {
				x.wrapping_add(1)
			}));
		let g =
			SendEndofunction::<ArcFnBrand, _>::new(<ArcFnBrand as SendLiftFn>::new(|x: i32| {
				x.wrapping_mul(2)
			}));
		let h =
			SendEndofunction::<ArcFnBrand, _>::new(<ArcFnBrand as SendLiftFn>::new(|x: i32| {
				x.wrapping_sub(3)
			}));

		let lhs = append(f.clone(), append(g.clone(), h.clone()));
		let rhs = append(append(f, g), h);

		lhs.0(val) == rhs.0(val)
	}

	#[quickcheck]
	fn monoid_left_identity(val: i32) -> bool {
		let f =
			SendEndofunction::<ArcFnBrand, _>::new(<ArcFnBrand as SendLiftFn>::new(|x: i32| {
				x.wrapping_add(1)
			}));
		let id = empty::<SendEndofunction<ArcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(val) == f.0(val)
	}

	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f =
			SendEndofunction::<ArcFnBrand, _>::new(<ArcFnBrand as SendLiftFn>::new(|x: i32| {
				x.wrapping_add(1)
			}));
		let id = empty::<SendEndofunction<ArcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(val) == f.0(val)
	}
}
