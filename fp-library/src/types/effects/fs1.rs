//! FS-1 effects rebuild, crate-internal work in progress.
//!
//! This module is the in-tree vertical slice of the unified-row effects
//! rebuild. It is deliberately `pub(crate)` and not part of the public
//! surface: per the adopted hybrid method, the slice was built to a
//! compiling, test-backed state before the dual-row subsystem it replaces
//! was deleted, so a half-built rewrite never shipped. The public
//! effect-definition macros (`define_effect!`, `define_row!`) are built on
//! top of it; the generic public runner surface comes next.
//!
//! FS-1 replaces the earlier dual-row design with one unified row of effect
//! brands and elaborates higher-order effects into first-order ones over that
//! row, rather than using boundary frames. This slice carries ten first-order
//! effects (`State`, `Throw`, `Reader`, `Writer`, `Fresh`, `Input`, `KVStore`,
//! `Empty`, `Except`, `Identity`) and five higher-order effects (`Catch`,
//! `Censor`, `Local`, `Listen`, `Bracket`) as in-row cells in one `Coyoneda`-wrapped
//! `CoproductBrand` row, interpreted by one pass that elaborates the
//! higher-order cells. It reproduces the behaviour-parity oracle's bucket-A
//! cases: State-with-Catch ordering (a write before a caught throw survives),
//! Writer post-censor (`"Hello world!!"`), a Reader + State + Catch composition,
//! the `Fresh` monotonic counter, the `Input` queue drain, the `KVStore`
//! lookup/update sequence, the `Empty` short-circuit, the `Local` scoped
//! environment, the `Listen` log observation, the `Bracket` acquire/use/release
//! ordering, and the typed `Except` throw recovered to a sentinel.
//!
//! Per-effect module layout: each effect lives in its own submodule
//! (`fs1/<effect>.rs`) exposing the effect definition (brand, functor, order
//! marker), its smart constructor(s), and its bucket-A parity test, so the
//! per-effect work is self-contained and collision-free. This parent module
//! holds only the shared surface that every effect threads through: the unified
//! `Row`, the order-classification machinery, the `Handlers` bundle, and the
//! `run` interpreter. Adding an effect touches only append-only points here, each
//! marked with a `FAN-OUT ANCHOR` comment: one `Row` cell, one interpreter
//! dispatch arm, and one `mod` declaration, with stateful effects also adding a
//! `Handlers` field and a `Fixture` default; the smart constructors inject by
//! type (`Coproduct::inject`), so none names its row position.
//!
//! Higher-order semantics fall out of how the interpreter shares or scopes its
//! accumulators at the recursive call: `Catch` shares the `State` cell (so the
//! write survives), while `Censor` gives its action a fresh local log (so the
//! censor scopes the accumulation), with no boundary frames.
//!
//! Scope of this slice: the substrate is the existing public `Free` (the
//! `Store = Box`, erased, `'static` form); the `Store`-parameterised Rc/Arc
//! forms and the concrete (non-`'static`) form are folded in later. The interpreter dispatches each
//! active row arm by its effect brand (type-directed selection over the
//! coproduct), not by the arm's position in the row, so the dispatch arms may
//! be written in any order and need not track the row's declared order; this is
//! the brand-keyed dispatch that removes the positional-sort footgun.
//! Per-brand order markers and the order-directed peel classify
//! whether an active arm is first-order or higher-order; they are exercised by
//! the order-classification test and become the routing layer when elaboration
//! is generalised over the row.
//!
//! Documentation status: this module and its effect submodules intentionally do
//! NOT yet use the `#[fp_macros::document_module]` wrapper that the rest of
//! `fp-library/src/` uses. Every effect here is a `define_effect!` invocation,
//! but the shared surface around them (this module's row, handler bundle, and
//! interpreter) is still crate-internal and pinned to the single-shot Box
//! store, so the runnable per-item doctests `document_module` requires cannot
//! yet be written against a settled surface. The wrapper and full per-item
//! documentation are applied once the substrate settles; until then this is a
//! tracked, temporary exception, not an oversight.

