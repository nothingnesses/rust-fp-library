use crate::{
	classes::{
		category::Category,
		clonable_fn::{ApplyClonableFn, ClonableFn},
		function::{ApplyFunction, Function},
		semigroupoid::Semigroupoid,
	},
	hkt::{Apply1L2T, Kind1L2T},
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcFnBrand;

impl Kind1L2T for ArcFnBrand {
	type Output<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;
}

impl Function for ArcFnBrand {
	type Output<'a, A, B> = Apply1L2T<'a, Self, A, B>;

	/// Creates a new `Arc`-wrapped function.
	///
	/// # Type Signature
	///
	/// `forall a b. Function ArcFnBrand => (a -> b) -> ArcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Arc`-wrapped function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::arc_fn::ArcFnBrand;
	/// use fp_library::classes::function::Function;
	///
	/// let f = <ArcFnBrand as Function>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyFunction<'a, Self, A, B> {
		Arc::new(f)
	}
}

impl ClonableFn for ArcFnBrand {
	type Output<'a, A, B> = Apply1L2T<'a, Self, A, B>;

	/// Creates a new `Arc`-wrapped clonable function.
	///
	/// # Type Signature
	///
	/// `forall a b. ClonableFn ArcFnBrand => (a -> b) -> ArcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Arc`-wrapped clonable function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::arc_fn::ArcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyClonableFn<'a, Self, A, B> {
		Arc::new(f)
	}
}

impl Semigroupoid for ArcFnBrand {
	/// Composes two `Arc`-wrapped functions.
	///
	/// # Type Signature
	///
	/// `forall b c d. Semigroupoid ArcFnBrand => (ArcFnBrand c d, ArcFnBrand b c) -> ArcFnBrand b d`
	///
	/// # Parameters
	///
	/// * `f`: The second function to apply.
	/// * `g`: The first function to apply.
	///
	/// # Returns
	///
	/// The composed function `f . g`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::arc_fn::ArcFnBrand;
	/// use fp_library::classes::semigroupoid::Semigroupoid;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let g = <ArcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	/// let h = ArcFnBrand::compose(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply1L2T<'a, Self, C, D>,
		g: Apply1L2T<'a, Self, B, C>,
	) -> Apply1L2T<'a, Self, B, D> {
		<Self as ClonableFn>::new(move |b| f(g(b)))
	}
}

impl Category for ArcFnBrand {
	/// Returns the identity function wrapped in an `Arc`.
	///
	/// # Type Signature
	///
	/// `forall a. Category ArcFnBrand => () -> ArcFnBrand a a`
	///
	/// # Returns
	///
	/// The identity function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::arc_fn::ArcFnBrand;
	/// use fp_library::classes::category::Category;
	///
	/// let id = ArcFnBrand::identity::<i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply1L2T<'a, Self, A, A> {
		Arc::new(|a| a)
	}
}
