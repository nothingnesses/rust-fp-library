# Plan: Ref Hierarchy Borrow Refactor

Change all Ref, SendRef, and ParRef trait methods, free functions, and dispatch
from consuming containers (`fa: Self::Of<'a, A>`) to borrowing them
(`fa: &Self::Of<'a, A>`). This is a full borrow change across the entire
public API, not just the trait methods.

## Status

Steps 1 and 6 are complete. The library compiles cleanly (`cargo check --lib`
passes). Remaining: Step 2a (RefTraversable optimization), Step 3 (dispatch
two-impl), Step 4 (macros), Step 7 (tests/docs).

| Step                                  | Status      | Notes                                                                                                                   |
| ------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------- |
| Step 1.1-1.13 (Ref traits + impls)    | Done        | 46 files changed. Library compiles.                                                                                     |
| Step 2 (verify Ref hierarchy)         | Partial     | `cargo check --lib` passes. 59 errors remain in inline tests and doc examples.                                          |
| Step 2a (RefTraversable optimization) | Not started | Replace `ta.clone()` delegation with direct by-reference iteration. Validated by 11 tests in `ref_traverse_direct.rs`.  |
| Step 3 (dispatch two-impl pattern)    | Not started | Add `FA` type parameter. Two impls: Val+owned, Ref+borrowed. Validated by 28 tests in `dispatch_borrow_feasibility.rs`. |
| Step 4 (macros)                       | Not started | Generate `&(expr)` in ref mode. Depends on Step 3.                                                                      |
| Step 5 (verify dispatch + macros)     | Not started |                                                                                                                         |
| Step 6 (SendRef + ParRef)             | Done        | Done in parallel with Step 1.                                                                                           |
| Step 7 (tests + docs)                 | Not started | 59 remaining errors in inline tests and doc examples. Agent-per-file approach.                                          |

## Deviations from Plan

### Steps 1 and 6 were done together, not sequentially

The plan called for Steps 1-2 (Ref traits), then verify, then Step 6
(SendRef + ParRef). In practice, the SendRef and ParRef trait definitions and
implementations were changed in parallel with the Ref traits using 8 agents
(6 in the first wave, 2 more for SendRef/ParRef trait definitions). This was
more efficient because the agents didn't touch overlapping files.

### `ref_if_m` and `ref_unless_m` require eager cloning

The plan didn't anticipate a lifetime issue with these free functions. When
`then_branch` and `else_branch` change from owned to `&Of<A>`, capturing the
borrows in a `move` closure for `ref_bind` causes E0621 (lifetime too short).
The fix is to clone the branches eagerly before the closure:

```rust
let then_branch = then_branch.clone();
let else_branch = else_branch.clone();
Brand::ref_bind(cond, move |c: &bool| {
    if *c { then_branch.clone() } else { else_branch.clone() }
})
```

This adds one extra clone per call compared to the previous design (which
moved the branches into the closure without cloning). The `Of<A>: Clone`
bound was already required, so no new bounds are needed.

### Bifunctorial `Clone` bounds go on the `impl` block, not the method

The plan said to add `E: Clone` / `First: Clone` to the method's `where`
clause. Rust does not allow impl methods to have stricter bounds than the
trait definition (E0276). The fix is to add `Clone` to the impl block's
generic bounds:

```rust
// Before: impl<E: 'static> RefFunctor for ResultErrAppliedBrand<E>
// After:  impl<E: Clone + 'static> RefFunctor for ResultErrAppliedBrand<E>
```

This means `ResultErrAppliedBrand<E>` only implements `RefFunctor` when
`E: Clone`. This is a narrowing of the impl scope, but it's the correct
trade-off: producing `Err(e)` from `&Err(e)` inherently requires cloning.

### RefTraversable uses `ta.clone()` delegation (Approach B)

The plan recommended Approach A (rewrite to iterate by reference) for Vec,
CatList, Option, etc. In practice, all RefTraversable impls initially used
Approach B (`Self::traverse(move |a: A| func(&a), ta.clone())`). Step 2a
will replace these with direct by-reference iteration (Approach A), which
has been validated by 11 tests in `ref_traverse_direct.rs`.

### Dispatch adapter approach (temporary, not two-impl)

The dispatch Ref impls currently use the adapter approach: they take `fa` by
value and pass `&fa` to the trait method. Step 3 will replace this with the
two-impl `FA` type parameter design.

## Decisions

### D6: Dispatch uses the two-impl `FA` pattern (not adapter)

The adapter approach (dispatch takes by-value, borrows internally) was
considered but rejected in favor of the two-impl pattern. Rationale: the
adapter means `map(|x: &i32| ..., vec)` silently consumes the container,
which is inconsistent with the borrow semantics of the Ref hierarchy. The
two-impl pattern rejects this at compile time, requiring `map(|x: &i32| ...,
&vec)` and making the `&` on the closure and on the container consistent.

### D7: RefTraversable uses direct by-reference iteration (not `ta.clone()`)

