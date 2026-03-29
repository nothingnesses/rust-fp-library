//! Types that can be constructed lazily from a computation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let eval: Thunk<i32> = defer(|| Thunk::pure(42));
//! assert_eq!(eval.evaluate(), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;
	/// A type class for types that can be constructed lazily.
	///
	/// `Deferrable` is the dual of [`Extract`](crate::classes::Extract): where
	/// `Extract` forces/extracts the inner value, `Deferrable` constructs a value
	/// lazily from a thunk. For types implementing both (like
	/// [`Thunk`](crate::types::Thunk)), `extract(defer(|| x)) == x` forms a
	/// round-trip.
	///
	/// ### Laws
	///
	/// `Deferrable` instances must satisfy the following law:
	/// * Transparency: The value produced by `defer(|| x)` is identical to `x`. This law
	///   does not constrain *when* evaluation occurs; some implementations may evaluate eagerly.
	///
	/// ### Why there is no generic `fix`
	///
	/// In PureScript, `fix :: Lazy l => (l -> l) -> l` enables lazy self-reference,
	/// which is essential for tying the knot in recursive values. In Rust, lazy
	/// self-reference requires shared ownership (`Rc`/`Arc`) and interior mutability,
	/// which are properties specific to [`Lazy`](crate::types::Lazy) rather than
	/// all `Deferrable` types. For example, [`Thunk`](crate::types::Thunk) is consumed
	/// on evaluation, so self-referential construction is not possible.
	///
	/// The concrete functions [`rc_lazy_fix`](crate::types::lazy::rc_lazy_fix) and
	/// [`arc_lazy_fix`](crate::types::lazy::arc_lazy_fix) provide this capability for
	/// `Lazy` specifically.
	///
	/// # Warning
	///
	/// Some implementations may evaluate the thunk eagerly when the produced type requires
	/// `Send`. For example, `ArcLazy`'s `Deferrable` implementation evaluates the outer thunk
	/// immediately because `ArcLazy::new` requires a `Send` closure, but the `Deferrable`
	/// trait does not impose that bound. If you need guaranteed deferred evaluation with
	/// thread-safe types, prefer [`SendDeferrable`](crate::classes::SendDeferrable) instead.
	#[document_type_parameters("The lifetime of the computation.")]
	#[document_examples]
	///
	/// Transparency law for [`Thunk`](crate::types::Thunk):
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Transparency: defer(|| x) is equivalent to x when evaluated.
	/// let x = Thunk::pure(42);
	/// let deferred: Thunk<i32> = defer(|| Thunk::pure(42));
	/// assert_eq!(deferred.evaluate(), x.evaluate());
	/// ```
	pub trait Deferrable<'a> {
		/// Creates a value from a computation that produces the value.
		///
		/// This function takes a thunk and creates a deferred value that will be computed using the thunk.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the value.")]
		///
		#[document_returns("The deferred value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval: Thunk<i32> = defer(|| Thunk::pure(42));
		/// assert_eq!(eval.evaluate(), 42);
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized;
	}

	/// Creates a value from a computation that produces the value.
	///
	/// Free function version that dispatches to [the type class' associated function][`Deferrable::defer`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the deferred value."
	)]
	///
	#[document_parameters("A thunk that produces the value.")]
	///
	#[document_returns("The deferred value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let eval: Thunk<i32> = defer(|| Thunk::pure(42));
	/// assert_eq!(eval.evaluate(), 42);
	/// ```
	pub fn defer<'a, D: Deferrable<'a>>(f: impl FnOnce() -> D + 'a) -> D {
		D::defer(f)
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		crate::{
			functions::*,
			types::*,
		},
		quickcheck_macros::quickcheck,
	};

	/// Deferrable transparency law: evaluate(defer(|| x)) == x.
	#[quickcheck]
	fn prop_deferrable_transparency(x: i32) -> bool {
		let deferred: Thunk<i32> = defer(|| Thunk::pure(x));
		deferred.evaluate() == x
	}

	/// Deferrable nesting law: evaluate(defer(|| defer(|| x))) == evaluate(defer(|| x)).
	#[quickcheck]
	fn prop_deferrable_nesting(x: i32) -> bool {
		let nested: Thunk<i32> = defer(|| defer(|| Thunk::pure(x)));
		let single: Thunk<i32> = defer(|| Thunk::pure(x));
		nested.evaluate() == single.evaluate()
	}
}
