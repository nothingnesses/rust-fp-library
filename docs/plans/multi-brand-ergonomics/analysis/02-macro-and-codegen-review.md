---
title: Macro and Code Generation Review
reviewer: Agent 2
date: 2026-04-17
scope: impl_kind!, trait_kind!, hash coordination, projection skip rule, attribute rename
---

# Macro and Code Generation Review

## 1. Hash coordination between Kind, InferableBrand, and Slot

### Issue

Today, `Kind_{hash}` and `InferableBrand_{hash}` share the same hash suffix
via `generate_prefixed_name` in `fp-macros/src/hkt/canonicalizer.rs`. The plan
adds `Slot_{hash}` as a third trait family using the same hash. This is
straightforward in principle, but the implementation must ensure that:

(a) A `generate_slot_name` function (or equivalent) is added to the
canonicalizer, reusing `generate_prefixed_name` with a new `SLOT_PREFIX`
constant.

(b) The `SLOT_PREFIX` constant is added to
`fp-macros/src/core/constants.rs` alongside `KIND_PREFIX` and
`INFERABLE_BRAND_PREFIX`.

(c) Every consumer of `INFERABLE_BRAND_PREFIX` (the dispatch analysis module
in `fp-macros/src/analysis/dispatch.rs` lines 648, and the trait classifier
in `fp-macros/src/analysis/traits.rs` line 42) must be updated to reference
`SLOT_PREFIX` instead after the removal of `InferableBrand`.

The dispatch analysis module uses the prefix to classify traits into the
`Kind` category for purposes of HM signature generation. If `Slot_*` traits
appear in dispatch wrapper bounds (which they will, since `map` will bound on
`Slot`), the analysis must recognize them. Currently, `Slot_*` would fall
through to `TraitCategory::Other` and be treated as a semantic type class,
producing incorrect HM signatures.

### Approaches

**A1.** Add `SLOT_PREFIX` to constants and a `generate_slot_name` function.
Update `classify_trait` and `is_semantic_type_class` to handle `Slot_*` the
same way they currently handle `InferableBrand_*`. Remove the
`INFERABLE_BRAND_PREFIX` constant and all references after migration.

- Trade-off: Clean, but requires touching every file that references the
  old constant. There are ~6 such sites across `analysis/dispatch.rs`,
  `analysis/traits.rs`, `documentation/document_module.rs`, and the
  canonicalizer.

**A2.** Rename `INFERABLE_BRAND_PREFIX` to `SLOT_PREFIX` in place and update
the string value from `"InferableBrand_"` to `"Slot_"`.

- Trade-off: Fewer code changes, but the old variable name lingering in
  git history could confuse future readers. Not a real risk given good
  commit messages.

**A3.** Keep both prefixes temporarily, with `INFERABLE_BRAND_PREFIX`
deprecated. Remove in a follow-up.

- Trade-off: Adds dead code. Since this is a single-release change
  (Decision B3), there is no reason to keep both.

### Recommendation

A1. The constant count is small, the changes are mechanical, and carrying
dead constants adds confusion. Do it in one pass.

---

## 2. `trait_kind!` changes to emit Slot traits

### Issue

`trait_kind_worker` in `fp-macros/src/hkt/trait_kind.rs` currently emits
exactly two items per invocation: a `Kind_{hash}` trait and an
`InferableBrand_{hash}` trait with a `&T` blanket impl. The plan requires it
to emit a `Slot_{hash}` trait (with an associated `type Marker`) and a `&T`
blanket impl for that Slot trait instead of the InferableBrand pair.

The Slot trait signature differs from InferableBrand in three ways:

1. Slot has three type parameters (`'a`, `Brand`, `A: 'a`) whereas
   InferableBrand has none (it is implemented on concrete types with an
   associated `type Brand`).
2. Slot has an associated `type Marker` instead of `type Brand`.
3. The `&T` blanket for Slot is structurally different: it delegates Marker
   to `Ref` rather than delegating Brand to the inner type's Brand.

