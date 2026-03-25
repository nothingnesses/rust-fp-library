//! Thread-safe deferred, non-memoized computation.
//!
//! Like [`Thunk`](crate::types::Thunk) but with a `Send` bound on the inner closure,
//! enabling thread-safe deferred computation chains and truly lazy
//! [`memoize_arc`](SendThunk::memoize_arc) without eager evaluation.
//!
//! Standard HKT traits (`Functor`, `Semimonad`, etc.) cannot be implemented because
//! their signatures do not require `Send` on mapping functions. Use the inherent
//! methods ([`map`](SendThunk::map), [`bind`](SendThunk::bind)) instead.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::SendThunkBrand,
			classes::{
				Deferrable,
				Monoid,
				Semigroup,
				SendDeferrable,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcLazy,
				ArcLazyConfig,
				Lazy,
				Thunk,
			},
		},
		fp_macros::*,
		std::fmt,
	};

	/// A thread-safe deferred computation that produces a value of type `A`.
	///
	/// `SendThunk` is the `Send`-capable counterpart of [`Thunk`]. It wraps a
	/// `Box<dyn FnOnce() -> A + Send + 'a>`, so it can be transferred across thread
	/// boundaries. Like `Thunk`, it is NOT memoized and does not cache results.
	///
	/// The key advantage over `Thunk` is that [`memoize_arc`](SendThunk::memoize_arc)
	/// can wrap the closure lazily in an [`ArcLazy`] without forcing evaluation
	/// first, because the inner closure satisfies `Send`.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is
	/// [`SendThunkBrand`](crate::brands::SendThunkBrand), which is fully
	/// polymorphic over the result type.
	///
	/// ### HKT Trait Limitations
	///
	/// Standard HKT traits such as `Functor`, `Pointed`, `Semimonad`, and
	/// `Semiapplicative` cannot be implemented for `SendThunkBrand` because
	/// their signatures do not require `Send` on the mapping or binding
	/// functions. Since `SendThunk` stores a `Box<dyn FnOnce() -> A + Send>`,
	/// composing it with a non-`Send` closure would violate the `Send` invariant.
	///
	/// Use the inherent methods ([`map`](SendThunk::map),
	/// [`bind`](SendThunk::bind)) instead, which accept `Send` closures
	/// explicitly.
	///
	/// ### Stack Safety
	///
	/// Like `Thunk`, `SendThunk::bind` chains are **not** stack-safe. Each nested
	/// [`bind`](SendThunk::bind) adds a frame to the call stack.
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	///
	pub struct SendThunk<'a, A>(
		/// The thread-safe closure that performs the computation.
		Box<dyn FnOnce() -> A + Send + 'a>,
	);

	// SAFETY: SendThunk is Send because its inner closure is Send.
	// The Box<dyn FnOnce() -> A + Send + 'a> already guarantees Send on the closure.
	// Rust auto-derives Send for Box<dyn ... + Send>, so this is sound.

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	#[document_parameters("The send thunk instance.")]
	impl<'a, A: 'a> SendThunk<'a, A> {
		/// Creates a new `SendThunk` from a thread-safe closure.
		#[document_signature]
		///
		#[document_parameters("The thread-safe closure to wrap.")]
		///
		#[document_returns("A new `SendThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::new(|| 42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn new(f: impl FnOnce() -> A + Send + 'a) -> Self {
			SendThunk(Box::new(f))
		}

		/// Returns a pure value (already computed).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `SendThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self
		where
			A: Send + 'a, {
			SendThunk::new(move || a)
		}

		/// Defers a computation that returns a `SendThunk`.
		#[document_signature]
		///
		#[document_parameters("The thunk that returns a `SendThunk`.")]
		///
		#[document_returns("A new `SendThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::defer(|| SendThunk::pure(42));
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn defer(f: impl FnOnce() -> SendThunk<'a, A> + Send + 'a) -> Self {
			SendThunk::new(move || f().evaluate())
		}

		/// Monadic bind: chains computations.
		///
		/// Note: Each `bind` adds to the call stack. For deep recursion,
		/// consider converting to [`Trampoline`](crate::types::Trampoline).
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `SendThunk` instance representing the chained computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(21).bind(|x| SendThunk::pure(x * 2));
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn bind<B: 'a>(
			self,
			f: impl FnOnce(A) -> SendThunk<'a, B> + Send + 'a,
		) -> SendThunk<'a, B> {
			SendThunk::new(move || {
				let a = (self.0)();
				let thunk_b = f(a);
				(thunk_b.0)()
			})
		}

		/// Functor map: transforms the result.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the transformation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `SendThunk` instance with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(21).map(|x| x * 2);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn map<B: 'a>(
			self,
			f: impl FnOnce(A) -> B + Send + 'a,
		) -> SendThunk<'a, B> {
			SendThunk::new(move || f((self.0)()))
		}

		/// Forces evaluation and returns the result.
		#[document_signature]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn evaluate(self) -> A {
			(self.0)()
		}

		/// Converts this `SendThunk` into a memoized [`ArcLazy`] value.
		///
		/// Unlike [`Thunk::memoize_arc`](crate::types::Thunk::memoize_arc), this
		/// does **not** evaluate eagerly. The inner `Send` closure is passed
		/// directly into `ArcLazy::new`, so evaluation is deferred until the
		/// `ArcLazy` is first accessed.
		#[document_signature]
		///
		#[document_returns("A thread-safe `ArcLazy` that evaluates this thunk on first access.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::new(|| 42);
		/// let lazy = thunk.memoize_arc();
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		#[inline]
		pub fn memoize_arc(self) -> ArcLazy<'a, A> {
			Lazy::<'a, A, ArcLazyConfig>::new(move || self.evaluate())
		}
	}

	#[document_type_parameters("The lifetime of the computation.", "The type of the value.")]
	impl<'a, A: 'a> From<Thunk<'a, A>> for SendThunk<'a, A>
	where
		A: Send,
	{
		/// Converts a [`Thunk`] into a [`SendThunk`].
		///
		/// The `Thunk` closure is not `Send`, so the conversion eagerly
		/// evaluates it and wraps the owned result in a new `SendThunk`.
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A new `SendThunk` wrapping the evaluated result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = Thunk::pure(42);
		/// let send_thunk = SendThunk::from(thunk);
		/// assert_eq!(send_thunk.evaluate(), 42);
		/// ```
		fn from(thunk: Thunk<'a, A>) -> Self {
			let val = thunk.evaluate();
			SendThunk::new(move || val)
		}
	}

	impl_kind! {
		for SendThunkBrand {
			type Of<'a, A: 'a>: 'a = SendThunk<'a, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: 'a> Deferrable<'a> for SendThunk<'a, A> {
		/// Creates a `SendThunk` from a computation that produces it.
		///
		/// The thunk `f` is called eagerly because `Deferrable::defer` does not
		/// require `Send` on the closure.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the send thunk.")]
		///
		#[document_returns("The deferred send thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::Deferrable,
		/// 	types::*,
		/// };
		///
		/// let task: SendThunk<i32> = Deferrable::defer(|| SendThunk::pure(42));
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			f()
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Send + 'a> SendDeferrable<'a> for SendThunk<'a, A> {
		/// Creates a `SendThunk` from a thread-safe computation that produces it.
		#[document_signature]
		///
		#[document_parameters("A thread-safe thunk that produces the send thunk.")]
		///
		#[document_returns("The deferred send thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::SendDeferrable,
		/// 	types::*,
		/// };
		///
		/// let task: SendThunk<i32> = SendDeferrable::send_defer(|| SendThunk::pure(42));
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
		where
			Self: Sized, {
			SendThunk::new(move || f().evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Semigroup + Send + 'a> Semigroup for SendThunk<'a, A> {
		/// Combines two `SendThunk`s by combining their results.
		#[document_signature]
		///
		#[document_parameters("The first `SendThunk`.", "The second `SendThunk`.")]
		///
		#[document_returns("A new `SendThunk` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let t1 = SendThunk::pure("Hello".to_string());
		/// let t2 = SendThunk::pure(" World".to_string());
		/// let t3 = append::<_>(t1, t2);
		/// assert_eq!(t3.evaluate(), "Hello World");
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			SendThunk::new(move || Semigroup::append(a.evaluate(), b.evaluate()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Monoid + Send + 'a> Monoid for SendThunk<'a, A> {
		/// Returns the identity `SendThunk`.
		#[document_signature]
		///
		#[document_returns("A `SendThunk` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let t: SendThunk<String> = SendThunk::empty();
		/// assert_eq!(t.evaluate(), "");
		/// ```
		fn empty() -> Self {
			SendThunk::new(|| Monoid::empty())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	#[document_parameters("The send thunk to format.")]
	impl<'a, A> fmt::Debug for SendThunk<'a, A> {
		/// Formats the send thunk without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = SendThunk::pure(42);
		/// assert_eq!(format!("{:?}", thunk), "SendThunk(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("SendThunk(<unevaluated>)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::classes::{
			monoid::empty,
			semigroup::append,
		},
	};

	#[test]
	fn test_send_thunk_pure_and_evaluate() {
		let thunk = SendThunk::pure(42);
		assert_eq!(thunk.evaluate(), 42);
	}

	#[test]
	fn test_send_thunk_new() {
		let thunk = SendThunk::new(|| 1 + 2);
		assert_eq!(thunk.evaluate(), 3);
	}

	#[test]
	fn test_send_thunk_map() {
		let thunk = SendThunk::pure(10).map(|x| x * 3);
		assert_eq!(thunk.evaluate(), 30);
	}

	#[test]
	fn test_send_thunk_bind() {
		let thunk = SendThunk::pure(5).bind(|x| SendThunk::pure(x + 10));
		assert_eq!(thunk.evaluate(), 15);
	}

	#[test]
	fn test_send_thunk_defer() {
		let thunk = SendThunk::defer(|| SendThunk::pure(99));
		assert_eq!(thunk.evaluate(), 99);
	}

	#[test]
	fn test_send_thunk_memoize_arc() {
		let thunk = SendThunk::new(|| 42);
		let lazy = thunk.memoize_arc();
		assert_eq!(*lazy.evaluate(), 42);
		// Second access returns cached value.
		assert_eq!(*lazy.evaluate(), 42);
	}

	#[test]
	fn test_send_thunk_semigroup() {
		let t1 = SendThunk::pure("Hello".to_string());
		let t2 = SendThunk::pure(" World".to_string());
		let t3 = append(t1, t2);
		assert_eq!(t3.evaluate(), "Hello World");
	}

	#[test]
	fn test_send_thunk_monoid() {
		let t: SendThunk<String> = empty();
		assert_eq!(t.evaluate(), "");
	}

	#[test]
	fn test_send_thunk_from_thunk() {
		let thunk = crate::types::Thunk::pure(42);
		let send_thunk = SendThunk::from(thunk);
		assert_eq!(send_thunk.evaluate(), 42);
	}

	#[test]
	fn test_send_thunk_debug() {
		let thunk = SendThunk::pure(42);
		assert_eq!(format!("{:?}", thunk), "SendThunk(<unevaluated>)");
	}

	#[test]
	fn test_send_thunk_is_send() {
		fn assert_send<T: Send>() {}
		assert_send::<SendThunk<'static, i32>>();
	}

	#[test]
	fn test_send_thunk_deferrable() {
		use crate::classes::Deferrable;
		let task: SendThunk<i32> = Deferrable::defer(|| SendThunk::pure(42));
		assert_eq!(task.evaluate(), 42);
	}

	#[test]
	fn test_send_thunk_send_deferrable() {
		use crate::classes::SendDeferrable;
		let task: SendThunk<i32> = SendDeferrable::send_defer(|| SendThunk::pure(42));
		assert_eq!(task.evaluate(), 42);
	}
}
