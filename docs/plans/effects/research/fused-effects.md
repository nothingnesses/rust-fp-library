# fused-effects

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/fused-effects/`

## Purpose

Stage 1 research document: classify `fused-effects` against the five
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

The core substrate is a coproduct of effect functors combined with the
`Algebra` typeclass, not a Free monad. Each effect (e.g., `Reader`, `State`,
`Writer`) is defined as a GADT functor over `(m :: Type -> Type)` and a
return type `k`. Effects are combined using the `:+:` operator (a
right-nested coproduct; see `src/Control/Effect/Sum.hs:18-28`). The `Algebra`
class (src/Control/Algebra.hs:74) defines how a signature of effects is
interpreted into a carrier monad `m`. Programs are not first-class AST values;
instead, a program is a generic Haskell function that is polymorphic over the
carrier monad, constrained by `Has eff sig m` (src/Control/Algebra.hs:133).
The carrier is instantiated only when effects are run; until then, the
computation lives entirely as a generic function awaiting a concrete
interpreter.

### Distinctive contribution relative to baseline

The key distinctive feature is the absence of any intermediate Free monad
layer. Rather than building `Free (VariantF sig) a` and then interpreting it,
fused-effects directly dispatches effects through the `Algebra` typeclass
instance. The Haskell documentation (docs/overview.md:37-45) explicitly states:
"there is no representation of the computation as a free monad." Computations
are performed in a carrier type specific to the selected handler; the fusion
happens at compile time via GHC's eagerness to inline typeclass instances.
This avoids the overhead of constructing and evaluating intermediate free
structures. Each effect definition includes both first-order operations (like
`State.Get`) and higher-order/scoped operations in the same GADT
(e.g., `Reader.Local`), which receive different Algebra interpretations but
are not syntactically distinguished from ordinary effects.

### Classification against port-plan section 4.1

Fused-effects is a variant of **Option 5 (trait-bound set / mtl-style)** with
a crucial hybrid twist. On the surface, programs are written as generic
functions `forall m. Has eff sig m => m a`, which is pure mtl-style: no
first-class program value, just a polymorphic function. However, the row is
encoded as an explicit coproduct `sig = eff1 :+: eff2 :+: ... :+: ()`, which
gives it a type-level structure analogous to Option 1. The `Member` constraint
(src/Control/Effect/Sum.hs:40-67) provides injection and projection machinery
over this coproduct. Thus fused-effects is **Option 5 with a Option 1 substrate
underneath**, but crucially without Free: the coproduct is not wrapped in a
monad, and the program is not a data structure. This makes it unsuitable for
porting to Rust as-is if the port must satisfy the Free-family commitment in
port-plan section 4.4 (programs must be first-class values for multi-shot
interpretation and handler composition via `peel`/`send`).

### Scoped-operations handling (`local`, `catch`, and similar)

Scoped operations are integrated directly into effect GADTs as additional
constructors. For example, `Reader` (src/Control/Effect/Reader/Internal.hs:7-9)
includes both `Ask` and the scoped operation `Local`. Both are interpreted by
the same `Algebra (Reader r) ((->) r)` instance (src/Control/Algebra.hs:167-171).
The `Local` case chains the handler through a locally-modified environment.
Higher-order effects like `Catch` in `Error` (visible in
src/Control/Algebra.hs:162-165) are handled by the Algebra instance, which
receives the nested action `m` and unwraps it using the provided `Handler`. No
separate HFunctor machinery is required; the GADT's type-level structure
encodes the higher-order nature, and the Algebra's `Handler` parameter
(src/Control/Algebra.hs:94-95) provides the means to lower nested actions.

### Openness approach

Extensibility to new effects is achieved via the coproduct `:+:` operator, which
allows any function to remain generic over a tail type parameter `R` in a call
like `Coproduct<State, R>`. Adding a new effect to an existing program requires
the function to declare the new effect in its `Has` constraint, and the caller
must provide a carrier that handles all declared effects. This is identical to
Option 1's openness story: functions are polymorphic in the tail of the row.
However, because programs are not first-class values, handler composition is
fundamentally limited compared to Option 1 / Free approaches. You cannot write
code that takes one program and passes it to multiple different carriers in
sequence; instead, you run the entire computation end-to-end with a chosen
carrier stack.

### Relevance to port-plan

Fused-effects reveals a critical tension: the port-plan's Free-family
commitment (section 4.4) requires first-class programs to enable multi-shot
interpretation and dynamic handler composition. Fused-effects explicitly
rejects Free and uses direct dispatch instead. If the Rust port is to support
the Free-family semantics (multi-shot, replayable computations), then
fused-effects' architecture, while elegant and efficient, does not serve as a
direct model. The coproduct encoding (Option 1) can be copied verbatim, but the
dispatch mechanism must be wrapped in Free or an equivalent AST layer. This is
not a drawback of fused-effects; it is the intended design choice for Haskell's
runtime. For Rust, it means the port should prioritize Option 1 or Option 2
with Free as the substrate, and use fused-effects' Algebra pattern as
inspiration for handler combinators, not as the core execution model.

### References

- `src/Control/Algebra.hs:74`: Algebra typeclass definition.
- `src/Control/Algebra.hs:133`: Has constraint (Members + Algebra).
- `src/Control/Effect/Sum.hs:18-28`: `:+:` coproduct definition.
- `src/Control/Effect/Sum.hs:40-67`: Member constraint and instance hierarchy.
- `src/Control/Effect/Reader/Internal.hs:7-9`: Reader GADT with Ask and Local.
- `src/Control/Effect/State/Internal.hs`: State GADT (Get, Put).
- `src/Control/Algebra.hs:162-171`: Example Algebra instances for Error and Reader.
- `docs/overview.md:37-45`: Explicit statement of no Free monad, fusion via inlining.
- `examples/Teletype.hs:48-52`: Program written as generic function with Has constraint.
- `examples/Teletype.hs:58-62`: Carrier type with Algebra instance; no program value.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
