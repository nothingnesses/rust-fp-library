# heftia

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/heftia/`

## Purpose

Stage 1 research document: classify `heftia` against the five effect-row
encodings catalogued in [../port-plan.md](../port-plan.md) section 4.1.
Identify whether this codebase is a variant of an existing option or
represents a genuinely novel encoding worth deeper investigation in Stage 2.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

Heftia uses a Free monad over a Freer structure as its fundamental encoding. The `Eff` type is defined as `type Eff = D.Eff Freer` (heftia/src/Control/Monad/Hefty/Types.hs:32), where `Freer f a` is a two-constructor free monad (Types.hs:23-30). The `Freer` type uses `Val a` for pure values and `forall x. Op (f x) (FTCQueue (Freer f) x a)` for effectful operations. The queue-based continuation (`FTCQueue`) enables O(1) bind and provides a fast-track monadic substrate (FTCQueue.hs:38-83, which is the "Reflection without Remorse" structure from Oleg Kiselyov's work).

The row encoding is delegated to the `data-effects` package, which Heftia reexports. The effect row is represented as an open union of effect types, indexed by membership traits that locate each effect in the row. The row itself is not visible in Heftia's own source; it is mediated through trait bounds like `FOEs es` (first-order effects) and `PolyHFunctors` (polymorphic higher-order functors) (Control/Monad/Hefty.hs:498-514). The row is effectively one ordered list of effects, not two.

### Distinctive contribution relative to baseline

The key innovation of Heftia relative to a standard `Free + Coproduct + Member` system is the dual-row architecture: the library distinguishes between first-order effect signatures and higher-order effect signatures and treats scoped / higher-order effects as first-class AST nodes (heftia/README.md lines 9-37). Higher-order effects such as `Catch`, `Span`, and `Provider` are reified as constructors in their own row, not encoded as Tactical-style continuations or wrapped in the first-order effect functor.

The dual row appears in effect definitions: `makeEffectF` generates first-order effects (e.g., `Log :: String -> Log f ()` from Control/Monad/Hefty.hs:30), while `makeEffectH` generates higher-order effects (e.g., `Span :: String -> f a -> Span f a` from Control/Monad/Hefty.hs:34). Higher-order handlers like `handleCatch` use `interposeWith` to rewrite scoped operations into composition points inside the computation (Control/Monad/Hefty/Except.hs:50-51). This is fundamentally different from polysemy's Tactical or fused-effects' HFunctor, which embed continuations as data inside the first-order effect algebraic type.

### Classification against port-plan section 4.1

Classification: Variant of Option 1 (type-level heterogeneous list / nested coproduct), enhanced by a dual-row structure that is orthogonal to the row encoding itself.

The underlying row representation is a nested coproduct with type-level membership indices (likely Peano-style or similar, coming from data-effects). This maps directly to option 1 as described in port-plan.md section 4.1 (lines 125-145). The two-row structure (first-order + higher-order) is not a novel row encoding; it is an architectural layer on top of a standard row. Heftia ships one row of first-order effects and one row of higher-order effects, both using the same underlying coproduct machinery; the novelty lies in how scoped operations become first-class elaboration targets, not in how the row itself is represented.

### Scoped-operations handling (`local`, `catch`, and similar)

Scoped operations are expressed as constructors in the higher-order effect row, not hidden inside first-order effect functors. The `Catch e` effect is defined in data-effects (reexported by Control/Monad/Hefty.hs) with signature `Catch :: String -> f a -> Catch f a`. Its handler (Control/Monad/Hefty/Except.hs:50) pattern-matches on `Catch action hdl` and uses `interposeWith` to interpose a `Throw` catcher into the `action` computation. The `action` itself is an `Eff es a`; a full, inspectable, reinterpretable program; not a captured one-shot continuation. This enables the reduction-semantics behavior documented in Control/Monad/Hefty.hs (lines 131-235): handlers rewrite the AST top-down, and the "rest of the computation" is available as a structurally-visible program for the handler to manipulate. Compared to polysemy (which uses `Tactical` continuation functions), this is more explicit and encourages term rewriting reasoning.

### Openness approach

Heftia achieves openness via the standard Open-Union machinery from data-effects. Functions are written polymorphic in a "tail" row variable, e.g., `fn my_program<R>(...) -> Eff (EFFECT + R) a`, where `R` is an unknown extension. The `Member<E, Row>` constraint (or its data-effects analogue `In`) proves membership of an effect in the row, and composition happens via row-polymorphic type arguments. This is the standard open-union approach used in polysemy, freer-simple, and other Haskell effect libraries. There is no novelty here; Heftia inherits the openness strategy from the underlying data-effects framework.

### Relevance to port-plan

Recommendation: No change needed to port-plan.md; consider Stage 2 deep-dive on scoped-operation ergonomics only if a compelling case emerges.

Heftia's row encoding (option 1 + nested coproduct) offers no new information relative to the existing research. Its key contribution is the dual row separating first-order from higher-order effects; this is orthogonal to the row encoding question that is blocking the port. The port-plan already contemplates first-order and higher-order effects as separate concerns (port-plan.md section 4.2 mentions "Functor dictionary" and the need to handle both); Heftia demonstrates that dual rows can coexist using standard coproduct machinery. However, whether a Rust port should adopt the same dual-row structure is a design question, not an encoding blocker question. If the port were to explore "how should we ergonomically express scoped operations in Rust?", Heftia's `elaborate` / `interpose` pattern is worth studying in Stage 2. For now, the port's immediate blocker (which of options 1-5 for the row encoding) remains unaffected.

### References

- heftia/src/Control/Monad/Hefty.hs (primary module; documentation and handler signatures)
- heftia/src/Control/Monad/Hefty/Types.hs (Freer free monad, FreeView, qApp)
- heftia/src/Control/Monad/Hefty/Interpret.hs (interpretation functions; membership & union dispatch)
- heftia/heftia-effects/src/Control/Monad/Hefty/Except.hs (Catch/Throw example showing higher-order handler pattern)
- heftia/src/Data/FTCQueue.hs (fast type-aligned queue for continuations)
- heftia/README.md (architecture overview; dual-row motivation)
- heftia/heftia.cabal (dependency on data-effects ^>= 0.4.2)

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