use {
	crate::{
		Apply,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::{
				coproduct::{
					CNil,
					Coproduct,
				},
				order::{
					FirstOrder,
					HigherOrder,
					OrderOf,
				},
			},
		},
	},
	std::{
		cell::{
			Cell,
			RefCell,
		},
		collections::{
			BTreeMap,
			VecDeque,
		},
	},
};

mod catch;
mod censor;
mod fresh;
mod reader;
mod throw;
// FAN-OUT ANCHOR (effect module): a ported effect appends its `mod <effect>;` here.
mod bracket;
mod empty;
mod except;
mod identity;
mod input;
mod interpose;
mod kv_store;
mod listen;
mod local;

// The smart constructors are re-exported flat (`fs1::get`, ...) so an effect's
// parity test names a sibling effect's constructor (and its own) by the flat
// path, without reaching into each effect submodule. The rule: the flat block
// covers smart constructors only; program transformers (the interpose walkers)
// are imported via their module path.
pub(crate) use self::{
	bracket::bracket,
	catch::catch,
	censor::censor,
	empty::empty,
	except::throw as throw_e,
	fresh::fresh,
	identity::identity_op,
	input::input,
	kv_store::{
		lookup,
		update,
	},
	listen::listen,
	local::{
		local,
		ref_local,
	},
	reader::ask,
	throw::throw,
};
// The effect brands and functor payloads the shared `Row` and interpreter name.
use self::{
	bracket::{
		BracketBrand,
		BracketF,
	},
	catch::{
		CatchBrand,
		CatchF,
	},
	censor::{
		CensorBrand,
		CensorF,
	},
	empty::EmptyBrand,
	except::{
		ExceptBrand,
		ExceptF,
	},
	fresh::{
		FreshBrand,
		FreshF,
	},
	identity::{
		IdentityBrand,
		IdentityF,
	},
	input::{
		InputBrand,
		InputF,
	},
	kv_store::{
		KVStoreBrand,
		KVStoreF,
	},
	listen::{
		ListenBrand,
		ListenF,
	},
	local::{
		LocalBrand,
		LocalF,
	},
	reader::{
		ReaderBrand,
		ReaderF,
	},
	throw::ThrowBrand,
};
use crate::types::effects::{
	state::{
		StateBrand,
		StateF,
	},
	writer::{
		WriterBrand,
		WriterF,
	},
};
// `State` and `Writer` are promoted to the public catalog; the slice consumes
// the public definitions (pinned by `StatePinned`/`WriterPinned` below) and
// keeps the flat constructor re-exports for its parity tests.
pub(crate) use crate::types::effects::{
	state::{
		get,
		put,
	},
	writer::tell,
};

// -- The order-directed peel over the public order markers --
//
// The order markers themselves (`OrderOf`, `FirstOrder`, `HigherOrder`) live
// in the public `types::effects::order` module; each effect submodule
// implements `OrderOf` for its brand. What stays here is the interpreter-side
// peel machinery that reads them.

/// The runtime reflection of an order marker, so an interpreter can branch on
/// the active arm's order (the order-directed peel).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OrderTag {
	First,
	Higher,
}
trait OrderTagged {
	fn tag() -> OrderTag;
}
impl OrderTagged for FirstOrder {
	fn tag() -> OrderTag {
		OrderTag::First
	}
}
impl OrderTagged for HigherOrder {
	fn tag() -> OrderTag {
		OrderTag::Higher
	}
}

/// A row cell exposes the order of its effect. For a `Coyoneda`-wrapped cell the
/// order is the wrapped brand's [`OrderOf::Order`].
trait CellOrder {
	type Order;
}
// Naming `Coyoneda<'a, E, _>` requires its brand `E` to satisfy the
// `type Of<'a, T: 'a>: 'a` kind trait, spelled here by its stable
// `kinds` alias.
impl<'a, E: OrderOf + LifetimeUnaryKind, A> CellOrder for Coyoneda<'a, E, A> {
	type Order = E::Order;
}

/// Classify the active arm of a suspended row layer by order, walking the
/// coproduct to the live cell. This is the order-directed peel (POC-3): one row
/// holds both kinds, and the interpreter reads the order off the active cell.
trait ClassifyActive {
	fn classify(&self) -> OrderTag;
}
impl ClassifyActive for CNil {
	fn classify(&self) -> OrderTag {
		match *self {}
	}
}
impl<Cell: CellOrder, Rest: ClassifyActive> ClassifyActive for Coproduct<Cell, Rest>
where
	Cell::Order: OrderTagged,
{
	fn classify(&self) -> OrderTag {
		match self {
			Coproduct::Inl(_) => <Cell::Order as OrderTagged>::tag(),
			Coproduct::Inr(rest) => rest.classify(),
		}
	}
}

