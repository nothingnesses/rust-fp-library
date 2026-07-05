//! First-order versus higher-order effect classification.
//!
//! Every effect brand in a unified row carries an order marker: an effect is
//! first-order when no operation owns a sub-program (its representation does
//! not mention the row it lives in), and higher-order when some operation
//! does (the interpreter elaborates the owned sub-programs by recursive
//! interpretation). The marker is declarative type-level data: interpreters
//! and row machinery read it through [`OrderOf`] to route an active cell,
//! and the `define_effect!` macro computes it from the operation shapes, so
//! it cannot drift from the emitted cell.

#[fp_macros::document_module]
mod inner {
	/// The first-order order marker: the effect's representation does not own
	/// a sub-program, so interpreting it never recurses into the row.
	pub struct FirstOrder;

	/// The higher-order order marker: the effect owns at least one
	/// sub-program over its row, and interpreters elaborate it by recursive
	/// interpretation (native stack use grows with the nesting depth of
	/// higher-order cells, not with program length).
	pub struct HigherOrder;

	/// The order of an effect brand, as an associated marker type
	/// ([`FirstOrder`] or [`HigherOrder`]).
	///
	/// First-order and higher-order effects live in the same unified row and
	/// are told apart by this marker rather than by separate rows.
	pub trait OrderOf {
		/// The order marker for this effect brand.
		type Order;
	}
}

pub use inner::*;
