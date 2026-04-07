# Dispatch System Analysis

## Overview

The dispatch system lives in `fp-library/src/classes/dispatch.rs` and its
sub-modules (`dispatch/functor.rs`, `dispatch/semimonad.rs`,
`dispatch/foldable.rs`, `dispatch/lift.rs`). It uses marker types `Val`
and `Ref` to let a single free function (e.g., `map`, `bind`,
`fold_right`) route to either by-value trait methods or by-reference
trait methods based on the closure's argument type. The `ClosureMode`
trait maps markers to `dyn Fn` target types, used by `CloneFn` and
`SendCloneFn` to parameterize wrapped closures.

Dispatch traits currently exist for:

- `FunctorDispatch` (map)
- `BindDispatch` (bind)
- `BindFlippedDispatch` (bind_flipped)
- `ComposeKleisliDispatch` (compose_kleisli)
- `ComposeKleisliFlippedDispatch` (compose_kleisli_flipped)
- `FoldRightDispatch` (fold_right)
- `FoldLeftDispatch` (fold_left)
- `FoldMapDispatch` (fold_map)
- `Lift2Dispatch` through `Lift5Dispatch` (lift2 through lift5)

Total: 12 dispatch traits, each with Val and Ref impls = 24 impl blocks.

## 1. Inference Correctness

### General case: works correctly

The compiler resolves the `Marker` type parameter from the `Fn` impl
of the closure. A closure `|x: i32| x * 2` implements `Fn(i32) -> i32`
and satisfies only the `Val` impl (which requires `F: Fn(A) -> B`).
A closure `|x: &i32| *x * 2` implements `Fn(&i32) -> i32` and
satisfies only the `Ref` impl (which requires `F: Fn(&A) -> B`).
These two `Fn` traits are distinct, so there is no ambiguity.

### Edge case: untyped closures

When the closure parameter type is not annotated, Rust must infer it
from context. For `map::<OptionBrand, _, _, _>(|x| x * 2, Some(5))`,
the compiler infers `x: i32` from the `Some(5)` argument and the
Brand's `Kind::Of` associated type, resolving to `Val`. This works
because the container type constrains `A`.

However, with `None`, inference fails without type annotations:
`map::<OptionBrand, _, _, _>(|x| x * 2, None)` cannot determine `A`.
The test in `dispatch.rs` handles this by specifying
`map::<OptionBrand, i32, i32, _>(|x| x * 2, None)`. This is a
standard Rust limitation, not a dispatch system flaw.

### Edge case: closures taking `&T` where T is itself a reference

If the container holds references (e.g., `Vec<&str>`), then:

- A `Val` closure would be `|x: &str| ...` (takes `&str` by value).
- A `Ref` closure would be `|x: &&str| ...` (takes `&&str`).

