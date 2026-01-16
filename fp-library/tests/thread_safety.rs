use fp_library::{
	brands::{ArcFnBrand, VecBrand},
	classes::{par_foldable::par_fold_map, send_clonable_fn::new_send},
};
use std::thread;

#[test]
fn test_spawn_thread_with_send_fn() {
	let f = new_send::<ArcFnBrand, _, _>(|x: i32| x * 2);
	let handle = thread::spawn(move || f(21));
	assert_eq!(handle.join().unwrap(), 42);
}

#[test]
fn test_share_send_fn_across_threads() {
	let f = new_send::<ArcFnBrand, _, _>(|x: i32| x * 2);
	let f_clone1 = f.clone();
	let f_clone2 = f.clone();

	let handle1 = thread::spawn(move || f_clone1(10));
	let handle2 = thread::spawn(move || f_clone2(20));

	assert_eq!(handle1.join().unwrap(), 20);
	assert_eq!(handle2.join().unwrap(), 40);
	assert_eq!(f(30), 60);
}

#[test]
fn test_par_foldable_in_thread() {
	let v = vec![1, 2, 3, 4, 5];
	let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());

	let handle = thread::spawn(move || par_fold_map::<ArcFnBrand, VecBrand, _, _>(v, f));

	assert_eq!(handle.join().unwrap(), "12345".to_string());
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Sum(i32);

impl fp_library::classes::semigroup::Semigroup for Sum {
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		Sum(a.0 + b.0)
	}
}

impl fp_library::classes::monoid::Monoid for Sum {
	fn empty() -> Self {
		Sum(0)
	}
}

#[test]
fn test_par_foldable_concurrent_access() {
	let v = vec![1, 2, 3];
	let f = new_send::<ArcFnBrand, _, _>(|x: i32| Sum(x * 2));
	let f_clone = f.clone();
	let v_clone = v.clone();

	// Thread 1 uses f to fold v
	let handle1 =
		thread::spawn(move || par_fold_map::<ArcFnBrand, VecBrand, _, _>(v_clone, f_clone));

	// Main thread uses f to fold v
	let result2 = par_fold_map::<ArcFnBrand, VecBrand, _, _>(v, f);

	// Wait for thread 1
	let result1 = handle1.join().unwrap();

	assert_eq!(result1, Sum(12));
	assert_eq!(result2, Sum(12));
}
