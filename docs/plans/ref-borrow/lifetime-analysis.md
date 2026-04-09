# Lifetime analysis: borrowing containers in Ref trait methods

## Overview

This document analyzes what happens when every Ref trait method changes its
container parameter from `fa: Self::Of<'a, A>` (consuming) to
`fa: &Self::Of<'a, A>` (borrowing). The borrow introduces an implicit lifetime
`'b` alongside the existing HKT lifetime `'a`.

## 1. Lifetime elision

Consider the proposed signature:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl Fn(&A) -> B + 'a,
    fa: &Self::Of<'a, A>,
) -> Self::Of<'a, B>;
```

The borrow lifetime `'b` on `fa` is fully elided. Rust's elision rules handle
this straightforwardly: `&Self::Of<'a, A>` is sugar for
`&'_ Self::Of<'a, A>`, and because `'b` does not appear in the return type, the
compiler never needs to name it.

**No elision failures are expected.** The borrow lifetime only appears in
input position and is absent from the return type. The compiler can always
assign a fresh anonymous lifetime to it. This holds for all Ref trait methods
examined below, because none of them return a type that borrows `fa` (see
section 2).

One subtlety: if a trait method were to return `&B` or `&Self::Of<'a, B>`, the
compiler could not decide whether the returned reference's lifetime should match
`func`'s borrow, `fa`'s borrow, or `'a`. But no Ref trait method has this
shape; they all return owned `Self::Of<'a, B>`.

## 2. Returned references

No Ref trait method returns a reference to the input container. Every method
returns an owned `Self::Of<'a, ...>` or a plain value type (`M`, `B`, etc.):

| Method             | Return type                                     | Borrows `fa`? |
| ------------------ | ----------------------------------------------- | ------------- |
| `ref_map`          | `Self::Of<'a, B>`                               | No            |
| `ref_fold_map`     | `M`                                             | No            |
| `ref_fold_right`   | `B`                                             | No            |
| `ref_fold_left`    | `B`                                             | No            |
| `ref_lift2`        | `Self::Of<'a, C>`                               | No            |
| `ref_apply`        | `Self::Of<'a, B>`                               | No            |
| `ref_bind`         | `Self::Of<'a, B>`                               | No            |
| `ref_traverse`     | `F::Of<'a, Self::Of<'a, B>>`                    | No            |
| `ref_wilt`         | `M::Of<'a, (Self::Of<'a, E>, Self::Of<'a, O>)>` | No            |
| `ref_wither`       | `M::Of<'a, Self::Of<'a, B>>`                    | No            |
| `ref_apply_first`  | `Self::Of<'a, A>`                               | No            |
| `ref_apply_second` | `Self::Of<'a, B>`                               | No            |

Because no return type references `fa`, the borrow lifetime `'b` never needs
to appear in any return position. This is the key property that makes the
change safe from an elision and lifetime-propagation perspective.

## 3. GAT interaction

The `Kind` trait uses GATs with an outlives bound:

```rust
type Of<'a, A: 'a>: 'a;
```

When we write `&'b Self::Of<'a, A>`, we have a reference whose lifetime `'b`
may be shorter than `'a`. This is entirely fine in Rust. The GAT bound
`Of<'a, A>: 'a` means the _value_ `Of<'a, A>` is valid for `'a`, which is a
prerequisite for creating a `&'b` reference (the value must live at least as
long as the borrow). Rust enforces `'a: 'b` implicitly at the call site, which
is always satisfiable because the caller owns the value for `'a` and borrows it
for some shorter `'b`.

No interaction issues are expected. The GAT bound constrains the associated
type's validity, not how it may be borrowed. A `&'b T where T: 'a` is a
standard Rust pattern.

One potential concern: if an implementor tried to store `fa` (the borrow) inside
the returned `Self::Of<'a, B>`, they would need `'b: 'a`, but the return type
is `Self::Of<'a, B>: 'a`, which outlives the borrow. This would be a compile
error, which is the _desired_ behavior: the returned container should not hold
a borrow of the input.

