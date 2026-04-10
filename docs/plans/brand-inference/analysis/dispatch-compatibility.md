# Dispatch Compatibility Analysis: Brand Inference After the Ref-Borrow Refactor

## Summary

The ref-borrow refactor changed all dispatch traits to use a two-impl
pattern with an `FA` type parameter. The Val impl sets
`FA = Apply!(Brand::Of<'a, A>)` (owned container); the Ref impl sets
`FA = &'b Apply!(Brand::Of<'a, A>)` (borrowed container). This analysis
examines how the brand-inference plan interacts with this new dispatch
system.

## 1. How the Two-Impl `FA` Parameter Changes Brand Inference

### Old dispatch (pre-refactor)

The old `FunctorDispatch` had no `FA` type parameter. The POC's
`map_infer` function had to:

1. Accept `FA` as the concrete container type.
2. Use `FA: Into<Brand::Of<'a, A>>` to convert the concrete type into the
   brand's associated type before passing it to `FunctorDispatch`.
3. Hardcode `Brand::Of<'a, A>` as the fixed container type for dispatch.
4. Only support Val dispatch. The POC comment states: "Ref dispatch with
   brand inference would require a separate ref_map_infer function."

### New dispatch (post-refactor)

The new `FunctorDispatch` has `FA` as a type parameter on the trait:

```
trait FunctorDispatch<'a, Brand, A, B, FA, Marker>
```

The Val impl sets `FA = Brand::Of<'a, A>`; the Ref impl sets
`FA = &'b Brand::Of<'a, A>`. The free function `map` accepts `fa: FA`
directly, and the compiler selects the correct impl based on whether
`FA` is owned or borrowed.

### Revised inference signature

The proposed inference-based `map` would be:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
    fa: FA,
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
    FA: DefaultBrand,
```

This works for both Val and Ref dispatch, because:

- **Val path:** `FA` is `Option<A>` (for example). `DefaultBrand` resolves
  `Brand = OptionBrand`. The compiler selects the Val impl of
  `FunctorDispatch` where `FA = Brand::Of<'a, A> = Option<A>`. The `FA`
  in the dispatch trait matches the `FA` in the free function directly.

- **Ref path:** `FA` is `&'b Option<A>`. This requires `DefaultBrand`
  to be implemented for `&'b Option<A>` (delegating to `Option<A>`'s
  impl). The compiler selects the Ref impl of `FunctorDispatch` where
  `FA = &'b Brand::Of<'a, A> = &'b Option<A>`. Again, `FA` matches
  directly.

### The `Into` trait becomes unnecessary

In the old POC, `Into` was needed because the concrete type (`FA`) had
to be converted to the brand's associated type before being passed to
a dispatch trait that did not accept `FA`. Now that `FA` is a type
parameter on the dispatch trait, the dispatch impls already constrain
`FA` to be either `Brand::Of<'a, A>` (Val) or `&'b Brand::Of<'a, A>`
(Ref). The type equality is enforced by impl selection, not by a runtime
conversion. The `Into` bound can be removed entirely.

### `DefaultBrand` for references

For the Ref path to work, `DefaultBrand` must be implemented for
reference types. The cleanest approach is a blanket impl:

```rust
impl<'b, T: DefaultBrand> DefaultBrand for &'b T {
    type Brand = T::Brand;
}
```

This allows `&Option<A>` to resolve to `OptionBrand` without individual
impls for each reference type. The POC did not need this because it only
supported Val dispatch; the new system requires it.

## 2. Is the GAT Equality Constraint Still Needed?

### Old plan's constraint

The plan required:

```rust
<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>
```

This bound ensured that the brand's `Of<'a, A>` associated type equals
`FA`, preventing mismatches where `DefaultBrand` maps a type to a brand
whose `Of` produces a different type.

### With `FA` as a dispatch trait parameter

The dispatch impl already constrains `FA`. For example, the Val impl of
`FunctorDispatch` is:

```rust
impl<...> FunctorDispatch<'a, Brand, A, B, Brand::Of<'a, A>, Val> for F
where Brand: Functor, ...
```

For the compiler to select this impl, it must unify `FA` (from the free
function) with `Brand::Of<'a, A>` (from the impl). This means the type
equality `FA = Brand::Of<'a, A>` is already enforced by trait resolution
when the dispatch impl is selected.