// -- The unified row --

/// The unified effect row for this slice: one `Coyoneda`-wrapped cell per
/// effect, terminated by `CNilBrand`. All effects, first-order and higher-order
/// alike, live in this single row (the defining FS-1 property; the dual scoped
/// row is gone). New effects tail-append a cell here; type-directed injection
/// keeps every existing constructor unchanged.
/// The slice's pinned instantiations of the parameterised effect brands: the
/// brands are generic (state, environment, log, resource, result, and row
/// types), and the row is where a slice commits to concrete types. These
/// aliases are that commitment, stated once and reused by the row and the
/// dispatch arms.
pub(crate) type StatePinned = StateBrand<bool>;
pub(crate) type WriterPinned = WriterBrand<String>;
pub(crate) type CatchPinned = CatchBrand<Row, ()>;
pub(crate) type LocalPinned = LocalBrand<Row, i32, i32>;
pub(crate) type ListenPinned = ListenBrand<Row, i32, String>;
pub(crate) type BracketPinned = BracketBrand<Row, i32, i32>;
pub(crate) type CensorPinned = CensorBrand<Row, String, ()>;

fp_macros::define_row! {
	/// The unified effect row for this slice: one `Coyoneda`-wrapped cell per
	/// effect. All effects, first-order and higher-order alike, live in this
	/// single row (the defining FS-1 property; the dual scoped row is gone).
	/// New effects tail-append a member; type-directed injection keeps every
	/// existing constructor unchanged. The row is a nominal brand rather than
	/// a type alias because `CatchPinned`'s cell stores `Free<Row, _>`
	/// sub-programs: the row names itself, which is a definition cycle for an
	/// alias but lazy and legal through the nominal brand's kind projection.
	#[crate_path(crate)]
	pub(crate) row Row {
		StatePinned,
		ThrowBrand,
		CatchPinned,
		ReaderBrand,
		WriterPinned,
		CensorPinned,
		FreshBrand,
		InputBrand,
		KVStoreBrand,
		EmptyBrand,
		LocalPinned,
		ListenPinned,
		BracketPinned,
		ExceptBrand<&'static str>,
		IdentityBrand,
		// FAN-OUT ANCHOR (Row tail): a ported effect appends its member brand here.
	}
}

/// The row cell over a result `A`, as handed to [`Free::lift_f`] by the smart
/// constructors: one `Coyoneda`-wrapped operation at its coproduct position
/// whose hole is the operation's result `A`. (The interpreter's [`Free::resume`]
/// hands back the same row shape but with the hole instantiated to the
/// continuation `Free<Row, A>`; that shape is inferred in `run`, not named.)
type Node<A> = Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

// -- The interpreter: one pass, elaborating the higher-order cells --

/// The interpreter's handler state, bundled into one value so [`run`] takes a
/// single handler argument rather than one positional parameter per effect.
/// Each field is the state a first-order effect's dispatch arm reads or writes:
/// `state` the shared `State` cell, `env` the `Reader` environment, `log` the
/// `Writer` accumulator, `fresh` the shared `Fresh` counter and its successor
/// function, `input` the `Input` queue, and `kv_store` the `KVStore` map. The
/// higher-order effects reuse the same `Handlers`: `Catch` passes it through
/// unchanged (so a write before a caught throw survives), while `Censor`
/// derives a variant whose `log` is a fresh local cell
/// (`Handlers { log: &local, ..*handlers }`). The struct is `Copy` over its
/// reference fields, so that derivation needs no clone and the pass-through
/// calls reuse the borrow. Adding an effect appends one field here, which has
/// no argument-count lint ceiling (unlike growing `run`'s parameter list).
#[derive(Clone, Copy)]
pub(crate) struct Handlers<'h> {
	/// The shared `State` cell (read by `Get`, written by `Put`).
	state: &'h Cell<bool>,
	/// The `Reader` environment (read by `Ask`).
	env: i32,
	/// The `Writer` log accumulator (appended by `Tell`).
	log: &'h RefCell<String>,
	/// The shared `Fresh` monotonic counter (read and advanced by `Fresh`).
	fresh: &'h Cell<usize>,
	/// The `Fresh` successor policy: the default advances the counter by one; a
	/// custom runner supplies another (for example `|c| c + 2`).
	fresh_succ: fn(usize) -> usize,
	/// The shared `Input` queue (drained by `Input`, `None` once empty).
	input: &'h RefCell<VecDeque<&'static str>>,
	/// The shared `KVStore` map (read by `Lookup`, written by `Update`).
	kv_store: &'h RefCell<BTreeMap<&'static str, i32>>,
	// FAN-OUT ANCHOR (Handlers field): a stateful effect appends its `&'h`
	// reference field here, with a matching `Fixture` field and default below.
}

