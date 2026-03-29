//! Cooperative extension of a local context-dependent computation to a global computation.
//!
//! [`Extend`] is the dual of [`Semimonad`](crate::classes::Semimonad): where `bind` sequences
//! computations by extracting a value and feeding it to a function that produces a new context,
//! `extend` takes a function that consumes a whole context and lifts it to operate within
//! the context. In categorical terms, `extend` is co-Kleisli extension.
//!
//! This module is a port of PureScript's
//! [`Control.Extend`](https://pursuit.purescript.org/packages/purescript-control/docs/Control.Extend).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for types that support co-Kleisli extension.
	///
	/// `Extend` is the dual of [`Semimonad`](crate::classes::Semimonad). Where
	/// `bind : (A -> F<B>) -> F<A> -> F<B>` feeds a single extracted value into a
	/// function that produces a new context, `extend : (F<A> -> B) -> F<A> -> F<B>`
	/// feeds an entire context into a function and re-wraps the result.
	///
	/// `class Functor w <= Extend w`
	///
	/// # Laws
	///
	/// **Associativity:** composing two extensions is the same as extending with
	/// a pre-composed function.
	///
	/// For any `f: F<B> -> C` and `g: F<A> -> B`:
	///
	/// ```text
	/// extend(f, extend(g, w)) == extend(|w| f(extend(g, w)), w)
	/// ```
	///
	/// This is dual to the associativity law for `bind`.
	///
	/// # Note on `LazyBrand`
	///
	/// `LazyBrand` cannot implement `Extend` because `Extend: Functor` and
	/// `LazyBrand` cannot implement `Functor` (its `evaluate` returns `&A`,
	/// not owned `A`). PureScript's `Lazy` has `Extend`/`Comonad` because GC
	/// provides owned values from `force`. In this library, `ThunkBrand` fills
	/// the `Functor + Extend + Comonad` role for lazy types, while `LazyBrand`
	/// provides memoization via [`RefFunctor`](crate::classes::RefFunctor).
	#[document_examples]
	///
	/// Associativity law for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let w = vec![1, 2, 3];
	/// let f = |v: Vec<i32>| v.iter().sum::<i32>();
	/// let g = |v: Vec<i32>| v.len() as i32;
	///
	/// // extend(f, extend(g, w)) == extend(|w| f(extend(g, w)), w)
	/// let lhs = extend::<VecBrand, _, _>(f, extend::<VecBrand, _, _>(g, w.clone()));
	/// let rhs = extend::<VecBrand, _, _>(|w| f(extend::<VecBrand, _, _>(g, w)), w);
	/// assert_eq!(lhs, rhs);
	/// ```
	pub trait Extend: Functor {
		/// Extends a local context-dependent computation to a global computation.
		///
		/// Given a function that consumes an `F<A>` and produces a `B`, and a
		/// value of type `F<A>`, produces an `F<B>` by applying the function in
		/// a context-sensitive way.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the comonadic context.",
			"The result type of the extension function."
		)]
		///
		#[document_parameters(
			"The function that consumes a whole context and produces a value.",
			"The comonadic context to extend over."
		)]
		///
		#[document_returns(
			"A new comonadic context containing the results of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let result = IdentityBrand::extend(|id: Identity<i32>| id.0 * 2, Identity(5));
		/// assert_eq!(result, Identity(10));
		/// ```
		fn extend<'a, A: 'a + Clone, B: 'a>(
			f: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);

		/// Duplicates a comonadic context, wrapping it inside another layer of the same context.
		///
		/// `duplicate(wa)` is equivalent to `extend(identity, wa)`. It is the dual of
		/// [`join`](crate::functions::join) for monads.
		///
		/// Produces `F<F<A>>` from `F<A>`, embedding the original context as the inner value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the comonadic context."
		)]
		///
		#[document_parameters("The comonadic context to duplicate.")]
		///
		#[document_returns("A doubly-wrapped comonadic context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let result = IdentityBrand::duplicate(Identity(5));
		/// assert_eq!(result, Identity(Identity(5)));
		/// ```
		fn duplicate<'a, A: 'a + Clone>(
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): 'a, {
			Self::extend(|w| w, wa)
		}

		/// Extends with the arguments flipped.
		///
		/// A version of [`extend`](Extend::extend) where the comonadic context comes
		/// first, followed by the extension function.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the comonadic context.",
			"The result type of the extension function."
		)]
		///
		#[document_parameters(
			"The comonadic context to extend over.",
			"The function that consumes a whole context and produces a value."
		)]
		///
		#[document_returns(
			"A new comonadic context containing the results of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let result = IdentityBrand::extend_flipped(Identity(5), |id| id.0 * 3);
		/// assert_eq!(result, Identity(15));
		/// ```
		fn extend_flipped<'a, A: 'a + Clone, B: 'a>(
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::extend(f, wa)
		}

		/// Forwards co-Kleisli composition.
		///
		/// Composes two co-Kleisli functions left-to-right: first applies `f` via
		/// [`extend`](Extend::extend), then applies `g` to the result. This is the
		/// dual of [`compose_kleisli`](crate::functions::compose_kleisli).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the comonadic context.",
			"The result type of the first co-Kleisli function.",
			"The result type of the second co-Kleisli function."
		)]
		///
		#[document_parameters(
			"The first co-Kleisli function.",
			"The second co-Kleisli function.",
			"The comonadic context to operate on."
		)]
		///
		#[document_returns(
			"The result of composing both co-Kleisli functions and applying them to the context."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let f = |id: Identity<i32>| id.0 + 1;
		/// let g = |id: Identity<i32>| id.0 * 10;
		/// let result = IdentityBrand::compose_co_kleisli(f, g, Identity(5));
		/// assert_eq!(result, 60);
		/// ```
		fn compose_co_kleisli<'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
			g: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)) -> C + 'a,
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> C {
			g(Self::extend(f, wa))
		}

		/// Backwards co-Kleisli composition.
		///
		/// Composes two co-Kleisli functions right-to-left: first applies `g` via
		/// [`extend`](Extend::extend), then applies `f` to the result. This is the
		/// dual of [`compose_kleisli_flipped`](crate::functions::compose_kleisli_flipped).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the comonadic context.",
			"The result type of the second co-Kleisli function (applied first).",
			"The result type of the first co-Kleisli function (applied second)."
		)]
		///
		#[document_parameters(
			"The second co-Kleisli function (applied after `g`).",
			"The first co-Kleisli function (applied first to the context).",
			"The comonadic context to operate on."
		)]
		///
		#[document_returns(
			"The result of composing both co-Kleisli functions and applying them to the context."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let f = |id: Identity<i32>| id.0 * 10;
		/// let g = |id: Identity<i32>| id.0 + 1;
		/// let result = IdentityBrand::compose_co_kleisli_flipped(f, g, Identity(5));
		/// assert_eq!(result, 60);
		/// ```
		fn compose_co_kleisli_flipped<'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)) -> C + 'a,
			g: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> C {
			f(Self::extend(g, wa))
		}
	}

	/// Extends a local context-dependent computation to a global computation.
	///
	/// Free function version that dispatches to [the type class' associated function][`Extend::extend`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the comonadic context.",
		"The type of the value(s) inside the comonadic context.",
		"The result type of the extension function."
	)]
	///
	#[document_parameters(
		"The function that consumes a whole context and produces a value.",
		"The comonadic context to extend over."
	)]
	///
	#[document_returns("A new comonadic context containing the results of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // extend on Identity: apply f to the whole container
	/// let w = Identity(10);
	/// let result = extend::<IdentityBrand, _, _>(|id| id.0 * 2, w);
	/// assert_eq!(result, Identity(20));
	///
	/// // extend on Vec: apply f to each suffix
	/// let v = vec![1, 2, 3];
	/// let sums = extend::<VecBrand, _, _>(|s: Vec<i32>| s.iter().sum::<i32>(), v);
	/// assert_eq!(sums, vec![6, 5, 3]);
	/// ```
	pub fn extend<'a, Brand: Extend, A: 'a + Clone, B: 'a>(
		f: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::extend(f, wa)
	}

	/// Duplicates a comonadic context, wrapping it inside another layer of the same context.
	///
	/// `duplicate(wa)` is equivalent to `extend(identity, wa)`. It is the dual of
	/// [`join`](crate::functions::join) for monads.
	///
	/// Produces `F<F<A>>` from `F<A>`, embedding the original context as the inner value.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the comonadic context.",
		"The type of the value(s) inside the comonadic context."
	)]
	///
	#[document_parameters("The comonadic context to duplicate.")]
	///
	#[document_returns("A doubly-wrapped comonadic context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // duplicate on Vec produces all suffixes
	/// let v = vec![1, 2, 3];
	/// let d = duplicate::<VecBrand, _>(v);
	/// assert_eq!(d, vec![vec![1, 2, 3], vec![2, 3], vec![3]]);
	/// ```
	pub fn duplicate<'a, Brand: Extend, A: 'a + Clone>(
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): 'a, {
		Brand::duplicate(wa)
	}

	/// Forwards co-Kleisli composition.
	///
	/// Composes two co-Kleisli functions left-to-right: first applies `f` via
	/// [`extend()`](crate::classes::extend::extend), then applies `g` to the result. This is the dual of
	/// [`compose_kleisli`](crate::functions::compose_kleisli).
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the comonadic context.",
		"The type of the value(s) inside the comonadic context.",
		"The result type of the first co-Kleisli function.",
		"The result type of the second co-Kleisli function."
	)]
	///
	#[document_parameters(
		"The first co-Kleisli function.",
		"The second co-Kleisli function.",
		"The comonadic context to operate on."
	)]
	///
	#[document_returns(
		"The result of composing both co-Kleisli functions and applying them to the context."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let f = |id: Identity<i32>| id.0 + 1;
	/// let g = |id: Identity<i32>| id.0 * 10;
	/// let w = Identity(5);
	/// // compose_co_kleisli(f, g, w): extend f, then apply g
	/// let result = compose_co_kleisli::<IdentityBrand, _, _, _>(f, g, w);
	/// assert_eq!(result, 60);
	/// ```
	pub fn compose_co_kleisli<'a, Brand: Extend, A: 'a + Clone, B: 'a + Clone, C: 'a>(
		f: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
		g: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)) -> C + 'a,
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> C {
		Brand::compose_co_kleisli(f, g, wa)
	}

	/// Backwards co-Kleisli composition.
	///
	/// Composes two co-Kleisli functions right-to-left: first applies `g` via
	/// [`extend()`](crate::classes::extend::extend), then applies `f` to the result. This is the dual of
	/// [`compose_kleisli_flipped`](crate::functions::compose_kleisli_flipped).
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the comonadic context.",
		"The type of the value(s) inside the comonadic context.",
		"The result type of the second co-Kleisli function (applied first).",
		"The result type of the first co-Kleisli function (applied second)."
	)]
	///
	#[document_parameters(
		"The second co-Kleisli function (applied after `g`).",
		"The first co-Kleisli function (applied first to the context).",
		"The comonadic context to operate on."
	)]
	///
	#[document_returns(
		"The result of composing both co-Kleisli functions and applying them to the context."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let f = |id: Identity<i32>| id.0 * 10;
	/// let g = |id: Identity<i32>| id.0 + 1;
	/// let w = Identity(5);
	/// // compose_co_kleisli_flipped(f, g, w): extend g, then apply f
	/// let result = compose_co_kleisli_flipped::<IdentityBrand, _, _, _>(f, g, w);
	/// assert_eq!(result, 60);
	/// ```
	pub fn compose_co_kleisli_flipped<'a, Brand: Extend, A: 'a + Clone, B: 'a + Clone, C: 'a>(
		f: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)) -> C + 'a,
		g: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> C {
		Brand::compose_co_kleisli_flipped(f, g, wa)
	}

	/// Extends with the arguments flipped.
	///
	/// A version of [`extend()`](crate::classes::extend::extend) where the comonadic context comes first, followed
	/// by the extension function. Useful for pipelines where the value is known
	/// before the function.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the comonadic context.",
		"The type of the value(s) inside the comonadic context.",
		"The result type of the extension function."
	)]
	///
	#[document_parameters(
		"The comonadic context to extend over.",
		"The function that consumes a whole context and produces a value."
	)]
	///
	#[document_returns("A new comonadic context containing the results of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let w = Identity(5);
	/// let result = extend_flipped::<IdentityBrand, _, _>(w, |id| id.0 * 3);
	/// assert_eq!(result, Identity(15));
	/// ```
	pub fn extend_flipped<'a, Brand: Extend, A: 'a + Clone, B: 'a>(
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::extend_flipped(wa, f)
	}
}

pub use inner::*;
