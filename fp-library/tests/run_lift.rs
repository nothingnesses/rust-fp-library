#![expect(
	clippy::panic,
	reason = "Tests use panic! in match arms that should be unreachable on success, for brevity and clarity."
)]

// Integration tests for `*Run::lift` across all six Run wrappers.
//
// Each test exercises one or more of:
// - Single-effect row: lift -> peel -> peel two-step round-trip
//   (lift suspends at `Node::First(Inl(Coyoneda(...)))`; lower the
//   Coyoneda's stored continuation; peel that to reach the pure
//   value).
// - Multi-effect row: second-branch injection
//   (`Node::First(Inr(Inl(...)))`) proves `Member` resolves the
//   row position correctly.
// - Inferred-`Idx`: turbofish on the wrapper omits the position
//   witness; compilation alone proves Rust resolves it.
// - Composition: `lift().bind(...)` chains like any other Run
//   program.
//
// `RcRun::peel` and `ArcRun::peel` carry `Clone` bounds on the
// row-projection that Coyoneda-headed rows do not satisfy
// (`Coyoneda`'s `Box<dyn FnOnce>` continuation is not `Clone`).
// For those two wrappers, only construction tests run; the
// Explicit-family round-trip tests verify the structural
// correctness, and the Erased-family construction tests verify
// the per-method bounds compile.

use {
	core::marker::PhantomData,
	fp_library::{
		brands::{
			ArcCoyonedaBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
			OptionBrand,
			RcCoyonedaBrand,
		},
		types::{
			Identity,
			effects::{
				arc_run::ArcRun,
				arc_run_explicit::ArcRunExplicit,
				coproduct::Coproduct,
				node::Node,
				rc_run::RcRun,
				rc_run_explicit::RcRunExplicit,
				run::Run,
				run_explicit::RunExplicit,
			},
		},
	},
};

// Row brands: each wrapper pairs with the Coyoneda variant whose
// pointer kind matches its substrate's pointer. This satisfies the
// `Of<'_, *Free<..., *TypeErasedValue>>: Clone` bound the substrate
// carries through `*Run::send` (and through `*Run::peel`'s
// row-projection Clone bound for the shared-pointer wrappers).
//
// - Run            -> bare Coyoneda (Free is single-shot; no Clone bound)
// - RcRun          -> RcCoyoneda    (RcFree's shared Rc state needs Clone)
// - ArcRun         -> ArcCoyoneda   (ArcFree's shared Arc state needs Clone+Send+Sync)
// - RunExplicit    -> bare Coyoneda (FreeExplicit has no shared state)
// - RcRunExplicit  -> RcCoyoneda    (RcFreeExplicit's shared Rc state, same as RcRun)
// - ArcRunExplicit -> ArcCoyoneda   (ArcFreeExplicit's shared Arc state, same as ArcRun)
type SingleRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcSingleRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type ArcRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type Scoped = CNilBrand;
type TwoRow = CoproductBrand<
	CoyonedaBrand<IdentityBrand>,
	CoproductBrand<CoyonedaBrand<OptionBrand>, CNilBrand>,
>;

// -- Single-effect: lift -> peel -> peel round-trip --
//
// `lift` desugars to `Free::lift_f(Node::First(...))`, which is
// `Free::wrap(F::map(|a| Free::pure(a), node))`. The suspended
// layer's stored A is the continuation `Run::pure(a)`, not the raw
// `a`. To recover the original value, lower the Coyoneda's
// continuation and peel again.

