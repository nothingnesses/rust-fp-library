//! FS-1 effects rebuild, crate-internal work in progress.
//!
//! This module is the in-tree vertical slice of the unified-row effects
//! rebuild. It is deliberately `pub(crate)` and not part of the public
//! surface: per the adopted hybrid method, the slice was built to a
//! compiling, test-backed state before the dual-row subsystem it replaces
//! was deleted, so a half-built rewrite never shipped; the public FS-1
//! effect API is built on top of it next.
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
//! the brand-keyed dispatch that removes the positional-sort footgun (item 8's
//! mechanism). Per-brand order markers and the order-directed peel classify
//! whether an active arm is first-order or higher-order; they are exercised by
//! the order-classification test and become the routing layer when elaboration
//! is generalised over the row.
//!
//! Documentation status: this module and its effect submodules intentionally do
//! NOT yet use the `#[fp_macros::document_module]` wrapper that the rest of
//! `fp-library/src/` uses. The effects here are hand-written placeholders that
//! the forthcoming FS-1 `define_effect!` macro will regenerate, so
//! hand-documenting them now would be throwaway: `document_module` requires
//! signature/type-parameter/parameter/return/example attributes with runnable
//! doctests on every method. The wrapper and full per-item documentation are
//! added once the FS-1 macro and the remaining prerequisites (the code the
//! production tests need) exist; until then this is a tracked, temporary
//! exception, not an oversight.

#![allow(
	dead_code,
	reason = "FS-1 rebuild in progress (item 4): these items form the vertical slice and are currently exercised only by this module's tests; the public surface that consumes them is added in later steps, and item 20 sweeps any residual allowances at the end of the rebuild."
)]

use {
	crate::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::coproduct::{
				CNil,
				Coproduct,
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
mod state;
mod throw;
mod writer;
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
// A build-gated proof that the continuation-as-data async driver re-points onto
// the FS-1 `Free` substrate (peel, project the await brand, lower and await).
#[cfg(test)]
mod async_poc;

// The smart constructors are re-exported flat (`fs1::get`, ...) so an effect's
// parity test names a sibling effect's constructor (and its own) by the flat
// path, without reaching into each effect submodule. The rule: the flat block
// covers smart constructors only; program transformers (the interpose walkers)
// are imported via their module path.
#[allow(
	unused_imports,
	reason = "the smart constructors are exercised only by this slice's tests, exactly like the dead_code allowance above, so the flat re-exports have no non-test consumer yet and read as unused in a lib-only build; both clear once item 11's public surface consumes the slice."
)]
pub(crate) use self::{
	bracket::bracket,
	catch::catch,
	censor::censor,
	empty::empty,
	except::throw_e,
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
	state::{
		get,
		put,
	},
	throw::throw,
	writer::tell,
};
// The effect brands and functor payloads the shared `Row` and interpreter name.
use self::{
	bracket::{
		BracketBrand,
		BracketCell,
	},
	catch::{
		CatchBrand,
		CatchCell,
	},
	censor::{
		CensorBrand,
		CensorCell,
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
		ListenCell,
	},
	local::{
		LocalBrand,
		LocalCell,
	},
	reader::{
		ReaderBrand,
		ReaderF,
	},
	state::{
		StateBrand,
		StateF,
	},
	throw::ThrowBrand,
	writer::{
		WriterBrand,
		WriterF,
	},
};

// -- Per-brand order markers and the order-directed peel --

/// First-order order marker: the effect's representation does not depend on the
/// carrier (no sub-program in a negative position).
pub(crate) struct FirstOrder;
/// Higher-order order marker: the effect owns a sub-program (it is elaborated).
pub(crate) struct HigherOrder;