**However, the explicit GAT equality bound is still useful** for one
reason: without it, the return type
`<<FA as DefaultBrand>::Brand as Kind>::Of<'a, B>` is a fully abstract
projection. The compiler may not be able to normalize it to a concrete
type (e.g., `Option<B>`) without the equality bound tying `FA` to the
brand's `Of`. The bound gives the compiler the information it needs to
conclude that `Brand::Of<'a, A> = FA`, which in turn helps it normalize
`Brand::Of<'a, B>` to the expected concrete return type.

In practice, whether this bound is needed depends on how Rust's trait
solver handles the interaction between `DefaultBrand` resolution and
dispatch impl selection. The POC confirmed it works with the bound;
removing it should be tested experimentally.

**Verdict:** Keep the GAT equality bound for now. It may be removable,
but removing it is a micro-optimization that risks inference failures.

## 3. Per-Dispatch-Trait Analysis

### FunctorDispatch

```rust
trait FunctorDispatch<'a, Brand, A, B, FA, Marker>
```

Current turbofish: `map::<Brand, _, _, _, _>(...)`
Type params: `Brand, A, B, FA, Marker` (5 params, 1 explicit)

Inferred version: `map::<_, _, _, _>(...)` or just `map(...)`
Type params: `FA, A, B, Marker` (4 params, 0 explicit)

Brand inference composes cleanly. `DefaultBrand` resolves `Brand` from
`FA`; the closure's argument type resolves `Marker`. No extra type
parameters complicate inference.

### BindDispatch

```rust
trait BindDispatch<'a, Brand, A, B, FA, Marker>
```

Current turbofish: `bind::<Brand, _, _, _, _>(...)`
Type params: `Brand, A, B, FA, Marker` (5 params, 1 explicit)

Inferred version: `bind(...)`
Type params: `FA, A, B, Marker` (4 params, 0 explicit)

Same structure as `FunctorDispatch`. Composes cleanly.

Note: `bind` takes `(ma, f)` with `ma: FA` as the first argument. Brand
is inferred from `ma`'s type. No issues.

### ComposeKleisliDispatch

```rust
trait ComposeKleisliDispatch<'a, Brand, A, B, C, Marker>
```

Current turbofish: `compose_kleisli::<Brand, _, _, _, _>(...)`
Type params: `Brand, A, B, C, Marker` (5 params, 1 explicit)

This trait does NOT have an `FA` parameter. The function takes `(fg, a)`
where `a: A` is a plain value, not a container. There is no container
type to resolve `DefaultBrand` from. The closures return `Brand::Of<B>`
and `Brand::Of<C>`, but the return type is not usable for `DefaultBrand`
inference (Rust infers forward, not backward from return types).

**Brand inference does not apply** to `compose_kleisli` and
`compose_kleisli_flipped`. They must keep the explicit Brand parameter.

### Lift2Dispatch through Lift5Dispatch

```rust
trait Lift2Dispatch<'a, Brand, A, B, C, FA, FB, Marker>
```

Current turbofish: `lift2::<Brand, _, _, _, _, _, _>(...)`
Type params: `Brand, A, B, C, FA, FB, Marker` (7 params, 1 explicit)

Inferred version: `lift2(...)`
Type params: `FA, A, B, C, FB, Marker` (6 params, 0 explicit)

Brand inference works: `DefaultBrand` resolves from `FA` (the first
container argument). `FB` is the second container argument and must be
the same brand's container (or reference to it). The dispatch impl
already enforces `FB = Brand::Of<'a, B>` or `FB = &'b Brand::Of<'a, B>`,
so no additional constraint is needed on `FB` for brand resolution.

One subtlety: `FA` and `FB` must resolve to the same brand. With
`DefaultBrand` resolving from `FA`, the dispatch impl constrains `FB`
to be consistent. If someone passes `Some(1)` and `vec![2]`, the
dispatch impl selection will fail because no impl exists for mismatched
brands. This is correct behavior.

Higher-arity lifts (lift3 through lift5) follow the same pattern with
additional container type parameters. Brand inference works identically,
resolving from `FA`.

### FoldRightDispatch, FoldLeftDispatch, FoldMapDispatch

```rust
trait FoldRightDispatch<'a, FnBrand, Brand, A, B, FA, Marker>
```

Current turbofish: `fold_right::<FnBrand, Brand, _, _, _, _>(...)`
Type params: `FnBrand, Brand, A, B, FA, Marker` (6 params, 2 explicit)

Inferred version: `fold_right::<FnBrand, _, _, _, _>(...)`
Type params: `FnBrand, FA, A, B, Marker` (5 params, 1 explicit)

