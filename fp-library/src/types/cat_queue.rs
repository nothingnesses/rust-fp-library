//! A double-ended queue with O(1) amortized operations.
//!
//! This module provides [`CatQueue`], a "banker's queue" implementation using two `Vec`s.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::cat_queue::CatQueue;
//!
//! let mut q = CatQueue::empty();
//! q = q.snoc(1).snoc(2).snoc(3);
//!
//! let (a, q) = q.uncons().unwrap();
//! assert_eq!(a, 1);
//!
//! let (b, q) = q.uncons().unwrap();
//! assert_eq!(b, 2);
//! ```

/// A double-ended queue with O(1) amortized operations.
///
/// This is a "banker's queue" implementation using two `Vec`s.
/// - `front`: Elements in FIFO order (head is next to dequeue)
/// - `back`: Elements in LIFO order (to be reversed when front empties)
///
/// ### Type Parameters
///
/// * `A`: The type of the elements in the queue.
///
/// ### Fields
///
/// * `front`: Elements ready to be dequeued (in order).
/// * `back`: Elements recently enqueued (in reverse order).
///
/// ### Examples
///
/// ```
/// use fp_library::types::cat_queue::CatQueue;
///
/// let q: CatQueue<i32> = CatQueue::empty();
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatQueue<A> {
	/// Elements ready to be dequeued (in order).
	front: Vec<A>,
	/// Elements recently enqueued (in reverse order).
	back: Vec<A>,
}

impl<A> Default for CatQueue<A> {
	fn default() -> Self {
		Self::empty()
	}
}

impl<A> CatQueue<A> {
	/// Creates an empty queue.
	///
	/// ### Type Signature
	///
	/// `forall a. () -> CatQueue a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the queue.
	///
	/// ### Returns
	///
	/// An empty `CatQueue`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q: CatQueue<i32> = CatQueue::empty();
	/// assert!(q.is_empty());
	/// ```
	#[inline]
	pub const fn empty() -> Self {
		CatQueue { front: Vec::new(), back: Vec::new() }
	}

