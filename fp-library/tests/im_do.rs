// Integration tests for `im_do!` (Inherent Monadic do-notation).
//
// Covers:
// - By-value mode on all six Run wrappers (Run, RcRun, ArcRun,
//   RunExplicit, RcRunExplicit, ArcRunExplicit).
// - `ref` mode on the four Clone-able wrappers (RcRun, ArcRun,
//   RcRunExplicit, ArcRunExplicit).
// - `pure(x)` rewriting (val and ref forms).
// - Statement variety: typed binds, discard binds, sequence
//   statements, let bindings.
// - One inferred-mode invocation (no wrapper before the brace).

use {
	fp_library::{
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
		},
		types::effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			run::Run,
			run_explicit::RunExplicit,
		},
	},
	fp_macros::im_do,
};

// `Run` over `Free` requires a Coyoneda-headed row to avoid the
// layout cycle for identity-shaped functors (see plan.md gotchas).
type FirstRowCoyo = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
// `RcRun`/`ArcRun`/the Explicit family escape the cycle via outer
// pointer wrapping or `Box`-in-Wrap; they accept identity-headed rows.
type FirstRowId = CoproductBrand<IdentityBrand, CNilBrand>;
type Scoped = CNilBrand;

// -- By-value mode --

#[test]
fn run_by_value() {
	let result: Run<FirstRowCoyo, Scoped, i32> = im_do!(Run {
		x <- Run::pure(2);
		y <- Run::pure(x + 1);
		pure(x * y)
	});
	assert!(matches!(result.peel(), Ok(6)));
}

#[test]
fn rc_run_by_value() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!(RcRun {
		x <- RcRun::pure(2);
		y <- RcRun::pure(x + 1);
		pure(x * y)
	});
	assert!(matches!(result.peel(), Ok(6)));
}

#[test]
fn arc_run_by_value() {
	let result: ArcRun<FirstRowId, Scoped, i32> = im_do!(ArcRun {
		x <- ArcRun::pure(2);
		y <- ArcRun::pure(x + 1);
		pure(x * y)
	});
	assert!(matches!(result.peel(), Ok(6)));
}

#[test]
fn run_explicit_by_value() {
	let result: RunExplicit<'_, FirstRowId, Scoped, i32> = im_do!(RunExplicit {
		x <- RunExplicit::pure(2);
		y <- RunExplicit::pure(x + 1);
		pure(x * y)
	});
	assert!(matches!(result.peel(), Ok(6)));
}

#[test]
fn rc_run_explicit_by_value() {
	let result: RcRunExplicit<'_, FirstRowId, Scoped, i32> = im_do!(RcRunExplicit {
		x <- RcRunExplicit::pure(2);
		y <- RcRunExplicit::pure(x + 1);
		pure(x * y)
	});
	assert!(matches!(result.peel(), Ok(6)));
}

#[test]
fn arc_run_explicit_by_value() {
	let result: ArcRunExplicit<'_, FirstRowId, Scoped, i32> = im_do!(ArcRunExplicit {
		x <- ArcRunExplicit::pure(2);
		y <- ArcRunExplicit::pure(x + 1);
		pure(x * y)
	});
	assert!(matches!(result.peel(), Ok(6)));
}

// -- ref mode (only on Clone-able wrappers) --

#[test]
fn rc_run_ref_mode() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!(ref RcRun {
		x: &i32 <- RcRun::pure(2);
		pure(*x + 1)
	});
	assert!(matches!(result.peel(), Ok(3)));
}

#[test]
fn arc_run_ref_mode() {
	let result: ArcRun<FirstRowId, Scoped, i32> = im_do!(ref ArcRun {
		x: &i32 <- ArcRun::pure(2);
		pure(*x + 1)
	});
	assert!(matches!(result.peel(), Ok(3)));
}

#[test]
fn rc_run_explicit_ref_mode() {
	let result: RcRunExplicit<'_, FirstRowId, Scoped, i32> = im_do!(ref RcRunExplicit {
		x: &i32 <- RcRunExplicit::pure(2);
		pure(*x + 1)
	});
	assert!(matches!(result.peel(), Ok(3)));
}

#[test]
fn arc_run_explicit_ref_mode() {
	let result: ArcRunExplicit<'_, FirstRowId, Scoped, i32> = im_do!(ref ArcRunExplicit {
		x: &i32 <- ArcRunExplicit::pure(2);
		pure(*x + 1)
	});
	assert!(matches!(result.peel(), Ok(3)));
}

// -- Statement variety --

#[test]
fn let_binding() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!(RcRun {
		x <- RcRun::pure(2);
		let z = x * 5;
		pure(z)
	});
	assert!(matches!(result.peel(), Ok(10)));
}

#[test]
fn typed_let_binding() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!(RcRun {
		x <- RcRun::pure(2);
		let z: i32 = x * 5;
		pure(z)
	});
	assert!(matches!(result.peel(), Ok(10)));
}

#[test]
fn discard_bind() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!(RcRun {
		_ <- RcRun::pure(());
		pure(42)
	});
	assert!(matches!(result.peel(), Ok(42)));
}

#[test]
fn sequence_statement() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!(RcRun {
		RcRun::pure(());
		pure(42)
	});
	assert!(matches!(result.peel(), Ok(42)));
}

#[test]
fn typed_bind() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!(RcRun {
		x: i32 <- RcRun::pure(2);
		pure(x + 1)
	});
	assert!(matches!(result.peel(), Ok(3)));
}

// -- Inferred mode (no wrapper) --
//
// In inferred mode, bare `pure(x)` cannot be used (no wrapper to
// qualify the call); the user writes the concrete constructor
// instead. Method dispatch on `bind` works via the receiver type.

#[test]
fn inferred_mode() {
	let result: RcRun<FirstRowId, Scoped, i32> = im_do!({
		x <- RcRun::pure(2);
		y <- RcRun::pure(x + 1);
		RcRun::pure(x * y)
	});
	assert!(matches!(result.peel(), Ok(6)));
}
