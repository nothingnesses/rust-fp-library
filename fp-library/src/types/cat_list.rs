//! A catenable list with O(1) append and O(1) amortized uncons.
//!
//! This module provides [`CatList`], the "Reflection without Remorse" data structure that enables
//! O(1) left-associated bind operations in the Free monad.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::cat_list::CatList;
//!
//! let list = CatList::singleton(1)
//!     .snoc(2)
//!     .snoc(3)
//!     .append(CatList::singleton(4));
//!
//! let mut result = Vec::new();
//! let mut current = list;
//! while let Some((head, tail)) = current.uncons() {
//!     result.push(head);
//!     current = tail;
//! }
//! assert_eq!(result, vec![1, 2, 3, 4]);
//! ```

use crate::types::cat_queue::CatQueue;

/// A catenable list with O(1) append and O(1) amortized uncons.
///
/// This is the "Reflection without Remorse" data structure that enables
/// O(1) left-associated bind operations in the Free monad.
///
/// ### Type Parameters
///
/// * `A`: The type of the elements in the list.
///
/// ### Examples
///
/// ```
/// use fp_library::types::cat_list::CatList;
///
/// let list: CatList<i32> = CatList::empty();
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CatList<A> {
	/// Empty list
	#[default]
	Nil,
	/// Head element plus queue of sublists and total length
	Cons(A, CatQueue<CatList<A>>, usize),
}

impl<A> CatList<A> {
	/// Creates an empty CatList.
	///
	/// ### Type Signature
	///
	/// `forall a. () -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the list.
	///
	/// ### Returns
	///
	/// An empty `CatList`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list: CatList<i32> = CatList::empty();
	/// assert!(list.is_empty());
	/// ```
	#[inline]
	pub const fn empty() -> Self {
		CatList::Nil
	}

	/// Returns `true` if the list is empty.
	///
	/// ### Type Signature
	///
	/// `forall a. CatList a -> bool`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the list.
	///
	/// ### Parameters
	///
	/// * `self`: The list to check.
	///
	/// ### Returns
	///
	/// `true` if the list is empty, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list: CatList<i32> = CatList::empty();
	/// assert!(list.is_empty());
	/// ```
	#[inline]
	pub fn is_empty(&self) -> bool {
		matches!(self, CatList::Nil)
	}

	/// Creates a CatList with a single element.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `a`: The element to put in the list.
	///
	/// ### Returns
	///
	/// A `CatList` containing the single element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::singleton(1);
	/// assert!(!list.is_empty());
	/// ```
	#[inline]
	pub fn singleton(a: A) -> Self {
		CatList::Cons(a, CatQueue::empty(), 1)
	}

	/// Appends an element to the front of the list.
	///
	/// ### Type Signature
	///
	/// `forall a. (CatList a, a) -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	/// * `a`: The element to append.
	///
	/// ### Returns
	///
	/// The new list with the element appended to the front.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::empty().cons(1);
	/// ```
	#[inline]
	pub fn cons(
		self,
		a: A,
	) -> Self {
		Self::link(CatList::singleton(a), self)
	}

	/// Appends an element to the back of the list.
	///
	/// ### Type Signature
	///
	/// `forall a. (CatList a, a) -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the element.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	/// * `a`: The element to append.
	///
	/// ### Returns
	///
	/// The new list with the element appended to the back.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::empty().snoc(1);
	/// ```
	#[inline]
	pub fn snoc(
		self,
		a: A,
	) -> Self {
		Self::link(self, CatList::singleton(a))
	}

	/// Concatenates two CatLists.
	///
	/// This is the key operation that makes CatList special:
	/// concatenation is O(1), not O(n).
	///
	/// ### Type Signature
	///
	/// `forall a. (CatList a, CatList a) -> CatList a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The first list.
	/// * `other`: The second list.
	///
	/// ### Returns
	///
	/// The concatenated list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list1 = CatList::singleton(1);
	/// let list2 = CatList::singleton(2);
	/// let list3 = list1.append(list2);
	/// ```
	pub fn append(
		self,
		other: Self,
	) -> Self {
		Self::link(self, other)
	}

	/// Internal linking operation.
	///
	/// Links two CatLists by enqueueing the second onto the first's sublist queue.
	fn link(
		left: Self,
		right: Self,
	) -> Self {
		match (left, right) {
			(CatList::Nil, cat) => cat,
			(cat, CatList::Nil) => cat,
			(CatList::Cons(a, q, len), cat) => {
				let new_len = len + cat.len();
				CatList::Cons(a, q.snoc(cat), new_len)
			}
		}
	}

