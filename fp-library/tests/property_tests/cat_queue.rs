use fp_library::types::cat_queue::CatQueue;
use quickcheck_macros::quickcheck;
use std::collections::VecDeque;

// =========================================================================
// CatQueue Property Tests
// =========================================================================

/// Property: cons adds element to the front
/// Verifies that `cons` adds an element to the front of the queue.
/// We compare against a VecDeque-like behavior using Vec.
#[quickcheck]
fn prop_cons_adds_to_front(
	head: i32,
	tail: Vec<i32>,
) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &tail {
		q = q.snoc(x);
	}
	q = q.cons(head);

	let mut expected = vec![head];
	expected.extend(tail);

	let result: Vec<_> = q.into_iter().collect();
	result == expected
}

/// Property: snoc adds element to the back
/// Verifies that `snoc` adds an element to the back of the queue.
#[quickcheck]
fn prop_snoc_adds_to_back(
	init: Vec<i32>,
	last: i32,
) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &init {
		q = q.snoc(x);
	}
	q = q.snoc(last);

	let mut expected = init;
	expected.push(last);

	let result: Vec<_> = q.into_iter().collect();
	result == expected
}

/// Property: uncons removes from front (FIFO)
/// Verifies that `uncons` removes the first element added.
#[quickcheck]
fn prop_uncons_fifo(xs: Vec<i32>) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &xs {
		q = q.snoc(x);
	}

	match q.uncons() {
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

/// Property: unsnoc removes from back (LIFO)
/// Verifies that `unsnoc` removes the last element added.
#[quickcheck]
fn prop_unsnoc_lifo(xs: Vec<i32>) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &xs {
		q = q.snoc(x);
	}

	match q.unsnoc() {
		None => xs.is_empty(),
		Some((last, rest)) => {
			if xs.is_empty() {
				false
			} else {
				let expected_last = xs[xs.len() - 1];
				let expected_rest = &xs[..xs.len() - 1];
				let rest_vec: Vec<_> = rest.into_iter().collect();

				last == expected_last && rest_vec == expected_rest
			}
		}
	}
}

/// Property: len returns correct length
/// Verifies that `len()` returns the number of elements in the queue.
#[quickcheck]
fn prop_len_correct(xs: Vec<i32>) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &xs {
		q = q.snoc(x);
	}
	q.len() == xs.len()
}

/// Property: is_empty is true iff length is 0
/// Verifies that `is_empty()` returns true if and only if the queue has no elements.
#[quickcheck]
fn prop_is_empty_correct(xs: Vec<i32>) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &xs {
		q = q.snoc(x);
	}
	q.is_empty() == xs.is_empty()
}

/// Property: cons then uncons is identity for non-empty queue
/// Verifies that `uncons(cons(x, q))` yields `x` and `q`.
#[quickcheck]
fn prop_cons_uncons_identity(
	x: i32,
	xs: Vec<i32>,
) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &val in &xs {
		q = q.snoc(val);
	}

	let q_cons = q.clone().cons(x);
	let (head, tail) = q_cons.uncons().unwrap();

	head == x && tail.into_iter().collect::<Vec<_>>() == q.into_iter().collect::<Vec<_>>()
}

/// Property: snoc then unsnoc is identity for non-empty queue
/// Verifies that `unsnoc(snoc(q, x))` yields `x` and `q`.
#[quickcheck]
fn prop_snoc_unsnoc_identity(
	xs: Vec<i32>,
	x: i32,
) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &val in &xs {
		q = q.snoc(val);
	}

	let q_snoc = q.clone().snoc(x);
	let (last, rest) = q_snoc.unsnoc().unwrap();

	last == x && rest.into_iter().collect::<Vec<_>>() == q.into_iter().collect::<Vec<_>>()
}

/// Property: cons increases length by 1
/// Verifies that `cons` increases the length of the queue by exactly 1.
#[quickcheck]
fn prop_cons_increases_len(
	head: i32,
	tail: Vec<i32>,
) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &tail {
		q = q.snoc(x);
	}
	let initial_len = q.len();
	q = q.cons(head);
	q.len() == initial_len + 1
}

/// Property: snoc increases length by 1
/// Verifies that `snoc` increases the length of the queue by exactly 1.
#[quickcheck]
fn prop_snoc_increases_len(
	init: Vec<i32>,
	last: i32,
) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &init {
		q = q.snoc(x);
	}
	let initial_len = q.len();
	q = q.snoc(last);
	q.len() == initial_len + 1
}