The Slot parameters (`'a`, `A: 'a`) must match the Kind trait's associated
type signature. For the most common arity (`type Of<'a, A: 'a>: 'a`), Slot
has `<'a, Brand, A: 'a>`. For arity-2 (`type Of<'a, A: 'a, B: 'a>: 'a`),
the plan mentions a separate arity-2 Slot, confirmed by POC 6.

The question is how `trait_kind!` determines the Slot parameters from the
associated type signature. The associated type's generics (lifetimes and type
parameters) need to be threaded into the Slot trait's parameter list, but
`Brand` must be inserted as an additional type parameter that is bounded by
the corresponding `Kind_{hash}` trait.

For multi-associated-type Kind traits (which exist in the codebase, e.g.,
`type Of<...>; type SendOf<...>;`), the plan does not specify how Slot
generation works. Currently the codebase has 8 `trait_kind!` invocations in
`kinds.rs`, all with a single associated type. If a Kind trait had multiple
associated types, the Slot trait's parameter list would need to be derived
from... which one? The plan says Slot is "one trait per Kind arity (same
pattern as today's Kind*\* and InferableBrand*\* families)," but multiple
associated types within one Kind trait can have different arities.

### Approaches

**B1.** Generate Slot only from the first (or only) associated type's
signature. Error or skip if the Kind trait has multiple associated types.

- Trade-off: Simple, but theoretically limits future multi-associated-type
  Kinds from participating in Slot-based inference. In practice, no such
  Kind traits exist today, and adding them later is unlikely.

**B2.** Generate one Slot per associated type, with each Slot keyed on that
associated type's name (e.g., `Slot_Of_{hash}`, `Slot_SendOf_{hash}`).

- Trade-off: More flexible but increases trait proliferation and naming
  complexity. No current use case.

**B3.** Only generate Slot when the Kind trait has exactly one associated
type. Skip Slot generation for multi-associated-type Kinds silently.

- Trade-off: Safe default. Multi-associated-type Kinds are architectural
  infrastructure (e.g., for brands with both `Of` and `SendOf`), and their
  concrete types are typically already reachable through a simpler
  single-associated-type Kind.

### Recommendation

B3. No multi-associated-type Kind currently needs Slot, and the plan's
own examples only show single-associated-type Slots. Skip silently rather
than erroring, since multi-associated-type Kinds are not a mistake.

---

## 3. `impl_kind!` changes to emit Slot impls

### Issue

`impl_kind_worker` in `fp-macros/src/hkt/impl_kind.rs` currently emits a
`Kind_{hash}` impl and optionally an `InferableBrand_{hash}` impl. The plan
requires replacing the InferableBrand impl with a Slot impl.

The Slot impl is structurally more complex than InferableBrand:

- InferableBrand impl: `impl<...> InferableBrand_{hash} for TargetType { type Brand = BrandType; }`
- Slot impl: `impl<'a, A: 'a, ...extra...> Slot_{hash}<'a, BrandType, A> for TargetType<A, ...> { type Marker = Val; }`

The macro must synthesize the Slot impl's parameter list from the Kind
associated type's generics. Specifically:

1. The associated type's lifetime and type parameters become the Slot's
   generic parameters (e.g., `'a, A: 'a` for arity 1).
2. The Brand type is the `impl_kind!` input's `brand` field.
3. The target type's generic parameters from `impl_generics` (e.g., `E` in
   `impl<E: 'static> for ResultErrAppliedBrand<E>`) must be included in
   the Slot impl's generics.
4. The `For` type is the associated type's RHS (the target type expression,
   e.g., `Result<A, E>`).

The existing `build_inferable_brand_generics` function already performs the
work of collecting which impl generics appear in the target type. This logic
can be adapted for the Slot impl, but requires additional work to merge in
the associated type's own generics (which InferableBrand didn't need because
it had no type parameters).

