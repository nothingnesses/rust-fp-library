# Analysis: `lazy.rs`

**File:** `fp-library/src/types/lazy.rs`
**Role:** `Lazy<'a, A, Config>`, memoized lazy evaluation with shared reference-counted caching.

## Design

`Lazy<'a, A, Config: LazyConfig = RcLazyConfig>` is a newtype over `Config::Lazy<'a, A>`, which resolves to:

- `RcLazyConfig`: `Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>`
- `ArcLazyConfig`: `Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'a>>>`

Type aliases: `RcLazy<'a, A> = Lazy<'a, A, RcLazyConfig>`, `ArcLazy<'a, A> = Lazy<'a, A, ArcLazyConfig>`.

Key properties:

- **Memoized**: Computed at most once; result is cached.
- **Shared**: `Clone` produces a new reference-counted handle to the same cell.
- **Evaluate returns `&A`**: This is the fundamental constraint that shapes the entire design.

## Comparison with PureScript

PureScript's `Data.Lazy` is a foreign type where `force` returns an owned `a`. This enables full Functor/Applicative/Monad/Comonad/Traversable instances. Rust's `Lazy` returns `&A`, which prevents all of these standard instances.

| PureScript Instance | Rust Equivalent                          | Status                    |
| ------------------- | ---------------------------------------- | ------------------------- |
| Functor             | RefFunctor / SendRefFunctor              | Partial (reference-based) |
| Applicative         | None                                     | Missing                   |
| Monad               | None                                     | Missing                   |
| Comonad             | None                                     | Missing                   |
| Extend              | None                                     | Missing                   |
| Traversable         | None                                     | Missing                   |
| Foldable            | Foldable (requires `A: Clone`)           | Weaker                    |
| Eq, Ord             | PartialEq, Eq, Ord                       | Full                      |
| Semigroup, Monoid   | Semigroup, Monoid (require `A: Clone`)   | Weaker                    |
| Show                | Display (forces), Debug (does not force) | Partial                   |

## Assessment

### Correct decisions

1. **`LazyConfig` strategy pattern.** Elegantly abstracts over `Rc`/`Arc` without code duplication within `lazy.rs` itself.
2. **Shared memoization via `Rc`/`Arc` cloning.** All clones share the same cache. Well-tested.
3. **`RefFunctor`/`SendRefFunctor` split.** Avoids forcing `Send` bounds on the single-threaded path.
4. **Fix combinators (`rc_lazy_fix`, `arc_lazy_fix`).** Correctly implement knot-tying using `Weak` + `OnceCell`/`OnceLock`.
5. **Cache chain documentation.** Properly warns about memory retention from chained `ref_map` calls.

### Issues

#### 1. No standard Functor/Applicative/Monad

The root cause is `evaluate() -> &A`. PureScript's `force` returns an owned value because the JS runtime transparently reference-counts all values. In Rust, returning owned values from a shared memoized cell would require `Clone`. The library chose not to provide a `Clone`-bounded Functor to avoid deviating from the standard `Functor` trait signature.

**Impact:** High. This is the most significant limitation of the Lazy hierarchy. Users cannot compose `Lazy` values monadically.

**Alternatives considered:** A `Functor` impl that requires `A: Clone` would enable monadic composition at the cost of an extra constraint. This could be a separate trait (e.g., `CloneFunctor`) or a conditional impl. See plan.md for details.

#### 2. `Deferrable` for `ArcLazy` evaluates eagerly

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self { f() }
```

The closure is not `Send`, so `ArcLazy` cannot store it. Evaluating eagerly satisfies the transparency law but defeats the purpose of deferral. The `SendDeferrable::send_defer` method provides true deferral for `ArcLazy`.

**Impact:** Moderate. Well-documented but semantically surprising in generic `Deferrable` code.

#### 3. `Clone` bounds pervade where PureScript has none

`Deferrable`, `Semigroup`, `Monoid`, `Foldable` all require `A: Clone` because `evaluate()` returns `&A` but the operations need owned values. PureScript's equivalents have no such constraint.

**Impact:** Moderate. Restricts which types can be used with these operations.

#### 4. `pure` allocates unnecessarily

`Lazy::pure(a)` wraps the value in `Box::new(move || a)` and a `LazyCell`. For a known value, this adds two unnecessary indirections. The standard library does not expose `LazyCell::from_value`, so this limitation partly comes from std.

**Impact:** Low. The overhead is minimal (one branch on first access).

#### 5. `ArcLazy::new` does not require `Send + Sync` on `A`

`ArcLazy::new` accepts any `A`, but the resulting `Arc<LazyLock<A>>` is only `Send + Sync` when `A: Send + Sync`. This allows constructing `ArcLazy` values that cannot be sent across threads, defeating `ArcLazy`'s primary purpose. The type system catches misuse at the send site, but the API permits constructing unusable values.

**Impact:** Low-moderate. Could add `A: Send + Sync` bounds to `ArcLazy::new` and `ArcLazy::pure` for better API safety.

#### 6. `ref_map` takes `self` by value (move)

`ref_map(self, f)` moves `self` into the new closure. To keep the original, users must `clone()` first. PureScript's `map` does not have this issue because values are freely copyable. This is standard Rust ownership semantics but differs from PureScript's user experience.

**Impact:** Low. Users can `clone()` when needed.

#### 7. Fix combinators have limited utility and lack comprehensive tests

`rc_lazy_fix`: forcing the self-reference during initialization panics (LazyCell reentrant detection).
`arc_lazy_fix`: forcing the self-reference during initialization deadlocks (LazyLock blocks).

Tests only exercise the case where the self-reference is ignored. There are no tests demonstrating actual knot-tying (using the self-reference after initialization completes).

**Impact:** Moderate. The combinators exist but are hard to use correctly and lack tests for their primary use case.

#### 8. `From` conversions between `RcLazy` and `ArcLazy` always force evaluation

Converting between Rc and Arc variants always evaluates and clones. This is necessary because `RcLazy` is `!Send`, but it means the conversion is never lazy.

**Impact:** Low. Inherent and correctly documented.

## Strengths

- Clean strategy pattern via `LazyConfig`.
- Correct shared memoization semantics.
- Well-documented cache chain behavior.
- Comprehensive PartialEq/Eq/Ord/Hash implementations.
- Display forces; Debug does not. A reasonable design choice.
- Thorough property-based test coverage.
