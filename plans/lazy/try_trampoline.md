# Analysis: `try_trampoline.rs`

**File:** `fp-library/src/types/try_trampoline.rs`
**Role:** `TryTrampoline<A, E>`, fallible stack-safe recursion.

## Design

`TryTrampoline<A: 'static, E: 'static>` is a newtype over `Trampoline<Result<A, E>>`. The full nesting is:

```
TryTrampoline<A, E> -> Trampoline<Result<A, E>> -> Free<ThunkBrand, Result<A, E>>
```

## Assessment

### Correct decisions

1. **Newtype over `Trampoline<Result<A, E>>`.** Consistent with `TryThunk` over `Thunk<Result<A, E>>`.
2. **Error short-circuiting in `bind`.** The bind implementation correctly short-circuits on `Err`:
   ```rust
   self.0.bind(|result| match result {
       Ok(a) => f(a).0,
       Err(e) => Trampoline::pure(Err(e)),
   })
   ```
3. **Stack-safe `tail_rec_m`.** Uses `Trampoline::defer` internally for recursion, maintaining stack safety.

### Issues

#### 1. Error path in `bind` allocates unnecessarily

When `bind` encounters an `Err(e)`, it wraps it in `Trampoline::pure(Err(e))`, which allocates a `Box<dyn Any>` for the `Err(e)`. In a chain like `err(e).bind(f1).bind(f2)...bind(fn)`, each `bind` pattern-matches on `Ok`/`Err` and re-wraps the error, performing `n` allocations and pattern matches. A first-class error channel in `Free` would allow immediate short-circuiting.

**Impact:** Low-moderate. Each step is O(1), but the constant factor accumulates. In practice, acceptable for most use cases.

#### 2. Code duplication with `trampoline.rs`

Many methods are duplicated: `new`, `pure`, `defer`, `bind`, `map`, `map_err`, `bimap`, `evaluate`, conversions, etc. Less severe than the `TryLazy`/`Lazy` duplication because `TryTrampoline` delegates most logic to `Trampoline`.

**Impact:** Low-moderate.

#### 3. `map` goes through `bind` internally

`self.0.map(|result| result.map(func))` calls `Free::map` which internally calls `Free::bind`, adding a CatList continuation. A simple `map` on `TryTrampoline` thus adds overhead compared to a hypothetical optimized path. However, the cost is O(1) per operation.

**Impact:** Low.

#### 4. No HKT brand (same as `Trampoline`)

Cannot participate in generic HKT abstractions.

**Impact:** Same as `Trampoline`.

#### 5. `'static` required on both `A` and `E`

Both type parameters must be `'static` due to `Box<dyn Any>`. This prevents fallible computations with borrowed error or success types.

**Impact:** Moderate. Same root cause as `Trampoline`.

## Strengths

- Stack-safe fallible recursion with O(1) bind.
- Complete error-handling API: `map`, `map_err`, `bimap`, `bind`, `catch`, `catch_with`, `catch_unwind`.
- Consistent design with the rest of the Try\* family.
- Well-tested with property-based tests.
