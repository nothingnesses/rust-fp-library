//! Thread-safe by-value mapping with [`send_map`].
//!
//! Like [`Functor::map`](crate::classes::Functor::map), but requires
//! `Send + Sync` on the closure (and on `A`, `B`) so the result can cross
//! thread boundaries. By-value parallel of
//! [`SendRefFunctor`](crate::classes::SendRefFunctor).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
//! let mapped = send_map::<ArcCoyonedaBrand<VecBrand>, _, _>(|x: i32| x * 2, coyo);
//! assert_eq!(mapped.lower_ref(), vec![2, 4, 6]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for thread-safe by-value mapping.
	///
	/// This is the thread-safe by-value counterpart of
	/// [`Functor`](crate::classes::Functor). The `Send + Sync` bound on
	/// the closure parameter ensures the closure can be stored in
	/// thread-safe containers like `Arc<dyn Fn + Send + Sync>` (matching
	/// what [`FnBrand<ArcBrand>`](crate::brands::FnBrand) resolves to via
	/// [`SendCloneFn`](crate::classes::SendCloneFn)).
	///
	/// ### Why a Separate Trait?
	///
	/// A single trait with `Send + Sync` bounds on `Functor` would exclude
	/// `RcCoyoneda`, which uses `Rc` (a `!Send` type). By keeping `Functor`
	/// free of thread-safety bounds and providing `SendFunctor` separately,
	/// `RcCoyoneda` can implement `Functor` while `ArcCoyoneda` implements
	/// only `SendFunctor`. This mirrors the
	/// [`CloneFn`](crate::classes::CloneFn) /
	/// [`SendCloneFn`](crate::classes::SendCloneFn) split.
	///
	/// ### Laws
	///
	/// `SendFunctor` instances must satisfy the standard functor laws:
	///
	/// * Identity: `send_map(|x| x, fa)` produces a value equal to `fa`.
	/// * Composition: `send_map(|x| g(f(x)), fa)` produces the same value as
	///   `send_map(g, send_map(f, fa))`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendFunctor {
		/// Maps a thread-safe function over the values in the functor context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor. Must be `Send + Sync`.",
			"The type of the result(s) of applying the function. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor. Must be `Send + Sync`.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
		/// let mapped = ArcCoyonedaBrand::<VecBrand>::send_map(|x: i32| x * 2, coyo);
		/// assert_eq!(mapped.lower_ref(), vec![2, 4, 6]);
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			func: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Maps a thread-safe function over the values in the functor context.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendFunctor::send_map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
	/// let mapped = send_map::<ArcCoyonedaBrand<VecBrand>, _, _>(|x: i32| x * 2, coyo);
	/// assert_eq!(mapped.lower_ref(), vec![2, 4, 6]);
	/// ```
	pub fn send_map<'a, Brand: SendFunctor, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		func: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_map(func, fa)
	}
}

pub use inner::*;
