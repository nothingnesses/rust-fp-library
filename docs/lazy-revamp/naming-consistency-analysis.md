# Naming Consistency Analysis

This document analyzes naming inconsistencies across deferred computation types and proposes alternatives for consistency.

## Scope

Files analyzed:

- `fp-library/src/types/free.rs`
- `fp-library/src/types/lazy.rs`
- `fp-library/src/types/thunk.rs`
- `fp-library/src/types/trampoline.rs`
- `fp-library/src/classes/defer.rs`

---

## Current State: Inconsistencies

### 1. Execution/Forcing Methods

| Type             | Method        | Signature        | Returns          |
| ---------------- | ------------- | ---------------- | ---------------- |
| `Lazy`           | `get()`       | `&self`          | `&A`             |
| `Thunk`          | `run()`       | `self`           | `A`              |
| `Trampoline`     | `run()`       | `self`           | `A`              |
| `Free`           | `run()`       | `self`           | `A`              |
| `LazyConfig`     | `force()`     | `&Self::Lazy`    | `&A`             |
| `LazyConfig`     | `force_try()` | `&Self::TryLazy` | `Result<&A, &E>` |
| `Runnable` trait | `run()`       | `fa`             | `A`              |

**Issue**: Mixed naming (`get`, `run`, `force`) for the same conceptual operation.

### 2. Constructor Methods: From Value

| Type         | Method    | Signature            |
| ------------ | --------- | -------------------- |
| `Lazy`       | _(none)_  | —                    |
| `Thunk`      | `pure(a)` | `A -> Thunk<A>`      |
| `Trampoline` | `pure(a)` | `A -> Trampoline<A>` |
| `Free`       | `pure(a)` | `A -> Free<F, A>`    |

**Issue**: `Lazy` lacks a constructor from a pre-computed value.

### 3. Constructor Methods: From Thunk/Closure

| Type         | Method       | Signature          | Behavior              |
| ------------ | ------------ | ------------------ | --------------------- |
| `Lazy`       | `new(f)`     | `F: FnOnce() -> A` | Memoizes result       |
| `Thunk`      | `new(f)`     | `F: FnOnce() -> A` | Re-evaluates each run |
| `Trampoline` | `new(f)`     | `F: FnOnce() -> A` | Re-evaluates each run |
| `Free`       | _(via roll)_ | —                  | —                     |

**Issue**: Same method name `new` has different semantics (memoizing vs non-memoizing).

### 4. Constructor Methods: Deferred Self / Flatten

| Type         | Method     | Signature                      |
| ------------ | ---------- | ------------------------------ |
| `Lazy`       | _(none)_   | —                              |
| `Thunk`      | `defer(f)` | `F: FnOnce() -> Thunk<A>`      |
| `Trampoline` | `defer(f)` | `F: FnOnce() -> Trampoline<A>` |
| `Free`       | `roll(fa)` | `F<Free<F, A>> -> Free<F, A>`  |

**Issue**: `roll` differs from the `defer` naming pattern; `Lazy` lacks this capability.

### 5. Defer Trait vs Instance Methods

| Location            | Name           | Signature                                    | Purpose                              |
| ------------------- | -------------- | -------------------------------------------- | ------------------------------------ |
| `classes/defer.rs`  | `Defer::defer` | `CloneableFn<(), Self> -> Self`              | Lazy construction from wrapped thunk |
| `Thunk` method      | `defer`        | `FnOnce() -> Thunk<A> -> Thunk<A>`           | Flatten deferred construction        |
| `Trampoline` method | `defer`        | `FnOnce() -> Trampoline<A> -> Trampoline<A>` | Flatten deferred construction        |

**Issue**: Name collision between trait and methods with different semantics.

### 6. Free Monad Internal Variants

| Current Name | Standard FP Name   | Description                      |
| ------------ | ------------------ | -------------------------------- |
| `Pure`       | `Pure`             | ✓ Consistent                     |
| `Roll`       | `Suspend` / `Lift` | Suspended computation in functor |
| `Bind`       | `FlatMap` / `Bind` | Continuation chain               |

**Issue**: `Roll` is non-standard; most FP libraries use `Suspend`.

### 7. Brand/HKT Naming

| Type         | Brand                               |
| ------------ | ----------------------------------- |
| `Thunk`      | `ThunkBrand`                        |
| `Lazy`       | `LazyBrand<Config>`                 |
| `Free`       | _(no brand - cannot implement HKT)_ |
| `Trampoline` | _(no brand - uses Free internally)_ |

**Note**: Brand naming is consistent where applicable.

---

## Proposed Alternatives

### Option A: Cats/Scala-Aligned Naming

Based on Scala Cats' `Eval` type which is the closest analogue.

#### Execution Methods

