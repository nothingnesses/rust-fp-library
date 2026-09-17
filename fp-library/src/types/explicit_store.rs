//! Per-`Store` recursion-indirection pointer for the concrete
//! `FreeExplicit` free monad.
//!
//! [`ExplicitStore`] is the sibling of the closure-storage and `Coyoneda`
//! pointer-storage interfaces
//! ([`ClosureStorage`](crate::types::closure_storage::ClosureStorage) and
//! [`CoyoStore`](crate::types::coyo_store::CoyoStore)), for the concrete
//! (non-`'static`, O(N)-bind) free monad's self pointer. Where the
//! closure-storage interface abstracts a stored closure and the `Coyoneda`
//! interface abstracts the outer pointer to an existential cell,
//! `ExplicitStore` abstracts the sized self-pointer that the `Wrap` variant
//! stores so the recursive type has heap indirection: `Box<T>` for the
//! single-shot by-value form, `Rc<T>`/`Arc<T>` for the multi-shot,
//! cheaply-clonable arms. `try_into_inner` recovers ownership of the
//! pointee (always for `Box`; for the refcounted pointers only when the
//! pointer is unique), which is what lets one iterative `Drop` be generic
//! over the store.
//!
//! It is `pub` (mirroring the closure-storage interface) because it bounds
//! the public `FreeExplicit`, so its associated pointer type must not leak
//! a crate-private trait.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::BoxBrand,
//! 	types::explicit_store::ExplicitStore,
//! };
//!
//! let ptr: Box<i32> = Box::new(5);
//! assert_eq!(<BoxBrand as ExplicitStore>::try_into_inner(ptr), Some(5));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::brands::{
			ArcBrand,
			BoxBrand,
			RcBrand,
		},
		fp_macros::*,
		std::{
			rc::Rc,
			sync::Arc,
		},
	};

	/// The per-`Store` recursion-indirection pointer interface for
	/// `FreeExplicit`.
	///
	/// `: 'static` so that `SelfPtr<'a, T>` (which holds the store)
	/// satisfies its own `: 'a` bound for every `'a` without each user
	/// restating `Store: 'a`.
	pub trait ExplicitStore: 'static {
		/// The sized self-pointer the `Wrap` variant stores: a
		/// `Box`/`Rc`/`Arc` of the next step in the computation.
		type SelfPtr<'a, T: 'a>: 'a;

		/// Recovers ownership of the pointee: always for `Box`; for
		/// `Rc`/`Arc` only when the pointer is unique (otherwise `None`,
		/// leaving other holders to dismantle their own share).
		#[document_signature]
		#[document_type_parameters("The lifetime of the pointee.", "The pointee type.")]
		#[document_parameters("The self-pointer to recover from.")]
		#[document_returns(
			"`Some(pointee)` when ownership is recovered, `None` when the pointer is shared."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::explicit_store::ExplicitStore,
		/// };
		///
		/// let ptr: Box<i32> = Box::new(5);
		/// assert_eq!(<BoxBrand as ExplicitStore>::try_into_inner(ptr), Some(5));
		/// ```
		fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T>;
	}

	impl ExplicitStore for BoxBrand {
		type SelfPtr<'a, T: 'a> = Box<T>;

		/// Recovers the pointee unconditionally by moving it out of the
		/// box; the by-value store is never shared.
		#[document_signature]
		#[document_type_parameters("The lifetime of the pointee.", "The pointee type.")]
		#[document_parameters("The self-pointer to recover from.")]
		#[document_returns("Always `Some(pointee)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::BoxBrand,
		/// 	types::explicit_store::ExplicitStore,
		/// };
		///
		/// let ptr: Box<i32> = Box::new(5);
		/// assert_eq!(<BoxBrand as ExplicitStore>::try_into_inner(ptr), Some(5));
		/// ```
		fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T> {
			Some(*ptr)
		}
	}

	impl ExplicitStore for RcBrand {
		type SelfPtr<'a, T: 'a> = Rc<T>;

		/// Recovers the pointee only when this `Rc` is the unique holder.
		#[document_signature]
		#[document_type_parameters("The lifetime of the pointee.", "The pointee type.")]
		#[document_parameters("The self-pointer to recover from.")]
		#[document_returns("`Some(pointee)` when the pointer is unique, `None` when it is shared.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::RcBrand,
		/// 		types::explicit_store::ExplicitStore,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let unique = Rc::new(5);
		/// assert_eq!(<RcBrand as ExplicitStore>::try_into_inner(unique), Some(5));
		///
		/// let shared = Rc::new(5);
		/// let other = Rc::clone(&shared);
		/// assert_eq!(<RcBrand as ExplicitStore>::try_into_inner(shared), None);
		/// assert_eq!(*other, 5);
		/// ```
		fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T> {
			Rc::try_unwrap(ptr).ok()
		}
	}

	impl ExplicitStore for ArcBrand {
		type SelfPtr<'a, T: 'a> = Arc<T>;

		/// Recovers the pointee only when this `Arc` is the unique holder.
		#[document_signature]
		#[document_type_parameters("The lifetime of the pointee.", "The pointee type.")]
		#[document_parameters("The self-pointer to recover from.")]
		#[document_returns("`Some(pointee)` when the pointer is unique, `None` when it is shared.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::ArcBrand,
		/// 		types::explicit_store::ExplicitStore,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let unique = Arc::new(5);
		/// assert_eq!(<ArcBrand as ExplicitStore>::try_into_inner(unique), Some(5));
		///
		/// let shared = Arc::new(5);
		/// let other = Arc::clone(&shared);
		/// assert_eq!(<ArcBrand as ExplicitStore>::try_into_inner(shared), None);
		/// assert_eq!(*other, 5);
		/// ```
		fn try_into_inner<'a, T: 'a>(ptr: Self::SelfPtr<'a, T>) -> Option<T> {
			Arc::try_unwrap(ptr).ok()
		}
	}
}

pub use inner::*;
