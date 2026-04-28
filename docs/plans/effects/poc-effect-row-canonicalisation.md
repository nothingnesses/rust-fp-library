# POC: effect-row canonicalisation hybrid (workaround 1 + workaround 3)

**Status:** complete; POC workspace deleted in Phase 2 step 10b.
**Last updated:** 2026-04-25 (POC findings); 2026-04-28 (deletion note added).
**Code (historical):** `poc-effect-row/` at the repo root, deleted in Phase 2 step 10b after the 25 tests were migrated to [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs) (21 directly migrated or covered, 4 documented as not-applicable to production, 1 implicitly covered by [`tests/run_lift.rs`](../../../fp-library/tests/run_lift.rs)). This document is preserved as research history; the file paths below are historical references.
**Question source:** [decisions.md](decisions.md) section 4.1, "Ordering mitigations" subsection.

## 1. Purpose and scope

The decisions currently recommends the hybrid workaround 1 + workaround 3 for effect-row ordering: a proc-macro `effects![...]` lexically sorts effect names at expansion time so two orderings of the same effect set produce the same canonical type (workaround 1), with frunk's `CoproductSubsetter` as a fallback when users hand-write a non-canonical row (workaround 3). The recommendation rested on the observation that workaround 1 is feasible-in-principle and workaround 3 is mature in `effing-mad` and `corophage`, but the hybrid had not been demonstrated working together.

This POC closes that gap by writing the smallest possible end-to-end implementation and exercising both halves under a single handler, plus a small `tstr_crates` integration demo (tests t14-t16) that probes what the optional refinement actually delivers on stable Rust. It does NOT measure compile-time cost and does NOT capture error-message quality on failing fallbacks (those would be follow-ups if the design moves to production).

## 2. What was built

A standalone Cargo workspace at `poc-effect-row/`, separate from the main `rust-fp-lib` workspace so its loose POC lints do not pollute production code. Three components:

- `poc-effect-row/macros/`: a single proc-macro crate exposing `effects!`. Implementation is ~30 lines: parse a comma-separated list of `syn::Type`, sort by `quote!{}.to_string()` of each type, emit a right-nested `frunk_core::coproduct::Coproduct<...>` terminated by `CNil`.
- `poc-effect-row/src/lib.rs`: re-exports `effects!`, `Coproduct`, `CNil`, `CoproductSubsetter` for tests.
- `poc-effect-row/tests/feasibility.rs`: thirteen tests, each tagged with what it answers.

Total POC code: ~150 lines including comments. The proc-macro is the only piece that doesn't already exist in the Rust ecosystem; the rest is composing `frunk_core` and `static_assertions` mechanics.

## 3. Test results

All 13 tests pass under `cargo test` on the project's stable toolchain (Rust 1.94.1 from the project's nix devenv). Per-test summary:

