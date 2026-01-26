use fp_library::types::cat_list::CatList;
use quickcheck_macros::quickcheck;
use std::collections::LinkedList;

// =========================================================================
// CatList Property Tests
// =========================================================================

/// Property: cons adds element to the front
/// Verifies that `cons` adds an element to the beginning of the list.
/// We compare against a standard Vec implementation.
#[quickcheck]
fn prop_cons_adds_to_front(
	head: i32,
	tail: Vec<i32>,
) -> bool {
	let list: CatList<_> = tail.iter().cloned().collect();
	let list = list.cons(head);

	let mut expected = vec![head];
	expected.extend(tail);

	let result: Vec<_> = list.into_iter().collect();
	result == expected
}

/// Property: snoc adds element to the back
/// Verifies that `snoc` adds an element to the end of the list.
/// We compare against a standard Vec implementation.
#[quickcheck]
fn prop_snoc_adds_to_back(
	init: Vec<i32>,
	last: i32,
) -> bool {
	let list: CatList<_> = init.iter().cloned().collect();
	let list = list.snoc(last);

	let mut expected = init;
	expected.push(last);

	let result: Vec<_> = list.into_iter().collect();
	result == expected
}

/// Property: append concatenates two lists
/// Verifies that `append` correctly concatenates two lists.
/// We compare against Vec concatenation.
#[quickcheck]
fn prop_append_concatenates(
	xs: Vec<i32>,
	ys: Vec<i32>,
) -> bool {
	let list1: CatList<_> = xs.iter().cloned().collect();
	let list2: CatList<_> = ys.iter().cloned().collect();
	let list3 = list1.append(list2);

	let mut expected = xs;
	expected.extend(ys);

	let result: Vec<_> = list3.into_iter().collect();
	result == expected
}

/// Property: append is associative
/// Verifies that `(xs ++ ys) ++ zs == xs ++ (ys ++ zs)`.
#[quickcheck]
fn prop_append_associative(
	xs: Vec<i32>,
	ys: Vec<i32>,
	zs: Vec<i32>,
) -> bool {
	let l1: CatList<_> = xs.iter().cloned().collect();
	let l2: CatList<_> = ys.iter().cloned().collect();
	let l3: CatList<_> = zs.iter().cloned().collect();

	let left = l1.clone().append(l2.clone()).append(l3.clone());
	let right = l1.append(l2.append(l3));

	let r1: Vec<_> = left.into_iter().collect();
	let r2: Vec<_> = right.into_iter().collect();
	r1 == r2
}

/// Property: append with empty is identity
/// Verifies that `empty ++ xs == xs` and `xs ++ empty == xs`.
#[quickcheck]
fn prop_append_identity(xs: Vec<i32>) -> bool {
	let list: CatList<_> = xs.iter().cloned().collect();
	let empty = CatList::empty();

	let left = empty.clone().append(list.clone());
	let right = list.clone().append(empty);

	let r1: Vec<_> = left.into_iter().collect();
	let r2: Vec<_> = right.into_iter().collect();

	r1 == xs && r2 == xs
}

/// Property: len returns correct length
/// Verifies that `len()` returns the number of elements in the list.
#[quickcheck]
fn prop_len_correct(xs: Vec<i32>) -> bool {
	let list: CatList<_> = xs.iter().cloned().collect();
	list.len() == xs.len()
}

/// Property: is_empty is true iff length is 0
/// Verifies that `is_empty()` returns true if and only if the list has no elements.
#[quickcheck]
fn prop_is_empty_correct(xs: Vec<i32>) -> bool {
	let list: CatList<_> = xs.iter().cloned().collect();
	list.is_empty() == xs.is_empty()
}

/// Property: uncons returns head and tail
/// Verifies that `uncons` correctly splits the list into head and tail.
#[quickcheck]
fn prop_uncons_correct(xs: Vec<i32>) -> bool {
	let list: CatList<_> = xs.iter().cloned().collect();

	match list.uncons() {
		None => xs.is_empty(),
		Some((head, tail)) => {
			if xs.is_empty() {
				false
			} else {
				let expected_head = xs[0];
				let expected_tail = &xs[1..];
				let tail_vec: Vec<_> = tail.into_iter().collect();

				head == expected_head && tail_vec == expected_tail
			}
		}
	}
}

/// Property: cons increases length by 1
/// Verifies that `cons` increases the length of the list by exactly 1.
#[quickcheck]
fn prop_cons_increases_len(
	head: i32,
	tail: Vec<i32>,
) -> bool {
	let list: CatList<_> = tail.iter().cloned().collect();
	let initial_len = list.len();
	let list = list.cons(head);
	list.len() == initial_len + 1
}

/// Property: snoc increases length by 1
/// Verifies that `snoc` increases the length of the list by exactly 1.
#[quickcheck]
fn prop_snoc_increases_len(
	init: Vec<i32>,
	last: i32,
) -> bool {
	let list: CatList<_> = init.iter().cloned().collect();
	let initial_len = list.len();
	let list = list.snoc(last);
	list.len() == initial_len + 1
}

/// Property: append sums lengths
/// Verifies that the length of the concatenated list is the sum of the lengths of the input lists.
#[quickcheck]
fn prop_append_sums_len(
	xs: Vec<i32>,
	ys: Vec<i32>,
) -> bool {
	let list1: CatList<_> = xs.iter().cloned().collect();
	let list2: CatList<_> = ys.iter().cloned().collect();
	let len1 = list1.len();
	let len2 = list2.len();

	let list3 = list1.append(list2);
	list3.len() == len1 + len2
}

/// Property: uncons decreases length by 1
/// Verifies that `uncons` decreases the length of the list by exactly 1 (if not empty).
#[quickcheck]
fn prop_uncons_decreases_len(xs: Vec<i32>) -> bool {
	let list: CatList<_> = xs.iter().cloned().collect();
	let initial_len = list.len();

	match list.uncons() {
		None => initial_len == 0,
		Some((_, tail)) => tail.len() == initial_len - 1,
	}
}

// =========================================================================
// Equivalence Tests (LinkedList)
// =========================================================================

/// Property: cons equivalence with LinkedList
/// Verifies that building a list via `cons` produces the same result as `LinkedList::push_front`.
#[quickcheck]
fn prop_cons_equivalence_linked_list(xs: Vec<i32>) -> bool {
	let mut cat_list = CatList::empty();
	let mut linked_list = LinkedList::new();

	for &x in &xs {
		cat_list = cat_list.cons(x);
		linked_list.push_front(x);
	}

	let cat_vec: Vec<_> = cat_list.into_iter().collect();
	let linked_vec: Vec<_> = linked_list.into_iter().collect();

	cat_vec == linked_vec
}

/// Property: uncons equivalence with LinkedList
/// Verifies that consuming a list via `uncons` produces the same elements as `LinkedList::pop_front`.
#[quickcheck]
fn prop_uncons_equivalence_linked_list(xs: Vec<i32>) -> bool {
	let mut cat_list: CatList<_> = xs.iter().cloned().collect();
	let mut linked_list: LinkedList<_> = xs.iter().cloned().collect();

	loop {
		let cat_res = cat_list.uncons();
		let linked_res = linked_list.pop_front();

		match (cat_res, linked_res) {
			(Some((a, tail)), Some(b)) => {
				if a != b {
					return false;
				}
				cat_list = tail;
			}
			(None, None) => return true,
			_ => return false,
		}
	}
}