	/// Returns `true` if the queue contains no elements.
	///
	/// ### Type Signature
	///
	/// `forall a. CatQueue a -> bool`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the queue.
	///
	/// ### Parameters
	///
	/// * `self`: The queue to check.
	///
	/// ### Returns
	///
	/// `true` if the queue is empty, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q: CatQueue<i32> = CatQueue::empty();
	/// assert!(q.is_empty());
	/// ```
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.front.is_empty() && self.back.is_empty()
	}

	/// Returns the number of elements in the queue.
	///
	/// ### Type Signature
	///
	/// `forall a. CatQueue a -> usize`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the queue.
	///
	/// ### Parameters
	///
	/// * `self`: The queue to check.
	///
	/// ### Returns
	///
	/// The number of elements in the queue.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q = CatQueue::singleton(1);
	/// assert_eq!(q.len(), 1);
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		self.front.len() + self.back.len()
	}

	/// Creates a queue containing a single element.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> CatQueue a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `a`: The element to put in the queue.
	///
	/// ### Returns
	///
	/// A `CatQueue` containing the single element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q = CatQueue::singleton(1);
	/// assert_eq!(q.len(), 1);
	/// ```
	#[inline]
	pub fn singleton(a: A) -> Self {
		CatQueue { front: vec![a], back: Vec::new() }
	}

	/// Appends an element to the front of the queue.
	///
	/// ### Type Signature
	///
	/// `forall a. (CatQueue a, a) -> CatQueue a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `self`: The queue.
	/// * `a`: The element to append.
	///
	/// ### Returns
	///
	/// The new queue with the element appended to the front.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q = CatQueue::empty().cons(1);
	/// assert_eq!(q.len(), 1);
	/// ```
	#[inline]
	pub fn cons(
		mut self,
		a: A,
	) -> Self {
		self.front.push(a);
		// Note: This puts 'a' at the end of front, but we read from the end
		// Actually, we need to reverse our mental model:
		// front is stored in reverse order (last element is head)
		self
	}

	/// Appends an element to the back of the queue.
	///
	/// ### Type Signature
	///
	/// `forall a. (CatQueue a, a) -> CatQueue a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `self`: The queue.
	/// * `a`: The element to append.
	///
	/// ### Returns
	///
	/// The new queue with the element appended to the back.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q = CatQueue::empty().snoc(1);
	/// assert_eq!(q.len(), 1);
	/// ```
	#[inline]
	pub fn snoc(
		mut self,
		a: A,
	) -> Self {
		self.back.push(a);
		self
	}

	/// Removes and returns the first element.
	///
	/// Returns `None` if the queue is empty.
	///
	/// ### Type Signature
	///
	/// `forall a. CatQueue a -> Option (a, CatQueue a)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The queue.
	///
	/// ### Returns
	///
	/// An option containing the first element and the rest of the queue, or `None` if empty.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q = CatQueue::singleton(1);
	/// let (a, q) = q.uncons().unwrap();
	/// assert_eq!(a, 1);
	/// assert!(q.is_empty());
	/// ```
	pub fn uncons(mut self) -> Option<(A, Self)> {
		if self.front.is_empty() {
			if self.back.is_empty() {
				return None;
			}
			// Reverse back into front
			self.back.reverse();
			std::mem::swap(&mut self.front, &mut self.back);
		}

		// Pop from the end of front (which is the "head" in our representation)
		let a = self.front.pop()?;
		Some((a, self))
	}

	/// Removes and returns the last element.
	///
	/// Returns `None` if the queue is empty.
	///
	/// ### Type Signature
	///
	/// `forall a. CatQueue a -> Option (a, CatQueue a)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The queue.
	///
	/// ### Returns
	///
	/// An option containing the last element and the rest of the queue, or `None` if empty.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_queue::CatQueue;
	///
	/// let q = CatQueue::singleton(1);
	/// let (a, q) = q.unsnoc().unwrap();
	/// assert_eq!(a, 1);
	/// assert!(q.is_empty());
	/// ```
	pub fn unsnoc(mut self) -> Option<(A, Self)> {
		if self.back.is_empty() {
			if self.front.is_empty() {
				return None;
			}
			// Reverse front into back
			self.front.reverse();
			std::mem::swap(&mut self.front, &mut self.back);
		}

		let a = self.back.pop()?;
		Some((a, self))
	}

	/// Returns an iterator over the elements of the queue.
	///
	/// ### Type Signature
	///
	/// `forall a. &CatQueue a -> Iterator a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The queue.
	///
	/// ### Returns
	///
	/// An iterator over references to the elements.
	pub fn iter(&self) -> impl Iterator<Item = &A> {
		self.front.iter().rev().chain(self.back.iter())
	}
}

// Iteration support for convenient use
impl<A> IntoIterator for CatQueue<A> {
	type Item = A;
	type IntoIter = CatQueueIter<A>;

	fn into_iter(self) -> Self::IntoIter {
		CatQueueIter { queue: self }
	}
}

/// An iterator that consumes a `CatQueue`.
///
/// ### Type Parameters
///
/// * `A`: The type of the elements in the queue.
///
/// ### Fields
///
/// * `queue`: The queue being iterated over.
pub struct CatQueueIter<A> {
	queue: CatQueue<A>,
}

impl<A> Iterator for CatQueueIter<A> {
	type Item = A;

	fn next(&mut self) -> Option<Self::Item> {
		let (a, rest) = std::mem::take(&mut self.queue).uncons()?;
		self.queue = rest;
		Some(a)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.queue.len();
		(len, Some(len))
	}
}

impl<A> DoubleEndedIterator for CatQueueIter<A> {
	fn next_back(&mut self) -> Option<Self::Item> {
		let (a, rest) = std::mem::take(&mut self.queue).unsnoc()?;
		self.queue = rest;
		Some(a)
	}
}

