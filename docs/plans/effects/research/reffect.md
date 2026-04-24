# reffect

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/reffect/`

## Purpose

Stage 1 research document: classify `reffect` against the five
effect-row encodings catalogued in [../port-plan.md](../port-plan.md)
section 4.1. Reffect is already named in the plan as the reference
implementation for option 2 (typenum-indexed sum list); this research
confirms or updates that characterisation and surfaces any details the
plan's summary missed.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

Reffect uses **coroutines as the execution substrate** (Rust native `Coroutine` and `CoroutineState`), not Free monads or generators backed by continuations. The fundamental type is the `Effectful<E: EffectList>` trait in `src/effect.rs:97-107`, which bounds a coroutine sending `Sum<E>` (effects) and resuming with `Sum<(Begin, E::ResumeList)>` (resume values tagged by effect type). This is a first-class coroutine, not a one-shot function.

The effect row is encoded as a right-nested tuple: `(Effect1, (Effect2, (Effect3, ())))`. The **core union type** is `Sum<S: SumList>` defined in `src/util/sum_type.rs:27-30`. It stores a compile-time-erased discriminant tag as a `u8` and a `ManuallyDrop<Repr<S>>` where `Repr<S>` is a right-nested union type produced by the `repr` module. Each position in the nested coproduct is accessed via the `Split<T, U: Tag>` trait (src/util/sum_type/repr.rs:45-67), which proves membership and computes pointer offsets. The implementation is unsafe but rigorous, with comprehensive bounds checking and memory safety at the use sites.

### Distinctive contribution relative to baseline

Reffect contributes a **compile-time-indexed handler composition pipeline** and **proc-macro sugar for coroutine decoration**, not fundamentally novel indexing. Its main distinction from a "Free + Coproduct + Member" baseline:

1. **Coroutine-native execution:** Programs are live, pauseable coroutines that yield effects and resume with provided values, rather than AST nodes evaluated by a free-monad interpreter. The handler `Handle<Coro, H, Markers>` struct in `src/adapter/handle.rs:36-42` wraps a coroutine and a handler, running them in a loop: yield an effect, invoke the handler, broaden the resume value, resume the coroutine. No Free monad needed.

2. **Handler composition and narrowing:** Reffect implements `Catcher<T, E, F>` (src/effect.rs:109-115) and `Handler<T, E>` (src/effect.rs:149-171) traits that separate catching (building the next effect) from handling (returning a value). This allows `catch0`, `catch1` methods (src/adapter.rs:59-86) to narrow effect rows at the type level while preserving first-class program semantics.

3. **Macro-sugar for effect groups and row specification:** The `#[effectful(...)]` attribute (macros/src/lib.rs:146-156) and `#[group]` trait macro (macros/src/lib.rs:159-171) normalize effect rows by converting flat effect lists into nested tuples via the `Effect` enum parsing. No canonical sort is applied; order is preserved as written by the user.

### Classification against port-plan section 4.1

**Confirmed as Option 2 (typenum-indexed sum list), with strong coroutine-execution overlap from Option 4 (hybrid approach).**

The tag type `U: Tag` in the `Split<T, U>` trait is a Peano-like chain, not a true typenum binary natural. The trait impls in `src/util/sum_type/repr.rs:69-107` and `src/util/sum_type/repr.rs:109-151` show that `UTerm` maps to index 0, and `UInt<U>` increments by 1 for each nesting level, giving O(n) type depth, not O(log n). This is technically closer to **Option 1 (Peano-indexed coproduct) than Option 2**.

The plan's summary claims O(log n) depth via typenum binary naturals; reffect does not realize that optimization. However, reffect's tag type avoids the `There<There<...>>` naming in error messages by using phantom-type wrappers (`UInt`, `UTerm`) instead of tuple nesting, which marginally improves readability. The runtime tag is a simple `u8` in all cases, independent of the type-level encoding.

### Scoped-operations handling (`local`, `catch`, and similar)

**Partial support via handler combinators, not dedicated primitives.** Reffect does not expose `local`, `mask`, or `async-safe` as named operations. Instead:

- **`catch`:** Modeled via the `Catcher` trait (src/effect.rs:109-115) and `catch0` / `catch1` methods (src/adapter.rs:59-86). A catcher is a mutable function that takes an effect and returns an `Effectful<F>` (a coroutine that may yield further effects or return a value). Allows nested effect handling and transformation of effect rows within the handler.

- **`local`-like scoped-state operations:** Not a built-in primitive; deferred to custom effect implementations (e.g., passing a mutable `Gc<T>` or thread-local state through a handler). Example in `examples/gc.rs:71-86` shows state threaded via handler mutable references.

The `ControlFlow` enum (src/effect.rs line 98, imports from `core::ops`) is used to signal handler early return (`Break(value)`) or continuation (`Continue(resume_value)`, mapped to the next coroutine resume).

### Openness approach

**Openness is achieved via generic coroutine composition and trait bounds on `SumList`, not macro-based normalization.**

A function polymorphic in a tail effect set `R` can be written as:

```rust
fn my_prog<R: SumList>() -> impl Effectful<ConcatList<SomeEffects, R>> { ... }
```

Here, `ConcatList` (src/util/sum_type/range.rs:34-48) is a type-level concatenation that appends `R` to a fixed effect list. The user does not write tuples directly; the `#[effectful(E1, E2, ...)]` attribute expands the flat list into nested tuples at compile time.

**Row-ordering is user-controlled, not normalized.** Two functions returning `(E1, (E2, ()))` and `(E2, (E1, ()))` have distinct types and will not unify. Reffect does not provide `Embedder` or `Subsetter` traits to convert between orderings. Instead, the handler-composition pipeline via `catch0` and `catch1` (which invoke `SplitList::narrow_tag` in src/util/sum_type/range.rs:50-57) allows type-level reordering at handler boundaries. This defers the cost to composition time, not definition time.

### Relevance to port-plan

**Change required: rethink the O(log n) claim in section 4.1 option 2 summary.**

Reffect's tag encoding is Peano, not typenum-binary. The plan states:

> "Binary naturals scale better; index-type depth grows logarithmically with effect count."

Reffect contradicts this: it uses `UInt<UInt<...UTerm...>>` (O(n) type nesting), not `UInt<UInt<UTerm, B1>, B0>` (O(log n)). The performance advantage of option 2 over option 1 is partly theoretical; in practice, `UInt` has phantom-type structure that may improve error messages, but compile-time scaling is still O(n) in effect count because trait resolution recurses through the full nesting depth on every `Split` lookup.

**Execution model note:** Reffect demonstrates that option 4 (hybrid macro-sugar coproduct) can be married with coroutine-native execution and still preserve first-class program semantics. Programs are coroutines, not Free-monad ASTs, yet remain composable via handler chaining. The port should evaluate whether this hybrid execution model (coroutines + handler composition, not Free + interpret) is preferable to the traditional Free-monad path the plan currently favours.

**Row normalization note:** Corophage (the plan's other option 4 reference) applies lexical sorting to effect names via macro, preserving user order only in the output type name. Reffect does not sort; row order is preserved exactly as written. For the port, this is a decision point: does normalizing order improve ergonomics enough to justify the macro complexity, or should the port accept that functions returning effects in different orders have incompatible types?

### References

- Core types and traits: `src/effect.rs` (Effect, EffectList, Effectful, Catcher, Handler, ResumeTy).
- Sum type and membership: `src/util/sum_type.rs` (Sum struct), `src/util/sum_type/repr.rs` (SumList trait, Split trait, Cons union, tag-indexed access).
- Tag encoding: `src/util/tag.rs` (UTerm, UInt, Tag trait, constant VALUE).
- Handler and adapter: `src/adapter.rs` (EffectfulExt trait, run, handle, catch0, catch1), `src/adapter/handle.rs` (Handle struct, coroutine resume loop).
- Macro support: `macros/src/lib.rs` (effectful, group, group_handler, handler, catch macros and their expansion), `macros/src/func.rs`, `macros/src/group.rs`.
- Example programs: `examples/gc.rs` (custom Gc memory effect with mutable handler state).
- Port-plan comparison: effect-row options 1-5, section 4.1; section 4.4 Free-family commitment.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
