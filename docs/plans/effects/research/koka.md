# koka

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/koka/`

## Purpose

Stage 1 research document: classify `koka` against the five effect-row
encodings catalogued in [../decisions.md](../decisions.md) section 4.1.
Koka has native row polymorphism, so the interesting question is not "is
this one of options 1-5" but "how does Koka lower its row types into a
runtime representation, and could that lowering strategy inform a Rust
encoding?". The second question is where Stage 2 might follow up.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

Koka's row types are lowered via evidence passing: handlers are not encoded
as syntactic continuations at the compiler level, but as _evidence_ (opaque
values) threaded through the program. The effect row is represented at the
type level as a right-associative `effectExtend` application:
`effectExtend(label1, effectExtend(label2, effectEmpty))` (src/Type/Type.hs:692-708).
Each label is a type constant, and the row is linearized into a flat list
via `extractHandledEffect` (src/Type/Kind.hs:69-72), which extracts only the
handlers that appear in evidence vectors. At runtime, evidence is stored in
a tagged vector `KK_TAG_EVV_VECTOR` in the context (kklib/include/kklib.h:72,442).
The transformation happens in the `Monadic` pass (src/Core/Monadic.hs) which
introduces monadic bindings for effectful code, and the `MonadicLift` pass
(src/Core/MonadicLift.hs) which lifts local functions to top level. The
`OpenResolve` pass (src/Core/OpenResolve.hs:110-204) inserts runtime calls
to `open` functions that load the evidence vector; the indices are computed
by `evIndexOf` and assembled into a vector via `makeVector typeEvIndex`
(src/Core/OpenResolve.hs:164-170, 199).

### Distinctive contribution relative to baseline

Unlike a Free + Coproduct + Member baseline (which builds the effect type
structurally at the value level), Koka maintains _type-level row polymorphism
throughout compilation_. The key innovation is that row types are not
desugared to concrete types until the `OpenResolve` pass, which is relatively
late in the pipeline. Rather than storing the entire row structure at
runtime, Koka extracts only the _handled_ effects (those providing evidence)
and discards the rest. The evidence vector is thus a linear array of indices
and handler pointers, not a nested coproduct. This allows pure code (code
not using operations) to have no runtime evidence at all. The row system
also tracks scoped handler nesting: duplicate labels are tagged with a mask
level to support multiple handlers for the same effect (addLevels in
src/Core/OpenResolve.hs:208-218). This is more sophisticated than typical
Free implementations, which cannot express nested scopes for the same effect
without additional machinery.

### Classification against decisions section 4.1

Koka's lowering does not map cleanly to options 1-5. It is closest to
Option 3 (TypeId-like dispatch) but inverted: rather than tagging values
with runtime type identifiers, it pre-computes effect indices at compile
time and uses them to index into a fixed vector. The row type itself
(at the language level) is a genuine row: unordered, polymorphic tail,
extensible. But its lowering is neither a heterogeneous list (Option 1),
binary-indexed sum (Option 2), dynamic TypeId dispatch (Option 3), nor
macro sugar (Option 4). Instead, the _row is erased during compilation
and replaced with a pre-computed vector of evidence indices_. This is
genuinely novel relative to the decisions's baseline, and suggests a
sixth option: row-polymorphic source, but eager linearization to a
flat index vector at compile time, with no runtime row structure.

### Scoped-operations handling (`local`, `catch`, and similar)

Koka's standard library defines effect handlers as values (test/algeff/diverge.kk:12-13).
A handler is declared inline with `handler` syntax and bound to a name.
Scoped operations (e.g., `ctl` for control operations that can resume) are
declared as operations within an `effect` block (test/algeff/diverge.kk:5-6).
The handler uses `resume` to invoke the resumption; at the type level,
this is checked against the operation's resume mode (ResumeOnce vs ResumeMany,
src/Type/Kind.hs:85-91). At runtime, scoped handler nesting is managed via
mask levels, which allow the same effect to be installed multiple times in
the evidence vector (addLevels in src/Core/OpenResolve.hs:208-218). The
mechanism is subtle: when a handler is applied via `open`, the evidence
vector is passed to the handler as part of the semantic value, allowing
the handler to recurse or re-invoke the handled code with the previous
evidence restored.

### Openness approach

Extensibility to new effects is achieved at the _language level_ via row
polymorphism: a function can accept a `<e, ...>` effect type, where `...`
is a tail variable. The row is "open" (polymorphic) at the type level.
At the compiler level, extensibility is handled during the `openResolve`
pass (src/Core/OpenResolve.hs:141-143): when an effect from `effFrom` is
not fully resolved to `effTo`, an `open` wrapper is inserted. The wrapper
is selected by the arity and count of handled effects (nameOpenAt, nameOpen,
nameOpenNone in src/Core/OpenResolve.hs:187-203). Once lowered to indices,
the system cannot dynamically add new effects; instead, all effects must
be statically known by the type inference stage. New effects are integrated
by re-compiling code that uses them, not by runtime dispatch. This is
closure: the compiler fully resolves the effect row before code generation.

### Rust portability assessment

Koka's lowering strategy is _partially_ portable to Rust, but faces two blockers.
First, Koka relies on multi-prompt delimited continuations for the runtime
semantics of scoped operations (the `ctl` construct and resumption points).
The decisions (section 1.2) has ruled out delimited continuations as a
substrate, so Koka's scoped-operation model cannot be ported directly.
Second, the evidence vector stores pointers to closures and handler tables;
in Rust, this would require dynamic allocation or unsafe pointer arithmetic,
and the Brand/Kind machinery does not currently support this pattern.
However, the indexing strategy (pre-computed effect indices into a flat
array, computed at compile time) is directly applicable and portable.
A Rust port could use Option 3 or 4 (TypeId dispatch or macro sugar) and
apply Koka's compile-time index-generation strategy to assign stable,
predictable indices to effects in place of dynamic type IDs. This would
reduce runtime overhead and improve cache locality. Cost: moderate;
requires extending the type-inference pass to emit index assignments and
modifying the code-generation backend to use them. Feasibility: high for
the indexing strategy alone, but low for the full handler system without
delimited continuations.

### Relevance to decisions

Findings suggest a minor revision to decisions section 4.1. The indexing
strategy (pre-computed effect indices, flat vector at runtime, scoped
nesting via mask levels) is novel and worth considering as Option 6 or as
a hybrid with Option 3. Concretely: if the port uses TypeId or macro-sugar
dispatch, incorporating Koka's compile-time index assignment would improve
performance and eliminate the need for runtime type inspection. However, the
scoped-operation model (multi-prompt continuations) cannot be ported without
a different foundation (e.g., shift/reset or algebraic effects with resumptions
in a different form), so the full handler-with-resumption system is not
portable to a Free-monad-based Rust port. Recommendation: create a Stage 2
deep dive on the index-generation pass (src/Core/OpenResolve.hs) to assess
whether the strategy can be adapted to Option 3 or 4 in Rust. No change to
the current decisions direction is required at this stage.

### References

- src/Type/Type.hs:692-708: effectExtend and effectExtends (row encoding)
- src/Type/Kind.hs:69-72: extractHandledEffect (linearization of effects into evidence vector)
- src/Core/Monadic.hs: monTransform (monadic binding transformation)
- src/Core/MonadicLift.hs: monadicLift (lifting of local functions)
- src/Core/OpenResolve.hs:110-204: resOpen (insertion of open calls and index computation)
- src/Core/OpenResolve.hs:164-170: evIndexOf (index computation for evidence vector)
- src/Core/OpenResolve.hs:208-218: addLevels (mask-level computation for scoped handlers)
- kklib/include/kklib.h:72,442: evidence vector runtime representation (KK_TAG_EVV_VECTOR)
- test/algeff/diverge.kk:5-13: handler and scoped operation declarations

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
