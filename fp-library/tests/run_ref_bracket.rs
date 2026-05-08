#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]
#![recursion_limit = "512"]

// Shape-only integration tests for the substrate-level scoped
// `ref_bracket<A, Idx>` smart constructor across the refcounted
// Run-wrapper family. Each wrapper's section verifies:
//   T1: `ref_bracket(acquire, body, release)` produces a program
//       suspended at a `Node::Scoped` layer carrying a `RefBracket` /
//       `SendRefBracket` / `RefBracketExplicit` /
//       `SendRefBracketExplicit` cell projected via `Member::inject`
//       at the head of the scoped row.
//   T2: invoking the cell's stored `acquire` thunk (with the unit
//       argument) yields a substrate-typed program that peels to the
//       original resource value after wrapping with the wrapper's
//       `from_*` constructor.
//   T3: invoking the cell's stored `body` closure with a sample
//       refcounted resource pointer yields a substrate-typed program
//       that peels to the expected body result.
//   T4: invoking the cell's stored `release` closure with a sample
//       refcounted resource pointer yields a substrate-typed program
//       that peels to `()`.
//   T5: cloning the suspended program produces two independent
//       peelable handles; each clone's acquire thunk materialises to
//       the original resource value.
//
// End-to-end resource lifecycle semantics are not exercised here.
// That path comes online when the scoped-handler dispatch protocol and
// the standard bracket dispatcher are in place; this file restricts
// itself to verifying that the substrate produces the expected
// suspended shape and that the three stored closures fire.

use fp_library::{
	Apply,
	brands::{
		ArcBrand,
		CNilBrand,
		CoproductBrand,
		NodeBrand,
		RcBrand,
		RefBracketBrand,
		RefBracketExplicitBrand,
		SendRefBracketExplicitBrand,
	},
	classes::{
		Functor,
		SendFunctor,
		WrapDrop,
	},
	impl_kind,
	kinds::*,
	types::{
		ArcFree,
		effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			coproduct::{
				CNil,
				Coproduct,
				Here,
			},
			member::Member,
			node::Node,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			ref_bracket::{
				RefBracket,
				RefBracketExplicit,
				SendRefBracket,
				SendRefBracketExplicit,
			},
		},
	},
};

// -- RcRun --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunRefBracketRow;

type RcRunUnderlyingRow = CoproductBrand<
	RefBracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunRefBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunRefBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunRefBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunFirstRow = CNilBrand;
type RcRunAcquireProg = RcRun<RcRunFirstRow, RcRunRefBracketRow, i32>;
type RcRunRefBracketProg = RcRun<RcRunFirstRow, RcRunRefBracketRow, i32>;
type RcRunReleaseProg = RcRun<RcRunFirstRow, RcRunRefBracketRow, ()>;

fn make_rc_run_ref_bracket() -> RcRunRefBracketProg {
	let acquire: RcRunAcquireProg = RcRun::pure(7);
	RcRun::<RcRunFirstRow, RcRunRefBracketRow, i32>::ref_bracket::<i32, _>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRun::pure(*resource + 35),
		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
	)
}

#[test]
fn rc_run_t1_ref_bracket_produces_scoped_layer() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(RefBracket::Bracket))"),
	}
}

#[test]
fn rc_run_t2_acquire_thunk_materialises_resource_program() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: RcRunAcquireProg = RcRun::from_rc_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_t3_body_materialises_body_program() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			body, ..
		}))) => {
			let materialised: RcRunRefBracketProg = RcRun::from_rc_free(body(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_t4_release_materialises_unit_program() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			release, ..
		}))) => {
			let materialised: RcRunReleaseProg = RcRun::from_rc_free(release(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_t5_clone_yields_two_independent_peels() {
	let prog = make_rc_run_ref_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: RcRunRefBracketProg| -> RcRunAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
				acquire, ..
			}))) => RcRun::from_rc_free(acquire(())),
			_ => panic!("expected scoped ref-bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

// -- ArcRun --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcRunRefBracketRow;

// `SendRefBracketBrand` in a recursive marker row triggers the same
// Send + Sync overflow shape as `ArcRun::bracket`'s `SendBracketBrand`
// test fixture. This custom scoped row still exercises
// `ArcRun::ref_bracket`, `Member::inject`, and the stored
// `SendRefBracket` cell without routing through that recursive brand
// projection.
enum ArcRunRefBracketLayer<'a, X>
where
	X: 'a, {
	Layer(
		SendRefBracket<'a, ArcBrand, NodeBrand<CNilBrand, ArcRunRefBracketRow>, i32, i32>,
		core::marker::PhantomData<fn() -> X>,
	),
}