/// The interpreter's abort channel: each aborting effect is a distinct case,
/// so boundaries can be selective. `Throw` is the bare abort the `Catch`
/// elaboration recovers; `Empty` is the nondeterministic dead branch, which
/// nothing in the single-shot slice recovers; `Except` carries a typed error
/// for [`run_except`]. Distinct cases keep `Catch` from becoming a catch-all
/// over failure kinds it was never meant to handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Abort {
	/// A bare `Throw`, recoverable by the `Catch` elaboration.
	Throw,
	/// An `Empty` dead branch; propagates through `Catch`.
	Empty,
	/// A typed `Except` throw, recoverable by [`run_except`]; propagates
	/// through `Catch` with its payload intact.
	Except(&'static str),
}

/// Interpret a program over the unified row, given the [`Handlers`] bundle of
/// effect state. Aborts surface as [`Abort`]: a bare `Throw` as
/// `Err(Abort::Throw)`, an `Empty` dead branch as `Err(Abort::Empty)`, and a
/// typed `Except` throw as `Err(Abort::Except(e))`, carrying its error in the
/// return channel so [`run_except`] can recover it. `Catch` is elaborated by
/// interpreting its action under the same `Handlers` (so writes survive a
/// caught throw) and recovers `Abort::Throw` only; other aborts propagate
/// through it. `Censor` is elaborated by interpreting its action under a
/// `Handlers` whose `log` is a fresh local cell, then emitting `f(total)` to the
/// outer log. No boundary frames.
pub(crate) fn run<A: 'static>(
	program: Free<Row, A>,
	handlers: &Handlers,
) -> Result<A, Abort> {
	let mut program = program;
	loop {
		let layer = match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => layer,
		};
		// Brand-keyed dispatch: select the active arm by its effect brand via
		// type-directed `uninject`, independent of the brand's position in the
		// row. The arms below are deliberately not in the row's declared order
		// (Reader and Writer are handled before Catch even though they sit after
		// it in `Row`), which is exactly the property that removes the
		// positional-sort footgun: dispatch correctness no longer depends on the
		// arm order matching the row order. Each `uninject` peels its brand's cell
		// out wherever it sits, yielding the active cell or the remaining row; the
		// chain bottoms out at the uninhabited terminal row.
		let selected: Result<Coyoneda<'static, StatePinned, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				program = match coyo.lower() {
					StateF::Get(k) => k(handlers.state.get()),
					StateF::Put(s, k) => {
						handlers.state.set(s);
						k(())
					}
				};
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, ThrowBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(_throw) => return Err(Abort::Throw),
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, ReaderBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				program = match coyo.lower() {
					ReaderF::Ask(k) => k(handlers.env),
				};
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, WriterPinned, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				match coyo.lower() {
					WriterF::Tell(w, k) => {
						handlers.log.borrow_mut().push_str(&w);
						program = k(());
					}
				}
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, FreshBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				match coyo.lower() {
					FreshF::Fresh(k) => {
						let n = handlers.fresh.get();
						handlers.fresh.set((handlers.fresh_succ)(n));
						program = k(n);
					}
				}
				continue;
			}
			Err(rest) => rest,
		};
		// FAN-OUT ANCHOR (dispatch arm): a ported effect appends its brand-keyed
		// `uninject` arm here; the chain is position-independent, so order is free.
		let selected: Result<Coyoneda<'static, InputBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				match coyo.lower() {
					InputF::Input(k) => {
						let v = handlers.input.borrow_mut().pop_front();
						program = k(v);
					}
				}
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, KVStoreBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				match coyo.lower() {
					KVStoreF::Lookup(key, k) => {
						let value = handlers.kv_store.borrow().get(key).copied();
						program = k(value);
					}
					KVStoreF::Update(key, value, k) => {
						match value {
							Some(v) => {
								handlers.kv_store.borrow_mut().insert(key, v);
							}
							None => {
								handlers.kv_store.borrow_mut().remove(key);
							}
						}
						program = k(());
					}
				}
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, EmptyBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(_empty) => return Err(Abort::Empty),
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, ExceptBrand<&'static str>, Free<Row, A>>, _> =
			layer.uninject();
		let layer = match selected {
			Ok(coyo) => match coyo.lower() {
				ExceptF::Throw(e, _) => return Err(Abort::Except(e)),
			},
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, IdentityBrand, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				let IdentityF::IdentityOp(value, k) = coyo.lower();
				program = k(value);
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, CatchPinned, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				let CatchF::Catch {
					action,
					recover,
					k,
				} = coyo.lower();
				// Recover `Abort::Throw` only: an `Empty` dead branch and a typed
				// `Except` throw are different effects with their own boundaries,
				// so they propagate through the catch with their payloads intact.
				// The action's value (or the recovery's) threads to the
				// continuation.
				#[expect(
					clippy::unit_arg,
					reason = "The continuation input is the action's (or recovery's) value; this slice pins the catch result type to unit, but the value-threading shape is the general elaboration."
				)]
				{
					program = k(match run(action, handlers) {
						Ok(value) => value,
						Err(Abort::Throw) => run(recover(), handlers)?,
						Err(other) => return Err(other),
					});
				}
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, LocalPinned, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				let LocalF::Local {
					modify,
					action,
					k,
				} = coyo.lower();
				// Run the action under a `Handlers` whose `env` is the inherited
				// environment transformed by `modify`, so every `Reader` ask inside
				// the action reads `modify(env)`; the action's result then flows to `k`.
				let scoped = Handlers {
					env: modify(handlers.env),
					..*handlers
				};
				let v = run(action, &scoped)?;
				program = k(v);
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, ListenPinned, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				let ListenF::Listen {
					action,
					k,
				} = coyo.lower();
				// Run the action under the SAME `Handlers` so its writes are preserved
				// into the outer `log`; the slice of `log` the action appended is the
				// observed output, paired with the action's value.
				let start = handlers.log.borrow().len();
				let value = run(action, handlers)?;
				let observed = handlers.log.borrow()[start ..].to_string();
				program = k((value, observed));
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, BracketPinned, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				let BracketF::Bracket {
					acquire,
					body,
					release,
					k,
				} = coyo.lower();
				// Acquire, use, then release the resource in order, all under the same
				// `Handlers`; the resource threads through as a plain value. An
				// acquire abort skips both body and release (no resource exists yet).
				// A body abort still runs `release` and then propagates, the body's
				// abort taking priority: an abort raised by `release` during that
				// unwind is discarded, while on a successful body a `release` abort
				// propagates normally.
				let resource = run(acquire, handlers)?;
				match run(body(resource), handlers) {
					Ok(result) => {
						run(release(resource), handlers)?;
						program = k(result);
						continue;
					}
					Err(abort) => {
						let _ = run(release(resource), handlers);
						return Err(abort);
					}
				}
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, CensorPinned, Free<Row, A>>, _> = layer.uninject();
		let remainder = match selected {
			Ok(coyo) => {
				let CensorF::Censor {
					f,
					action,
					k,
				} = coyo.lower();
				// The censored action writes into a fresh local log, and only the
				// transformed total is merged into the outer log afterwards. That
				// makes the censor transactional on abort BY DESIGN: an abort
				// inside the action drops the local log entirely (the `?` below),
				// because a censor is a listen-shaped boundary and the reference
				// elaboration (listen the sub-log, transform, tell) never tells
				// when the enclosed action aborts. Writes outside a censor survive
				// an abort (the shared-cell property `Catch` documents); the
				// durability difference is a property of where the boundary sits.
				let local = RefCell::new(String::new());
				run(
					action,
					&Handlers {
						log: &local,
						..*handlers
					},
				)?;
				let censored = f(local.into_inner());
				handlers.log.borrow_mut().push_str(&censored);
				program = k(());
				continue;
			}
			Err(rest) => rest,
		};
		// Every brand in the row has been peeled, so the remainder is the
		// uninhabited terminal row: this point is unreachable for any program.
		match remainder {}
	}
}

