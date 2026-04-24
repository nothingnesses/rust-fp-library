# freer-simple

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/freer-simple/`

## Purpose

Stage 1 research document: classify `freer-simple` against the five
effect-row encodings catalogued in [../port-plan.md](../port-plan.md)
section 4.1. Identify whether this codebase is a variant of an existing
option or represents a genuinely novel encoding worth deeper investigation
in Stage 2.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

`freer-simple` uses a Free monad over an open union (type-indexed coproduct)
of effect functors. The `Eff` type is defined at `src/Control/Monad/Freer/Internal.hs:116-121`
as a sum of two constructors: `Val a` for pure values, and `E (Union effs b) (Arrs effs b a)`
for suspended effects. The `Union` type at `src/Data/OpenUnion/Internal.hs:40-41` is
an existential pair: a runtime `Word` tag (position index) plus a value.
Membership is proved at compile time by the `Member` trait (`src/Data/OpenUnion/Internal.hs:132`)
which resolves to a `Word` position via `FindElem` (`src/Data/OpenUnion/Internal.hs:80-96`).
The tail of effect calls is stored in an `Arrs` (type-aligned fast queue) at lines
84 and 90. The monad instance (`src/Control/Monad/Freer/Internal.hs:151-154`) binds by
appending continuations to the queue.

### Distinctive contribution relative to baseline

The primary distinction is `freer-simple`'s use of a **fast type-aligned continuation queue (FTCQueue)**
to store the tail of the program as a rope of pending function calls, rather than
allocating a new heap node per bind. The queue is defined at `src/Data/FTCQueue.hs:33-59`
as a binary tree of function applications. This is an optimization for stack-safety
and allocation efficiency; the fundamental Free + Union + Member structure is standard.
Second, the codebase achieves constant-time injection and projection (`src/Data/OpenUnion/Internal.hs:51-69`)
by storing a `Word` index directly in the Union constructor and using `unsafeCoerce` to
treat all payloads as the same size at runtime. This is a known approach, not novel.

### Classification against port-plan section 4.1

`freer-simple` is a direct instance of **Option 1: Type-level heterogeneous list / nested coproduct**
with Peano indices. The effect row is represented as a nested type: `Eff (Reader e ': State s ': []) a`
expands to a coproduct-shaped list. Membership is proved via `FindElem`, which recurses
through the type list and produces a Peano-styled index (plain `Word` at runtime, but
the proof is erased). The index resolution at `src/Data/OpenUnion/Internal.hs:80-96` uses
a classic overlappable instance chain: base case when the effect is at the head, recursive
case when it is not. This is textbook option 1. The row-ordering problem is present: the
same set of effects in different orders produces different types that require explicit
conversion machinery to unify (though freer-simple does not expose coproduct embedding
traits as heavily as frunk does; users compose effects via handler combinators instead).

### Scoped-operations handling (`local`, `catch`, and similar)

`freer-simple` implements scoped operations via **interposition**, not higher-order effects.
The `local` combinator for Reader (`src/Control/Monad/Freer/Reader.hs:62-69`) uses `asks`
to fetch a modified environment, then calls `interpose` to temporarily override the effect's
behavior for the duration of the continuation. The `interpose` primitive (`src/Control/Monad/Freer/Internal.hs:310-325`)
is a handler that sits "in the middle" of the effect stack: it inspects passing effects,
optionally intercepts a specific effect type, and relays others. The Error effect's `catchError`
(`src/Control/Monad/Freer/Error.hs:40-46`) follows the same pattern, using `interposeWith`
to install an exception handler that survives the rest of the computation. These are **not**
higher-order effects in the sense of effects that manipulate continuations; they are library-level
abstractions built from first-order operations. The library does not support true scoped effects
like delimited continuations or the `ask`/`local` that unwinds the stack. Confirmation: the
codebase includes no `shift`, `reset`, `callCC`, or multi-shot choice combinator; `Choose`
or `Coroutine` must be implemented by users via interpret/reinterpret chains.

### Openness approach

Openness is achieved via **type-level effect-set parameters**. A function is polymorphic over
the tail of the effect list using the `Member` constraint. For example, `ask :: Member (Reader r) effs => Eff effs r`
(`src/Control/Monad/Freer/Reader.hs:41-42`) works with any `effs` that contains `Reader r` somewhere,
allowing the caller to extend the effect set with additional effects. Handler combinators like
`interpret` and `reinterpret` (`src/Control/Monad/Freer.hs:266, 284`) thread the remaining effects
through the type, so a handler can be stacked with others. New effects are defined as plain GADTs
(no special infrastructure required); the open union automatically accommodates them as long as
the user provides a handler. This is the baseline freer design; openness is "built in" by parametricity.

### Relevance to port-plan

No change needed. `freer-simple` confirms that option 1 (Peano-indexed nested coproduct)
is a well-proven design with real-world adoption and good ergonomics when paired with
handler-combinator libraries. The stack-safe tail queue is an implementation detail that
does not affect the row encoding choice. The absence of higher-order scoped effects (true
delimited continuations) is a deliberate design trade-off, not a limitation of the encoding;
the port-plan section 4.4 already commits to Free family and handler composability,
which freer-simple demonstrates. The plan's note at section 4.1 that "freer-simple" and
"polysemy" are real-world references for option 1 is validated.

### References

- `src/Control/Monad/Freer/Internal.hs` lines 116-121 (Eff data type).
- `src/Data/OpenUnion/Internal.hs` lines 40-41 (Union definition).
- `src/Data/OpenUnion/Internal.hs` lines 80-96 (FindElem and Member traits).
- `src/Data/OpenUnion/Internal.hs` lines 51-69 (unsafe injection/projection).
- `src/Data/FTCQueue.hs` lines 33-59 (type-aligned queue definition).
- `src/Control/Monad/Freer.hs` lines 266, 284 (interpret and reinterpret handlers).
- `src/Control/Monad/Freer/Reader.hs` lines 41-42, 62-69 (Reader and local).
- `src/Control/Monad/Freer/Error.hs` lines 40-46 (catchError via interposition).
- `src/Control/Monad/Freer/Internal.hs` lines 310-325 (interpose primitive).

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