	/// Removes and returns the first element.
	///
	/// Returns `None` if the list is empty.
	///
	/// ### Type Signature
	///
	/// `forall a. CatList a -> Option (a, CatList a)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	///
	/// ### Returns
	///
	/// An option containing the first element and the rest of the list, or `None` if empty.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::singleton(1);
	/// let (a, list) = list.uncons().unwrap();
	/// assert_eq!(a, 1);
	/// assert!(list.is_empty());
	/// ```
	pub fn uncons(self) -> Option<(A, Self)> {
		match self {
			CatList::Nil => None,
			CatList::Cons(a, q, _) => {
				if q.is_empty() {
					Some((a, CatList::Nil))
				} else {
					// Flatten the queue of sublists into a single CatList
					let tail = Self::flatten_queue(q);
					Some((a, tail))
				}
			}
		}
	}

	/// Flattens a queue of CatLists into a single CatList.
	///
	/// This is equivalent to `foldr link CatNil queue` in PureScript.
	///
	/// We use an iterative approach with an explicit stack to avoid
	/// stack overflow on deeply nested structures.
	fn flatten_queue(queue: CatQueue<CatList<A>>) -> Self {
		// Collect all sublists
		let sublists: Vec<CatList<A>> = queue.into_iter().collect();

		// Right fold: link(list[0], link(list[1], ... link(list[n-1], Nil)))
		// We process from right to left
		sublists.into_iter().rev().fold(CatList::Nil, |acc, list| Self::link(list, acc))
	}

	/// Returns the number of elements.
	///
	/// ### Type Signature
	///
	/// `forall a. CatList a -> usize`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements.
	///
	/// ### Parameters
	///
	/// * `self`: The list.
	///
	/// ### Returns
	///
	/// The number of elements in the list.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::cat_list::CatList;
	///
	/// let list = CatList::singleton(1);
	/// assert_eq!(list.len(), 1);
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		match self {
			CatList::Nil => 0,
			CatList::Cons(_, _, len) => *len,
		}
	}
}

// Iteration support
impl<A> IntoIterator for CatList<A> {
	type Item = A;
	type IntoIter = CatListIter<A>;

	fn into_iter(self) -> Self::IntoIter {
		CatListIter { list: self }
	}
}

/// An iterator that consumes a `CatList`.
///
/// ### Type Parameters
///
/// * `A`: The type of the elements in the list.
///
/// ### Fields
///
/// * `list`: The list being iterated over.
pub struct CatListIter<A> {
	list: CatList<A>,
}

impl<A> Iterator for CatListIter<A> {
	type Item = A;

	fn next(&mut self) -> Option<Self::Item> {
		let (head, tail) = std::mem::take(&mut self.list).uncons()?;
		self.list = tail;
		Some(head)
	}
}

// FromIterator for easy construction
impl<A> FromIterator<A> for CatList<A> {
	fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
		iter.into_iter().fold(CatList::Nil, |acc, a| acc.snoc(a))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests basic list operations: creation, emptiness check, and length.
	/// This ensures that a new list is empty and has length 0, and a singleton list is not empty and has length 1.
	#[test]
	fn test_basic_operations() {
		let list: CatList<i32> = CatList::empty();
		assert!(list.is_empty());
		assert_eq!(list.len(), 0);

		let list = CatList::singleton(1);
		assert!(!list.is_empty());
		assert_eq!(list.len(), 1);
	}

	/// Tests the concatenation of two lists.
	/// We create two lists and append them.
	/// We verify that the resulting list contains all elements in the correct order.
	#[test]
	fn test_concatenation() {
		let list1 = CatList::singleton(1).snoc(2);
		let list2 = CatList::singleton(3).snoc(4);
		let list3 = list1.append(list2);

		let vec: Vec<_> = list3.into_iter().collect();
		assert_eq!(vec, vec![1, 2, 3, 4]);
	}

	/// Tests the flattening of nested lists.
	/// We create a nested structure by appending multiple lists: ((1 ++ 2) ++ (3 ++ 4)).
	/// This exercises the `flatten_queue` logic in `uncons`.
	/// We verify that the list is flattened correctly and elements are retrieved in order.
	#[test]
	fn test_flattening() {
		// Create a nested structure: ((1 ++ 2) ++ (3 ++ 4))
		let l1 = CatList::singleton(1);
		let l2 = CatList::singleton(2);
		let l3 = CatList::singleton(3);
		let l4 = CatList::singleton(4);

		let left = l1.append(l2);
		let right = l3.append(l4);
		let combined = left.append(right);

		assert_eq!(combined.len(), 4);
		let vec: Vec<_> = combined.into_iter().collect();
		assert_eq!(vec, vec![1, 2, 3, 4]);
	}

