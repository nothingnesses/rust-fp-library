//! Regression test documenting the GAT-normalization limit encountered
//! while implementing `ArcRun::send` in Phase 2 step 5.
//!
//! ## The limit
//!
//! When a struct, impl block, or function carries an HRTB on a generic
//! associated type (e.g.,
//! `NodeBrand<R, S>: Kind<Of<'static, ArcFree<...>>: Send + Sync>`), the
//! Rust compiler refuses to normalize *any* projection of that GAT in
//! the same scope, even at different instantiations. So
//! `<NodeBrand<R, S> as Kind>::Of<'static, A>` cannot be unified with the
//! literal `Node<'static, R, S, A>` value (despite `impl_kind!` declaring
//! them equal) inside such a scope.
//!
//! The probe at `compile_fail/arc_run_hrtb_*.rs` (UI tests) captures the
//! failures concretely. This file captures the **patterns that work**
//! despite the limit:
//!
//! - **Passing projection-typed values into HRTB scope** (workaround).
//! - **HRTB-free helpers** that produce projection-typed values for
//!   downstream HRTB-bearing consumers.
//! - **Free-substrate calls in non-HRTB contexts** (baseline: confirms
//!   the issue is HRTB-specific, not Free-vs-ArcFree-specific).
//!
//! ## Why this stays in tests/
//!
//! The Phase 2 step 5 commit changes `*Run::send` across all six
//! wrappers to take the `Node`-projection value (rather than the row
//! variant) so the workaround becomes the API. Future maintainers
//! confused by the Node-projection `send` signature can read this file
//! to understand the GAT-normalization constraint that drove the
//! design.

#![allow(dead_code)]
#![allow(unused_imports)]

use {
	fp_library::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			IdentityBrand,
			NodeBrand,
		},
		classes::{
			Functor,
			WrapDrop,
		},
		kinds::*,
		types::{
			ArcFree,
			Free,
			RcFree,
			arc_free::ArcTypeErasedValue,
			effects::node::Node,
			rc_free::RcTypeErasedValue,
		},
	},
	fp_macros::*,
};

// -- Pattern A: HRTB-bearing function takes projection, not row variant --
// The workaround. `send` accepts the `Node`-projection value already
// constructed by an HRTB-free caller.

mod pattern_a_pass_projection_in {
	use super::*;

	struct Wrapper<R, S, A>(ArcFree<NodeBrand<R, S>, A>)
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static;

	impl<R, S, A> Wrapper<R, S, A>
	where
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: 'static,
	{
		fn send(
			node: Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		) -> Self
		where
			NodeBrand<R, S>: Functor,
			A: Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			Wrapper(ArcFree::<NodeBrand<R, S>, A>::lift_f(node))
		}
	}
}

// -- Pattern B: HRTB-free helper produces projection from Node literal --
// Companion to pattern A: a generic helper without HRTB that constructs
// the `Node` literal and returns it at the projection type. Callers
// invoke this helper in their (HRTB-free) context, then pass the result
// to HRTB-scope methods like Pattern A's `send`.

mod pattern_b_hrtb_free_helper {
	use super::*;

	fn make_first_layer<R, S, A>(
		layer: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	) -> Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'static, {
		Node::First(layer)
	}
}

// -- Pattern C: Free-substrate call in HRTB-free function -----------------
// Baseline confirming the limit is HRTB-specific, not Free-family-wide.
// Same construction (`Node::First` + `Free::lift_f`) succeeds when no
// HRTB is in scope.

mod pattern_c_free_no_hrtb {
	use super::*;

	fn send<R, S, A>(
		layer: Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	) -> Free<NodeBrand<R, S>, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static, {
		let node = Node::First(layer);
		Free::<NodeBrand<R, S>, A>::lift_f(node)
	}
}

// -- Pattern D: Identity-cast helper ---------------------------------------
// Demonstrates that `Node<'static, R, S, A> -> projection` is a valid
// identity in HRTB-free scope. This is the type-laundering primitive that
// other helpers (like Pattern B) rely on.

mod pattern_d_identity_cast {
	use super::*;

	fn coerce<R, S, A>(
		x: Node<'static, R, S, A>
	) -> Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		S: Kind_cdc7cd43dac7585f + 'static,
		A: 'static, {
		x
	}
}
