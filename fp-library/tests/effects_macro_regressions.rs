//! Macro hygiene regressions for effect generics.
//!
//! A generic occurs only when a type path is rooted at that generic: the
//! final `X` segment of `marker::X` is unrelated. Separately, a user generic
//! named `T` remains legal when a no-resume operation needs an internal
//! result parameter for its constructor.

#![cfg(feature = "effects")]

use fp_library::{
	brands::{
		BoxBrand,
		RcBrand,
	},
	define_effect,
	define_row,
	types::Free,
};

mod marker {
	/// A concrete payload whose final path segment deliberately matches the
	/// generic parameter used by the surrounding effect.
	pub struct X;
}

define_effect! {
	/// Uses generic `X` in one operation and the unrelated concrete
	/// `marker::X` in another.
	#[handler_state(none)]
	pub effect QualifiedPath<X: 'static> {
		/// Resume after consuming the actual effect generic.
		fn use_generic(value: X) -> ();
		/// Abort with a qualified concrete payload.
		fn stop(value: marker::X) -> !;
	}
}

define_effect! {
	/// Mixes one re-callable operation with an ordinary first-order operation.
	#[handler_state(none)]
	pub effect MixedShot {
		/// Fork through a re-callable continuation.
		#[multi_shot]
		fn fork() -> bool;
		/// Perform an ordinary single-resume step.
		fn ordinary() -> ();
	}
}

define_row! {
	/// A one-cell row used to pin constructor store types.
	pub row MixedShotRow {
		MixedShotBrand,
	}
}

define_effect! {
	/// Uses the formerly colliding generic name `T`.
	#[handler_state(none)]
	pub effect GenericT<T: 'static> {
		/// Abort with the generic payload.
		fn stop_t(value: T) -> !;
	}
}

#[test]
fn qualified_paths_and_generic_t_expand_without_phantom_parameters() {
	assert_eq!(std::mem::size_of::<QualifiedPathBrand<i32>>(), 0);
	assert_eq!(std::mem::size_of::<GenericTBrand<i32>>(), 0);
}

#[test]
fn only_the_marked_constructor_is_rc_pinned() {
	let _: Free<MixedShotRow, bool, RcBrand> = fork::<MixedShotRow, _>();
	let _: Free<MixedShotRow, (), BoxBrand> = ordinary::<MixedShotRow, _, BoxBrand>();
}