A subtle point: for Slot, the "Self" type in the impl is the **target type**
(e.g., `Result<A, E>`), whereas for Kind the "Self" type is the **brand**
(e.g., `ResultErrAppliedBrand<E>`). This reversal already exists for
InferableBrand, so the pattern is established.

### Approaches

**C1.** Extend `build_inferable_brand_generics` (renamed appropriately) to
merge both the associated type's generics and the impl generics, producing
the full Slot impl generics. Reuse the `TypeIdentCollector` for determining
which parameters appear in the target type.

- Trade-off: Incremental change to existing code. Some complexity in
  merging two Generics sources, but manageable.

**C2.** Write the Slot impl generation as a separate function, not sharing
code with the InferableBrand path (which is being removed anyway).

- Trade-off: Cleaner separation, avoids trying to adapt code that will be
  deleted. Slightly more code, but no need to reason about backward
  compatibility with a function that no longer has its original purpose.

**C3.** Template the Slot impl as a formatted string rather than using
`quote!`, avoiding the generics-merging problem entirely.

- Trade-off: Fragile, hard to maintain, poor diagnostics on errors. Not
  recommended for proc macros.

### Recommendation

C2. Since InferableBrand generation is being removed in the same change,
there is no value in adapting its code. A fresh function is clearer, and
`TypeIdentCollector` can still be reused as a utility.

---

## 4. Projection auto-skip rule

### Issue

The plan (Decision G) states that `impl_kind!` should skip Slot generation
when the brand's `Of` target type "contains `Apply!` or `::`." The current
code already implements a similar skip for InferableBrand in
`should_generate_inferable_brand` (line 217 of `impl_kind.rs`):

```rust
let target_str = quote!(#target).to_string();
if target_str.contains("::") || target_str.contains("Apply") {
    return false;
}
```

This is a string-based heuristic on the token stream. It works today because:

- Projection brands like `BifunctorFirstAppliedBrand` have `Of` targets
  that use `Apply!(<Brand as Kind!(...)>::Of<...>)`, which contains both
  `Apply` and `::`.
- Normal brands have `Of` targets like `Option<A>` or `Result<A, E>`, which
  contain neither.

However, the heuristic has a false-positive risk: a target type whose name
coincidentally contains `Apply` (e.g., a user-defined `Applicable<A>` type)
would be incorrectly skipped. Similarly, any fully-qualified type path (e.g.,
`std::option::Option<A>`) contains `::` and would be skipped.

In the current codebase, no such false positives exist. But the heuristic is
fragile for downstream users invoking `impl_kind!` with their own types.

### Approaches

**D1.** Keep the string heuristic as-is (renaming the function). Document
the limitation.

- Trade-off: Simple, works for all current cases. Downstream users who
  write fully-qualified paths would hit the false positive, but this is
  uncommon in `impl_kind!` invocations (which typically use short brand
  names imported into scope).

**D2.** Parse the target type AST structurally: check for `syn::Macro` nodes
(which `Apply!` expands through) and path segments with more than one
component.

- Trade-off: More robust, but significantly more complex. `Apply!` is
  itself a macro, so at `proc_macro` expansion time it appears as a
  `syn::TypeMacro` node, which is easy to detect. Multi-segment paths
  can be detected by checking `syn::TypePath` segment count.

**D3.** Introduce an explicit `#[no_slot]` attribute for projection brands,
replacing the heuristic entirely.

- Trade-off: Explicit is better than implicit. But it adds another
  attribute to learn. The current heuristic catches all projection brands
  automatically without annotation.

### Recommendation

D2. The structural check is not much harder than the string check (match on
`Type::Macro` for `Apply!`, check path segment count for `::`), and it
eliminates the class of false positives entirely. The `syn::visit::Visit`
infrastructure is already used in this module.

---

## 5. `#[no_inferable_brand]` -> `#[multi_brand]` rename and semantic shift

### Issue

