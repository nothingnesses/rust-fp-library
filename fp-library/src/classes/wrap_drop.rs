//! Drop-time decomposition for the [`Free`](crate::types::Free) family.
//!
//! [`WrapDrop`] is the structural counterpart of
//! [`Extract`](crate::classes::Extract). `Extract` answers the semantic
//! question "given `F::Of<X>`, what is the inner `X`?" and is used by
//! interpreters such as [`Free::evaluate`](crate::types::Free::evaluate)
//! and [`Free::fold_free`](crate::types::Free::fold_free). `WrapDrop`
//! answers a narrower, structural question: "if you can hand the inner
//! `X` back without running user code, do so; otherwise return `None`
//! and let the layer drop in place."
//!
//! Returning `Some(x)` lets the [`Free`](crate::types::Free) family's
//! `Drop` impl keep using the existing iterative path that prevents
//! stack overflow on deep `Wrap` chains. Returning `None` is the
//! safe fallback for brands that do not materially store the inner
//! `X`; `Drop` then falls through to recursive drop on the layer.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	types::*,
//! };
//!
//! let id = Identity(42);
//! assert_eq!(<IdentityBrand as WrapDrop>::drop(id), Some(42));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// Drop-time decomposition for a higher-kinded type.
	///
	/// `WrapDrop` is the structural counterpart of
	/// [`Extract`](crate::classes::Extract). It exists so the
	/// [`Free`](crate::types::Free) family's `Drop` impl can iteratively
	/// dismantle deep `Wrap` chains without committing the struct to the
	/// stronger semantic guarantee that [`Extract`](crate::classes::Extract)
	/// represents (a total `F ~> Id`). Brands that materially store the
	/// inner `X` opt in by returning `Some`; brands that don't (e.g.,
	/// effect-row brands like
	/// [`CoyonedaBrand`](crate::brands::CoyonedaBrand)) return `None` and
	/// rely on recursive drop being sound for the patterns they get used
	/// in.
	///
	/// # Soundness of `None`
	///
	/// Returning `None` makes `Drop` fall through to recursive drop on
	/// the layer. For Run-shaped programs (effects injected via
	/// `lift_f` and chained via `bind`), the structural `Wrap` depth is
	/// at most 1; the depth that grows with chain length lives in the
	/// `CatList` of continuations, which the iterative drop loop
	/// already dismantles without recursion. Artificial deep
	/// `wrap(...)` chains over an `F` whose `WrapDrop::drop` returns
	/// `None` overflow the stack on `Drop`; brands that need to
	/// support such patterns must return `Some(x)` instead.
	///
	/// # Relationship to `Extract`
	///
	/// Any `F: Extract` can implement `WrapDrop` by delegating:
	/// `WrapDrop::drop(fa) = Some(<F as Extract>::extract(fa))`. The
	/// reverse is not true: many useful brands (effect-row brands, the
	/// `Coyoneda` and `Coproduct` families) have no canonical
	/// `Extract` impl but can still answer `WrapDrop::drop(_) = None`
	/// safely.
	///
	/// # Method name
	///
	/// The method is named `drop` to reflect the operation's role
	/// during the `Free` family's `Drop`. It does not clash with
	/// [`std::ops::Drop::drop`] because they live on different traits
	/// with different receiver shapes (the standard one takes
	/// `&mut self`; this one takes an owned `F::Of<'_, X>`). Call
	/// sites use fully-qualified syntax such as
	/// `<F as WrapDrop>::drop(fa)`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait WrapDrop {
		/// Drop-time decomposition of `F::Of<'a, X>`.
		///
		/// Returns `Some(x)` to indicate that `F::Of<X>` materially
		/// holds an `X` that the caller can iterate on; returns `None`
		/// to indicate the layer should be dropped in place by the
		/// caller (typically because the inner `X` is closure-captured
		/// or absent).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value the layer would yield."
		)]
		///
		#[document_parameters("The functor layer being decomposed.")]
		///
		#[document_returns(
			"`Some(x)` if `F::Of<X>` materially holds an inner `X` that the caller may iterate on; `None` if the layer should be dropped in place."
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
		/// let id = Identity(42);
		/// assert_eq!(<IdentityBrand as WrapDrop>::drop(id), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X>;
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use crate::{
		brands::*,
		classes::*,
		types::*,
	};

	#[test]
	fn identity_brand_wrap_drop_returns_some() {
		let id = Identity(42);
		assert_eq!(<IdentityBrand as WrapDrop>::drop(id), Some(42));
	}

	#[test]
	fn thunk_brand_wrap_drop_returns_some() {
		let thunk = Thunk::new(|| 7);
		assert_eq!(<ThunkBrand as WrapDrop>::drop(thunk), Some(7));
	}
}
