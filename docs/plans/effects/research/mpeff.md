# MpEff

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/MpEff/`

## Purpose

Stage 1 research document: classify `MpEff` against the five effect-row
encodings catalogued in [../decisions.md](../decisions.md) section 4.1.
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

MpEff implements the Generalized Evidence Passing semantics (Xie & Leijen 2021) atop multi-prompt delimited continuations. The core type is `Eff e a` (src/Control/Mp/Eff.hs:173), a newtype wrapping `Context e -> Ctl e a`, where `Context e` is a linked-list structure holding the evidence for effect handlers, and `Ctl e a` is the control monad (lines 166-170). The effect row `e` is encoded as a type-level inductive list using the infix `:*` operator (line 137): `Context (h :* e)` contains a handler of type `h e' ans` linked to a tail `Context e`. Handler lookup is performed by the `In h e` constraint (lines 308-325), which recursively walks the context until finding a matching effect type `h`, using `HEqual` type family (lines 314-316) to discriminate on exact type match. This is evidence passing because handlers are indexed at runtime by a unique `Marker h e a` tag (lines 107-116) rather than stored in a coproduct or typeclass; dispatch is by marker unification at the prompt, not by static position.

### Distinctive contribution relative to baseline

MpEff differs fundamentally from `Free + Coproduct + Member` in that it abandons the free monad altogether in favor of delimited continuations. Instead of building an AST, computations directly yield control to a prompt (line 218-219 `yield` function), passing a handler operation and a continuation. The program is not a reifiable data structure; it is a control-flow graph whose branches are determined by where prompts lie. Scoped operations like `local` and `catch` (lines 79-82) become native control features rather than requiring additional abstraction (HFunctor, Tactical). Evidence passing replaces member indices: instead of proving "effect X is at position N in the coproduct", MpEff proves "the effect context contains a handler of type X" via the `In` constraint, then recovers the actual handler value at runtime via marker match (line 265 `mmatch`). This is a more direct correspondence to how handlers are actually used than a statically-indexed position would be.

### Classification against decisions section 4.1

MpEff is not a direct fit for any single option 1-5; it represents a genuinely distinct approach that cannot be implemented with option 1's nested coproduct, option 2's typenum indices, option 3's trait-object dispatch, option 4's macro-sugar coproduct, or option 5's trait-bound set. The closest superficial analogue is option 1 in that both use a type-level list (`:*` vs. nested `Coproduct`), but the similarity ends there. Option 1 builds a static type-indexed data structure amenable to per-position trait resolution; MpEff discards the static index and uses runtime marker identity instead. This is closer in spirit to option 3's dynamic dispatch (`TypeId` downcasting), but with two key differences: (a) MpEff tracks the row type statically (the `Context e` type depends on the full row), preserving first-class programs and exhaustiveness checking, and (b) MpEff's marker matching is bidirectional (handler saves its marker; operation yields with a marker; both must match), not unidirectional tag-to-implementation lookup. If forced to classify, MpEff is a hybrid of option 1's static row tracking and option 3's dynamic dispatch semantics, unified under a control-flow abstraction rather than either list-machinery or trait-object machinery.

### Scoped-operations handling (`local`, `catch`, and similar)

Scoped operations are native to MpEff's multi-prompt control model. The key mechanism is that prompts save and restore context across resumptions. The `local` function (lines 459-461) wraps an operation in a built-in `Local a` effect that uses an IORef to save/restore state on yield/resume. More generally, `prompt` (lines 256-266) automatically propagates continuation extensions; each resumption passes through the same prompt, which re-applies the context transformer. Thus `mask` (lines 296-297) simply removes the top handler from the context before executing the body, and any handler defined with `handler` (lines 269-271) automatically supports scoped semantics because `prompt` is itself scoped (line 262: each resumption re-establishes the prompt). This is vastly simpler than polysemy's Tactical machinery or EvEff's HFunctor approach because the control abstraction already captures context; no extra monad transformer or effect type is needed.

### Openness approach

MpEff achieves extensibility by leaving the type-level row `e` universally quantified in the signatures of handler combinators. A function can be written as `foo :: (In h e) => Eff e a -> Eff e a` to work with any effect row containing `h`; callers can pass additional effects in the tail, and the handler works transparently. Effect definitions are open user-defined datatypes (e.g., `data Reader a e ans = Reader { ask :: Op () a e ans }` in src/Control/Mp/Util.hs:39); each effect is a record of operations. New effects are added by writing a new record type and a handler function (e.g., `reader` in src/Control/Mp/Util.hs:44-46), with no modification to the library core. The `VariantF` analogue is implicit: operations are not values in a sum; they are fields of a handler record. This trades explicit sum construction for implicit record-field dispatch.

### Relevance to decisions

No change needed. MpEff is a reference point for understanding how evidence passing can be implemented, but it cannot be directly ported to Rust because its core depends on Haskell's `prompt#` and `control0#` RTS primitives. Section 1.2 of the decisions already rules out delimited continuations as a substrate. However, MpEff's proof that evidence passing yields clean scoped-operation semantics is valuable context for evaluating the four primary row encodings (options 1, 2, 4 under consideration). If a stage 2 deep-dive were to compare how each Rust option handles `local`/`catch`/`mask`, MpEff's simplicity should be the aspirational target; the presence of extra boilerplate in option 1 or option 4 would signal that the encoding is fighting against the semantics.

### References

- src/Control/Mp/Eff.hs:107-116, Marker type definition and mmatch unification.
- src/Control/Mp/Eff.hs:137-145, Context inductive list structure (:\* constructor).
- src/Control/Mp/Eff.hs:166-170, Ctl control monad type.
- src/Control/Mp/Eff.hs:173, Eff newtype wrapping the control monad.
- src/Control/Mp/Eff.hs:217-219, yield function for raising to a prompt.
- src/Control/Mp/Eff.hs:256-267, prompt function implementing scoped control.
- src/Control/Mp/Eff.hs:308-325, In constraint and HEqual type family for handler lookup.
- src/Control/Mp/Eff.hs:345-349, perform function dispatching to handler via In constraint.
- src/Control/Mp/Eff.hs:459-461, local function for scoped state.
- src/Control/Mp/Eff.hs:296-297, mask function removing top handler.
- src/Control/Mp/Util.hs:39-46, Reader effect definition and reader handler example.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