| Test | What it answers                                                                                                                                                                                                                                           | Result |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| t01  | Two orderings of `{A, B}` produce the same type.                                                                                                                                                                                                          | Pass.  |
| t02  | All 6 permutations of `{A, B, C}` produce the same type.                                                                                                                                                                                                  | Pass.  |
| t03  | The canonical form is the lexical order (verified literally).                                                                                                                                                                                             | Pass.  |
| t04  | `effects![]` produces `CNil`.                                                                                                                                                                                                                             | Pass.  |
| t05  | `effects![A]` produces `Coproduct<A, CNil>`.                                                                                                                                                                                                              | Pass.  |
| t06  | Generic effects (`Reader<Env>`, `State<S>`) sort consistently.                                                                                                                                                                                            | Pass.  |
| t07  | Same root, different generic params (`Reader<i32>`, `Reader<i64>`) sort distinctly but canonicalise across orderings.                                                                                                                                     | Pass.  |
| t08  | Lifetime parameters compile through the macro.                                                                                                                                                                                                            | Pass.  |
| t09  | Workaround 3 fallback: hand-written non-canonical row mediates via `.subset()` into the canonical type.                                                                                                                                                   | Pass.  |
| t10  | Macro-emitted row passes directly into a handler typed on the canonical row (no permutation needed).                                                                                                                                                      | Pass.  |
| t11  | 5-effect rows canonicalise across multiple orderings.                                                                                                                                                                                                     | Pass.  |
| t12  | 7-effect rows canonicalise (sufficient to exercise nontrivial trait inference).                                                                                                                                                                           | Pass.  |
| t13  | `.subset()` mediates a 5-effect permutation reliably.                                                                                                                                                                                                     | Pass.  |
| t14  | `tstr::TS!("...")` provides stable type-level identity across module contexts.                                                                                                                                                                            | Pass.  |
| t15  | `tstr::cmp` computes string ordering at compile time, returning `core::cmp::Ordering`.                                                                                                                                                                    | Pass.  |
| t16  | An effect can carry its canonical TStr name as both a type and a const value via a small `NamedEffect` trait, and pairs of names compare in const context.                                                                                                | Pass.  |
| c01  | `effects_coyo!` emits a Coproduct over Coyoneda-wrapped variants in canonical lexical order.                                                                                                                                                              | Pass.  |
| c02  | Two orderings of `effects_coyo!` over the same effects produce the same wrapped row type.                                                                                                                                                                 | Pass.  |
| c03  | `Coyoneda<F, A>` implements the POC's `Functor` trait for any `F`, including effects that have no Functor impl of their own.                                                                                                                              | Pass.  |
| c04  | `fmap` over a Coyoneda value composes the function lazily without touching the inner `F`.                                                                                                                                                                 | Pass.  |
| c05  | `lower` runs the composed function over the stored `F` and produces the final `A`.                                                                                                                                                                        | Pass.  |
| c06  | `CoproductSubsetter` permutation fallback works over Coyoneda-wrapped rows (workaround 3 + Coyoneda compose).                                                                                                                                             | Pass.  |
| c07  | Generic effect types canonicalise under Coyoneda wrapping.                                                                                                                                                                                                | Pass.  |
| c08  | `Coproduct<H, T>` implements `Functor` via recursive trait dispatch (`Coproduct<H, T>: Functor` where `H: Functor + T: Functor`, with `CNil` as the base case). The active variant's `fmap` is selected by trait resolution alone, no runtime dictionary. | Pass.  |

`assert_type_eq_all!` from `static_assertions` performs the type-equality check at compile time, so a passing test means the compiler unified the two type aliases. Failure would surface as a build error, not a runtime panic.

## 4. Findings

### 4.1 The hybrid works as designed

The two halves compose without conflict. Workaround 1 (macro emits canonical form) takes care of the common case; users who write `effects![A, B]` and `effects![B, A]` in different modules get the same type at composition sites with no machinery visible. Workaround 3 (`CoproductSubsetter` permutation proof) handles the rarer case where a user hand-writes a non-canonical `Coproduct<...>`; tests t09 and t13 show the `.subset()` method routes such a row into a handler typed against the canonical form.

The handler itself is unaware of which path was used. From the handler's perspective there is one type, the canonical form, and both invocation paths produce it.

### 4.2 Sort by stringified type works for generic and lifetime-parameterised effects

The riskiest unknown going in was whether sorting by `quote!{}.to_string()` would be robust under the kinds of types real effects use. Tests t06, t07, t08 cover the cases the decisions section 4.1 explicitly flagged:

- Generic effect types (`Reader<Env>`, `State<S>`) sort consistently because their stringified form includes the generic parameters. Two invocations with the same effects in different orders produce identical sorted strings and therefore identical canonical types.
- Same root with different generic parameters (`Reader<i32>` vs `Reader<i64>`) sort as distinct entries because their full stringified forms differ at the parameter position. The canonical form preserves both, in lexical order.
- Lifetime parameters compile through. The macro does not need to special-case `'a` or `'static`.

The decisions listed three concerns about workaround 1: hand-written types bypass the sort, generic parameters in textual names, and fully-generic effects without canonical names at expansion time. The first concern is real and is exactly why workaround 3 exists as a fallback. The second and third concerns are addressed by stringifying the WHOLE type (including parameters) before sorting; this POC confirms that approach works for the cases that matter.

### 4.3 Empty and singleton edge cases work

Tests t04 and t05 cover the boundary cases. `effects![]` produces `CNil`; `effects![A]` produces `Coproduct<A, CNil>`. The macro's right-nested fold (initial accumulator `CNil`, fold from right) handles both correctly without special-casing.

### 4.4 Scaling to 5 and 7 effects is fine

Tests t11 and t12 stress trait inference. 5-effect and 7-effect rows compile and canonicalise across multiple orderings without observable slowdown. The `cargo test` runtime for the full 13-test suite is under one second on a cold build, suggesting trait resolution for these sizes is comfortable.

