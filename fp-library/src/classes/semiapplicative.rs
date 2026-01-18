//! Semiapplicative type class.
//!
//! This module defines the [`Semiapplicative`] trait, which provides the ability to apply functions within a context to values within a context.

use super::{clonable_fn::ClonableFn, functor::Functor, lift::Lift};
use crate::{Apply, kinds::*};

/// A type class for types that support function application within a context.
///
/// `Semiapplicative` provides the ability to apply functions that are themselves
/// wrapped in a context to values that are also wrapped in a context.
///
/// ### Laws
///
/// `Semiapplicative` instances must satisfy the following law:
/// * Composition: `apply(apply(f, g), x) = apply(f, apply(g, x))`.
pub trait Semiapplicative: Lift + Functor {
	/// Applies a function within a context to a value within a context.
	///
	/// This method applies a function wrapped in a context to a value wrapped in a context.
	///
	/// **Important**: This operation requires type erasure for heterogeneous functions.
	/// When a container (like `Vec`) holds multiple different closures, they must be
	/// type-erased via `Rc<dyn Fn>` or `Arc<dyn Fn>` because each Rust closure is a
	/// distinct anonymous type.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semiapplicative f => (f (a -> b), f a) -> f b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function wrapper.
	/// * `A`: The type of the input value.
	/// * `B`: The type of the output value.
	///
	/// ### Parameters
	///
	/// * `ff`: The context containing the function(s).
	/// * `fa`: The context containing the value(s).
	///
	/// ### Returns
	///
	/// A new context containing the result(s) of applying the function(s) to the value(s).
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::Semiapplicative;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// let x = Some(5);
	/// let y = OptionBrand::apply::<RcFnBrand, i32, i32>(f, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as ClonableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
}

/// Applies a function within a context to a value within a context.
///
/// Free function version that dispatches to [the type class' associated function][`Semiapplicative::apply`].
///
/// ### Type Signature
///
/// `forall a b. Semiapplicative f => (f (a -> b), f a) -> f b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the context.
/// * `FnBrand`: The brand of the clonable function wrapper.
/// * `A`: The type of the input value.
/// * `B`: The type of the output value.
///
/// ### Parameters
///
/// * `ff`: The context containing the function(s).
/// * `fa`: The context containing the value(s).
///
/// ### Returns
///
/// A new context containing the result(s) of applying the function(s) to the value(s).
///
/// ### Examples
///
/// ```
/// use fp_library::classes::semiapplicative::apply;
/// use fp_library::classes::clonable_fn::ClonableFn;
/// use fp_library::brands::OptionBrand;
/// use fp_library::brands::RcFnBrand;
/// use std::rc::Rc;
///
/// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
/// let x = Some(5);
/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn apply<'a, FnBrand: 'a + ClonableFn, Brand: Semiapplicative, A: 'a + Clone, B: 'a>(
	ff: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as ClonableFn>::Of<'a, A, B>>),
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
	Brand::apply::<FnBrand, A, B>(ff, fa)
}
