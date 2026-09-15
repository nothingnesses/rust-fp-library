//! Reference-counted catenable list with O(1) `Clone`.
//!
//! [`RcCatList`] is a variant of [`CatList`](crate::types::CatList) whose
//! sublist queue is an immutable [`Rc`]-linked stack, making
//! `Clone` a refcount bump rather than a deep recursive copy. This is used by
//! the Rc-store `Free`'s continuation queue so that the
//! `to_view` materialisation can capture the queue inside an `Fn`
//! closure and re-clone it on every invocation, supporting multi-shot
//! handlers (e.g. the `Choose` effect) that fire the same continuation
//! more than once.
//!
//! ## Trade-offs vs `CatList`
//!
//! - **Clone:** `RcCatList::clone` is O(1) (one head-element clone plus
//!   a refcount bump). [`CatList::clone`](crate::types::CatList) is O(N)
//!   because its sublist deque is value-typed and recursively cloned.
//! - **Queue operations:** structural operations
//!   ([`snoc`](RcCatList::snoc), [`append`](RcCatList::append),
//!   [`cons`](RcCatList::cons)) add one immutable child-stack node in O(1).
//!   [`uncons`](RcCatList::uncons) consumes those nodes persistently and is
//!   O(1) amortised along each queue lineage. A shared queue is never copied.
//! - **Bounds:** structural operations and `uncons` require `A: Clone`.
//! - **Thread safety:** `RcCatList` is `!Send + !Sync`. Use
//!   [`ArcCatList`](crate::types::ArcCatList) in thread-safe contexts.
//!
//! ## Use in the Free monad
//!
//! the Rc-store `Free`'s `to_view` flattens the
//! pending continuation queue into a closure passed to `F::map`. With
//! a value-typed `CatList`, that closure can move the queue exactly
//! once, making the closure single-shot at the type level. Replacing
//! the queue with `RcCatList` lets the closure capture the queue by
//! move and clone it (O(1)) on every call, which is required for
//! handler functors that invoke the closure more than once per layer
//! (e.g. the `Choose` handler running both branches of a non-
//! deterministic computation).
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::RcCatList;
//!
//! let list = RcCatList::singleton(1).snoc(2).snoc(3);
//! let cloned = list.clone();
//! assert_eq!(list.len(), 3);
//! assert_eq!(cloned.len(), 3);
//! ```

use std::rc::Rc;

/// One immutable child in the persistent sublist stack.
#[derive(Debug)]
struct RcChild<A> {
	/// The child list appended at this position.
	list: inner::RcCatList<A>,
	/// The previously appended children, newest first.
	previous: Option<Rc<RcChild<A>>>,
}

/// A persistent stack of appended sublists.
///
/// Pushing and cloning each allocate or clone one [`Rc`]. Popping a shared
/// stack clones only the selected child list and the preceding pointer; it
/// never copies a container of pending children.
#[derive(Debug)]
struct RcChildren<A>(Option<Rc<RcChild<A>>>);

impl<A> RcChildren<A> {
	/// Creates an empty child stack.
	const fn empty() -> Self {
		Self(None)
	}

	/// Pushes one child without mutating any shared node.
	fn push(
		&mut self,
		list: inner::RcCatList<A>,
	) {
		self.0 = Some(Rc::new(RcChild {
			list,
			previous: self.0.take(),
		}));
	}

	/// Returns whether the stack has no children.
	fn is_empty(&self) -> bool {
		self.0.is_none()
	}
}

impl<A: Clone> RcChildren<A> {
	/// Pops one child persistently. Shared nodes clone two O(1) handles, not
	/// the complete pending-child collection.
	fn pop(mut self) -> Option<(inner::RcCatList<A>, Self)> {
		let node = self.0.take()?;
		match Rc::try_unwrap(node) {
			Ok(node) => Some((node.list, Self(node.previous))),
			Err(node) => Some((node.list.clone(), Self(node.previous.clone()))),
		}
	}
}

impl<A> Clone for RcChildren<A> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<A> Drop for RcChildren<A> {
	fn drop(&mut self) {
		let mut current = self.0.take();
		while let Some(node) = current {
			match Rc::try_unwrap(node) {
				Ok(mut node) => current = node.previous.take(),
				Err(_) => break,
			}
		}
	}
}

#[fp_macros::document_module]
mod inner {
	use {
		super::{
			RcChild,
			RcChildren,
		},
		fp_macros::*,
		std::rc::Rc,
	};