/// Each effect brand carries its order as an associated marker. This is the
/// unified row's classification: first-order and higher-order effects live in
/// the same row and are told apart by this marker, not by a separate row. The
/// per-effect implementations live in the effect submodules.
pub(crate) trait OrderOf {
	type Order;
}

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
// `Kind_cdc7cd43dac7585f` is the macro-generated `Kind` trait for the
// `type Of<'a, T: 'a>: 'a` shape (from the `kinds` module); naming
// `Coyoneda<'a, E, _>` requires its brand `E` to satisfy it. This matches how
// the library's own generated impls reference the trait.
impl<'a, E: OrderOf + Kind_cdc7cd43dac7585f, A> CellOrder for Coyoneda<'a, E, A> {
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
/// The slice's pinned instantiations of the parameterised higher-order cell
/// brands: the cells are generic (environment, log, resource, and result
/// types), and the row is where a slice commits to concrete types. These
/// aliases are that commitment, stated once and reused by the row and the
/// dispatch arms.
pub(crate) type LocalPinned = LocalBrand<i32, i32>;
pub(crate) type ListenPinned = ListenBrand<i32, String>;
pub(crate) type BracketPinned = BracketBrand<i32, i32>;
pub(crate) type CensorPinned = CensorBrand<String, ()>;

pub(crate) type Row = CoproductBrand<
	CoyonedaBrand<StateBrand>,
	CoproductBrand<
		CoyonedaBrand<ThrowBrand>,
		CoproductBrand<
			CoyonedaBrand<CatchBrand<()>>,
			CoproductBrand<
				CoyonedaBrand<ReaderBrand>,
				CoproductBrand<
					CoyonedaBrand<WriterBrand>,
					CoproductBrand<
						CoyonedaBrand<CensorPinned>,
						CoproductBrand<
							CoyonedaBrand<FreshBrand>,
							CoproductBrand<
								CoyonedaBrand<InputBrand>,
								CoproductBrand<
									CoyonedaBrand<KVStoreBrand>,
									CoproductBrand<
										CoyonedaBrand<EmptyBrand>,
										CoproductBrand<
											CoyonedaBrand<LocalPinned>,
											CoproductBrand<
												CoyonedaBrand<ListenPinned>,
												CoproductBrand<
													CoyonedaBrand<BracketPinned>,
													CoproductBrand<
														CoyonedaBrand<ExceptBrand<&'static str>>,
														// FAN-OUT ANCHOR (Row tail): append `CoyonedaBrand<NewBrand>` by wrapping
														// the terminal `CNilBrand` as `CoproductBrand<CoyonedaBrand<NewBrand>, CNilBrand>`.
														CoproductBrand<
															CoyonedaBrand<IdentityBrand>,
															CNilBrand,
														>,
													>,
												>,
											>,
										>,
									>,
								>,
							>,
						>,
					>,
				>,
			>,
		>,
	>,
>;

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
		let selected: Result<Coyoneda<'static, StateBrand, Free<Row, A>>, _> = layer.uninject();
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
		let selected: Result<Coyoneda<'static, WriterBrand, Free<Row, A>>, _> = layer.uninject();
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
				let IdentityF(next) = coyo.lower();
				program = next;
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, CatchBrand<()>, Free<Row, A>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				let CatchCell {
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
				let LocalCell {
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
				let ListenCell {
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
				let BracketCell {
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
				let CensorCell {
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
			let coyo: Coyoneda<'static, StateBrand, bool> =
				Coyoneda::lift(StateF::Get(Box::new(|s| s)));
			Coproduct::inject(coyo)
		}
		let _box: Free<Row, bool, BoxBrand> = Free::lift_f(get_cell());
		let _rc: Free<Row, bool, RcBrand> = Free::lift_f(get_cell());
		let _arc: Free<Row, bool, ArcBrand> = Free::lift_f(get_cell());
	}

	// Item 4 step 3: the order-directed peel classifies the active arm of a
	// suspended layer by order, over the one unified row. A `State` operation is
	// first-order; a `Catch` cell is higher-order.
	#[test]
	fn order_directed_peel_classifies_the_active_arm() {
		let state_layer = get().resume();
		assert!(state_layer.is_err());
		if let Err(layer) = state_layer {
			assert_eq!(layer.classify(), OrderTag::First);
		}

		let catch_layer = catch(Free::pure(()), || Free::pure(())).resume();
		assert!(catch_layer.is_err());
		if let Err(layer) = catch_layer {
			assert_eq!(layer.classify(), OrderTag::Higher);
		}
	}

	// Item 4 step 3: brand-keyed dispatch selects the active arm by effect brand,
	// not by its position in the row. `State` is the head arm of `Row` while
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
		let state_layer = get().resume().expect_err("a suspended Get is a layer");
		let head: Result<Coyoneda<'static, StateBrand, Free<Row, bool>>, _> =
			state_layer.uninject();
		assert!(head.is_ok(), "the head brand is found by brand-keyed selection");

		// `Censor` is the tail arm of `Row`; brand-keyed selection reaches it the
		// same way, without walking coproduct positions by hand.
		let censor_layer =
			censor(|s| s, Free::pure(())).resume().expect_err("a suspended Censor is a layer");
		let tail: Result<Coyoneda<'static, CensorPinned, Free<Row, ()>>, _> =
			censor_layer.uninject();
		assert!(tail.is_ok(), "the tail brand is found by brand-keyed selection");
	}
}
