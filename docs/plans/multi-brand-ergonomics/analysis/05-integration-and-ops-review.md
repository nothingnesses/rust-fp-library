---
title: Integration and Operations Review
reviewer: Agent 5
date: 2026-04-17
scope: Per-operation Slot shapes, non-Slot operations, dispatch modules, phasing, testing
---

# Integration and Operations Review

## 1. Per-operation Slot shape analysis

### 1.1 map (functor.rs) - Correct

The current `map` inference wrapper projects `<FA as InferableBrand>::Brand`.
The plan replaces this with `FA: Slot<Brand, A>` and projects
`<FA as Slot<Brand, A>>::Marker`. This matches the POC 5 validated design.
The FunctorDispatch trait itself is unchanged (it already takes Brand and
Marker as parameters). No issues.

### 1.2 bind, bind_flipped (semimonad.rs) - Correct

BindDispatch has the same shape as FunctorDispatch: `(Brand, A, B, FA, Marker)`.
The closure is `Fn(A) -> Of<B>` (Val) or `Fn(&A) -> Of<B>` (Ref). The closure
input type pins A, enabling the same closure-directed inference as map. POC 7
validates this. No issues.

### 1.3 join (semimonad.rs) - Needs attention

**Issue:** `join` has no closure. The current inference wrapper is:

```rust
pub fn join<'a, FA, A: 'a, Marker>(mma: FA) -> ...
where FA: InferableBrand + JoinDispatch<...>
```

The JoinDispatch impl for Val takes `Of<Of<A>>` and for Ref takes
`&Of<Of<A>>`. There is no closure input to disambiguate Brand for
multi-brand types. The plan lists `join` in phase 2 under semimonad
operations but does not call out that `join` is closureless.

**Approaches:**

a. Keep `join` inference-only for single-brand types; multi-brand `join`
requires `explicit::join`. Document this alongside `pure`/`alt`/`empty`.
Trade-off: simple, but creates inconsistency with `bind` which does
support multi-brand inference.

b. Have Slot resolve Brand from `Of<Of<A>>` via a nested Slot bound
(`FA: Slot<Brand, Of<A>>`). This could work if `Of<Of<A>>` is
concrete enough to select a unique Brand. Trade-off: plausible but
unvalidated; needs a POC.

c. Add `join` to the "cannot use Slot inference" list in the plan.
Trade-off: honest about limitations.

**Recommendation:** Approach (c). `join` structurally lacks a disambiguating
payload. It should be listed alongside `pure`, `alt`, `empty`, and
`sequence` in the "Operations that cannot use Slot inference" section. The
plan's phase 2 list should flag `join` as explicit-only for multi-brand.

### 1.4 compose_kleisli, compose_kleisli_flipped (semimonad.rs) - Needs attention

**Issue:** These take a tuple of closures `(F, G)` and a raw value `A`,
not a container `FA`. The current signatures require Brand as a turbofish:
`compose_kleisli::<Brand, _, _, _, _>(...)`. There is no `InferableBrand`
bound; they are already explicit-only. The plan lists them in phase 2
("bind, bind_flipped, join, compose_kleisli\*") implying they will get
Slot-based inference.

However, compose_kleisli takes a plain `A` value, not an `FA` container.
There is nowhere to attach a `Slot` bound. The closures do return `Of<B>`,
so in principle the return type could drive inference, but Slot is keyed on
`FA`, not on return types.

**Approaches:**

a. Leave compose_kleisli as explicit-only (already its current state).
The plan's mention of "compose_kleisli\*" in phase 2 only means
replacing `InferableBrand` references (there are none) with Slot
references (also none). In practice, no code change needed.

b. Add a Slot bound on the return container type of F. Non-standard and
not validated by any POC.

**Recommendation:** Approach (a). Clarify in the plan that compose_kleisli
and compose_kleisli_flipped are already explicit-only and remain so.
No Slot migration is needed for them since they have no InferableBrand usage.

### 1.5 bimap (bifunctor.rs) - Correct

