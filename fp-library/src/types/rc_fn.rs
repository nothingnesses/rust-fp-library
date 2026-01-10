use crate::{
	classes::{
		category::Category,
		clonable_fn::{ApplyClonableFn, ClonableFn},
		function::{ApplyFunction, Function},
		semigroupoid::Semigroupoid,
	},
	hkt::{Apply1L2T, Kind1L2T},
};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;

impl Kind1L2T for RcFnBrand {
	type Output<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
}

impl Function for RcFnBrand {
	type Output<'a, A, B> = Apply1L2T<'a, Self, A, B>;

	/// Creates a new `Rc`-wrapped function.
	///
	/// # Type Signature
	///
	/// `forall a b. Function RcFnBrand => (a -> b) -> RcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Rc`-wrapped function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::rc_fn::RcFnBrand;
	/// use fp_library::classes::function::Function;
	///
	/// let f = <RcFnBrand as Function>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyFunction<'a, Self, A, B> {
		Rc::new(f)
	}
}

impl ClonableFn for RcFnBrand {
	type Output<'a, A, B> = Apply1L2T<'a, Self, A, B>;

	/// Creates a new `Rc`-wrapped clonable function.
	///
	/// # Type Signature
	///
	/// `forall a b. ClonableFn RcFnBrand => (a -> b) -> RcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Rc`-wrapped clonable function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::rc_fn::RcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyClonableFn<'a, Self, A, B> {
		Rc::new(f)
	}
}

impl Semigroupoid for RcFnBrand {
	/// Composes two `Rc`-wrapped functions.
	///
	/// # Type Signature
	///
	/// `forall b c d. Semigroupoid RcFnBrand => (RcFnBrand c d, RcFnBrand b c) -> RcFnBrand b d`
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
	/// use fp_library::types::rc_fn::RcFnBrand;
	/// use fp_library::classes::semigroupoid::Semigroupoid;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let g = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	/// let h = RcFnBrand::compose(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply1L2T<'a, Self, C, D>,
		g: Apply1L2T<'a, Self, B, C>,
	) -> Apply1L2T<'a, Self, B, D> {
		<Self as ClonableFn>::new(move |b| f(g(b)))
	}
}

impl Category for RcFnBrand {
	/// Returns the identity function wrapped in an `Rc`.
	///
	/// # Type Signature
	///
	/// `forall a. Category RcFnBrand => () -> RcFnBrand a a`
	///
	/// # Returns
	///
	/// The identity function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::rc_fn::RcFnBrand;
	/// use fp_library::classes::category::Category;
	///
	/// let id = RcFnBrand::identity::<i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply1L2T<'a, Self, A, A> {
		Rc::new(|a| a)
	}
}
