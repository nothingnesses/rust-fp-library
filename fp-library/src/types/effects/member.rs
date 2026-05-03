//! [`Member`] trait: first-order injection and projection over a Coproduct row.
//!
//! Mirrors the `Member` row-membership constraint that PureScript Run
//! and similar Haskell effect libraries use as a single bound on smart
//! constructors and handlers. A bound `R: Member<E, Idx>` reads as "the
//! row `R` contains effect `E` at type-level position `Idx`", and the
//! trait provides [`inject`](Member::inject) (lift an `E` into the row)
//! and [`project`](Member::project) (try to extract an `E`, returning
//! the rest of the row on failure).
//!
//! ## Layered on top of frunk
//!
//! The trait is a thin facade over
//! [`CoprodInjector`](crate::types::effects::coproduct::CoprodInjector)
//! and [`CoprodUninjector`](crate::types::effects::coproduct::CoprodUninjector). A blanket impl forwards
//! [`inject`](Member::inject) / [`project`](Member::project) to the
//! corresponding frunk methods, so every Coproduct value automatically
//! implements `Member<E, Idx>` for whichever `(E, Idx)` pairs frunk_core
//! can prove.
//!
//! Row narrowing (subset / sculpt) stays through
//! [`CoproductSubsetter`](crate::types::effects::coproduct::CoproductSubsetter)
//! directly. A separate `Members<Targets, Indices>` plural trait may be
//! added later if Phase 3 handler code wants the same single-bound
//! convenience for multi-effect narrowing; this module covers
//! single-effect membership only.
//!
//! ## What lives in row position
//!
//! `Member` is agnostic to whether row variants are bare effect types
//! `E`, [`Coyoneda<E, A>`](crate::types::Coyoneda)-wrapped effects, or
//! anything else. The Coyoneda-wrapping policy belongs to the smart
//! constructors that the [`effects!`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/plan.md)
//! macro emits (Phase 2 step 9), not to `Member` itself.

#[fp_macros::document_module]
mod inner {
	use crate::types::effects::coproduct::{
		CoprodInjector,
		CoprodUninjector,
	};

	/// Witness that the type `Self` (a Coproduct row) carries `E` at
	/// type-level position `Idx`.
	///
	/// Bounds: any `Self` that implements both
	/// [`CoprodInjector<E, Idx>`](crate::types::effects::coproduct::CoprodInjector)
	/// and
	/// [`CoprodUninjector<E, Idx>`](crate::types::effects::coproduct::CoprodUninjector)
	/// gets `Member<E, Idx>` via the blanket impl below. There is no need
	/// to add per-row impls.
	///
	/// # Example
	///
	/// ```
	/// use fp_library::types::effects::{
	/// 	coproduct::{
	/// 		CNil,
	/// 		Coproduct,
	/// 		Here,
	/// 	},
	/// 	member::Member,
	/// };
	///
	/// type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;
	///
	/// // Inject an `i32` at the head position of the row.
	/// let row: Row = <Row as Member<i32, Here>>::inject(7);
	/// assert!(matches!(row, Coproduct::Inl(7)));
	///
	/// // Project the `i32` back out.
	/// let result: Result<i32, Coproduct<&'static str, CNil>> =
	/// 	<Row as Member<i32, Here>>::project(row);
	/// assert_eq!(result, Ok(7));
	/// ```
	#[fp_macros::document_type_parameters(
		"The effect type to inject or project.",
		"The type-level position witness identifying where `E` lives in the row."
	)]
	#[fp_macros::document_parameters("The Coproduct row carrying `E` at position `Idx`.")]
	pub trait Member<E, Idx> {
		/// The remainder row produced when [`project`](Member::project)
		/// fails: the same row with `E` removed at position `Idx`.
		type Remainder;

		/// Inject a value of type `E` into the row at position `Idx`.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The value to inject into the row.")]
		///
		#[fp_macros::document_returns("The row carrying `value` at position `Idx`.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	coproduct::{
		/// 		CNil,
		/// 		Coproduct,
		/// 		Here,
		/// 	},
		/// 	member::Member,
		/// };
		///
		/// type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;
		/// let row: Row = <Row as Member<i32, Here>>::inject(7);
		/// assert!(matches!(row, Coproduct::Inl(7)));
		/// ```
		fn inject(value: E) -> Self;

		/// Try to project a value of type `E` out of the row at position
		/// `Idx`. Returns `Err` carrying the rest of the row when the
		/// runtime variant is not at position `Idx`.
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_returns(
			"`Ok(e)` if the active variant is at position `Idx`; `Err(remainder)` otherwise."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	coproduct::{
		/// 		CNil,
		/// 		Coproduct,
		/// 		Here,
		/// 	},
		/// 	member::Member,
		/// };
		///
		/// type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;
		/// let row: Row = Coproduct::inject(123_i32);
		/// let result: Result<i32, Coproduct<&'static str, CNil>> =
		/// 	<Row as Member<i32, Here>>::project(row);
		/// assert_eq!(result, Ok(123));
		/// ```
		fn project(self) -> Result<E, Self::Remainder>;
	}

	#[fp_macros::document_type_parameters(
		"The Coproduct row type implementing both injector and uninjector.",
		"The effect type at position `Idx`.",
		"The type-level position witness.",
		"The remainder row produced when projection fails."
	)]
	#[fp_macros::document_parameters("The row instance.")]
	impl<S, E, Idx, Rem> Member<E, Idx> for S
	where
		S: CoprodInjector<E, Idx> + CoprodUninjector<E, Idx, Remainder = Rem>,
	{
		type Remainder = Rem;

		/// Forwards to
		/// [`CoprodInjector::inject`](crate::types::effects::coproduct::CoprodInjector::inject).
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_parameters("The value to inject into the row.")]
		///
		#[fp_macros::document_returns("The row carrying `value` at position `Idx`.")]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	coproduct::{
		/// 		CNil,
		/// 		Coproduct,
		/// 		Here,
		/// 	},
		/// 	member::Member,
		/// };
		///
		/// type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;
		/// let row: Row = <Row as Member<i32, Here>>::inject(99);
		/// assert!(matches!(row, Coproduct::Inl(99)));
		/// ```
		fn inject(value: E) -> Self {
			<S as CoprodInjector<E, Idx>>::inject(value)
		}

		/// Forwards to
		/// [`CoprodUninjector::uninject`](crate::types::effects::coproduct::CoprodUninjector::uninject).
		#[fp_macros::document_signature]
		///
		#[fp_macros::document_returns(
			"`Ok(e)` if the active variant is at position `Idx`; `Err(remainder)` otherwise."
		)]
		///
		#[fp_macros::document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	coproduct::{
		/// 		CNil,
		/// 		Coproduct,
		/// 		Here,
		/// 	},
		/// 	member::Member,
		/// };
		///
		/// type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;
		/// let row: Row = Coproduct::inject(7_i32);
		/// let result: Result<i32, Coproduct<&'static str, CNil>> =
		/// 	<Row as Member<i32, Here>>::project(row);
		/// assert_eq!(result, Ok(7));
		/// ```
		fn project(self) -> Result<E, Self::Remainder> {
			<S as CoprodUninjector<E, Idx>>::uninject(self)
		}
	}
}

