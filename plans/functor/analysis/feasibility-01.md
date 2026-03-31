# Feasibility: Generic Function Parameter on CoyonedaExplicit

## Proposed Change

Replace:

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a>
where F: Kind + 'a {
    fb: <F as Kind>::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}
```

With:

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a>
where F: Kind + 'a {
    fb: <F as Kind>::Of<'a, B>,
    func: Func,
}
```

So that `map` returns `CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C + 'a>` with no
boxing.

## Finding 1: Return-Position `impl Trait` in Methods

Rust stable (since 1.75, RPITIT) supports `impl Trait` in return position of trait
methods and inherent methods. A method like:

```rust
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
) -> CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C + 'a>
```

compiles on stable Rust. The compiler infers a unique, opaque type for the closure
returned by `compose(f, self.func)`. This works for `map`, `new`, `lift`, and any
method that constructs and returns a `CoyonedaExplicit`.

**Verdict: This part is feasible on stable Rust.**

## Finding 2: The `compose` Function

The `compose` function in `functions.rs` is:

```rust
pub fn compose<A, B, C>(
    f: impl Fn(B) -> C,
    g: impl Fn(A) -> B,
) -> impl Fn(A) -> C {
    move |a| f(g(a))
}
```

Its return type is `impl Fn(A) -> C`, which is an opaque type. This is fine as a `Func`
parameter: the caller sees `impl Fn(B) -> C + 'a`, and that satisfies `Fn(B) -> C + 'a`.
Each call to `compose` produces a distinct concrete type (a closure capturing `f` and
`g`), so each `.map()` step produces a `CoyonedaExplicit` with a different concrete
`Func` type, all hidden behind `impl Fn`.

**Verdict: `compose` works seamlessly as the `Func` parameter.**

## Finding 3: Impact on Each Method

### `new`

Current:

```rust
pub fn new(f: impl Fn(B) -> A + 'a, fb: F::Of<'a, B>) -> Self
```

Proposed: The return type becomes `CoyonedaExplicit<'a, F, B, A, impl Fn(B) -> A + 'a>`.
The caller passes a concrete function; the struct stores it directly. No issues.

### `lift`

Current:

```rust
pub fn lift(fa: F::Of<'a, A>) -> Self  // where Self = CoyonedaExplicit<'a, F, A, A>
```

This stores `Box::new(identity)`. In the proposed design, the return type becomes
`CoyonedaExplicit<'a, F, A, A, fn(A) -> A>` (since `identity` is a named function with
type `fn(A) -> A`). This is fine. Alternatively, the return type can be written as
`CoyonedaExplicit<'a, F, A, A, impl Fn(A) -> A + 'a>`.

**One subtlety:** `lift` currently returns `Self` where
`Self = CoyonedaExplicit<'a, F, A, A>`. With the extra parameter, `Self` would be
`CoyonedaExplicit<'a, F, A, A, Func>`, but `lift` cannot return `Self` because its
`Func` is always `fn(A) -> A`, not the caller's `Func`. The impl block for `lift` must
be on the specific type `CoyonedaExplicit<'a, F, A, A, fn(A) -> A>`, or `lift` must
return an explicit type rather than `Self`. This is workable but changes the API surface.

### `map`

Current:

```rust
pub fn map<C: 'a>(self, f: impl Fn(A) -> C + 'a) -> CoyonedaExplicit<'a, F, B, C>
```

Proposed:

```rust
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
) -> CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C + 'a>
```

This works. Each call to `map` produces a new type wrapping the composition. Chaining
`.map(f1).map(f2).map(f3)` produces a nested closure type like
`compose(f3, compose(f2, compose(f1, identity)))`, which is a deeply nested but
monomorphized, zero-cost type.

### `lower`

Current:

```rust
pub fn lower(self) -> F::Of<'a, A> where F: Functor
```

This calls `F::map(self.func, self.fb)`. The signature of `Functor::map` takes
`impl Fn(B) -> A`. Since `Func: Fn(B) -> A`, `self.func` satisfies this bound. No
issues.

### `hoist`

Current:

```rust
pub fn hoist<G>(self, nat: impl NaturalTransformation<F, G>)
    -> CoyonedaExplicit<'a, G, B, A>
```

Proposed: returns `CoyonedaExplicit<'a, G, B, A, Func>` where `Func` is the same
`Func` from `self`. This works because `hoist` just transforms `fb` and passes
`self.func` through unchanged. The return type can be written as
`CoyonedaExplicit<'a, G, B, A, Func>` using the impl block's `Func` parameter directly.
No need for `impl Fn` here; the concrete `Func` is forwarded.

