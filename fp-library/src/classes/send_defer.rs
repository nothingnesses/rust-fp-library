//! A trait for deferred lazy evaluation with thread-safe thunks.
//!
//! This module defines the [`SendDefer`] trait, which extends [`Kind`] to support creating deferred values
//! where the thunk is `Send + Sync`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::send_defer::*, functions::*, types::lazy::*};
//!
//! let lazy = send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(ArcLazyConfig::new_thunk(|_| 42)));
//! assert_eq!(Lazy::force(&lazy).unwrap(), &42);
//! ```

use crate::{Apply, kinds::*};

/// A trait for deferred lazy evaluation with thread-safe thunks.
///
/// This is similar to `Defer`, but the thunk must be `Send + Sync`.
pub trait SendDefer: Kind_cdc7cd43dac7585f {
	/// Creates a deferred value from a thread-safe thunk.
	///
	/// ### Type Signature
	///
	/// `forall f a. (SendDefer f, Send a, Sync a) => (() -> a) -> f a`
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
/// Free function version that dispatches to [the type class' associated function][`SendDefer::send_defer`].
///
/// ### Type Signature
///
/// `forall f a. (SendDefer f, Send a, Sync a) => (() -> a) -> f a`
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
pub fn send_defer<'a, Brand, A, F>(
	thunk: F
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
where
	Brand: SendDefer,
	A: Clone + Send + Sync + 'a,
	F: 'a + Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) + Send + Sync,
{
	Brand::send_defer(thunk)
}
