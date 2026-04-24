# polysemy

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/polysemy/`

## Purpose

Stage 1 research document: classify `polysemy` against the five
effect-row encodings catalogued in [../port-plan.md](../port-plan.md)
section 4.1. Identify whether this codebase is a variant of an existing
option or represents a genuinely novel encoding worth deeper
investigation in Stage 2.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

Polysemy uses a Free monad plus a type-level union encoding. The core type is `Sem r a` defined at `src/Polysemy/Internal.hs:214` as a newtype wrapping a continuation:

```haskell
newtype Sem r a = Sem
  { runSem
        :: forall m
         . Monad m
        => (forall x. Union r (Sem r) x -> m x)
        -> m a
  }
```

The effect row `r` is represented as a type-level list that is consumed by the `Union` type (defined at `src/Polysemy/Internal/Union.hs:65`). A `Union r mWoven a` holds either an effect or the tail of the list, witnessed by an `ElemOf e r` proof. The `ElemOf` type is defined at line 148 as a newtype wrapping an `Int` index:

```haskell
newtype ElemOf (e :: k) (r :: [k]) = UnsafeMkElemOf Int
```

This index is resolved via the `Member` typeclass (line 198) with instances that use `Here` (index 0) and `There` (recursive step) patterns to navigate the type-level list. The proof-construction happens at compile time through recursive trait instances at lines 202-206, which unroll to an integer position at runtime.

The `Weaving` type (line 82) bundles the effect together with functor state and distribution functions for threading stateful contexts through higher-order operations.

### Distinctive contribution relative to baseline

Polysemy's signature innovation is the `Weaving` abstraction and its supporting `Tactical` environment. Rather than requiring interpreters to manually thread state through higher-order effect parameters, `Weaving` wraps the effect along with (1) a functor `f ()` representing the state accumulated by all upstream effects, (2) a distribution function `forall x. f (Sem rInitial x) -> mAfter (f x)` to thread that state, and (3) an inspection function for attempting to peek inside values. This is defined in `src/Polysemy/Internal/Union.hs:82-107`.

The `Tactical` monad (defined at `src/Polysemy/Internal/Tactics.hs:77-78`) reifies this threading mechanism as a first-class environment. Instead of directly manipulating continuations, interpreters use combinators like `runT`, `bindT`, `pureT`, and `getInitialStateT` (lines 145-216) to route monadic actions and state. This makes writing higher-order interpreters (e.g., Error.catch, Resource.bracket) straightforward: the user writes code that looks sequential, and the Tactical machinery handles the state threading automatically.

The `Scoped` effect (documented in `src/Polysemy/Internal/Scoped.hs:106-109` and interpreted in `src/Polysemy/Scoped.hs`) is a second major contribution: it decouples resource allocation from effect interpretation by introducing a meta-effect that wraps the programmer's computation. The interpreter receives the wrapped computation and decides when to allocate/deallocate resources.

### Classification against port-plan section 4.1

Polysemy is a tight variant of **option 1: Type-level heterogeneous list / nested coproduct**. The effect row is encoded as a type-level list that maps to integer indices at runtime via `ElemOf`. The Member resolution is Peano-style (recursive instances at `src/Polysemy/Internal/Union.hs:202-206`), so index depth scales O(n). The indexing proof is hidden from users except in error messages.

However, polysemy differs from the port-plan's description of option 1 in one significant way: the actual member proofs are integers, not type-level witnesses. The compile-time proof uniqueness is guaranteed by Haskell's constraint solver, but the runtime representation is a bare `Int`. This avoids the monomorphisation overhead that would result from generating a distinct instance per call site if the index were a type-level phantom (as in frunk). Haskell's type erasure makes this trade-off safe; Rust would need to decide whether to accept the monomorphisation cost or use runtime tags (approaching option 3).

This is still squarely within option 1's design space.

### Scoped-operations handling (local, catch, and similar)

Polysemy uses two intertwined mechanisms for higher-order operations. First, the `Weaving` type packages effect values with state-threading infrastructure (line 82-107 in `src/Polysemy/Internal/Union.hs`). Second, the `Tactical` monad provides interpreter-facing combinators that manage that infrastructure.

For concrete examples: `Error.catch` (documented in `src/Polysemy.hs:90`) is defined in the effect as `Catch :: m a -> (e -> m a) -> Error e m a`. To interpret this, the user calls `runT` to lift the first action into the Tactical environment, then `bindT` to lift the error handler. The `runT` function (line 145-154 in `src/Polysemy/Internal/Tactics.hs`) returns a `Sem (e ': r) (f a)` that can be sequenced with the handler.

For scoped operations like resource acquisition, the `Scoped` effect (line 106-108 in `src/Polysemy/Internal/Scoped.hs`) wraps a computation with a tag and returns control to the interpreter before and after execution. The interpreter (e.g., `interpretScopedH` at `src/Polysemy/Scoped.hs:46-62`) uses `withResource` to perform allocation, then invokes the user's handler with the acquired resource inside a `Tactical` environment.

### Openness approach

Polysemy achieves openness by making the effect row `r` a type parameter to `Sem`. A function generic over `r` can be called with any extension. For example, a function returning `Sem (State Int ': r) a` is polymorphic in the tail and can be composed with any effects the caller provides.

The implementation relies on type-level list machinery to maintain this openness. The `Member e r` constraint is satisfied by a GHC plugin (referenced at line 61 in `src/Polysemy/Internal.hs`: `import Polysemy.Internal.PluginLookup`) that assists with instance resolution. The plugin is not required but dramatically improves compile time by avoiding exponential backtracking during constraint solving.

New effects can be defined as GADTs (as shown in `src/Polysemy.hs:44-53`) with smart constructors generated via Template Haskell (`makeSem`). This is UX sugar; the substrate is the `send` function which injects effects into the Union at any position in the row, using the Member proof.

### Relevance to port-plan

No change needed. Polysemy confirms that option 1 (type-level nested coproduct with Peano-style indexing) is a mature, production-grade encoding for extensible effects in a strongly-typed functional language. Its innovations (Weaving, Tactical, Scoped) are implementation conveniences layered on top of the core option-1 machinery, not departures from it.

The one wrinkle is that Haskell can erase type-level proofs to integers and recover the right behavior via type erasure at call sites. Rust cannot do this; it must choose between (a) keeping indices as types (incurring monomorphisation cost as noted above), (b) accepting runtime tags (option 3), or (c) using a hybrid (option 4). The port-plan already acknowledges these trade-offs; polysemy does not introduce new considerations.

The Tactical and Scoped mechanisms are valuable and should inform the design of handler combinators and higher-order effect support in Rust, but they do not require changes to the row encoding decision itself.

### References

- `src/Polysemy/Internal.hs:214-220`: Sem type definition
- `src/Polysemy/Internal/Union.hs:65-72`: Union type definition
- `src/Polysemy/Internal/Union.hs:82-107`: Weaving type definition
- `src/Polysemy/Internal/Union.hs:148`: ElemOf type definition
- `src/Polysemy/Internal/Union.hs:198-206`: Member typeclass and instances
- `src/Polysemy/Internal/Tactics.hs:77-78`: Tactical type alias
- `src/Polysemy/Internal/Tactics.hs:145-216`: runT, bindT, pureT, getInitialStateT
- `src/Polysemy/Internal/Tactics.hs:235-254`: runTactics interpreter
- `src/Polysemy/Internal/Scoped.hs:106-109`: Scoped effect definition
- `src/Polysemy/Scoped.hs:46-62`: interpretScopedH implementation
- `src/Polysemy.hs:44-100`: Effect GADT convention and higher-order effect documentation
- `src/Polysemy/Internal.hs:61`: PluginLookup mention

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
