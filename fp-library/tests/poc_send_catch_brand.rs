#![cfg(feature = "effects")]
// POC: parallel SendCatchBrand pattern for the scoped Catch effect.
//
// Question being answered: does the parallel-Send-brand workaround
// (StateBrand vs SendStateBrand, where the Send variant
// bakes Send + Sync into the trait-object bound at definition time)
// carry over to a scoped-effect constructor with a Box<dyn FnOnce>
// closure cell?
//
// Hypothesis: yes. The friction's structural cause is the same
// (`dyn Fn(...)` and `dyn FnOnce(...)` are not Send + Sync without
// explicit + Send + Sync in the dyn bound), and the workaround is the
// same (define a sibling type that bakes the marker traits in).
//
// Validation steps in this file:
//   1. Define Catch<'a, E, A> with `Box<dyn 'a + FnOnce(E) -> A>`.
//   2. Define SendCatch<'a, E, A> with `Box<dyn 'a + FnOnce(E) -> A + Send + Sync>`.
//   3. Register both as Kind brands via `impl_kind!`.
//   4. Implement Functor for CatchBrand<E>.
//   5. Implement SendFunctor for SendCatchBrand<E>; this is the call
//      site where the same Send-aware closure-storage friction
//      surfaced for State.
//   6. Static-assert SendCatch<'a, E, A>: Send + Sync whenever the
//      type parameters are Send + Sync.
//   7. Run a positive test exercising `map` and `send_map` to confirm
//      both compose user functions onto the stored handler closure.
//
// If this compiles and the static assertion holds, the parallel-brand
// pattern works for Catch the same way it does for State.
#![allow(dead_code)]
#![expect(
	clippy::expect_used,
	reason = "POC tests use panicking operations for brevity and clarity."
)]

use {
	core::marker::PhantomData,
	fp_library::{
		Apply,
		classes::{
			Functor,
			SendFunctor,
		},
		impl_kind,
		kinds::*,
	},
};

// -- Brand structs ---------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub struct CatchBrand<E>(PhantomData<E>);

#[derive(Clone, Copy, Debug, Default)]
pub struct SendCatchBrand<E>(PhantomData<E>);

// -- Catch (RcRun-family flavour: Box<dyn FnOnce>, no Send + Sync) ---

pub enum Catch<'a, E, A>
where
	E: 'a,
	A: 'a, {
	Catch { action: A, handler: Box<dyn 'a + FnOnce(E) -> A> },
}

impl_kind! {
	impl<E: 'static> for CatchBrand<E> {
		type Of<'a, A: 'a>: 'a = Catch<'a, E, A>;
	}
}

impl<E> Functor for CatchBrand<E>
where
	E: 'static,
{
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			Catch::Catch {
				action,
				handler,
			} => Catch::Catch {
				action: f(action),
				handler: Box::new(move |e: E| {
					let recovered: A = handler(e);
					f(recovered)
				}),
			},
		}
	}
}

// -- SendCatch (ArcRun-family flavour: + Send + Sync baked in) -------
//
// This is the structural test. If `SendCatch<'a, E, A>` is verifiably
// Send + Sync (under E, A: Send + Sync), and SendFunctor::send_map
// type-checks, then the parallel-Send-brand pattern works for Catch.

pub enum SendCatch<'a, E, A>
where
	E: 'a,
	A: 'a, {
	Catch { action: A, handler: Box<dyn 'a + FnOnce(E) -> A + Send + Sync> },
}

impl_kind! {
	impl<E: 'static> for SendCatchBrand<E> {
		type Of<'a, A: 'a>: 'a = SendCatch<'a, E, A>;
	}
}

impl<E> SendFunctor for SendCatchBrand<E>
where
	E: Send + Sync + 'static,
{
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			SendCatch::Catch {
				action,
				handler,
			} => SendCatch::Catch {
				action: f(action),
				handler: Box::new(move |e: E| {
					let recovered: A = handler(e);
					f(recovered)
				}),
			},
		}
	}
}

// -- Static assertions ----------------------------------------------
//
// The load-bearing claim: SendCatch<'a, E, A> is Send + Sync when its
// type parameters are. If this fails to compile, the parallel-Send-
// brand pattern does not carry to scoped effects with Box<dyn FnOnce>
// handlers and F2A's recommended remediation needs a different shape.

fn assert_send_sync<T: Send + Sync>() {}

fn _static_assertions() {
	// SendCatch<E, A>: Send + Sync when E: Send + Sync, A: Send + Sync.
	assert_send_sync::<SendCatch<'static, String, i32>>();
	assert_send_sync::<SendCatch<'static, i32, String>>();

	// Counterpoint: Catch<E, A> is NOT Send + Sync (the FnOnce trait
	// object lacks the marker bounds). We do not assert this at the
	// type level here, but we deliberately do not include it in the
	// `assert_send_sync` calls above.
}

// -- Positive functional tests --------------------------------------

#[test]
fn catch_functor_map_composes_handler() {
	// Build a Catch<E, A> with action = 7, handler returning 100.
	let catch: Catch<'static, &'static str, i32> = Catch::Catch {
		action: 7,
		handler: Box::new(|_e: &'static str| 100),
	};
	// Functor::map adds 1 to A.
	let mapped: Catch<'static, &'static str, i32> =
		<CatchBrand<&'static str> as Functor>::map(|x: i32| x + 1, catch);
	match mapped {
		Catch::Catch {
			action,
			handler,
		} => {
			assert_eq!(action, 8); // 7 mapped through (+1) is 8.
			assert_eq!(handler("oops"), 101); // 100 mapped through (+1).
		}
	}
}

#[test]
fn send_catch_functor_map_composes_handler() {
	// Build a SendCatch<E, A> with action = 7, handler returning 100.
	let catch: SendCatch<'static, &'static str, i32> = SendCatch::Catch {
		action: 7,
		handler: Box::new(|_e: &'static str| 100),
	};
	// SendFunctor::send_map composes (+1).
	let mapped: SendCatch<'static, &'static str, i32> =
		<SendCatchBrand<&'static str> as SendFunctor>::send_map(|x: i32| x + 1, catch);
	match mapped {
		SendCatch::Catch {
			action,
			handler,
		} => {
			assert_eq!(action, 8);
			assert_eq!(handler("oops"), 101);
		}
	}
}

#[test]
fn send_catch_is_actually_sendable_across_threads() {
	// Operational confirmation: a SendCatch can be moved into a
	// spawned thread.
	let catch: SendCatch<'static, &'static str, i32> = SendCatch::Catch {
		action: 42,
		handler: Box::new(|_e: &'static str| 0),
	};
	let handle = std::thread::spawn(move || match catch {
		SendCatch::Catch {
			action,
			handler,
		} => action + handler("ignored"),
	});
	let result = handle.join().expect("thread panicked");
	assert_eq!(result, 42);
}
