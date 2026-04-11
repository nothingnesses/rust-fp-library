//! Feasibility tests for changing Ref trait methods from consuming to borrowing containers.
//!
//! These tests verify that the key lifetime and ownership patterns work correctly
//! when container parameters are changed from `fa: T` to `fa: &T`.

#[cfg(test)]
mod tests {
	use std::{
		cell::OnceCell,
		rc::Rc,
	};

	// -- Minimal Lazy implementation for testing --

	struct Lazy<A> {
		inner: Rc<OnceCell<A>>,
		thunk: Rc<dyn Fn() -> A>,
	}

	impl<A> Clone for Lazy<A> {
		fn clone(&self) -> Self {
			Lazy {
				inner: self.inner.clone(),
				thunk: self.thunk.clone(),
			}
		}
	}

	impl<A> Lazy<A> {
		fn new(f: impl Fn() -> A + 'static) -> Self {
			Lazy {
				inner: Rc::new(OnceCell::new()),
				thunk: Rc::new(f),
			}
		}

		fn evaluate(&self) -> &A {
			self.inner.get_or_init(|| (self.thunk)())
		}
	}

	// ================================================================
	// Test 1: Lazy clone-and-capture pattern (ref_map)
	// ================================================================

	#[test]
	fn lazy_ref_map_from_borrow() {
		fn ref_map(
			fa: &Lazy<i32>,
			f: impl Fn(&i32) -> String + 'static,
		) -> Lazy<String> {
			let fa = fa.clone();
			Lazy::new(move || f(fa.evaluate()))
		}

		let original = Lazy::new(|| 42);
		let mapped = ref_map(&original, |x| format!("value: {}", x));

		assert_eq!(*original.evaluate(), 42);
		assert_eq!(mapped.evaluate(), "value: 42");
	}

	// ================================================================
	// Test 2: Lazy ref_lift2 with two borrowed containers
	// ================================================================

	#[test]
	fn lazy_ref_lift2_from_borrows() {
		fn ref_lift2(
			fa: &Lazy<i32>,
			fb: &Lazy<i32>,
			f: impl Fn(&i32, &i32) -> i32 + 'static,
		) -> Lazy<i32> {
			let fa = fa.clone();
			let fb = fb.clone();
			Lazy::new(move || f(fa.evaluate(), fb.evaluate()))
		}

		let a = Lazy::new(|| 10);
		let b = Lazy::new(|| 20);
		let c = ref_lift2(&a, &b, |x, y| x + y);

		assert_eq!(*a.evaluate(), 10);
		assert_eq!(*b.evaluate(), 20);
		assert_eq!(*c.evaluate(), 30);
	}

	// ================================================================
	// Test 3: Lazy ref_bind from borrow
	// The closure receives &A and must produce an owned Lazy<B>.
	// The user must dereference/clone the value if they want to capture it.
	// ================================================================

	#[test]
	fn lazy_ref_bind_from_borrow() {
		fn ref_bind(
			fa: &Lazy<i32>,
			f: impl Fn(&i32) -> Lazy<String>,
		) -> Lazy<String> {
			f(fa.evaluate())
		}

		let a = Lazy::new(|| 42);
		// The closure receives &i32 and must produce Lazy<String>.
		// It dereferences the value to capture it in the new Lazy's closure.
		let b = ref_bind(&a, |x| {
			let x = *x; // dereference to own
			Lazy::new(move || format!("got: {}", x))
		});

		assert_eq!(*a.evaluate(), 42);
		assert_eq!(b.evaluate(), "got: 42");
	}

	// ================================================================
	// Test 4: Lazy ref_fold_map from borrow
	// ================================================================

	#[test]
	fn lazy_ref_fold_map_from_borrow() {
		fn ref_fold_map(
			fa: &Lazy<i32>,
			f: impl Fn(&i32) -> String,
		) -> String {
			f(fa.evaluate())
		}

		let a = Lazy::new(|| 42);
		let result = ref_fold_map(&a, |x| x.to_string());
		assert_eq!(result, "42");
		assert_eq!(*a.evaluate(), 42);
	}

	// ================================================================
	// Test 5: Vec borrow patterns
	// ================================================================

	#[test]
	fn vec_ref_map_from_borrow() {
		fn ref_map(
			fa: &[i32],
			f: impl Fn(&i32) -> String,
		) -> Vec<String> {
			fa.iter().map(f).collect()
		}

		let v = vec![1, 2, 3];
		let mapped = ref_map(&v, |x| x.to_string());
		assert_eq!(mapped, vec!["1", "2", "3"]);
		assert_eq!(v, vec![1, 2, 3]);
	}

	#[test]
	fn vec_ref_bind_from_borrow() {
		fn ref_bind(
			fa: &[i32],
			f: impl Fn(&i32) -> Vec<i32>,
		) -> Vec<i32> {
			fa.iter().flat_map(f).collect()
		}

		let v = vec![1, 2, 3];
		let result = ref_bind(&v, |x| vec![*x, *x * 10]);
		assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
		assert_eq!(v, vec![1, 2, 3]);
	}

