---
title: Inference Edge Cases Review
reviewer: Agent 4
date: 2026-04-17
scope: Closure inference ordering, diagonal cases, generic contexts, nested containers, Ref interaction
---

# Inference Edge Cases Review

## 1. Closure type inference ordering: when does Rust commit A from the closure vs from the container?

### 1.1 Inference ordering is sound for the standard case

The plan's inference mechanism works as follows: FA pins from the
container argument, Marker projects from Slot (Val or Ref), then
FunctorDispatch selects the Val or Ref impl, whose `Fn(A) -> B` or
`Fn(&A) -> B` bound constrains A from the closure's parameter type.
Finally, with A known, the Slot impl uniquely determines Brand.

This ordering is validated by all POC tests. Rust's trait solver does
not enforce a strict left-to-right order; it accumulates constraints
and resolves them as a system. The critical property is that A appears
in both the closure bound and the Slot bound, so once A is pinned from
the closure annotation, Brand resolves.

### 1.2 Issue: unannotated closures with multi-brand types and coercible A

When the closure body could type-check under multiple A choices, Rust
cannot commit. The plan documents this for identical types
(`Result<T, T>`) but does not discuss the case where A is inferred
from the closure body rather than an explicit annotation.

Consider:

```rust
map(|x| x.to_string(), Ok::<i32, String>(5))
```

Here `x.to_string()` works for both `i32` (via `ToString`) and
`String` (via `ToString`). Without an annotation on `x`, Rust sees two
candidate Slot impls and cannot commit A.

**Finding:** This is correctly handled. The plan (Decision C) documents
that multi-brand types require closure-input annotations. The POC
(`slot_production_poc.rs`, commented-out
`annotation_matrix_multi_brand_annotation_on_call_return_type` test)
explicitly confirms that return-type annotation alone is insufficient.
No gap here.

### 1.3 Issue: single-brand types with deferred closure inference

For single-brand types like `Option`, the annotation matrix test in
`slot_production_poc.rs` confirms unannotated closures work because
there is only one Slot impl, so A is forced. This remains true after
the transition from InferableBrand to Slot. No issue.

## 2. Diagonal and near-diagonal cases

### 2.1 True diagonal: `Result<T, T>`

The plan correctly identifies `Result<T, T>` (and `Pair<T, T>`,
`(T, T)`, `ControlFlow<T, T>`, `TryThunk<T, T>`) as ambiguous. Both
Slot impls match, the closure annotation does not disambiguate, and
`explicit::map` is the prescribed fallback. The POC
(`slot_production_poc.rs`, commented-out
`diagonal_result_t_t_is_ambiguous`) confirms E0283.

**Finding:** The diagonal case is handled correctly. No gap.

### 2.2 Issue: near-diagonal where A and the fixed parameter are distinct but confusable by the user

Consider `Result<i32, u32>`. This is NOT a diagonal case from Rust's
perspective; `i32 != u32`, so only one Slot impl matches a given
closure annotation. For example:

```rust
map(|x: i32| x + 1, Ok::<i32, u32>(5))  // ResultErrAppliedBrand<u32>
map(|x: u32| x + 1, Err::<i32, u32>(5)) // ResultOkAppliedBrand<i32>
```

Both compile unambiguously. The risk is purely a user confusion issue:
a user might annotate the wrong type and get a confusing error about
missing `Functor` or `Slot` impls. Decision J's diagnostic message
("annotate the closure input type; if that doesn't disambiguate, use
`explicit::map`") partially addresses this but does not help the user
who annotated the wrong type and gets a trait-not-satisfied error
rather than an ambiguity error.

**Severity:** Low. The error message from Rust will at minimum show
which Slot impls exist. The user can see both
`ResultErrAppliedBrand<u32>` and `ResultOkAppliedBrand<i32>` in the
candidate list and correct their annotation.

**Recommendation:** No code change needed. Consider adding a note to
the documentation (phase 3) explaining that for multi-brand types, the
closure annotation selects which "slot" is being mapped over, with an
example showing both directions.

### 2.3 Issue: near-diagonal with generic fixed parameter

```rust
fn process<E>(r: Result<i32, E>) {
    map(|x: i32| x + 1, r)
}
```

Here `E` is generic. The Slot impls are:

- `ResultErrAppliedBrand<E>` with A = i32 (matches: A is concrete).
- `ResultOkAppliedBrand<i32>` with A = E (matches if E = i32).

Because `E` is unconstrained, Rust cannot rule out `E = i32`, so both
Slot impls are potentially applicable. The solver should select
`ResultErrAppliedBrand<E>` because the closure concretely pins
`A = i32`, which matches only the first impl without needing
`E = i32`. However, Rust's coherence checker may conservatively refuse
to commit.

**Severity:** Medium. This is a realistic scenario (generic error types
are common in Result-heavy code). If Rust refuses this, users must use
`explicit::map::<ResultErrAppliedBrand<E>, ...>` even for
straightforward generic-error cases.

**Approaches:**

1. **Accept and document.** If Rust's solver handles this (it likely
   does, because the closure pins A = i32 concretely, and
   `ResultOkAppliedBrand<i32>` requires A = E which is not forced),
   no action is needed. If it does not, document that generic
   fixed-parameter cases may need `explicit::`.
