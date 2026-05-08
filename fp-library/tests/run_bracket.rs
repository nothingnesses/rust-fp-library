#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]
#![recursion_limit = "512"]

// Shape-only integration tests for the substrate-level scoped
// `bracket<Idx>` smart constructor across the Run-wrapper family.
// Each wrapper's section verifies:
//   T1: `bracket(acquire, body, release)` produces a program suspended
//       at a `Node::Scoped` layer carrying a `BoxBracket` / `Bracket` /
//       `BoxBracketExplicit` / `BracketExplicit` / `SendBracketExplicit`
//       cell projected via `Member::inject` at the head of the scoped
//       row.
//   T2: invoking the cell's stored `acquire` thunk (with the unit
//       argument) yields a substrate-typed program (`Free` / `RcFree` /
//       `Box<FreeExplicit>` / `RcFreeExplicit` / `ArcFreeExplicit`)
//       that, after wrapping with the wrapper's `from_*` constructor,
//       peels to the original resource value.
//   T3: invoking the cell's stored `body` closure with a sample
//       resource value yields a substrate-typed program that peels to
//       the expected `(resource, body_result)` pair.
//   T4: invoking the cell's stored `release` closure with a sample
//       resource value yields a substrate-typed program that peels to
//       `()`.
//   T5 (clone-able wrappers only: RcRun, RcRunExplicit,
//       ArcRunExplicit): cloning the suspended program produces two
//       independent peelable handles; each clone's acquire thunk
//       materialises to the original resource value.
//
// `ArcRun::bracket` is intentionally not exercised in this file: the
// rustc Send + Sync evaluation cycle on `SendBracketBrand`'s
// GAT-projection-Send-Sync bound, when combined with the marker-struct
// workaround required to define the user-facing scoped row, exceeds
// rustc's overflow limit. The smart constructor itself compiles
// cleanly; end-to-end exercise of `ArcRun::bracket` lives in the
// downstream bracket-dispatcher tests where the dispatch API changes
// the test surface. If those tests still cannot exercise it, the
// follow-up redesigns `SendBracketBrand` to drop the
// GAT-projection-Send-Sync bound on `Sub`.
//
// End-to-end resource lifecycle semantics (the bracket dispatcher
// running acquire, threading the resource into body, then running
// release with panic-safety via a Drop guard) are not exercised here.
// That path comes online when the scoped-handler dispatch protocol and
// the standard bracket dispatcher are in place; this file restricts
// itself to verifying that the substrate produces the expected
// suspended shape and that the three thunks fire.

use fp_library::{
	Apply,
	brands::{
		ArcBrand,
		BoxBracketBrand,
		BoxBracketExplicitBrand,
		BoxBrand,
		BracketBrand,
		BracketExplicitBrand,
		CNilBrand,
		CoproductBrand,
		NodeBrand,
		RcBrand,
		SendBracketExplicitBrand,
	},
	classes::{
		Functor,
		SendFunctor,
		WrapDrop,
	},
	impl_kind,
	kinds::*,
	types::effects::{
		arc_run_explicit::ArcRunExplicit,
		bracket::{
			BoxBracket,
			BoxBracketExplicit,
			Bracket,
			BracketExplicit,
			SendBracketExplicit,
		},
		coproduct::Coproduct,
		node::Node,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		run::Run,
		run_explicit::RunExplicit,
	},
};

// -- Run --

// User-facing scoped rows containing `BoxBracketBrand` cannot be
// defined as type aliases (Rust rejects the recursion). The marker
// struct breaks the type-alias cycle by hosting the recursive
// reference inside an `impl_kind!` projection plus trait impls that
// delegate to an `UnderlyingRow` type alias. Same pattern applies to
// every wrapper section below.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RunBracketRow;

type RunUnderlyingRow = CoproductBrand<
	BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, RunBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RunBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RunUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RunBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RunUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RunBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RunUnderlyingRow as Functor>::map(f, fa)
	}
}

type RunFirstRow = CNilBrand;
type RunAcquireProg = Run<RunFirstRow, RunBracketRow, i32>;
type RunBracketProg = Run<RunFirstRow, RunBracketRow, (i32, i32)>;
type RunReleaseProg = Run<RunFirstRow, RunBracketRow, ()>;

fn make_run_bracket() -> RunBracketProg {
	let acquire: RunAcquireProg = Run::pure(7);
	Run::<RunFirstRow, RunBracketRow, (i32, i32)>::bracket::<_>(
		acquire,
		|resource: Box<i32>| Run::pure((*resource, 42)),
		|_resource: Box<i32>| Run::pure(()),
	)
}