	#[test]
	fn vec_ref_lift2_from_borrows() {
		fn ref_lift2(
			fa: &[i32],
			fb: &[i32],
			f: impl Fn(&i32, &i32) -> i32 + Copy,
		) -> Vec<i32> {
			fa.iter().flat_map(|a| fb.iter().map(move |b| f(a, b))).collect()
		}

		let a = vec![1, 2];
		let b = vec![10, 20];
		let c = ref_lift2(&a, &b, |x, y| x + y);
		assert_eq!(c, vec![11, 21, 12, 22]);
		assert_eq!(a, vec![1, 2]);
		assert_eq!(b, vec![10, 20]);
	}

	// ================================================================
	// Test 6: Option borrow patterns
	// ================================================================

	#[test]
	fn option_ref_map_from_borrow() {
		fn ref_map(
			fa: &Option<i32>,
			f: impl Fn(&i32) -> String,
		) -> Option<String> {
			fa.as_ref().map(f)
		}

		let v = Some(42);
		let mapped = ref_map(&v, |x| x.to_string());
		assert_eq!(mapped, Some("42".to_string()));
		assert_eq!(v, Some(42));
	}

	#[test]
	fn option_ref_bind_from_borrow() {
		fn ref_bind(
			fa: &Option<i32>,
			f: impl Fn(&i32) -> Option<String>,
		) -> Option<String> {
			fa.as_ref().and_then(f)
		}

		let v = Some(42);
		let result = ref_bind(&v, |x| Some(x.to_string()));
		assert_eq!(result, Some("42".to_string()));
		assert_eq!(v, Some(42));
	}

	// ================================================================
	// Test 7: Dispatch adapter pattern
	// Dispatch trait takes by value, calls borrowed trait method
	// ================================================================

	trait RefFunctorBorrowed {
		fn ref_map(
			f: impl Fn(&i32) -> String,
			fa: &[i32],
		) -> Vec<String>;
	}

	struct VecBrand;
	impl RefFunctorBorrowed for VecBrand {
		fn ref_map(
			f: impl Fn(&i32) -> String,
			fa: &[i32],
		) -> Vec<String> {
			fa.iter().map(f).collect()
		}
	}

	trait FunctorDispatch {
		fn dispatch(
			self,
			fa: Vec<i32>,
		) -> Vec<String>;
	}

	impl<F: Fn(&i32) -> String> FunctorDispatch for F {
		fn dispatch(
			self,
			fa: Vec<i32>,
		) -> Vec<String> {
			VecBrand::ref_map(self, &fa)
		}
	}