The plan renames `#[no_inferable_brand]` to `#[multi_brand]` (Decision E).
Under the old semantics, this attribute meant "do not generate an
InferableBrand impl," which effectively excluded the type from inference
entirely. Under the new semantics, `#[multi_brand]` means "generate
_multiple_ Slot impls (one per brand)."

The semantic shift is significant: the old attribute suppressed code
generation, while the new one triggers additional code generation. The macro
currently checks for the attribute in `should_generate_inferable_brand` and
returns `false` (skip) when found. After the change, the attribute must
trigger a different code path that generates multiple Slot impls.

The critical question is: **what information does `#[multi_brand]` convey
that the macro cannot already determine?** Today, multi-brand types have
separate `impl_kind!` invocations per brand (e.g., `ResultErrAppliedBrand`
and `ResultOkAppliedBrand` each have their own `impl_kind!` call). Each
invocation independently decides whether to generate a reverse-mapping impl.
The `#[no_inferable_brand]` attribute is placed on each individual
invocation.

Under Slot, each `impl_kind!` invocation still generates one Slot impl. The
attribute `#[multi_brand]` on a given invocation would mean "this brand shares
its target type with other brands, so do not expect uniqueness." But the
macro already generates a Slot impl regardless (Slot does not require
uniqueness, since Brand is a trait parameter). So what does `#[multi_brand]`
actually change in the Slot world?

Looking at the plan more carefully: for single-brand types, the existing
auto-skip rule (projection check and multiple-associated-type check) handles
the cases where Slot should not be generated. For multi-brand types, Slot
is _always_ generated, one per `impl_kind!` invocation. The only difference
is that single-brand types get exactly one Slot impl while multi-brand types
get multiple (from separate `impl_kind!` calls).

This means `#[multi_brand]` might be a no-op for Slot generation: every
non-projection, single-associated-type `impl_kind!` invocation generates
exactly one Slot impl, regardless of whether the type is single-brand or
multi-brand. The attribute could be retained purely as documentation or for
future use (e.g., diagnostics, compile-time checks).

### Approaches

**E1.** Make `#[multi_brand]` a documentation-only attribute that the macro
recognizes and strips (to avoid "unused attribute" warnings) but does not
change code generation behavior.

- Trade-off: Simple, avoids over-engineering. The attribute serves as a
  signal to human readers and to the `#[diagnostic::on_unimplemented]`
  message generation. But it is misleading if it has no effect.

**E2.** Use `#[multi_brand]` to emit a different `#[diagnostic::on_unimplemented]`
message on the generated Slot impl, mentioning closure annotation. Single-brand
Slot impls get no diagnostic (since they always infer). This is the only
behavioral difference.

- Trade-off: Provides user-facing value. But `#[diagnostic::on_unimplemented]`
  goes on the _trait_, not on individual impls, so per-impl diagnostics are
  not possible. This approach does not work with stable Rust.

**E3.** Use `#[multi_brand]` as a compile-time assertion: the macro verifies
that the target type does not already have a single-brand Slot impl at this
arity. This would catch accidental omission of the attribute.

- Trade-off: Cannot be implemented in a proc macro, since proc macros
  cannot query the trait impl landscape of other items.

### Recommendation

E1. The attribute should be retained for documentation and forward
compatibility, but should not attempt to change code generation. The plan's
statement that `#[multi_brand]` "tells impl_kind! to emit multiple Slot
impls" is slightly misleading: each `impl_kind!` invocation always emits at
most one Slot impl. Multiple Slot impls come from multiple `impl_kind!`
invocations. The attribute is better understood as a marker saying "this
brand is not the only one for its target type."

Update the plan to clarify that `#[multi_brand]` is a documentation marker,
not a code generation switch. If a future need arises for it to have
behavioral effects (e.g., custom diagnostics), it can be upgraded then.

---

## 6. Whether the macro can determine single-brand vs multi-brand correctly

### Issue

