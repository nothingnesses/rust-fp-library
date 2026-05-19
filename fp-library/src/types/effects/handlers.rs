//! Handler-list runtime values for the
//! [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
//! macro and manual handler-list builders.
//!
//! A natural transformation `VariantF<R> ~> M` is assembled at the user
//! level via the macro
//! `handlers!{ EBrand1: |op| ..., EBrand2: |op| ... }` (the primary
//! surface), via the natural-order manual builder
//! `handlers_ordered().on::<EBrand1, _>(|op| ...).on::<EBrand2, _>(|op| ...).finish()`,
//! or via the low-level prepend seed
//! `nt().prepend::<EBrand2, _>(|op| ...).prepend::<EBrand1, _>(|op| ...)`.
//! Each path evaluates to the same runtime shape: a type-level
//! cons-list whose structure mirrors the row's
//! [`CoproductBrand`](crate::brands::CoproductBrand) /
//! [`CNilBrand`](crate::brands::CNilBrand) chain cell-for-cell.
//!
//! This module ships only the runtime carrier; the interpreter family
//! (`handle` / `run` / `runAccum` and their `MonadRec` siblings)
//! is the consumer that recurses through the row and the handler list
//! in lock-step, dispatching each [`Coproduct::Inl`](crate::types::effects::coproduct::Coproduct::Inl)
//! variant to the matching [`HandlersCons::head`] and recursing into
//! [`HandlersCons::tail`] on [`Coproduct::Inr`](crate::types::effects::coproduct::Coproduct::Inr).
//! The closure shape carried inside each [`Handler`] is left fully
//! generic here; the interpreter pins it via a trait bound.
//!
//! Scoped handlers use parallel carrier types:
//! [`ScopedHandler<S, F>`], [`ScopedHandlersCons<H, T>`], and
//! [`ScopedHandlersNil`]. A scoped head stores a handler value
//! rather than requiring a plain closure, because scoped handler
//! methods are generic over the concrete first-order handler-list type.
//! The method-generic contract lives in
//! [`DispatchScopedHandler`](crate::types::effects::interpreter::DispatchScopedHandler);
//! this module only carries the values.
//!
//! ## Why a dedicated cons-list rather than reusing `frunk_core`'s `HList`
//!
//! `frunk_core::hlist::{HNil, HCons}` are already re-exported under
//! [`crate::types::effects::coproduct`] for the row-encoding indexing
//! machinery (`Here` / `There`, `CoprodInjector`, etc.). Reusing the
//! same types here would technically work but conflate two distinct
//! roles: the coproduct module's `HCons` / `HNil` mark type-level
//! positions for row-membership proofs, while this module's
//! [`HandlersCons`] / [`HandlersNil`] carry runtime handler closures
//! aligned with the row's value-level shape. Distinct types keep the
//! intent visible at call sites and let low-level construction live on
//! the handler-list types directly without an extension-trait dance.
//!
//! ## Builder ordering
//!
//! [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
//! and [`scoped_handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.scoped_handlers.html)
//! are the primary user-facing builder surfaces. Manual code that
//! wants the same left-to-right order should use [`handlers_ordered()`]
//! or [`scoped_handlers_ordered()`]:
//!
//! ```
//! use fp_library::types::effects::handlers::handlers_ordered;
//!
//! struct A;
//! struct B;
//!
//! let handlers = handlers_ordered()
//! 	.on::<A, _>(|value: i32| value + 1)
//! 	.on::<B, _>(|value: i32| value * 2)
//! 	.finish();
//!
//! assert_eq!((handlers.head.run)(4), 5);
//! assert_eq!((handlers.tail.head.run)(4), 8);
//! ```
//!
//! [`nt()`] and [`scoped_nt()`] remain the representation-level seeds.
//! They expose explicit `.prepend(...)` methods. Chained prepend calls
//! produce a list whose head is the most-recently-prepended handler.
//! This is useful for tests and generated code that need to spell the
//! cons-list shape directly, but normal examples should prefer
//! `handlers!`, `scoped_handlers!`, or the natural-order builders.
//!
//! ## Reading missing-handler errors
//!
//! Handler coverage is checked by Rust trait selection at the
//! `handle` call site. The `handlers!` and `scoped_handlers!`
//! macros only see the entries written inside the macro invocation;
//! they do not see the program's first-order or scoped row type, so
//! they cannot validate row coverage by themselves.
//!
//! A missing first-order handler usually appears as a
//! `DispatchHandlers<..., Coproduct<...>>` bound that is not
//! implemented for the provided handler-list tail, often
//! [`HandlersNil`]. Read the remaining `Coproduct` head in the error:
//! for rows built by `effects!`, it has the shape
//! `Coyoneda<'_, MissingBrand, NextProgram>` (or the Rc/Arc Coyoneda
//! variants for shared wrappers). Add a `MissingBrand: ...` entry to
//! `handlers!` or to the equivalent
//! `handlers_ordered().on::<MissingBrand, _>(...)` builder chain.
//!
//! A missing scoped handler similarly appears as a
//! `DispatchScopedHandlers` or wrapper-specific raw scoped-dispatch
//! bound that is not implemented for the provided scoped-handler-list
//! tail, often [`ScopedHandlersNil`]. Read the remaining scoped-row
//! `Coproduct` head in the error and add that scoped brand to
//! `scoped_handlers!`, for example
//! `BoxCatchBrand<BoxBrand, Error>: catch_handler::<_, RowMinusExcept, _>()`.

