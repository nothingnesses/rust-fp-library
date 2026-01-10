use crate::{
    hkt::Kind1L0T,
    v2::classes::{monoid::Monoid, semigroup::Semigroup},
};

impl Kind1L0T for String {
    type Output<'a> = String;
}

impl Semigroup for String {
    /// Appends one string to another.
    ///
    /// # Type Signature
    ///
    /// `forall. Semigroup String => (String, String) -> String`
    ///
    /// # Parameters
    ///
    /// * `a`: The first string.
    /// * `b`: The second string.
    ///
    /// # Returns
    ///
    /// The concatenated string.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semigroup::append;
    ///
    /// assert_eq!(append("Hello, ".to_string(), "World!".to_string()), "Hello, World!".to_string());
    /// ```
    fn append(a: Self, b: Self) -> Self {
        a + &b
    }
}

impl Monoid for String {
    /// Returns an empty string.
    ///
    /// # Type Signature
    ///
    /// `forall. Monoid String => () -> String`
    ///
    /// # Returns
    ///
    /// An empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::monoid::empty;
    ///
    /// assert_eq!(empty::<String>(), "".to_string());
    /// ```
    fn empty() -> Self {
        String::new()
    }
}
