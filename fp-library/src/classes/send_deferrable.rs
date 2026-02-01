//! A trait for deferred lazy evaluation with thread-safe thunks.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let memo: ArcLazy<i32> = send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(|| 42));
//! assert_eq!(*memo.evaluate(), 42);
//! ```

use crate::{Apply, kinds::*};
use fp_macros::hm_signature;

/// A trait for deferred lazy evaluation with thread-safe thunks.
///
/// This is similar to [`Deferrable`](crate::classes::Deferrable), but the thunk must be `Send + Sync`.
pub trait SendDeferrable: Kind_cdc7cd43dac7585f {
	/// Creates a deferred value from a thread-safe thunk.
	///
	/// ### Type Signature
	///
	/// `forall f a. (SendDeferrable f, Send a, Sync a) => (() -> a) -> f a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `thunk`: The function that produces the value.
	///
	/// ### Returns
	///
	/// A deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let memo: ArcLazy<i32> = send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(|| 42));
	/// assert_eq!(*memo.evaluate(), 42);
	/// ```
	fn send_defer<'a, A>(
		thunk: impl 'a
		+ Fn() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		+ Send
		+ Sync
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		A: Clone + Send + Sync + 'a;
}

/// Creates a deferred value from a thread-safe thunk.
///
/// Free function version that dispatches to [the type class' associated function][`SendDeferrable::send_defer`].
///
/// ### Type Signature
///
#[hm_signature(SendDeferrable)]
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the deferred type.
/// * `A`: The type of the value.
/// * `F`: The type of the thunk.
///
/// ### Parameters
///
/// * `thunk`: The function that produces the value.
///
/// ### Returns
///
/// A deferred value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let memo: ArcLazy<i32> = send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(|| 42));
/// assert_eq!(*memo.evaluate(), 42);
/// ```
pub fn send_defer<'a, Brand, A, Func>(
	thunk: Func
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
where
	Brand: SendDeferrable,
	A: Clone + Send + Sync + 'a,
	Func: 'a + Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) + Send + Sync,
{
	Brand::send_defer(thunk)
}