/// Property: uncons decreases length by 1
/// Verifies that `uncons` decreases the length of the queue by exactly 1 (if not empty).
#[quickcheck]
fn prop_uncons_decreases_len(xs: Vec<i32>) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &xs {
		q = q.snoc(x);
	}
	let initial_len = q.len();

	match q.uncons() {
		None => initial_len == 0,
		Some((_, rest)) => rest.len() == initial_len - 1,
	}
}

/// Property: unsnoc decreases length by 1
/// Verifies that `unsnoc` decreases the length of the queue by exactly 1 (if not empty).
#[quickcheck]
fn prop_unsnoc_decreases_len(xs: Vec<i32>) -> bool {
	let mut q: CatQueue<_> = CatQueue::empty();
	for &x in &xs {
		q = q.snoc(x);
	}
	let initial_len = q.len();

	match q.unsnoc() {
		None => initial_len == 0,
		Some((_, rest)) => rest.len() == initial_len - 1,
	}
}

// =========================================================================
// Equivalence Tests (VecDeque)
// =========================================================================

/// Property: cons equivalence with VecDeque
/// Verifies that building a queue via `cons` produces the same result as `VecDeque::push_front`.
#[quickcheck]
fn prop_cons_equivalence_vec_deque(xs: Vec<i32>) -> bool {
	let mut cat_queue = CatQueue::empty();
	let mut vec_deque = VecDeque::new();

	for &x in &xs {
		cat_queue = cat_queue.cons(x);
		vec_deque.push_front(x);
	}

	let cat_vec: Vec<_> = cat_queue.into_iter().collect();
	let deque_vec: Vec<_> = vec_deque.into_iter().collect();

	cat_vec == deque_vec
}

/// Property: snoc equivalence with VecDeque
/// Verifies that building a queue via `snoc` produces the same result as `VecDeque::push_back`.
#[quickcheck]
fn prop_snoc_equivalence_vec_deque(xs: Vec<i32>) -> bool {
	let mut cat_queue = CatQueue::empty();
	let mut vec_deque = VecDeque::new();

	for &x in &xs {
		cat_queue = cat_queue.snoc(x);
		vec_deque.push_back(x);
	}

	let cat_vec: Vec<_> = cat_queue.into_iter().collect();
	let deque_vec: Vec<_> = vec_deque.into_iter().collect();

	cat_vec == deque_vec
}

/// Property: uncons equivalence with VecDeque
/// Verifies that consuming a queue via `uncons` produces the same elements as `VecDeque::pop_front`.
#[quickcheck]
fn prop_uncons_equivalence_vec_deque(xs: Vec<i32>) -> bool {
	let mut cat_queue: CatQueue<_> = CatQueue::empty();
	let mut vec_deque: VecDeque<_> = VecDeque::new();

	// Fill both
	for &x in &xs {
		cat_queue = cat_queue.snoc(x);
		vec_deque.push_back(x);
	}

	loop {
		let cat_res = cat_queue.uncons();
		let deque_res = vec_deque.pop_front();

		match (cat_res, deque_res) {
			(Some((a, tail)), Some(b)) => {
				if a != b {
					return false;
				}
				cat_queue = tail;
			}
			(None, None) => return true,
			_ => return false,
		}
	}
}

/// Property: unsnoc equivalence with VecDeque
/// Verifies that consuming a queue via `unsnoc` produces the same elements as `VecDeque::pop_back`.
#[quickcheck]
fn prop_unsnoc_equivalence_vec_deque(xs: Vec<i32>) -> bool {
	let mut cat_queue: CatQueue<_> = CatQueue::empty();
	let mut vec_deque: VecDeque<_> = VecDeque::new();

	// Fill both
	for &x in &xs {
		cat_queue = cat_queue.snoc(x);
		vec_deque.push_back(x);
	}

	loop {
		let cat_res = cat_queue.unsnoc();
		let deque_res = vec_deque.pop_back();

		match (cat_res, deque_res) {
			(Some((a, tail)), Some(b)) => {
				if a != b {
					return false;
				}
				cat_queue = tail;
			}
			(None, None) => return true,
			_ => return false,
		}
	}
}