	/// Tests the iterator implementation.
	/// We create a list from a range and collect it back into a vector.
	/// We verify that the iterator yields all elements in the correct order.
	#[test]
	fn test_iteration() {
		let list: CatList<_> = (0..10).collect();
		let vec: Vec<_> = list.into_iter().collect();
		assert_eq!(vec, (0..10).collect::<Vec<_>>());
	}

	/// Tests the O(1) length tracking.
	/// We create a list with 100 elements.
	/// We verify that the length is reported correctly as 100.
	#[test]
	fn test_len() {
		let list: CatList<_> = (0..100).collect();
		assert_eq!(list.len(), 100);
	}

	/// Tests that cons increases length by 1.
	/// We start with an empty list and verify its length is 0.
	/// Then we prepend an element using `cons` and verify the length becomes 1.
	/// Finally, we prepend another element and verify the length becomes 2.
	#[test]
	fn test_len_cons() {
		let list = CatList::empty();
		assert_eq!(list.len(), 0);
		let list = list.cons(1);
		assert_eq!(list.len(), 1);
		let list = list.cons(2);
		assert_eq!(list.len(), 2);
	}

	/// Tests that snoc increases length by 1.
	/// We start with an empty list and verify its length is 0.
	/// Then we append an element using `snoc` and verify the length becomes 1.
	/// Finally, we append another element and verify the length becomes 2.
	#[test]
	fn test_len_snoc() {
		let list = CatList::empty();
		assert_eq!(list.len(), 0);
		let list = list.snoc(1);
		assert_eq!(list.len(), 1);
		let list = list.snoc(2);
		assert_eq!(list.len(), 2);
	}

	/// Tests that append results in sum of lengths.
	/// We create two lists: one with length 2 and another with length 3.
	/// We verify their individual lengths.
	/// Then we append the second list to the first and verify the resulting list has length 5 (2 + 3).
	#[test]
	fn test_len_append() {
		let list1 = CatList::singleton(1).snoc(2);
		let list2 = CatList::singleton(3).snoc(4).snoc(5);
		assert_eq!(list1.len(), 2);
		assert_eq!(list2.len(), 3);

		let list3 = list1.append(list2);
		assert_eq!(list3.len(), 5);
	}

	/// Tests that uncons decreases length by 1.
	/// We create a list with 3 elements and verify its length.
	/// We repeatedly call `uncons` to remove elements from the front.
	/// After each `uncons`, we verify that the length of the remaining tail decreases by 1,
	/// until the list is empty (length 0).
	#[test]
	fn test_len_uncons() {
		let list = CatList::singleton(1).snoc(2).snoc(3);
		assert_eq!(list.len(), 3);

		let (_, tail) = list.uncons().unwrap();
		assert_eq!(tail.len(), 2);

		let (_, tail) = tail.uncons().unwrap();
		assert_eq!(tail.len(), 1);

		let (_, tail) = tail.uncons().unwrap();
		assert_eq!(tail.len(), 0);
	}

	/// Tests appending empty lists.
	/// We verify that appending an empty list to a non-empty list (and vice versa)
	/// preserves the non-empty list's content and length.
	/// We also verify that appending two empty lists results in an empty list.
	#[test]
	fn test_append_empty() {
		let empty: CatList<i32> = CatList::empty();
		let list = CatList::singleton(1);

		// empty ++ list
		let res = empty.clone().append(list.clone());
		assert_eq!(res.len(), 1);
		assert_eq!(res.into_iter().collect::<Vec<_>>(), vec![1]);

		// list ++ empty
		let res = list.clone().append(empty.clone());
		assert_eq!(res.len(), 1);
		assert_eq!(res.into_iter().collect::<Vec<_>>(), vec![1]);

		// empty ++ empty
		let res = empty.clone().append(empty);
		assert_eq!(res.len(), 0);
		assert!(res.is_empty());
	}

	/// Tests uncons edge cases.
	/// We verify that uncons on an empty list returns None.
	/// We verify that uncons on a singleton list returns the element and an empty tail.
	#[test]
	fn test_uncons_edge_cases() {
		let empty: CatList<i32> = CatList::empty();
		assert!(empty.uncons().is_none());

		let list = CatList::singleton(1);
		let (head, tail) = list.uncons().unwrap();
		assert_eq!(head, 1);
		assert!(tail.is_empty());
		assert_eq!(tail.len(), 0);
	}
}
