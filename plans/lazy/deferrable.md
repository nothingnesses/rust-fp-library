# Analysis: `deferrable.rs`

**File:** `fp-library/src/classes/deferrable.rs`
**Role:** Defines the `Deferrable<'a>` trait and its free function `defer`.

## Design

`Deferrable<'a>` is modeled after PureScript's `Control.Lazy` class:

```purescript
class Lazy l where
  defer :: (Unit -> l) -> l
```

The Rust version uses `impl FnOnce() -> Self + 'a` instead of `Unit -> l`, which is the idiomatic translation. The lifetime parameter `'a` enables the thunk to capture borrows, which PureScript does not need (GC handles lifetimes).

## Assessment

### Correct decisions

1. **Value-level trait, not brand-level.** `Deferrable` is implemented by concrete types (`Thunk<'a, A>`, `RcLazy<'a, A>`) rather than brands. This is correct because deferral is a construction operation on values, not a structural transformation on type constructors.

2. **No generic `fix`.** The documentation correctly explains why PureScript's `fix :: Lazy l => (l -> l) -> l` cannot be generalized in Rust: self-referential construction requires shared ownership (`Rc`/`Arc`) and interior mutability, properties specific to `Lazy`, not all `Deferrable` types. Concrete `rc_lazy_fix` / `arc_lazy_fix` functions are provided instead.

3. **Transparency law.** The single stated law ("the value produced by `defer(|| x)` is identical to `x`") is minimal and correct. It does not constrain evaluation timing, which is appropriate since some implementations evaluate eagerly.

### Issues

#### 1. `ArcLazy` and `SendThunk` implement `Deferrable` with eager evaluation

The trait's name and documentation suggest deferred computation, but `ArcLazy::defer` calls `f()` immediately, and `SendThunk::defer` does the same. The warning in the doc comment acknowledges this, but it remains a semantic mismatch: generic code written against `Deferrable` cannot rely on laziness.

**Impact:** Moderate. Users writing generic `Deferrable`-bounded code may assume laziness and get eager evaluation for some types.

#### 2. The transparency law is too weak to be useful

The law says `defer(|| x)` is observationally equivalent to `x`. This is trivially satisfied by `fn defer(f) { f() }` (eager evaluation), which provides no laziness guarantee at all. A stronger law (e.g., "the thunk is not called during `defer`") would be more useful but would exclude the eager implementations.

**Impact:** Low. The law correctly reflects what the trait guarantees, but it means the trait provides almost no semantic guarantees beyond type-level plumbing.

#### 3. `Deferrable` is a supertrait of `SendDeferrable`, creating a forced relationship

Since `SendDeferrable: Deferrable`, every `SendDeferrable` type must also implement `Deferrable`. For types like `SendThunk` and `ArcLazy`, this forces an eager `Deferrable` implementation because the trait's closure is not `Send`. If `SendDeferrable` were independent of `Deferrable`, these types would not need the misleading eager impl.

**Impact:** Moderate. The supertrait relationship is the root cause of the "eager Deferrable" problem.

#### 4. Tests only cover `Thunk`

The QuickCheck tests verify transparency and nesting for `Thunk` only. No tests for `RcLazy`, `ArcLazy`, `SendThunk`, `TryThunk`, etc. The `send_deferrable.rs` tests partially fill this gap but `Deferrable` itself is only tested with `Thunk`.

**Impact:** Low. Other files test their own `Deferrable` impls, but the trait module itself should arguably test more implementors.

## Comparison with PureScript

PureScript's `Control.Lazy` has a `fix` combinator and instances for `(a -> b)` and `Unit`. The Rust version omits `fix` (correctly, for the reasons documented) and does not need instances for function types or `()`. The PureScript class takes `Unit -> l` (a thunk), while Rust uses `FnOnce() -> Self`, which is equivalent.
