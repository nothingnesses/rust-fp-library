---
title: Consolidated Review Findings
date: 2026-04-17
sources:
  - 01-trait-design-review.md (Agent 1)
  - 02-macro-and-codegen-review.md (Agent 2)
  - 03-migration-and-compat-review.md (Agent 3)
  - 04-inference-edge-cases-review.md (Agent 4)
  - 05-integration-and-ops-review.md (Agent 5)
---

# Consolidated Review Findings

Five review agents analyzed the multi-brand ergonomics plan from
different angles. This document deduplicates and prioritizes the
findings across all five.

## High severity

### H1. Removing InferableBrand breaks `pure`, `empty`, and other non-closure operations for ALL types (Agents 1, 3)

Decision D eliminates InferableBrand entirely. But `pure(5)`,
`empty()`, `sequence(xs)`, and similar non-closure operations
currently rely on `<FA as InferableBrand>::Brand` as an
associated-type projection to infer Brand. After removal, these
operations lose inference for single-brand types too, not just
multi-brand ones. This is a regression in functionality, not an
improvement.

**Approaches:**

- a) Retain InferableBrand (or a renamed equivalent like
  `UniqueBrand`) alongside Slot. InferableBrand serves non-closure
  operations; Slot serves closure-directed dispatch. Trade-off:
  two trait families, contradicting Decision D's "one family"
  goal, but no regression.
- b) Accept the regression: all non-closure operations require
  `explicit::` for every type. Trade-off: significant ergonomic
  loss for common calls like `pure(5)`.
- c) Add a `type DefaultBrand` associated type to Slot for
  single-brand types, derived from the sole Slot impl. Trade-off:
  Rust cannot express "there is exactly one impl" without
  specialization.

**Recommendation:** a). Non-closure operations need
InferableBrand's associated-type projection. Decision D should be
revised to "InferableBrand is no longer used by closure-taking
operations" rather than "InferableBrand is deleted." This preserves
`pure(5)`, `empty()`, etc. for single-brand types. Renaming to
something like `UniqueBrand` could clarify the new role.

### H2. Phase 1 step 5 (InferableBrand removal) blocks compilation before all dispatch modules are migrated (Agents 3, 5)

The plan removes InferableBrand in phase 1 step 5, but only `map`
and `explicit::map` are rewritten by that point. All other 18
dispatch modules still reference InferableBrand and will fail to
compile. This creates a non-compiling intermediate state that
prevents incremental testing.

**Approaches:**

- a) Defer InferableBrand removal to after phase 2 (all dispatch
  modules migrated). Trade-off: InferableBrand and Slot coexist
  during development, but each module is testable after migration.
- b) Migrate all 19 dispatch modules in phase 1 before removing
  InferableBrand. Trade-off: phase 1 becomes very large.
- c) Accept non-compiling intermediate state. Trade-off: loses
  incremental testability.

**Recommendation:** a). Keep InferableBrand present through phases
1-2, remove it as the final step of phase 2 (or phase 3). Note:
this aligns with H1's recommendation to retain InferableBrand for
non-closure operations; if H1 is adopted, InferableBrand is never
fully removed anyway.

### H3. Eight dispatch modules missing from the plan (Agent 5)

The plan's "Will change" table and phase 2 list omit eight dispatch
modules that use InferableBrand: `alt.rs`, `compactable.rs`,
`contravariant.rs`, `filterable_with_index.rs`,
`foldable_with_index.rs`, `functor_with_index.rs`,
`traversable_with_index.rs`, `witherable.rs`. All must be migrated
since they reference InferableBrand.

**Recommendation:** Add all eight to the phase 2 list.
Closure-taking modules (`functor_with_index`,
`foldable_with_index`, `filterable_with_index`,
`traversable_with_index`, `witherable`, `compactable`,
`contravariant`) gain multi-brand inference. Closureless modules
(`alt`) are a mechanical InferableBrand-to-Slot migration with
multi-brand remaining explicit-only.

## Medium severity

### M1. `brand-dispatch-traits.md` contradicts the adopted design (Agents 1, 3)

The document describes a Slot with `type Out<B>` GAT and an
InferableBrand blanket. The adopted design uses Marker-only Slot
with no blanket and no Out GAT. Also describes a three-trait
coexistence model that Decision D rejects.

**Recommendation:** Rewrite `brand-dispatch-traits.md` early
(phase 1 or before implementation begins) to avoid confusing
implementers. The current content is actively misleading.

### M2. Marker-agreement invariant is undocumented (Agent 1)

