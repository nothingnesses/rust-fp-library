use crate::{Apply, hkt::Kind_L1_T1_B0l0_Ol0};

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x, f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
pub trait Semimonad: Kind_L1_T1_B0l0_Ol0 {
	/// Sequences two computations, allowing the second to depend on the value computed by the first.
	///
	/// # Type Signature
	///
	/// `forall a b. Semimonad m => (m a, a -> m b) -> m b`
	///
	/// # Parameters
	///
	/// * `ma`: The first computation.
	/// * `f`: The function to apply to the result of the first computation.
	///
	/// # Returns
	///
	/// The result of the second computation.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::Semimonad;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::bind(x, |i| Some(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (A)),
		f: F,
	) -> Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (B))
	where
		F: Fn(A) -> Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (B)) + 'a;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the type class' associated function][`Semimonad::bind`].
///
/// # Type Signature
///
/// `forall a b. Semimonad m => (m a, a -> m b) -> m b`
///
/// # Parameters
///
/// * `ma`: The first computation.
/// * `f`: The function to apply to the result of the first computation.
///
/// # Returns
///
/// The result of the second computation.
///
/// # Examples
///
/// ```
/// use fp_library::classes::semimonad::bind;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
/// assert_eq!(y, Some(10));
/// ```
pub fn bind<'a, Brand: Semimonad, A: 'a, B: 'a, F>(
	ma: Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (A)),
	f: F,
) -> Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (B))
where
	F: Fn(A) -> Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (B)) + 'a,
{
	Brand::bind(ma, f)
}
