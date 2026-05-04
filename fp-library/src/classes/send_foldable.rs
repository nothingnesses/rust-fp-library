//! Thread-safe by-value variant of [`Foldable`](crate::classes::Foldable).
//!
//! Like [`Foldable`](crate::classes::Foldable), but requires `Send + Sync`
//! on the closure (and on `A`, `M`, `B`) so the fold can be driven from a
//! spawned thread. By-value parallel of
//! [`SendRefFoldable`](crate::classes::SendRefFoldable).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::send_foldable::*,
//! };
//!
//! let xs = vec![1, 2, 3];
//! let result = send_fold_map::<ArcFnBrand, VecBrand, _, _>(|a: i32| a.to_string(), xs);
//! assert_eq!(result, "123");
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				send_clone_fn::SendLiftFn,
				*,
			},
			kinds::*,
			types::{
				Dual,
				SendEndofunction,
			},
		},
		fp_macros::*,
	};

	/// A type class for thread-safe by-value folding.
	///
	/// This is the thread-safe by-value counterpart of
	/// [`Foldable`](crate::classes::Foldable). The `Send + Sync` bounds on
	/// the closure parameter and the elements ensure the fold can be driven
	/// from a spawned thread.
	///
	/// ### Why a Separate Trait?
	///
	/// A single trait with `Send + Sync` bounds on `Foldable` would exclude
	/// brands whose representation is not thread-safe. By keeping `Foldable`
	/// free of thread-safety bounds and providing `SendFoldable` separately,
	/// non-thread-safe brands can implement `Foldable` while thread-safe-only
	/// brands implement `SendFoldable`. This mirrors the
	/// [`Functor`](crate::classes::Functor) /
	/// [`SendFunctor`](crate::classes::SendFunctor) split.
	///
	/// All three methods (`send_fold_map`, `send_fold_right`, `send_fold_left`)
	/// have default implementations in terms of each other, so implementors
	/// only need to provide one.
	///
	/// ### Laws
	///
	/// `SendFoldable` instances must be internally consistent:
	/// * fold_map/fold_right consistency:
	///   `send_fold_map(f, fa) = send_fold_right(|a, m| append(f(a), m), empty(), fa)`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendFoldable {
		/// Maps values to a monoid and combines them (thread-safe).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure. Must be `Send + Sync`.",
			"The type of the monoid. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The function to map each element to a monoid. Must be `Send + Sync`.",
			"The structure to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_foldable::SendFoldable,
		/// };
		///
		/// let xs = vec![1, 2, 3];
		/// let result =
		/// 	<VecBrand as SendFoldable>::send_fold_map::<ArcFnBrand, _, _>(|a: i32| a.to_string(), xs);
		/// assert_eq!(result, "123");
		/// ```
		fn send_fold_map<'a, FnBrand, A: Send + Sync + 'a + Clone, M>(
			func: impl Fn(A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: SendLiftFn + 'a,
			M: Monoid + Send + Sync + 'a, {
			Self::send_fold_right::<FnBrand, A, M>(
				move |a: A, acc| Semigroup::append(func(a), acc),
				Monoid::empty(),
				fa,
			)
		}

		/// Folds the structure from the right (thread-safe).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure. Must be `Send + Sync`.",
			"The type of the accumulator. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator. Must be `Send + Sync`.",
			"The initial value of the accumulator.",
			"The structure to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_foldable::SendFoldable,
		/// };
		///
		/// let xs = vec![1, 2, 3];
		/// let result = <VecBrand as SendFoldable>::send_fold_right::<ArcFnBrand, _, _>(
		/// 	|a: i32, b: i32| a + b,
		/// 	0,
		/// 	xs,
		/// );
		/// assert_eq!(result, 6);
		/// ```
		fn send_fold_right<'a, FnBrand, A: Send + Sync + 'a + Clone, B: Send + Sync + 'a>(
			func: impl Fn(A, B) -> B + Send + Sync + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: SendLiftFn + 'a, {
			let f = <FnBrand as SendLiftFn>::new(move |(a, b): (A, B)| func(a, b));
			let m = Self::send_fold_map::<FnBrand, A, SendEndofunction<FnBrand, B>>(
				move |a: A| {
					let f = f.clone();
					SendEndofunction::<FnBrand, B>::new(<FnBrand as SendLiftFn>::new(move |b| {
						let a = a.clone();
						f((a, b))
					}))
				},
				fa,
			);
			m.0(initial)
		}

		/// Folds the structure from the left (thread-safe).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure. Must be `Send + Sync`.",
			"The type of the accumulator. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element. Must be `Send + Sync`.",
			"The initial value of the accumulator.",
			"The structure to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_foldable::SendFoldable,
		/// };
		///
		/// let xs = vec![1, 2, 3];
		/// let result = <VecBrand as SendFoldable>::send_fold_left::<ArcFnBrand, _, _>(
		/// 	|b: i32, a: i32| b + a,
		/// 	0,
		/// 	xs,
		/// );
		/// assert_eq!(result, 6);
		/// ```
		fn send_fold_left<'a, FnBrand, A: Send + Sync + 'a + Clone, B: Send + Sync + 'a>(
			func: impl Fn(B, A) -> B + Send + Sync + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: SendLiftFn + 'a, {
			let f = <FnBrand as SendLiftFn>::new(move |(b, a): (B, A)| func(b, a));
			let m = Self::send_fold_map::<FnBrand, A, Dual<SendEndofunction<FnBrand, B>>>(
				move |a: A| {
					let f = f.clone();
					Dual(SendEndofunction::<FnBrand, B>::new(<FnBrand as SendLiftFn>::new(
						move |b| {
							let a = a.clone();
							f((b, a))
						},
					)))
				},
				fa,
			);
			(m.0).0(initial)
		}
	}

	/// Maps values to a monoid and combines them (thread-safe).
	///
	/// Free function version that dispatches to [the type class' associated function][`SendFoldable::send_fold_map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the cloneable function to use.",
		"The brand of the structure.",
		"The type of the elements.",
		"The type of the monoid."
	)]
	///
	#[document_parameters(
		"The function to map each element to a monoid.",
		"The structure to fold."
	)]
	///
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::send_foldable::*,
	/// };
	///
	/// let xs = vec![1, 2, 3];
	/// let result = send_fold_map::<ArcFnBrand, VecBrand, _, _>(|a: i32| a.to_string(), xs);
	/// assert_eq!(result, "123");
	/// ```
	pub fn send_fold_map<
		'a,
		FnBrand: SendLiftFn + 'a,
		Brand: SendFoldable,
		A: Send + Sync + 'a + Clone,
		M,
	>(
		func: impl Fn(A) -> M + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + Send + Sync + 'a, {
		Brand::send_fold_map::<FnBrand, A, M>(func, fa)
	}
}

pub use inner::*;