2. **Add a targeted POC.** Write a test specifically for
   `fn foo<E>(r: Result<i32, E>)` with `map(|x: i32| ..., r)` to
   confirm whether Rust's solver commits. This is low cost and
   eliminates uncertainty.
3. **Provide a where-clause helper.** If inference fails, a helper
   like `map_ok(f, r)` would bypass Slot entirely. This conflicts
   with the plan's "out of scope" stance on named helpers.

**Recommendation:** Approach 2. Add a POC test for the generic
fixed-parameter case before implementation begins. If it passes, no
further action. If it fails, add a note to the coverage matrix (row:
"Val + multi-brand + generic fixed param -> may need `explicit::`").

## 3. Interaction between closure-directed Brand inference and Val/Ref Marker inference

### 3.1 Marker projection independence

The plan's key insight is that Marker projects from FA alone via
Slot's associated type, before Brand or A are resolved. This means
Val/Ref selection and Brand selection are decoupled:

- For `&Result<i32, String>`, the `&T` blanket immediately yields
  `Marker = Ref`.
- For `Result<i32, String>`, the direct impl yields `Marker = Val`.

Once Marker is committed, FunctorDispatch has a unique matching impl
(Val or Ref), and the closure's `Fn(A) -> B` or `Fn(&A) -> B` shape
pins A.

**Finding:** The Marker-via-Slot design correctly decouples Val/Ref
from Brand resolution. POC 5 (`slot_marker_via_slot_poc.rs`)
validates all four cells of the matrix. No issue.

### 3.2 Issue: double reference (`&&Result<i32, String>`)

The `&T` blanket for Slot is:

```rust
impl<T: ?Sized, Brand, A> Slot<Brand, A> for &T
where T: Slot<Brand, A>
```

For `&&Result<i32, String>`, this applies twice:

- `&&Result<i32, String>`: Marker = Ref (via blanket on `&T` where
  `T = &Result<i32, String>`).
- `&Result<i32, String>`: Marker = Ref (via blanket on `&T` where
  `T = Result<i32, String>`).

The Slot chain resolves correctly. However, FunctorDispatch's Ref impl
takes `&'b Apply!(Brand::Of<A>)` which is `&Result<i32, String>`, not
`&&Result<i32, String>`. The FA parameter in the unified `map` is
`&&Result<i32, String>`, but the Ref impl's FA is
`&Apply!(Brand::Of<A>)`.

**Severity:** Low. Double references are unusual at call sites, and
Rust's auto-deref typically collapses `&&T` to `&T` in practice.
The existing system (InferableBrand) has the same limitation. If a
user passes `&&result`, Rust will try to match and likely fail with a
type mismatch on FA; the error message will be clear.

**Recommendation:** No code change needed. This is a pre-existing
limitation unrelated to the Slot transition.

### 3.3 Ref dispatch and Slot's A parameter interaction

In the Ref case, FunctorDispatch's closure is `Fn(&A) -> B`. The `&`
is part of the closure's function signature, not part of A. Slot
resolves A from the inner type (e.g., for `&Result<i32, String>`, A
resolves to `i32` or `String` depending on the brand), and the Ref
impl's `Fn(&A) -> B` means the closure receives `&i32` or `&String`.

The question is whether `Fn(&A) -> B` correctly constrains A. Yes: if
the user writes `|x: &i32| ...`, Rust infers A = i32 from the `&i32`
pattern. This is confirmed by all Ref POC tests.

**Finding:** No issue. The interaction is correct and well-tested.

## 4. Generic contexts

### 4.1 Issue: container type is itself generic

```rust
fn process<F: Functor>(fa: F::Of<i32>) {
    // map(|x: i32| x + 1, fa)  -- no Slot impl for F::Of<i32>
}
```