Validated by 11 tests in `ref_traverse_direct.rs`. The fold-based pattern
`ta.iter().fold(F::pure(Vec::new()), |acc, a| F::lift2(..., acc, func(a)))`
works identically with borrowed iteration as with owned iteration. Explicit
type annotations are needed on the fold closure (the compiler cannot infer
the applicative type from context).

### D8: Test/doc fixes use agent-per-file approach

59 remaining compilation errors are all mechanical `&` additions at call
sites. Each type file's inline tests and doc examples will be handled by a
separate agent, preventing file conflicts.

## Next Steps

### Step 2a: RefTraversable optimization

Replace `ta.clone()` delegation with direct by-reference iteration in:
Vec, Option, CatList, Identity, Tuple1, Pair (2 brands), Tuple2 (2 brands),
Result (2 brands). Follow the validated pattern from `ref_traverse_direct.rs`.

The Vec pattern (which all collection types follow):

```rust
fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
    func: impl Fn(&A) -> F::Of<'a, B> + 'a,
    ta: &Vec<A>,
) -> F::Of<'a, Vec<B>>
where
    Vec<B>: Clone,
    F::Of<'a, B>: Clone,
{
    let len = ta.len();
    ta.iter().fold(
        F::pure::<Vec<B>>(Vec::with_capacity(len)),
        |acc: F::Of<'a, Vec<B>>, a| {
            F::lift2(|mut v: Vec<B>, b: B| { v.push(b); v }, acc, func(a))
        },
    )
}
```

### Step 3: Dispatch two-impl pattern

Add `FA` type parameter to each dispatch trait. Create two impls per trait:
Val (owned) and Ref (borrowed). Remove the current Ref+owned adapter impls.
Prototype on `FunctorDispatch` first, then replicate.

This is the most architecturally novel remaining change. See the "Phase 5"
section below for the full design.

### Step 4: Macros

Update `m_do!` and `a_do!` codegen to generate `&(expr)` in ref mode and
update turbofish underscore counts.

### Step 7: Tests and docs

Fix all 59 remaining compilation errors in inline tests and doc examples by
adding `&` to container arguments. Add compile-fail tests for
`map(|x: &i32| ..., vec)` rejection. Update documentation prose.

## Motivation

The Ref hierarchy's closures receive `&A` (element references), so the
container's elements are never moved. The current consume-by-value signatures
are more restrictive than necessary: users must clone a `Vec` to use it after
`ref_map`, even though the operation only iterates by reference. Borrowing
the container aligns the trait signature with what the operation actually needs.

## Research Summary

Five research documents inform this plan:

- [ref-trait-signatures.md](ref-trait-signatures.md) - All Ref trait method
  signatures and borrowability analysis.
- [implementations.md](implementations.md) - Per-type analysis of every
  concrete Ref trait implementation.
- [dispatch-and-macros.md](dispatch-and-macros.md) - Dispatch adapter pattern,
  macro codegen, free functions.
- [sendref-parref-impact.md](sendref-parref-impact.md) - Impact on SendRef and
  ParRef hierarchies.
- [lifetime-analysis.md](lifetime-analysis.md) - GAT interactions, closure
  capture, Apply! macro, default impl chains.

### Key findings

1. **No lifetime complications.** The borrow lifetime `'b` is always elided
   (never appears in return types). GAT bounds, closure capture, and the
   `Apply!` macro all work correctly with `&Self::Of<'a, A>`.

2. **No implementation truly requires ownership.** Every concrete Ref trait
   implementation either iterates by reference (Vec, CatList, Option, Identity,
   Tuple1) or can cheaply clone an Rc/Arc handle (Lazy, TryLazy). No in-place
   mutation occurs.

3. **Default implementations have consistent call chains.** Every default
   passes `fa` to another method on the same or a supertrait, so changing all
   traits together maintains consistency.

4. **SendRef and ParRef should change in tandem** for API consistency. All
   SendRef and ParRef implementations already borrow internally (via
   `evaluate()`, `iter()`, `par_iter()`).

5. **Feasibility validated with 19 passing tests.** See the Feasibility
   Validation section below.

## Feasibility Validation

Two test suites validate all critical patterns:

- `fp-library/tests/ref_borrow_feasibility.rs` (19 tests) - trait methods,
  free functions, Lazy clone-and-capture, temporary lifetimes, nested borrows.
- `fp-library/tests/dispatch_borrow_feasibility.rs` (28 tests) - two-impl
  dispatch pattern, GAT projections, inference, mixed modes, compile-fail
  verification.

All 47 tests pass.

### Patterns tested