#[test]
fn run_lift_round_trip() {
	let prog: Run<SingleRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 7),
				_ => panic!("expected continuation to be pure(7)"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

#[test]
fn run_explicit_lift_round_trip() {
	let prog: RunExplicit<'_, SingleRow, Scoped, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 7),
				_ => panic!("expected continuation to be pure(7)"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

#[test]
fn rc_run_explicit_lift_round_trip() {
	let prog: RcRunExplicit<'_, RcSingleRow, Scoped, i32> =
		RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower_ref();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 7),
				_ => panic!("expected continuation to be pure(7)"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

#[test]
fn arc_run_explicit_lift_round_trip() {
	let prog: ArcRunExplicit<'_, ArcRow, Scoped, i32> =
		ArcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower_ref();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 7),
				_ => panic!("expected continuation to be pure(7)"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

// `RcRun::peel` and `ArcRun::peel` require `Clone` bounds the
// Coyoneda-headed row does not satisfy. Construction-only tests
// verify the per-method bounds on `lift` itself compile.

// `RcRun::lift` and `RcRunExplicit::lift` deviate from the plan's
// per-wrapper delta table: they use `RcCoyoneda` (not bare
// `Coyoneda`) because the substrate's per-method `Clone` bounds are
// satisfied by `RcCoyoneda` (`Rc::clone`) but not by `Coyoneda` (the
// `Box<dyn FnOnce>` continuation is not `Clone`). See deviations.md.
// With the `RcCoyoneda`-paired row, `RcRun::peel` is callable, so
// the round-trip recovers the lifted value.
#[test]
fn rc_run_lift_round_trip() {
	let prog: RcRun<RcSingleRow, Scoped, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower_ref();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 7),
				_ => panic!("expected continuation to be pure(7)"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

#[test]
fn arc_run_lift_round_trip() {
	let prog: ArcRun<ArcRow, Scoped, i32> = ArcRun::lift::<IdentityBrand, _>(Identity(7));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower_ref();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 7),
				_ => panic!("expected continuation to be pure(7)"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

// -- Multi-effect: second branch (`Member` resolves position) --

#[test]
fn run_lift_second_branch() {
	let prog: Run<TwoRow, Scoped, i32> = Run::lift::<OptionBrand, _>(Some(99));
	match prog.peel() {
		Err(Node::First(Coproduct::Inr(Coproduct::Inl(coyo)))) => match coyo.lower() {
			Some(continuation) => match continuation.peel() {
				Ok(value) => assert_eq!(value, 99),
				_ => panic!("expected continuation to be pure(99)"),
			},
			None => panic!("expected Some continuation"),
		},
		Err(_) => panic!("expected Node::First(Inr(Inl(_)))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

#[test]
fn run_explicit_lift_second_branch() {
	let prog: RunExplicit<'_, TwoRow, Scoped, i32> = RunExplicit::lift::<OptionBrand, _>(Some(99));
	match prog.peel() {
		Err(Node::First(Coproduct::Inr(Coproduct::Inl(coyo)))) => match coyo.lower() {
			Some(continuation) => match continuation.peel() {
				Ok(value) => assert_eq!(value, 99),
				_ => panic!("expected continuation to be pure(99)"),
			},
			None => panic!("expected Some continuation"),
		},
		Err(_) => panic!("expected Node::First(Inr(Inl(_)))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

// -- Inferred `Idx`: type-equality check --
//
// Both call sites use `_` for `Idx`; compilation alone proves
// Rust infers the position witness. This dedicated test surfaces
// any future regression here rather than silently in user code.

#[test]
fn run_lift_idx_inferred() {
	fn _accept<T>(_: PhantomData<T>) {}
	let _: Run<SingleRow, Scoped, i32> = Run::lift::<IdentityBrand, _>(Identity(1));
	let _: Run<TwoRow, Scoped, i32> = Run::lift::<OptionBrand, _>(Some(1));
	_accept::<()>(PhantomData);
}

// -- Composition: lift().bind(...) --
//
// Phase 3's per-effect smart constructors will compose by binding
// after `lift`. Confirm the bind chain doesn't break the row:
// peel still yields the lifted layer at position Inl, and the
// continuation runs to the bound value.

#[test]
fn run_lift_bind_composes() {
	let prog: Run<SingleRow, Scoped, i32> =
		Run::lift::<IdentityBrand, _>(Identity(1)).bind(|x| Run::pure(x + 1));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 2),
				_ => panic!("expected continuation to evaluate to 2"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}

#[test]
fn run_explicit_lift_bind_composes() {
	let prog: RunExplicit<'_, SingleRow, Scoped, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(1)).bind(|x| RunExplicit::pure(x + 1));
	match prog.peel() {
		Err(Node::First(Coproduct::Inl(coyo))) => {
			let Identity(continuation) = coyo.lower();
			match continuation.peel() {
				Ok(value) => assert_eq!(value, 2),
				_ => panic!("expected continuation to evaluate to 2"),
			}
		}
		Err(_) => panic!("expected Node::First(Inl(_))"),
		Ok(_) => panic!("expected Err (suspended at the lifted effect)"),
	}
}