The `Val` closure `|x: &str|` is `Fn(&str) -> B`, which matches the
`Ref` impl's `F: Fn(&A) -> B` with `A = str`. This is an ambiguity.
In practice, if the Brand's `Of<'a, A>` resolves to `Vec<&str>`, then
`A = &str`, and a closure `|x: &str|` taking `A` by value matches
`Fn(&str) -> B` which is `Fn(A) -> B` with `A = &str` (Val path),
but also `Fn(&A) -> B` with `A = str` (Ref path). The compiler would
reject this as ambiguous. The user must annotate the closure type to
disambiguate.

This edge case is unlikely in practice (types stored in the library's
HKT containers are typically owned), but it is a fundamental limitation
of the dispatch-on-closure-argument approach.

### Edge case: mixed-mode Kleisli composition

The `ComposeKleisliDispatch` Ref impl requires both closures to take
references: `F: Fn(&A)` and `G: Fn(&B)`. There is no way to compose
a `Val` closure with a `Ref` closure (e.g., `Fn(A) -> Of<B>` then
`Fn(&B) -> Of<C>`) through this dispatch. Both closures must be the
same mode. This is intentional and documented, but may limit usage.

## 2. Kleisli Composition Dispatch Correctness

### Ref compose_kleisli

The Ref impl of `ComposeKleisliDispatch` (line 474-479 of
semimonad.rs):

```rust
fn dispatch(self, a: A) -> ... {
    Brand::ref_bind(self.0(&a), self.1)
}
```

The function takes `a: A` by value. The call `self.0(&a)` borrows the
owned local `a` and passes `&A` to the first Kleisli arrow. This is
correct: `a` is owned by the stack frame, so `&a` creates a valid
borrow. The result `Of<B>` is then passed to `ref_bind`, which will
internally provide `&B` to `self.1`.

### Ref compose_kleisli_flipped

Similarly correct. `self.1(&a)` borrows the owned local.

### Semantic question: should Kleisli take `&A` instead of `A`?

The `compose_kleisli` free function takes `a: A` (owned) even in Ref
mode. This means the caller must own the value. An alternative design
would take `a: &A` in Ref mode, but this would require a second
dispatch axis on the input parameter, adding complexity for little
practical gain. The current design is sound.

## 3. Code Duplication

### Pattern repetition

Each dispatch trait follows an identical pattern:

1. Define a trait with a `Marker` type parameter.
2. Implement it for `F where F: Fn(A...) -> R` with `Marker = Val`.
3. Implement it for `F where F: Fn(&A...) -> R` with `Marker = Ref`.
4. Define a free function that calls `f.dispatch_method(...)`.

This pattern is repeated 12 times. The boilerplate per trait is
approximately:

- 1 trait definition (~10 lines of logic + ~20 lines of doc attrs)
- 2 impl blocks (~15 lines each + ~20 lines of doc attrs)
- 1 free function (~5 lines of logic + ~20 lines of doc attrs)

Total: approximately 150-200 lines per dispatch trait, of which
perhaps 50 lines are structural logic and 100-150 lines are
documentation attributes.

### Could a macro reduce this?

Yes. A declarative macro could generate the trait, both impls, and the
free function from a specification like:

```rust
define_dispatch! {
    trait FunctorDispatch for map {
        val_trait: Functor,
        ref_trait: RefFunctor,
        args: (fa: Of<A>),
        returns: Of<B>,
        val_call: Brand::map(self, fa),
        ref_call: Brand::ref_map(self, fa),
    }
}
```

The main obstacle is documentation: the library has strict doc
standards with `#[document_signature]`, `#[document_type_parameters]`,
etc. Generating these from a macro would require either:

- Embedding all doc strings in the macro invocation (verbose).
- Using a proc macro that generates doc attributes (complex).
- Accepting less detailed documentation on dispatch traits (trade-off).

**Recommendation**: The current duplication is manageable for 12
traits. A macro would reduce correctness risk when adding new dispatch
traits but would obscure the code for readers. Given that the dispatch
set is unlikely to grow much further (the plan identifies the
remaining non-dispatchable operations), the status quo is acceptable.
If more dispatch traits are added, a macro should be considered.

### Specific duplication: BindFlippedDispatch vs BindDispatch

`BindFlippedDispatch` is nearly identical to `BindDispatch` but with
swapped argument order in the free function. The dispatch trait and
impls are duplicated. An alternative would be to have `bind_flipped`
call `bind` with reordered arguments. However, the dispatch trait is
on the closure, not the container, so this reordering is already
handled at the free function level. The duplication of the entire
dispatch trait is unnecessary; `bind_flipped` could be:

```rust
pub fn bind_flipped<...>(
    f: impl BindDispatch<..., Marker>,
    ma: ...,
) -> ... {
    f.dispatch_bind(ma)
}
```

This reuses `BindDispatch` directly. The separate
`BindFlippedDispatch` trait adds no dispatch capability that
`BindDispatch` doesn't already provide. The only difference is
argument order in the free function signature, which is handled by
the function itself, not the trait.

## 4. Foldable Clone Bounds

### The `A: Clone` requirement

All foldable dispatch impls require `A: Clone`, including the Ref
path. This is necessary because the underlying `Foldable` and
`RefFoldable` trait methods require `A: Clone`:

- `Foldable::fold_right` requires `A: Clone` because the default
  implementation via `fold_map` clones elements to build
  `Endofunction` closures.
- `RefFoldable::ref_fold_right` requires `A: Clone` because its
  default implementation calls `a.clone()` to capture elements into
  `Endofunction` closures that wrap `Fn(A, B) -> B` callbacks.

The Clone bound on the Ref path is structurally required by the
mutual derivation mechanism (fold_right <-> fold_map via
Endofunction), not by the dispatch system itself. Even though the
user-facing closure takes `&A`, the internal machinery needs to clone
elements to build deferred function compositions.

### Could this be relaxed?

For types that directly implement `ref_fold_right` without using the
default (e.g., Vec iterates by reference), the Clone bound is
unnecessary. However, the trait definition requires it because the
default implementation needs it, and Rust traits cannot have
conditional bounds on default methods. This is a limitation of the
foldable trait design, not the dispatch system.

**Observation**: The `fold_map` dispatch free function signature does
NOT have `A: Clone` in its own signature (line 526), but both Val and
Ref impls of `FoldMapDispatch` do. This means the bound is enforced
by the impl resolution, not the function signature. This is correct
but may produce confusing error messages when `A: Clone` is not
satisfied, pointing at the impl rather than the call site.

## 5. Brand Inference POC

### Current state

The `brand_inference_poc` module in `dispatch.rs` is test-only code
that validates a `DefaultBrand` trait mapping concrete types to their
canonical brands. It enables turbofish-free calls:
`map_infer(|x: i32| x * 2, Some(5))` instead of
`map::<OptionBrand, _, _, _>(...)`.

### Trade-offs for promotion

**Advantages**:

- Eliminates the most common ergonomic complaint (turbofish with
  Brand). Users could write `map(f, container)` without any type
  annotations.
- Works with dispatch (the POC composes `DefaultBrand` with
  `FunctorDispatch`).
- Backward compatible as an opt-in alternative.

**Disadvantages**:

- **Orphan rule conflicts**: `DefaultBrand` impls must exist for
  every concrete type. For types defined outside the crate, the impl
  must be in the crate that defines either the trait or the type.
  Third-party types cannot implement `DefaultBrand` without a newtype
  wrapper.
- **Ambiguity for types with multiple brands**: Some types might be
  representable by multiple brands (unlikely in practice, but
  possible with wrapper types).
- **Requires `Into` conversion**: The POC requires
  `FA: Into<Brand::Of<'a, A>>`. For types where `Of<'a, A>` is the
  same as `FA` (e.g., `Option<A>`), this is trivially satisfied.
  For types where `Of<'a, A>` wraps the concrete type, an explicit
  `Into` impl is needed.
- **Does not scale to all operations**: Operations like `fold_right`
  that take an `FnBrand` parameter cannot infer both Brand and
  FnBrand from the container type.

**Recommendation**: The POC validates the concept but has limited
applicability. It works well for simple operations (map, bind, lift2)
on common types (Option, Vec, Lazy). It does not generalize to all
dispatch operations. Consider promoting it as an ergonomic
convenience layer alongside (not replacing) the turbofish versions.
The trait should be documented as best-effort, with explicit brand
specification as the canonical API.

## 6. Import Path Fragility

### `super::super::Val` vs direct import

Three of four dispatch sub-modules (`functor.rs`, `semimonad.rs`,
`lift.rs`) reference the marker types via `super::super::Val` and
`super::super::Ref`. The path traverses two module boundaries:

1. `super` exits the `inner` module (used for `document_module`).
2. `super` exits the sub-module file (e.g., `functor.rs`).

This reaches `dispatch.rs`, which re-exports `Val` and `Ref` from
its own `inner` module.

The fourth sub-module (`foldable.rs`) imports `Val` and `Ref` via
`use crate::classes::dispatch::{Ref, Val}`. This is more robust
because it does not depend on the relative nesting depth.

### Fragility assessment

