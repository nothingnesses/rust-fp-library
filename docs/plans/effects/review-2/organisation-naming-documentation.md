# Organisation, Naming, and Documentation Findings

Snapshot: commit `2e0a7417`. Line numbers refer to that snapshot.

## 1. Code organisation

### 1.1 One effect's definition is spread across five locations

For a built-in effect such as State, the pieces live in:

1. `fp-library/src/brands/effects.rs`: the brands (`BoxStateBrand`, `StateBrand`, `SendStateBrand`).
2. `fp-library/src/types/effects/state.rs`: a thin shell invoking `define_effect! { effect State; }`.
3. `fp-macros/src/documentation/generator_builders/state_effect_items.rs` and `state_wrapper_impl_items.rs`: the actual operation enum, Functor/WrapDrop impls, smart-constructor bodies, and runner bodies, as quoted token streams.
4. `fp-library/src/types/effects/named_helpers/state.rs`: per-wrapper helper impl blocks invoking `define_run_wrapper!` plus some hand-written helpers (`gets`, `eval_state`, `exec_state`).
5. `fp-library/src/types/effects/run/smart_constructors.rs` (and five siblings): more `define_run_wrapper!` invocation shells grouped by wrapper rather than by effect.

Understanding or changing one effect therefore requires visiting both crates and both grouping axes (by-effect and by-wrapper). The by-wrapper smart-constructor shells and the by-effect named-helper shells could be merged into one axis (by effect is the natural one, since the wrapper dimension is what `define_run_wrapper!` abstracts over).

### 1.2 Effect codegen lives under the documentation subtree

`define_effect!` and `define_run_wrapper!` are not standalone proc macros; they are markers expanded by the `#[fp_macros::document_module]` attribute (`fp-macros/src/documentation/item_generators.rs`, `expand_define_effect` at line 190), with the generated token streams authored in `fp-macros/src/documentation/generator_builders/*.rs` (fourteen files: `first_order_effect_items`, `reader_effect_items`, `state_effect_items`, plus eleven `*_wrapper_impl_items` files and `run_wrapper_method_impl_items`).

There is a reason for the coupling (the documentation validator must see generated items to attach and check `document_*` attributes), but the result is that the de facto effect-definition system of the library is filed under "documentation" in the macro crate, where nobody would look for it. The registry is also keyed by hardcoded effect names, so adding a built-in effect means writing proc-macro builder code, not library Rust. Recommendation R7 in [refactoring-opportunities.md](refactoring-opportunities.md) covers relocating this under `fp-macros/src/effects/` with an explicit descriptor table, keeping only the documentation hooks in the documentation tree.

### 1.3 Names that obscure rather than reveal

- `named_helpers` says nothing about its contents (per-effect runner and convenience methods). `runners` or `effect_helpers` would be discoverable; better still, fold the contents into the per-effect modules (1.1).
- `effects_macro.rs` is named to dodge clippy's `module_inception` (documented in the file); harmless, but it signals that `fp-macros/src/effects/effects.rs` wants to be a differently-shaped module (for example `rows.rs`, since it builds rows).
- The internal `define_effect!` shares its name with the user-facing macro that `run.md` and `custom-effects.md` promise for the future ("A future `define_effect!` macro should remove stable boilerplate"). A reader who greps for `define_effect!` finds it already exists and is left to discover it is registry-keyed and internal-only. Rename the internal one (for example `define_builtin_effect!`) or, preferably, build the public one and make the internal registry an instance of it (R4).

### 1.4 Scaffolding accumulation

There are 43 `dead_code` expectation/allow lines under `fp-library/src/types/effects*`, several with reasons of the form "until production wiring consumes it" (`interpreter/scoped_resume.rs`: `ExplicitBoundaryOf` compatibility alias, `ScopedBoundaryTypes`; `run_explicit/boundary.rs`: action-supplied carriers; `async_interpreter.rs`: `handle_async`). Each is individually justified, but collectively they record an in-flight migration; once the boundary-carrier wiring is complete they should be swept, and any alias kept for compatibility should be removed per the project's no-compatibility-shims principle.

### 1.5 The shared prelude blob

`standard_scoped_handlers.rs` front-loads a 200-line `mod prelude` importing every brand, carrier, continuation, and protocol type for all six wrapper families, with an `allow(unused_imports)`. This is symptomatic of the six-fold matrix (each child module consumes a different sixth of the prelude) rather than a problem in itself; it will dissolve if R2 lands.

