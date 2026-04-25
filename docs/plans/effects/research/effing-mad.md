# effing-mad

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/effing-mad/`

## Purpose

Stage 1 research document: classify `effing-mad` against the five
effect-row encodings catalogued in [../decisions.md](../decisions.md)
section 4.1. Effing-mad is already named in the plan as a reference
implementation for option 1 (nested coproduct via frunk); this research
confirms or updates that characterisation and surfaces any details the
plan's summary missed, especially around its coroutine-based substrate.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

Effing-mad uses Rust's unstable `Coroutine` trait (feature flags `coroutines` and `coroutine_trait` required; see lib.rs:38-39) as its fundamental suspend-resume mechanism. Effects are encoded as nested coproducts using frunk's `Coproduct<H, T>` with `CNil` as the null case, giving a Peano-indexed linked list. Injections (values returned from effect handlers) are similarly nested coproducts, also built with frunk, but wrapped with `Tagged<T, Tag>` metadata to distinguish same-typed injections (injection.rs:20, lib.rs:58-60). The program itself is a first-class coroutine value you can pass around, inspect via pattern-matching on `CoroutineState::Yielded` vs `::Complete`, and resume multiple times (lib.rs:75-84 defines `run`, which only calls resume once on a pure coroutine, but `handle_group` at 185-228 shows that effectful programs are resumed in a loop until Complete).

### Distinctive contribution relative to baseline

Effing-mad differs from a basic Free + Coproduct + Member design in its choice of coroutine substrate. Instead of hand-rolling suspend/resume via an AST interpreter, it delegates to Rust's built-in `Coroutine` trait with the `#[coroutine]` proc-macro syntax, which is lighter than async (no allocation by default, no Waker) and more direct than state-machine enums. A second distinctive aspect is the use of `#[effectful(...)]` and `#[effectful::cloneable]` attributes (effing-macros/src/lib.rs:135-192) to emit coroutines from function-like syntax, including support for `yield expr` expressions that expand to effect injection and tagged extraction (lines 84-95). The `.do_` operator (lines 75-81) acts as an escape hatch to run nested effectful calls by yielding all their effects upward, similar to monadic bind. No Free monad wrapper is used; programs are bare coroutines.

### Classification against decisions section 4.1

Confirmed: effing-mad is Option 1, the nested coproduct with Peano indices approach. Evidence: frunk `Coproduct<H, T>` (lib.rs:60), `CNil` terminator (src/injection.rs:51), frunk trait-object re-exports `CoprodUninjector`, `CoproductSubsetter`, `CoproductEmbedder` (lib.rs:59), and handlers use these to manipulate rows (e.g., `handle` at lib.rs:136 constrains `PreEs: ... + CoprodUninjector<E, ...>` to extract an effect and `PostEs: ...` as the remainder). The macro `Coprod!(...)` at effing-macros/src/lib.rs:140, 340 expands to the nested type directly. No special normalization or sorting of the effect list is performed by the macro; the user-written order is preserved.

### Scoped-operations handling (`local`, `catch`, and similar)

Not documented in source. A grep search for "local", "catch", "mask", or "scope" across `src/` returns no matches. The library provides no built-in scoped-operations combinators; users must encode them themselves via nested effect handlers if needed.

### Openness approach

Extensibility is achieved via the trait-based design: any type `E` implementing `Effect` (lib.rs:88-91) can be used in an effectful computation. Custom effects are defined via `effects! {...}` (effing-macros/src/lib.rs:265-353), which generates both an effect group wrapper and individual effect types. Since coproducts are open (you can nest any head and tail), new effects can be added to a computation's row by composition: pass a computation with effects `(A, B)` to a handler for just `A`, yielding a computation with effects `(B)`, then pass that to a handler for another effect `C`, yielding a computation with effects `(B, C)`. The type system ensures this composition type-checks via frunk's coproduct membership traits.

### Relevance to decisions

Findings confirm the decisions's classification without requiring changes. One detail the plan's section 4.1 summary may have underemphasized: effing-mad's reliance on Rust's unstable `Coroutine` trait means any Rust port of PureScript's `Run` using this approach will require a nightly compiler indefinitely (or until the trait stabilizes, which has no committed timeline). This is a practical blocker for production use that the plan should flag explicitly in any Option 1 recommendation. Additionally, effing-mad's approach to handling groups via `CoproductSubsetter` / `CoproductEmbedder` (lib.rs:156, 205-206) shows that frunk's row-manipulation machinery works well, but the type-error messages when subset/embed relationships are violated are opaque (frunk generates 22+ type parameters for `handle` alone; lib.rs:136-149). The plan should consider whether this type-error experience is acceptable for users.

### References

- lib.rs:38-39: coroutine feature flags.
- lib.rs:58-60: frunk coproduct and trait re-exports.
- lib.rs:75-84: `run` function for pure coroutines.
- lib.rs:110-124: `map` combinator using `#[coroutine]` syntax.
- lib.rs:136-169: `handle` single-effect handler with type constraints.
- lib.rs:185-228: `handle_group` multi-effect handler.
- injection.rs:20, 45-56: `Tagged<T, Tag>` wrapper and `EffectList` trait.
- effing-macros/src/lib.rs:84-95: `yield` expansion logic.
- effing-macros/src/lib.rs:135-192: `#[effectful(...)]` attribute implementation.
- effing-macros/src/lib.rs:265-353: `effects!` macro generating groups and types.
- examples/nondet.rs:20-25: idiomatic `#[effectful(Nondet<T>)]` usage.
- src/effects/nondet.rs:26-34: `run_nondet` showing loop-based resumption.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
