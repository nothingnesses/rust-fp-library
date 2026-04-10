//! Reverse mapping from concrete types to their canonical brand.
//!
//! [`DefaultBrand`] provides the compiler with a way to infer the Brand type
//! parameter from a concrete container type, eliminating the need for turbofish
//! on free functions when the brand is unambiguous.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::default_brand::DefaultBrand,
//! };
//!
//! // Option<i32> has exactly one brand: OptionBrand
//! fn brand_of<T: DefaultBrand>() -> &'static str {
//! 	std::any::type_name::<T::Brand>()
//! }
//! assert!(brand_of::<Option<i32>>().contains("OptionBrand"));
//! assert!(brand_of::<Vec<String>>().contains("VecBrand"));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// Maps a concrete type back to its canonical brand.
	///
	/// Only implemented for types where the brand is unambiguous (one brand
	/// per concrete type). Types reachable through multiple brands (e.g.,
	/// `Result<A, E>` at arity 1) do not implement this trait and require
	/// explicit brand specification via turbofish.
	///
	/// A blanket implementation for references (`&T`) delegates to `T`'s
	/// implementation, enabling brand inference for both owned and borrowed
	/// containers.
	///
	/// ### When to implement
	///
	/// Implement `DefaultBrand` for types with exactly one brand at this arity.
	/// The `impl_kind!` macro will generate this automatically in the future;
	/// for now, implementations are hand-written.
	#[diagnostic::on_unimplemented(
		message = "`{Self}` does not have a unique brand and cannot use brand inference",
		note = "use the `_explicit` variant with a turbofish to specify the brand manually"
	)]
	pub trait DefaultBrand {
		/// The canonical brand for this type.
		type Brand: Kind_cdc7cd43dac7585f;
	}

	/// Blanket implementation for references.
	///
	/// Delegates to the underlying type's `DefaultBrand` implementation,
	/// enabling brand inference for borrowed containers (`&Vec<A>`, `&Option<A>`,
	/// etc.) via the same mechanism as owned containers.
	impl<'a, T: DefaultBrand + ?Sized> DefaultBrand for &'a T {
		type Brand = T::Brand;
	}
}

pub use inner::*;