This scenario uses the brand directly (F is the brand). Slot-based
inference requires an impl `Slot<Brand, A> for ConcreteType`. When
the container is an associated-type projection (`F::Of<i32>`), there
is no Slot impl available. The user must use
`explicit::map::<F, ...>(|x: i32| x + 1, fa)`.

**Finding:** This is correct behavior and matches the plan's design.
Slot impls are generated per-brand for concrete types; fully generic
HKT code always uses `explicit::` or calls trait methods directly.
This is no different from the current InferableBrand system. No gap.

### 4.2 Issue: partially generic container

```rust
fn process<T>(x: Result<T, String>) {
    map(|t: T| format!("{t:?}"), x)  // requires T: Debug
}
```

Here `A = T` is pinned by the closure annotation. `Result<T, String>`
has Slot impls for `ResultErrAppliedBrand<String>` (A = T) and
`ResultOkAppliedBrand<T>` (A = String). The closure pins A = T.

For `ResultErrAppliedBrand<String>`: A = T matches.
For `ResultOkAppliedBrand<T>`: A = String, which requires T = String.
Since T is unconstrained, Rust cannot rule out T = String.

This is structurally identical to issue 2.3. The solver must decide
whether the concrete closure annotation A = T is sufficient to prefer
`ResultErrAppliedBrand<String>` over `ResultOkAppliedBrand<T>`.

**Severity:** Medium. Same as 2.3; generic code over multi-brand
types is common.

**Recommendation:** Same as 2.3; a single POC covering both `fn
process<E>(r: Result<i32, E>)` and `fn process<T>(r: Result<T,
String>)` would resolve the uncertainty.

## 5. Higher-order cases

### 5.1 Closures returning closures

```rust
map(|x: i32| move |y: i32| x + y, Some(5))
// Result: Option<impl Fn(i32) -> i32>
```

This works because Slot resolves from `Some(5)` -> `Option<i32>` ->
A = i32, B = `impl Fn(i32) -> i32`. The closure's return type being a
closure is irrelevant to Slot resolution. B is unconstrained by Slot
(it only participates in the return type of map). Single-brand works
trivially.

For multi-brand:

```rust
map(|x: i32| move |y: i32| x + y, Ok::<i32, String>(5))
```

A = i32 pins `ResultErrAppliedBrand<String>`. The return type
`Result<impl Fn(i32) -> i32, String>` is well-formed. No issue.

**Finding:** Closures returning closures are handled correctly.

### 5.2 Issue: nested containers

```rust
map(|x: Option<i32>| x.unwrap_or(0), Some(Some(5)))
// A = Option<i32>, B = i32
```

For single-brand (`Option<Option<i32>>`), A = `Option<i32>` is
inferred from the outer Option's Slot impl. This works because Slot
keys on the outer container's A, not the inner container.

For multi-brand nested containers:

```rust
map(|inner: Result<i32, String>| inner.unwrap_or(0), Some(Ok::<i32, String>(5)))
```

This maps over the outer `Option`, which is single-brand. A =
`Result<i32, String>`. No Slot ambiguity because the outer container
is `Option`.

The more interesting case:

```rust
map(|x: i32| x + 1, Ok::<Option<i32>, String>(Some(5)))
```

Here the outer container is `Result<Option<i32>, String>`. The Slot
impls are:

- `ResultErrAppliedBrand<String>` with A = `Option<i32>`. But the
  closure annotates A = i32.
- `ResultOkAppliedBrand<Option<i32>>` with A = String. Closure says
  A = i32 which requires String = i32. Does not unify.

Neither Slot impl matches A = i32. This call would fail to compile.
The correct call would be:

```rust
map(|x: Option<i32>| x.map(|y| y + 1), Ok::<Option<i32>, String>(Some(5)))
```

**Finding:** This is correct behavior. The outer map operates on the
outer container's element type (`Option<i32>` or `String`), not the
inner element. Users who want to reach the inner i32 must compose maps
or use optics. No gap.

### 5.3 Issue: nested multi-brand in bind's return type

```rust
bind(Ok::<i32, String>(5), |x: i32| Ok::<String, String>(x.to_string()))
```

Here the closure returns `Result<String, String>`, which is a diagonal
Result. However, this does not cause ambiguity in the Slot resolution
for the outer container. The bind signature constrains the return type
to `Brand::Of<B>` where Brand is already committed from the input
container's Slot (`ResultErrAppliedBrand<String>`, pinned by A = i32).
So `Brand::Of<B>` = `Result<B, String>` = `Result<String, String>`.
The diagonal nature of the return value is irrelevant; Brand is already
committed.