| Current               | Proposed                | Rationale           |
| --------------------- | ----------------------- | ------------------- |
| `Lazy::get()`         | `evaluate()`            | Consistent verb     |
| `Thunk::run()`        | `evaluate()`            | Consistent verb     |
| `Trampoline::run()`   | `evaluate()`            | Consistent verb     |
| `Free::run()`         | `evaluate()`            | Consistent verb     |
| `LazyConfig::force()` | `evaluate()`            | Consistent verb     |
| `Runnable` trait      | `Evaluable`             | Matches method name |
| `Runnable::run()`     | `Evaluable::evaluate()` | Consistent          |

#### Constructors: From Value

| Current               | Proposed              | Rationale                         |
| --------------------- | --------------------- | --------------------------------- |
| `Thunk::pure(a)`      | `now(a)`              | Cats `Eval.now` - immediate value |
| `Trampoline::pure(a)` | `now(a)`              | Consistency                       |
| `Free::pure(a)`       | `now(a)` or `pure(a)` | Could keep `pure` for Free        |
| `Lazy` _(missing)_    | `now(a)`              | NEW: wrap pre-computed value      |

#### Constructors: From Thunk

| Current              | Proposed    | Rationale                         |
| -------------------- | ----------- | --------------------------------- |
| `Thunk::new(f)`      | `always(f)` | Cats `Eval.always` - re-evaluates |
| `Trampoline::new(f)` | `always(f)` | Re-evaluates each run             |
| `Lazy::new(f)`       | `later(f)`  | Cats `Eval.later` - memoizes      |

#### Constructors: Deferred Self

| Current                | Proposed      | Rationale                  |
| ---------------------- | ------------- | -------------------------- |
| `Thunk::defer(f)`      | `defer(f)`    | ✓ Keep (Cats `Eval.defer`) |
| `Trampoline::defer(f)` | `defer(f)`    | ✓ Keep                     |
| `Free::roll(fa)`       | `suspend(fa)` | Standard FP terminology    |

#### Traits

| Current          | Proposed                 | Rationale                                          |
| ---------------- | ------------------------ | -------------------------------------------------- |
| `Defer` trait    | `Deferrable`             | Adjective; avoids collision with `defer()` methods |
| `Defer::defer()` | `Deferrable::deferred()` | Avoids collision                                   |

#### Free Monad Variants

| Current           | Proposed  | Rationale            |
| ----------------- | --------- | -------------------- |
| `FreeInner::Pure` | `Pure`    | ✓ Keep               |
| `FreeInner::Roll` | `Suspend` | Standard terminology |
| `FreeInner::Bind` | `FlatMap` | Cats terminology     |

---

### Option B: Haskell/PureScript-Aligned Naming

#### Execution Methods

| Current           | Proposed    | Rationale              |
| ----------------- | ----------- | ---------------------- |
| `get()` / `run()` | `force()`   | PureScript terminology |
| `Runnable` trait  | `Forceable` | Matches method         |

#### Constructors

| Current    | Proposed     | Rationale                 |
| ---------- | ------------ | ------------------------- |
| `pure(a)`  | `pure(a)`    | ✓ Keep (Haskell standard) |
| `new(f)`   | `delay(f)`   | Haskell-ish               |
| `defer(f)` | `suspend(f)` | Free monad terminology    |

#### Traits

| Current       | Proposed | Rationale |
| ------------- | -------- | --------- |
| `Defer` trait | `Delay`  | Noun form |

#### Free Monad Variants

| Current | Proposed  | Rationale                  |
| ------- | --------- | -------------------------- |
| `Roll`  | `Suspend` | Standard                   |
| `Bind`  | `Bind`    | ✓ Keep (Haskell uses this) |

---

### Option C: Rust-Idiomatic Naming

Prioritizes clarity for Rust developers.

#### Execution Methods

| Current           | Proposed     | Rationale         |
| ----------------- | ------------ | ----------------- |
| `get()` / `run()` | `evaluate()` | Clear active verb |
| `Runnable` trait  | `Evaluable`  | Matches method    |

#### Constructors

| Current          | Proposed           | Rationale          |
| ---------------- | ------------------ | ------------------ |
| `Thunk::pure(a)` | `from_value(a)`    | Rust `From`-style  |
| `Thunk::new(f)`  | `from_fn(f)`       | Rust convention    |
| `Lazy::new(f)`   | `memoized(f)`      | Describes behavior |
| `defer(f)`       | `from_deferred(f)` | Explicit           |

#### Traits

| Current       | Proposed        | Rationale   |
| ------------- | --------------- | ----------- |
| `Defer` trait | `LazyConstruct` | Descriptive |

#### Free Monad Variants

| Current | Proposed    | Rationale               |
| ------- | ----------- | ----------------------- |
| `Roll`  | `Suspended` | Past participle (state) |
| `Bind`  | `Chained`   | Descriptive             |

---

## Summary: All Names to Consider Changing

### Methods

