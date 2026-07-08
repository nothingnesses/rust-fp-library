//! FS-1 slice: the `Interpose` transformer (a Free-rewriting walker).
//!
//! Unlike the per-effect cells, `Interpose` is a `Free<Row, A> -> Free<Row, A>`
//! rewriting walker: it resumes the program one layer at a time, and at each
//! suspended layer either replaces the matched effect's dispatch with a
//! user-supplied program or re-embeds the unmatched layer unchanged. It is the
//! deeper primitive heftia builds scoped `Catch` on (`interposeInWith`). The
//! `run` interpreter only `uninject`s and discards the remainder; interpose must
//! rebuild it, so the no-match arm `embed`s the remainder back into the full row
//! and re-`wrap`s it. The bucket A oracle programs are single-layer, so this
//! walker handles the match and the unmatched re-embed without deep continuation
//! recursion; re-handling within continuations is the general codegen's concern.
//!
//! The two targets the oracle exercises (`Identity`, `Except`) get one concrete
//! walker each: with the brand fixed, the compiler infers the `uninject` and
//! `embed` indices, so neither needs the index/bound noise a single brand-generic
//! walker would carry (that generalisation is the macro codegen's job).

use {
	super::{
		ExceptBrand,
		ExceptF,
		IdentityBrand,
		IdentityF,
		Row,
	},
	crate::types::{
		Coyoneda,
		Free,
	},
};

/// Interpose on the `Identity` effect: rewrite each matched `Identity` dispatch
/// with `replacement(op)`, and re-embed every other effect's dispatch unchanged.
pub(crate) fn interpose_identity<A: 'static>(
	program: Free<Row, A>,
	replacement: impl FnOnce(IdentityF<'static, Free<Row, A>>) -> Free<Row, A>,
) -> Free<Row, A> {
	match program.resume() {
		Ok(value) => Free::pure(value),
		Err(layer) => {
			let selected: Result<Coyoneda<'static, IdentityBrand, Free<Row, A>>, _> =
				layer.uninject();
			match selected {
				Ok(coyo) => replacement(coyo.lower()),
				Err(remainder) => Free::wrap(remainder.embed()),
			}
		}
	}
}

/// Interpose on the `Except` effect (the tail-position target in the oracle).
pub(crate) fn interpose_except<A: 'static>(
	program: Free<Row, A>,
	replacement: impl FnOnce(ExceptF<&'static str, Free<Row, A>>) -> Free<Row, A>,
) -> Free<Row, A> {
	match program.resume() {
		Ok(value) => Free::pure(value),
		Err(layer) => {
			let selected: Result<Coyoneda<'static, ExceptBrand<&'static str>, Free<Row, A>>, _> =
				layer.uninject();
			match selected {
				Ok(coyo) => replacement(coyo.lower()),
				Err(remainder) => Free::wrap(remainder.embed()),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::IdentityF,
		crate::types::{
			Free,
			effects::fs1::{
				Fixture,
				identity_op,
				interpose::{
					interpose_except,
					interpose_identity,
				},
				run,
				run_except,
				throw_e,
			},
		},
	};

	// Behaviour-parity oracle bucket A, interpose cases T1-T4.

	// T1: single-effect row, no-op replacement (resume with the echoed value,
	// which is exactly what running the effect would do), so the program is
	// unchanged and still yields 7.
	#[test]
	fn t1_no_op_interpose_leaves_the_program() {
		let fx = Fixture::new();
		let interposed = interpose_identity(identity_op(7), |op| match op {
			IdentityF::IdentityOp(value, k) => k(value),
		});
		assert_eq!(run(interposed, &fx.handlers()), Ok(7));
	}

	// T2: single-effect row, constant replacement (substitute the matched
	// dispatch with a fresh program); demonstrates the matched arm fires.
	#[test]
	fn t2_constant_replacement_fires_at_the_match() {
		let fx = Fixture::new();
		let interposed = interpose_identity(identity_op(7), |_op| Free::pure(99));
		assert_eq!(run(interposed, &fx.handlers()), Ok(99));
	}

	// T3: the interpose target (`Identity`) never appears in the program (only a
	// throw), so every dispatch is unmatched and walks through the embed-back
	// path; the throw is preserved and `run_except` then recovers it to 42.
	#[test]
	fn t3_unmatched_target_walks_through_and_preserves_the_throw() {
		let fx = Fixture::new();
		let interposed = interpose_identity(throw_e("from_t3"), |_op| Free::pure(0));
		let result = run_except(interposed, &fx.handlers(), |_e| Free::pure(42));
		assert_eq!(result, Ok(42));
	}

	// T4: mirror of T3 with the positions swapped: interpose on `Except` while the
	// program emits only `Identity`. Every dispatch is unmatched, the Identity
	// dispatch is preserved, and the program still yields 99.
	#[test]
	fn t4_unmatched_target_walks_through_and_preserves_the_identity() {
		let fx = Fixture::new();
		let interposed = interpose_except(identity_op(99), |_op| Free::pure(0));
		assert_eq!(run(interposed, &fx.handlers()), Ok(99));
	}
}