The `super::super` paths are fragile in two ways:

1. **Restructuring risk**: If the `inner` module wrapper is removed
   or the file hierarchy changes, the path breaks.
2. **Inconsistency**: The mixed approach (some files use
   `super::super`, one uses `use`) is a maintenance hazard. A
   developer following the pattern in one file will use the wrong
   approach in another.

**Recommendation**: Standardize on the `foldable.rs` approach. All
sub-modules should import `Val` and `Ref` via
`use crate::classes::dispatch::{Val, Ref}` or equivalently via the
`use` block at the top of the `inner` module. This is a mechanical
change with no semantic impact.

## 7. Missing Dispatchable Operations

### Operations with Ref variants that take closures

The following operations have both Val and Ref variants, take closures
whose argument type could drive dispatch, but are not currently in the
dispatch system:

| Operation               | Val trait              | Ref trait                 | Dispatchable? |
| ----------------------- | ---------------------- | ------------------------- | ------------- |
| `filter_map`            | `Filterable`           | `RefFilterable`           | Yes           |
| `filter`                | `Filterable`           | `RefFilterable`           | Yes           |
| `traverse`              | `Traversable`          | `RefTraversable`          | Yes           |
| `wither`                | `Witherable`           | `RefWitherable`           | Yes           |
| `wilt`                  | `Witherable`           | `RefWitherable`           | Yes           |
| `map_with_index`        | `FunctorWithIndex`     | `RefFunctorWithIndex`     | Yes           |
| `fold_map_with_index`   | `FoldableWithIndex`    | `RefFoldableWithIndex`    | Yes           |
| `filter_map_with_index` | `FilterableWithIndex`  | `RefFilterableWithIndex`  | Yes           |
| `filter_with_index`     | `FilterableWithIndex`  | `RefFilterableWithIndex`  | Yes           |
| `traverse_with_index`   | `TraversableWithIndex` | `RefTraversableWithIndex` | Yes           |

All of these take closures as `Fn(A, ...) -> R` (Val) or
`Fn(&A, ...) -> R` (Ref), so the dispatch mechanism would work
identically to existing dispatchers.

### Priority assessment

The indexed variants and filterable/traversable operations are less
frequently used than map/bind/fold/lift. Adding dispatch for all of
them would significantly increase the dispatch module's size without
proportional ergonomic benefit. Users who need both Val and Ref paths
for these operations can call the specific free functions directly.

**Recommendation**: Add dispatch for `filter_map` and `traverse` as
these are the most commonly used. Defer the rest unless user demand
emerges.

### Operations correctly excluded from dispatch

The plan (step 23) correctly identifies operations that cannot be
dispatched:

- `pure`, `ref_pure`: No closure argument to infer from.
- `apply`, `ref_apply`: Takes `Of<CloneFn::Of<...>>`, not a raw
  closure. Mode is determined by the `CloneFn<Mode>` parameter, not
  argument type inference.
- `apply_first`, `apply_second`: Take two containers, no closure.
- `join`, `ref_join`: No closure.
- `compact`, `separate`: No closure.
- `if_m`, `unless_m`, `when_m`, `when`, `unless`: Take containers,
  not closures.

## 8. FnBrand Parameter on Foldable Dispatch

### The complexity

Foldable dispatch functions require callers to specify an `FnBrand`
type parameter:

```rust
fold_right::<RcFnBrand, VecBrand, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3])
```

Compared to functor dispatch:

```rust
map::<VecBrand, _, _, _>(|x: i32| x + 1, vec![1, 2, 3])
```

The extra `RcFnBrand` parameter is required because the underlying
`Foldable::fold_right` needs to wrap closures in `CloneFn` objects
(via `Endofunction`) for the fold_map <-> fold_right mutual
derivation.

### Assessment

The `FnBrand` parameter is an implementation detail of the foldable
trait design. Users must know whether to use `RcFnBrand` or
`ArcFnBrand`, which is a threading concern leaked into the fold API.