impl<'a, X> Clone for ArcRunRefBracketLayer<'a, X>
where
	X: 'a,
{
	fn clone(&self) -> Self {
		match self {
			ArcRunRefBracketLayer::Layer(layer, marker) =>
				ArcRunRefBracketLayer::Layer(layer.clone(), *marker),
		}
	}
}

impl_kind! {
	impl for ArcRunRefBracketRow {
		type Of<'a, A: 'a>: 'a = ArcRunRefBracketLayer<'a, A>;
	}
}

impl WrapDrop for ArcRunRefBracketRow {
	fn drop<'a, X: 'a>(
		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		None
	}
}

impl SendFunctor for ArcRunRefBracketRow {
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		_f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ArcRunRefBracketLayer::Layer(layer, _) =>
				ArcRunRefBracketLayer::Layer(layer, core::marker::PhantomData),
		}
	}
}

impl
	Member<
		SendRefBracket<'static, ArcBrand, NodeBrand<CNilBrand, ArcRunRefBracketRow>, i32, i32>,
		Here,
	> for ArcRunRefBracketLayer<'static, ArcFree<NodeBrand<CNilBrand, ArcRunRefBracketRow>, i32>>
{
	type Remainder = CNil;

	fn inject(
		value: SendRefBracket<
			'static,
			ArcBrand,
			NodeBrand<CNilBrand, ArcRunRefBracketRow>,
			i32,
			i32,
		>
	) -> Self {
		ArcRunRefBracketLayer::Layer(value, core::marker::PhantomData)
	}

	fn project(
		self
	) -> Result<
		SendRefBracket<'static, ArcBrand, NodeBrand<CNilBrand, ArcRunRefBracketRow>, i32, i32>,
		Self::Remainder,
	> {
		match self {
			ArcRunRefBracketLayer::Layer(layer, _) => Ok(layer),
		}
	}
}

type ArcRunFirstRow = CNilBrand;
type ArcRunAcquireProg = ArcRun<ArcRunFirstRow, ArcRunRefBracketRow, i32>;
type ArcRunRefBracketProg = ArcRun<ArcRunFirstRow, ArcRunRefBracketRow, i32>;
type ArcRunReleaseProg = ArcRun<ArcRunFirstRow, ArcRunRefBracketRow, ()>;

fn make_arc_run_ref_bracket() -> ArcRunRefBracketProg {
	let acquire: ArcRunAcquireProg = ArcRun::pure(7);
	ArcRun::<ArcRunFirstRow, ArcRunRefBracketRow, i32>::ref_bracket::<i32, _>(
		acquire,
		|resource: std::sync::Arc<i32>| ArcRun::pure(*resource + 35),
		|_resource: std::sync::Arc<i32>| ArcRun::pure(()),
	)
}

#[test]
fn arc_run_t1_ref_bracket_produces_scoped_layer() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(ArcRunRefBracketLayer::Layer(
			SendRefBracket::Bracket {
				..
			},
			_,
		))) => {}
		_ => panic!("expected Node::Scoped(ArcRunRefBracketLayer::Layer(SendRefBracket::Bracket))"),
	}
}