/// Recover from a typed `Except` throw at the boundary. Runs `program`; on a
/// typed abort (`Err(Abort::Except(e))`) it runs `recover(e)` and returns its
/// result, so the error value reaches the recovery and the recovery's value
/// becomes the program result; the other aborts (`Abort::Throw`,
/// `Abort::Empty`) propagate. This is the slice's first-order `Except`
/// elaboration: the effect is just the typed throw, and recovery is supplied
/// here at the boundary as a runExcept-shaped narrowing rather than an
/// embedded scoping construct.
pub(crate) fn run_except<A: 'static>(
	program: Free<Row, A>,
	handlers: &Handlers,
	recover: impl FnOnce(&'static str) -> Free<Row, A>,
) -> Result<A, Abort> {
	match run(program, handlers) {
		Ok(value) => Ok(value),
		Err(Abort::Except(error)) => run(recover(error), handlers),
		Err(other) => Err(other),
	}
}

/// A test fixture owning a default cell for every effect, so a per-effect parity
/// test exercises its own effect and lets the [`Handlers`] default the rest
/// rather than constructing the other effects' state by hand. Override the
/// `Reader` environment with [`Fixture::with_env`]; read an effect's final state
/// off the corresponding field after `run`.
#[cfg(test)]
pub(crate) struct Fixture {
	pub(crate) state: Cell<bool>,
	pub(crate) env: i32,
	pub(crate) log: RefCell<String>,
	pub(crate) fresh: Cell<usize>,
	pub(crate) fresh_succ: fn(usize) -> usize,
	pub(crate) input: RefCell<VecDeque<&'static str>>,
	pub(crate) kv_store: RefCell<BTreeMap<&'static str, i32>>,
}

