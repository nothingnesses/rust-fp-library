# Coyoneda Implementation Analysis: Consolidated Summary

Date: 2026-03-31

This document synthesizes findings from five independent analyses of the Coyoneda
implementation (`fp-library/src/types/coyoneda.rs`) and its design document
(`plans/functor/coyoneda-design.md`). Each analysis was conducted without knowledge
of the others. The individual reports are:

- [01-analysis.md](01-analysis.md)
- [02-analysis.md](02-analysis.md)
- [03-analysis.md](03-analysis.md)
- [04-analysis.md](04-analysis.md)
- [05-analysis.md](05-analysis.md)

---

## Consensus Overview

All five agents independently converged on the same core set of issues. The table
below shows which issues each agent identified and their severity assessments.

| Issue                            | Agent 1 | Agent 2 | Agent 3 | Agent 4 | Agent 5 | Consensus Severity  |
| -------------------------------- | ------- | ------- | ------- | ------- | ------- | ------------------- |
| No map fusion                    | High    | High    | High    | Medium  | High    | **High**            |
| Foldable requires F: Functor     | Medium  | Low     | Medium  | Low     | Medium  | **Medium**          |
| Hoist requires F: Functor        | Low     | Low     | Low     | Low     | Low     | **Low**             |
| Stack overflow from deep nesting | Medium  | Medium  | Medium  | Low     | Medium  | **Medium**          |
| Fn vs FnOnce                     | None    | Low     | None    | None    | Low     | **None**            |
| Redundant allocation in `new`    | Low     | Low     | Low     | Low     | Low     | **Low**             |
| No Clone/Send/Sync               | Medium  | Medium  | Medium  | Medium  | Medium  | **Medium**          |
| Missing type class instances     | N/A     | Medium  | Medium  | Low     | Low     | **Low-Medium**      |
| Identity allocation in `lift`    | None    | None    | None    | None    | None    | **None (resolved)** |
| Foldable lower-then-fold cost    | N/A     | Medium  | N/A     | Low     | N/A     | **Low**             |
| Design document divergence       | Medium  | N/A     | Medium  | N/A     | N/A     | **Medium**          |
| Test coverage gaps               | Medium  | N/A     | Medium  | N/A     | N/A     | **Medium**          |
| No Debug implementation          | N/A     | N/A     | Medium  | Medium  | N/A     | **Low**             |

---

## Issue 1: No Map Fusion (High, unanimous)

**Consensus:** All five agents identified this as the most significant issue. The
implementation calls `F::map` once per `CoyonedaMapLayer` at `lower` time, meaning
k chained maps produce k traversals, identical in cost to calling `F::map` directly.
This contradicts the stated primary motivation of Coyoneda.

**Root cause:** Dyn-compatibility prevents a generic `map_inner<C>` method on the
`CoyonedaInner` trait. Composing functions across the existential boundary requires
this generic method, which cannot appear on a trait object.

**Recommended approach (unanimous):** All five agents recommend a two-part strategy:

1. **Accept the limitation for HKT-integrated `Coyoneda`.** Reframe its purpose
   around HKT integration (free functor property, deferred execution) rather than
   performance optimization.
2. **Provide `FunctorPipeline` as a companion type.** A generic struct exposing the
   intermediate type `B` as a type parameter, enabling zero-cost function composition
   with true fusion. Cannot participate in HKT but covers the performance use case.

Three agents (1, 2, 4) also mentioned unsafe pointer erasure as a theoretical path
to true fusion within the trait-object encoding, but all assessed the `unsafe`
burden as too high to recommend as a default approach. Agent 4 suggested it could
be explored behind a feature flag.

---

## Issue 2: No Clone, Send, or Sync (Medium, unanimous)

**Consensus:** All five agents identified the lack of `Clone`, `Send`, and `Sync` as
blocking several downstream type class implementations. `Box<dyn CoyonedaInner>` is
not cloneable, and the trait object bound does not include `Send`.

**What it blocks:**

- `Traversable` (requires `Clone`).
- `Semiapplicative` / `Applicative` (typically requires `Clone`).
- Thread-safe usage.

**Recommended approach (4/5 agents):** Introduce an Rc/Arc-wrapped variant
parameterized over the pointer brand, following the library's established pattern
with `FnBrand<P>` and `LazyBrand<Config>`. The inner trait object would be wrapped
in `Rc` or `Arc` instead of `Box`, enabling `Clone` via reference counting and (for
`Arc`) `Send + Sync`.

Agent 5 recommended the `Thunk`/`SendThunk` duplication pattern for `Send` and the
Rc/Arc pattern separately for `Clone`. The remaining four agents preferred a unified
parameterized approach.

---

## Issue 3: Foldable Requires F: Functor (Medium, unanimous)

**Consensus:** All five agents identified this divergence from PureScript. The
current implementation lowers first (requiring `F: Functor`), then folds. PureScript
composes the fold function with the accumulated mapping function via `unCoyoneda`,
folding the original `F B` in a single pass without requiring `Functor`.

**Recommended approach (unanimous):** Accept the limitation and document it. The
`F: Functor` bound is a pragmatic consequence of dyn-compatibility. In practice,
nearly all `Foldable` types in the library are also `Functor`. If unsafe type erasure
is ever adopted for map fusion, it would also resolve this issue as a side benefit.

---

## Issue 4: Stack Overflow from Deep Nesting (Medium, 4/5 agents)

**Consensus:** Each `map` adds a nested layer. At `lower` time, layers unwind
recursively with depth proportional to the number of chained maps. For thousands of
maps, this overflows the stack.