BimapDispatch uses `Kind_266801a817966495` (arity-2 hash). The inference
wrapper projects `<FA as InferableBrand_266801a817966495>::Brand`. The
plan correctly identifies this as an arity-2 Slot. POC 6 validates the
pattern. No issues, but see issue 3 below about arity-2 multi-brand types.

### 1.6 fold_right, fold_left, fold_map (foldable.rs) - Correct

These have closures (`Fn(A, B) -> B`, `Fn(B, A) -> B`, `Fn(A) -> M`)
whose input type pins A, enabling closure-directed inference. The FnBrand
parameter is already explicit in the turbofish. No issues.

### 1.7 traverse (traversable.rs) - Correct

Traverse has a closure `Fn(A) -> F::Of<B>` that pins A. The FnBrand and
F (applicative brand) parameters are already explicit. Only Brand (the
traversable container's brand) moves from InferableBrand to Slot. The plan
correctly identifies this as "outer brand only." No issues.

### 1.8 filter, filter_map, partition, partition_map (filterable.rs) - Correct

All four have closures whose input type pins A. The Slot pattern applies
directly. No issues.

### 1.9 lift2 through lift5 (lift.rs) - Correct

Lift2Dispatch takes `Fn(A, B) -> C` whose inputs pin A and B. The
inference wrapper infers Brand from FA (first container). For multi-brand
types, the closure inputs disambiguate which Slot impl to use. No issues.

### 1.10 apply, ref_apply (semiapplicative) - No dispatch module exists

**Issue:** The plan (Decision H, phase 2) specifies that `apply` and
`ref_apply` should get Slot-based inference. POC 8 validates this.
However, there is currently **no dispatch module** for semiapplicative in
`fp-library/src/dispatch/`. The `apply` and `ref_apply` functions exist
only as class methods in `classes/semiapplicative.rs` and
`classes/ref_semiapplicative.rs`. There is no `ApplyDispatch` trait, no
Val/Ref dispatch, and no inference wrapper.

This means phase 2 for apply is not "repeat the phase 1 rebinding" as
the plan states. It requires creating a new dispatch module from scratch:
an `ApplyDispatch` trait, Val and Ref impls, an inference wrapper with
dual Slot bounds (per Decision H), and an explicit wrapper.

**Approaches:**

a. Create `dispatch/semiapplicative.rs` as part of phase 2. The POC 8
already validates the signature shape. Trade-off: more work than
implied by "mechanical analogue."

b. Defer `apply` inference to a follow-up. Trade-off: leaves asymmetry
that Decision H specifically avoids.

c. Move `apply` dispatch to phase 1 alongside `map` to test the
two-Slot-bound pattern early. Trade-off: phase 1 becomes larger.

**Recommendation:** Approach (a). The plan should acknowledge that
creating `dispatch/semiapplicative.rs` is new work, not a rebinding of
existing code. The complexity is moderate (the POC already defines the
exact signature), but it should be sized accordingly.

### 1.11 apply_first, apply_second (dispatch modules exist) - Needs attention

**Issue:** `apply_first` and `apply_second` have dispatch modules with
InferableBrand-based inference wrappers. The plan lists them in phase 2.
However, their dispatch shape is closureless: the container `fa` is
dispatched directly, not via a closure. ApplyFirstDispatch and
ApplySecondDispatch have `FA` and `FB` as associated types, with Marker
selected from whether FA is owned or borrowed.

For multi-brand types, neither apply_first nor apply_second has a
disambiguating closure. They are structurally similar to `alt` (two
containers, no closure).

**Approaches:**

a. Add apply_first and apply_second to the "cannot use Slot inference"
list. Trade-off: honest; users use explicit for multi-brand.

b. Attempt to infer Brand from the intersection of two Slot bounds on
FA and FB. Without a closure to pin A, this only works if FA alone
has a unique Brand (i.e., single-brand). Trade-off: no benefit for
multi-brand.

c. Leave them with Slot-based inference that works for single-brand
(replacing InferableBrand with Slot) but acknowledge multi-brand is
explicit-only. Trade-off: same as today but with Slot plumbing.

**Recommendation:** Approach (c). The plan's phase 2 migration for
apply_first and apply_second is mechanical (replace InferableBrand with
Slot) and works for single-brand. Multi-brand should be documented as
explicit-only, consistent with alt and join.

## 2. Operations the plan says cannot use Slot

### 2.1 pure (Pointed::pure)

Verified. `pure` is a class method returning `Of<A>`. There is no
container argument and no closure. It cannot use Slot. The do-notation
macros already rewrite `pure(x)` to `pure::<Brand, _>(x)` when a brand
is provided. Confirmed correct.

### 2.2 alt (dispatch/alt.rs)

Verified. AltDispatch takes two containers of the same type `FA`. No
closure. The inference wrapper uses `InferableBrand`. For multi-brand
types, there is no disambiguating payload. The plan correctly identifies
this as non-Slot. The inference wrapper will still be migrated to Slot
(single-brand works), but multi-brand requires explicit.

### 2.3 empty (Plus::empty)

Verified. `empty` is a class method returning `Of<A>` with no arguments.
Same situation as `pure`. Cannot use Slot.

### 2.4 sequence (Traversable)

Verified. The plan says the inner Brand is inferred from the container
shape but the outer Brand may remain ambiguous for multi-brand outer types.
This is correct. `sequence` is not in any dispatch module; it is typically
defined as `traverse(identity)`. For multi-brand outer containers,
explicit is required. Confirmed correct.

## 3. Multi-brand type coverage completeness

The plan mentions five multi-brand types: `Result`, `Pair`, `Tuple2`,
`ControlFlow`, `TryThunk`. The `#[no_inferable_brand]` grep confirms all
10 affected `impl_kind!` blocks (2 per type, one for each applied brand):

| Concrete type  | Brand 1 (arity 1)                  | Brand 2 (arity 1)                |
| -------------- | ---------------------------------- | -------------------------------- |
| Result<A, E>   | ResultErrAppliedBrand<E>           | ResultOkAppliedBrand<T>          |
| Pair<A, B>     | PairFirstAppliedBrand<First>       | PairSecondAppliedBrand<Second>   |
| (A, B)         | Tuple2FirstAppliedBrand<First>     | Tuple2SecondAppliedBrand<Second> |
| ControlFlow    | ControlFlowContinueAppliedBrand<C> | ControlFlowBreakAppliedBrand<B>  |
| TryThunk<A, E> | TryThunkErrAppliedBrand<E>         | TryThunkOkAppliedBrand<A>        |

**Issue:** All 10 brands are arity-1. However, some of these types also
have arity-2 brands (the bifunctor brands): `ResultBrand`, `PairBrand`,
`Tuple2Brand`, `ControlFlowBrand`. These arity-2 brands are NOT marked
`#[no_inferable_brand]` because at arity-2 they are the sole brand for
their concrete type. The plan correctly handles this: `bimap` uses
`InferableBrand_266801a817966495` (arity-2), and these arity-2 brands
have unique InferableBrand impls. The Slot migration at arity-2 should
generate a single Slot impl per brand. No coverage gap.

**Issue:** The plan's Impl landscape section (line 96-107) shows Slot
impls for `ResultErrAppliedBrand<E>` and `ResultOkAppliedBrand<T>` but
does not show corresponding impls for the other four multi-brand types.
This is an editorial gap, not a design gap; the pattern is the same.

**Recommendation:** Add a sentence to the Impl landscape section noting
the pattern applies to all five multi-brand types, not just Result.

## 4. Do-notation macros (m_do!, a_do!)

### 4.1 m_do! macro expansion

`m_do!` expands binds into either:

- `explicit::bind::<Brand, _, _, _, _>(expr, |param| { ... })` (when brand given)
- `bind(expr, |param| { ... })` (inferred mode)

In explicit mode (brand specified), the expansion already uses
`explicit::bind` with Brand as turbofish. This will continue to work
with Slot because `explicit::bind` will be rebound to Slot with Brand
pinned (Decision F pattern).

In inferred mode (no brand), the expansion calls the inference `bind()`
wrapper. For single-brand types this works. For multi-brand types, the
user must annotate closure parameter types for inference to succeed.
But in `m_do!`, the closure parameters are the bind patterns
(`x <- Some(5)` becomes `|x| { ... }`). The user CAN annotate:
`x: i32 <- Some(5)` becomes `|x: i32| { ... }`.

**Issue:** The plan's Decision K says to "audit m_do! and a_do! against
multi-brand types" but does not address whether inferred-mode m_do!
(no brand specified) could work with multi-brand types via type-annotated
bind patterns. In principle:

```rust
m_do!({
    x: i32 <- Ok::<i32, String>(5);
    pure(x + 1) // compile_error: pure needs a brand
})
```

This fails at the `pure(x)` call because inferred mode cannot resolve
pure's Brand. So inferred-mode m_do! with multi-brand types is blocked
not by `bind` inference but by `pure`. Users must use explicit mode
(`m_do!(Brand { ... })`).

**Recommendation:** No plan change needed. The audit (Decision K) will
discover this naturally. Document in the macro docs that multi-brand
m_do! requires explicit brand specification.

### 4.2 a_do! macro expansion

`a_do!` desugars into:

- 0 binds: `pure::<Brand, _>(expr)` or `compile_error!`
- 1 bind: `explicit::map::<Brand, _, _, _, _>(|param| ..., expr)` or `map(|param| ..., expr)`
- 2-5 binds: `explicit::liftN::<Brand, ...>(|params| ..., exprs)` or `liftN(|params| ..., exprs)`

In inferred mode, the 1-bind case expands to `map(|param| ..., expr)`.
With Slot-based inference and annotated closure parameters, this could
work for multi-brand types. The 2-5 bind case uses `liftN`, which also
has closures.

**Issue:** For `liftN`, the inference wrapper infers Brand from the
FIRST container (FA). With Slot, the closure input types must
disambiguate. `a_do!` generates closures like `|a, b| { ... }` where
`a` and `b` come from bind patterns. If users annotate:
`x: i32 <- Ok::<i32, String>(5)`, the macro emits `|x: i32, ...|`.
This should work.

However, `liftN`'s Slot bound is on FA only (the first container), not
on all containers. If the first container is multi-brand but the second
is single-brand, inference relies on FA's Slot alone. This is the same
as the general lift2 inference story.

**Recommendation:** No plan change needed, but the Decision K audit
should include a test case like:

```rust
a_do!(ResultErrAppliedBrand<String> {
    x <- Ok::<i32, String>(5);
    y <- Ok::<i32, String>(10);
    x + y
})
```

to verify explicit-mode a_do! works with multi-brand types.

## 5. Dispatch modules coverage

All 19 dispatch modules in `fp-library/src/dispatch/`:

| Module                    | Has InferableBrand | Plan mentions | Status             |
| ------------------------- | ------------------ | ------------- | ------------------ |
| functor.rs                | Yes                | Yes (phase 1) | OK                 |
| semimonad.rs              | Yes                | Yes (phase 2) | See 1.3, 1.4 above |
| bifunctor.rs              | Yes                | Yes (phase 2) | OK                 |
| foldable.rs               | Yes                | Yes (phase 2) | OK                 |
| traversable.rs            | Yes                | Yes (phase 2) | OK                 |
| filterable.rs             | Yes                | Yes (phase 2) | OK                 |
| lift.rs                   | Yes                | Yes (phase 2) | OK                 |
| alt.rs                    | Yes                | No            | See issue below    |
| apply_first.rs            | Yes                | Yes (phase 2) | See 1.11 above     |
| apply_second.rs           | Yes                | Yes (phase 2) | See 1.11 above     |
| compactable.rs            | Yes                | No            | See issue below    |
| contravariant.rs          | Yes                | No            | See issue below    |
| filterable_with_index.rs  | Yes                | No            | See issue below    |
| foldable_with_index.rs    | Yes                | No            | See issue below    |
| functor_with_index.rs     | Yes                | No            | See issue below    |
| traversable_with_index.rs | Yes                | No            | See issue below    |
| witherable.rs             | Yes                | No            | See issue below    |
| bifoldable.rs             | Yes                | Yes (phase 2) | OK                 |
| bitraversable.rs          | Yes                | Yes (phase 2) | OK                 |

**Issue:** The plan's "Will change" table lists the seven modules it
explicitly names but omits EIGHT dispatch modules that also use
InferableBrand:

1. `alt.rs` - uses InferableBrand in its inference wrapper.
2. `compactable.rs` - uses InferableBrand in its inference wrapper.
3. `contravariant.rs` - uses InferableBrand in its inference wrapper.
4. `filterable_with_index.rs` - uses InferableBrand in its inference wrapper.
5. `foldable_with_index.rs` - uses InferableBrand in its inference wrapper.
6. `functor_with_index.rs` - uses InferableBrand in its inference wrapper.
7. `traversable_with_index.rs` - uses InferableBrand in its inference wrapper.
8. `witherable.rs` - uses InferableBrand in its inference wrapper.

Since Decision D eliminates InferableBrand entirely, ALL of these must
be migrated to Slot. The plan cannot ship with InferableBrand references
remaining in any dispatch module.

**Approaches:**

a. Expand the plan's phase 2 list to enumerate all 19 dispatch modules.
Trade-off: completeness at the cost of a longer plan.

b. Add a blanket statement: "All dispatch modules in `src/dispatch/`
that reference InferableBrand will be migrated to Slot in phase 2."
Trade-off: brief but less auditable.

c. Split into two groups: closure-taking modules (benefit from
multi-brand inference) and closureless/container-only modules
(mechanical migration, no multi-brand inference benefit). Trade-off:
most informative.

**Recommendation:** Approach (c). The eight missing modules divide into:

- Closure-taking (benefit from Slot inference for multi-brand):
  `functor_with_index`, `foldable_with_index`, `filterable_with_index`,
  `traversable_with_index`, `witherable`, `compactable`, `contravariant`.
- Closureless (migrate InferableBrand to Slot mechanically, multi-brand
  stays explicit): `alt`.

## 6. Phasing analysis

### 6.1 Phase 1 -> Phase 2 ordering

Phase 1 delivers Slot trait family, trait_kind! and impl_kind! macro
updates, InferableBrand removal, and map rebinding. Phase 2 extends
to remaining operations.

**Issue:** Phase 1 step 5 removes InferableBrand entirely. But at that
point, only `map` has been rebound (step 6). All other dispatch modules
still reference InferableBrand. This means after step 5, the codebase
does not compile until ALL dispatch modules are migrated.

**Approaches:**

a. Move InferableBrand removal to the end of phase 2, after all
dispatch modules are migrated. Trade-off: InferableBrand and Slot
coexist during development, which the plan says is unnecessary
(Decision D).

b. Migrate all dispatch modules in phase 1 (making phase 1 larger and
phase 2 smaller). Trade-off: phase 1 becomes "replace everything"
rather than an incremental checkpoint.

c. Keep the current ordering but acknowledge that step 5 through the end
of phase 2 is a single non-compiling delta. Since all phases ship
together (Decision B3), this is acceptable; the phases are internal
milestones only. Trade-off: developers cannot checkpoint at the
end of phase 1.

**Recommendation:** Approach (a). Moving InferableBrand removal to after
all dispatch modules are migrated allows compilation at each phase
boundary. This is better for development workflow even under Decision B3,
because it allows running the test suite incrementally. The plan should
restructure so that:

- Phase 1: Add Slot family, update macros, keep InferableBrand as
  compatibility layer, rebind `map` to Slot.
- Phase 2: Rebind all remaining dispatch modules to Slot.
- Phase 2 final step: Remove InferableBrand.
- Phase 3: Diagnostics and docs (unchanged).

### 6.2 trait_kind! -> impl_kind! ordering

Phase 1 step 2 (trait_kind! emits Slot) must precede step 3 (impl_kind!
emits Slot impls). This is correctly ordered.

### 6.3 Attribute rename timing

Phase 1 step 4 renames `#[no_inferable_brand]` to `#[multi_brand]`. This
affects impl_kind! macro input. It must happen before or simultaneously
with step 3 (impl_kind! changes). The plan has it at step 4, after
step 3. This creates a question: does step 3 (impl_kind! changes) read
the old attribute name or the new one?

**Recommendation:** Combine steps 3 and 4 into a single step: update
impl_kind! to recognize `#[multi_brand]`, generate Slot impls, and
simultaneously rename all use sites. This avoids an intermediate state
where impl_kind! expects one attribute name and the code has the other.

## 7. Benchmarks

The plan's "Unchanged" section states: "Benchmarks: no code changes.
Performance validated post-implementation."

**Issue:** The benchmarks likely use InferableBrand-based inference
wrappers (e.g., `map(f, some_value)`). After replacing InferableBrand
with Slot, these calls will use Slot-based inference. The benchmark code
may need no source changes if the API is backward-compatible for
single-brand types, but this should be verified.

More importantly, the plan does not call for benchmarking multi-brand
dispatch to verify it has the same zero-cost property as single-brand
dispatch. If Slot introduces additional trait resolution overhead at
compile time, it will not affect runtime, but if the Slot impl selection
introduces different codegen (e.g., different monomorphization paths),
that should be measured.

**Approaches:**

a. Add a benchmark for `map(|x: i32| x + 1, Ok::<i32, String>(5))`
and compare it against `explicit::map::<ResultErrAppliedBrand<String>,
   ...>(...)`. Trade-off: small effort, validates zero-cost.

b. Rely on the existing Criterion benchmarks plus manual inspection of
generated assembly. Trade-off: less systematic.

c. Do nothing; trust that `impl Fn` dispatch is zero-cost regardless
of how Brand is resolved. Trade-off: reasonable given the library's
design, but unverified for the new Slot path.

**Recommendation:** Approach (a). Add one benchmark comparing
multi-brand inference dispatch against explicit dispatch. A single
`map` benchmark on `Result` suffices to validate that Slot-based
inference produces identical codegen.

## 8. Property-based tests (QuickCheck)

The plan does not mention property-based tests.

**Issue:** The existing property-based tests (if any exist for Result
or other multi-brand types) likely use `explicit::` paths since
InferableBrand does not support multi-brand. After this plan ships,
property-based tests should also exercise the new inference path.

However, the grep for `prop_` in `result.rs` returned no matches.
Property-based tests may be concentrated in other files or may not
exist for multi-brand types at all.

**Approaches:**

a. Add property-based tests that verify type class laws (Functor,
Monad, etc.) for multi-brand types through the inference path.
For example: `prop_functor_identity_result_ok` that calls
`map(|x: i32| x, Ok::<i32, String>(v))` and asserts identity.
Trade-off: thorough, good regression coverage.

b. Rely on unit tests in phase 1 step 9 ("integration tests covering
every Val/Ref x single/multi-brand cell"). Trade-off: covers
correctness but not law-level properties.

c. Defer property tests to a follow-up. Trade-off: faster shipping.

**Recommendation:** Approach (b) for initial delivery. The plan's
phase 1 step 9 integration tests are sufficient for shipping. Property-
based tests for multi-brand inference can be added as a follow-up
since the underlying type class implementations are already law-tested
through explicit dispatch.

## Summary of findings

| #   | Issue                                             | Severity | Section |
| --- | ------------------------------------------------- | -------- | ------- |
| 1   | join is closureless; should be explicit-only      | Medium   | 1.3     |
| 2   | compose_kleisli has no InferableBrand to migrate  | Low      | 1.4     |
| 3   | No dispatch module for semiapplicative/apply      | Medium   | 1.10    |
| 4   | apply_first/apply_second closureless for multi    | Low      | 1.11    |
| 5   | Eight dispatch modules missing from plan          | High     | 5       |
| 6   | InferableBrand removal blocks compilation early   | High     | 6.1     |
| 7   | Attribute rename timing vs impl_kind! update      | Low      | 6.3     |
| 8   | No multi-brand benchmark planned                  | Low      | 7       |
| 9   | Impl landscape section only shows Result examples | Low      | 3       |