**`FnBrand` cannot be inferred.** It selects between `RcFnBrand` and
`ArcFnBrand` for the internal cloneable function representation. There
is no source to infer it from: it does not appear in the container type,
the closure type, or the accumulator type. The caller must still specify
`FnBrand` explicitly.

Brand inference removes one turbofish position (Brand) but `FnBrand`
remains. The turbofish changes from `fold_right::<FnBrand, Brand, ...>`
to `fold_right::<FnBrand, ...>`. This is still an improvement (one fewer
explicit parameter), but foldable functions cannot be fully turbofish-free.

`FoldLeftDispatch` and `FoldMapDispatch` have the same structure and the
same `FnBrand` complication.

### FilterMapDispatch

```rust
trait FilterMapDispatch<'a, Brand, A, B, FA, Marker>
```

Current turbofish: `filter_map::<Brand, _, _, _, _>(...)`
Type params: `Brand, A, B, FA, Marker` (5 params, 1 explicit)

Inferred version: `filter_map(...)`
Type params: `FA, A, B, Marker` (4 params, 0 explicit)

Same structure as `FunctorDispatch`. Composes cleanly. No extra type
parameters.

### TraverseDispatch

```rust
trait TraverseDispatch<'a, FnBrand, Brand, A, B, F, FTA, Marker>
```

Current turbofish: `traverse::<FnBrand, Brand, _, _, F, _, _>(...)`
Type params: `FnBrand, Brand, A, B, F, FTA, Marker` (7 params, 3 explicit)

**Two parameters resist inference:**

1. `FnBrand` - same issue as foldable. Cannot be inferred.
2. `F` (the applicative brand for the effect) - this is the brand of the
   return type's outer context (e.g., `OptionBrand` when traversing with
   `Option`). The closure returns `F::Of<'a, B>`, so in principle `F`
   could be inferred from the closure's return type. However, Rust does
   not infer generic parameters from return types of closures that are
   themselves generic. The closure `|x| Some(x)` has return type
   `Option<B>`, but the compiler cannot reverse-map that to
   `F = OptionBrand` without a `DefaultBrand`-like mechanism on the
   return type.

   If a second `DefaultBrand` resolution were added for the closure's
   return type, `F` could potentially be inferred. But this would require
   `DefaultBrand` to be implemented for the return type and the compiler
   to resolve it during closure type checking. This is speculative and
   should be deferred.

Inferred version (partial): `traverse::<FnBrand, _, _, F, _, _>(...)`
Type params: `FnBrand, FA, A, B, F, Marker` (6 params, 2 explicit)

Brand inference removes one turbofish position (Brand) but `FnBrand` and
`F` remain explicit. `traverse` is the hardest function to make
turbofish-free.

## 4. Turbofish Changes Summary

| Function                  | Current turbofish                              | Explicit params | Inferred turbofish                   | Explicit params | Change |
| ------------------------- | ---------------------------------------------- | --------------- | ------------------------------------ | --------------- | ------ |
| `map`                     | `map::<Brand, _, _, _, _>`                     | 1               | `map(...)`                           | 0               | -1     |
| `bind`                    | `bind::<Brand, _, _, _, _>`                    | 1               | `bind(...)`                          | 0               | -1     |
| `bind_flipped`            | `bind_flipped::<Brand, _, _, _, _>`            | 1               | `bind_flipped(...)`                  | 0               | -1     |
| `compose_kleisli`         | `compose_kleisli::<Brand, _, _, _, _>`         | 1               | N/A (no container arg)               | 1               | 0      |
| `compose_kleisli_flipped` | `compose_kleisli_flipped::<Brand, ...>`        | 1               | N/A (no container arg)               | 1               | 0      |
| `lift2`                   | `lift2::<Brand, _, _, _, _, _, _>`             | 1               | `lift2(...)`                         | 0               | -1     |
| `lift3`                   | `lift3::<Brand, _, _, _, _, _, _, _, _>`       | 1               | `lift3(...)`                         | 0               | -1     |
| `lift4`                   | `lift4::<Brand, _, _, _, _, _, _, _, _, _, _>` | 1               | `lift4(...)`                         | 0               | -1     |
| `lift5`                   | `lift5::<Brand, ...>`                          | 1               | `lift5(...)`                         | 0               | -1     |
| `filter_map`              | `filter_map::<Brand, _, _, _, _>`              | 1               | `filter_map(...)`                    | 0               | -1     |
| `fold_right`              | `fold_right::<FnBrand, Brand, _, _, _, _>`     | 2               | `fold_right::<FnBrand, _, _, _, _>`  | 1               | -1     |
| `fold_left`               | `fold_left::<FnBrand, Brand, _, _, _, _>`      | 2               | `fold_left::<FnBrand, _, _, _, _>`   | 1               | -1     |
| `fold_map`                | `fold_map::<FnBrand, Brand, _, _, _, _>`       | 2               | `fold_map::<FnBrand, _, _, _, _>`    | 1               | -1     |
| `traverse`                | `traverse::<FnBrand, Brand, _, _, F, _, _>`    | 3               | `traverse::<FnBrand, _, _, F, _, _>` | 2               | -1     |

