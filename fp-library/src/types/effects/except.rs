//! Error-throwing first-order effect type with the `Throw`
//! operation. The corresponding brand is
//! [`ExceptBrand`](crate::brands::ExceptBrand).
//!
//! `Except<'a, E, A>` mirrors PureScript Run's
//! `Except e a = Throw e` shape directly. The single `Throw`
//! variant carries an error value `E`; the result type `A` is
//! phantom because `Throw` never returns to the caller. The
//! `Functor` instance is structurally trivial: `map f (Throw e)`
//! produces `Throw e` (with the new result-type parameter).
//!
//! ## No pointer-brand parameter
//!
//! Unlike [`State`](crate::types::effects::state::State) or
//! [`Reader`](crate::types::effects::reader::Reader), `Except`
//! has no continuation, so it does not parameterise over a
//! pointer brand `P` and does not need a parallel
//! `SendExcept` for the Arc family. The same `Except<'a, E, A>`
//! type serves all six Run wrappers; per-wrapper smart
//! constructors handle the `Send + Sync` cascade on `E` alone.
//!
//! See the parent [`effects`](crate::types::effects) guide for the
//! consolidated wrapper and pointer-brand conventions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::ExceptBrand,
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

	/// Error-throwing first-order effect type.
	///
	/// The single `Throw` variant carries an error value `E`. The
	/// result type `A` is phantom (the program does not return to
	/// the caller after `Throw`); the `PhantomData<fn() -> A>`
	/// marker keeps the type covariant in `A` and satisfies the
	/// [`Kind`](crate::kinds) trait's
	/// `Of<'a, A: 'a>: 'a` contract without imposing variance
	/// constraints on `A` from references the type doesn't own.
	#[document_type_parameters(
		"The lifetime of the effect (carried for `Kind`-projection purposes only; nothing in the variants borrows from it).",
		"The error type.",
		"The phantom result type."
	)]
	pub enum Except<'a, E, A: 'a> {
		/// Raise an error of type `E`. The program does not return
		/// after this constructor; the phantom `A` parameter exists
		/// so the effect fits into the row's `Functor` interface.
		Throw(E, PhantomData<&'a A>),
	}

	impl_kind! {
		impl<E: 'static> for ExceptBrand<E> {
			type Of<'a, A: 'a>: 'a = Except<'a, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the effect.",
		"The error type.",
		"The phantom result type."
	)]
	#[document_parameters("The except effect to clone.")]
	impl<'a, E, A> Clone for Except<'a, E, A>
	where
		E: Clone,
		A: 'a,
	{
		/// Clones the except effect by cloning the carried error.
		#[document_signature]
		///
		#[document_returns("A new except effect carrying a clone of the error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::types::effects::except::Except,
		/// };
		///
		/// let original: Except<'static, &'static str, i32> = Except::Throw("oops", PhantomData);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	Except::Throw(e, _) => assert_eq!(e, "oops"),
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Except::Throw(e, _) => Except::Throw(e.clone(), PhantomData),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E> Functor for ExceptBrand<E>
	where
		E: 'static,
	{
		/// Maps `f` over the (phantom) result type of this except
		/// effect.
		///
		/// `Throw` carries no `A`-typed payload, so the function is
		/// discarded; the variant is rebuilt with the new phantom
		/// type parameter.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original phantom result type.",
			"The new phantom result type."
		)]
		///
		#[document_parameters(
			"The function to map over the (phantom) result type.",
			"The except effect to map over."
		)]
		///
		#[document_returns("A new except effect with the new phantom result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::except::Except,
		/// 	},
		/// };
		///
		/// let throw: Except<'static, &'static str, i32> = Except::Throw("err", PhantomData);
		/// let mapped: Except<'static, &'static str, String> =
		/// 	<ExceptBrand<&'static str> as Functor>::map(|x: i32| x.to_string(), throw);
		/// match mapped {
		/// 	Except::Throw(e, _) => assert_eq!(e, "err"),
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			_f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Except::Throw(e, _) => Except::Throw(e, PhantomData),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E> SendFunctor for ExceptBrand<E>
	where
		E: Send + Sync + 'static,
	{
		/// Maps `f` over the (phantom) result type of this except
		/// effect, with `Send + Sync` bounds so the operation
		/// composes inside thread-safe contexts.
		///
		/// Body is structurally identical to [`Functor::map`]'s; the
		/// `f` argument is discarded because `Throw` carries no
		/// `A`-typed payload.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original phantom result type.",
			"The new phantom result type."
		)]
		///
		#[document_parameters(
			"The function to map over the (phantom) result type. Must be `Send + Sync`.",
			"The except effect to map over."
		)]
		///
		#[document_returns("A new except effect with the new phantom result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::except::Except,
		/// 	},
		/// };
		///
		/// let throw: Except<'static, &'static str, i32> = Except::Throw("err", PhantomData);
		/// let mapped: Except<'static, &'static str, String> =
		/// 	<ExceptBrand<&'static str> as SendFunctor>::send_map(|x: i32| x.to_string(), throw);
		/// match mapped {
		/// 	Except::Throw(e, _) => assert_eq!(e, "err"),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			_f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Except::Throw(e, _) => Except::Throw(e, PhantomData),
			}
		}
	}
}

pub use inner::*;
