# Brand Inference Plan: Delta Analysis

Summary of what changed after the ref-hierarchy remediation and ref-borrow
refactor, what remains valid, what needs updating, and recommended changes.

## Overall Assessment

The brand-inference plan is **more feasible now than when it was written**.
The ref-borrow refactor's two-impl dispatch pattern with `FA` as a type
parameter is a natural fit for brand inference, making the implementation
simpler and more capable. The core mechanism (DefaultBrand trait) is sound.
One significant limitation was discovered: `pure` cannot use brand inference.

## What Changed

### Improvements (things that got easier)

1. **Ref dispatch now works with a single function.** The original POC could
   only do Val dispatch. The plan noted "Ref dispatch would require a
   separate function." With `FA` as a dispatch trait parameter and a blanket
   `impl DefaultBrand for &T`, a single `map` function handles both Val
   and Ref dispatch. Validated by 18 passing tests.

2. **The `Into` bound is eliminated.** The old POC required
   `FA: Into<Brand::Of<A>>` to convert the concrete type to the GAT
   projection. Now `FA` flows directly into the dispatch trait. No
   conversion needed. This simplifies both the signature and the impl.

3. **The GAT equality constraint may be unnecessary.** With `FA` as a
   dispatch trait parameter, the dispatch impl selection already constrains
   `FA` to the correct projection type. The explicit
   `Kind<Of<'a, A> = FA>` bound may be redundant. (Keeping it is safe;
   removing it needs testing.)

### Issues discovered

1. **`pure` cannot use brand inference (E0283).** Rust cannot infer the
   `Brand` parameter of `pure` from return-type context, even inside a
   `bind` closure that constrains the return type. This is a fundamental
   limitation of Rust's type inference with GAT projections. Impact:
   - `pure` keeps its current signature with explicit Brand turbofish.
   - `pure` is NOT renamed to `pure_explicit`.
   - In inferred-mode macros (`m_do!({ ... })`), `pure(expr)` cannot be
     rewritten without a brand. Users must write concrete constructors
     (e.g., `Some(expr)`) or use the explicit-brand macro syntax.

2. **`compose_kleisli` and `compose_kleisli_flipped` cannot use inference.**
   These take `(fg, a)` where `a: A` is a plain value, not a container.
   No `FA` to resolve `DefaultBrand` from. These keep explicit Brand.

### Things that stay the same

1. **Multi-brand types (Result, Pair, Tuple2, ControlFlow, TryThunk) still
   cannot use arity-1 inference.** This is inherent to the design. Arity-2
   inference via bifunctor `DefaultBrand` is still viable.

2. **Generic code still requires explicit brands.** `DefaultBrand` resolves
   at concrete types only. Library internals still parameterize over Brand.

3. **`impl_kind!` generation** approach is unchanged. Generate `DefaultBrand`
   by default, opt out with `#[no_default_brand]`.

4. **`#[diagnostic::on_unimplemented]`** for error messages is still needed
   and still works.

## Plan Sections That Need Updating

### Must update

| Section                                   | Issue                                                              | Fix                                                         |
| ----------------------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------- |
| "Inference-Based Free Functions"          | Signature uses old dispatch (no `FA` param), includes `Into` bound | Rewrite with the validated signature from poc-validation.md |
| "The GAT Equality Bound Must Be Verified" | Marked as "unverified assumption"                                  | Mark as verified. Bound may be unnecessary but is harmless. |
| "Interaction with FunctorDispatch"        | Omits `FA` parameter, describes old dispatch                       | Rewrite to describe both Brand and Marker flowing from `FA` |
| "Interaction with Dispatch Extensions"    | Doesn't address FnBrand complication                               | Add per-trait analysis from dispatch-compatibility.md       |
| "Why pure Cannot Use Brand Inference"     | Speculative ("Rust can sometimes do via return type")              | Confirmed: pure CANNOT use inference. Remove hedging.       |
| POC Results                               | Outdated (Val-only, Into-based)                                    | Replace with new POC results (18 tests, both Val and Ref)   |
| Turbofish examples                        | Use old turbofish counts (pre-FA)                                  | Update all turbofish counts                                 |
| Implementation Steps 5-6                  | Use old signature, old dispatch                                    | Rewrite with new signature                                  |
| Naming Convention                         | Says `pure` stays as-is; correct but now confirmed                 | Add explicit note that this was validated by POC failure    |

### Should add

| Topic                              | Why                                                          |
| ---------------------------------- | ------------------------------------------------------------ |
| Blanket `impl DefaultBrand for &T` | Required for Ref dispatch. Not in current plan.              |
| Per-dispatch-trait analysis        | FnBrand/F complications for foldable/traversable.            |
| `compose_kleisli` exclusion        | Cannot use inference (no container arg).                     |
| `pure` in inferred-mode macros     | Confirmed failure; document concrete-constructor workaround. |
| Macro parser changes               | `{` detection for inferred mode, all 4 syntax combinations.  |
| Test file reference                | Point to `brand_inference_feasibility.rs` as the new POC.    |

## Recommended Plan Changes