**Recommended approach:** Four agents recommend documenting the limitation. Agent 4
additionally proposes a `collapse` method that periodically lowers and re-lifts to
flatten the layer stack (requiring `F: Functor`). All agents note that
`FunctorPipeline` would naturally avoid this issue.

If the Vec-based flat representation from the fusion discussion (Issue 1) is ever
implemented, it would eliminate the recursion entirely.

---

## Issue 5: Redundant Allocation in `new` (Low, unanimous)

**Consensus:** `Coyoneda::new(f, fb)` creates a `CoyonedaMapLayer` wrapping a
`CoyonedaBase`, requiring 3 heap allocations. A unified struct holding both `fb` and
`func` directly (matching the design document's `CoyonedaImpl`) would need only 2.

Two agents (2, 3) additionally noted that the design document's claim that `new`
"saves one box allocation vs `lift(fb).map(f)`" appears to be incorrect, since both
paths produce 3 allocations.

**Recommended approach (unanimous):** Add a `CoyonedaImpl` / `CoyonedaSingle` /
`CoyonedaWithFunc` struct (names varied across agents) that stores `fb` and `func`
together. Low-effort change that saves one allocation per `new` call.

---

## Issue 6: Missing Type Class Instances (Low-Medium, 4/5 agents)

**Consensus:** PureScript provides many more instances (`Apply`, `Applicative`,
`Bind`, `Monad`, `Traversable`, `Extend`, `Comonad`, `Eq`, `Ord`, `Show`). The
current implementation provides only `Functor`, `Pointed`, and `Foldable`.

**Recommended approach:** Implement incrementally:

1. **Now (no blockers):** `Eq`, `Ord`, `Debug` via lowering.
2. **After Semiapplicative/Semimonad design:** Implement via lower-apply-lift pattern.
3. **After Rc/Arc variant:** `Traversable`, `Applicative`.

---

## Issue 7: Hoist Requires F: Functor (Low, unanimous)

**Consensus:** The current `hoist` lowers, transforms, and re-lifts. PureScript
applies the natural transformation directly to the hidden `F B`. All agents agree
this is a minor limitation.

**Recommended approach (unanimous):** Accept. Well-documented; `F: Functor` is a
reasonable requirement for `hoist`.

---

## Issue 8: Fn vs FnOnce (None, unanimous)

**Consensus:** All five agents analyzed whether `FnOnce` should replace `Fn` in the
stored mapping function. All independently concluded that `Fn` is correct because
`Functor::map` calls the function once per element, and multi-element containers
(like `Vec`) require multiple calls. No change needed.

---

## Issue 9: Design Document Divergence (Medium, 2/5 agents)

**Finding:** Agents 1 and 3 flagged that the design document contains two conflicting
narratives. Early sections describe and promise map fusion; later sections explain
that fusion was not achieved. Agent 3 also noted an incorrect allocation count claim
for `new`.

**Recommended approach:** Add a prominent note at the top of the design document
clarifying that the implementation does not achieve fusion, and correct the allocation
count claim.

---

## Issue 10: Test Coverage Gaps (Medium, 2/5 agents)

**Finding:** Agents 1 and 3 identified missing test types:

- No property-based tests (functor laws tested with single hardcoded inputs).
- No compile-fail tests for error messages.
- No lifetime tests with borrowed data.
- No type-changing map chain tests.

**Recommended approach:** Add QuickCheck property tests for functor and foldable laws
as the highest priority. Add compile-fail tests for the `F: Functor` constraint on
`lower`.

---

## Issue 11: Identity Allocation in `lift` (None, unanimous)

**Consensus:** Already optimized. The `CoyonedaBase` struct stores `fa` directly
without an identity function. No action needed.

---

## Priority Ranking

Based on frequency of identification, severity consensus, and impact on the library:

| Priority | Issue                        | Action                                           |
| -------- | ---------------------------- | ------------------------------------------------ |
| 1        | No map fusion                | Build `FunctorPipeline`; reframe Coyoneda docs.  |
| 2        | No Clone/Send/Sync           | Rc/Arc variant parameterized over pointer brand. |
| 3        | Test coverage gaps           | Property-based tests and compile-fail tests.     |
| 4        | Design document divergence   | Correct claims; add clarifying note at top.      |
| 5        | Missing type class instances | Implement `Debug`, `Eq`, `Ord` via lowering.     |
| 6        | Stack overflow risk          | Document limitation; consider `collapse` method. |
| 7        | Redundant allocation in new  | Add `CoyonedaImpl` struct.                       |
| --       | Foldable requires Functor    | Accept; already documented.                      |
| --       | Hoist requires Functor       | Accept; already documented.                      |
| --       | Fn vs FnOnce                 | No action; current design is correct.            |
| --       | Lift identity allocation     | No action; already optimized.                    |

---

## Key Themes

**Dyn-compatibility is the root constraint.** Issues 1, 3, 4, and 7 all stem from
the same fundamental limitation: trait objects cannot have generic methods. This
single constraint prevents map fusion, forces `F: Functor` on `Foldable` and `hoist`,
and prevents single-pass folding. Any solution that addresses the dyn-compatibility
barrier (such as unsafe type erasure) would resolve multiple issues simultaneously.

**The implementation provides structural value, not performance value.** All agents
converged on reframing Coyoneda's purpose: HKT integration (free functor property)
and deferred execution, rather than map fusion. The `FunctorPipeline` companion type
was universally recommended for users who need actual fusion.

**The Rc/Arc variant is the highest-impact new feature.** It unblocks `Clone`,
`Traversable`, `Semiapplicative`, and thread safety. Four of five agents recommended
parameterizing over the pointer brand following established library patterns.
