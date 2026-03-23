//! Memoized lazy evaluation for fallible computations.
//!
//! Computes a [`Result`] at most once and caches either the success value or error. All clones share the same cache. Available in both single-threaded [`RcTryLazy`] and thread-safe [`ArcTryLazy`] variants.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::TryLazyBrand,
			classes::{
				Deferrable,
				SendDeferrable,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcLazyConfig,
				Lazy,
				LazyConfig,
				RcLazyConfig,
				TryThunk,
				TryTrampoline,
			},
		},
		fp_macros::*,
		std::fmt,
	};

	/// A lazily-computed, memoized value that may fail.
	///
	/// The computation runs at most once. If it succeeds, the value is cached.
	/// If it fails, the error is cached. Subsequent accesses return the cached result.
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error.",
		"The memoization configuration."
	)]
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is [`TryLazyBrand<E, Config>`](crate::brands::TryLazyBrand),
	/// which is parameterized by both the error type and the `LazyConfig`, and is polymorphic over the success value type.
	pub struct TryLazy<'a, A, E, Config: LazyConfig = RcLazyConfig>(
		/// The internal lazy cell.
		pub(crate) Config::TryLazy<'a, A, E>,
	)
	where
		A: 'a,
		E: 'a;

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error.",
		"The memoization configuration."
	)]
	#[document_parameters("The instance to clone.")]
	impl<'a, A, E, Config: LazyConfig> Clone for TryLazy<'a, A, E, Config>
	where
		A: 'a,
		E: 'a,
	{
		#[document_signature]
		#[document_returns(
			"A new `TryLazy` instance that shares the same underlying memoized result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// let cloned = memo.clone();
		/// assert_eq!(cloned.evaluate(), Ok(&42));
		/// ```
		fn clone(&self) -> Self {
			Self(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error.",
		"The memoization configuration."
	)]
	#[document_parameters("The `TryLazy` instance.")]
	impl<'a, A, E, Config: LazyConfig> TryLazy<'a, A, E, Config>
	where
		A: 'a,
		E: 'a,
	{
		/// Gets the memoized result, computing on first access.
		#[document_signature]
		///
		#[document_returns("A result containing a reference to the value or error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		pub fn evaluate(&self) -> Result<&A, &E> {
			Config::try_evaluate(&self.0)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> TryLazy<'a, A, E, RcLazyConfig>
	where
		A: 'a,
		E: 'a,
	{
		/// Creates a new `TryLazy` that will run `f` on first access.
		#[document_signature]
		///
		#[document_parameters("The closure that produces the result.")]
		///
		#[document_returns("A new `TryLazy` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		pub fn new(f: impl FnOnce() -> Result<A, E> + 'a) -> Self {
			TryLazy(RcLazyConfig::try_lazy_new(Box::new(f)))
		}

		/// Creates a `TryLazy` containing an already-computed success value.
		#[document_signature]
		///
		#[document_parameters("The success value to wrap.")]
		///
		#[document_returns("A new `TryLazy` instance that evaluates to `Ok(&a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = RcTryLazy::<i32, ()>::ok(42);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		pub fn ok(a: A) -> Self {
			Self::new(move || Ok(a))
		}

		/// Creates a `TryLazy` containing an already-computed error value.
		#[document_signature]
		///
		#[document_parameters("The error value to wrap.")]
		///
		#[document_returns("A new `TryLazy` instance that evaluates to `Err(&e)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = RcTryLazy::<i32, String>::err("error".to_string());
		/// assert_eq!(memo.evaluate(), Err(&"error".to_string()));
		/// ```
		pub fn err(e: E) -> Self {
			Self::new(move || Err(e))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<TryThunk<'a, A, E>> for TryLazy<'a, A, E, RcLazyConfig> {
		#[document_signature]
		#[document_parameters("The fallible thunk to convert.")]
		#[document_returns(
			"A new `TryLazy` instance that will evaluate the thunk on first access."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = TryThunk::new(|| Ok::<i32, ()>(42));
		/// let memo = TryLazy::from(thunk);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(eval: TryThunk<'a, A, E>) -> Self {
			Self::new(move || eval.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<TryTrampoline<A, E>> for TryLazy<'a, A, E, RcLazyConfig>
	where
		A: Send,
		E: Send,
	{
		#[document_signature]
		#[document_parameters("The fallible trampoline to convert.")]
		#[document_returns(
			"A new `TryLazy` instance that will evaluate the trampoline on first access."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let task = TryTrampoline::<_, ()>::ok(42);
		/// let memo: TryLazy<_, (), _> = TryLazy::from(task);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(task: TryTrampoline<A, E>) -> Self {
			Self::new(move || task.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<Lazy<'a, A, ArcLazyConfig>> for TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: Clone + Send + Sync + 'a,
		E: Send + Sync + 'a,
	{
		#[document_signature]
		#[document_parameters("The thread-safe lazy value to convert.")]
		#[document_returns("A new `TryLazy` instance that wraps the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, ArcLazyConfig>::pure(42);
		/// let memo: TryLazy<_, (), _> = TryLazy::from(lazy);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(memo: Lazy<'a, A, ArcLazyConfig>) -> Self {
			Self::new(move || Ok(memo.evaluate().clone()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<Lazy<'a, A, RcLazyConfig>> for TryLazy<'a, A, E, RcLazyConfig>
	where
		A: Clone + 'a,
		E: 'a,
	{
		#[document_signature]
		#[document_parameters("The lazy value to convert.")]
		#[document_returns("A new `TryLazy` instance that wraps the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// let memo: TryLazy<_, (), _> = TryLazy::from(lazy);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(memo: Lazy<'a, A, RcLazyConfig>) -> Self {
			Self::new(move || Ok(memo.evaluate().clone()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A: 'a, E: 'a> From<Result<A, E>> for TryLazy<'a, A, E, RcLazyConfig> {
		#[document_signature]
		#[document_parameters("The result to convert.")]
		#[document_returns("A new `TryLazy` instance that produces the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let ok_memo: RcTryLazy<i32, String> = RcTryLazy::from(Ok(42));
		/// assert_eq!(ok_memo.evaluate(), Ok(&42));
		///
		/// let err_memo: RcTryLazy<i32, String> = RcTryLazy::from(Err("error".to_string()));
		/// assert_eq!(err_memo.evaluate(), Err(&"error".to_string()));
		/// ```
		fn from(result: Result<A, E>) -> Self {
			Self::new(move || result)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<Result<A, E>> for TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: 'a,
		E: 'a,
		Result<A, E>: Send,
	{
		#[document_signature]
		#[document_parameters("The result to convert.")]
		#[document_returns("A new `TryLazy` instance that produces the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let ok_memo: ArcTryLazy<i32, String> = ArcTryLazy::from(Ok(42));
		/// assert_eq!(ok_memo.evaluate(), Ok(&42));
		///
		/// let err_memo: ArcTryLazy<i32, String> = ArcTryLazy::from(Err("error".to_string()));
		/// assert_eq!(err_memo.evaluate(), Err(&"error".to_string()));
		/// ```
		fn from(result: Result<A, E>) -> Self {
			Self::new(move || result)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A> TryLazy<'a, A, String, RcLazyConfig>
	where
		A: 'a,
	{
		/// Creates a `TryLazy` that catches unwinds (panics).
		#[document_signature]
		///
		#[document_parameters("The closure that might panic.")]
		///
		#[document_returns("A new `TryLazy` instance where panics are converted to `Err(String)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = TryLazy::<_, String, RcLazyConfig>::catch_unwind(|| {
		/// 	if true {
		/// 		panic!("oops")
		/// 	}
		/// 	42
		/// });
		/// assert_eq!(memo.evaluate(), Err(&"oops".to_string()));
		/// ```
		pub fn catch_unwind(f: impl FnOnce() -> A + std::panic::UnwindSafe + 'a) -> Self {
			Self::new(move || {
				std::panic::catch_unwind(f).map_err(|e| {
					if let Some(s) = e.downcast_ref::<&str>() {
						s.to_string()
					} else if let Some(s) = e.downcast_ref::<String>() {
						s.clone()
					} else {
						"Unknown panic".to_string()
					}
				})
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: 'a,
		E: 'a,
	{
		/// Creates a new `TryLazy` that will run `f` on first access.
		#[document_signature]
		///
		#[document_parameters("The closure that produces the result.")]
		///
		#[document_returns("A new `TryLazy` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = TryLazy::<_, _, ArcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		pub fn new(f: impl FnOnce() -> Result<A, E> + Send + 'a) -> Self {
			TryLazy(ArcLazyConfig::try_lazy_new(Box::new(f)))
		}

		/// Creates a thread-safe `TryLazy` containing an already-computed success value.
		#[document_signature]
		///
		#[document_parameters("The success value to wrap.")]
		///
		#[document_returns("A new `ArcTryLazy` instance that evaluates to `Ok(&a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = ArcTryLazy::<i32, ()>::ok(42);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		pub fn ok(a: A) -> Self
		where
			A: Send,
			E: Send, {
			Self::new(move || Ok(a))
		}

		/// Creates a thread-safe `TryLazy` containing an already-computed error value.
		#[document_signature]
		///
		#[document_parameters("The error value to wrap.")]
		///
		#[document_returns("A new `ArcTryLazy` instance that evaluates to `Err(&e)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = ArcTryLazy::<i32, String>::err("error".to_string());
		/// assert_eq!(memo.evaluate(), Err(&"error".to_string()));
		/// ```
		pub fn err(e: E) -> Self
		where
			A: Send,
			E: Send, {
			Self::new(move || Err(e))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> Deferrable<'a> for TryLazy<'a, A, E, RcLazyConfig>
	where
		A: Clone + 'a,
		E: Clone + 'a,
	{
		/// Defers a computation that produces a `TryLazy` value.
		///
		/// This flattens the nested structure: instead of `TryLazy<TryLazy<A, E>, E>`, we get `TryLazy<A, E>`.
		/// The inner `TryLazy` is computed only when the outer `TryLazy` is evaluated.
		#[document_signature]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		#[document_returns("A new `TryLazy` value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = TryLazy::<_, (), RcLazyConfig>::defer(|| RcTryLazy::new(|| Ok(42)));
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			Self::new(move || f().evaluate().cloned().map_err(Clone::clone))
		}
	}

	impl_kind! {
		impl<E: 'static, Config: LazyConfig> for TryLazyBrand<E, Config> {
			#[document_default]
			type Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> SendDeferrable<'a> for TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: Clone + Send + Sync + 'a,
		E: Clone + Send + Sync + 'a,
	{
		/// Defers a computation that produces a thread-safe `TryLazy` value.
		///
		/// This flattens the nested structure: instead of `ArcTryLazy<ArcTryLazy<A, E>, E>`, we get `ArcTryLazy<A, E>`.
		/// The inner `TryLazy` is computed only when the outer `TryLazy` is evaluated.
		#[document_signature]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		#[document_returns("A new `ArcTryLazy` value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy: ArcTryLazy<i32, ()> = ArcTryLazy::send_defer(|| ArcTryLazy::new(|| Ok(42)));
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		fn send_defer(f: impl FnOnce() -> Self + Send + Sync + 'a) -> Self
		where
			Self: Sized, {
			Self::new(move || f().evaluate().cloned().map_err(Clone::clone))
		}
	}

	/// Single-threaded fallible memoization alias.
	pub type RcTryLazy<'a, A, E> = TryLazy<'a, A, E, RcLazyConfig>;

	/// Thread-safe fallible memoization alias.
	pub type ArcTryLazy<'a, A, E> = TryLazy<'a, A, E, ArcLazyConfig>;

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error.",
		"The memoization configuration."
	)]
	#[document_parameters("The try-lazy value to format.")]
	impl<'a, A, E, Config: LazyConfig> fmt::Debug for TryLazy<'a, A, E, Config>
	where
		A: 'a,
		E: 'a,
	{
		/// Formats the try-lazy value without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(format!("{:?}", memo), "TryLazy(..)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("TryLazy(..)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::{
			RcLazy,
			TryThunk,
			TryTrampoline,
		},
		std::{
			cell::RefCell,
			rc::Rc,
		},
	};

	/// Tests that `TryLazy` caches successful results.
	///
	/// Verifies that the initializer is called only once for success.
	#[test]
	fn test_try_memo_caching_ok() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryLazy<i32, ()> = RcTryLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			Ok(42)
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(memo.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(memo.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `TryLazy` caches error results.
	///
	/// Verifies that the initializer is called only once for error.
	#[test]
	fn test_try_memo_caching_err() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryLazy<i32, i32> = RcTryLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			Err(0)
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(memo.evaluate(), Err(&0));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(memo.evaluate(), Err(&0));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `TryLazy` shares the cache across clones.
	///
	/// Verifies that clones see the same result.
	#[test]
	fn test_try_memo_sharing() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryLazy<i32, ()> = RcTryLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			Ok(42)
		});
		let shared = memo.clone();

		assert_eq!(memo.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(shared.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests `catch_unwind`.
	///
	/// Verifies that panics are caught and converted to errors.
	#[test]
	fn test_catch_unwind() {
		let memo = RcTryLazy::catch_unwind(|| {
			if true {
				panic!("oops")
			}
			42
		});

		match memo.evaluate() {
			Err(e) => assert_eq!(e, "oops"),
			Ok(_) => panic!("Should have failed"),
		}
	}

	/// Tests creation from `TryThunk`.
	#[test]
	fn test_try_memo_from_try_eval() {
		let eval = TryThunk::new(|| Ok::<i32, ()>(42));
		let memo = RcTryLazy::from(eval);
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests creation from `TryTrampoline`.
	#[test]
	fn test_try_memo_from_try_task() {
		let task = TryTrampoline::<i32, ()>::ok(42);
		let memo = RcTryLazy::from(task);
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests conversion to TryLazy.
	#[test]
	fn test_try_memo_from_rc_memo() {
		let memo = RcLazy::new(|| 42);
		let try_memo: crate::types::RcTryLazy<i32, ()> = crate::types::RcTryLazy::from(memo);
		assert_eq!(try_memo.evaluate(), Ok(&42));
	}

	/// Tests conversion to ArcTryLazy.
	#[test]
	fn test_try_memo_from_arc_memo() {
		use crate::types::ArcLazy;
		let memo = ArcLazy::new(|| 42);
		let try_memo: crate::types::ArcTryLazy<i32, ()> = crate::types::ArcTryLazy::from(memo);
		assert_eq!(try_memo.evaluate(), Ok(&42));
	}

	/// Tests SendDefer implementation.
	#[test]
	fn test_send_defer() {
		use crate::classes::send_deferrable::send_defer;

		let memo: ArcTryLazy<i32, ()> = send_defer(|| ArcTryLazy::new(|| Ok(42)));
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests `RcTryLazy::ok` convenience constructor.
	#[test]
	fn test_rc_try_lazy_ok() {
		let memo = RcTryLazy::<i32, ()>::ok(42);
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests `RcTryLazy::err` convenience constructor.
	#[test]
	fn test_rc_try_lazy_err() {
		let memo = RcTryLazy::<i32, String>::err("error".to_string());
		assert_eq!(memo.evaluate(), Err(&"error".to_string()));
	}

	/// Tests `ArcTryLazy::ok` convenience constructor.
	#[test]
	fn test_arc_try_lazy_ok() {
		let memo = ArcTryLazy::<i32, ()>::ok(42);
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests `ArcTryLazy::err` convenience constructor.
	#[test]
	fn test_arc_try_lazy_err() {
		let memo = ArcTryLazy::<i32, String>::err("error".to_string());
		assert_eq!(memo.evaluate(), Err(&"error".to_string()));
	}

	/// Tests `From<Result>` for `RcTryLazy` with `Ok`.
	#[test]
	fn test_rc_try_lazy_from_result_ok() {
		let memo: RcTryLazy<i32, String> = RcTryLazy::from(Ok(42));
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests `From<Result>` for `RcTryLazy` with `Err`.
	#[test]
	fn test_rc_try_lazy_from_result_err() {
		let memo: RcTryLazy<i32, String> = RcTryLazy::from(Err("error".to_string()));
		assert_eq!(memo.evaluate(), Err(&"error".to_string()));
	}

	/// Tests `From<Result>` for `ArcTryLazy` with `Ok`.
	#[test]
	fn test_arc_try_lazy_from_result_ok() {
		let memo: ArcTryLazy<i32, String> = ArcTryLazy::from(Ok(42));
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests `From<Result>` for `ArcTryLazy` with `Err`.
	#[test]
	fn test_arc_try_lazy_from_result_err() {
		let memo: ArcTryLazy<i32, String> = ArcTryLazy::from(Err("error".to_string()));
		assert_eq!(memo.evaluate(), Err(&"error".to_string()));
	}
}