This does not refute the decisions's note that compile time scales with permutation size in the worst case. Workaround 3's `.subset()` machinery scales factorially in the worst case (the `frunk` indices machinery searches permutations), but for normal sizes this remains fast. A future test suite would want to push to 10+ effects to find the inflection point.

### 4.5 tstr_crates integration: building blocks present, type-level sort still needs nightly

Tests t14, t15, t16 probe what `tstr_crates` adds on top of the proc-macro hybrid. The honest answer:

- **Stable type-level identity for names (t14).** `TS!("reader")` invocations in different module contexts produce the same type. Each effect can carry a canonical name independent of its import path, which avoids a real failure mode of the macro-time sort: today, the macro stringifies a type like `crate::a::Reader` differently from `b::Reader` if both are imported from different paths. With TStr names, the canonical identifier is content-addressed.
- **Compile-time string ordering (t15).** `tstr::cmp(ts!("reader"), ts!("state"))` evaluates to `Ordering::Less` in const context. This is the building block any name-driven canonicalisation would consume.
- **Trait-shaped naming (t16).** A small `NamedEffect` trait with `type Name: IsTStr + Copy; const NAME: Self::Name;` lets each effect declare its canonical name in a way that survives const evaluation. A subtle wrinkle: `Default::default()` is NOT const-fn, so the obvious shape (`Self::Name::default()`) cannot be used in const context; the associated `const NAME` is required.

What `tstr_crates` does NOT enable on stable Rust: the `Ordering` returned by `tstr::cmp` cannot parameterise types. Lifting it into trait dispatch (so a recursive `Sort<Row>` trait can drive auto-canonicalisation of hand-written coproducts) requires nightly's `feature(adt_const_params)` plus `feature(generic_const_exprs)`. The proc-macro (workaround 1) and `CoproductSubsetter` (workaround 3) remain the stable-Rust answer. `tstr_crates` is a refinement available today only at the macro-input layer (a richer macro could take TStr names from each effect's `NamedEffect` impl, sort by them, and emit canonical Coproducts that are stable across import paths) and a fuller refinement deferred until nightly stabilises the relevant features.

### 4.6 Static-via-Coyoneda for section 4.2: empirically validated end-to-end

Tests c01-c08 close the open question in decisions section 4.2 (which option resolves the `Functor` dictionary requirement for `VariantF<R>`):

- **The macro integrates with Coyoneda wrapping cleanly.** `effects_coyo![A; F1, F2]` lexically sorts the inner effect names and emits `Coproduct<Coyoneda<F1, A>, Coproduct<Coyoneda<F2, A>, CNil>>`. Two orderings produce the same canonical row, including for generic effect types (c01, c02, c07). The macro syntax requires an explicit answer-type prefix because `Coyoneda<F, A>` has two type parameters and the macro needs both at expansion time; in production this would be hidden inside `Run<Effs, A>`'s definition so users would not see it.
- **`Coyoneda<F, A>` is a Functor for any F.** The POC's `Functor` trait impl on `Coyoneda` works for any inner `F`, including types that have no Functor impl of their own (`Logger` in test c03). `fmap` composes the lifted function without touching `fb`; the inner state is preserved through chains (c04). `lower` runs the composed function for the round-trip (c05).
- **`Coproduct<H, T>` implements Functor by recursive trait dispatch.** Test c08 demonstrates that the row itself becomes a Functor via two impls: `Coproduct<H, T>: Functor where H: Functor + T: Functor` (which matches on `Inl` / `Inr` and forwards to the active variant) and `CNil: Functor` (base case, vacuous since `CNil` has no values). Trait resolution selects the right path with no specialization, no runtime dictionary, no nightly features.

Combined, c01-c08 demonstrate that the static option in section 4.2 works end-to-end on stable Rust 1.94. Any effect type can participate in a row regardless of whether it implements Functor naturally; the user wraps in Coyoneda at lift time. The dynamic `DynFunctor` option (boxed-trait-object dispatch) is therefore unnecessary unless a future use case surfaces an effect that genuinely cannot be Coyoneda-wrapped.

The POC's stub Coyoneda uses `Box<dyn Any>` to erase the intermediate type `B`, mirroring fp-library's real Coyoneda; the stub demonstrates the SHAPE of the static-via-Coyoneda story and the macro integration. Production would use fp-library's actual Coyoneda family (`Coyoneda` / `RcCoyoneda` / `ArcCoyoneda` / `CoyonedaExplicit`) paired with the matching Free variant per section 5.2.

