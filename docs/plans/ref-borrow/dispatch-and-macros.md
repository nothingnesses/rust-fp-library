# Dispatch System and Macros: Interaction with Ref Trait Methods

## Part 1: Dispatch System

The dispatch system uses marker types `Val` and `Ref` (defined in
`fp-library/src/classes/dispatch.rs`) to route calls based on whether the
user's closure takes `A` or `&A`. Each dispatch trait has two impl blocks (one
for `Val`, one for `Ref`) and a unified free function.

### 1.1 How Each Ref Impl Calls the Underlying Trait Method

| Dispatch trait                 | Ref impl calls                                              | Container parameter name |
| ------------------------------ | ----------------------------------------------------------- | ------------------------ |
| `FunctorDispatch`              | `Brand::ref_map(self, fa)`                                  | `fa`                     |
| `BindDispatch`                 | `Brand::ref_bind(ma, self)`                                 | `ma`                     |
| `FoldRightDispatch`            | `Brand::ref_fold_right::<FnBrand, A, B>(self, initial, fa)` | `fa`                     |
| `FoldLeftDispatch`             | `Brand::ref_fold_left::<FnBrand, A, B>(self, initial, fa)`  | `fa`                     |
| `FoldMapDispatch`              | `Brand::ref_fold_map::<FnBrand, A, M>(self, fa)`            | `fa`                     |
| `FilterMapDispatch`            | `Brand::ref_filter_map(self, fa)`                           | `fa`                     |
| `TraverseDispatch`             | `Brand::ref_traverse::<FnBrand, A, B, F>(self, ta)`         | `ta`                     |
| `Lift2Dispatch`                | `Brand::ref_lift2(self, fa, fb)`                            | `fa`, `fb`               |
| `Lift3Dispatch`                | `Brand::ref_lift2(...)` (nested calls)                      | `fa`, `fb`, `fc`         |
| `Lift4Dispatch`                | `Brand::ref_lift2(...)` (nested calls)                      | `fa`, `fb`, `fc`, `fd`   |
| `Lift5Dispatch`                | `Brand::ref_lift2(...)` (nested calls)                      | `fa`-`fe`                |
| `ComposeKleisliDispatch` (Ref) | `Brand::ref_bind(self.0(&a), self.1)`                       | result of `self.0(&a)`   |

In every case, the container is passed **by value** to the underlying trait
method. The dispatch impl's `fn dispatch(self, fa: ...)` takes `fa` by value,
and forwards it by value to e.g. `Brand::ref_map(self, fa)`.

### 1.2 Would Changing Ref Trait Methods to Take `&fa` Work Without Other Changes?

**No, not without mechanical changes to the dispatch impls**, but the dispatch
_free function signatures_ would not need to change. Consider what happens at
each layer:

1. **Dispatch free function**: Takes `fa` by value (e.g., `pub fn map(..., fa: Apply!(...))`)
2. **Dispatch trait impl**: Takes `fa` by value in `fn dispatch(self, fa: ...)`
3. **Underlying trait method call**: Currently `Brand::ref_map(self, fa)`, would become `Brand::ref_map(self, &fa)`

If only the trait method signature changed (to take `&Apply!(...)`) and the
dispatch impl changed to pass `&fa`, this would compile at the dispatch layer
because the dispatch impl owns `fa` and can borrow it. The dispatch free
function would not need to change its signature, since it passes `fa` by value
to the dispatch trait, which then borrows internally.

**However**, this only works if:

- The trait method's return type does not borrow from the container (it does not
  in any current Ref trait; they all return owned `Apply!(Of<B>)`).
- The underlying trait implementations can actually work with `&fa` instead of
  owned `fa`. This is the real question, which is outside the dispatch layer.

At the dispatch layer specifically, passing `&fa` to the trait method after
receiving `fa` by value is mechanically straightforward. The dispatch layer
would act as an adapter: take ownership, borrow, call the trait.

### 1.3 Container Use After Trait Method Call

**No dispatch impl uses the container after the trait method call.** In every
Ref dispatch impl, the container (`fa`, `ma`, `ta`) is passed directly to the
trait method as the last operation, and the result is returned immediately.
There is no post-call use that would be invalidated by borrowing.

The `ComposeKleisliDispatch` Ref impl is a slight variation: it calls
`self.0(&a)` first (where `a` is the raw input value, not a container), then
passes the _result_ to `Brand::ref_bind(...)`. The result of `self.0(&a)` is a
fresh container, so it is used only once. No issues here either.

For `Lift3Dispatch` through `Lift5Dispatch`, the Ref impls chain nested
`Brand::ref_lift2(...)` calls. Each intermediate result is passed by value into
the next `ref_lift2` call. If `ref_lift2` changed to borrow its containers,
these intermediate results would need to be bound to temporaries (let bindings)
to ensure they live long enough. Currently they are inline expressions, which
would work as temporaries in Rust's expression evaluation model (temporaries
live until the end of the enclosing statement). So this should work without
structural changes.