## 4. Closure capture for Lazy implementations

The proposed pattern:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl Fn(&A) -> B + 'a,
    fa: &Lazy<'a, A>,
) -> Lazy<'a, B> {
    let fa = fa.clone(); // Rc/Arc clone, cheap
    Lazy::new(move || func(fa.evaluate()))
}
```

This works correctly. The `fa.clone()` produces an owned `Lazy<'a, A>` (via
`Rc`/`Arc` cloning) whose lifetime is independent of `'b`. The closure captures
the owned clone, so its lifetime is `'a` (matching the `+ 'a` bound on `func`
and the return type). The borrow `&Lazy<'a, A>` is released immediately after
the clone, and the closure never touches the original borrow.

**Key insight:** `Lazy` is `Clone` because it wraps an `Rc`/`Arc`, so the clone
is O(1) and does not copy the underlying data. This is the same mechanism that
already keeps Lazy chains alive today.

For the current code, the `Lazy` inherent method `ref_map` takes `self` (owned)
and moves it into the closure:

```rust
pub fn ref_map<B: 'a>(self, f: impl FnOnce(&A) -> B + 'a) -> Lazy<'a, B, RcLazyConfig> {
    RcLazy::new(move || f(self.evaluate()))
}
```

The trait impl delegates to this: `fa.ref_map(f)`. Under the proposed change,
the trait would clone first, then call the inherent method:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl Fn(&A) -> B + 'a,
    fa: &Lazy<'a, A, RcLazyConfig>,
) -> Lazy<'a, B, RcLazyConfig> {
    fa.clone().ref_map(func)
}
```

No lifetime issues arise. The clone is owned and lives for `'a`.

For `ref_bind`, the same pattern applies:

```rust
fn ref_bind<'a, A: 'a, B: 'a>(
    fa: &Lazy<'a, A, RcLazyConfig>,
    f: impl Fn(&A) -> Lazy<'a, B, RcLazyConfig> + 'a,
) -> Lazy<'a, B, RcLazyConfig> {
    let fa = fa.clone();
    f(fa.evaluate())
}
```

The `fa.evaluate()` returns `&A` with a lifetime tied to the cloned `Lazy`, not
to the original borrow. No issues.

For `ref_fold_map` on Lazy:

```rust
fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
    func: impl Fn(&A) -> M + 'a,
    fa: &Lazy<'a, A, RcLazyConfig>,
) -> M { func(fa.evaluate()) }
```

Here `fa.evaluate()` returns `&A` with lifetime tied to the borrow of `fa`.
This works: `func` takes `&A` and returns owned `M`. The borrow of `fa` lasts
for the duration of `func(fa.evaluate())`, which is fine because `M` is owned
and does not borrow from `fa`.

## 5. Multiple borrows

Methods like `ref_lift2` take two containers:

```rust
fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
    func: impl Fn(&A, &B) -> C + 'a,
    fa: &Self::Of<'a, A>,
    fb: &Self::Of<'a, B>,
) -> Self::Of<'a, C>;
```

Both borrows are immutable (`&`), so there is no aliasing conflict. Rust freely
allows multiple shared borrows. Both have the same HKT lifetime `'a` but
independent borrow lifetimes (both elided to anonymous lifetimes).

For `Vec`'s implementation, the current code:

```rust
fa.iter().flat_map(|a| fb.iter().map(move |b| func(a, b))).collect()
```

This would work directly with `fa: &Vec<A>, fb: &Vec<B>` because `iter()`
borrows the collection anyway.

For `Option`'s implementation:

```rust
match (fa.as_ref(), fb.as_ref()) {
    (Some(a), Some(b)) => Some(func(a, b)),
    _ => None,
}
```