| Test                             | Pattern                                                 | Result |
| -------------------------------- | ------------------------------------------------------- | ------ |
| `lazy_ref_map_from_borrow`       | `ref_map(&lazy, f)`: clone Rc, capture in closure       | Pass   |
| `lazy_ref_lift2_from_borrows`    | `ref_lift2(&a, &b, f)`: clone both Rc handles           | Pass   |
| `lazy_ref_bind_from_borrow`      | `ref_bind(&lazy, f)`: evaluate via `&self`, user derefs | Pass   |
| `lazy_ref_fold_map_from_borrow`  | `ref_fold_map(&lazy, f)`: evaluate via `&self`          | Pass   |
| `lazy_ref_bind_complex_closure`  | `ref_bind(&lazy_vec, f)`: closure processes `&Vec`      | Pass   |
| `lazy_nested_bind_temporary`     | `ref_bind(&ref_bind(&a, f1), f2)`: nested temporaries   | Pass   |
| `vec_ref_map_from_borrow`        | `ref_map(&vec, f)`: iterate via `.iter()`               | Pass   |
| `vec_ref_bind_from_borrow`       | `ref_bind(&vec, f)`: flat_map via `.iter()`             | Pass   |
| `vec_ref_lift2_from_borrows`     | `ref_lift2(&a, &b, f)`: nested `.iter()`                | Pass   |
| `option_ref_map_from_borrow`     | `ref_map(&opt, f)`: `.as_ref().map()`                   | Pass   |
| `option_ref_bind_from_borrow`    | `ref_bind(&opt, f)`: `.as_ref().and_then()`             | Pass   |
| `dispatch_adapter_pattern`       | Dispatch takes by-value, borrows internally             | Pass   |
| `fully_borrowed_dispatch`        | Free function takes `&fa` directly                      | Pass   |
| `temporary_lifetime_in_argument` | `ref_map(f, &make_vec())`: temporary borrow             | Pass   |
| `nested_borrowed_bind`           | `ref_bind(&ref_bind(&v, f1), f2)`: Vec nested temps     | Pass   |
| `simulated_m_do_fully_borrowed`  | `ref_bind(&Some(5), \|x\| ref_bind(&Some(10), ...))`    | Pass   |
| `simulated_a_do_fully_borrowed`  | `ref_lift2(f, &Some(5), &Some(10))`                     | Pass   |
| `result_ref_map_borrowed`        | `ref_map(&result, f)` with `E: Clone` for passthrough   | Pass   |
| `pair_ref_map_borrowed`          | `ref_map(&pair, f)` with `First: Clone` for fixed field | Pass   |

### Dispatch feasibility tests (`dispatch_borrow_feasibility.rs`)

| Test                                   | Pattern                                                          | Result |
| -------------------------------------- | ---------------------------------------------------------------- | ------ |
| `val_owned`                            | `map(\|x: i32\| ..., vec)`: Val dispatch, consumes               | Pass   |
| `ref_borrowed`                         | `map(\|x: &i32\| ..., &vec)`: Ref dispatch, borrows              | Pass   |
| `ref_owned`                            | `map(\|x: &i32\| ..., vec)`: three-impl pattern (concrete types) | Pass   |
| `ref_borrowed_reuse`                   | Multiple `map(\|x: &i32\| ..., &v)` calls on same `v`            | Pass   |
| `option_val_owned`                     | Option Val dispatch                                              | Pass   |
| `option_ref_borrowed`                  | Option Ref dispatch with borrow                                  | Pass   |
| `option_ref_owned`                     | Option Ref dispatch with owned (three-impl, concrete)            | Pass   |
| `bind_val_owned`                       | `bind(vec, \|x: i32\| ...)`: Val bind                            | Pass   |
| `bind_ref_borrowed`                    | `bind(&vec, \|x: &i32\| ...)`: Ref bind, borrows                 | Pass   |
| `bind_ref_owned`                       | `bind(vec, \|x: &i32\| ...)`: Ref bind, owned (concrete)         | Pass   |
| `lift2_val_owned`                      | `lift2(\|x: i32, y: i32\| ..., a, b)`                            | Pass   |
| `lift2_ref_borrowed`                   | `lift2(\|x: &i32, y: &i32\| ..., &a, &b)`                        | Pass   |
| `dispatch_temporary_borrow`            | `map(\|x: &i32\| ..., &make_vec())`: temp borrow                 | Pass   |
| `dispatch_nested_bind_borrowed`        | `bind(&bind(&v, f1), f2)`: nested temporaries                    | Pass   |
| `dispatch_mixed_modes`                 | Borrow then consume in same scope                                | Pass   |
| `dispatch_inference_typed_val`         | Type inference with annotated Val closure                        | Pass   |
| `dispatch_inference_ref_borrow`        | Type inference with annotated Ref closure                        | Pass   |
| `dispatch_ref_closure_owned_container` | Three-impl pattern (concrete)                                    | Pass   |
| `dispatch_no_ambiguity_ref_owned`      | No impl ambiguity, concrete types                                | Pass   |
| `dispatch_no_ambiguity_ref_borrowed`   | No impl ambiguity, borrowed                                      | Pass   |
| `two_impl_val_owned`                   | Two-impl: Val+owned                                              | Pass   |
| `two_impl_ref_borrowed`                | Two-impl: Ref+borrowed                                           | Pass   |
| `two_impl_ref_borrowed_reuse`          | Two-impl: multiple borrows                                       | Pass   |
| `two_impl_ref_temporary`               | Two-impl: `&vec![1,2,3]` temporary                               | Pass   |
| `two_impl_nested_bind`                 | Two-impl: nested bind with temporaries                           | Pass   |
| `two_impl_mixed_modes`                 | Two-impl: borrow then consume                                    | Pass   |
| `gat_projection_val`                   | GAT projection `<Brand as Kind>::Of<A>`: Val                     | Pass   |
| `gat_projection_ref_borrowed`          | GAT projection: Ref+borrowed                                     | Pass   |