pub use inner::*;

#[cfg(test)]
#[expect(clippy::panic, reason = "Tests panic on unreachable Coproduct branches for clarity.")]
mod tests {
	use {
		super::*,
		crate::types::effects::coproduct::{
			CNil,
			Coproduct,
			Here,
			There,
		},
	};

	type Row = Coproduct<i32, Coproduct<&'static str, CNil>>;

	#[test]
	fn inject_at_head() {
		let row: Row = <Row as Member<i32, Here>>::inject(42);
		assert!(matches!(row, Coproduct::Inl(42)));
	}

	#[test]
	fn inject_at_tail() {
		let row: Row = <Row as Member<&'static str, There<Here>>>::inject("hello");
		match row {
			Coproduct::Inl(_) => panic!("expected Inr"),
			Coproduct::Inr(rest) => assert!(matches!(rest, Coproduct::Inl("hello"))),
		}
	}

	#[test]
	fn project_present_at_head() {
		let row: Row = Coproduct::inject(123_i32);
		let result: Result<i32, Coproduct<&'static str, CNil>> =
			<Row as Member<i32, Here>>::project(row);
		assert_eq!(result, Ok(123));
	}

	#[test]
	fn project_present_at_tail() {
		let row: Row = Coproduct::inject("found");
		let result: Result<&'static str, Coproduct<i32, CNil>> =
			<Row as Member<&'static str, There<Here>>>::project(row);
		assert_eq!(result, Ok("found"));
	}

	#[test]
	fn project_absent_returns_remainder() {
		let row: Row = Coproduct::inject("present");
		let result: Result<i32, Coproduct<&'static str, CNil>> =
			<Row as Member<i32, Here>>::project(row);
		match result {
			Ok(_) => panic!("expected Err carrying the remainder"),
			Err(rest) => assert!(matches!(rest, Coproduct::Inl("present"))),
		}
	}

	#[test]
	fn round_trip_through_member() {
		let injected: Row = <Row as Member<i32, Here>>::inject(99);
		let projected: Result<i32, Coproduct<&'static str, CNil>> =
			<Row as Member<i32, Here>>::project(injected);
		assert_eq!(projected, Ok(99));
	}
}
