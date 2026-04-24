# fx-rs

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/fx-rs/`

## Purpose

Stage 1 research document: classify `fx-rs` against the five effect-row
encodings catalogued in [../port-plan.md](../port-plan.md) section 4.1.
This codebase was mentioned in an earlier ecosystem survey but has not yet
been characterised in the plan; this research places it in the
classification space for the first time.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

fx-rs does not use a Free monad. Instead, it implements a simplified algebraic-effects system using a custom `Eff` enum that is either `Immediate(S, V)` or `Pending(Box<dyn Ability<S, S, V>>)`, stored inside a wrapper type `Fx<S, V>` (crates/fx/src/kernel/fx.rs:4-10). The runtime is evaluative: `Fx::eval()` pattern-matches on the enum, unwraps immediate values, and applies pending abilities to the unit value `()` (crates/fx/src/kernel/fx.rs:12-22).

Effects are parameterized by a _requirement_ type `S` (the "ability set") and a return value `V`. An `Ability` is a trait (crates/fx/src/kernel/ability.rs:6-13) whose `apply` method has signature `I -> Fx<S, O>`, expressing "given input I, produce an effect requiring S and returning O". The system is structured around "evidence passing": the requirement type `S` is carried through the computation and must be provided explicitly before evaluation.

Multi-effect composition happens through the `Has<T>` trait (crates/fx/src/core/has_put.rs:3-8), which proves "type S contains a value of type T and can extract it". Tuples and user-defined structs that implement `Has<T> + Put<T, U>` form the effect environment. There is no coproduct or nested enumeration; the effect set is represented as a heterogeneous struct or tuple whose member types encode which effects are available (crates/fx/src/core/tests/fx_test.rs:7-36 shows a struct `S` implementing `Has<A>` and `Has<B>` for two effects).

### Distinctive contribution relative to baseline

fx-rs is fundamentally different from Free + coproduct designs. It abandons the AST model entirely: there is no tree of bind nodes, no continuation capture, and no multi-shot evaluation. Instead, it embraces a simpler "immediate evaluation with deferred provider" model. Programs are not values; they are lazy placeholders that wait for their requirement `S` to be provided. Once provided, they evaluate immediately.

The effect environment is unordered: `S` is just a type that implements the necessary `Has` traits. This avoids row-ordering problems inherent to nested coproducts. The user builds effects by stacking `Has` implementations on a struct, and the compiler automatically infers which effects are available through trait resolution.

This comes at a cost: there is no exhaustiveness checking, no AST inspection, and no multi-shot continuation support. Handlers are simple functions `Fx<S, V> -> Fx<T, V>` that transform requirements, not complex transformations over an effect tree.

### Classification against port-plan section 4.1

fx-rs is a novel design that does not fit neatly into options 1-5. The closest conceptual match is _option 5 (trait-bound set)_, except that fx-rs uses concrete struct member types rather than Rust trait bounds. Like option 5, it is fundamentally non-Free, preserves openness via type parameters, and avoids row-ordering issues. Unlike option 5, it does not store effects in method-call form; it wraps them in the `Fx` type and delays evaluation.

It is not option 3 (trait-objects with TypeId) because there is no `Box<dyn Any>` or runtime downcast.

It is not options 1, 2, or 4 (coproduct-based) because there is no nested sum type or indexing machinery.

The distinguishing novelty is the replacement of both Free-monad AST and trait-bound constraints with a tuple/struct-based environment type that is traversed lazily at evaluation time via evidence-passing (Has trait resolution).

### Scoped-operations handling (`local`, `catch`, and similar)

fx-rs supports scoped operations through the `lift` and `lift_map` methods (crates/fx/src/core/fx.rs:93-110). `lift<T>` converts an effect `Fx<S, V>` into `Fx<T, V>` where `T: HasPut<S, T>`, allowing the effect to be "lifted" into a larger context. `lift_map` composes lifted effects with a monadic continuation function, enabling sequential chaining of effects with different requirement scopes.

The `adapt` method (crates/fx/src/kernel/fx.rs:36-50) is the primitive for scoped transformations; it takes a "contravariant map" (getter/setter pair) and applies it to requirements and results. This supports effect-local state manipulation and scope-bounded operations, though the documentation does not showcase exception handling or try-catch semantics explicitly.

### Openness approach

Openness is achieved through the `S` type parameter in `Fx<S, V>`. A function generic over `S` can be called with any struct that provides the needed evidence (implements `Has<E>` for each required effect E). New effects are added by extending `S` with additional `Has` implementations or by nesting structs; no code changes to existing functions are needed (crates/fx/src/core/tests/fx_test.rs:142-151 shows how two separate effects are lifted and combined).

The `Abilities` type and macros (`abilities_macro`, `field_macro`, `builder_macro` in crates/) provide ergonomic wrappers for defining effect-specific APIs and deriving evidence implementations, but the core mechanism is trait-based evidence passing, not type-level lists or macros.

### Relevance to port-plan

fx-rs is a significant conceptual departure from the five enumerated options and suggests that the port-plan's commitment to the Free-monad family may be too restrictive. However, fx-rs also forecloses on multi-shot continuations (a stated requirement for `Choose` support), exhaustiveness checking, and first-class program values. For the rust-fp-lib port, which explicitly targets Run-style free-monad semantics, fx-rs does not constitute a viable alternative encoding for the row.

The **only material relevance** is architectural humility: if the Free + coproduct approach proves unmanageable, option 5 (trait-bound sets) and fx-rs's evidence-passing hybrid represent the next frontier, trading AST composability for simpler type-level machinery. No change to the port-plan's direction is recommended at this stage.

### References

- crates/fx/src/kernel/fx.rs:1-52 (Eff enum, Fx wrapper, eval logic)
- crates/fx/src/kernel/ability.rs:1-29 (Ability trait definition)
- crates/fx/src/core/has_put.rs:1-34 (Has, Put, HasPut evidence traits)
- crates/fx/src/core/fx.rs:7-110 (map, bind, lift, and lift_map implementations)
- crates/fx/src/core/tests/fx_test.rs:7-51 (struct-based effect environment with Has implementations)
- book/src/concepts.md:1-37 (high-level effect/ability/handler model)
- book/src/macros.md:1-110 (macro-based ergonomics for effect definition and provision)

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