#[cfg(test)]
impl Fixture {
	pub(crate) fn new() -> Self {
		Self {
			state: Cell::new(false),
			env: 0,
			log: RefCell::new(String::new()),
			fresh: Cell::new(0),
			fresh_succ: |c| c + 1,
			input: RefCell::new(VecDeque::new()),
			kv_store: RefCell::new(BTreeMap::new()),
		}
	}

	pub(crate) fn with_env(env: i32) -> Self {
		Self {
			env,
			..Self::new()
		}
	}

	pub(crate) fn with_input(items: impl IntoIterator<Item = &'static str>) -> Self {
		Self {
			input: RefCell::new(items.into_iter().collect()),
			..Self::new()
		}
	}

	pub(crate) fn with_kv_store(initial: BTreeMap<&'static str, i32>) -> Self {
		Self {
			kv_store: RefCell::new(initial),
			..Self::new()
		}
	}

	pub(crate) fn with_fresh(
		start: usize,
		succ: fn(usize) -> usize,
	) -> Self {
		Self {
			fresh: Cell::new(start),
			fresh_succ: succ,
			..Self::new()
		}
	}

	// FAN-OUT ANCHOR (Fixture builder): a stateful effect appends its field, a
	// default in `new`, an optional `with_<effect>` seeder, and a `handlers()` binding.

	pub(crate) fn handlers(&self) -> Handlers<'_> {
		Handlers {
			state: &self.state,
			env: self.env,
			log: &self.log,
			fresh: &self.fresh,
			fresh_succ: self.fresh_succ,
			input: &self.input,
			kv_store: &self.kv_store,
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::brands::{
			ArcBrand,
			BoxBrand,
			RcBrand,
		},
	};