Compile-fail verification: `map(|x: &i32| ..., vec![1, 2, 3])` (Ref closure
with owned container, no matching two-impl) correctly fails with E0631
("type mismatch in closure arguments").

E0119 verification: three-impl pattern with GAT projections causes conflicting
impl errors. Two-impl pattern does not. This confirms that the two-impl design
is the correct approach for the real dispatch system.

### Findings from initial test failures

The first test run revealed three patterns that needed adjustment:

1. **Lazy `ref_bind` closure capture (E0521).** Writing
   `ref_bind(&lazy, |x| Lazy::new(move || x + 1))` fails because `x: &i32`
   cannot be captured into a `'static` closure. The fix is to dereference:
   `ref_bind(&lazy, |x| { let x = *x; Lazy::new(move || x + 1) })`. This is
   **not a regression**: the current Ref hierarchy already delivers `&A` to
   bind closures, so users already handle this. The borrow change does not
   affect the closure's element access pattern.

2. **Vec `ref_lift2` closure move (E0507).** The closure `f` in
   `fa.iter().flat_map(|a| fb.iter().map(move |b| f(a, b)))` is moved on each
   outer iteration. Adding `+ Copy` (or `+ Clone` with explicit cloning)
   resolves this. The actual library uses `impl Fn` which is `Copy` when the
   closure captures no owned data, so this is not an issue in practice.

3. **`once_cell` crate not available (E0433).** Incidental; switched to
   `std::cell::OnceCell`.

None of these are fundamental blockers. Pattern 1 is unchanged from the
current design (closures already receive `&A`). Pattern 2 is an artifact of
the simplified test, not present in the real implementations.

## Decisions

### D1: Full borrowing across the entire API

All layers change to borrow:

- **Trait methods:** `fa: &Self::Of<'a, A>` (the honest contract).
- **Non-dispatch free functions:** `fa: &Apply!(...)` (matches the trait).
- **Dispatch traits:** Add an `FA` type parameter. Val impl uses
  `FA = Apply!(Of<A>)` (owned). Ref impl uses `FA = &Apply!(Of<A>)`
  (borrowed). Two impls per dispatch trait, not three. The third impl
  (Ref+owned) is dropped to avoid E0119 with GAT projections.
- **Dispatch free functions:** Take `fa: FA` where `FA` is inferred from the
  argument. `map(|x: i32| ..., vec)` infers owned; `map(|x: &i32| ..., &vec)`
  infers borrowed. `map(|x: &i32| ..., vec)` does not compile (no matching
  impl), which is the intended behavior: the `&` on the closure and on the
  container must be consistent.
- **Macros:** Generate `&(expr)` for container arguments in ref mode. Temporary
  lifetime is safe because Rust extends temporaries in function arguments to
  the enclosing statement. Validated by feasibility tests.

This is a breaking change across the board. Every call site passing an owned
container to a Ref/SendRef/ParRef trait method, free function, or dispatch
function will need `&`. Dispatch turbofish gains one `_` for the `FA` parameter.

### D2: Result and Pair/Tuple2 passthrough fields

Add `Clone` bounds on passthrough fields where missing:

- `ResultErrApplied::ref_map`: add `E: Clone`.
- `ResultOkApplied::ref_map`: add `T: Clone`.
- `PairFirstApplied::ref_map`: add `First: Clone`.
- `PairSecondApplied::ref_map`: add `Second: Clone`.
- Same for Tuple2 applied brands.

These bounds already exist on RefLift, RefSemiapplicative, RefSemimonad for
these types. Only RefFunctor (and RefFoldable for Result) is affected.

### D3: RefTraversable delegation

Rewrite RefTraversable implementations to iterate by reference instead of
delegating to owned `Self::traverse`. For types where the rewrite is trivial
(Vec, CatList, Option, Identity, Tuple1), write a direct implementation. For
Pair/Tuple2/Result, clone the fixed field and construct directly.

### D4: Lazy inherent methods

Change Lazy/TryLazy inherent methods (`ref_map`, etc.) from `self` to `&self`,
with an internal `self.clone()` (Rc/Arc bump) before closure capture. Users
calling `lazy.ref_map(f)` will no longer consume the Lazy.

### D5: Filter operations

