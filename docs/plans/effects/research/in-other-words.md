# in-other-words

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/in-other-words/`

## Purpose

Stage 1 research document: classify `in-other-words` against the five
effect-row encodings catalogued in [../decisions.md](../decisions.md)
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

The fundamental encoding is a first-class AST of effect operations, using a nested coproduct (sum type) indexed by membership proofs. The core type is `Union (r :: [Effect]) m a` (src/Control/Effect/Internal/Union.hs:42-43), which has two constructors: `Union :: Coercible z m => ElemOf e r -> e z a -> Union r m a`. This mirrors option 1 (nested coproduct with Peano indices) from the decisions. The `ElemOf e r` proof is a simple GADT with `Here` and `There` constructors (src/Control/Effect/Internal/Membership.hs:8-10), encoding the position in a type-level list. Programs are constrained by `Carrier m` (src/Control/Effect/Internal.hs:29), which is a typeclass associating a monad `m` with two type-family lists: `Derivs m` (derived effects) and `Prims m` (primitive higher-order effects). The `Carrier` class defines an algebra (`algDerivs`, line 96) which is a handler function of type `forall x. Union r m x -> m x` (src/Control/Effect/Internal/Union.hs:47). This is a first-class value representing the effect handler; programs are values (not functions), allowing them to be inspected, cloned (where types allow), and reinterpreted. There is no Free monad wrapping; the coproduct Union is the direct carrier of effect operations.

### Distinctive contribution relative to baseline

The key distinction is the separation of _derived_ from _primitive_ effects via the `Derivs / Prims` family pair (src/Control/Effect/Internal.hs:48 and 66). Derived effects are eventually expressed in terms of primitive effects via a `reformulate` function (line 81-82). This enables a "carrier abstraction" where the same derived effect (e.g., `Local` or `Ask` from `Reader`) can be compiled down to different primitive implementations (`ReaderPrim`, src/Control/Effect/Internal/Reader.hs:9-10) depending on the carrier. For scoped (higher-order) effects, this avoids the quadratic instance problem that afflicts polysemy and fused-effects: instead of needing one instance per (derived-effect, interpreter) pair, a handler only needs to exist for the primitive layer. Higher-order effects are handled via a "carrier thunk" pattern where the continuation (the rest of the program) is passed as a value of type `Effly z` (src/Control/Effect/Carrier/Internal/Interpret.hs, line 144-158), a wrapper that provides deferred access to effect capabilities. This is architecturally similar to what Heftia does with its evaluation function; it defers the final interpretation until the handler knows what monad it is targeting.

### Classification against decisions section 4.1

`in-other-words` is a clean instance of **option 1: Type-level heterogeneous list / nested coproduct with Peano indices**. The row is a type-level list encoded as a recursively-nested `Union`; membership is proved by a Peano-depth index (`Here` / `There<Here>` / `There<There<Here>>`). Unlike polysemy (which also uses option 1 but with a Van Laarhoven encoding and doesn't separate derived/primitive), in-other-words adds a carrier-based layer that makes the derived-to-primitive reformulation explicit and reusable. This is genuinely not novel from the decisions's perspective; it is a well-understood encoding. However, the _decomposition_ into derived + primitive + reformulate is the distinctive architectural contribution. The library sits between polysemy (option 1, no primitive distinction) and mtl (option 5, full typeclass dispatch). Because `Carrier` is a typeclass and `Eff e m` is defined as a constraint `(Member e (Derivs m), Carrier m)` (src/Control/Effect/Internal.hs:113 and 125), the library is _not_ mtl-style in the problematic sense: the program value is the `Union` coproduct, not a generic function, so multi-shot interpretation remains possible. The commitment to option 1 is clear and unambiguous.

### Scoped-operations handling (local, catch, and similar)

Higher-order / scoped operations are handled by passing the continuation as a _wrapped monad value_ rather than a bare function. When interpreting a scoped effect like `Local i :: m a -> Local i m a`, the handler receives `Local i (Effly z) a` where `z` is the continuation monad (src/Control/Effect/Carrier/Internal/Interpret.hs:155-158). The `Effly z` wrapper (src/Control/Effect/Internal/Effly.hs, not fully read but referenced) provides limited effect capabilities to the continuation within the handler context. For primitive scoped effects, the approach is different: the handler uses `Regional s :: Effect` (src/Control/Effect/Type/Regional.hs:54-55), a helper primitive effect that abstracts the ability to "hoist a natural transformation" over the monad, allowing access to control-flow operations. This avoids the complexity of Tactical in polysemy; the continuation is not a complex opaque handler but simply the next step of the program bound within a modified monad context. The `Reader` effect example shows how `Local` is reformulated from a derived effect into `ReaderPrim` operations (src/Control/Effect/Internal/Reader.hs:51-59), where the continuation `m` is passed to `ReaderPrimLocal` as a value.

### Openness approach

Openness is achieved via the same mechanism as option 1: a function polymorphic in a tail variable `R` can be called with any extension. For example, `fn foo<R>() -> Run<Coproduct<State, R>, ()>` in Rust terms maps to a Haskell function polymorphic in the tail of the effect list. The library uses the `Carrier` typeclass and the `Derivs m` / `Prims m` type families to track the effect sets statically. New effects are added by creating a new effect GADT (e.g., `data Ask i :: Effect where ...`) and then writing an interpreter that reformulates it in terms of existing effects or primitives. The core mechanism is `Member e (Derivs m)` resolution (src/Control/Effect/Internal/Membership.hs:36-45), which traverses the list to find the effect's position at compile time. The `send` function (src/Control/Effect/Internal.hs:141-142) uses this to inject an effect operation into the union and apply the algebra. There is no macro sugar for declaring the effect row; users write out the nesting explicitly, analogous to writing `Coproduct<..., Void>` in Rust rather than using a `coprod![...]` macro.

### Relevance to decisions

**Recommendation: No change needed to the decisions, but insights should inform the decision between options 1 and 2.**

The in-other-words architecture confirms that option 1 (Peano indices, nested coproduct) is viable and can be paired with sophisticated effect interpretation machinery. The derived / primitive / reformulate separation shows a path forward for Rust: the carrier abstraction can reduce the boilerplate of threading constraints across many monad transformers. However, the decisions already acknowledges option 2 (typenum indices) as a compile-time and error-message improvement over option 1, and this library does not engage with that question (it is Haskell-only and has no compile-time concerns analogous to Rust's).

The library's approach to scoped effects via a continuation monad plus helper primitives like Regional is instructive but not a blocker for Rust. The decisions's option 4 (hybrid: coproduct + macro sugar) already accommodates the machinery needed.

**No material change** to the five options or the Free-family commitment (section 4.4). The library reaffirms that nested coproducts work well for effect rows; the reformulate pattern is an implementation detail orthogonal to the row encoding choice.

### References

- `src/Control/Effect/Internal.hs:29-98`: Core `Carrier` typeclass with `Derivs`, `Prims`, `algPrims`, `reformulate` definitions.
- `src/Control/Effect/Internal/Union.hs:42-43`: `Union` data type definition.
- `src/Control/Effect/Internal/Membership.hs:8-10`: `ElemOf` GADT.
- `src/Control/Effect/Internal/Membership.hs:36-45`: `Member` constraint resolution.
- `src/Control/Effect/Internal.hs:113, 125`: `Eff` and `Effs` as constraint aliases.
- `src/Control/Effect/Internal.hs:141-142`: `send` function.
- `src/Control/Effect/Internal/Reader.hs:14-59`: `Ask`, `Local`, `ReaderPrim` effect definitions and `Reader` carrier example.
- `src/Control/Effect/Type/Regional.hs:54-55`: `Regional` helper primitive effect.
- `src/Control/Effect/Carrier/Internal/Interpret.hs:144-158`: Effect handler type `EffHandler` with continuation wrapping via `Effly`.
- `README.md`: High-level description of the library's approach to higher-order effects.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
