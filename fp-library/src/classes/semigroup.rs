/// A type class for types that support an associative binary operation.
///
/// `Semigroup` instances must satisfy the associative law:
/// * Associativity: `append(a, append(b, c)) = append(append(a, b), c)`.
pub trait Semigroup {
	/// The result of combining the two values using the semigroup operation.
	///
	/// # Type Signature
	///
	/// `forall a. Semigroup a => (a, a) -> a`
	///
	/// # Parameters
	///
	/// * `a`: The first value.
	/// * `b`: The second value.
	///
	/// # Returns
	///
	/// The combined value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semigroup::Semigroup;
	/// use fp_library::types::string; // Import Semigroup impl for String
	///
	/// let x = "Hello, ".to_string();
	/// let y = "World!".to_string();
	/// let z = String::append(x, y);
	/// assert_eq!(z, "Hello, World!".to_string());
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self;
}

/// The result of combining the two values using the semigroup operation.
///
/// Free function version that dispatches to [the type class' associated function][`Semigroup::append`].
///
/// # Type Signature
///
/// `forall a. Semigroup a => (a, a) -> a`
///
/// # Parameters
///
/// * `a`: The first value.
/// * `b`: The second value.
///
/// # Returns
///
/// The combined value.
///
/// # Examples
///
/// ```
/// use fp_library::classes::semigroup::append;
/// use fp_library::types::string; // Import Semigroup impl for String
///
/// let x = "Hello, ".to_string();
/// let y = "World!".to_string();
/// let z = append(x, y);
/// assert_eq!(z, "Hello, World!".to_string());
/// ```
pub fn append<S: Semigroup>(
	a: S,
	b: S,
) -> S {
	S::append(a, b)
}