The plan's inference mechanism relies on all Slot impls for a given
Self type agreeing on the same Marker value (Val for owned, Ref for
references). This invariant is never stated explicitly.

**Recommendation:** Document the invariant in the Slot trait's
rustdoc and add a comment in `impl_kind!` explaining the
requirement. Enforcement is not needed since `impl_kind!` is the
sole Slot generator.

### M3. Marker projection relies on unspecified solver evaluation order (Agent 1)

The claim that "Marker commits from FA alone, before (Brand, A) are
resolved" depends on current rustc solver behaviour, not a language
guarantee. The new trait solver (rust-lang/rust#107374) could change
this.

**Recommendation:** Accept the risk. The design works on stable
rustc 1.94.1 and all foreseeable stable releases. Consider adding a
periodic nightly CI check with `-Znext-solver` for early warning.

### M4. Generic fixed-parameter case may be ambiguous (Agent 4)

`fn process<E>(r: Result<i32, E>) { map(|x: i32| x + 1, r) }`
could be ambiguous if Rust's solver cannot rule out `E = i32`. This
is a realistic pattern (generic error types in Result-heavy code).

**Recommendation:** Write a targeted POC test before
implementation. If inference fails, document the limitation in the
coverage matrix ("Val + multi-brand + generic fixed param -> may
need `explicit::`").

### M5. `#[multi_brand]` is a documentation marker, not a codegen switch (Agent 2)

The plan says `#[multi_brand]` "tells impl_kind! to emit multiple
Slot impls," but each `impl_kind!` invocation independently emits
at most one Slot impl. Multiple impls come from multiple
invocations. The attribute has no behavioral effect on Slot
generation.

**Recommendation:** Clarify in the plan that `#[multi_brand]` is a
documentation marker meaning "this brand is one of several for its
target type." No codegen behavior change.

### M6. `join`, `apply_first`, `apply_second` are closureless (Agent 5)

The plan lists these under phase 2 but does not note they lack a
closure for Brand disambiguation. For multi-brand types, they
cannot use Slot-based inference.

**Recommendation:** Add `join`, `apply_first`, and `apply_second`
to the "Operations that cannot use Slot inference" list for
multi-brand types. Their Slot migration is mechanical
(InferableBrand -> Slot) but multi-brand stays explicit-only.

### M7. No dispatch module exists for `apply`/`ref_apply` (Agents 3, 5)

The plan lists `apply` and `ref_apply` for phase 2, but there is no
`dispatch/semiapplicative.rs`. Creating one from scratch is more
work than "repeat the phase 1 rebinding."

**Recommendation:** Acknowledge in the plan that
`dispatch/semiapplicative.rs` must be created (not merely rebound).
The POC 8 already validates the signature shape.

### M8. Hash coordination needs SLOT_PREFIX + consumer updates (Agent 2)

A `SLOT_PREFIX` constant must be added, and consumer sites in
`analysis/dispatch.rs`, `analysis/traits.rs`, and
`documentation/generation.rs` must be updated. Without this, Slot
bounds in dispatch wrappers will be misclassified, producing
incorrect HM signatures.

**Recommendation:** Add `SLOT_PREFIX` to constants, update
`classify_trait`, `is_semantic_type_class`, and
`is_dispatch_container_param` in one pass.

### M9. HM signature rendering and snapshot tests will break (Agent 3)

`is_dispatch_container_param()` checks for `InferableBrand_`
prefix. After removal, it must check for `Slot_`. Signature
snapshot tests have hardcoded `InferableBrand_abc123` strings.

**Recommendation:** Update the prefix check and regenerate snapshot
expectations as part of phase 1.

### M10. `InferableBrand!` macro and `resolve_inferable_brand()` not mentioned for removal (Agent 3)

The `Apply!` macro contains `resolve_inferable_brand()` preprocessing
that scans for `InferableBrand!(SIG)` tokens. This becomes dead code
after InferableBrand removal (or stale if InferableBrand is retained
per H1).

**Recommendation:** If InferableBrand is retained (H1), this code
stays. If not, remove `resolve_inferable_brand()`, the
`InferableBrand!` proc macro, and the `INFERABLE_BRAND_MACRO`
constant.

### M11. 37 explicit functions need rewriting, not just `explicit::map` (Agent 3)

Decision F mentions `explicit::map` but all 37 explicit functions
across 19 dispatch modules use `<FA as InferableBrand>::Brand`. All
must be rewritten before InferableBrand can be removed.

**Recommendation:** Expand Decision F to cover all explicit
functions, or add a blanket statement to the integration surface
table.

## Low severity

### L1. `'static` bounds on multi-brand Slot impls (Agents 1, 4)

Multi-brand brands require `E: 'static` or `T: 'static` for
coherence. This prevents non-static fixed parameters from using
inference. Pre-existing limitation, not introduced by Slot.

**Recommendation:** Document as a known limitation.

### L2. Nested `&&T` behaviour (Agent 1)

`&&Option<A>` resolves through the `&T` blanket recursively but
FunctorDispatch's Ref impl only matches one level of reference.
Pre-existing limitation.

**Recommendation:** Add a compile-fail UI test for `&&T` to lock
in the expected behaviour.

### L3. Projection skip rule fragility (Agent 2)

The `contains("::")` / `contains("Apply")` string heuristic could
false-positive on fully-qualified paths or types named `Applicable`.

**Recommendation:** Switch to structural AST checks
(`Type::Macro` for `Apply!`, path segment count for `::`) during
implementation.

### L4. `compose_kleisli` already bypasses InferableBrand (Agents 3, 5)

These functions take Brand as turbofish and have no InferableBrand
usage. The plan lists them for phase 2 migration, but no code
change is needed.

**Recommendation:** Remove from the phase 2 list or note as
"no-op."

### L5. No multi-brand benchmark planned (Agent 5)

No benchmark validates that Slot-based multi-brand dispatch produces
identical codegen to explicit dispatch.

**Recommendation:** Add one benchmark comparing `map(|x: i32| x +
1, Ok::<i32, String>(5))` against its explicit equivalent.

### L6. Pre-bound closures lose deferred inference (Agent 4)

`let f = |x| x + 1; map(f, Ok::<i32, String>(5))` may fail
because inference context is not propagated to the let binding.

**Recommendation:** Document in phase 3 that pre-bound closures
for multi-brand types should include parameter annotations.

### L7. Attribute rename timing vs impl_kind! update (Agent 5)

Steps 3 (impl_kind! changes) and 4 (attribute rename) should be
combined to avoid an intermediate state where impl_kind! expects
one attribute name and the code has the other.

**Recommendation:** Combine steps 3 and 4 into one step.

### L8. Do-notation macros in multi-brand inferred mode blocked by `pure` (Agent 5)

Inferred-mode `m_do!` with multi-brand types fails at the `pure`
call, not the `bind` calls. Users must use explicit mode
(`m_do!(Brand { ... })`).

**Recommendation:** Decision K's audit will discover this
naturally. Document in macro docs that multi-brand `m_do!`
requires explicit brand specification.

## Informational (no action needed)

- `&mut T` / `Pin<&mut T>` Marker variant is a future concern, not
  current scope (Agent 1).
- Coherence and downstream extensibility matches existing design
  (Agent 1).
- Arity generalisation works; verify `trait_kind!` generates
  blankets per arity (Agent 1).
- Code generation volume is modest (~14 additional items)
  (Agent 2).
- InferableBrand removal scope is well-defined and mechanical
  (Agent 2).
- Macro cannot determine single-vs-multi-brand and does not need to
  (Agent 2).
- Near-diagonal types (e.g., `Result<i32, u32>`) are handled
  correctly (Agent 4).
- Nested containers, closures returning closures, and bind's
  return-type diagonal are all handled (Agent 4).
- Argument order does not affect inference (Agent 4).
- Arity-2 multi-brand types are correctly scoped (single brand at
  arity 2, multi at arity 1) (Agent 5).
- Property-based tests can be deferred; unit tests in phase 1
  step 9 are sufficient for shipping (Agent 5).

## Cross-cutting themes

Three themes recur across multiple reviews:

1. **InferableBrand cannot be fully removed.** Agents 1 and 3
   independently identified that non-closure operations (`pure`,
   `empty`, `join`, `alt`, `sequence`, `apply_first`,
   `apply_second`) depend on InferableBrand's associated-type
   projection. Decision D's "eliminate entirely" framing must be
   revised.

2. **Phase ordering needs rework.** Agents 3 and 5 both identified
   that removing InferableBrand in phase 1 step 5 creates a
   non-compiling state. Combined with theme 1, the cleanest
   resolution is: retain InferableBrand for non-closure operations,
   migrate closure-taking operations to Slot, and never remove
   InferableBrand.

3. **The plan understates migration scope.** Agents 3 and 5
   identified that the plan mentions `explicit::map` and a handful
   of dispatch modules, but the actual scope is 19 dispatch
   modules, 37+ explicit functions, macro preprocessing code, HM
   signature rendering, snapshot tests, and documentation. The
   plan's integration surface table should be expanded accordingly.