#[test]
fn run_t1_bracket_produces_scoped_layer() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxBracket::Bracket))"),
	}
}

#[test]
fn run_t2_acquire_thunk_materialises_resource_program() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: RunAcquireProg = Run::from_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn run_t3_body_materialises_paired_program() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			body, ..
		}))) => {
			let materialised: RunBracketProg = Run::from_free(body(Box::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn run_t4_release_materialises_unit_program() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			release, ..
		}))) => {
			let materialised: RunReleaseProg = Run::from_free(release(Box::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

// -- RcRun --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunBracketRow;

type RcRunUnderlyingRow = CoproductBrand<
	BracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunFirstRow = CNilBrand;
type RcRunAcquireProg = RcRun<RcRunFirstRow, RcRunBracketRow, i32>;
type RcRunBracketProg = RcRun<RcRunFirstRow, RcRunBracketRow, (i32, i32)>;
type RcRunReleaseProg = RcRun<RcRunFirstRow, RcRunBracketRow, ()>;

fn make_rc_run_bracket() -> RcRunBracketProg {
	let acquire: RcRunAcquireProg = RcRun::pure(7);
	RcRun::<RcRunFirstRow, RcRunBracketRow, (i32, i32)>::bracket::<_>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRun::pure((*resource, 42)),
		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
	)
}

#[test]
fn rc_run_t1_bracket_produces_scoped_layer() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(Bracket::Bracket))"),
	}
}

#[test]
fn rc_run_t2_acquire_thunk_materialises_resource_program() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: RcRunAcquireProg = RcRun::from_rc_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_t3_body_materialises_paired_program() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			body, ..
		}))) => {
			let materialised: RcRunBracketProg = RcRun::from_rc_free(body(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_t4_release_materialises_unit_program() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			release, ..
		}))) => {
			let materialised: RcRunReleaseProg = RcRun::from_rc_free(release(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_t5_clone_yields_two_independent_peels() {
	let prog = make_rc_run_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: RcRunBracketProg| -> RcRunAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
				acquire, ..
			}))) => RcRun::from_rc_free(acquire(())),
			_ => panic!("expected scoped bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

// -- RunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RunExplicitBracketRow;

type RunExplicitUnderlyingRow = CoproductBrand<
	BoxBracketExplicitBrand<BoxBrand, NodeBrand<CNilBrand, RunExplicitBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RunExplicitBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RunExplicitBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RunExplicitBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RunExplicitUnderlyingRow as Functor>::map(f, fa)
	}
}

type RunExplicitFirstRow = CNilBrand;
type RunExplicitAcquireProg = RunExplicit<'static, RunExplicitFirstRow, RunExplicitBracketRow, i32>;
type RunExplicitBracketProg =
	RunExplicit<'static, RunExplicitFirstRow, RunExplicitBracketRow, (i32, i32)>;
type RunExplicitReleaseProg = RunExplicit<'static, RunExplicitFirstRow, RunExplicitBracketRow, ()>;

fn make_run_explicit_bracket() -> RunExplicitBracketProg {
	let acquire: RunExplicitAcquireProg = RunExplicit::pure(7);
	RunExplicit::<'static, RunExplicitFirstRow, RunExplicitBracketRow, (i32, i32)>::bracket::<_>(
		acquire,
		|resource: Box<i32>| RunExplicit::pure((*resource, 42)),
		|_resource: Box<i32>| RunExplicit::pure(()),
	)
}

#[test]
fn run_explicit_t1_bracket_produces_scoped_layer() {
	match make_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracketExplicit::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxBracketExplicit::Bracket))"),
	}
}

#[test]
fn run_explicit_t2_acquire_thunk_materialises_resource_program() {
	match make_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracketExplicit::Bracket {
			acquire, ..
		}))) => {
			let materialised: RunExplicitAcquireProg =
				RunExplicit::from_free_explicit(*acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn run_explicit_t3_body_materialises_paired_program() {
	match make_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracketExplicit::Bracket {
			body, ..
		}))) => {
			let materialised: RunExplicitBracketProg =
				RunExplicit::from_free_explicit(*body(Box::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn run_explicit_t4_release_materialises_unit_program() {
	match make_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracketExplicit::Bracket {
			release, ..
		}))) => {
			let materialised: RunExplicitReleaseProg =
				RunExplicit::from_free_explicit(*release(Box::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

// -- RcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunExplicitBracketRow;

type RcRunExplicitUnderlyingRow = CoproductBrand<
	BracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, RcRunExplicitBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunExplicitBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunExplicitBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunExplicitBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunExplicitUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunExplicitFirstRow = CNilBrand;
type RcRunExplicitAcquireProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, i32>;
type RcRunExplicitBracketProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, (i32, i32)>;
type RcRunExplicitReleaseProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, ()>;

fn make_rc_run_explicit_bracket() -> RcRunExplicitBracketProg {
	let acquire: RcRunExplicitAcquireProg = RcRunExplicit::pure(7);
	RcRunExplicit::<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, (i32, i32)>::bracket::<_>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRunExplicit::pure((*resource, 42)),
		|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
	)
}

#[test]
fn rc_run_explicit_t1_bracket_produces_scoped_layer() {
	match make_rc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BracketExplicit::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BracketExplicit::Bracket))"),
	}
}