However, this is not a dispatch system problem; it is inherited from
the `Foldable` trait's design. The dispatch system faithfully
propagates the parameter.

### Possible improvement

A default `FnBrand = RcFnBrand` on the dispatch free functions would
reduce call-site noise for the common single-threaded case:

```rust
pub fn fold_right<'a, FnBrand = RcFnBrand, Brand: ..., A: ..., B: ..., Marker>(...)
```

However, Rust does not support default type parameters on functions
(only on types and traits). This improvement is blocked by a language
limitation.

An alternative is to provide convenience aliases:

```rust
pub fn fold_right_rc<...>(...) {
    fold_right::<RcFnBrand, Brand, A, B, Marker>(...)
}
```

This adds API surface but reduces call-site noise. Whether this
trade-off is worthwhile depends on how frequently fold operations
appear at call sites.

## 9. Additional Observations

### Documentation weight

The dispatch module files are heavily weighted toward documentation.
For example, `dispatch/functor.rs` is 231 lines, of which
approximately 50 lines are logic and 180 lines are documentation
attributes and doc comments. This is consistent with the project's
documentation standards but makes the dispatch code harder to scan
for structural issues. The documentation is thorough and accurate.

### Lift3-5 Ref impls require Clone on intermediate types

The Ref impls for `Lift3Dispatch` through `Lift5Dispatch` require
`Clone` on intermediate tuple types because they build N-ary lifts
from binary `ref_lift2` calls. For example, `lift3` Ref:

```rust
Brand::ref_lift2(
    move |(a, b): &(A, B), c: &C| self(a, b, c),
    Brand::ref_lift2(|a: &A, b: &B| (a.clone(), b.clone()), fa, fb),
    fc,
)
```

The inner `ref_lift2` clones `A` and `B` to construct a `(A, B)`
tuple. This requires `A: Clone + 'a` and `B: Clone + 'a`, even
though the outer user closure receives references. The last parameter
(`C` for lift3, `D` for lift4, `E` for lift5) does NOT require Clone
because it is passed directly to the final `ref_lift2` without
intermediate materialization.

This is correct but may surprise users who expect the Ref path to
avoid all cloning. The Clone bounds are documented in the trait impls
but not prominently called out in the free function documentation.

### The `Marker` type parameter never appears in return types

All dispatch traits use `Marker` purely for impl selection, never in
the return type. This means the compiler can always resolve `Marker`
from the closure argument alone, without needing return-type
information. This is a sound design that avoids inference
difficulties.

### No SendRef dispatch

The dispatch system covers `Val` (by-value traits) and `Ref`
(by-reference traits). There is no dispatch for `SendRef` variants
(e.g., `SendRefFunctor`, `SendRefSemimonad`). This is correct because
`SendRef` traits have different bounds (`Send + Sync` on closures)
that would require a third marker type. The `Send + Sync` distinction
is orthogonal to the `Val`/`Ref` distinction and is better handled by
separate free functions or the `SendCloneFn` system.

## Summary of Recommendations

1. **Import consistency**: Standardize all dispatch sub-modules on
   `use crate::classes::dispatch::{Val, Ref}` instead of mixed
   `super::super` paths.

2. **Remove BindFlippedDispatch**: Reuse `BindDispatch` for
   `bind_flipped` by swapping arguments in the free function only.

3. **Add dispatch for filter_map and traverse**: These are the
   highest-value missing dispatch operations.

4. **Document Clone requirements on Ref lift paths**: The lift3-5 Ref
   impls require Clone on intermediate types; this should be mentioned
   in the free function documentation.

5. **Brand inference**: Consider promoting the POC as an opt-in
   ergonomic layer for common operations (map, bind, lift2) on
   standard types. Document its limitations clearly.

6. **FnBrand convenience**: If fold operations are common at call
   sites, consider adding `fold_right_rc` / `fold_left_rc` / etc.
   convenience functions that default to `RcFnBrand`.

7. **Macro generation**: Defer. The current 12 dispatch traits are
   manageable. Revisit if the set grows significantly.