**Finding:** Nested diagonal in bind's return type is not an issue.
Brand resolution happens on the input container, not the output.

## 6. Fn(&A) -> B and Slot's A parameter

### 6.1 The Ref impl correctly routes through A, not &A

The FunctorDispatch Ref impl (line 152-193 of functor.rs) bounds the
closure as `Fn(&A) -> B`. The FA parameter is
`&'b Apply!(Brand::Of<A>)`. Slot resolves with the underlying type's
A parameter, not `&A`.

When the user writes `|x: &i32| *x + 1`, Rust sees `Fn(&i32) -> i32`
and matches `Fn(&A) -> B` with A = i32, B = i32. Slot then resolves
with A = i32 on the inner (non-reference) type.

**Finding:** Correct. The POC tests (e.g., `ref_result_ok_mapping` in
`slot_marker_via_slot_poc.rs`) validate this for multi-brand Ref
dispatch.

### 6.2 Issue: closure annotated as `|x: &A|` where A is generic

```rust
fn process<A: 'static>(r: &Result<A, String>) {
    map(|x: &A| format!("{x:?}"), r)
}
```

This combines the generic-container issue (section 4.2) with Ref
dispatch. The Ref blanket commits Marker = Ref. Then `Fn(&A) -> B`
pins A from the closure. But A is generic, and `ResultOkAppliedBrand<A>`
with inner-A = String is also a candidate (requiring A = String).

This is the same ambiguity as 2.3/4.2, just through the Ref path.
The Ref blanket does not introduce additional ambiguity; it simply
delegates to the inner type's Slot impls.

**Recommendation:** Covered by the same POC recommended in 2.3.

## 7. Order-dependent and fragile inference

### 7.1 Argument order: closure before container

In the current `map` signature:

```rust
pub fn map<FA, A, B, Brand>(
    f: impl FunctorDispatch<..., <FA as Slot<Brand, A>>::Marker>,
    fa: FA,
)
```

The closure `f` appears before the container `fa`. Rust's type
inference does not depend on argument evaluation order for inference
purposes; all constraints are accumulated before solving. The order of
parameters in the function signature does not affect whether inference
succeeds.

**Finding:** No issue with argument order.

### 7.2 Issue: let-binding splits inference context

```rust
let f = |x: i32| x + 1;
map(f, Ok::<i32, String>(5))
```

When the closure is pre-bound to a variable, Rust infers its type as
`impl Fn(i32) -> i32` at the let-binding site. At the `map` call
site, `f`'s type is fully known, so A = i32 is still available.

```rust
let f = |x| x + 1;  // A is not yet pinned
map(f, Ok::<i32, String>(5))
```

Here `f`'s type is `impl Fn({integer}) -> {integer}` at the let site.
At the `map` call, Rust may or may not propagate the constraint back.
In practice, Rust defers closure inference when the closure is used as
a function argument, but a pre-bound closure without annotation loses
this deferral.

**Severity:** Low. This affects all Rust code with pre-bound closures,
not just this library. The workaround is to annotate the closure
parameter, which the plan already requires for multi-brand types.

**Recommendation:** Document in phase 3 that pre-bound closures for
multi-brand types should include parameter annotations.

### 7.3 Issue: turbofish on map bypassing Slot

The plan's `explicit::map` signature (Decision F) is:

```rust
pub fn map<Brand: Kind, A, B, FA, Marker>(
    f: impl FunctorDispatch<Brand, A, B, FA, Marker>,
    fa: FA,
)
```

With turbofish `explicit::map::<Brand, _, _, _, _>(f, fa)`, Brand is
pinned directly. The Slot bound is no longer needed for Brand
resolution; it is only needed for Marker projection. If the explicit
signature still bounds on Slot (as the plan describes in Decision F),
then Marker can still project. If the explicit signature drops the
Slot bound and threads Marker through turbofish, that would change the
turbofish shape.

The current `explicit::map` (pre-Slot) already takes Brand as the
first turbofish argument and infers the rest. The post-Slot version
should behave identically.

**Finding:** The explicit path correctly covers every case the
inference path cannot, including diagonals, generic fixed-parameters
(if inference fails), and unannotated multi-brand closures. The
turbofish shape `explicit::map::<Brand, _, _, _, _>` is preserved.
No gap.

## 8. The `'static` bound on the fixed parameter

### 8.1 Issue: `E: 'static` in Slot impls prevents non-static error types

The Slot impls for Result use:

```rust
impl<A: 'a, E: 'static> Slot<ResultErrAppliedBrand<E>, A> for Result<A, E>
impl<T: 'static, A: 'a> Slot<ResultOkAppliedBrand<T>, A> for Result<T, A>
```

