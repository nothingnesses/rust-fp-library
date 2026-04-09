//! Thread-safe by-reference variant of [`Foldable`](crate::classes::Foldable).
//!
//! **User story:** "I want to fold over a thread-safe memoized value by reference."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::send_ref_foldable::*,
//! 	types::*,
//! };
//!
//! let lazy = ArcLazy::new(|| 10);
//! let result = send_ref_fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(
//! 	|a: &i32| a.to_string(),
//! 	lazy,
//! );
//! assert_eq!(result, "10");
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

	/// Thread-safe by-reference folding over a structure.
	///
	/// Similar to [`RefFoldable`], but closures and elements must be `Send + Sync`.
	/// Unlike [`ParRefFunctor`](crate::classes::ParRefFunctor) (which requires
	/// [`RefFunctor`](crate::classes::RefFunctor) as a supertrait), `SendRefFoldable`
	/// does not require `RefFoldable`. This is because the SendRef monadic traits
	/// (functor, pointed, lift, applicative) construct new containers internally,
	/// and `ArcLazy::new` requires `Send` on closures, which `Ref` trait signatures
	/// do not guarantee. The SendRef and Ref hierarchies are therefore independent.
	///
	/// All three methods (`send_ref_fold_map`, `send_ref_fold_right`, `send_ref_fold_left`)
	/// have default implementations in terms of each other, so implementors
	/// only need to provide one.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefFoldable {
		/// Maps values to a monoid by reference and combines them (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to map each element reference to a monoid. Must be `Send + Sync`.",
			"The structure to fold."
		)]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_foldable::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 5);
		/// let result = send_ref_fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(
		/// 	|a: &i32| a.to_string(),
		/// 	lazy,
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn send_ref_fold_map<'a, FnBrand, A: Send + Sync + 'a + Clone, M>(
			func: impl Fn(&A) -> M + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: SendLiftFn + 'a,
			M: Monoid + Send + Sync + 'a, {
			Self::send_ref_fold_right::<FnBrand, A, M>(
				move |a: &A, acc| Semigroup::append(func(a), acc),
				Monoid::empty(),
				fa,
			)
		}

		/// Folds the structure from the right by reference (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The type of the accumulator."
		)]
		#[document_parameters(
			"The function to apply to each element reference and accumulator. Must be `Send + Sync`.",
			"The initial value of the accumulator.",
			"The structure to fold."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_foldable::SendRefFoldable,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 10);
		/// let result = <LazyBrand<ArcLazyConfig> as SendRefFoldable>::send_ref_fold_right::<
		/// 	ArcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|a: &i32, acc: i32| acc + *a, 0, lazy);
		/// assert_eq!(result, 10);
		/// ```
		fn send_ref_fold_right<'a, FnBrand, A: Send + Sync + 'a + Clone, B: Send + Sync + 'a>(
			func: impl Fn(&A, B) -> B + Send + Sync + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: SendLiftFn + 'a, {
			let f = <FnBrand as SendLiftFn>::new(move |(a, b): (A, B)| func(&a, b));
			let m = Self::send_ref_fold_map::<FnBrand, A, SendEndofunction<FnBrand, B>>(
				move |a: &A| {
					let a = a.clone();
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

		/// Folds the structure from the left by reference (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The type of the accumulator."
		)]
		#[document_parameters(
			"The function to apply to the accumulator and each element reference. Must be `Send + Sync`.",
			"The initial value of the accumulator.",
			"The structure to fold."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_foldable::SendRefFoldable,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 10);
		/// let result = <LazyBrand<ArcLazyConfig> as SendRefFoldable>::send_ref_fold_left::<
		/// 	ArcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|acc: i32, a: &i32| acc + *a, 0, lazy);
		/// assert_eq!(result, 10);
		/// ```
		fn send_ref_fold_left<'a, FnBrand, A: Send + Sync + 'a + Clone, B: Send + Sync + 'a>(
			func: impl Fn(B, &A) -> B + Send + Sync + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: SendLiftFn + 'a, {
			let f = <FnBrand as SendLiftFn>::new(move |(b, a): (B, A)| func(b, &a));
			let m = Self::send_ref_fold_map::<FnBrand, A, Dual<SendEndofunction<FnBrand, B>>>(
				move |a: &A| {
					let a = a.clone();
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

	/// Maps values to a monoid by reference and combines them (thread-safe).
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefFoldable::send_ref_fold_map`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the cloneable function to use.",
		"The brand of the structure.",
		"The type of the elements.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to map each element reference to a monoid.",
		"The structure to fold."
	)]
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::send_ref_foldable::*,
	/// 	types::*,
	/// };
	///
	/// let lazy = ArcLazy::new(|| 5);
	/// let result = send_ref_fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(
	/// 	|a: &i32| a.to_string(),
	/// 	lazy,
	/// );
	/// assert_eq!(result, "5");
	/// ```
	pub fn send_ref_fold_map<
		'a,
		FnBrand: SendLiftFn + 'a,
		Brand: SendRefFoldable,
		A: Send + Sync + 'a + Clone,
		M,
	>(
		func: impl Fn(&A) -> M + Send + Sync + 'a,
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + Send + Sync + 'a, {
		Brand::send_ref_fold_map::<FnBrand, A, M>(func, fa)
	}
}

pub use inner::*;