No in-place mutation optimization is lost. Current implementations all
allocate new containers via `.collect()`.

## Implementation Plan

### Execution strategy

The critical path is: Step 1 (Ref traits + impls) -> Step 2 (verify) ->
Step 3 (dispatch) -> Step 4 (macros) -> Step 5 (verify) -> Step 6
(SendRef + ParRef) -> Step 7 (tests + docs).

**Do not** change all trait definitions first and then all implementations.
Instead, change each trait and its implementations together, verify it
compiles with `just check --workspace`, then move to the next. This catches
issues early and keeps the blast radius small.

### Step 1: Ref traits + implementations (together, one trait at a time)

Change each trait definition, its default implementations, its free functions,
and all concrete implementations in one pass, ordered by dependency:

**1.1 RefFunctor** (foundation, no dependencies)

Trait: `fp-library/src/classes/ref_functor.rs`
Impls: Vec, Option, CatList, Identity, Tuple1, Lazy, TryLazy, Result
(ErrApplied + OkApplied), Pair (FirstApplied + SecondApplied), Tuple2
(FirstApplied + SecondApplied)

- Change `fa: Apply!(Of<A>)` to `fa: &Apply!(Of<A>)` in trait method and
  free function.
- Vec/CatList/Option/Identity/Tuple1: signature change only, bodies use
  `.iter()` / `.as_ref()` / `&fa.0`.
- Lazy/TryLazy: change inherent `ref_map` to `&self`, add `self.clone()`
  before closure capture.
- Result: add `E: Clone` / `T: Clone` bound, add `.clone()` on passthrough
  `Err(e)` / `Ok(t)`.
- Pair/Tuple2: add `.clone()` on fixed field.

Verify: `just check --workspace`

**1.2 RefFoldable** (independent of RefFunctor at trait level)

Trait: `fp-library/src/classes/ref_foldable.rs`
Impls: Vec, Option, CatList, Identity, Tuple1, Lazy, TryLazy, Result, Pair,
Tuple2

- Change `fa` param in `ref_fold_map`, `ref_fold_right`, `ref_fold_left`.
- Change default implementations (pass `&fa` through the chain).
- All impls: signature change only. Vec uses `.iter()`, Lazy uses
  `evaluate(&self)`.

Verify: `just check --workspace`

**1.3 RefFunctorWithIndex** (depends on RefFunctor)

Trait: `fp-library/src/classes/ref_functor_with_index.rs`
Impls: Vec, CatList, Lazy, Option, Identity

Verify: `just check --workspace`

**1.4 RefFoldableWithIndex** (depends on RefFoldable)

Trait: `fp-library/src/classes/ref_foldable_with_index.rs`
Impls: Vec, CatList, Lazy, Option, Identity

Verify: `just check --workspace`

**1.5 RefLift** (depends on RefFunctor via RefSemiapplicative)

Trait: `fp-library/src/classes/ref_lift.rs`
Impls: Vec, Option, CatList, Identity, Tuple1, Lazy, Result, Pair, Tuple2

- Both `fa` and `fb` change to `&Apply!(...)`.
- Lazy: clone both `fa` and `fb` before closure capture.

Verify: `just check --workspace`

**1.6 RefSemiapplicative + RefApplyFirst + RefApplySecond**

Traits: `ref_semiapplicative.rs`, `ref_apply_first.rs`, `ref_apply_second.rs`
Impls: Vec, Option, CatList, Identity, Tuple1, Lazy, Result, Pair, Tuple2

- `ref_apply`: both `ff` and `fa` change to `&Apply!(...)`.
- `ref_apply_first` / `ref_apply_second`: blanket impls, default calls
  `Self::ref_lift2(...)` with `&fa` and `&fb`. Should work automatically
  after 1.5.

Verify: `just check --workspace`

**1.7 RefPointed** (no container param)

Trait: `fp-library/src/classes/ref_pointed.rs`
Check free function `ref_pure`; it already takes `&A`, no container change.

**1.8 RefApplicative** (marker trait)

Trait: `fp-library/src/classes/ref_applicative.rs`
Blanket impl combining RefPointed + RefSemiapplicative + RefApplyFirst +
RefApplySecond. Should work automatically after 1.6 and 1.7.

**1.9 RefSemimonad** (independent)

Trait: `fp-library/src/classes/ref_semimonad.rs`
Impls: Vec, Option, CatList, Identity, Tuple1, Lazy, Result, Pair, Tuple2

- Change `fa` (or `ma`) to `&Apply!(...)`.
- Lazy: `ref_bind` calls `f(fa.evaluate())`; `evaluate` is `&self`, so
  `&fa` works directly without cloning.

Verify: `just check --workspace`

**1.10 RefMonad + free functions**

Trait: `fp-library/src/classes/ref_monad.rs`
Blanket impl. Free functions: `ref_join`, `ref_if_m`, `ref_unless_m`.