With borrowed containers, `fa` is already `&Option<A>`, so `fa.as_ref()` returns
`Option<&A>` (calling `Option::as_ref` on the reference). This works
identically. Actually, `&Option<A>` has an inherent `as_ref()` method that
returns `Option<&A>`, so the code remains unchanged.

For `Lazy`'s implementation:

```rust
fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
    func: impl Fn(&A, &B) -> C + 'a,
    fa: &Lazy<'a, A, RcLazyConfig>,
    fb: &Lazy<'a, B, RcLazyConfig>,
) -> Lazy<'a, C, RcLazyConfig> {
    let fa = fa.clone();
    let fb = fb.clone();
    RcLazy::new(move || func(fa.evaluate(), fb.evaluate()))
}
```

Both clones are independent owned values; no issues.

For `ref_apply_first` and `ref_apply_second`, both `fa` and `fb` would be
borrowed. Their default implementations delegate to `ref_lift2`, so they
simply pass both borrows through. No complications.

## 6. Default implementations call-chain analysis

### `RefFoldable::ref_fold_right` (default, derives from `ref_fold_map`)

```rust
fn ref_fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
    func: impl Fn(&A, B) -> B + 'a,
    initial: B,
    fa: Apply!(... ::Of<'a, A>),
) -> B
```

The default body passes `fa` to `Self::ref_fold_map(...)`. If `fa` changes to
`&Self::Of<'a, A>`, the call to `ref_fold_map` must also accept a borrow.
Since both methods are on the same trait and would be updated together, this is
consistent. `fa` is only passed once (to `ref_fold_map`), so there is no
move-after-borrow issue.

Call chain: `ref_fold_right` -> `ref_fold_map`. Single pass of `fa`, no issues.

### `ref_fold_left` (default, derives from `ref_fold_right`)

Call chain: `ref_fold_left` -> `ref_fold_right` -> `ref_fold_map`. Single pass
of `fa` through the chain. Each method would accept `&Self::Of<'a, A>` and
forward the borrow. No issues.

### `RefSemiapplicative::ref_apply` (no default implementation)

`ref_apply` has no default implementation; it is a required method. Implementors
provide their own body. No call-chain concern.

However, if a future default were added in terms of `ref_lift2` (as
`Semiapplicative::apply` derives from `lift2` in some designs), both `ff` and
`fa` would be borrowed, which works per section 5.

### `RefApplyFirst::ref_apply_first` (default, derives from `ref_lift2`)

```rust
fn ref_apply_first<'a, A: Clone + 'a, B: 'a>(
    fa: ..., fb: ...,
) -> ... {
    Self::ref_lift2(|a: &A, _: &B| a.clone(), fa, fb)
}
```

If both `fa` and `fb` become borrows, they are simply forwarded to `ref_lift2`
which would also accept borrows. This is a direct pass-through. No issues.

### `RefApplySecond::ref_apply_second` (default, derives from `ref_lift2`)

Same analysis as `ref_apply_first`. Direct pass-through to `ref_lift2`.

### `RefTraversable::ref_traverse` (required, no default)

`ref_traverse` is a required method with no default implementation. Implementors
provide their own. For `Vec`:

```rust
Self::traverse::<A, B, F>(move |a: A| func(&a), ta)
```

This delegates to `Traversable::traverse`, which consumes `ta`. If `ta` becomes
`&Vec<A>`, the implementation would need to change to iterate by reference
rather than delegating to the consuming `traverse`. This is straightforward:

```rust
// Direct implementation instead of delegating to traverse
ta.iter().fold(F::pure(Vec::new()), |acc, a| {
    F::lift2(|mut v: Vec<B>, b: B| { v.push(b); v }, acc, func(a))
})
```

Or more simply, clone the vec first: `ta.clone()` then delegate. The clone
approach is simpler but less efficient.

**This is a design choice, not a lifetime problem.** The key point is that the
delegation pattern `traverse(|a| func(&a), ta)` consumes `ta` and would fail
if `ta` is borrowed. Implementors must either clone or rewrite to use iterators.

