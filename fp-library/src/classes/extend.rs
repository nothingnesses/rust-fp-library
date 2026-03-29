//! Cooperative extension of a local context-dependent computation to a global computation.
//!
//! [`Extend`] is the dual of [`Semimonad`](crate::classes::Semimonad): where `bind` sequences
//! computations by extracting a value and feeding it to a function that produces a new context,
//! `extend` takes a function that consumes a whole context and lifts it to operate within
//! the context. In categorical terms, `extend` is co-Kleisli extension.
//!
//! This module is a port of PureScript's
//! [`Control.Extend`](https://pursuit.purescript.org/packages/purescript-control/docs/Control.Extend).

#[fp_macros::document_module(no_validation)]
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
		fn extend<'a, A: 'a, B: 'a>(
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
		fn duplicate<'a, A: 'a>(
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): 'a, {
			Self::extend(|w| w, wa)
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
	pub fn extend<'a, Brand: Extend, A: 'a, B: 'a>(
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
	pub fn duplicate<'a, Brand: Extend, A: 'a>(
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): 'a, {
		Brand::extend(|w| w, wa)
	}

	/// Forwards co-Kleisli composition.
	///
	/// Composes two co-Kleisli functions left-to-right: first applies `f` via
	/// [`extend`], then applies `g` to the result. This is the dual of
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
	pub fn compose_co_kleisli<'a, Brand: Extend, A: 'a, B: 'a, C: 'a>(
		f: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
		g: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)) -> C + 'a,
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> C {
		g(Brand::extend(f, wa))
	}

	/// Backwards co-Kleisli composition.
	///
	/// Composes two co-Kleisli functions right-to-left: first applies `g` via
	/// [`extend`], then applies `f` to the result. This is the dual of
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
	pub fn compose_co_kleisli_flipped<'a, Brand: Extend, A: 'a, B: 'a, C: 'a>(
		f: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)) -> C + 'a,
		g: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> C {
		f(Brand::extend(g, wa))
	}

	/// Extends with the arguments flipped.
	///
	/// A version of [`extend`] where the comonadic context comes first, followed
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
	pub fn extend_flipped<'a, Brand: Extend, A: 'a, B: 'a>(
		wa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl Fn(Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::extend(f, wa)
	}
}

pub use inner::*;
