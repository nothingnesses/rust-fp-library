# POC Validation: Brand Inference with Ref Dispatch

## Hypothesis

After the ref-borrow refactor (which added an `FA` type parameter to dispatch
traits for the two-impl Val/Ref pattern), brand inference can support both
Val and Ref dispatch by:

1. Using `FA` directly as the `FunctorDispatch` container parameter (instead
   of projecting through `Brand::Of<'a, A>`).
2. Adding a blanket `impl<T: DefaultBrand + ?Sized> DefaultBrand for &T`.
3. Eliminating the `Into` bound that the original POC required.

The revised `map_infer` signature:

```rust
fn map_infer<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
    fa: FA,
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
    FA: DefaultBrand,
{
    f.dispatch(fa)
}
```

## Result: Hypothesis Confirmed (map, bind); pure Inference Fails

All 18 tests pass for `map_infer` and `bind_infer`. `pure_infer` (return-type
inference) fails with E0283 in all tested contexts.

## Test Results

| Test                             | Function   | Dispatch | Status       |
| -------------------------------- | ---------- | -------- | ------------ |
| val_option_infer                 | map_infer  | Val      | Pass         |
| val_vec_infer                    | map_infer  | Val      | Pass         |
| val_none_infer                   | map_infer  | Val      | Pass         |
| val_different_output_type        | map_infer  | Val      | Pass         |
| val_option_different_output_type | map_infer  | Val      | Pass         |
| ref_option_infer                 | map_infer  | Ref      | Pass         |
| ref_vec_infer                    | map_infer  | Ref      | Pass         |
| ref_lazy_infer                   | map_infer  | Ref      | Pass         |
| ref_option_reuse_after_map       | map_infer  | Ref      | Pass         |
| ref_vec_reuse_after_map          | map_infer  | Ref      | Pass         |
| ref_temporary_borrow             | map_infer  | Ref      | Pass         |
| ref_temporary_option             | map_infer  | Ref      | Pass         |
| mixed_val_then_ref               | map_infer  | Both     | Pass         |
| bind_val_option_infer            | bind_infer | Val      | Pass         |
| bind_val_vec_infer               | bind_infer | Val      | Pass         |
| bind_ref_option_infer            | bind_infer | Ref      | Pass         |
| bind_ref_vec_infer               | bind_infer | Ref      | Pass         |
| bind_ref_lazy_infer              | bind_infer | Ref      | Pass         |
| pure_inferred_from_bind_return   | pure_infer | N/A      | FAIL (E0283) |
| pure_inferred_from_nested_bind   | pure_infer | N/A      | FAIL (E0283) |
| pure_inferred_vec                | pure_infer | N/A      | FAIL (E0283) |

## Key Findings

### 1. The `Into` bound is unnecessary

The original POC in `dispatch.rs` used
`FA: Into<<Brand as Kind>::Of<'a, A>>` to convert the concrete type into
the brand's projected type before passing it to `FunctorDispatch::dispatch`.
This was needed because the old `FunctorDispatch` trait hardcoded the `FA`
parameter as `Brand::Of<'a, A>` for Val and `&Brand::Of<'a, A>` for Ref.

After the ref-borrow refactor, `FunctorDispatch` has a free `FA` type
parameter. The Val impl sets `FA = Brand::Of<'a, A>` (which equals the
concrete type, e.g., `Option<A>`), and the Ref impl sets
`FA = &'b Brand::Of<'a, A>`. Since `map_infer` passes `FA` directly as
both the function's parameter type and the dispatch's `FA` parameter,
the compiler unifies `FA` with the appropriate dispatch impl without any
conversion.

### 2. The blanket `&T` impl composes correctly with Ref dispatch

When `FA = &Option<i32>`:

- `DefaultBrand for &T` resolves `Brand = OptionBrand`
- `FunctorDispatch` matches the Ref impl where `FA = &'b Brand::Of<'a, A>`
- The compiler unifies `&Option<i32>` with `&'b Option<i32>`, confirming
  that the Ref dispatch impl applies

