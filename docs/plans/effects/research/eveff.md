# EvEff

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/EvEff/`

## Purpose

Stage 1 research document: classify `EvEff` against the five effect-row
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

The fundamental encoding is a typed runtime evidence vector (linked list) paired with a Free-like continuation monad. The effect context `e` is encoded as a type-level list constructed from the `:*` cons operator and `()` nil (src/Control/Ev/Eff.hs:99, 121). At runtime, the `Context e` GADT (src/Control/Ev/Eff.hs:124-126) is a strongly-typed linked list: `CCons` nodes hold a `Marker` (unique prompt identifier), the actual handler value `h e' ans` (the "evidence"), a context transformer `ContextT`, and the tail. The `Eff` monad wraps a continuation that takes a `Context e -> Ctl a` (src/Control/Ev/Eff.hs:154), where `Ctl` is a multi-prompt control monad (src/Control/Ev/Ctl.hs:77-86). Operations do not pattern-match through a coproduct; instead, `perform` (src/Control/Ev/Eff.hs:312-315) uses the membership constraint `In h e` to linearly search the context and extract the handler by type equality (`HEqual` type family, src/Control/Ev/Eff.hs:263-265). The extracted handler's operation is then applied with the marker and context as arguments.

### Distinctive contribution relative to baseline

EvEff's core innovation is **evidence passing**: instead of reifying effects as a discriminated union (coproduct) whose interpretation depends on pattern-matching during execution, each effect handler is stored directly as a runtime value in a type-level context. When an operation is performed, the handler is retrieved by a typed lookup (recursively through the context list using type equality to find the right handler), not by explicitly dispatching on a sum type. This avoids boxing, tag inspection, and coproduct overhead at operation invocation time. The baseline Free + Coproduct + Member (Run-style) approach requires the handler to unpack and match on the coproduct at each `liftOp` or equivalent; EvEff eliminates that branching entirely by storing the handler directly. The result is a flatter, more cache-friendly dispatch at runtime while maintaining full type safety.

### Classification against decisions section 4.1

EvEff is **genuinely novel** and does not cleanly fit any of options 1-5. It shares surface structure with option 1 (type-level heterogeneous list of effects, via `:*` cons), but the dispatch mechanism is categorically different. Option 1 assumes a coproduct (nested sum) with Peano index traversal; EvEff does not use a coproduct at all. Instead, it is a **capability/evidence vector**: the type-level list is a structural description of the set of available handlers, and each handler value is stored directly in the runtime context. This is closer in spirit to option 3 (trait-object dynamic dispatch with tags) in that it uses runtime lookup, but EvEff's lookup is **typed and linear** (by type equality), not hash-based or tag-based. It is also incompatible with option 5 (MTL-style traits) because it does not rely on a trait per effect. The closest analogue is a hybrid of option 1's type-level list with option 3's runtime dispatch, but mediated by type equality rather than existential erasure. This design is the core contribution of the Xie-Leijen ICFP 2020 paper.

### Scoped-operations handling (`local`, `catch`, and similar)

Scoped operations use **multi-prompt delimited continuations** via the `Ctl` monad (src/Control/Ev/Ctl.hs). The `prompt` function (src/Control/Ev/Ctl.hs:133-136) installs a new prompt marker, and `yield` (src/Control/Ev/Ctl.hs:93-94) allows operations to yield to that marker with a resumption. Higher-order handlers (e.g., `operation` in src/Control/Ev/Eff.hs:346-349) receive a resumption function `k` that can be called multiple times, enabling backtracking, exception handling, and nondeterminism. The `handlerLocal` function (src/Control/Ev/Eff.hs:442-445) demonstrates scoped local state by pairing `local` (which creates an IORef-backed prompt context) with `handlerHide` (src/Control/Ev/Eff.hs:218-221) to isolate state from the outer computation. Examples in src/Examples.hs include the `allResults` handler for `Amb` (line 173-179), which backtracks by invoking the resumption twice, and `parse` (lines 229-238), which uses local state with multi-prompt semantics. The `guard` function (src/Control/Ev/Eff.hs:358-359) enforces that resumptions are called only in the correct handler context, preventing use-after-escape bugs.

### Openness approach

EvEff achieves extensibility through **algebraic effect definition**: users define custom effect types as data records (e.g., `Reader a e ans` in src/Examples.hs:18, `State a e ans` in line 109). Each field is an `Op` (operation) defined in terms of `value`, `function`, `operation`, or `except` (src/Control/Ev/Eff.hs:319-355). Handlers are ordinary Haskell values, instantiated via the `handler` combinator (src/Control/Ev/Eff.hs:184-187). New effects require no changes to the core library; they are ordinary datatype definitions. The type-level membership constraint `:?` (src/Control/Ev/Eff.hs:250-275) ensures that only effects in scope can be performed. This is fully compositional: effects can be stacked in any order (src/Examples.hs:195-198 shows `Exn :* Amb :* e`), and new effects can be defined without touching existing code. The design is equivalent in openness to Run / MTL but achieved through a different mechanism (type-level list + evidence lookup, rather than coproduct pattern-matching or constraint-based dispatch).

### Relevance to decisions

This design introduces a **sixth category deserving Stage 2 deep-dive**. While the type-level structure resembles option 1, the evidence-passing dispatch is a qualitatively different approach to effect row representation and operation invocation. Key implications for the decisions:

1. **Option 1 viability**: If Rust support for type-level heterogeneous lists and linear type equality unification is feasible, EvEff's approach could reduce dispatch overhead vs. nested coproduct traversal, offsetting the compile-time complexity.

2. **Runtime behavior**: Evidence passing trades static coproduct overhead for linear scan at each `perform`, plus direct handler value availability (no boxing/unboxing). In languages without multi-prompt delimited continuations (e.g., plain Rust without trampolines), the control substrate becomes critical.

3. **Scoped-operation encoding**: EvEff's reliance on `Ctl` (explicit yield/prompt) and IORef-backed local state suggests that Rust implementations would require explicit continuation reification or async/generator machinery to match the expressiveness. The safety properties (guard, context equality checking) are architectural, not linguistic.

**Recommendation**: Open a deep-dive on evidence passing as an alternative dispatch mechanism. Specifically, measure whether the linear typed lookup + direct handler invocation model is applicable to Rust and whether it offers performance or clarity benefits over a nested coproduct approach. This should inform the final choice in decisions section 4.1.

### References

- src/Control/Ev/Eff.hs: Core `Eff` monad, `Context` GADT, `:*` and `:?` operators, `perform` and `handler` combinators (lines 49-446).
- src/Control/Ev/Ctl.hs: Multi-prompt `Ctl` monad, `Marker`, `prompt`, `yield` (lines 14-172).
- src/Examples.hs: Concrete effect definitions (Reader, State, Exn, Amb) and usage patterns (lines 17-305).
- src/Control/Ev/Util.hs: Standard effects (Reader, State, Writer, Except, Choose) instantiated via library combinators (lines 35-172).

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