For `Option`:

```rust
Self::traverse::<A, B, F>(move |a: A| func(&a), ta)
```

Same issue: delegates to consuming `traverse`. Would need `ta.clone()` or a
direct implementation via `match ta { Some(a) => ..., None => ... }`.

### `RefWitherable::ref_wilt` (default, derives from `ref_traverse`)

```rust
fn ref_wilt<...>(func, ta) -> ... {
    M::map(
        |res| Self::separate::<E, O>(res),
        Self::ref_traverse::<FnBrand, A, Result<O, E>, M>(func, ta),
    )
}
```

`ta` is passed directly to `ref_traverse`. If both methods accept borrows,
the borrow flows through. `M::map` receives the _result_ of `ref_traverse`
(an owned value), not `ta`. No issues.

### `RefWitherable::ref_wither` (default, derives from `ref_traverse`)

Same structure as `ref_wilt`:

```rust
fn ref_wither<...>(func, ta) -> ... {
    M::map(
        |opt| Self::compact(opt),
        Self::ref_traverse::<FnBrand, A, Option<B>, M>(func, ta),
    )
}
```

`ta` flows to `ref_traverse`. Same analysis as `ref_wilt`. No issues.

## 7. `Apply!` macro with reference types

The `Apply!` macro expands `Apply!(<B as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>)` into `<B as Kind_xxx>::Of<'a, A>`. It performs purely syntactic
expansion, producing a type-level projection.

If parameters change from `Apply!(...)` to `&Apply!(...)`, the `&` sits
_outside_ the macro invocation. The macro itself is never asked to produce a
reference type. For example:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl Fn(&A) -> B + 'a,
    fa: &Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
```

This expands to:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl Fn(&A) -> B + 'a,
    fa: &<Self as Kind_cdc7cd43dac7585f>::Of<'a, A>,
) -> <Self as Kind_cdc7cd43dac7585f>::Of<'a, B>;
```

The `&` before the macro invocation is standard Rust syntax. The macro does not
need to "handle" reference types because it never sees them; `&Apply!(...)` is
parsed as `& <macro-expansion>`.

**No issues with the `Apply!` macro.** The reference is syntactically outside
the macro boundary.

## Summary of findings

| Concern                  | Risk | Notes                                                            |
| ------------------------ | ---- | ---------------------------------------------------------------- |
| Lifetime elision         | None | `'b` only in input position, never in return type.               |
| Returned references      | None | All methods return owned types.                                  |
| GAT interaction          | None | `&'b T where T: 'a` is standard; `'a: 'b` enforced at call site. |
| Closure capture (Lazy)   | None | Clone before capture; owned clone is independent of borrow.      |
| Multiple borrows         | None | All borrows are shared (`&`); no aliasing conflicts.             |
| Default impl call chains | None | Borrows flow through single-pass chains; no re-borrow issues.    |
| `Apply!` macro           | None | `&` sits outside the macro; macro expansion is unaffected.       |

The main work items are not lifetime-related but implementation-related:

1. **Delegation to consuming methods must be replaced.** Several `ref_traverse`
   and `ref_fold_map` implementations delegate to their consuming counterparts
   (e.g., `Self::traverse(move \|a\| func(&a), ta)`). When `ta` is borrowed,
   these implementations must either clone `ta` first or be rewritten to work
   with iterators/references directly.

2. **Lazy implementations must clone before capturing.** Every Lazy trait method
   that currently moves `fa` into a closure must clone the `Rc`/`Arc` wrapper
   first. This is O(1) per clone and semantically identical to the current
   behavior.

3. **All Ref trait methods must be updated in lockstep.** Because default
   implementations call other trait methods (e.g., `ref_fold_right` calls
   `ref_fold_map`), all methods on a given trait must change to borrowed
   parameters simultaneously.

No lifetime soundness issues were identified. The change is
mechanically safe from a lifetime perspective.
