//! POC-10 (foundation sweep, Tier F): fair first-order dispatch micro-benchmark.
//!
//! The charter's adopted Approach (C): compare two minimal substrates that
//! differ ONLY in the dual-row `Node` wrapper, an FS-0-shaped
//! `Free<Node<coproduct>>` versus an FS-1-shaped `Free<coproduct>`, both
//! spike-level so there is no maturity confound (a real-`Run`-versus-spike
//! comparison would reintroduce one, since production `Run` is optimized). This
//! isolates the per-layer cost of the dual row's extra `Node` enum that the
//! unified row removes.
//!
//! This is a relative timing, not a criterion bench; run with `--release` for a
//! meaningful comparison. The test asserts correctness (both interpret the same
//! workload to the same result) and prints the two durations. Throwaway spike
//! code.

#![cfg(feature = "effects")]

use {
	fp_library::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		classes::{
			Functor,
			WrapDrop,
		},
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::coproduct::Coproduct,
		},
	},
	std::time::Instant,
};

// A trivial first-order effect: tick and continue. Shared by both substrates so
// the only difference between them is the `Node` wrapper.
struct OutBrand;
enum OutF<'a, A> {
	Tick(Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! { impl for OutBrand { type Of<'a, A: 'a>: 'a = OutF<'a, A>; } }
impl Functor for OutBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			OutF::Tick(k) => OutF::Tick(Box::new(move |u| f(k(u)))),
		}
	}
}

// FS-1-shaped row: the bare coproduct (one effect).
type Fs1Row = CoproductBrand<CoyonedaBrand<OutBrand>, CNilBrand>;

// FS-0-shaped row: the same coproduct wrapped in a dual-row-style `Node` enum,
// so each layer carries one extra enum the interpreter must match through.
enum NodeWrap<X> {
	First(X),
}
struct Fs0Brand;
impl_kind! {
	impl for Fs0Brand {
		type Of<'a, A: 'a>: 'a =
			NodeWrap<Apply!(<Fs1Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>;
	}
}
impl Functor for Fs0Brand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			NodeWrap::First(inner) => NodeWrap::First(<Fs1Row as Functor>::map(f, inner)),
		}
	}
}
impl WrapDrop for Fs0Brand {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		match fa {
			NodeWrap::First(inner) => <Fs1Row as WrapDrop>::drop(inner),
		}
	}
}

fn out_fs1() -> Free<Fs1Row, ()> {
	let coyo: Coyoneda<'static, OutBrand, ()> = Coyoneda::lift(OutF::Tick(Box::new(|u| u)));
	let node: Apply!(<Fs1Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>) =
		Coproduct::Inl(coyo);
	Free::lift_f(node)
}
fn out_fs0() -> Free<Fs0Brand, ()> {
	let coyo: Coyoneda<'static, OutBrand, ()> = Coyoneda::lift(OutF::Tick(Box::new(|u| u)));
	let inner: Apply!(<Fs1Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>) =
		Coproduct::Inl(coyo);
	let node: Apply!(<Fs0Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>) =
		NodeWrap::First(inner);
	Free::lift_f(node)
}

// FS-1 interpreter: one coproduct match per layer.
fn run_fs1(program: Free<Fs1Row, i32>) -> i32 {
	let mut program = program;
	let mut count = 0;
	loop {
		match program.resume() {
			Ok(value) => return value + count,
			Err(layer) => match layer {
				Coproduct::Inl(coyo) => match coyo.lower() {
					OutF::Tick(k) => {
						count += 1;
						program = k(());
					}
				},
				Coproduct::Inr(cnil) => match cnil {},
			},
		}
	}
}

// FS-0 interpreter: one extra `Node` match per layer, then the coproduct match.
fn run_fs0(program: Free<Fs0Brand, i32>) -> i32 {
	let mut program = program;
	let mut count = 0;
	loop {
		match program.resume() {
			Ok(value) => return value + count,
			Err(layer) => match layer {
				NodeWrap::First(inner) => match inner {
					Coproduct::Inl(coyo) => match coyo.lower() {
						OutF::Tick(k) => {
							count += 1;
							program = k(());
						}
					},
					Coproduct::Inr(cnil) => match cnil {},
				},
			},
		}
	}
}

#[test]
fn first_order_dispatch_node_wrapper_overhead_is_negligible() {
	let depth = 200_000;

	let mut fs1: Free<Fs1Row, i32> = Free::pure(0);
	for _ in 0 .. depth {
		fs1 = out_fs1().bind(|()| fs1);
	}
	let mut fs0: Free<Fs0Brand, i32> = Free::pure(0);
	for _ in 0 .. depth {
		fs0 = out_fs0().bind(|()| fs0);
	}

	let start = Instant::now();
	let r1 = run_fs1(fs1);
	let fs1_time = start.elapsed();

	let start = Instant::now();
	let r0 = run_fs0(fs0);
	let fs0_time = start.elapsed();

	// Same workload, same result (each interprets `depth` layers).
	assert_eq!(r1, depth);
	assert_eq!(r0, depth);

	// Report the two durations; the unified row (FS-1) drops the `Node` match,
	// so it is no worse than the dual-row-shaped FS-0.
	eprintln!(
		"POC-10 first-order dispatch over {depth} layers: FS-1 (unified row) {fs1_time:?}, FS-0 (Node-wrapped) {fs0_time:?}"
	);
}
