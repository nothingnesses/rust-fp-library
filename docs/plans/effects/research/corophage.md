# corophage

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/corophage/`

## Purpose

Stage 1 research document: classify `corophage` against the five
effect-row encodings catalogued in [../decisions.md](../decisions.md)
section 4.1. Corophage is already named in the plan as a reference
implementation for option 4 (hybrid coproduct plus macro sugar); this
research confirms or updates that characterisation and surfaces any
details the plan's summary missed.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

Corophage uses a **Free-monad-like architecture built on async coroutines and frunk coproducts**. The primary value type is `Program<'a, Effs, R, L, Remaining, Handlers>` (corophage/src/program.rs:44), which wraps a `GenericCo<'a, Effs, Result, L>` (the coroutine). The coroutine is a thin wrapper around `fauxgen`'s `SyncGenerator` (corophage/src/coroutine.rs:25-29), which is a synchronous generator yielding `CanStart<Effs>` (the effect coproduct prefixed with a `Start` signal) and receiving back `Resumes<'a, CanStart<Effs>>` (a coproduct of resume types).

Effects are encoded as a right-nested `Coproduct<Head, Tail>` (corophage/src/effect.rs:1), imported from `frunk_core::coproduct`. The substrate is a pure data structure: user code constructs a `Program` closure, yields effects via `Yielder::yield_()` (corophage/src/coroutine.rs:164), suspends, and resumes with handler-provided values. No reflection, no trait objects at runtime beyond the async machinery itself.

### Distinctive contribution relative to baseline

Corophage's primary distinction is **lifting effect yields into an async/await syntax** via a `Yielder` type that wraps `fauxgen`'s generator token (corophage/src/coroutine.rs:140-149). User code reads: `y.yield_(MyEffect).await`, suspending the async block and resuming when the handler provides a value. This is syntactically indistinguishable from an await on a real Future; the abstraction is transparent.

A secondary distinction: **the `Program` type decouples computation from handlers**, allowing incremental handler attachment via `.handle()` (corophage/src/program.rs, around line 99 onwards). Handler order is flexible because `Program` uses `CoproductSubsetter` (from frunk) to remove handled effects from the `Remaining` phantom type (corophage/src/program.rs:47), so handlers can be added in any order when using the `Program` API. Low-level `sync::run` / `asynk::run` functions still require handlers in `Effects![...]` order, but the public API (`.handle()` chaining) hides this.

A third distinction: **single-shot-only handlers** (README.md:62). Each handler runs once and cannot replay or duplicate the continuation. This is a deliberate design constraint enforced by accepting `FnOnce` payloads in `Control::resume(value)`.

### Classification against decisions section 4.1

**Confirmed: Option 4 (Hybrid coproduct plus macro sugar).** Corophage is the reference implementation named in decisions.md:201.

Corophage uses **Peano-indexed coproducts internally** (Option 1, not Option 2 with typenum). Evidence: corophage/src/effect.rs:1-2 imports `Here, There` from `frunk_core::indices`, and the `InjectResume` trait uses these directly (effect.rs:104-120). `Here` marks the head, `There<T>` marks each tail position, producing O(n) index depth.

The `Effects![...]` macro (corophage/src/macros.rs:40-53) expands a flat list into nested `Coproduct<A, Coproduct<B, Coproduct<C, CNil>>>` with **no sorting or deduplication**; the expansion is left-recursive and preserves the user's order (macros.rs:47-48): `Effects![A, B, C]` becomes `Coproduct<A, $crate::Effects![B, C]>`. The macro accepts a `...Tail` spread syntax (macros.rs:44-45, effectful.rs:174-175) mirroring frunk's `Coprod!(...Tail)`.

### Scoped-operations handling (`local`, `catch`, and similar)

Not applicable. The corophage codebase contains no implementation of scoped operations like `local`, `catch`, `mask`, or other higher-order effect handlers. The library is designed around single-shot effect yields: handlers resume with a value, and that is all. There is no mechanism to suspend execution, run a sub-computation in a modified context, and resume. Multi-effect composition is supported via `invoke()` (coroutine.rs:212-263), which forwards effects from a sub-program to the outer program's handlers, but this is not a scoped operation in the PureScript sense (no `local :: ((forall x. Effect a x) -> Effect a x) -> m a -> m a` signature).

### Openness approach

Corophage achieves openness via **generic type parameters over the effect coproduct tail**. A function can be written with `fn foo<R>(program: Effectful<R, ...>) -> ...` where `R` is generic; the caller can instantiate `R` with any additional effects. More commonly, users write `type MyEffects = Effects![E1, E2, ...];` and then extend it via `type Extended = Effects![E3, ...MyEffects];` using the spread syntax. This is syntactic sugar over the coproduct's natural openness: adding an effect means prepending a new `Coproduct<NewEffect, OldCoproduct>`.

The `#[effectful]` macro (corophage-macros/src/effectful.rs:146-230) similarly supports spread syntax in the attribute: `#[effectful(E1, ...BaseEffects)]` expands to nested `Coproduct` with all effects combined (effectful.rs:173-176).

### Relevance to decisions

No change needed. Corophage's characterisation as Option 4 is accurate and complete. The plan's recommendation (section 4.1, "Leaning") stands: adopt Option 4 (macro-sugar hybrid) with Peano indices (Option 1) as the substrate, noting Option 2 (typenum binary indices) as a future refinement to mitigate error-message bloat. Corophage demonstrates that Option 1's error messages are tolerable in practice and that the permutation-proofs approach (`CoproductSubsetter`) for row-ordering is workable.

One implementation detail worth noting: corophage uses `'a` as a lifetime parameter on all effects (`Effects<'a>: MapResume + Send + Sync + 'a`, effect.rs:83), allowing effects to borrow non-`'static` data. This is valuable for programs that hold references to local state. The decisions's section 4.4 commitment to a four-variant `Free` family (with `FreeExplicit` supporting non-`'static` payloads) aligns with this design. Corophage demonstrates that the pattern is already practical and compatible with coroutine-based execution.

### References

- **Encoding and index system:** corophage/src/effect.rs (lines 1, 26-39, 83, 104-120) and corophage/src/coproduct.rs (lines 1-5).
- **Macro expansion:** corophage/src/macros.rs (lines 40-53), corophage-macros/src/effectful.rs (lines 146-230), especially lines 173-176 for spread syntax.
- **Program and handler composition:** corophage/src/program.rs (lines 44-98).
- **Yielder and effect yield syntax:** corophage/src/coroutine.rs (lines 140-192).
- **Row-ordering flexibility:** corophage/CLAUDE.md (design constraints), lines 20-25 discussing `CoproductSubsetter` and handler order for `Program.handle()`.
- **Single-shot constraint:** corophage/README.md, line 62.
- **Sub-program composition (invoke):** corophage/src/coroutine.rs (lines 212-263).
- **GAT lifetime pattern:** corophage/README.md (lines 369-407, "Borrowed resume types"), effect.rs (lines 26-39).

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding template boilerplate)