| File            | Current                   | Option A         | Option B      | Option C           |
| --------------- | ------------------------- | ---------------- | ------------- | ------------------ |
| `lazy.rs`       | `Lazy::get()`             | `evaluate()`     | `force()`     | `evaluate()`       |
| `lazy.rs`       | `Lazy::new(f)`            | `later(f)`       | `delay(f)`    | `memoized(f)`      |
| `lazy.rs`       | _(missing)_               | `now(a)`         | `pure(a)`     | `from_value(a)`    |
| `lazy.rs`       | `LazyConfig::force()`     | `evaluate()`     | `force()`     | `evaluate()`       |
| `lazy.rs`       | `LazyConfig::force_try()` | `evaluate_try()` | `force_try()` | `evaluate_try()`   |
| `lazy.rs`       | `LazyConfig::new_lazy()`  | `new_later()`    | `new_delay()` | `new_memoized()`   |
| `thunk.rs`      | `Thunk::run()`            | `evaluate()`     | `force()`     | `evaluate()`       |
| `thunk.rs`      | `Thunk::pure(a)`          | `now(a)`         | `pure(a)`     | `from_value(a)`    |
| `thunk.rs`      | `Thunk::new(f)`           | `always(f)`      | `delay(f)`    | `from_fn(f)`       |
| `thunk.rs`      | `Thunk::defer(f)`         | `defer(f)` ✓     | `suspend(f)`  | `from_deferred(f)` |
| `trampoline.rs` | `Trampoline::run()`       | `evaluate()`     | `force()`     | `evaluate()`       |
| `trampoline.rs` | `Trampoline::pure(a)`     | `now(a)`         | `pure(a)`     | `from_value(a)`    |
| `trampoline.rs` | `Trampoline::new(f)`      | `always(f)`      | `delay(f)`    | `from_fn(f)`       |
| `trampoline.rs` | `Trampoline::defer(f)`    | `defer(f)` ✓     | `suspend(f)`  | `from_deferred(f)` |
| `free.rs`       | `Free::run()`             | `evaluate()`     | `force()`     | `evaluate()`       |
| `free.rs`       | `Free::pure(a)`           | `pure(a)` ✓      | `pure(a)` ✓   | `from_value(a)`    |
| `free.rs`       | `Free::roll(fa)`          | `suspend(fa)`    | `suspend(fa)` | `suspended(fa)`    |

### Traits

| File          | Current           | Option A                 | Option B             | Option C                          |
| ------------- | ----------------- | ------------------------ | -------------------- | --------------------------------- |
| `defer.rs`    | `Defer`           | `Deferrable`             | `Delay`              | `LazyConstruct`                   |
| `defer.rs`    | `Defer::defer()`  | `Deferrable::deferred()` | `Delay::delay()`     | `LazyConstruct::lazy_construct()` |
| `runnable.rs` | `Runnable`        | `Evaluable`              | `Forceable`          | `Evaluable`                       |
| `runnable.rs` | `Runnable::run()` | `Evaluable::evaluate()`  | `Forceable::force()` | `Evaluable::evaluate()`           |

### Enum Variants

| File      | Current           | Option A  | Option B  | Option C    |
| --------- | ----------------- | --------- | --------- | ----------- |
| `free.rs` | `FreeInner::Pure` | `Pure` ✓  | `Pure` ✓  | `Pure` ✓    |
| `free.rs` | `FreeInner::Roll` | `Suspend` | `Suspend` | `Suspended` |
| `free.rs` | `FreeInner::Bind` | `FlatMap` | `Bind` ✓  | `Chained`   |

### Type Aliases

| File      | Current   | Option A          | Option B          | Option C          |
| --------- | --------- | ----------------- | ----------------- | ----------------- |
| `free.rs` | `Val`     | `ErasedValue`     | `AnyValue`        | `TypeErased`      |
| `free.rs` | `Cont<F>` | `Continuation<F>` | `Continuation<F>` | `Continuation<F>` |

---

## Recommendation

**Option A (Cats-Aligned)** is recommended because:

1. The codebase already uses Cats-like patterns (`pure`, `bind`, `MonadRec`, `tail_rec_m`)
2. Provides clear semantic distinction: `now` (immediate) vs `always` (re-eval) vs `later` (memoize)
3. `Suspend` for Free monad is standard across FP literature
4. Maintains `defer` for the important stack-safety pattern
5. `Deferrable` as trait name avoids collision while staying descriptive

### Priority Changes (High Impact)

1. Unify execution to `evaluate()` across all types
2. Rename `Thunk::new()` to `always()` and `Lazy::new()` to `later()`
3. Add `Lazy::now(a)` constructor
4. Rename `FreeInner::Roll` to `Suspend` and `Free::roll()` to `suspend()`
5. Rename `Defer` trait to `Deferrable` with method `deferred()`
6. Rename `Runnable` trait to `Evaluable` with method `evaluate()`

### Secondary Changes (Lower Impact)

1. Rename `FreeInner::Bind` to `FlatMap`
2. Rename type aliases `Val` → `ErasedValue`, `Cont` → `Continuation`
3. Consider `now(a)` vs keeping `pure(a)` for value constructors