This means the same `map_infer` function handles both modes without
separate functions or special-casing.

### 3. Temporary borrows work

Expressions like `map_infer(|x: &i32| *x + 1, &vec![1, 2, 3])` work
correctly. The temporary `Vec` lives long enough for the borrow because
Rust extends temporary lifetimes for references in function arguments.

### 4. Container reuse after ref map works

Multiple `map_infer` calls on the same `&container` work as expected:

```rust
let v = vec![1, 2, 3];
let r1 = map_infer(|x: &i32| *x * 10, &v);
let r2 = map_infer(|x: &i32| *x + 100, &v);
```

This validates the primary ergonomic benefit of Ref dispatch: the
container is not consumed.

### 5. Mixed Val/Ref in the same scope works

A single scope can use Ref dispatch (borrowing) followed by Val dispatch
(consuming) on the same container, respecting standard Rust borrow rules.

## Changes from the Original POC

The original POC in `fp-library/src/classes/dispatch.rs` (`brand_inference_poc`
module):

- Only supported Val dispatch
- Required `FA: Into<Brand::Of<'a, A>>` bound
- Had a comment noting Ref dispatch was deferred

The new test file at `fp-library/tests/brand_inference_feasibility.rs`:

- Supports both Val and Ref dispatch
- No `Into` bound needed
- Blanket `DefaultBrand for &T` enables Ref dispatch brand inference
- 13 tests covering Val, Ref, temporary borrows, reuse, and mixed modes

### 6. `bind_infer` works identically to `map_infer`

The `bind_infer` function uses the same pattern:

```rust
fn bind_infer<'a, FA, A: 'a, B: 'a, Marker>(
    fa: FA,
    f: impl BindDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
    FA: DefaultBrand,
{
    f.dispatch_bind(fa)
}
```

All 5 bind tests pass, covering Val Option, Val Vec, Ref Option, Ref Vec,
and Ref Lazy. The pattern is directly portable to all dispatch-based
functions.

### 7. `pure` return-type inference FAILS (E0283)

`pure_infer` defined as:

```rust
fn pure_infer<Brand: Pointed, A>(a: A) -> Brand::Of<'static, A>
```

Fails in all contexts, including inside `bind_infer` closures:

```rust
bind_infer(Some(5), |x: i32| pure_infer(x + 1))
// E0283: cannot infer type of the type parameter `Brand`
// note: cannot satisfy `_: Pointed`
```

The compiler lists 19+ types implementing `Pointed` and cannot select one
from the return-position constraint. Even though `bind_infer`'s return type
constrains the closure's return to `<OptionBrand as Kind>::Of<i32>`, Rust
does not propagate this constraint backward through `pure_infer`'s `Brand`
parameter.

This is a confirmed, fundamental limitation of Rust's type inference:
return-type constraints do not resolve generic parameters on called
functions when multiple types satisfy the trait bound.

**Impact on macros:** In inferred-mode `m_do!`/`a_do!`, `pure(expr)`
cannot be rewritten to a turbofish-free `pure(expr)` call. The macro must
either:

1. Require users to write concrete constructors (e.g., `Some(expr)`) in
   inferred mode.
2. Use a different mechanism (see macro-interaction.md for options).
3. Not support `pure()` in inferred mode at all.

## Implications for Implementation

The `map_infer` and `bind_infer` signatures validated here should become
the basis for the production `map` and `bind` functions (with the current
versions renamed to `map_explicit` and `bind_explicit`). The same pattern
extends to `apply`, `lift2`, `filter_map`, and other dispatch-based free
functions.

The `pure` function cannot use brand inference and must retain its explicit
Brand turbofish. It should NOT be renamed to `pure_explicit`.

The old POC in `dispatch.rs` can be removed or updated, as the feasibility
test file supersedes it with broader coverage.
