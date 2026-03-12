//! Types that form a Heyting algebra (bounded lattice with implication).
//!
//! ### Examples
//!
//! ```
//! use fp_library::classes::HeytingAlgebra;
//!
//! assert_eq!(bool::conjoin(true, false), false);
//! assert_eq!(bool::disjoin(true, false), true);
//! assert_eq!(bool::not(true), false);
//! ```

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;

	/// A type class for types that form a Heyting algebra.
	///
	/// A Heyting algebra is a bounded lattice with conjunction, disjunction, implication, and negation.
	///
	/// ### Laws
	///
	/// * Associativity: `disjoin(a, disjoin(b, c)) = disjoin(disjoin(a, b), c)` and
	///   `conjoin(a, conjoin(b, c)) = conjoin(conjoin(a, b), c)`
	/// * Commutativity: `disjoin(a, b) = disjoin(b, a)` and `conjoin(a, b) = conjoin(b, a)`
	/// * Absorption: `disjoin(a, conjoin(a, b)) = a` and `conjoin(a, disjoin(a, b)) = a`
	/// * Idempotence: `disjoin(a, a) = a` and `conjoin(a, a) = a`
	/// * Identity: `disjoin(a, false_value) = a` and `conjoin(a, true_value) = a`
	/// * Implication: `imply(a, a) = true_value` and `conjoin(a, imply(a, b)) = conjoin(a, b)`
	/// * Complementation: `not(a) = imply(a, false_value)`
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::HeytingAlgebra;
	///
	/// // Identity: conjoin(a, true_value) = a
	/// assert_eq!(bool::conjoin(true, bool::true_value()), true);
	/// assert_eq!(bool::conjoin(false, bool::true_value()), false);
	///
	/// // Implication: imply(a, a) = true_value
	/// assert_eq!(bool::imply(true, true), bool::true_value());
	/// assert_eq!(bool::imply(false, false), bool::true_value());
	/// ```
	pub trait HeytingAlgebra {
		/// Returns the bottom element (false).
		#[document_signature]
		///
		#[document_returns("The bottom element.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::false_value(), false);
		/// ```
		fn false_value() -> Self;

		/// Returns the top element (true).
		#[document_signature]
		///
		#[document_returns("The top element.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::true_value(), true);
		/// ```
		fn true_value() -> Self;

		/// Computes material implication.
		#[document_signature]
		///
		#[document_parameters("The antecedent.", "The consequent.")]
		///
		#[document_returns("The result of implication.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::imply(true, false), false);
		/// assert_eq!(bool::imply(false, true), true);
		/// ```
		fn imply(
			a: Self,
			b: Self,
		) -> Self;

		/// Computes the conjunction (logical AND).
		#[document_signature]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The conjunction of the two values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::conjoin(true, true), true);
		/// assert_eq!(bool::conjoin(true, false), false);
		/// ```
		fn conjoin(
			a: Self,
			b: Self,
		) -> Self;

		/// Computes the disjunction (logical OR).
		#[document_signature]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The disjunction of the two values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::disjoin(false, false), false);
		/// assert_eq!(bool::disjoin(true, false), true);
		/// ```
		fn disjoin(
			a: Self,
			b: Self,
		) -> Self;

		/// Computes the logical negation.
		#[document_signature]
		///
		#[document_parameters("The value to negate.")]
		///
		#[document_returns("The negation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::not(true), false);
		/// assert_eq!(bool::not(false), true);
		/// ```
		fn not(a: Self) -> Self;
	}

	/// Returns the bottom element (false).
	///
	/// Free function version that dispatches to [`HeytingAlgebra::false_value`].
	#[document_signature]
	///
	#[document_type_parameters("The Heyting algebra type.")]
	///
	#[document_returns("The bottom element.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::heyting_algebra::false_value;
	///
	/// assert_eq!(false_value::<bool>(), false);
	/// ```
	pub fn false_value<H: HeytingAlgebra>() -> H {
		H::false_value()
	}

	/// Returns the top element (true).
	///
	/// Free function version that dispatches to [`HeytingAlgebra::true_value`].
	#[document_signature]
	///
	#[document_type_parameters("The Heyting algebra type.")]
	///
	#[document_returns("The top element.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::heyting_algebra::true_value;
	///
	/// assert_eq!(true_value::<bool>(), true);
	/// ```
	pub fn true_value<H: HeytingAlgebra>() -> H {
		H::true_value()
	}

	/// Computes material implication.
	///
	/// Free function version that dispatches to [`HeytingAlgebra::imply`].
	#[document_signature]
	///
	#[document_type_parameters("The Heyting algebra type.")]
	///
	#[document_parameters("The antecedent.", "The consequent.")]
	///
	#[document_returns("The result of implication.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::heyting_algebra::imply;
	///
	/// assert_eq!(imply(true, false), false);
	/// ```
	pub fn imply<H: HeytingAlgebra>(
		a: H,
		b: H,
	) -> H {
		H::imply(a, b)
	}

	/// Computes the conjunction (logical AND).
	///
	/// Free function version that dispatches to [`HeytingAlgebra::conjoin`].
	#[document_signature]
	///
	#[document_type_parameters("The Heyting algebra type.")]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The conjunction of the two values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::heyting_algebra::conjoin;
	///
	/// assert_eq!(conjoin(true, false), false);
	/// ```
	pub fn conjoin<H: HeytingAlgebra>(
		a: H,
		b: H,
	) -> H {
		H::conjoin(a, b)
	}

	/// Computes the disjunction (logical OR).
	///
	/// Free function version that dispatches to [`HeytingAlgebra::disjoin`].
	#[document_signature]
	///
	#[document_type_parameters("The Heyting algebra type.")]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The disjunction of the two values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::heyting_algebra::disjoin;
	///
	/// assert_eq!(disjoin(true, false), true);
	/// ```
	pub fn disjoin<H: HeytingAlgebra>(
		a: H,
		b: H,
	) -> H {
		H::disjoin(a, b)
	}

	/// Computes the logical negation.
	///
	/// Free function version that dispatches to [`HeytingAlgebra::not`].
	#[document_signature]
	///
	#[document_type_parameters("The Heyting algebra type.")]
	///
	#[document_parameters("The value to negate.")]
	///
	#[document_returns("The negation.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::heyting_algebra::not;
	///
	/// assert_eq!(not(true), false);
	/// ```
	pub fn not<H: HeytingAlgebra>(a: H) -> H {
		H::not(a)
	}

	impl HeytingAlgebra for bool {
		/// Returns `false`.
		#[document_signature]
		///
		#[document_returns("`false`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::false_value(), false);
		/// ```
		fn false_value() -> Self {
			false
		}

		/// Returns `true`.
		#[document_signature]
		///
		#[document_returns("`true`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::true_value(), true);
		/// ```
		fn true_value() -> Self {
			true
		}

		/// Computes material implication (`!a || b`).
		#[document_signature]
		///
		#[document_parameters("The antecedent.", "The consequent.")]
		///
		#[document_returns("The result of implication.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::imply(true, false), false);
		/// ```
		fn imply(
			a: Self,
			b: Self,
		) -> Self {
			!a || b
		}

		/// Computes conjunction (`a && b`).
		#[document_signature]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The conjunction.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::conjoin(true, false), false);
		/// ```
		fn conjoin(
			a: Self,
			b: Self,
		) -> Self {
			a && b
		}

		/// Computes disjunction (`a || b`).
		#[document_signature]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The disjunction.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::disjoin(true, false), true);
		/// ```
		fn disjoin(
			a: Self,
			b: Self,
		) -> Self {
			a || b
		}

		/// Computes logical negation (`!a`).
		#[document_signature]
		///
		#[document_parameters("The value to negate.")]
		///
		#[document_returns("The negation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::HeytingAlgebra;
		///
		/// assert_eq!(bool::not(true), false);
		/// ```
		fn not(a: Self) -> Self {
			!a
		}
	}
}

pub use inner::*;