## Part 2: m_do! and a_do! Macros

### 2.1 What m_do!(ref { x <- expr; ... }) Generates

In ref mode, `m_do!` generates calls to the **dispatch** `bind` function (not
`ref_bind` directly). For example:

```rust
m_do!(ref Brand {
    x <- expr1;
    y <- expr2;
    pure(x + y)
})
```

Expands to (approximately):

```rust
bind::<Brand, _, _, _>(expr1, move |x: &_| {
    bind::<Brand, _, _, _>(expr2, move |y: &_| {
        ref_pure::<Brand, _>(&(x + y))
    })
})
```

Key observations:

- `expr1` and `expr2` are passed **by value** to the dispatch `bind` function.
- The closure parameters get `&_` type annotations, which triggers the `Ref`
  dispatch path (the compiler infers the `Ref` marker from `Fn(&A) -> ...`).
- `pure(x)` is rewritten to `ref_pure::<Brand, _>(&(x))`.
- Sequence statements (`expr;`) use `_: &_` as the discard pattern.

### 2.2 What a_do!(ref { x <- expr; ... }) Generates

In ref mode, `a_do!` generates calls to the dispatch free functions `map` and
`liftN`. For example:

```rust
a_do!(ref Brand {
    x <- expr1;
    y <- expr2;
    x + y
})
```

With 2 bindings, expands to:

```rust
lift2::<Brand, _, _, _, _>(|x: &_, y: &_| { x + y }, expr1, expr2)
```

With 1 binding:

```rust
map::<Brand, _, _, _>(|x: &_| { final_expr }, expr1)
```

With 0 bindings:

```rust
ref_pure::<Brand, _>(&(final_expr))
```

Again, all `exprN` values are passed **by value** to the dispatch functions.
The `&_` annotations on closure parameters trigger `Ref` dispatch.

### 2.3 Macro Changes Needed if Ref Trait Methods Change to Take `&container`

**No macro changes would be needed**, assuming the dispatch layer absorbs the
change (takes by value, borrows internally).

The macros generate calls to the dispatch free functions (`bind`, `map`,
`lift2`, etc.), not to the Ref trait methods directly. The dispatch free
functions' signatures would remain unchanged (taking containers by value). The
dispatch trait impls would handle the internal borrow.

If instead the dispatch free functions themselves changed to take `&container`,
then the macros **would** need to change. They would need to:

1. Bind each `expr` to a `let` temporary.
2. Pass `&temp` to the dispatch function.

This would require codegen changes in both `m_do_worker` and `a_do_worker`.

However, the more natural approach is to keep the dispatch free functions
taking by value and let the dispatch impls borrow internally, which requires
**zero** macro changes.

### 2.4 Lifetime Issues with Generating Borrows in Macro Output

If the macros were to generate `&expr` instead of `expr`, there would be
significant lifetime concerns:

**Temporary lifetime problem**: In `bind::<Brand, _, _, _>(&some_function(), ...)`,
the temporary returned by `some_function()` would be dropped at the end of the
statement, but the borrow would need to live for the duration of the `bind`
call. In Rust, temporaries in function arguments live until the end of the
enclosing statement, so simple cases like `bind(&f(), ...)` would work.

**Nested bind chains**: The bigger issue is nested `bind` calls. In:

```rust
bind(&expr1, move |x: &_| {
    bind(&expr2_using_x, move |y: &_| { ... })
})
```

The closure captures the borrow of `expr1`'s result (via `x`), and inside the
closure, `expr2_using_x` might depend on `x`. If `expr2_using_x` produces a
temporary, its borrow would need to outlive the inner closure, but the
temporary is created inside the outer closure's body and lives only until the
inner `bind` call completes. This should work because the inner `bind` returns
before the temporary is dropped.

**However**, if `bind` stored the borrow for later (e.g., in a lazy context),
the temporary would be dropped before the stored borrow is used. This is the
fundamental tension: by-value passing lets the callee take ownership and store
the value as long as needed, while borrows tie the value's lifetime to the
caller's scope.

**Conclusion**: Generating `&expr` in macros would work for eager evaluation
(Option, Result, Vec) but would fail for lazy/deferred contexts (Lazy,
Trampoline) where the container outlives the call site. This is exactly why the
dispatch-absorbs-borrow pattern (dispatch takes by value, borrows internally)
is the correct approach.

## Part 3: Free Functions

### 3.1 Free Functions in Ref Trait Modules

Each Ref trait module defines free functions that wrap the trait's associated
functions. All of them take the container **by value**.