	/// Internal representation of an [`RcCatList`].
	///
	/// The immutable child stack is shared so cloning the outer list is a
	/// refcount bump rather than a deep copy.
	#[derive(Debug)]
	enum RcCatListInner<A> {
		/// Empty list.
		Nil,
		/// Head element, persistent stack of sublists, total length.
		Cons(A, RcChildren<A>, usize),
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The inner state to clone.")]
	impl<A: Clone> Clone for RcCatListInner<A> {
		/// Clones the inner state: head element clone plus a
		/// refcount bump on the persistent child stack.
		#[document_signature]
		#[document_returns("A clone of the inner state.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// // `RcCatListInner` is private; observe via `RcCatList::clone`.
		/// let list = RcCatList::singleton(1);
		/// let cloned = list.clone();
		/// assert_eq!(cloned.len(), 1);
		/// ```
		fn clone(&self) -> Self {
			match self {
				RcCatListInner::Nil => RcCatListInner::Nil,
				RcCatListInner::Cons(a, children, len) =>
					RcCatListInner::Cons(a.clone(), children.clone(), *len),
			}
		}
	}

	#[document_type_parameters("The element type.")]
	impl<A> Default for RcCatListInner<A> {
		/// Returns the `Nil` variant.
		#[document_signature]
		#[document_returns("`RcCatListInner::Nil`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// // observe via `RcCatList::default`.
		/// let list: RcCatList<i32> = Default::default();
		/// assert!(list.is_empty());
		/// ```
		fn default() -> Self {
			RcCatListInner::Nil
		}
	}

	/// A reference-counted catenable list with O(1) [`Clone`].
	///
	/// See the [module-level docs](self) for the use case and
	/// trade-offs versus [`CatList`](crate::types::CatList).
	#[document_type_parameters("The element type.")]
	#[derive(Debug)]
	pub struct RcCatList<A>(RcCatListInner<A>);

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list to clone.")]
	impl<A: Clone> Clone for RcCatList<A> {
		/// Clones in O(1) (head-element clone plus a refcount bump
		/// on the persistent child stack). No deep recursion into sublists.
		#[document_signature]
		#[document_returns("A clone of the list.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list = RcCatList::singleton(1).snoc(2);
		/// let cloned = list.clone();
		/// assert_eq!(list.len(), 2);
		/// assert_eq!(cloned.len(), 2);
		/// ```
		fn clone(&self) -> Self {
			RcCatList(self.0.clone())
		}
	}

	#[document_type_parameters("The element type.")]
	impl<A> Default for RcCatList<A> {
		/// Returns an empty list.
		#[document_signature]
		#[document_returns("An empty `RcCatList`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list: RcCatList<i32> = Default::default();
		/// assert!(list.is_empty());
		/// ```
		fn default() -> Self {
			RcCatList::empty()
		}
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list to inspect.")]
	impl<A> RcCatList<A> {
		/// Creates an empty `RcCatList`.
		#[document_signature]
		#[document_returns("An empty `RcCatList`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list: RcCatList<i32> = RcCatList::empty();
		/// assert!(list.is_empty());
		/// ```
		#[inline]
		pub const fn empty() -> Self {
			RcCatList(RcCatListInner::Nil)
		}

		/// Returns `true` if the list contains no elements.
		#[document_signature]
		#[document_parameters]
		#[document_returns("`true` if empty.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list: RcCatList<i32> = RcCatList::empty();
		/// assert!(list.is_empty());
		/// ```
		#[inline]
		pub fn is_empty(&self) -> bool {
			matches!(self.0, RcCatListInner::Nil)
		}

		/// Returns the number of elements in the list.
		#[document_signature]
		#[document_parameters]
		#[document_returns("The element count.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list = RcCatList::singleton(1).snoc(2);
		/// assert_eq!(list.len(), 2);
		/// ```
		#[inline]
		pub fn len(&self) -> usize {
			match &self.0 {
				RcCatListInner::Nil => 0,
				RcCatListInner::Cons(_, _, len) => *len,
			}
		}

		/// Creates an `RcCatList` containing a single element.
		#[document_signature]
		#[document_parameters("The element to wrap.")]
		#[document_returns("A singleton `RcCatList`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list = RcCatList::singleton(42);
		/// assert_eq!(list.len(), 1);
		/// ```
		#[inline]
		pub fn singleton(a: A) -> Self {
			RcCatList(RcCatListInner::Cons(a, RcChildren::empty(), 1))
		}
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list to manipulate.")]
	impl<A: Clone> RcCatList<A> {
		/// Prepends an element to the front of the list.
		#[document_signature]
		#[document_parameters("The element to prepend.")]
		#[document_returns("A new list with the element at the front.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list = RcCatList::singleton(2).cons(1);
		/// assert_eq!(list.len(), 2);
		/// ```
		#[inline]
		pub fn cons(
			self,
			a: A,
		) -> Self {
			Self::link(RcCatList::singleton(a), self)
		}