**This is actually cleaner than the boxed version**, since the function type is preserved
exactly.

### `fold_map`

Current:

```rust
pub fn fold_map<FnBrand, M>(self, func: impl Fn(A) -> M + 'a) -> M
```

This calls `F::fold_map::<FnBrand, B, M>(compose(func, self.func), self.fb)`. The
`compose` call takes `self.func` (which is `Func: Fn(B) -> A`) and `func` (which is
`impl Fn(A) -> M`). The result is `impl Fn(B) -> M`. `Foldable::fold_map` takes
`impl Fn(B) -> M`. No issues.

### `into_coyoneda`

Current:

```rust
pub fn into_coyoneda(self) -> Coyoneda<'a, F, A>
```

This calls `Coyoneda::new(self.func, self.fb)`. `Coyoneda::new` takes
`impl Fn(B) -> A + 'a`. Since `Func: Fn(B) -> A + 'a`, this works.

### `apply`

Current:

```rust
pub fn apply<FnBrand, Bf, C>(
    ff: CoyonedaExplicit<'a, F, Bf, CloneableFn::Of<'a, A, C>>,
    fa: Self,
) -> CoyonedaExplicit<'a, F, C, C>
```

This calls `ff.lower()` and `fa.lower()`, then `F::apply(...)`, then
`CoyonedaExplicit::lift(...)`. With the generic `Func` parameter:

- `ff` needs its own `Func` parameter: `FuncF: Fn(Bf) -> CloneableFn::Of<'a, A, C>`.
- `fa` has `Func` from the impl block.
- The result is `CoyonedaExplicit::lift(...)` which returns with `fn(C) -> C` as the
  function parameter.

The signature becomes unwieldy but is feasible:

```rust
pub fn apply<FnBrand, Bf, C, FuncF>(
    ff: CoyonedaExplicit<'a, F, Bf, CloneableFn::Of<'a, A, C>, FuncF>,
    fa: Self,
) -> CoyonedaExplicit<'a, F, C, C, fn(C) -> C>
where
    FuncF: Fn(Bf) -> CloneableFn::Of<'a, A, C> + 'a,
    // ...
```

Callers must be able to name or infer `FuncF`. Since it is typically inferred, this
works in practice.

### `bind`

Current:

```rust
pub fn bind<C>(
    self,
    f: impl Fn(A) -> CoyonedaExplicit<'a, F, C, C> + 'a,
) -> CoyonedaExplicit<'a, F, C, C>
```

With the extra parameter, the closure `f` returns
`CoyonedaExplicit<'a, F, C, C, fn(C) -> C>` (since `bind` lifts internally). But the
user-facing closure also returns a `CoyonedaExplicit`, and the user might construct it
via `lift` (giving `fn(C) -> C`) or via `new` (giving `impl Fn(C) -> C`). The signature
of `f` must accept any `Func2`:

```rust
pub fn bind<C>(
    self,
    f: impl Fn(A) -> CoyonedaExplicit<'a, F, C, C, impl Fn(C) -> C + 'a> + 'a,
) -> CoyonedaExplicit<'a, F, C, C, fn(C) -> C>
```

**Problem:** `impl Trait` in argument position inside a closure return type is not
supported in Rust. The closure's return type must be a single concrete type. If the user
returns `CoyonedaExplicit::lift(...)` from the closure, that has type
`CoyonedaExplicit<..., fn(C) -> C>`. If they return
`CoyonedaExplicit::new(some_closure, ...)`, that has a different type. The closure `f`
can only return one concrete type, so the user would be forced to always use the same
constructor, or call `.boxed()` to erase the function type.

Alternatively, `bind` can require the returned `CoyonedaExplicit` to use
`Box<dyn Fn(C) -> C>` (the erased form), which defeats the purpose.

**This is a significant ergonomic problem.**

### `pure`

Current:

```rust
pub fn pure(a: A) -> Self  // in impl CoyonedaExplicit<'a, F, A, A>
```

Delegates to `Self::lift(F::pure(a))`. Same situation as `lift`: must return a specific
`Func` type (`fn(A) -> A`), not `Self` in the general sense.

## Finding 4: The Loop Problem

The `many_chained_maps` test does:

```rust
let mut coyo = CoyonedaExplicit::<VecBrand, _, _>::lift(vec![0i64]);
for _ in 0 .. 100 {
    coyo = coyo.map(|x| x + 1);
}
```