The `'static` bound on the fixed parameter (E or T) comes from the
brand definition (e.g., `ResultErrAppliedBrand<E>` requires E: 'static
because brand types are marker types that cannot carry lifetimes).
This means `Result<i32, &str>` cannot use the inference path for
ok-mapping because `&str` is not `'static`.

**Severity:** Medium. `&str` error types are uncommon in production
code (most errors are owned), but `Result<&'a str, E>` (non-static
success type) would also be blocked from err-mapping.

**Finding:** This is a pre-existing limitation of the Brand pattern,
not introduced by Slot. The current system has the same `'static`
bounds on `Kind` impls. The Slot transition does not change this. It
is worth noting in the documentation but is not a blocker or
regression.

**Recommendation:** No action needed for the Slot plan. If the
`'static` bound is relaxed on brands in the future (a separate
effort), Slot impls would automatically benefit.

## 9. `filter` and `fold` closure shapes

### 9.1 filter: `Fn(&A) -> bool` vs `Fn(A) -> bool`

`filter`'s closure returns `bool`, not a container. The closure still
takes A (Val) or &A (Ref), so A is still pinned by the closure
annotation. Slot resolution proceeds identically to `map`.

**Finding:** No issue specific to `filter`'s closure shape.

### 9.2 fold: accumulator does not participate in Slot

`fold_right`'s closure is `Fn(A, B) -> B` (or `Fn(&A, B) -> B` for
Ref). A is the element type from the container; B is the accumulator.
Only A participates in Slot resolution. B is unconstrained by Slot
and is inferred from the accumulator argument and closure return type.

**Finding:** No issue. The accumulator type is orthogonal to Slot.

### 9.3 fold_map: `Fn(A) -> M` where M: Monoid

`fold_map` returns a Monoid value, not a container. The closure takes
A, pinning it for Slot resolution. The return type M is independent
of Brand. Slot resolution works the same as `map`.

**Finding:** No issue.

## 10. Do-notation macro interaction

### 10.1 Issue: m_do! / a_do! and closure annotations

Decision K prescribes auditing do-notation macros. The key question is
whether the macro-generated closures carry enough type information for
Slot resolution with multi-brand types.

If `m_do!` desugars to nested `bind` calls, each closure in the chain
needs A annotated for multi-brand types. Depending on how the macro
generates closures, annotations may or may not survive.

**Severity:** Medium. Do-notation is a primary ergonomic feature.
If it cannot support multi-brand types without manual intervention, the
asymmetry would be surprising.

**Approaches:**

1. **Audit only (Decision K's current stance).** Run existing tests,
   add multi-brand tests, document limitations.
2. **Emit type annotations in macro expansion.** The macro could
   inspect the container type and insert closure parameter annotations
   when it detects a multi-brand type (via the `#[multi_brand]`
   attribute or a similar marker).
3. **Accept that do-notation uses `explicit::` for multi-brand.** The
   macro could emit `explicit::bind` calls with Brand turbofish when
   the type is multi-brand.

**Recommendation:** Approach 1 first (audit). If the audit reveals
that multi-brand do-notation is unusable without annotations, consider
approach 3 as a targeted fix.

## Summary of issues

| #    | Issue                                                        | Severity | Recommendation                                  |
| ---- | ------------------------------------------------------------ | -------- | ----------------------------------------------- |
| 2.3  | Generic fixed parameter (`Result<i32, E>`) may be ambiguous  | Medium   | Add targeted POC before implementation.         |
| 4.2  | Partially generic container (`Result<T, String>`) same issue | Medium   | Same POC covers this.                           |
| 6.2  | Generic Ref variant of 2.3/4.2                               | Medium   | Same POC covers this.                           |
| 8.1  | `'static` bound on fixed parameter blocks non-static types   | Medium   | Pre-existing; document but no code change.      |
| 10.1 | Do-notation macros may not carry annotations for multi-brand | Medium   | Audit per Decision K; consider macro-level fix. |
| 2.2  | Near-diagonal user confusion                                 | Low      | Documentation note in phase 3.                  |
| 3.2  | Double reference (`&&T`)                                     | Low      | Pre-existing limitation; no change needed.      |
| 7.2  | Pre-bound closures lose deferred inference                   | Low      | Document in phase 3.                            |

All medium-severity issues have clear resolution paths. The single
highest-priority action item is adding a POC for the generic
fixed-parameter case (issues 2.3, 4.2, 6.2), which would either
confirm the solver handles it or surface a concrete limitation to
document.