### Change 1: Add `DefaultBrand for &T` to Step 2

When defining the `DefaultBrand` trait, include the blanket reference impl
as a one-line addition:

```rust
impl<'a, T: DefaultBrand + ?Sized> DefaultBrand for &'a T {
    type Brand = T::Brand;
}
```

This is required for Ref dispatch brand inference. Without it, only Val
dispatch works.

### Change 2: Update the free function signature

Replace the plan's proposed signature:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>,
    fa: FA,
) -> <<FA as DefaultBrand>::Brand as Kind>::Of<'a, B>
where
    FA: DefaultBrand + 'a,
    <FA as DefaultBrand>::Brand: Kind<Of<'a, A> = FA>,
```

With the validated signature:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
    fa: FA,
) -> <<FA as DefaultBrand>::Brand as Kind>::Of<'a, B>
where
    FA: DefaultBrand,
```

Key differences: `FA` appears in `FunctorDispatch`, no `Into` bound, no
GAT equality bound (or optionally kept as safety net).

### Change 3: Exclude `pure`, `compose_kleisli`, `compose_kleisli_flipped`

These three functions cannot use brand inference:

- `pure`: No container argument; return-type inference fails (E0283).
- `compose_kleisli`/`compose_kleisli_flipped`: Input is a plain value,
  not a container.

These functions keep their current names and signatures. They are NOT
renamed to `_explicit`.

### Change 4: Document foldable/traversable partial inference

For `fold_right`, `fold_left`, `fold_map`, and `traverse`:

- Brand is inferred from the container (one fewer turbofish param).
- `FnBrand` remains explicit (no inference source).
- `traverse`'s applicative brand `F` also remains explicit.

These functions get partial turbofish reduction, not elimination.

### Change 5: Update macro design

Inferred-mode macros (`m_do!({ ... })`, `a_do!({ ... })`) have a `pure`
problem:

- `bind(fa, |x| pure(expr))` fails because `pure` cannot infer Brand.
- **Recommended approach:** In inferred mode, do not rewrite `pure(expr)`.
  Users write concrete constructors (`Some(expr)`, `vec![expr]`) or use
  the explicit-brand syntax.
- Alternatively, `pure(expr)` could still be rewritten using a "first bind
  brand" approach if the macro threads a type witness, but this adds
  complexity. Concrete constructors are the pragmatic choice.

Add parser support for four syntax combinations:

- `m_do!(Brand { ... })` / `m_do!(ref Brand { ... })` (explicit, unchanged)
- `m_do!({ ... })` / `m_do!(ref { ... })` (inferred, new)

### Change 6: Replace the POC

Remove the old `brand_inference_poc` module from `dispatch.rs`. Reference
the new test file `fp-library/tests/brand_inference_feasibility.rs` (18
passing tests) as the canonical POC.

## Open Questions

### Q1: Is the GAT equality bound needed?

The validated `map_infer` signature works WITHOUT the GAT equality bound
`<FA as DefaultBrand>::Brand: Kind<Of<'a, A> = FA>`. The dispatch trait
impl selection handles type unification. However, this bound may help
the compiler normalize the return type projection. Keeping it is safe;
removing it saves one where clause per function.

**Recommendation:** Test removal on a case-by-case basis during
implementation. If removing it causes inference failures, add it back.

### Q2: Should 0-bind `a_do!` be supported in inferred mode?

`a_do!({ 42 })` generates `pure::<Brand, _>(42)`, which requires a brand.
In inferred mode, this cannot work without a type annotation on the result.

**Recommendation:** Do not support 0-bind inferred-mode `a_do!`. It is
a rare edge case (just wrapping a value in `pure`) and users can write
`let x: Option<i32> = a_do!(OptionBrand { 42 })` or just `Some(42)`.

### Q3: DefaultBrand trait naming

The plan proposes `DefaultBrand_{hash}` traits matching the Kind hash.
This is a macro-generation detail. The user-facing trait name should be
`DefaultBrand!( type Of<'a, A: 'a>: 'a; )` (mirroring `Kind!(...)`).

No change needed from the plan.

## Summary

| Aspect               | Before (plan as written)          | After (post-refactor)                                 |
| -------------------- | --------------------------------- | ----------------------------------------------------- |
| Val dispatch         | Works (POC validated)             | Works (18 tests)                                      |
| Ref dispatch         | "Would require separate function" | Works (same function, blanket &T impl)                |
| `Into` bound         | Required                          | Eliminated                                            |
| GAT equality bound   | "Unverified assumption"           | Likely unnecessary (works without it)                 |
| `pure` inference     | "Unreliable" (hedging)            | Confirmed failure (E0283)                             |
| Dispatch signature   | Old (no FA param)                 | New (FA param, simpler)                               |
| Macro `pure` rewrite | Assumed possible                  | Confirmed impossible; needs workaround                |
| Foldable/traversable | Not addressed                     | Partial inference (FnBrand stays explicit)            |
| compose_kleisli      | Not addressed                     | Cannot use inference (no container)                   |
| Overall feasibility  | Viable with caveats               | More feasible, simpler, with one confirmed limitation |