This reassigns `coyo` in a loop. Each iteration, `map` returns a different concrete
type (the closure nests deeper). But `coyo` must have a single type for the loop
variable. With `Box<dyn Fn>`, every iteration produces the same type. With a generic
`Func`, each iteration produces a different type, so **this loop does not compile**.

The workaround is to call `.boxed()` before reassignment, but that re-introduces the
heap allocation the change was meant to eliminate.

**This is a fundamental limitation.** Linear chains like `.map(f1).map(f2).map(f3)` work
because each step is a different variable with a different type. But any code that
accumulates maps in a loop (a common pattern for dynamic map counts) cannot use the
unboxed form.

## Finding 5: The `.boxed()` Escape Hatch

A `.boxed()` method can erase the `Func` parameter:

```rust
impl<'a, F, B, A, Func> CoyonedaExplicit<'a, F, B, A, Func>
where
    F: Kind + 'a,
    Func: Fn(B) -> A + 'a,
{
    pub fn boxed(self) -> CoyonedaExplicit<'a, F, B, A, Box<dyn Fn(B) -> A + 'a>> {
        CoyonedaExplicit {
            fb: self.fb,
            func: Box::new(self.func),
        }
    }
}
```

This works and gives users an explicit opt-in to dynamic dispatch when needed (loops,
heterogeneous collections, struct fields). The tradeoff is clear: `.boxed()` allocates
once; without it, everything is zero-cost.

## Finding 6: PhantomData and Struct Size

With `Box<dyn Fn>`, the struct is always two words (fat pointer) for `func`, regardless
of how many maps have been composed. With a generic `Func`, the struct size grows with
each `compose` nesting, since the closure captures the previous closure. For a chain of
N maps, the function field is N closures deep, each capturing the previous one. This is
the same memory, just on the stack instead of the heap, but it means the struct size is
not fixed, which could matter for very deep chains.

## Summary

| Aspect                                  | Feasible?           | Notes                                                                           |
| --------------------------------------- | ------------------- | ------------------------------------------------------------------------------- |
| Return-position `impl Trait` in methods | Yes                 | Stable since Rust 1.75.                                                         |
| `compose` as generic Func               | Yes                 | Returns `impl Fn`, satisfies bounds.                                            |
| `new`                                   | Yes                 | Stores the caller's function directly.                                          |
| `lift`                                  | Yes                 | Returns `fn(A) -> A` (named function type).                                     |
| `map` (linear chain)                    | Yes                 | Each step is a new type; zero-cost.                                             |
| `map` (in a loop)                       | No                  | Loop variable needs a single type. Requires `.boxed()`.                         |
| `lower`                                 | Yes                 | `Functor::map` takes `impl Fn`.                                                 |
| `hoist`                                 | Yes                 | Forwards `Func` unchanged.                                                      |
| `fold_map`                              | Yes                 | Composes via `compose`, passes to `fold_map`.                                   |
| `into_coyoneda`                         | Yes                 | `Coyoneda::new` takes `impl Fn`.                                                |
| `apply`                                 | Yes (verbose)       | Extra type parameter for `ff`'s function.                                       |
| `bind`                                  | Problematic         | Closure return type must be uniform; forces `.boxed()` or specific constructor. |
| `pure`                                  | Yes                 | Returns with `fn(A) -> A`.                                                      |
| `.boxed()` escape hatch                 | Yes                 | Erases `Func` back to `Box<dyn Fn>` for loops/storage.                          |
| Collections of CoyonedaExplicit         | No (without boxing) | Each has a different `Func` type.                                               |

## Recommendation

The generic `Func` parameter is feasible for the primary use case: linear `.map()` chains
that terminate in `.lower()`. This is the pattern shown in the module documentation and
represents the core value proposition of `CoyonedaExplicit`.

However, two patterns break:

1. **Loops that accumulate maps** (the `many_chained_maps` test). These require `.boxed()`
   to compile, which re-introduces allocation.
2. **`bind` ergonomics.** The closure's return type must be uniform, forcing users to
   pick a single constructor or use `.boxed()`.

A practical approach would be:

- Add the `Func` type parameter with a default: `Func = Box<dyn Fn(B) -> A + 'a>`.
  This preserves backward compatibility for code that does not care about the function
  type.
- `lift` and `new` return the unboxed form.
- `map` returns the unboxed composed form.
- Provide `.boxed()` for when dynamic dispatch is needed.
- `bind` and `apply` return the boxed form (since they reset the pipeline via `lift`
  anyway, one allocation is acceptable).

This way, the zero-cost path is available for the common linear-chain case, while the
boxed path remains available (and is the default) for everything else.
