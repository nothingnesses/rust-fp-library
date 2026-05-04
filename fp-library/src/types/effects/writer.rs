//! Log-emitting first-order effect type with the `Tell`
//! operation. The corresponding brand is
//! [`WriterBrand`](crate::brands::WriterBrand).
//!
//! `Writer<'a, W, A>` mirrors PureScript Run's
//! `Writer w a = Writer w a` shape directly. The single `Tell`
//! variant carries a log value `W` and the next program's value
//! `A` together; the `Functor` instance composes a user-supplied
//! `f: A -> B` with the stored `A` to produce a new
//! `Writer<'a, W, B>`, leaving the log value untouched.
//!
//! ## No pointer-brand parameter
//!
//! Unlike [`State`](crate::types::effects::state::State) or
//! [`Reader`](crate::types::effects::reader::Reader), `Writer`
//! has no `dyn Fn` continuation, so it does not parameterise over
//! a pointer brand `P` and does not need a parallel `SendWriter`
//! for the Arc family. The same `Writer<'a, W, A>` type serves
//! all six Run wrappers; per-wrapper smart constructors handle
//! the `Send + Sync` cascade on `W` alone.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::WriterBrand,
			classes::{
				Functor,
				SendFunctor,
			},
			impl_kind,
			kinds::*,
		},
		core::marker::PhantomData,
		fp_macros::*,
	};

	/// Log-emitting first-order effect type.
	///
	/// The single `Tell` variant carries a log value `W` and the
	/// next program's value `A` together. Mirrors PureScript Run's
	/// `Writer w a`.
	#[document_type_parameters(
		"The lifetime of the effect (carried for `Kind`-projection purposes only; nothing in the variants borrows from it).",
		"The log type.",
		"The result type produced by running the effect."
	)]
	pub enum Writer<'a, W, A: 'a> {
		/// Emit a log value of type `W` and continue with the next
		/// program's value `A`. The `Functor` instance composes a
		/// user-supplied `f: A -> B` with the stored `A`; the log
		/// value is carried unchanged.
		Tell(W, A, PhantomData<&'a ()>),
	}

	impl_kind! {
		impl<W: 'static> for WriterBrand<W> {
			type Of<'a, A: 'a>: 'a = Writer<'a, W, A>;
		}
	}

	#[document_type_parameters("The lifetime of the effect.", "The log type.", "The result type.")]
	#[document_parameters("The writer effect to clone.")]
	impl<'a, W, A> Clone for Writer<'a, W, A>
	where
		W: Clone,
		A: Clone + 'a,
	{
		/// Clones the writer effect by cloning the carried log
		/// value and the next program's value.
		#[document_signature]
		///
		#[document_returns("A new writer effect carrying clones of the log and the next value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::types::effects::writer::Writer,
		/// };
		///
		/// let original: Writer<'static, &'static str, i32> = Writer::Tell("logged", 42, PhantomData);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	Writer::Tell(log, next, _) => {
		/// 		assert_eq!(log, "logged");
		/// 		assert_eq!(next, 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Writer::Tell(log, next, _) => Writer::Tell(log.clone(), next.clone(), PhantomData),
			}
		}
	}

	#[document_type_parameters("The log type.")]
	impl<W> Functor for WriterBrand<W>
	where
		W: 'static,
	{
		/// Maps `f` over the next-program value of this writer
		/// effect.
		///
		/// The log value is carried unchanged; only the stored `A`
		/// is replaced with `f(A)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original next-program type.",
			"The new next-program type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to apply to the next-program value.",
			"The writer effect to map over."
		)]
		///
		#[document_returns(
			"A new writer effect with the same log value and `f` applied to the next-program value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::writer::Writer,
		/// 	},
		/// };
		///
		/// let tell: Writer<'static, &'static str, i32> = Writer::Tell("logged", 7, PhantomData);
		/// let mapped: Writer<'static, &'static str, i32> =
		/// 	<WriterBrand<&'static str> as Functor>::map(|x: i32| x * 2, tell);
		/// match mapped {
		/// 	Writer::Tell(log, next, _) => {
		/// 		assert_eq!(log, "logged");
		/// 		assert_eq!(next, 14);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Writer::Tell(log, next, _) => Writer::Tell(log, f(next), PhantomData),
			}
		}
	}

	#[document_type_parameters("The log type.")]
	impl<W> SendFunctor for WriterBrand<W>
	where
		W: Send + Sync + 'static,
	{
		/// Maps `f` over the next-program value of this writer
		/// effect, with `Send + Sync` bounds so the operation
		/// composes inside thread-safe contexts.
		///
		/// Body is structurally identical to [`Functor::map`]'s; the
		/// log value is carried unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original next-program type.",
			"The new next-program type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to apply to the next-program value. Must be `Send + Sync`.",
			"The writer effect to map over."
		)]
		///
		#[document_returns(
			"A new writer effect with the same log value and `f` applied to the next-program value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::writer::Writer,
		/// 	},
		/// };
		///
		/// let tell: Writer<'static, &'static str, i32> = Writer::Tell("logged", 7, PhantomData);
		/// let mapped: Writer<'static, &'static str, i32> =
		/// 	<WriterBrand<&'static str> as SendFunctor>::send_map(|x: i32| x * 2, tell);
		/// match mapped {
		/// 	Writer::Tell(log, next, _) => {
		/// 		assert_eq!(log, "logged");
		/// 		assert_eq!(next, 14);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Writer::Tell(log, next, _) => Writer::Tell(log, f(next), PhantomData),
			}
		}
	}
}

pub use inner::*;
