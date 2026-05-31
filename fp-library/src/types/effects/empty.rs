//! Abortive first-order effect type for nondeterministic branches with
//! no results. The corresponding brand is
//! [`EmptyBrand`](crate::brands::EmptyBrand).
//!
//! `Empty<'a, A>` mirrors Heftia's separate `Empty` effect: the
//! operation has no continuation and never produces an `A`. This keeps
//! nondeterministic failure distinct from `Choose`, whose operation is
//! branching rather than abortive.
//!
//! ## No pointer-brand parameter
//!
//! Like [`Except`](crate::types::effects::except::Except), `Empty` has
//! no `dyn Fn` continuation. The same effect type and brand work across
//! all six Run wrappers; thread-safe wrappers rely only on the
//! structurally trivial [`SendFunctor`](crate::classes::SendFunctor)
//! implementation.
//!
//! See the parent [`effects`](crate::types::effects) guide for the
//! consolidated wrapper and pointer-brand conventions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::EmptyBrand,
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

	/// Abortive first-order effect type.
	///
	/// The single `Empty` variant carries only a phantom result type.
	/// A handler decides how the branch disappears in the target
	/// interpretation, for example by returning an empty `Vec` in a
	/// nondeterministic list interpreter.
	#[document_type_parameters(
		"The lifetime of the effect (carried for `Kind`-projection purposes only; nothing in the variant borrows from it).",
		"The phantom result type."
	)]
	#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub enum Empty<'a, A: 'a> {
		/// Abort the current branch without producing an `A`.
		Empty(PhantomData<&'a A>),
	}

	#[document_type_parameters(
		"The lifetime carried by the Empty effect.",
		"The phantom result type."
	)]
	#[document_parameters("The Empty operation to clone.")]
	impl<'a, A> Clone for Empty<'a, A>
	where
		A: 'a,
	{
		/// Clones the empty effect without requiring the phantom result
		/// type to be cloneable.
		#[document_signature]
		#[document_returns("A new Empty operation with the same phantom result type.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::types::effects::empty::Empty,
		/// };
		///
		/// let original: Empty<'static, String> = Empty::Empty(PhantomData);
		/// let cloned = original.clone();
		/// assert!(matches!(cloned, Empty::Empty(_)));
		/// ```
		fn clone(&self) -> Self {
			*self
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the Empty effect.",
		"The phantom result type."
	)]
	impl<'a, A> Copy for Empty<'a, A> where A: 'a {}

	#[document_type_parameters(
		"The lifetime carried by the Empty effect.",
		"The phantom result type."
	)]
	impl<'a, A> Default for Empty<'a, A>
	where
		A: 'a,
	{
		/// Builds an empty effect with the requested phantom result
		/// type.
		#[document_signature]
		#[document_returns("An Empty operation with the requested phantom result type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::effects::empty::Empty;
		///
		/// let op: Empty<'static, i32> = Empty::default();
		/// assert!(matches!(op, Empty::Empty(_)));
		/// ```
		fn default() -> Self {
			Empty::Empty(PhantomData)
		}
	}

	impl_kind! {
		impl for EmptyBrand {
			type Of<'a, A: 'a>: 'a = Empty<'a, A>;
		}
	}

	impl Functor for EmptyBrand {
		/// Maps over the phantom result type of an `Empty` operation.
		///
		/// `Empty` carries no `A`-typed payload, so the function is
		/// discarded and the operation is rebuilt with the new phantom
		/// result type.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original phantom result type.",
			"The new phantom result type."
		)]
		#[document_parameters(
			"The function to map over the phantom result type.",
			"The Empty operation to map over."
		)]
		#[document_returns("A new Empty operation with the new phantom result type.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::EmptyBrand,
		/// 		classes::Functor,
		/// 		types::effects::empty::Empty,
		/// 	},
		/// };
		///
		/// let op: Empty<'static, i32> = Empty::Empty(PhantomData);
		/// let mapped: Empty<'static, String> =
		/// 	<EmptyBrand as Functor>::map(|value: i32| value.to_string(), op);
		/// assert!(matches!(mapped, Empty::Empty(_)));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			_f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Empty::Empty(_) => Empty::Empty(PhantomData),
			}
		}
	}

	impl SendFunctor for EmptyBrand {
		/// Maps over the phantom result type of an `Empty` operation in
		/// thread-safe contexts.
		///
		/// Body is structurally identical to [`Functor::map`]'s; the
		/// extra bounds only satisfy the `SendFunctor` contract.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original phantom result type.",
			"The new phantom result type."
		)]
		#[document_parameters(
			"The function to map over the phantom result type.",
			"The Empty operation to map over."
		)]
		#[document_returns("A new Empty operation with the new phantom result type.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::EmptyBrand,
		/// 		classes::SendFunctor,
		/// 		types::effects::empty::Empty,
		/// 	},
		/// };
		///
		/// let op: Empty<'static, i32> = Empty::Empty(PhantomData);
		/// let mapped: Empty<'static, String> =
		/// 	<EmptyBrand as SendFunctor>::send_map(|value: i32| value.to_string(), op);
		/// assert!(matches!(mapped, Empty::Empty(_)));
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			_f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Empty::Empty(_) => Empty::Empty(PhantomData),
			}
		}
	}
}

pub use inner::*;