The plan implies a distinction between single-brand and multi-brand in the
macro's behavior, but each `impl_kind!` invocation only sees one brand at a
time. The macro has no visibility into other `impl_kind!` invocations for the
same target type. It cannot count how many brands map to `Result<A, E>`.

This means the macro cannot automatically determine whether a brand is
single-brand or multi-brand. The `#[multi_brand]` attribute is the only
signal, and as discussed in issue 5, it has no current behavioral effect on
Slot generation.

The real question is whether this matters. Under the Slot design, every
brand gets a Slot impl with `Marker = Val`, and the reference blanket
provides `Marker = Ref`. Coherence handles overlapping Slot impls because
the Brand parameter differs. The macro does not need to know the total
number of brands for a given target type.

The only scenario where single-vs-multi matters is if the plan wanted to
generate different code for single-brand types (e.g., an additional
convenience trait). The plan does not call for this.

### Approaches

**F1.** Accept that the macro cannot distinguish single-brand from
multi-brand and design accordingly. Each `impl_kind!` invocation generates
one Slot impl; coherence and the solver handle the rest.

- Trade-off: Clean separation of concerns. The macro is simple.

**F2.** Require `#[multi_brand]` for correctness (e.g., generate Slot only
when the attribute is present for multi-brand, and always for non-attributed
brands).

- Trade-off: This would mean single-brand types generate Slot without the
  attribute, and multi-brand types only generate Slot with the attribute.
  But this is backwards: the macro already generates a Slot impl by
  default (since it replaces InferableBrand which was also generated by
  default). Having multi-brand types require an opt-in attribute to get
  Slot impls would be a regression from InferableBrand, where the attribute
  _suppressed_ generation.

### Recommendation

F1. The macro does not need to distinguish single-brand from multi-brand for
correct Slot generation. Each invocation independently emits one Slot impl,
and the trait solver handles disambiguation at the call site. The
`#[multi_brand]` attribute remains a documentation marker.

---

## 7. Code generation volume and compile-time impact

### Issue

Today, each non-projection `impl_kind!` invocation generates one Kind impl
and optionally one InferableBrand impl. After the change, each generates one
Kind impl and one Slot impl. The Slot impl is slightly larger than the
InferableBrand impl (it has more generic parameters), but the count is
similar: roughly the same number of impls, replacing InferableBrand with
Slot.

Counting from the codebase: there are 54 `impl_kind!` invocations across 36
files. Of these, 10 have `#[no_inferable_brand]` (which currently suppresses
InferableBrand but under the new design would still generate Slot). The
projection-brand invocations (e.g., `BifunctorFirstAppliedBrand`,
`BifunctorSecondAppliedBrand`) whose target types contain `Apply!` will
continue to be skipped (approximately 4 invocations based on the bifunctor
and profunctor modules).

Net change in generated impls:

- Removed: ~44 InferableBrand impls (54 total minus 10 no_inferable_brand).
- Added: ~50 Slot impls (54 total minus ~4 projection skips).
- Net: approximately +6 trait impls.

Additionally, `trait_kind!` currently generates 2 items per invocation (Kind
trait + InferableBrand trait + blanket). After the change, it generates 3
items (Kind trait + Slot trait + Slot blanket). With 8 `trait_kind!`
invocations, that is +8 items.

The total increase is modest: roughly 14 additional trait items. This is
unlikely to cause measurable compile-time regression. The plan's Decision I
sets a generous threshold (50% regression, ~36s vs 24s baseline), and the
incremental cost of ~14 additional small trait items is negligible compared
to the baseline.

However, the Slot trait has three generic parameters (`'a`, `Brand`, `A`)
compared to InferableBrand's zero, which means the trait solver does more
work per Slot bound resolution. This is a per-call-site cost, not a
per-declaration cost. The number of call sites that use `map`, `bind`, etc.
is much larger than 54, but each call site already involves complex trait
resolution (FunctorDispatch with 6 type parameters). Adding a Slot bound
is incremental.

