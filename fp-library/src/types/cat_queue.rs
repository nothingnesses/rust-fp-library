//! FS-1 substrate: the catenable continuation-queue interface.
//!
//! This module lives in core `crate::types` (not under the
//! `effects`-feature-gated subsystem) because the core `Free<F, A, Store>`
//! carries its queue as `<Store as ClosureStorage>::Queue<_>` and the
//! multi-shot stepping is written against this trait.
//!
//! [`CatQueue`] is the small interface the unified multi-shot stepping needs
//! from a continuation queue: an empty queue (the [`Default`] supertrait),
//! `snoc`, `append`, and `uncons`. It is implemented by the three catenable
//! lists that back the per-`Store` `Queue` associated type on
//! [`ClosureStorage`](crate::types::closure_storage::ClosureStorage): the
//! by-value [`CatList`](crate::types::CatList) for the one-shot Box store
//! (implemented at every element type), and the refcounted
//! [`RcCatList`](crate::types::RcCatList) and
//! [`ArcCatList`](crate::types::ArcCatList) for the
//! multi-shot Rc/Arc stores (implemented only at `C: Clone`, since their
//! `link` clones the shared sublist deque through `Rc`/`Arc::make_mut`).
//!
//! The trait deliberately has no `Clone` supertrait: the Box queue holds
//! non-`Clone` `FnOnce` continuations, so a blanket `Clone` bound would
//! exclude it. The multi-shot stepping that clones the queue per branch adds
//! the `Clone` bound at its own use site, where the element is always a
//! `Clone` `Rc`/`Arc` continuation. Likewise the queue is bounded only by
//! `Default` on `ClosureStorage`, which is all the construction-free
//! accessors and the iterative `Drop` require; `uncons` and the rest are
//! required of the queue only where the multi-shot stepping names this trait.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::{
//! 	CatList,
//! 	cat_queue::CatQueue,
//! };
//!
//! let queue = CatQueue::snoc(CatList::default(), 1);
//! let queue = CatQueue::snoc(queue, 2);
//! let (head, rest) = CatQueue::uncons(queue).unwrap();
//! assert_eq!(head, 1);
//! let (next, _) = CatQueue::uncons(rest).unwrap();
//! assert_eq!(next, 2);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::types::{
			arc_cat_list::ArcCatList,
			cat_list::CatList,
			rc_cat_list::RcCatList,
		},
		fp_macros::*,
	};

	/// One catenable continuation-queue interface over the per-`Store`
	/// queues. The empty queue is the [`Default`] supertrait; `snoc` appends
	/// one element, `append` concatenates two queues, and `uncons` removes
	/// the head. No `Clone` supertrait: the one-shot Box queue holds
	/// non-`Clone` `FnOnce` elements, so cloning is bound at the multi-shot
	/// use site instead.
	#[document_type_parameters("The element type of the queued continuations.")]
	#[document_parameters("The queue to operate on.")]
	pub trait CatQueue<C>: Default + Sized {
		/// Appends one element to the back of the queue.
		#[document_signature]
		#[document_parameters("The element to append.")]
		#[document_returns("The queue with the element at the back.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	CatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue = CatQueue::snoc(CatList::default(), 1);
		/// let (head, _) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn snoc(
			self,
			element: C,
		) -> Self;

		/// Concatenates two queues.
		#[document_signature]
		#[document_parameters("The queue to concatenate to the back.")]
		#[document_returns("The concatenated queue.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	CatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let front = CatQueue::snoc(CatList::default(), 1);
		/// let back = CatQueue::snoc(CatList::default(), 2);
		/// let (head, _) = CatQueue::uncons(CatQueue::append(front, back)).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn append(
			self,
			other: Self,
		) -> Self;

		/// Removes and returns the head element and the rest of the queue,
		/// or `None` when empty.
		#[document_signature]
		#[document_parameters]
		#[document_returns("`Some((head, rest))`, or `None` when the queue is empty.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	CatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue = CatQueue::snoc(CatList::default(), 1);
		/// let (head, rest) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// assert!(CatQueue::uncons(rest).is_none());
		/// ```
		fn uncons(self) -> Option<(C, Self)>;
	}

	#[document_type_parameters("The element type of the queued continuations.")]
	#[document_parameters("The queue to operate on.")]
	impl<C> CatQueue<C> for CatList<C> {
		/// Appends one element to the back of the queue by delegating to the
		/// inherent [`CatList::snoc`].
		#[document_signature]
		#[document_parameters("The element to append.")]
		#[document_returns("The queue with the element at the back.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	CatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue: CatList<i32> = CatQueue::snoc(CatList::default(), 1);
		/// let (head, _) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn snoc(
			self,
			element: C,
		) -> Self {
			CatList::snoc(self, element)
		}

		/// Concatenates two queues by delegating to the inherent
		/// [`CatList::append`].
		#[document_signature]
		#[document_parameters("The queue to concatenate to the back.")]
		#[document_returns("The concatenated queue.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	CatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let front: CatList<i32> = CatQueue::snoc(CatList::default(), 1);
		/// let back: CatList<i32> = CatQueue::snoc(CatList::default(), 2);
		/// let (head, _) = CatQueue::uncons(CatQueue::append(front, back)).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn append(
			self,
			other: Self,
		) -> Self {
			CatList::append(self, other)
		}

		/// Removes the head by delegating to the inherent
		/// [`CatList::uncons`].
		#[document_signature]
		#[document_parameters]
		#[document_returns("`Some((head, rest))`, or `None` when the queue is empty.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	CatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue: CatList<i32> = CatQueue::snoc(CatList::default(), 1);
		/// let (head, rest) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// assert!(CatQueue::uncons(rest).is_none());
		/// ```
		fn uncons(self) -> Option<(C, Self)> {
			CatList::uncons(self)
		}
	}

	#[document_type_parameters("The element type of the queued continuations.")]
	#[document_parameters("The queue to operate on.")]
	impl<C: Clone> CatQueue<C> for RcCatList<C> {
		/// Appends one element to the back of the queue by delegating to the
		/// inherent [`RcCatList::snoc`].
		#[document_signature]
		#[document_parameters("The element to append.")]
		#[document_returns("The queue with the element at the back.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	RcCatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue: RcCatList<i32> = CatQueue::snoc(RcCatList::default(), 1);
		/// let (head, _) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn snoc(
			self,
			element: C,
		) -> Self {
			RcCatList::snoc(self, element)
		}

		/// Concatenates two queues by delegating to the inherent
		/// [`RcCatList::append`].
		#[document_signature]
		#[document_parameters("The queue to concatenate to the back.")]
		#[document_returns("The concatenated queue.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	RcCatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let front: RcCatList<i32> = CatQueue::snoc(RcCatList::default(), 1);
		/// let back: RcCatList<i32> = CatQueue::snoc(RcCatList::default(), 2);
		/// let (head, _) = CatQueue::uncons(CatQueue::append(front, back)).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn append(
			self,
			other: Self,
		) -> Self {
			RcCatList::append(self, other)
		}

		/// Removes the head by delegating to the inherent
		/// [`RcCatList::uncons`].
		#[document_signature]
		#[document_parameters]
		#[document_returns("`Some((head, rest))`, or `None` when the queue is empty.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	RcCatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue: RcCatList<i32> = CatQueue::snoc(RcCatList::default(), 1);
		/// let (head, rest) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// assert!(CatQueue::uncons(rest).is_none());
		/// ```
		fn uncons(self) -> Option<(C, Self)> {
			RcCatList::uncons(self)
		}
	}

	#[document_type_parameters("The element type of the queued continuations.")]
	#[document_parameters("The queue to operate on.")]
	impl<C: Clone> CatQueue<C> for ArcCatList<C> {
		/// Appends one element to the back of the queue by delegating to the
		/// inherent [`ArcCatList::snoc`].
		#[document_signature]
		#[document_parameters("The element to append.")]
		#[document_returns("The queue with the element at the back.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	ArcCatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue: ArcCatList<i32> = CatQueue::snoc(ArcCatList::default(), 1);
		/// let (head, _) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn snoc(
			self,
			element: C,
		) -> Self {
			ArcCatList::snoc(self, element)
		}

		/// Concatenates two queues by delegating to the inherent
		/// [`ArcCatList::append`].
		#[document_signature]
		#[document_parameters("The queue to concatenate to the back.")]
		#[document_returns("The concatenated queue.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	ArcCatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let front: ArcCatList<i32> = CatQueue::snoc(ArcCatList::default(), 1);
		/// let back: ArcCatList<i32> = CatQueue::snoc(ArcCatList::default(), 2);
		/// let (head, _) = CatQueue::uncons(CatQueue::append(front, back)).unwrap();
		/// assert_eq!(head, 1);
		/// ```
		fn append(
			self,
			other: Self,
		) -> Self {
			ArcCatList::append(self, other)
		}

		/// Removes the head by delegating to the inherent
		/// [`ArcCatList::uncons`].
		#[document_signature]
		#[document_parameters]
		#[document_returns("`Some((head, rest))`, or `None` when the queue is empty.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	ArcCatList,
		/// 	cat_queue::CatQueue,
		/// };
		///
		/// let queue: ArcCatList<i32> = CatQueue::snoc(ArcCatList::default(), 1);
		/// let (head, rest) = CatQueue::uncons(queue).unwrap();
		/// assert_eq!(head, 1);
		/// assert!(CatQueue::uncons(rest).is_none());
		/// ```
		fn uncons(self) -> Option<(C, Self)> {
			ArcCatList::uncons(self)
		}
	}
}

pub use inner::*;