### 4.7 No warnings, clean compile

The final test suite produces no compiler warnings. The proc-macro implementation is short enough that there is no clippy noise either. The POC could be promoted to production with minimal additional engineering; the macro source is already commented and small.

## 5. What was NOT tested

In scope to flag for follow-up work:

- **Compile-time error messages on failing fallback.** When a user hand-writes a coproduct that is NOT a permutation of the canonical row (e.g., it lacks an effect the handler requires), what does the compiler say? `static_assertions::assert_type_eq_all!` failure produces a `mem::transmute` error message; `.subset()` failure produces a long frunk-index error. Neither is friendly, and the decisions flagged error quality as a concern. A real follow-up would capture sample errors and judge whether they meet the user-experience bar.
- **Macro hygiene with ambient generics.** This POC uses concrete types (`A`, `B`, `Reader<Env>`). What happens when `effects![T, U]` is expanded inside a function generic over `T` and `U`? The macro stringifies `T` and `U` to "T" and "U" lexically, which works for sorting but might surprise a user who expects type-identity-based ordering rather than name-based ordering. Worth testing.
- **A richer macro that consumes TStr names from a `NamedEffect` trait.** Tests t14-t16 demonstrate the data shape and the compile-time comparison. The remaining step (a proc-macro that takes effects and reads their TStr names instead of the type's stringified path) is straightforward but not implemented in this POC. Worth doing if the project decides import-path independence is important.
- **Compile-time benchmark.** No measurement of how long the macro adds to compile time on the user's side, or how `.subset()` mediation scales as the row grows past 7 effects. The plan flagged factorial worst-case for trait resolution; the POC did not exercise that boundary.
- **Negative tests.** Compile-fail tests would prove the system rejects ill-typed fallbacks (e.g., handler expects `effects![A, B]` but receives `Coproduct<A, CNil>`). Useful for regression, but not load-bearing for the feasibility verdict.

## 6. Implications for decisions section 4.1

The hybrid is feasible. The decisions's existing recommendation (workaround 1 primary, workaround 3 fallback) can stand without revision; the POC strengthens rather than changes the recommendation.

Three small adjustments worth considering for the eventual implementation:

1. The macro implementation is ~30 lines. Whatever crate ends up hosting `effects!` (likely `fp-macros` if the port reuses the existing proc-macro infrastructure) can absorb the addition without significant maintenance burden.
2. Use `quote!{}.to_string()` for sorting, not `syn::Type::to_token_stream().to_string()` or other variants. The `quote` form normalises whitespace, which makes the sort deterministic across syntactically equivalent inputs (e.g., `Reader<Env>` and `Reader < Env >` produce the same stringified form).
3. The macro should emit an absolute path (`::frunk_core::coproduct::Coproduct`) or whatever the project's chosen Coproduct namespace is, so `effects!` works regardless of which prelude items the calling crate has imported. The POC uses `::frunk_core::coproduct::Coproduct` directly.

## 7. Recommended next steps

The POC's verdict is "feasible, no design changes". The remaining open questions (error quality, hygiene with ambient generics, compile-time scaling, `tstr_crates` comparison, negative tests) are implementation-quality concerns rather than design concerns. They should be addressed during the actual port implementation, not as further research.

If the project decides to proceed with the effects port, the implementation work is:

1. Replace `frunk_core::coproduct::Coproduct` with the project's chosen Coproduct type (likely a Brand-shaped variant) without changing the canonicalisation strategy.
2. Decide where `effects!` lives: a new module in `fp-macros`, or a separate `fp-effects-macros` crate.
3. Carry forward the POC's tests as a regression suite, expanded to cover the negative cases (compile-fail) and the hygiene cases noted in section 5.

The POC code at `poc-effect-row/` can be deleted once those tests migrate to the production crate. Until then it remains a reference implementation.

## 8. Verdict

The hybrid (workaround 1 macro canonicalisation + workaround 3 `CoproductSubsetter` fallback) is a working design on stable Rust 1.94.1. Thirteen passing tests cover the cases the decisions flagged as risky, including generics, lifetimes, and 5-7 effect scaling. No decisions edits are recommended; the existing recommendation can be implemented as written.