impl<A> ExactSizeIterator for CatQueueIter<A> {}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests basic queue operations: creation, emptiness check, and length.
	/// This ensures that a new queue is empty and has length 0, and a singleton queue is not empty and has length 1.
	#[test]
	fn test_basic_operations() {
		let q: CatQueue<i32> = CatQueue::empty();
		assert!(q.is_empty());
		assert_eq!(q.len(), 0);

		let q = CatQueue::singleton(1);
		assert!(!q.is_empty());
		assert_eq!(q.len(), 1);
	}

	/// Tests the First-In-First-Out (FIFO) property of the queue.
	/// Elements are added using `snoc` (append to back) and removed using `uncons` (remove from front).
	/// We verify that elements are retrieved in the same order they were added (1, 2, 3).
	#[test]
	fn test_fifo_behavior() {
		let mut q = CatQueue::empty();
		q = q.snoc(1).snoc(2).snoc(3);

		let (a, q) = q.uncons().unwrap();
		assert_eq!(a, 1);
		let (b, q) = q.uncons().unwrap();
		assert_eq!(b, 2);
		let (c, q) = q.uncons().unwrap();
		assert_eq!(c, 3);
		assert!(q.uncons().is_none());
	}

	/// Tests the amortization logic where the `back` stack is reversed into the `front` stack.
	/// We fill the queue using `snoc` (which pushes to `back`), then empty it using `uncons`.
	/// The first `uncons` should trigger the reversal of `back` into `front`.
	/// We verify that all elements are retrieved correctly after this internal operation.
	#[test]
	fn test_amortization_logic() {
		// Fill back
		let mut q = CatQueue::empty();
		for i in 0..10 {
			q = q.snoc(i);
		}
		// uncons triggers reversal
		for i in 0..10 {
			let (val, next_q) = q.uncons().unwrap();
			assert_eq!(val, i);
			q = next_q;
		}
	}

	/// Tests the double-ended nature of the queue.
	/// We use `cons` (prepend) and `snoc` (append) to add elements to both ends.
	/// We use `uncons` (pop front) and `unsnoc` (pop back) to remove elements from both ends.
	/// This verifies that the queue correctly handles operations at both ends simultaneously.
	#[test]
	fn test_double_ended() {
		let mut q = CatQueue::empty();
		q = q.cons(1).snoc(2).cons(0).snoc(3);
		// Queue: [0, 1, 2, 3]

		let (val, q) = q.uncons().unwrap();
		assert_eq!(val, 0);
		let (val, q) = q.unsnoc().unwrap();
		assert_eq!(val, 3);
		let (val, q) = q.uncons().unwrap();
		assert_eq!(val, 1);
		let (val, q) = q.unsnoc().unwrap();
		assert_eq!(val, 2);
		assert!(q.is_empty());
	}

	/// Tests the iterator implementation.
	/// We construct a queue with elements added to both ends.
	/// We verify that the iterator yields elements in the correct order (from front to back).
	#[test]
	fn test_iter() {
		let mut q = CatQueue::empty();
		q = q.cons(1).snoc(2).cons(0).snoc(3);
		// Queue: [0, 1, 2, 3]

		let vec: Vec<_> = q.iter().cloned().collect();
		assert_eq!(vec, vec![0, 1, 2, 3]);
	}

	/// Tests length updates for queue operations.
	/// We verify that cons and snoc increase length by 1.
	/// We verify that uncons and unsnoc decrease length by 1.
	#[test]
	fn test_queue_len_ops() {
		let mut q = CatQueue::empty();
		assert_eq!(q.len(), 0);

		q = q.cons(1);
		assert_eq!(q.len(), 1);

		q = q.snoc(2);
		assert_eq!(q.len(), 2);

		let (_, q_next) = q.uncons().unwrap();
		q = q_next;
		assert_eq!(q.len(), 1);

		let (_, q_next) = q.unsnoc().unwrap();
		q = q_next;
		assert_eq!(q.len(), 0);
	}

	/// Tests queue edge cases.
	/// We verify uncons/unsnoc on empty queue return None.
	#[test]
	fn test_queue_edge_cases() {
		let q: CatQueue<i32> = CatQueue::empty();
		assert!(q.clone().uncons().is_none());
		assert!(q.unsnoc().is_none());
	}
}