#[test]
fn arc_run_t2_acquire_thunk_materialises_resource_program() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(ArcRunRefBracketLayer::Layer(
			SendRefBracket::Bracket {
				acquire, ..
			},
			_,
		))) => {
			let materialised: ArcRunAcquireProg = ArcRun::from_arc_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_t3_body_materialises_body_program() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(ArcRunRefBracketLayer::Layer(
			SendRefBracket::Bracket {
				body, ..
			},
			_,
		))) => {
			let materialised: ArcRunRefBracketProg =
				ArcRun::from_arc_free(body(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_t4_release_materialises_unit_program() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(ArcRunRefBracketLayer::Layer(
			SendRefBracket::Bracket {
				release, ..
			},
			_,
		))) => {
			let materialised: ArcRunReleaseProg =
				ArcRun::from_arc_free(release(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_t5_clone_yields_two_independent_peels() {
	let prog = make_arc_run_ref_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: ArcRunRefBracketProg| -> ArcRunAcquireProg {
		match p.peel() {
			Err(Node::Scoped(ArcRunRefBracketLayer::Layer(
				SendRefBracket::Bracket {
					acquire, ..
				},
				_,
			))) => ArcRun::from_arc_free(acquire(())),
			_ => panic!("expected scoped ref-bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

// -- RcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunExplicitRefBracketRow;

type RcRunExplicitUnderlyingRow = CoproductBrand<
	RefBracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, RcRunExplicitRefBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunExplicitRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunExplicitRefBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunExplicitRefBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunExplicitUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunExplicitFirstRow = CNilBrand;
type RcRunExplicitAcquireProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitRefBracketRow, i32>;
type RcRunExplicitRefBracketProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitRefBracketRow, i32>;
type RcRunExplicitReleaseProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitRefBracketRow, ()>;

fn make_rc_run_explicit_ref_bracket() -> RcRunExplicitRefBracketProg {
	let acquire: RcRunExplicitAcquireProg = RcRunExplicit::pure(7);
	RcRunExplicit::<'static, RcRunExplicitFirstRow, RcRunExplicitRefBracketRow, i32>::ref_bracket::<
		i32,
		_,
	>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRunExplicit::pure(*resource + 35),
		|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
	)
}

#[test]
fn rc_run_explicit_t1_ref_bracket_produces_scoped_layer() {
	match make_rc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracketExplicit::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(RefBracketExplicit::Bracket))"),
	}
}

#[test]
fn rc_run_explicit_t2_acquire_thunk_materialises_resource_program() {
	match make_rc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracketExplicit::Bracket {
			acquire, ..
		}))) => {
			let materialised: RcRunExplicitAcquireProg =
				RcRunExplicit::from_rc_free_explicit(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_explicit_t3_body_materialises_body_program() {
	match make_rc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracketExplicit::Bracket {
			body, ..
		}))) => {
			let materialised: RcRunExplicitRefBracketProg =
				RcRunExplicit::from_rc_free_explicit(body(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_explicit_t4_release_materialises_unit_program() {
	match make_rc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracketExplicit::Bracket {
			release, ..
		}))) => {
			let materialised: RcRunExplicitReleaseProg =
				RcRunExplicit::from_rc_free_explicit(release(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_explicit_t5_clone_yields_two_independent_peels() {
	let prog = make_rc_run_explicit_ref_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: RcRunExplicitRefBracketProg| -> RcRunExplicitAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(RefBracketExplicit::Bracket {
				acquire, ..
			}))) => RcRunExplicit::from_rc_free_explicit(acquire(())),
			_ => panic!("expected scoped ref-bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

// -- ArcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcRunExplicitRefBracketRow;

type ArcRunExplicitUnderlyingRow = CoproductBrand<
	SendRefBracketExplicitBrand<
		ArcBrand,
		NodeBrand<CNilBrand, ArcRunExplicitRefBracketRow>,
		i32,
		i32,
	>,
	CNilBrand,
>;

impl_kind! {
	impl for ArcRunExplicitRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<ArcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for ArcRunExplicitRefBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<ArcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl SendFunctor for ArcRunExplicitRefBracketRow {
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<ArcRunExplicitUnderlyingRow as SendFunctor>::send_map(f, fa)
	}
}

type ArcRunExplicitFirstRow = CNilBrand;
type ArcRunExplicitAcquireProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitRefBracketRow, i32>;
type ArcRunExplicitRefBracketProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitRefBracketRow, i32>;
type ArcRunExplicitReleaseProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitRefBracketRow, ()>;

fn make_arc_run_explicit_ref_bracket() -> ArcRunExplicitRefBracketProg {
	let acquire: ArcRunExplicitAcquireProg = ArcRunExplicit::pure(7);
	ArcRunExplicit::<'static, ArcRunExplicitFirstRow, ArcRunExplicitRefBracketRow, i32>::ref_bracket::<
		i32,
		_,
	>(
		acquire,
		|resource: std::sync::Arc<i32>| ArcRunExplicit::pure(*resource + 35),
		|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
	)
}

#[test]
fn arc_run_explicit_t1_ref_bracket_produces_scoped_layer() {
	match make_arc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracketExplicit::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendRefBracketExplicit::Bracket))"),
	}
}

#[test]
fn arc_run_explicit_t2_acquire_thunk_materialises_resource_program() {
	match make_arc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracketExplicit::Bracket {
			acquire, ..
		}))) => {
			let materialised: ArcRunExplicitAcquireProg =
				ArcRunExplicit::from_arc_free_explicit(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_explicit_t3_body_materialises_body_program() {
	match make_arc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracketExplicit::Bracket {
			body, ..
		}))) => {
			let materialised: ArcRunExplicitRefBracketProg =
				ArcRunExplicit::from_arc_free_explicit(body(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_explicit_t4_release_materialises_unit_program() {
	match make_arc_run_explicit_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracketExplicit::Bracket {
			release, ..
		}))) => {
			let materialised: ArcRunExplicitReleaseProg =
				ArcRunExplicit::from_arc_free_explicit(release(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_explicit_t5_clone_yields_two_independent_peels() {
	let prog = make_arc_run_explicit_ref_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: ArcRunExplicitRefBracketProg| -> ArcRunExplicitAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(SendRefBracketExplicit::Bracket {
				acquire, ..
			}))) => ArcRunExplicit::from_arc_free_explicit(acquire(())),
			_ => panic!("expected scoped ref-bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}