## 2. Naming

### 2.1 The brand sibling scheme

Current scheme per effect (where the effect has closure-typed continuations):

| Flavour      | Brand                  | Continuation storage                          | Used by                    |
| ------------ | ---------------------- | --------------------------------------------- | -------------------------- |
| Box          | `BoxStateBrand<P, S>`  | `Box<dyn FnOnce>`                             | `Run`, `RunExplicit`       |
| (unprefixed) | `StateBrand<P, S>`     | `Rc<dyn Fn>` via `ToDynCloneFn`               | `RcRun`, `RcRunExplicit`   |
| Send         | `SendStateBrand<P, S>` | `Arc<dyn Fn + Send + Sync>` via `ToDynSendFn` | `ArcRun`, `ArcRunExplicit` |

Problems:

- The `P` parameter on the Box flavour is redundant; the docs say "`P: ToDynFnOnce` (`BoxBrand` only)". A parameter with exactly one legal instantiation is noise in every signature and row alias.
- The unprefixed name denotes the Rc flavour, but the default wrapper (`Run`) uses the Box-prefixed brands; the "default-looking" name and the default wrapper disagree. Users writing their first row for plain `Run` must learn to reach for `BoxStateBrand`.
- `Send` names the bound, while `Box` names the pointer; the prefix axis is not even internally consistent. The rest of the library spells the pointer (`RcFnBrand`, `ArcFnBrand`, `FnBrand<P>` in optics).
- Effects without continuations (`ExceptBrand<E>`, `WriterBrand<W>`, `EmptyBrand`, `FailBrand`, `LogBrand<Message>`, `OutputBrand<Out>`, `AwaitBrand`) correctly have no siblings, which is good, but it means rows mix three-sibling and no-sibling brands, and a user cannot predict from the effect name which spelling a given wrapper needs.
- Bracket adds a second axis: `BoxBracketExplicitBrand`, `BracketExplicitBrand`, `SendBracketExplicitBrand` exist alongside the non-Explicit trio (six brands for one effect), because the Explicit substrate stores the action differently.