	#[test]
	fn dispatch_adapter_pattern() {
		fn map(
			f: impl Fn(&i32) -> String,
			fa: Vec<i32>,
		) -> Vec<String> {
			f.dispatch(fa)
		}

		let result = map(|x: &i32| x.to_string(), vec![1, 2, 3]);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	// ================================================================
	// Test 8: Fully borrowed dispatch (free function also borrows)
	// ================================================================

	#[test]
	fn fully_borrowed_dispatch() {
		fn map_ref(
			f: impl Fn(&i32) -> String,
			fa: &[i32],
		) -> Vec<String> {
			VecBrand::ref_map(f, fa)
		}

		let v = vec![1, 2, 3];
		let a = map_ref(|x| x.to_string(), &v);
		let b = map_ref(|x| format!("{}!", x), &v);
		assert_eq!(a, vec!["1", "2", "3"]);
		assert_eq!(b, vec!["1!", "2!", "3!"]);
	}

	// ================================================================
	// Test 9: Temporary lifetime with borrowed free function
	// ================================================================

	fn make_vec() -> Vec<i32> {
		vec![1, 2, 3]
	}

	#[test]
	fn temporary_lifetime_in_argument() {
		fn ref_map(
			f: impl Fn(&i32) -> i32,
			fa: &[i32],
		) -> Vec<i32> {
			fa.iter().map(f).collect()
		}

		let result = ref_map(|x| x * 2, &make_vec());
		assert_eq!(result, vec![2, 4, 6]);
	}

	// ================================================================
	// Test 10: Nested bind with temporaries
	// ================================================================

	#[test]
	fn nested_borrowed_bind() {
		fn ref_bind(
			fa: &[i32],
			f: impl Fn(&i32) -> Vec<i32>,
		) -> Vec<i32> {
			fa.iter().flat_map(f).collect()
		}

		let v = vec![1, 2];
		let result = ref_bind(&ref_bind(&v, |x| vec![*x, *x * 10]), |y| vec![*y + 100]);
		assert_eq!(result, vec![101, 110, 102, 120]);
		assert_eq!(v, vec![1, 2]);
	}

	// ================================================================
	// Test 11: Lazy nested bind with temporaries
	// The closure must dereference/clone values to capture them
	// ================================================================

	#[test]
	fn lazy_nested_bind_temporary() {
		fn ref_bind(
			fa: &Lazy<i32>,
			f: impl Fn(&i32) -> Lazy<i32>,
		) -> Lazy<i32> {
			f(fa.evaluate())
		}

		let a = Lazy::new(|| 10);
		let result = ref_bind(
			&ref_bind(&a, |x| {
				let x = *x;
				Lazy::new(move || x + 1)
			}),
			|y| {
				let y = *y;
				Lazy::new(move || y * 2)
			},
		);
		assert_eq!(*result.evaluate(), 22); // (10 + 1) * 2
		assert_eq!(*a.evaluate(), 10);
	}

	// ================================================================
	// Test 12: Result passthrough with borrow (needs Clone on E)
	// ================================================================

	#[test]
	fn result_ref_map_borrowed() {
		fn ref_map<E: Clone>(
			fa: &Result<i32, E>,
			f: impl Fn(&i32) -> String,
		) -> Result<String, E> {
			match fa {
				Ok(a) => Ok(f(a)),
				Err(e) => Err(e.clone()),
			}
		}

		let ok: Result<i32, String> = Ok(42);
		let err: Result<i32, String> = Err("bad".to_string());

		assert_eq!(ref_map(&ok, |x| x.to_string()), Ok("42".to_string()));
		assert_eq!(ref_map(&err, |x| x.to_string()), Err("bad".to_string()));
		assert_eq!(ok, Ok(42));
		assert_eq!(err, Err("bad".to_string()));
	}

	// ================================================================
	// Test 13: Pair with borrowed fixed field (needs Clone)
	// ================================================================

	#[derive(Debug, Clone, PartialEq)]
	struct Pair<A, B>(A, B);

	#[test]
	fn pair_ref_map_borrowed() {
		fn ref_map<First: Clone>(
			fa: &Pair<First, i32>,
			f: impl Fn(&i32) -> String,
		) -> Pair<First, String> {
			Pair(fa.0.clone(), f(&fa.1))
		}

		let p = Pair("hello", 42);
		let mapped = ref_map(&p, |x| x.to_string());
		assert_eq!(mapped, Pair("hello", "42".to_string()));
		assert_eq!(p, Pair("hello", 42));
	}

	// ================================================================
	// Test 14: Fully borrowed m_do!-style chain (simulated)
	// This simulates what m_do!(ref Brand { x <- e1; y <- e2; pure(x + y) })
	// would generate if ALL functions took borrows
	// ================================================================

	#[test]
	fn simulated_m_do_fully_borrowed() {
		fn ref_bind(
			fa: &Option<i32>,
			f: impl Fn(&i32) -> Option<i32>,
		) -> Option<i32> {
			fa.as_ref().and_then(f)
		}

		fn ref_pure(a: &i32) -> Option<i32> {
			Some(*a)
		}

		// Simulating: m_do!(ref { x <- Some(5); y <- Some(10); pure(x + y) })
		// With fully borrowed signatures, we need to handle temporaries carefully
		let expr1 = Some(5);
		let expr2 = Some(10);
		let result = ref_bind(&expr1, |x| ref_bind(&expr2, move |y| ref_pure(&(x + y))));
		assert_eq!(result, Some(15));

		// But what about inline temporaries?
		let result2 = ref_bind(&Some(5), |x| ref_bind(&Some(10), move |y| ref_pure(&(x + y))));
		assert_eq!(result2, Some(15));
	}

	// ================================================================
	// Test 15: Fully borrowed a_do!-style (simulated)
	// ================================================================

	#[test]
	fn simulated_a_do_fully_borrowed() {
		fn ref_lift2(
			f: impl Fn(&i32, &i32) -> i32,
			fa: &Option<i32>,
			fb: &Option<i32>,
		) -> Option<i32> {
			match (fa, fb) {
				(Some(a), Some(b)) => Some(f(a, b)),
				_ => None,
			}
		}

		// a_do!(ref { x <- Some(5); y <- Some(10); x + y })
		let result = ref_lift2(|x, y| x + y, &Some(5), &Some(10));
		assert_eq!(result, Some(15));

		// With variables (reuse)
		let a = Some(5);
		let b = Some(10);
		let r1 = ref_lift2(|x, y| x + y, &a, &b);
		let r2 = ref_lift2(|x, y| x * y, &a, &b);
		assert_eq!(r1, Some(15));
		assert_eq!(r2, Some(50));
	}

	// ================================================================
	// Test 16: Lazy ref_bind where closure captures complex data
	// Verifies that the closure's &A reference is valid
	// ================================================================

	#[test]
	fn lazy_ref_bind_complex_closure() {
		fn ref_bind(
			fa: &Lazy<Vec<i32>>,
			f: impl Fn(&Vec<i32>) -> Lazy<i32>,
		) -> Lazy<i32> {
			f(fa.evaluate())
		}

		let a = Lazy::new(|| vec![1, 2, 3]);
		let sum = ref_bind(&a, |v| {
			let total: i32 = v.iter().sum();
			Lazy::new(move || total)
		});

		assert_eq!(*sum.evaluate(), 6);
		assert_eq!(a.evaluate(), &vec![1, 2, 3]);
	}
}