		/// Appends an element to the back of the list.
		#[document_signature]
		#[document_parameters("The element to append.")]
		#[document_returns("A new list with the element at the back.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let list = RcCatList::singleton(1).snoc(2);
		/// assert_eq!(list.len(), 2);
		/// ```
		#[inline]
		pub fn snoc(
			self,
			a: A,
		) -> Self {
			Self::link(self, RcCatList::singleton(a))
		}

		/// Concatenates two lists in O(1) amortised.
		#[document_signature]
		#[document_parameters("The list to concatenate to the back.")]
		#[document_returns("The concatenated list.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// let xs = RcCatList::singleton(1);
		/// let ys = RcCatList::singleton(2);
		/// assert_eq!(xs.append(ys).len(), 2);
		/// ```
		#[inline]
		pub fn append(
			self,
			other: Self,
		) -> Self {
			Self::link(self, other)
		}

		/// Internal: links two lists by pushing `right` onto `left`'s
		/// immutable child stack.
		#[document_signature]
		#[document_parameters("The left list.", "The right list.")]
		#[document_returns("The linked list.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because link is a private persistent structural helper; public cons, snoc, append, and uncons exercise it while preserving RcCatList invariants."
		)]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// // observe via `append`.
		/// let xs = RcCatList::singleton(1);
		/// let ys = RcCatList::singleton(2);
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
			if let RcCatListInner::Cons(_, children, len) = &mut left.0 {
				*len += right.len();
				children.push(right);
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
		/// use fp_library::types::RcCatList;
		///
		/// let list = RcCatList::singleton(1).snoc(2);
		/// let (h, t) = list.uncons().unwrap();
		/// assert_eq!(h, 1);
		/// assert_eq!(t.len(), 1);
		/// ```
		pub fn uncons(mut self) -> Option<(A, Self)> {
			let inner = std::mem::replace(&mut self.0, RcCatListInner::Nil);
			// SAFETY: `inner` now owns the original data and `self.0` is
			// `Nil`. `mem::forget` skips `RcCatList`'s custom `Drop`,
			// which would walk the now-empty sentinel. Sound because
			// `RcCatListInner::Nil` carries no resources.
			std::mem::forget(self);
			match inner {
				RcCatListInner::Nil => None,
				RcCatListInner::Cons(a, children, _) =>
					if children.is_empty() {
						Some((a, RcCatList::empty()))
					} else {
						Some((a, Self::flatten_children(children)))
					},
			}
		}

		/// Internal: flattens a persistent stack of sublists into a single
		/// list, linking the newest child first so the oldest becomes the head.
		#[document_signature]
		#[document_parameters("The persistent stack of sublists to flatten.")]
		#[document_returns("A single flattened `RcCatList`.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because flatten_children is a private restructuring helper; public uncons exercises it when linked sublists are consumed."
		)]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// // observe via `uncons` (which calls `flatten_children`).
		/// let list = RcCatList::singleton(1).snoc(2).snoc(3);
		/// let (_, t) = list.uncons().unwrap();
		/// assert_eq!(t.len(), 2);
		/// ```
		fn flatten_children(mut children: RcChildren<A>) -> Self {
			let mut flattened = RcCatList::empty();
			while let Some((child, previous)) = children.pop() {
				flattened = Self::link(child, flattened);
				children = previous;
			}
			flattened
		}
	}

	#[document_type_parameters("The element type.")]
	#[document_parameters("The list being dropped.")]
	impl<A> Drop for RcCatList<A> {
		/// Iterative drop to avoid stack overflow on long chains.
		///
		/// Walks the tree by transferring uniquely-owned child-stack nodes
		/// onto an explicit worklist. Shared nodes stay with their other owners,
		/// which short-circuits recursive destruction at the sharing boundary.
		#[document_signature]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because Drop::drop cannot be called directly from public examples; leaving the value to go out of scope exercises the destructor."
		)]
		///
		/// ```
		/// use fp_library::types::RcCatList;
		///
		/// {
		/// 	let _list = RcCatList::singleton(1).snoc(2).snoc(3);
		/// } // drop called here
		/// // Reaching this point without panicking proves drop completed.
		/// let post_drop: RcCatList<i32> = RcCatList::singleton(7);
		/// assert!(matches!(post_drop.uncons().map(|(h, _)| h), Some(7)));
		/// ```
		fn drop(&mut self) {
			let mut worklist: Vec<Rc<RcChild<A>>> = Vec::new();

			if let RcCatListInner::Cons(_, children, _) = &mut self.0
				&& let Some(node) = children.0.take()
			{
				worklist.push(node);
			}

			while let Some(node) = worklist.pop() {
				if let Ok(mut node) = Rc::try_unwrap(node) {
					if let RcCatListInner::Cons(_, children, _) = &mut node.list.0
						&& let Some(child) = children.0.take()
					{
						worklist.push(child);
					}
					if let Some(previous) = node.previous.take() {
						worklist.push(previous);
					}
				}
			}
		}
	}
}

pub use inner::*;