	// The erased `Free`'s per-`Store` arms all accept the unified row. The
	// production interpreter runs on the `Box` store; this confirms the same
	// row also type-composes at the `Rc` and `Arc` stores (built, not run here:
	// interpreting effects at those stores needs the multi-shot effect stepping
	// added separately). `Free::lift_f` is one definition over every store, so
	// the only thing under test is that each `Free<Row, _, Store>` type-checks.
	#[test]
	fn row_composes_across_stores() {
		fn get_cell() -> Node<bool> {
			let coyo: Coyoneda<'static, StatePinned, bool> =
				Coyoneda::lift(StateF::Get(Box::new(|s| s)));
			Coproduct::inject(coyo)
		}
		let _box: Free<Row, bool, BoxBrand> = Free::lift_f(get_cell());
		let _rc: Free<Row, bool, RcBrand> = Free::lift_f(get_cell());
		let _arc: Free<Row, bool, ArcBrand> = Free::lift_f(get_cell());
	}

	// The order-directed peel classifies the active arm of a suspended layer by
	// order, over the one unified row. A `State` operation is first-order; a
	// `Catch` cell is higher-order.
	#[test]
	fn order_directed_peel_classifies_the_active_arm() {
		let state_program: Free<Row, bool> = get();
		let state_layer = state_program.resume();
		assert!(state_layer.is_err());
		if let Err(layer) = state_layer {
			assert_eq!(layer.classify(), OrderTag::First);
		}

		let catch_program: Free<Row, ()> = catch(Free::pure(()), || Free::pure(()));
		let catch_layer = catch_program.resume();
		assert!(catch_layer.is_err());
		if let Err(layer) = catch_layer {
			assert_eq!(layer.classify(), OrderTag::Higher);
		}
	}

	// Brand-keyed dispatch selects the active arm by effect brand, not by its
	// position in the row. `State` is the head arm of `Row` while
	// `Censor` is the tail arm; both are found by a type-directed `uninject` keyed
	// on the brand's cell, with the position inferred. This is the property that
	// makes the interpreter's dispatch-arm order independent of the row's declared
	// order, so the positional-sort footgun is gone.
	#[test]
	#[expect(
		clippy::expect_used,
		reason = "a suspended effect's `resume()` is `Err(layer)` by construction; `expect_err` extracts the layer this test then inspects, and a wrong `Ok` should fail the test loudly."
	)]
	fn brand_keyed_dispatch_selects_by_brand_not_position() {
		// `State` is the head arm of `Row`.
		let state_program: Free<Row, bool> = get();
		let state_layer = state_program.resume().expect_err("a suspended Get is a layer");
		let head: Result<Coyoneda<'static, StatePinned, Free<Row, bool>>, _> =
			state_layer.uninject();
		assert!(head.is_ok(), "the head brand is found by brand-keyed selection");

		// `Censor` is the tail arm of `Row`; brand-keyed selection reaches it the
		// same way, without walking coproduct positions by hand.
		// The row-generic `censor` constructor needs its row pinned before
		// `resume`, so the injector bound can force the log type from `Row`.
		let censor_program: Free<Row, ()> = censor(|s| s, Free::pure(()));
		let censor_layer = censor_program.resume().expect_err("a suspended Censor is a layer");
		let tail: Result<Coyoneda<'static, CensorPinned, Free<Row, ()>>, _> =
			censor_layer.uninject();
		assert!(tail.is_ok(), "the tail brand is found by brand-keyed selection");
	}
}

/// Depth cases through the reference interpreter: it is the reference
/// implementation of the elaboration recursion contract (an iterative
/// dispatch loop that recurses only per higher-order cell), so a deep chain
/// and a deep action under one `catch` must run without native stack
/// overflow. The public-surface deep-program suite covers the same
/// properties in the form user code takes.
#[cfg(test)]
mod depth_tests {
	use super::*;

	const DEPTH: usize = 100_000;

	#[test]
	fn deep_chain_runs_without_overflow() {
		let mut program: Free<Row, bool> = get();
		for _ in 1 .. DEPTH {
			program = program.bind(|_| get());
		}
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(false));
	}

	#[test]
	fn deep_action_under_one_catch_runs_without_overflow() {
		let mut action: Free<Row, ()> = put(true);
		for _ in 1 .. DEPTH {
			action = action.bind(|()| put(true));
		}
		let program = catch(action, || Free::pure(())).bind(|()| get());
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(true));
		assert!(fx.state.get());
	}
}