#[fp_macros::document_module]
mod inner {
	use core::marker::PhantomData;

	/// Newtype tagging a handler closure with the brand `E` it handles.
	///
	/// `Handler<E, F>` pins the brand identity at the type level so the
	/// interpreter can match each handler against the row's head brand
	/// without the closure's type signature having to encode the brand
	/// explicitly. The closure value `F` stays opaque here; the
	/// interpreter side adds the concrete `F: FnMut(...) -> ...` bound.
	#[derive(Clone, Copy)]
	pub struct Handler<E, F> {
		/// The handler closure for effect brand `E`.
		pub run: F,
		#[doc(hidden)]
		pub _brand: PhantomData<fn() -> E>,
	}

	#[fp_macros::document_type_parameters(
		"The effect brand identifier.",
		"The closure type stored in this handler cell."
	)]
	impl<E, F> Handler<E, F> {
		/// Wraps a closure as a [`Handler`] for effect brand `E`.
		/// Zero-cost.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The handler closure to wrap.")]
		///
		#[fp_macros::document_returns("A [`Handler`] tagged with brand `E` carrying the closure.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::Handler;
		///
		/// struct StateBrand;
		///
		/// let handler = Handler::<StateBrand, _>::new(|x: i32| x + 1);
		/// assert_eq!((handler.run)(2), 3);
		/// ```
		#[inline]
		pub const fn new(run: F) -> Self {
			Handler {
				run,
				_brand: PhantomData,
			}
		}
	}

	/// Newtype tagging a scoped-handler handler value with the scoped
	/// effect brand `S`.
	///
	/// `ScopedHandler<S, F>` parallels [`Handler<E, F>`], but the stored
	/// value is expected to implement
	/// [`DispatchScopedHandler`](crate::types::effects::interpreter::DispatchScopedHandler)
	/// rather than `Fn`. Scoped handlers receive both the scoped layer
	/// and the first-order handler list, and the latter is method-generic.
	#[derive(Clone, Copy)]
	pub struct ScopedHandler<S, F> {
		/// The handler value for scoped-effect brand `S`.
		pub run: F,
		#[doc(hidden)]
		pub _brand: PhantomData<fn() -> S>,
	}

	#[fp_macros::document_type_parameters(
		"The scoped-effect brand identifier.",
		"The handler value stored in this handler cell."
	)]
	impl<S, F> ScopedHandler<S, F> {
		/// Wraps a handler value as a [`ScopedHandler`] for scoped
		/// effect brand `S`.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The scoped handler value to wrap.")]
		///
		#[fp_macros::document_returns(
			"A [`ScopedHandler`] tagged with brand `S` carrying the handler value."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::ScopedHandler;
		///
		/// struct SpanBrand;
		///
		/// let handler = ScopedHandler::<SpanBrand, _>::new(7);
		/// assert_eq!(handler.run, 7);
		/// ```
		#[inline]
		pub const fn new(run: F) -> Self {
			ScopedHandler {
				run,
				_brand: PhantomData,
			}
		}
	}

	/// Empty handler list, mirrors [`CNilBrand`](crate::brands::CNilBrand)
	/// at the row-shape level.
	///
	/// Returned by [`nt()`] as the seed of a builder chain. The
	/// [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
	/// macro emits this as the terminator of its cons chain.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct HandlersNil;

	/// Cons cell of the handler list, mirrors [`CoproductBrand`](crate::brands::CoproductBrand)
	/// at the row-shape level.
	///
	/// `HandlersCons<H, T>` carries a head handler `H` (typically a
	/// [`Handler<EBrand, F>`](Handler)) and a tail `T` that is itself
	/// either another `HandlersCons` or [`HandlersNil`]. The shape mirrors
	/// the row brand `CoproductBrand<EBrand, Tail>` cell-for-cell so the
	/// interpreter can recurse through both in lock-step.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct HandlersCons<H, T> {
		/// The handler at this row position.
		pub head: H,
		/// The remaining handlers, aligned with the tail of the row.
		pub tail: T,
	}

	/// Empty scoped-handler list, mirrors [`CNilBrand`](crate::brands::CNilBrand)
	/// at the scoped-row-shape level.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct ScopedHandlersNil;

	/// Cons cell of the scoped-handler list, mirrors
	/// [`CoproductBrand`](crate::brands::CoproductBrand) at the
	/// scoped-row-shape level.
	///
	/// `ScopedHandlersCons<H, T>` carries a head scoped handler `H`
	/// (typically a [`ScopedHandler<SBrand, F>`](ScopedHandler)) and a
	/// tail `T` that is either another `ScopedHandlersCons` or
	/// [`ScopedHandlersNil`].
	#[derive(Clone, Copy, Debug, Default)]
	pub struct ScopedHandlersCons<H, T> {
		/// The scoped handler at this row position.
		pub head: H,
		/// The remaining scoped handlers, aligned with the tail of the scoped row.
		pub tail: T,
	}

	/// Natural-order manual builder for first-order handler lists.
	///
	/// `HandlersOrdered<L>` stores the handler-list shape built so far.
	/// Its [`on`](HandlersOrdered::on) method appends the new handler to
	/// the tail, so chained calls read in the same order as the final
	/// cons-list shape. Call [`finish`](HandlersOrdered::finish) to
	/// recover the underlying [`HandlersNil`] / [`HandlersCons`] list.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct HandlersOrdered<L> {
		list: L,
	}

	/// Natural-order manual builder for scoped-handler lists.
	///
	/// `ScopedHandlersOrdered<L>` is the scoped counterpart of
	/// [`HandlersOrdered`]. Chained [`on`](ScopedHandlersOrdered::on)
	/// calls append scoped handlers in the order written, and
	/// [`finish`](ScopedHandlersOrdered::finish) returns the underlying
	/// scoped-handler cons-list.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct ScopedHandlersOrdered<L> {
		list: L,
	}

	#[doc(hidden)]
	#[fp_macros::document_type_parameters(
		"The effect brand identifier for the handler being appended.",
		"The handler closure type for the handler being appended."
	)]
	#[fp_macros::document_parameters("The handler list receiving the appended handler.")]
	pub trait AppendHandler<E, F> {
		type Output;

		/// Appends an already tagged handler cell to the tail of this
		/// first-order handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The tagged handler cell to append.")]
		///
		#[fp_macros::document_returns(
			"The handler-list shape produced after appending the handler at the tail."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct StateBrand;
		///
		/// let list = HandlersNil.append_handler(Handler::<StateBrand, _>::new(|x: i32| x + 1));
		/// assert_eq!((list.head.run)(1), 2);
		/// ```
		fn append_handler(
			self,
			handler: Handler<E, F>,
		) -> Self::Output;
	}

	#[fp_macros::document_type_parameters(
		"The effect brand identifier for the handler being appended.",
		"The handler closure type for the handler being appended."
	)]
	#[fp_macros::document_parameters("The empty handler list receiving the appended handler.")]
	impl<E, F> AppendHandler<E, F> for HandlersNil {
		type Output = HandlersCons<Handler<E, F>, HandlersNil>;

		/// Appends a tagged handler cell to an empty first-order handler
		/// list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The tagged handler cell to append.")]
		///
		#[fp_macros::document_returns("A single-cell first-order handler list.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct StateBrand;
		///
		/// let list = HandlersNil.append_handler(Handler::<StateBrand, _>::new(|x: i32| x + 1));
		/// assert_eq!((list.head.run)(1), 2);
		/// ```
		#[inline]
		fn append_handler(
			self,
			handler: Handler<E, F>,
		) -> Self::Output {
			HandlersCons {
				head: handler,
				tail: self,
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The existing head handler cell type.",
		"The existing tail handler-list type.",
		"The effect brand identifier for the handler being appended.",
		"The handler closure type for the handler being appended."
	)]
	#[fp_macros::document_parameters("The non-empty handler list receiving the appended handler.")]
	impl<H, T, E, F> AppendHandler<E, F> for HandlersCons<H, T>
	where
		T: AppendHandler<E, F>,
	{
		type Output = HandlersCons<H, <T as AppendHandler<E, F>>::Output>;

		/// Appends a tagged handler cell after this non-empty
		/// first-order handler list's tail.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The tagged handler cell to append.")]
		///
		#[fp_macros::document_returns(
			"A first-order handler list with the existing head preserved and the new handler appended to the tail."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct StateBrand;
		/// struct ReaderBrand;
		///
		/// let list = nt()
		/// 	.prepend::<StateBrand, _>(|x: i32| x)
		/// 	.append_handler(Handler::<ReaderBrand, _>::new(|x: i32| x + 1));
		/// assert_eq!((list.head.run)(1), 1);
		/// assert_eq!((list.tail.head.run)(1), 2);
		/// ```
		#[inline]
		fn append_handler(
			self,
			handler: Handler<E, F>,
		) -> Self::Output {
			HandlersCons {
				head: self.head,
				tail: self.tail.append_handler(handler),
			}
		}
	}

	#[doc(hidden)]
	#[fp_macros::document_type_parameters(
		"The scoped-effect brand identifier for the handler being appended.",
		"The scoped handler value type for the handler being appended."
	)]
	#[fp_macros::document_parameters("The scoped-handler list receiving the appended handler.")]
	pub trait AppendScopedHandler<S, F> {
		type Output;

		/// Appends an already tagged scoped-handler cell to the tail of
		/// this scoped-handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The tagged scoped-handler cell to append.")]
		///
		#[fp_macros::document_returns(
			"The scoped-handler-list shape produced after appending the handler at the tail."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct LocalBrand;
		///
		/// let list = ScopedHandlersNil.append_scoped_handler(ScopedHandler::<LocalBrand, _>::new(1));
		/// assert_eq!(list.head.run, 1);
		/// ```
		fn append_scoped_handler(
			self,
			handler: ScopedHandler<S, F>,
		) -> Self::Output;
	}

	#[fp_macros::document_type_parameters(
		"The scoped-effect brand identifier for the handler being appended.",
		"The scoped handler value type for the handler being appended."
	)]
	#[fp_macros::document_parameters(
		"The empty scoped-handler list receiving the appended handler."
	)]
	impl<S, F> AppendScopedHandler<S, F> for ScopedHandlersNil {
		type Output = ScopedHandlersCons<ScopedHandler<S, F>, ScopedHandlersNil>;

		/// Appends a tagged scoped-handler cell to an empty
		/// scoped-handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The tagged scoped-handler cell to append.")]
		///
		#[fp_macros::document_returns("A single-cell scoped-handler list.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct LocalBrand;
		///
		/// let list = ScopedHandlersNil.append_scoped_handler(ScopedHandler::<LocalBrand, _>::new(1));
		/// assert_eq!(list.head.run, 1);
		/// ```
		#[inline]
		fn append_scoped_handler(
			self,
			handler: ScopedHandler<S, F>,
		) -> Self::Output {
			ScopedHandlersCons {
				head: handler,
				tail: self,
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The existing head scoped-handler cell type.",
		"The existing tail scoped-handler-list type.",
		"The scoped-effect brand identifier for the handler being appended.",
		"The scoped handler value type for the handler being appended."
	)]
	#[fp_macros::document_parameters(
		"The non-empty scoped-handler list receiving the appended handler."
	)]
	impl<H, T, S, F> AppendScopedHandler<S, F> for ScopedHandlersCons<H, T>
	where
		T: AppendScopedHandler<S, F>,
	{
		type Output = ScopedHandlersCons<H, <T as AppendScopedHandler<S, F>>::Output>;

		/// Appends a tagged scoped-handler cell after this non-empty
		/// scoped-handler list's tail.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The tagged scoped-handler cell to append.")]
		///
		#[fp_macros::document_returns(
			"A scoped-handler list with the existing head preserved and the new handler appended to the tail."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct LocalBrand;
		/// struct BracketBrand;
		///
		/// let list = scoped_nt()
		/// 	.prepend::<LocalBrand, _>(1)
		/// 	.append_scoped_handler(ScopedHandler::<BracketBrand, _>::new(2));
		/// assert_eq!(list.head.run, 1);
		/// assert_eq!(list.tail.head.run, 2);
		/// ```
		#[inline]
		fn append_scoped_handler(
			self,
			handler: ScopedHandler<S, F>,
		) -> Self::Output {
			ScopedHandlersCons {
				head: self.head,
				tail: self.tail.append_scoped_handler(handler),
			}
		}
	}

	#[fp_macros::document_parameters("The empty handler list.")]
	impl HandlersNil {
		/// Prepends a new handler for effect brand `E` at the head of the
		/// low-level list, transitioning [`HandlersNil`] to a single-cell
		/// [`HandlersCons<Handler<E, F>, HandlersNil>`](HandlersCons).
		///
		/// `E` is the brand identity (usually turbofished;
		/// `nt().prepend::<StateBrand, _>(...)`); `F` is inferred from the
		/// closure literal.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The effect brand identifier (typically turbofished).",
			"The handler closure type (inferred from the closure literal)."
		)]
		///
		#[fp_macros::document_parameters("The handler closure to prepend.")]
		///
		#[fp_macros::document_returns("A single-cell handler list with `handler` at the head.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct StateBrand;
		///
		/// let h = nt().prepend::<StateBrand, _>(|x: i32| x + 1);
		/// assert_eq!((h.head.run)(2), 3);
		/// ```
		#[inline]
		pub fn prepend<E, F>(
			self,
			handler: F,
		) -> HandlersCons<Handler<E, F>, Self> {
			HandlersCons {
				head: Handler::new(handler),
				tail: self,
			}
		}
	}

	#[fp_macros::document_parameters("The empty scoped-handler list.")]
	impl ScopedHandlersNil {
		/// Prepends a new scoped handler for scoped-effect brand `S` at
		/// the head of the low-level scoped-handler list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The scoped-effect brand identifier (typically turbofished).",
			"The scoped handler value type."
		)]
		///
		#[fp_macros::document_parameters("The scoped handler value to prepend.")]
		///
		#[fp_macros::document_returns(
			"A single-cell scoped-handler list with `handler` at the head."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct SpanBrand;
		///
		/// let h = fp_library::types::effects::scoped_nt().prepend::<SpanBrand, _>(7);
		/// assert_eq!(h.head.run, 7);
		/// ```
		#[inline]
		pub fn prepend<S, F>(
			self,
			handler: F,
		) -> ScopedHandlersCons<ScopedHandler<S, F>, Self> {
			ScopedHandlersCons {
				head: ScopedHandler::new(handler),
				tail: self,
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The head handler type at this position.",
		"The tail handler list (another [`HandlersCons`] or [`HandlersNil`])."
	)]
	#[fp_macros::document_parameters("The handler list instance.")]
	impl<H, T> HandlersCons<H, T> {
		/// Prepends a new handler for effect brand `E` at the head of
		/// the list. The previous list becomes the tail.
		///
		/// Chained prepend calls produce a list whose head is the
		/// most-recently-prepended handler. Use
		/// [`handlers_ordered()`] for manual builder code that should
		/// read left-to-right in the resulting list order.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The effect brand identifier for the new handler.",
			"The handler closure type."
		)]
		///
		#[fp_macros::document_parameters("The handler closure to prepend at the head.")]
		///
		#[fp_macros::document_returns("A new [`HandlersCons`] with `handler` prepended.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct StateBrand;
		/// struct ReaderBrand;
		///
		/// let h = nt().prepend::<StateBrand, _>(|x: i32| x).prepend::<ReaderBrand, _>(|x: i32| x * 2);
		/// assert_eq!((h.head.run)(5), 10);
		/// assert_eq!((h.tail.head.run)(5), 5);
		/// ```
		#[inline]
		pub fn prepend<E, F>(
			self,
			handler: F,
		) -> HandlersCons<Handler<E, F>, Self> {
			HandlersCons {
				head: Handler::new(handler),
				tail: self,
			}
		}
	}

	#[fp_macros::document_type_parameters(
		"The head scoped-handler type at this position.",
		"The tail scoped-handler list (another [`ScopedHandlersCons`] or [`ScopedHandlersNil`])."
	)]
	#[fp_macros::document_parameters("The scoped-handler list instance.")]
	impl<H, T> ScopedHandlersCons<H, T> {
		/// Prepends a new scoped handler for scoped-effect brand `S` at
		/// the head of the list. The previous list becomes the tail.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The scoped-effect brand identifier for the new handler.",
			"The scoped handler value type."
		)]
		///
		#[fp_macros::document_parameters("The scoped handler value to prepend at the head.")]
		///
		#[fp_macros::document_returns("A new [`ScopedHandlersCons`] with `handler` prepended.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct SpanBrand;
		/// struct CatchBrand;
		///
		/// let h = fp_library::types::effects::scoped_nt()
		/// 	.prepend::<SpanBrand, _>(1)
		/// 	.prepend::<CatchBrand, _>(2);
		/// assert_eq!(h.head.run, 2);
		/// assert_eq!(h.tail.head.run, 1);
		/// ```
		#[inline]
		pub fn prepend<S, F>(
			self,
			handler: F,
		) -> ScopedHandlersCons<ScopedHandler<S, F>, Self> {
			ScopedHandlersCons {
				head: ScopedHandler::new(handler),
				tail: self,
			}
		}
	}

	#[fp_macros::document_type_parameters("The handler-list shape accumulated so far.")]
	#[fp_macros::document_parameters("The natural-order handler builder instance.")]
	impl<L> HandlersOrdered<L> {
		/// Appends a handler for effect brand `E` to the tail of this
		/// builder.
		///
		/// Chained calls preserve the order written: adding `A` and then
		/// `B` produces a handler list whose head is `A` and whose tail
		/// starts with `B`.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The effect brand identifier for the appended handler.",
			"The handler closure type."
		)]
		///
		#[fp_macros::document_parameters("The handler closure to append at the tail.")]
		///
		#[fp_macros::document_returns(
			"A natural-order builder whose accumulated list includes the new tail handler."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct StateBrand;
		/// struct ReaderBrand;
		///
		/// let h = handlers_ordered()
		/// 	.on::<StateBrand, _>(|x: i32| x + 1)
		/// 	.on::<ReaderBrand, _>(|x: i32| x * 2)
		/// 	.finish();
		///
		/// assert_eq!((h.head.run)(5), 6);
		/// assert_eq!((h.tail.head.run)(5), 10);
		/// ```
		#[inline]
		pub fn on<E, F>(
			self,
			handler: F,
		) -> HandlersOrdered<<L as AppendHandler<E, F>>::Output>
		where
			L: AppendHandler<E, F>, {
			HandlersOrdered {
				list: self.list.append_handler(Handler::new(handler)),
			}
		}

		/// Finishes this natural-order builder and returns the underlying
		/// handler cons-list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_returns("The handler-list shape accumulated by this builder.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct StateBrand;
		///
		/// let h = handlers_ordered().on::<StateBrand, _>(|x: i32| x + 1).finish();
		///
		/// assert_eq!((h.head.run)(2), 3);
		/// ```
		#[inline]
		pub fn finish(self) -> L {
			self.list
		}
	}

	#[fp_macros::document_type_parameters("The scoped-handler-list shape accumulated so far.")]
	#[fp_macros::document_parameters("The natural-order scoped-handler builder instance.")]
	impl<L> ScopedHandlersOrdered<L> {
		/// Appends a scoped handler for scoped-effect brand `S` to the
		/// tail of this builder.
		///
		/// Chained calls preserve the order written: adding `A` and then
		/// `B` produces a scoped-handler list whose head is `A` and whose
		/// tail starts with `B`.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_type_parameters(
			"The scoped-effect brand identifier for the appended handler.",
			"The scoped handler value type."
		)]
		///
		#[fp_macros::document_parameters("The scoped handler value to append at the tail.")]
		///
		#[fp_macros::document_returns(
			"A natural-order builder whose accumulated scoped-handler list includes the new tail handler."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct LocalBrand;
		/// struct BracketBrand;
		///
		/// let h = scoped_handlers_ordered().on::<LocalBrand, _>(1).on::<BracketBrand, _>(2).finish();
		///
		/// assert_eq!(h.head.run, 1);
		/// assert_eq!(h.tail.head.run, 2);
		/// ```
		#[inline]
		pub fn on<S, F>(
			self,
			handler: F,
		) -> ScopedHandlersOrdered<<L as AppendScopedHandler<S, F>>::Output>
		where
			L: AppendScopedHandler<S, F>, {
			ScopedHandlersOrdered {
				list: self.list.append_scoped_handler(ScopedHandler::new(handler)),
			}
		}

		/// Finishes this natural-order builder and returns the underlying
		/// scoped-handler cons-list.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_returns("The scoped-handler-list shape accumulated by this builder.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::handlers::*;
		///
		/// struct LocalBrand;
		///
		/// let h = scoped_handlers_ordered().on::<LocalBrand, _>(1).finish();
		/// assert_eq!(h.head.run, 1);
		/// ```
		#[inline]
		pub fn finish(self) -> L {
			self.list
		}
	}

	/// Entry point for the natural-order manual builder for first-order
	/// handler lists.
	///
	/// Chain `.on::<EBrand, _>(handler)` calls in the same order as the
	/// final row shape, then call `.finish()` to recover the underlying
	/// handler cons-list. The
	/// [`handlers!`](https://docs.rs/fp-macros/latest/fp_macros/macro.handlers.html)
	/// macro remains the primary surface for normal code.
	#[fp_macros::document_signature]
	///
	#[fp_macros::document_returns("A natural-order builder seeded with an empty handler list.")]
	///
	#[fp_macros::document_examples]
	///
	/// ```
	/// use fp_library::types::effects::handlers::*;
	///
	/// struct StateBrand;
	/// struct ReaderBrand;
	///
	/// let h = handlers_ordered()
	/// 	.on::<StateBrand, _>(|x: i32| x + 1)
	/// 	.on::<ReaderBrand, _>(|x: i32| x * 2)
	/// 	.finish();
	///
	/// assert_eq!((h.head.run)(3), 4);
	/// assert_eq!((h.tail.head.run)(3), 6);
	/// ```
	#[inline]
	#[must_use]
	pub const fn handlers_ordered() -> HandlersOrdered<HandlersNil> {
		HandlersOrdered {
			list: HandlersNil,
		}
	}

	/// Entry point for the natural-order manual builder for scoped
	/// handler lists.
	#[fp_macros::document_signature]
	///
	#[fp_macros::document_returns(
		"A natural-order builder seeded with an empty scoped-handler list."
	)]
	///
	#[fp_macros::document_examples]
	///
	/// ```
	/// use fp_library::types::effects::handlers::*;
	///
	/// struct LocalBrand;
	/// struct BracketBrand;
	///
	/// let h = scoped_handlers_ordered().on::<LocalBrand, _>(1).on::<BracketBrand, _>(2).finish();
	///
	/// assert_eq!(h.head.run, 1);
	/// assert_eq!(h.tail.head.run, 2);
	/// ```
	#[inline]
	#[must_use]
	pub const fn scoped_handlers_ordered() -> ScopedHandlersOrdered<ScopedHandlersNil> {
		ScopedHandlersOrdered {
			list: ScopedHandlersNil,
		}
	}

	/// Entry point for the low-level prepend builder for assembling a
	/// handler list.
	///
	/// Returns [`HandlersNil`]; chain `.prepend::<EBrand, _>(handler)`
	/// calls when spelling the cons-list representation directly. The
	/// most recently prepended handler becomes the head of the list. Use
	/// [`handlers_ordered()`] for manual code that should read in final
	/// list order.
	#[fp_macros::document_signature]
	///
	#[fp_macros::document_returns("The empty handler list, ready for `.prepend(...)` calls.")]
	///
	#[fp_macros::document_examples]
	///
	/// ```
	/// use fp_library::types::effects::handlers::*;
	///
	/// struct StateBrand;
	///
	/// let h = nt().prepend::<StateBrand, _>(|x: i32| x + 1);
	/// assert_eq!((h.head.run)(0), 1);
	/// ```
	#[inline]
	#[must_use]
	pub const fn nt() -> HandlersNil {
		HandlersNil
	}

	/// Entry point for the low-level prepend builder for assembling a
	/// scoped-handler list.
	#[fp_macros::document_signature]
	///
	#[fp_macros::document_returns(
		"The empty scoped-handler list, ready for `.prepend(...)` calls."
	)]
	///
	#[fp_macros::document_examples]
	///
	/// ```
	/// use fp_library::types::effects::handlers::*;
	///
	/// struct LocalBrand;
	///
	/// let h = fp_library::types::effects::scoped_nt().prepend::<LocalBrand, _>(1);
	/// assert_eq!(h.head.run, 1);
	/// ```
	#[inline]
	#[must_use]
	pub const fn scoped_nt() -> ScopedHandlersNil {
		ScopedHandlersNil
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use super::*;

	struct StateBrand;
	struct ReaderBrand;
	struct ExceptBrand;
	struct ScopedBrand;
	struct ScopedTailBrand;

	#[test]
	fn nt_returns_empty_list() {
		let h = nt();
		let _: HandlersNil = h;
	}

	#[test]
	fn prepend_at_nil_produces_single_cell() {
		let h = nt().prepend::<StateBrand, _>(|x: i32| x + 1);
		let _: HandlersCons<Handler<StateBrand, _>, HandlersNil> = h;
		let result = (h.head.run)(7);
		assert_eq!(result, 8);
	}

	#[test]
	fn prepend_at_cons_prepends_new_head() {
		let h = nt().prepend::<StateBrand, _>(|x: i32| x).prepend::<ReaderBrand, _>(|x: i32| x * 2);
		let _: HandlersCons<
			Handler<ReaderBrand, _>,
			HandlersCons<Handler<StateBrand, _>, HandlersNil>,
		> = h;
		assert_eq!((h.head.run)(5), 10);
		assert_eq!((h.tail.head.run)(5), 5);
	}

	#[test]
	fn three_handler_chain() {
		let h = nt()
			.prepend::<StateBrand, _>(|x: i32| x)
			.prepend::<ReaderBrand, _>(|x: i32| x + 1)
			.prepend::<ExceptBrand, _>(|x: i32| x + 2);
		assert_eq!((h.head.run)(0), 2);
		assert_eq!((h.tail.head.run)(0), 1);
		assert_eq!((h.tail.tail.head.run)(0), 0);
	}

	#[test]
	fn handlers_ordered_finish_preserves_written_order() {
		type OrderedTail<Reader, Except> = HandlersCons<
			Handler<ReaderBrand, Reader>,
			HandlersCons<Handler<ExceptBrand, Except>, HandlersNil>,
		>;
		type OrderedShape<State, Reader, Except> =
			HandlersCons<Handler<StateBrand, State>, OrderedTail<Reader, Except>>;

		let h = handlers_ordered()
			.on::<StateBrand, _>(|x: i32| x)
			.on::<ReaderBrand, _>(|x: i32| x + 1)
			.on::<ExceptBrand, _>(|x: i32| x + 2)
			.finish();
		let _: OrderedShape<_, _, _> = h;
		assert_eq!((h.head.run)(0), 0);
		assert_eq!((h.tail.head.run)(0), 1);
		assert_eq!((h.tail.tail.head.run)(0), 2);
	}

	#[test]
	fn handler_new_round_trips_closure() {
		let handler = Handler::<StateBrand, _>::new(|x: i32| x * 3);
		assert_eq!((handler.run)(4), 12);
	}

	#[test]
	fn handlers_cons_struct_literal_works() {
		// The macro emits struct-literal HandlersCons values; this
		// regression-checks the field-name shape stays compatible.
		type SingleStateCell = HandlersCons<Handler<StateBrand, fn(i32) -> i32>, HandlersNil>;
		let h: SingleStateCell = HandlersCons {
			head: Handler::new(|x: i32| x + 100),
			tail: HandlersNil,
		};
		assert_eq!((h.head.run)(1), 101);
	}

	#[test]
	fn scoped_nt_returns_empty_list() {
		let h = scoped_nt();
		let _: ScopedHandlersNil = h;
	}

	#[test]
	fn scoped_prepend_at_nil_produces_single_cell() {
		let h = scoped_nt().prepend::<ScopedBrand, _>(7);
		let _: ScopedHandlersCons<ScopedHandler<ScopedBrand, _>, ScopedHandlersNil> = h;
		assert_eq!(h.head.run, 7);
	}

	#[test]
	fn scoped_prepend_at_cons_prepends_new_head() {
		let h = scoped_nt().prepend::<ScopedBrand, _>(1).prepend::<ScopedTailBrand, _>(2);
		let _: ScopedHandlersCons<
			ScopedHandler<ScopedTailBrand, _>,
			ScopedHandlersCons<ScopedHandler<ScopedBrand, _>, ScopedHandlersNil>,
		> = h;
		assert_eq!(h.head.run, 2);
		assert_eq!(h.tail.head.run, 1);
	}

	#[test]
	fn scoped_handlers_ordered_finish_preserves_written_order() {
		let h =
			scoped_handlers_ordered().on::<ScopedBrand, _>(1).on::<ScopedTailBrand, _>(2).finish();
		let _: ScopedHandlersCons<
			ScopedHandler<ScopedBrand, _>,
			ScopedHandlersCons<ScopedHandler<ScopedTailBrand, _>, ScopedHandlersNil>,
		> = h;
		assert_eq!(h.head.run, 1);
		assert_eq!(h.tail.head.run, 2);
	}
}