The end-state worth pursuing (R3): one brand per effect, parameterized by a closure-storage brand with an associated dyn type (a unifying class over today's `ToDynFnOnce`/`ToDynCloneFn`/`ToDynSendFn`), so the table above collapses to `StateBrand<BoxBrand, S>` / `StateBrand<RcBrand, S>` / `StateBrand<ArcBrand, S>`. If a unifying class is blocked by the type system (the FnOnce/Fn arity-and-receiver split is the hard part), the fallback is a uniform prefix scheme (`BoxStateBrand`/`RcStateBrand`/`ArcStateBrand`, no redundant `P`), with the limitation documented.

### 2.2 Alias methods double the API surface

Every wrapper exposes both `handle` and `run`, and both `handle_rec` and `run_rec`, where one of each pair is a literal one-line alias kept "for naming parity with PureScript Run". Across six wrappers that is twenty-four exported methods for twelve behaviours, doubled documentation, and a permanent "which one is canonical" question. PureScript itself aliases `interpret = run` for historical reasons, not as a design feature. Pick one family (`handle`/`handle_rec` matches the rest of the library's vocabulary) and drop the other (R8).

### 2.3 Other naming notes

- `ScopedCoproduct`/`ScopedNil` are transparent aliases of `CoproductBrand`/`CNilBrand` whose stated purpose is readability of signatures. They enforce nothing (a first-order brand placed in `S` is caught later, by handler resolution, not by these names). Either keep them purely as documentation (fine) or consider making the scoped row a distinct brand if the extra type-level separation would improve diagnostics; do not let them be mistaken for a safety boundary.
- `im_do!` is cryptic next to `m_do!`/`a_do!`; the "inherent-method" rationale is macro-implementation trivia, not user-facing meaning. A name like `run_do!` would say what it is for. Low priority.
- `handle_with_either` reads as "runExcept returning a program", but it is a driver: it interprets every other effect with the supplied handlers and returns `Result<A, matched-op>` for the whole program. A name like `drive_until` or `handle_all_or_intercept` would be honest; alternatively implement the true `run_except`-shaped narrowing (it exists separately as the generated `run_except`) and reserve the either-name for it. See R9.
- `Writer` (the tell effect) versus `WriterCensorBrand`/`WriterListenBrand` (the scoped ops) is consistent with purescript-run, and `Span` is a sensible library-original; no change needed, noted for completeness.
- `effects!` builds Box-family rows only (`CoyonedaBrand` wrapping); Rc/Arc rows require `define_effect_row_aliases!` with `rc_first_order [...]` / `arc_first_order [...]`. The asymmetry (an expression-position macro for one family, item-position aliases for the others) is undocumented in `run.md` and surprising; either add `rc_effects!`/`arc_effects!` or document the alias-macro path as the only one for shared wrappers.

## 3. Documentation accuracy (drift list)

The subsystem's documentation is unusually extensive, which makes the stale parts actively misleading. Concrete drift found:

1. `types/effects/node.rs` lines 29 to 31: "Currently the scoped row is structurally a second CoproductBrand chain ... Future work will populate it with the standard scoped constructors (Catch, Local, ...)". The constructors landed; the module doc and the `Functor` impl doc (line 95: "the scoped row will satisfy it once scoped constructors land") both predate them.
2. `types/effects/interpreter.rs` lines 76 to 99: the "Async / IO workaround: spawn_blocking" section states "No `async fn` interpreter variant ships" and recommends blocking-thread workarounds. `Run::run_async`/`Await` shipped; this section must be rewritten or deleted.
3. `types/effects/standard_scoped_handlers/catch.rs` (CatchHandler doc): "This handler is implemented for Rc-backed and Arc-backed wrappers. Box-backed `Run` / `RunExplicit` need a separate design". The same file's doctest handles a Box-backed `Run` program with `catch_handler` via the boundary-frame mechanism; the limitation paragraph is stale.
4. `types/effects/kv_store.rs` line 4, `fresh.rs` line 4, `input.rs` line 4: "The standard W11 runner will use ..." (future tense). The runners exist (`run_kv_store`, `run_fresh`, `run_fresh_with`, `run_input_seq`, on all six wrappers).
5. `fp-library/docs/run.md`: the "Built-in first-order effects currently include" list names six effects (State, Reader, Except, Writer, Choose, Empty) of the fourteen shipped (omitting Coroutine, Fresh, Fail, Input, KVStore, Log, Output, Await), and the document never mentions the generated runner families (`run_state`, `run_reader`, `run_writer`, `fold_writer`, `run_except`, `run_nondet`, `run_coroutine`, and the rest), which are the API most users will actually call. The doc teaches only the raw `handlers!` path, which is the advanced path.
6. `fp-library/docs/run.md` and `custom-effects.md`: "A future `define_effect!` macro" reads as if no such macro exists, while an internal one does (see 1.3). Clarify the relationship.
7. `Run::handle` doc (`run.rs` lines 996 to 1000): claims per-layer host-stack recursion; the body is iterative (see architecture section 3.8).
8. `custom-effects.md` does not state the single-continuation-hole requirement for Box-family effects (architecture section 3.4), nor `WrapDrop`'s role in drop-time semantics beyond the brief step 5 note, nor that the effect type must be given `SendFunctor`/`RefFunctor`/`Extract` impls before it can appear in rows used by the Arc family or by `peel`-style traversals (the worked example compiles because it only targets the default wrapper).

## 4. Self-containedness violations

The project rule (AGENTS.md, "Self-Contained Documentation") forbids plan-workstream references in source files. Current violations in the effects subsystem:

- `types/effects/await_future.rs` line 3: "This is the W13 `Future`-embedding (base-lift) effect ...".
- `types/effects/async_interpreter.rs` line 104: `expect(dead_code)` reason beginning "W13 async-interpreter foundation, retained until ...".
- `types/effects/kv_store.rs` line 4, `fresh.rs` line 4, `input.rs` line 4: "The standard W11 runner ...".
- `types/effects/row_embed.rs` line 12: "Generated wrapper methods call this module in the following W1 slice."
- `fp-library/tests/run_heftia_semantics.rs` lines 6 to 11: the module doc cites pinned GitHub URLs into the heftia repository. The rule as recorded for this project also covers GitHub-URL references in `.rs` files; if provenance must be kept, restate the ported cases in concrete terms (which the file partially does) and move the URLs to a plan document.

Each of these should be rewritten in self-contained terms (what the thing is and why it exists, not which workstream produced it); the W-labels will be meaningless to any reader once the plan documents move on.
