//! A trait for thread-safe cloneable wrappers over closures.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{functions::*, brands::*};
//! use std::thread;
//!
//! let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
//!
//! // Can be sent to another thread
//! let handle = thread::spawn(move || {
//!     assert_eq!(f(5), 10);
//! });
//! handle.join().unwrap();
//! ```

use super::cloneable_fn::CloneableFn;
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
use std::ops::Deref;

/// Abstraction for thread-safe cloneable wrappers over closures.
///
/// This trait extends [`CloneableFn`] to enforce `Send + Sync` bounds on the
/// wrapped closure and the wrapper itself. This is implemented by types like
/// [`ArcFnBrand`][crate::brands::ArcFnBrand] but not [`RcFnBrand`][crate::brands::RcFnBrand].
///
/// The lifetime `'a` ensures the function doesn't outlive referenced data,
/// while generic types `A` and `B` represent the input and output types, respectively.
pub trait SendCloneableFn: CloneableFn {
	type SendOf<'a, A, B>: Clone + Send + Sync + Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>;

	/// Creates a new thread-safe cloneable function wrapper.
	///
	/// This method wraps a closure into a thread-safe cloneable function wrapper.
	///
	/// ### Type Signature
	///
	/// `forall a b. SendCloneableFn f => (a -> b) -> f a b`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to wrap. Must be `Send + Sync`.", "Undocumented")]
	/// ### Returns
	///
	/// The wrapped thread-safe cloneable function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, brands::*};
	/// use std::thread;
	///
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
	///
	/// // Can be sent to another thread
	/// let handle = thread::spawn(move || {
	///     assert_eq!(f(5), 10);
	/// });
	/// handle.join().unwrap();
	/// ```
	fn send_cloneable_fn_new<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> <Self as SendCloneableFn>::SendOf<'a, A, B>;
}

/// Creates a new thread-safe cloneable function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`SendCloneableFn::send_cloneable_fn_new`].
///
/// ### Type Signature
///
#[hm_signature(SendCloneableFn)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of the thread-safe cloneable function wrapper.",
	"The input type of the function.",
	"The output type of the function."
)]
///
/// ### Parameters
///
#[doc_params("The closure to wrap. Must be `Send + Sync`.", "Undocumented")]
/// ### Returns
///
/// The wrapped thread-safe cloneable function.
///
/// ### Examples
///
/// ```
/// use fp_library::{functions::*, brands::*};
/// use std::thread;
///
/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
///
/// // Can be sent to another thread
/// let handle = thread::spawn(move || {
///     assert_eq!(f(5), 10);
/// });
/// handle.join().unwrap();
/// ```
pub fn new<'a, Brand, A, B>(
	f: impl 'a + Fn(A) -> B + Send + Sync
) -> <Brand as SendCloneableFn>::SendOf<'a, A, B>
where
	Brand: SendCloneableFn,
{
	<Brand as SendCloneableFn>::send_cloneable_fn_new(f)
}