#[test]
fn rc_run_explicit_t2_acquire_thunk_materialises_resource_program() {
	match make_rc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BracketExplicit::Bracket {
			acquire, ..
		}))) => {
			let materialised: RcRunExplicitAcquireProg =
				RcRunExplicit::from_rc_free_explicit(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_explicit_t3_body_materialises_paired_program() {
	match make_rc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BracketExplicit::Bracket {
			body, ..
		}))) => {
			let materialised: RcRunExplicitBracketProg =
				RcRunExplicit::from_rc_free_explicit(body(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_explicit_t4_release_materialises_unit_program() {
	match make_rc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BracketExplicit::Bracket {
			release, ..
		}))) => {
			let materialised: RcRunExplicitReleaseProg =
				RcRunExplicit::from_rc_free_explicit(release(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_explicit_t5_clone_yields_two_independent_peels() {
	let prog = make_rc_run_explicit_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: RcRunExplicitBracketProg| -> RcRunExplicitAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(BracketExplicit::Bracket {
				acquire, ..
			}))) => RcRunExplicit::from_rc_free_explicit(acquire(())),
			_ => panic!("expected scoped bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

// -- ArcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcRunExplicitBracketRow;

type ArcRunExplicitUnderlyingRow = CoproductBrand<
	SendBracketExplicitBrand<ArcBrand, NodeBrand<CNilBrand, ArcRunExplicitBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for ArcRunExplicitBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<ArcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for ArcRunExplicitBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<ArcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl SendFunctor for ArcRunExplicitBracketRow {
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<ArcRunExplicitUnderlyingRow as SendFunctor>::send_map(f, fa)
	}
}

type ArcRunExplicitFirstRow = CNilBrand;
type ArcRunExplicitAcquireProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitBracketRow, i32>;
type ArcRunExplicitBracketProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitBracketRow, (i32, i32)>;
type ArcRunExplicitReleaseProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitBracketRow, ()>;

fn make_arc_run_explicit_bracket() -> ArcRunExplicitBracketProg {
	let acquire: ArcRunExplicitAcquireProg = ArcRunExplicit::pure(7);
	ArcRunExplicit::<'static, ArcRunExplicitFirstRow, ArcRunExplicitBracketRow, (i32, i32)>::bracket::<
		_,
	>(
		acquire,
		|resource: std::sync::Arc<i32>| ArcRunExplicit::pure((*resource, 42)),
		|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
	)
}

#[test]
fn arc_run_explicit_t1_bracket_produces_scoped_layer() {
	match make_arc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracketExplicit::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendBracketExplicit::Bracket))"),
	}
}

#[test]
fn arc_run_explicit_t2_acquire_thunk_materialises_resource_program() {
	match make_arc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracketExplicit::Bracket {
			acquire, ..
		}))) => {
			let materialised: ArcRunExplicitAcquireProg =
				ArcRunExplicit::from_arc_free_explicit(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn arc_run_explicit_t3_body_materialises_paired_program() {
	match make_arc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracketExplicit::Bracket {
			body, ..
		}))) => {
			let materialised: ArcRunExplicitBracketProg =
				ArcRunExplicit::from_arc_free_explicit(body(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn arc_run_explicit_t4_release_materialises_unit_program() {
	match make_arc_run_explicit_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracketExplicit::Bracket {
			release, ..
		}))) => {
			let materialised: ArcRunExplicitReleaseProg =
				ArcRunExplicit::from_arc_free_explicit(release(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn arc_run_explicit_t5_clone_yields_two_independent_peels() {
	let prog = make_arc_run_explicit_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: ArcRunExplicitBracketProg| -> ArcRunExplicitAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(SendBracketExplicit::Bracket {
				acquire, ..
			}))) => ArcRunExplicit::from_arc_free_explicit(acquire(())),
			_ => panic!("expected scoped bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}