| Free function       | File                 | Container param           | Signature pattern                            |
| ------------------- | -------------------- | ------------------------- | -------------------------------------------- |
| `ref_join`          | `ref_semimonad.rs`   | `mma` by value            | `fn ref_join(mma: Apply!(...))`              |
| `ref_partition_map` | `ref_filterable.rs`  | `fa` by value             | `fn ref_partition_map(..., fa: Apply!(...))` |
| `ref_partition`     | `ref_filterable.rs`  | `fa` by value             | `fn ref_partition(..., fa: Apply!(...))`     |
| `ref_filter_map`    | `ref_filterable.rs`  | `fa` by value             | `fn ref_filter_map(..., fa: Apply!(...))`    |
| `ref_filter`        | `ref_filterable.rs`  | `fa` by value             | `fn ref_filter(..., fa: Apply!(...))`        |
| `ref_traverse`      | `ref_traversable.rs` | `ta` by value             | `fn ref_traverse(..., ta: Apply!(...))`      |
| `ref_pure`          | `ref_pointed.rs`     | `a: &A` (already borrows) | `fn ref_pure(a: &A)`                         |
| `ref_if_m`          | `ref_monad.rs`       | `cond` by value           | `fn ref_if_m(cond: Apply!(...), ...)`        |
| `ref_unless_m`      | `ref_monad.rs`       | `cond` by value           | `fn ref_unless_m(cond: Apply!(...), ...)`    |

Note: `ref_pure` already takes `&A`, not a container. It is the only Ref free
function that borrows its primary argument, because it constructs a container
from a reference rather than transforming an existing container.

The Ref trait methods themselves (on the traits `RefFunctor`, `RefSemimonad`,
`RefFoldable`, `RefFilterable`, `RefTraversable`, `RefLift`) all take
containers by value. There are no standalone free functions for `ref_map`,
`ref_bind`, `ref_fold_right`, `ref_fold_left`, `ref_fold_map`, or `ref_lift2`,
because the dispatch system provides unified `map`, `bind`, `fold_right`,
`fold_left`, `fold_map`, and `lift2` functions that handle both Val and Ref
paths.

### 3.2 Should Free Functions Change to Borrow or Stay By-Value?

Two viable strategies:

**Strategy A: Free functions take by value, borrow internally (adapter pattern)**

```rust
pub fn ref_filter_map<Brand: RefFilterable, A, B>(
    func: impl Fn(&A) -> Option<B>,
    fa: Apply!(...),   // still by value
) -> Apply!(...) {
    Brand::ref_filter_map(func, &fa)  // borrow here
}
```

Pros:

- No breaking change to any call site.
- Consistent with the dispatch system's approach.
- Works for both eager and lazy consumers.

Cons:

- The caller still gives up ownership even though the trait only needs a borrow.
  This is semantically misleading but practically harmless.

**Strategy B: Free functions change to borrow**

```rust
pub fn ref_filter_map<Brand: RefFilterable, A, B>(
    func: impl Fn(&A) -> Option<B>,
    fa: &Apply!(...),   // borrow
) -> Apply!(...) {
    Brand::ref_filter_map(func, fa)
}
```

Pros:

- Honest about what the operation actually needs.
- Callers can reuse the container after the call.

Cons:

- Breaking change to all call sites.
- Requires careful lifetime management at call sites.
- The dispatch free functions would either need to match (also taking borrows,
  which breaks the dispatch trait's by-value `self` pattern) or diverge from
  the non-dispatch free functions.

**Recommendation**: Strategy A (by-value free functions, internal borrow) is
the better approach. It is non-breaking, aligns with how the dispatch system
works, and avoids lifetime complications at call sites. The dispatch free
functions (`map`, `bind`, `fold_right`, etc.) would not change their signatures
at all; only the dispatch trait impls would add `&` when calling through to
the underlying Ref trait method.

The non-dispatch free functions (`ref_filter_map`, `ref_traverse`, etc.) would
similarly keep their by-value signatures and add `&` internally when calling
the trait method.

## Summary

| Aspect                              | Finding                                                                                                               |
| ----------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Dispatch Ref impls                  | All pass container by value to trait methods. No reuse after call. Mechanical change to `&fa` in impl bodies.         |
| Dispatch free functions             | Take containers by value. Should stay that way (adapter pattern).                                                     |
| m_do! codegen                       | Generates dispatch `bind` calls with `expr` by value. `&_` on closure params triggers Ref dispatch. No change needed. |
| a_do! codegen                       | Generates `map`/`liftN` calls with exprs by value. `&_` on closure params triggers Ref dispatch. No change needed.    |
| Lifetime risk of `&expr` in macros  | Would fail for lazy contexts where container outlives the call site. Adapter pattern avoids this.                     |
| Non-dispatch `ref_*` free functions | Currently take by value. Recommend keeping by value with internal borrow for consistency.                             |