- `ref_join(&mma)`: calls `Brand::ref_bind(&mma, |ma| ma.clone())`. Works
  because `Of<A>: Clone` is already required.
- `ref_if_m(&cond, &then_branch, &else_branch)`: captures `&then_branch`
  and `&else_branch` in bind closure, clones inside. Works because
  `Of<A>: Clone` is already required.
- `ref_unless_m`: same pattern.

Verify: `just check --workspace`

**1.11 RefFilterable + RefFilterableWithIndex**

Traits: `ref_filterable.rs`, `ref_filterable_with_index.rs`
Impls: Vec, Option, CatList

- Default `ref_filter_map` calls `Self::compact(Self::ref_map(func, &fa))`.
  After 1.1, `ref_map` takes `&fa`. `compact` takes the result (owned).
  Works.
- Default `ref_partition_map` calls
  `Self::separate(Self::ref_map(func, &fa))`. Same pattern.

Verify: `just check --workspace`

**1.12 RefTraversable + RefTraversableWithIndex** (most work)

Traits: `ref_traversable.rs`, `ref_traversable_with_index.rs`
Impls: Vec, Option, CatList, Identity, Tuple1, Result, Pair, Tuple2

Current impls delegate to owned `Self::traverse(move |a: A| func(&a), ta)`.
With `ta: &Self::Of<'a, A>`, this delegation breaks. Each type needs a
standalone by-reference traversal.

**Order within 1.12:** Do Vec first (most complex, establishes the pattern),
then Option (simplest), then replicate for the rest.

- **Vec:** Iterate `ta.iter()`, fold with applicative `lift2` to build up
  the result. This is the standard fold-based traverse pattern but using
  `&A` references.
- **Option:** Match on `&Option<A>`: `Some(a)` gives `&A`, call `func(a)`,
  wrap in `F::map(|b| Some(b), ...)`. `None` gives `F::pure(None)`.
- **CatList:** Same as Vec (iterate, fold with applicative).
- **Identity:** `F::map(|b| Identity(b), func(&ta.0))`.
- **Tuple1:** `F::map(|b| (b,), func(&ta.0))`.
- **Pair/Tuple2:** Clone fixed field, traverse the functored field, wrap.
- **Result:** Clone passthrough variant, traverse the functored variant.

Verify: `just check --workspace`

**1.13 RefWitherable**

Trait: `fp-library/src/classes/ref_witherable.rs`
Impls: Vec, Option, CatList

- Default `ref_wither` calls `Self::ref_traverse(func, &ta)` then
  `M::map(|opt| Self::compact(opt), ...)`. After 1.12, `ref_traverse`
  takes `&ta`. Works.
- Default `ref_wilt` calls `Self::ref_traverse(func, &ta)` then
  `M::map(|res| Self::separate(res), ...)`. Same.

Verify: `just check --workspace`

### Step 2: Full verification of Ref hierarchy

Run `just verify` (fmt, clippy, doc, test) to catch any remaining issues
including doc test breakage (many `&` additions needed in examples).

### Step 3: Dispatch (two-impl pattern with `FA` type parameter)

Do this before SendRef/ParRef because it is the most architecturally novel
change. If it hits unexpected issues, better to know before replicating.

**Start with FunctorDispatch as prototype:** Add `FA`, write both impls,
update free function, update tests. Once working, replicate for the other
5 dispatch modules.

Add an `FA` type parameter to each dispatch trait, enabling the free function
to accept both owned containers (for Val) and borrowed containers (for Ref).

**Design:** Each dispatch trait gets two impls instead of the current
three (Val+owned, Ref+owned):

- **Val impl:** `FA = Apply!(Of<A>)` (owned). Closure is `Fn(A) -> B`.
- **Ref impl:** `FA = &Apply!(Of<A>)` (borrowed). Closure is `Fn(&A) -> B`.

The third impl (Ref+owned, which borrows internally) is dropped. This means
`map(|x: &i32| ..., vec)` no longer compiles; the user must write
`map(|x: &i32| ..., &vec)`. This is a breaking change, but it makes the
API consistent: the `&` in the closure argument and the `&` on the container
are aligned.

**Validated by feasibility tests:** `dispatch_borrow_feasibility.rs` (28 tests
passing) confirms:

- Val+owned compiles and works.
- Ref+borrowed compiles and works.
- Ref+owned correctly fails to compile.
- GAT projection types (`<Brand as Kind>::Of<A>`) work with two impls (no
  E0119). Three impls cause E0119 because the compiler cannot prove a
  projection type is never a reference.
- Temporary borrows (`&make_vec()`), nested borrows
  (`bind(&bind(&v, f1), f2)`), and mixed modes all work.
- Type inference works when closure params are annotated.

**Turbofish change:** The `FA` type parameter adds one more `_` to the
turbofish. `map::<Brand, _, _, _>(...)` becomes `map::<Brand, _, _, _, _>(...)`.
`FA` is inferred from the argument and never needs explicit specification.

Files:

- `fp-library/src/classes/dispatch/functor.rs`
- `fp-library/src/classes/dispatch/semimonad.rs`
- `fp-library/src/classes/dispatch/foldable.rs`
- `fp-library/src/classes/dispatch/lift.rs`
- `fp-library/src/classes/dispatch/filterable.rs`
- `fp-library/src/classes/dispatch/traversable.rs`
- `fp-library/src/classes/dispatch.rs` (re-exports, tests)

Changes per file:

- Add `FA` type parameter to dispatch trait definition. For multi-container
  operations (lift2-5), add `FA` and `FB` (and `FC`, `FD`, `FE`) as separate
  type parameters, one per container. `Marker` is still a single type (Val or
  Ref), so all containers must be the same mode: either all owned or all
  borrowed. Mixed mode (e.g., `lift2(f, owned_vec, &borrowed_vec)`) is not
  supported, which is consistent with the closure determining the mode.
- Change Val impl: `FA = Apply!(Of<A>)`, body unchanged.
- Change Ref impl: `FA = &'b Apply!(Of<A>)`, body calls `Brand::ref_map(self, fa)`.
- Remove the current Ref+owned impl (which takes `fa` by value).
- Update dispatch free function to take `fa: FA` (and `fb: FB`, etc.) with
  `impl DispatchTrait<..., FA, ..., Marker>` bound.
- Update all turbofish call sites (one more `_` per container parameter).
- `compose_kleisli` and `compose_kleisli_flipped`: the input `a: A` is always
  owned (it is the raw value, not a container). The intermediate container
  produced by the first Kleisli arrow is always a fresh owned value. Only the
  final `a` parameter is relevant, and it stays owned in both modes. These
  dispatch traits may not need `FA` at all; the mode is determined solely by
  the closure types.

**Resolved:** `send_thunk.rs` does not implement any SendRef traits, so it
requires no changes.

Verify: `just check --workspace` after FunctorDispatch, then after all six.

### Step 4: Macros

Update `m_do!` and `a_do!` codegen for ref mode to generate `&expr` for
container arguments, matching the new Ref dispatch requirement.

Files:

- `fp-macros/src/m_do/codegen.rs`
- `fp-macros/src/a_do/codegen.rs`

Changes:

- `m_do!(ref ...)`: Change generated `bind::<Brand, _, _, _>(expr, ...)`
  to `bind::<Brand, _, _, _, _>(&(expr), ...)`. The `&(expr)` wraps the
  expression in a borrow. Parentheses ensure correct precedence.
- `a_do!(ref ...)`: Change generated `liftN::<Brand, ...>(f, expr1, ...)`
  to `liftN::<Brand, ...>(f, &(expr1), ...)`. Each container expression
  gets `&(...)`.
- `pure(x)` rewriting: already generates `ref_pure::<Brand, _>(&(x))`,
  no change needed.
- Update turbofish underscore counts to match new dispatch signatures.

**Temporary lifetime safety:** Validated by feasibility tests. Rust extends
temporary lifetimes in function argument position to the enclosing statement,
so `bind(&(some_function()), ...)` keeps the temporary alive for the duration
of the `bind` call. Nested chains like `bind(&(bind(&v, f1)), f2)` also work
because each temporary lives until the end of its enclosing statement.

Verify: `just check --workspace`, then run macro tests.

### Step 5: Verify dispatch + macros

Run `just verify` to catch any issues from Steps 3-4 before replicating
the pattern across SendRef/ParRef.

### Step 6: SendRef + ParRef (can be parallelized)

Mechanical replication of the Step 1 pattern. SendRef and ParRef are
independent and can be done in parallel.

**Step 6a: SendRef traits and implementations**

Apply the same borrow changes to all SendRef traits and their
ArcLazy/ArcTryLazy implementations. Follow the same trait-by-trait order
as Step 1 (functor, foldable, functorWithIndex, etc.).

Files:

- `fp-library/src/classes/send_ref_functor.rs`
- `fp-library/src/classes/send_ref_foldable.rs`
- `fp-library/src/classes/send_ref_foldable_with_index.rs`
- `fp-library/src/classes/send_ref_functor_with_index.rs`
- `fp-library/src/classes/send_ref_lift.rs`
- `fp-library/src/classes/send_ref_semiapplicative.rs`
- `fp-library/src/classes/send_ref_semimonad.rs`
- `fp-library/src/classes/send_ref_applicative.rs`
- `fp-library/src/classes/send_ref_monad.rs`
- `fp-library/src/classes/send_ref_apply_first.rs`
- `fp-library/src/classes/send_ref_apply_second.rs`
- `fp-library/src/classes/send_ref_pointed.rs`
- `fp-library/src/types/lazy.rs` (ArcLazy SendRef impls)

Smaller than Step 1: fewer traits (no SendRefFilterable, SendRefTraversable,
SendRefWitherable) and only one implementor (ArcLazy).

**Step 6b: ParRef traits and implementations**