Every dispatch function gains exactly one fewer explicit turbofish
parameter. Functions that only had Brand as their explicit parameter
become fully turbofish-free. Functions with additional explicit
parameters (`FnBrand`, `F`) still require a partial turbofish.

## 5. What Breaks or Needs Updating in the Plan

### 5.1 POC is outdated

The POC at the bottom of `fp-library/src/classes/dispatch.rs` uses the
pre-refactor dispatch system:

- It hardcodes `Brand::Of<'a, A>` as the `FA` parameter to
  `FunctorDispatch`, bypassing the new `FA` type parameter design.
- It uses `Into<Brand::Of<'a, A>>` which is no longer needed.
- It only tests Val dispatch. The plan states "Ref dispatch with brand
  inference would require a separate ref_map_infer function," which was
  true under the old system but is false under the new system. With `FA`
  as a dispatch trait parameter, a single `map` function handles both
  Val and Ref dispatch when `DefaultBrand` is implemented for references.

The POC should be replaced with one that uses the current dispatch system
and tests both Val and Ref paths.

### 5.2 Plan section "Interaction with FunctorDispatch" is incorrect

The plan says:

> "FunctorDispatch resolves the Marker parameter (Val vs Ref).
> DefaultBrand resolves the Brand parameter."

This is still directionally correct but omits the `FA` parameter, which
is now central to how both mechanisms work. The `FA` type parameter is
the pivot: `DefaultBrand` resolves `Brand` from `FA`, and the dispatch
impl selection resolves `Marker` from `FA` (owned -> Val, reference -> Ref).
Both inference axes flow from `FA`.

### 5.3 Plan section "Inference-Based Free Functions" signature is outdated

The plan shows:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>,
    fa: FA,
) -> ...
```

The `FunctorDispatch` trait now has 6 type parameters (including `FA`),
not 5. The correct signature is:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
    fa: FA,
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
    FA: DefaultBrand,
```

### 5.4 Plan omits `DefaultBrand` for references

The plan does not discuss implementing `DefaultBrand` for `&T`. Under
the old system, this was not needed because Ref dispatch was unsupported.
Under the new system, a blanket `impl<'b, T: DefaultBrand> DefaultBrand
for &'b T` is required for Ref dispatch to work with brand inference.
This needs to be added to the plan.

### 5.5 Plan's `Into` bound is unnecessary

The plan's signature includes:

```rust
<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>
```

The `Into` conversion in the POC (`fa.into()`) is no longer needed
because `FA` flows directly into the dispatch trait. The plan should
remove any mention of `Into`-based conversion. The GAT equality bound
may still be needed (see section 2) but is no longer paired with `Into`.

### 5.6 Foldable and traversable functions are underspecified

The plan's implementation order (steps 5-6) mentions extending to `bind`,
`lift2`, `apply` but does not address the `FnBrand` complication for
foldable and traversable functions. These functions cannot become fully
turbofish-free because `FnBrand` has no inference source. The plan should
explicitly document this limitation and clarify that for `fold_right`,
`fold_left`, `fold_map`, and `traverse`, brand inference reduces the
turbofish by one position but does not eliminate it.

### 5.7 `compose_kleisli` and `compose_kleisli_flipped` cannot use inference

The plan does not mention these functions at all. They take `(fg, a)`
where `a: A` is a plain value, not a container. There is no `FA` to
resolve `DefaultBrand` from. These functions must keep explicit Brand
parameters and should not be renamed to `_explicit` variants.

### 5.8 Plan's turbofish count is wrong

The plan states the current `map` has signature
`map::<Brand, _, _, _>(...)` with 4 type params. The actual current
signature has 5 type params: `Brand, A, B, FA, Marker`. The plan was
written before the refactor added `FA`. All turbofish examples in the
plan need updating.

### 5.9 The `map_infer` POC name collision concern is resolved

The plan notes that `map_infer` is a temporary POC name. Since the POC
is outdated anyway (section 5.1), this is moot. A new POC should be
written against the current dispatch system.