### Approaches

**G1.** Proceed with the plan as-is. Measure after implementation per
Decision I.

- Trade-off: If the regression exceeds the threshold, the remediation
  options (listed in Decision I) are available.

**G2.** Proactively skip Slot generation for brands that will never appear
in inference wrappers (e.g., optics profunctor brands like `ForgetBrand`,
`ShopBrand`, `MarketBrand`, `ExchangeBrand`, etc.). These brands are only
used in optics internals and never through `map` or `bind`.

- Trade-off: Reduces Slot count by ~10-15 but requires a new attribute
  or heuristic to identify "optics-only" brands. Premature optimization
  before measuring.

### Recommendation

G1. The code generation volume is modest. Measure first, optimize only if
the threshold is exceeded. The plan already identifies this approach.

---

## 8. Removal of InferableBrand generation from the macro pipeline

### Issue

The removal touches three sites:

1. `trait_kind_worker`: stop emitting the `InferableBrand_{hash}` trait and
   its `&T` blanket.
2. `impl_kind_worker`: stop emitting `InferableBrand_{hash}` impls (remove
   `should_generate_inferable_brand`, `build_inferable_brand_generics`, and
   the conditional block in `impl_kind_worker`).
3. The `generate_inferable_brand_name` function in the canonicalizer becomes
   dead code and should be removed.

Additionally, the `INFERABLE_BRAND_PREFIX` constant and all consumers in
`analysis/dispatch.rs`, `analysis/traits.rs`, and
`documentation/document_module.rs` must be updated (see issue 1).

The risk is incomplete removal: if any code path still references
`InferableBrand_*` after the change, the library will fail to compile (since
the traits no longer exist). This is actually a safety net; the compiler
will catch any missed references. The only subtle case is string-based
references (e.g., in `classify_trait` and `is_semantic_type_class`), which
would silently become dead branches rather than causing compile errors.

### Approaches

**H1.** Remove all InferableBrand generation and references in one pass.
Use `grep` for the string `InferableBrand` across the entire codebase to
ensure completeness.

- Trade-off: Most thorough. Small risk of missing a dynamic string
  construction, but the codebase uses constants for prefixes, so a search
  for both `InferableBrand` and `INFERABLE_BRAND` covers everything.

**H2.** Remove generation first, then fix compile errors iteratively.

- Trade-off: Slower but may surface unexpected dependencies. Given the
  codebase's use of constants, the dependency chain is well-defined.

### Recommendation

H1. The removal is mechanical and the codebase is well-structured with
constants. A single-pass removal with a full-text search verification is
sufficient. The compiler will catch any concrete type references, and a
text search for the string literal catches the string-based references.

---

## Summary of findings

| #   | Issue                                                         | Severity | Recommendation                                                                  |
| --- | ------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------- |
| 1   | Hash coordination needs SLOT_PREFIX + consumer updates        | Medium   | Add SLOT_PREFIX, update all 6 consumer sites in one pass.                       |
| 2   | trait_kind! Slot generation for multi-associated-type Kinds   | Low      | Skip Slot for multi-associated-type Kinds silently.                             |
| 3   | impl_kind! Slot impl synthesis requires generics merging      | Medium   | Write fresh generation function, reuse TypeIdentCollector.                      |
| 4   | Projection skip rule uses fragile string heuristic            | Low      | Switch to structural AST check (TypeMacro + path segment count).                |
| 5   | #[multi_brand] semantic shift is actually a no-op for codegen | Medium   | Clarify in plan that attribute is a documentation marker, not a codegen switch. |
| 6   | Macro cannot determine single-vs-multi-brand                  | None     | By design; Slot does not require this distinction.                              |
| 7   | Code generation volume is modest (+14 items)                  | None     | Measure after implementation per Decision I.                                    |
| 8   | InferableBrand removal has well-defined scope                 | Low      | Single-pass removal with full-text search verification.                         |