Apply the same borrow changes to all ParRef traits and their Vec/CatList
implementations.

Files:

- `fp-library/src/classes/par_ref_functor.rs`
- `fp-library/src/classes/par_ref_foldable.rs`
- `fp-library/src/classes/par_ref_foldable_with_index.rs`
- `fp-library/src/classes/par_ref_functor_with_index.rs`
- `fp-library/src/classes/par_ref_filterable.rs`
- `fp-library/src/classes/par_ref_filterable_with_index.rs`
- `fp-library/src/types/vec.rs` (ParRef impls)
- `fp-library/src/types/cat_list.rs` (ParRef impls)

Smaller than Step 1: no monadic chain (no ParRefPointed, ParRefSemimonad,
etc.) and implementations already use `.par_iter()` / `.iter()` which borrow.

Verify: `just verify` after both 6a and 6b complete.

### Step 7: Tests and documentation

Most doc test breakage will have been caught during Steps 1-5 (doc examples
are compiled by `just check`). This step covers remaining test files,
documentation prose, and new compile-fail tests.

- Update all test files that call Ref trait methods, `ref_*` free functions,
  or dispatch functions with Ref closures (adding `&` to container arguments).
- Update doc examples throughout.
- Update documentation explaining the borrow semantics.
- Rerun UI tests / compile-fail tests.
- Add compile-fail tests verifying that `map(|x: &i32| ..., vec)` (Ref closure
  with owned container) does not compile.
- Update `m_do!` and `a_do!` ref-mode tests.

Files:

- `fp-library/tests/*.rs`
- Doc comments in all modified files.
- `fp-library/docs/pointer-abstraction.md`
- `fp-library/docs/features.md`
- `fp-library/docs/parallelism.md`
- `CLAUDE.md` (if architecture description needs updating)

Verify: `just verify` (final full verification).

## Risk Assessment

| Risk                                                         | Likelihood | Mitigation                                                                                                                                                                         |
| ------------------------------------------------------------ | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Lifetime errors in complex generic code                      | Low        | 47 feasibility tests show no issues. Borrow lifetime is always elided.                                                                                                             |
| Performance regression from Rc/Arc clones in Lazy            | Negligible | Rc clone is a single non-atomic increment. Arc clone is a single atomic increment. Both are O(1). Validated by tests.                                                              |
| Breaking change: all Ref call sites need `&`                 | Certain    | Intentional. Applies to trait methods, free functions, dispatch, and macros.                                                                                                       |
| Breaking change: `map(\|x: &i32\| ..., vec)` stops compiling | Certain    | Intentional. Users write `map(\|x: &i32\| ..., &vec)` instead. The `&` is consistent between closure and container.                                                                |
| Breaking change: dispatch turbofish gains one `_`            | Certain    | `FA` is always inferred; users add one `_`.                                                                                                                                        |
| RefTraversable rewrite introduces bugs                       | Medium     | Each type needs a standalone ref-based traverse instead of delegating. Test thoroughly with existing traverse property tests.                                                      |
| Missing Clone bounds on Result/Pair                          | Low        | Required bounds already exist on most trait impls. Only RefFunctor for bifunctorial types is affected.                                                                             |
| Macro temporary lifetime issues                              | Low        | Validated by `simulated_m_do_fully_borrowed`, `nested_borrowed_bind`, and `dispatch_temporary_borrow` tests. Rust temporary lifetime rules handle `&(expr)` in function arguments. |
| E0119 with three dispatch impls                              | N/A        | Avoided by using two-impl pattern. Three impls (owned+Val, borrowed+Ref, owned+Ref) cause E0119 with GAT projections. Validated by `dispatch_borrow_feasibility.rs`.               |

## Estimated Scope

| Step                        | Files                                 | Difficulty                                                                 |
| --------------------------- | ------------------------------------- | -------------------------------------------------------------------------- |
| Step 1 (Ref traits + impls) | ~17 trait files + ~10 type files      | Mostly mechanical. RefTraversable (1.12) is the hardest.                   |
| Step 2 (verify)             | 0                                     | Just run `just verify`.                                                    |
| Step 3 (dispatch)           | ~7 dispatch files                     | Architecturally novel (two-impl + FA). Prototype on FunctorDispatch first. |
| Step 4 (macros)             | ~2 macro codegen files                | Small, focused change.                                                     |
| Step 5 (verify)             | 0                                     | Just run `just verify`.                                                    |
| Step 6a (SendRef)           | ~12 trait files + lazy.rs             | Mechanical replication of Step 1 pattern.                                  |
| Step 6b (ParRef)            | ~6 trait files + vec.rs + cat_list.rs | Mechanical replication. Impls already borrow internally.                   |
| Step 7 (tests + docs)       | ~6 test files + ~5 doc files          | Mostly adding `&` to call sites and examples.                              |

Total: ~65+ files. The critical items are Step 1.12 (RefTraversable rewrites)
and Step 3 (dispatch two-impl pattern). Everything else is mechanical.
