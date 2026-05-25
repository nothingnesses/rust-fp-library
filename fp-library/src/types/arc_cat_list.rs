//! Atomically reference-counted catenable list with O(1) `Clone`.
//!
//! [`ArcCatList`] is the [`Arc`](std::sync::Arc)-backed parallel of
//! [`RcCatList`](crate::types::RcCatList): same shape and trade-offs,
//! but the sublist deque is held behind an `Arc` instead of an `Rc`,
//! so the type is `Send + Sync` whenever the element type is. This
//! is the queue used by [`ArcFree`](crate::types::ArcFree)'s
//! continuation chain so that the `to_view` materialisation can
//! capture it inside an `Fn` closure that is also `Send + Sync` and
//! supports multi-shot dispatch (e.g. the `Choose` effect's handler
//! invoking the same continuation for both branches).
//!
//! ## Trade-offs vs `CatList`
//!
//! - **Clone:** O(1) (head clone plus an atomic refcount bump).
//!   [`CatList::clone`](crate::types::CatList) is O(N) deep-recursive.
//! - **Mutation cost:** structural mutations use
//!   [`std::sync::Arc::make_mut`] for copy-on-write; uniquely-
//!   owned deques mutate in place, shared deques are cloned one
//!   level deep.
//! - **Bounds:** mutation methods and `uncons` require `A: Clone`.
//!   `Send + Sync` propagate from the element type via the auto-
//!   trait derivation on `Arc`.
//! - **Single-thread cost:** `Arc::clone` uses an atomic increment;
//!   programs that never cross threads pay this overhead. Use
//!   [`RcCatList`](crate::types::RcCatList) when thread safety is
//!   not required.
//!
//! ## Use in the Free monad
//!
//! Mirrors the role described in
//! [`RcCatList`](crate::types::RcCatList)'s module docs, with the
//! added requirement that the closure produced by
//! [`ArcFree::to_view`](crate::types::ArcFree::to_view) is itself
//! `Send + Sync` so it can be stored in an
//! `Arc<dyn Fn(...) + Send + Sync>`. Because `ArcCatList<A>` is
//! `Send + Sync` whenever `A` is, capturing it by move and cloning
//! it per call satisfies that bound without resorting to
//! synchronisation primitives like `Mutex`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::ArcCatList;
//!
//! let list = ArcCatList::singleton(1).snoc(2).snoc(3);
//! let cloned = list.clone();
//! assert_eq!(list.len(), 3);
//! assert_eq!(cloned.len(), 3);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		fp_macros::*,
		std::{
			collections::VecDeque,
			sync::Arc,
		},
	};

	/// Internal representation of an [`ArcCatList`].
	///
	/// The sublist deque is wrapped in [`Arc`] so cloning the outer
	/// list is an atomic refcount bump rather than a deep copy.
	#[derive(Debug)]
	enum ArcCatListInner<A> {
		/// Empty list.
		Nil,
		/// Head element, refcounted deque of sublists, total length.
		Cons(A, Arc<VecDeque<ArcCatList<A>>>, usize),
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The inner state to clone.")]
	impl<A: Clone> Clone for ArcCatListInner<A> {
		/// Clones the inner state: head element clone plus an
		/// atomic refcount bump on the sublist deque.
		#[document_signature]
		#[document_returns("A clone of the inner state.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// // observe via `ArcCatList::clone`.
		/// let list = ArcCatList::singleton(1);
		/// let cloned = list.clone();
		/// assert_eq!(cloned.len(), 1);
		/// ```
		fn clone(&self) -> Self {
			match self {
				ArcCatListInner::Nil => ArcCatListInner::Nil,
				ArcCatListInner::Cons(a, q, len) =>
					ArcCatListInner::Cons(a.clone(), Arc::clone(q), *len),
			}
		}
	}

	#[document_type_parameters("The element type.")]
	impl<A> Default for ArcCatListInner<A> {
		/// Returns the `Nil` variant.
		#[document_signature]
		#[document_returns("`ArcCatListInner::Nil`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list: ArcCatList<i32> = Default::default();
		/// assert!(list.is_empty());
		/// ```
		fn default() -> Self {
			ArcCatListInner::Nil
		}
	}

	/// An atomically reference-counted catenable list with O(1)
	/// [`Clone`].
	///
	/// See the [module-level docs](self) for the use case and
	/// trade-offs versus [`CatList`](crate::types::CatList).
	#[document_type_parameters("The element type.")]
	#[derive(Debug)]
	pub struct ArcCatList<A>(ArcCatListInner<A>);

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list to clone.")]
	impl<A: Clone> Clone for ArcCatList<A> {
		/// Clones in O(1) (head-element clone plus an atomic
		/// refcount bump on the sublist deque).
		#[document_signature]
		#[document_returns("A clone of the list.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list = ArcCatList::singleton(1).snoc(2);
		/// let cloned = list.clone();
		/// assert_eq!(list.len(), 2);
		/// assert_eq!(cloned.len(), 2);
		/// ```
		fn clone(&self) -> Self {
			ArcCatList(self.0.clone())
		}
	}

	#[document_type_parameters("The element type.")]
	impl<A> Default for ArcCatList<A> {
		/// Returns an empty list.
		#[document_signature]
		#[document_returns("An empty `ArcCatList`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list: ArcCatList<i32> = Default::default();
		/// assert!(list.is_empty());
		/// ```
		fn default() -> Self {
			ArcCatList::empty()
		}
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list to inspect.")]
	impl<A> ArcCatList<A> {
		/// Creates an empty `ArcCatList`.
		#[document_signature]
		#[document_returns("An empty `ArcCatList`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list: ArcCatList<i32> = ArcCatList::empty();
		/// assert!(list.is_empty());
		/// ```
		#[inline]
		pub const fn empty() -> Self {
			ArcCatList(ArcCatListInner::Nil)
		}

		/// Returns `true` if the list contains no elements.
		#[document_signature]
		#[document_parameters]
		#[document_returns("`true` if empty.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list: ArcCatList<i32> = ArcCatList::empty();
		/// assert!(list.is_empty());
		/// ```
		#[inline]
		pub fn is_empty(&self) -> bool {
			matches!(self.0, ArcCatListInner::Nil)
		}

		/// Returns the number of elements in the list.
		#[document_signature]
		#[document_parameters]
		#[document_returns("The element count.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list = ArcCatList::singleton(1).snoc(2);
		/// assert_eq!(list.len(), 2);
		/// ```
		#[inline]
		pub fn len(&self) -> usize {
			match &self.0 {
				ArcCatListInner::Nil => 0,
				ArcCatListInner::Cons(_, _, len) => *len,
			}
		}

		/// Creates an `ArcCatList` containing a single element.
		#[document_signature]
		#[document_parameters("The element to wrap.")]
		#[document_returns("A singleton `ArcCatList`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list = ArcCatList::singleton(42);
		/// assert_eq!(list.len(), 1);
		/// ```
		#[inline]
		pub fn singleton(a: A) -> Self {
			ArcCatList(ArcCatListInner::Cons(a, Arc::new(VecDeque::new()), 1))
		}
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list to manipulate.")]
	impl<A: Clone> ArcCatList<A> {
		/// Prepends an element to the front of the list.
		#[document_signature]
		#[document_parameters("The element to prepend.")]
		#[document_returns("A new list with the element at the front.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list = ArcCatList::singleton(2).cons(1);
		/// assert_eq!(list.len(), 2);
		/// ```
		#[inline]
		pub fn cons(
			self,
			a: A,
		) -> Self {
			Self::link(ArcCatList::singleton(a), self)
		}

		/// Appends an element to the back of the list.
		#[document_signature]
		#[document_parameters("The element to append.")]
		#[document_returns("A new list with the element at the back.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list = ArcCatList::singleton(1).snoc(2);
		/// assert_eq!(list.len(), 2);
		/// ```
		#[inline]
		pub fn snoc(
			self,
			a: A,
		) -> Self {
			Self::link(self, ArcCatList::singleton(a))
		}

		/// Concatenates two lists in O(1) amortised.
		#[document_signature]
		#[document_parameters("The list to concatenate to the back.")]
		#[document_returns("The concatenated list.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let xs = ArcCatList::singleton(1);
		/// let ys = ArcCatList::singleton(2);
		/// assert_eq!(xs.append(ys).len(), 2);
		/// ```
		#[inline]
		pub fn append(
			self,
			other: Self,
		) -> Self {
			Self::link(self, other)
		}

		/// Internal: links two lists by pushing `right` into `left`'s
		/// sublist deque via [`Arc::make_mut`] (copy-on-write).
		#[document_signature]
		#[document_parameters("The left list.", "The right list.")]
		#[document_returns("The linked list.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// // observe via `append`.
		/// let xs = ArcCatList::singleton(1);
		/// let ys = ArcCatList::singleton(2);
		/// assert_eq!(xs.append(ys).len(), 2);
		/// ```
		fn link(
			mut left: Self,
			right: Self,
		) -> Self {
			if left.is_empty() {
				return right;
			}
			if right.is_empty() {
				return left;
			}
			if let ArcCatListInner::Cons(_, q, len) = &mut left.0 {
				*len += right.len();
				Arc::make_mut(q).push_back(right);
			}
			left
		}

		/// Removes the first element, returning it and the rest of
		/// the list. O(1) amortised.
		#[document_signature]
		#[document_parameters]
		#[document_returns("`Some((head, tail))` or `None` if empty.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// let list = ArcCatList::singleton(1).snoc(2);
		/// let (h, t) = list.uncons().unwrap();
		/// assert_eq!(h, 1);
		/// assert_eq!(t.len(), 1);
		/// ```
		pub fn uncons(mut self) -> Option<(A, Self)> {
			let inner = std::mem::replace(&mut self.0, ArcCatListInner::Nil);
			// SAFETY: see `RcCatList::uncons`; the same reasoning applies
			// (Nil sentinel carries no resources, so skipping the
			// custom Drop on the consumed `self` is sound).
			std::mem::forget(self);
			match inner {
				ArcCatListInner::Nil => None,
				ArcCatListInner::Cons(a, q_rc, _) =>
					if q_rc.is_empty() {
						Some((a, ArcCatList::empty()))
					} else {
						let owned: VecDeque<ArcCatList<A>> =
							Arc::try_unwrap(q_rc).unwrap_or_else(|shared| (*shared).clone());
						Some((a, Self::flatten_deque(owned)))
					},
			}
		}

		/// Internal: flattens an owned deque of sublists into a
		/// single list via a right fold.
		#[document_signature]
		#[document_parameters("The deque of sublists to flatten.")]
		#[document_returns("A single flattened `ArcCatList`.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// // observe via `uncons` (which calls `flatten_deque`).
		/// let list = ArcCatList::singleton(1).snoc(2).snoc(3);
		/// let (_, t) = list.uncons().unwrap();
		/// assert_eq!(t.len(), 2);
		/// ```
		fn flatten_deque(deque: VecDeque<ArcCatList<A>>) -> Self {
			deque.into_iter().rfold(ArcCatList::empty(), |acc, list| Self::link(list, acc))
		}
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list being dropped.")]
	impl<A> Drop for ArcCatList<A> {
		/// Iterative drop to avoid stack overflow on long chains.
		///
		/// See [`RcCatList::drop`](crate::types::RcCatList) for the
		/// algorithm; this version differs only in `Arc` vs `Rc`.
		#[document_signature]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::types::ArcCatList;
		///
		/// {
		/// 	let _list = ArcCatList::singleton(1).snoc(2).snoc(3);
		/// } // drop called here
		/// // Reaching this point without panicking proves drop completed.
		/// let post_drop: ArcCatList<i32> = ArcCatList::singleton(7);
		/// assert!(matches!(post_drop.uncons().map(|(h, _)| h), Some(7)));
		/// ```
		fn drop(&mut self) {
			let mut worklist: Vec<VecDeque<ArcCatList<A>>> = Vec::new();

			if let ArcCatListInner::Cons(_, q, _) = &mut self.0
				&& let Some(deque) = Arc::get_mut(q)
				&& !deque.is_empty()
			{
				worklist.push(std::mem::take(deque));
			}

			while let Some(mut deque) = worklist.pop() {
				for mut child in deque.drain(..) {
					if let ArcCatListInner::Cons(_, inner_q, _) = &mut child.0
						&& let Some(inner_deque) = Arc::get_mut(inner_q)
						&& !inner_deque.is_empty()
					{
						worklist.push(std::mem::take(inner_deque));
					}
				}
			}
		}
	}
}

pub use inner::*;
