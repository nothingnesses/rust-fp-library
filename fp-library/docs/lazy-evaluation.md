### Lazy Evaluation & Effect System

Rust is an eagerly evaluated language. To enable functional patterns like deferred execution and safe recursion, `fp-library` provides a granular set of types that let you opt-in to specific behaviors without paying for unnecessary overhead.

**User stories:**

- **Thunk / SendThunk:** "I want to defer a computation and evaluate it later."
- **Trampoline (Free):** "I want to chain binds without blowing the stack."
- **Lazy (RcLazy / ArcLazy):** "I want to compute a value at most once and cache it."
- **Try\* variants:** "I want any of the above, but the computation may fail."

#### Type Overview

The hierarchy consists of infallible computation types, fallible counterparts, and the `Free` monad infrastructure. Each type makes different trade-offs around stack safety, memoization, lifetimes, and thread safety.

| Type                     | Underlying                                 | HKT                                    | Stack Safe                  | Memoized | Lifetimes | Send |
| ------------------------ | ------------------------------------------ | -------------------------------------- | --------------------------- | -------- | --------- | ---- |
| `Thunk<'a, A>`           | `Box<dyn FnOnce() -> A + 'a>`              | Yes (full)                             | Partial (`tail_rec_m` only) | No       | `'a`      | No   |
| `SendThunk<'a, A>`       | `Box<dyn FnOnce() -> A + Send + 'a>`       | No                                     | Partial (`tail_rec_m` only) | No       | `'a`      | Yes  |
| `Trampoline<A>`          | `Free<ThunkBrand, A>`                      | No                                     | Yes                         | No       | `'static` | No   |
| `RcLazy<'a, A>`          | `Rc<LazyCell<A, ...>>`                     | Partial (`RefFunctor`)                 | N/A                         | Yes      | `'a`      | No   |
| `ArcLazy<'a, A>`         | `Arc<LazyLock<A, ...>>`                    | Partial (`SendRefFunctor`)             | N/A                         | Yes      | `'a`      | Yes  |
| `TryThunk<'a, A, E>`     | `Thunk<'a, Result<A, E>>`                  | Yes (full)                             | Partial (`tail_rec_m` only) | No       | `'a`      | No   |
| `TrySendThunk<'a, A, E>` | `SendThunk<'a, Result<A, E>>`              | No                                     | Partial (`tail_rec_m` only) | No       | `'a`      | Yes  |
| `TryTrampoline<A, E>`    | `Trampoline<Result<A, E>>`                 | No                                     | Yes                         | No       | `'static` | No   |
| `RcTryLazy<'a, A, E>`    | `Rc<LazyCell<Result<A, E>, ...>>`          | Partial (`RefFunctor`, `Foldable`)     | N/A                         | Yes      | `'a`      | No   |
| `ArcTryLazy<'a, A, E>`   | `Arc<LazyLock<Result<A, E>, ...>>`         | Partial (`SendRefFunctor`, `Foldable`) | N/A                         | Yes      | `'a`      | Yes  |
| `Free<F, A>`             | CatList-based "Reflection without Remorse" | No                                     | Yes                         | No       | `'static` | No   |

**Config-dependent Send:** `ArcLazy`/`ArcTryLazy` are `Send + Sync`; `RcLazy`/`RcTryLazy` are not. The `LazyConfig` trait abstracts over the underlying pointer and cell types; see [Pointer Abstraction](./pointer-abstraction.md) for how `FnBrand<P>` and `LazyConfig` generalize over `Rc`/`Arc`.

At a glance, the primary use cases are:

| Type                   | Primary Use Case                                                                                                            |
| :--------------------- | :-------------------------------------------------------------------------------------------------------------------------- |
| **`Thunk<'a, A>`**     | **Glue Code & Borrowing.** Lightweight deferred computation. Best for short chains and working with references.             |
| **`SendThunk<'a, A>`** | **Thread-Safe Glue Code.** Like `Thunk`, but the closure is `Send`. Enables truly lazy `into_arc_lazy()`.                   |
| **`Trampoline<A>`**    | **Deep Recursion & Pipelines.** Heavy-duty computation. Uses a trampoline to guarantee stack safety for infinite recursion. |
| **`Lazy<'a, A>`**      | **Caching.** Wraps a computation to ensure it runs at most once. `RcLazy` for single-threaded, `ArcLazy` for thread-safe.   |

Each of these has a fallible counterpart that wraps `Result<A, E>` with ergonomic error-handling combinators (`TryThunk`, `TrySendThunk`, `TryTrampoline`, `TryLazy`).

#### Supporting Traits

| Trait                | Purpose                                                 | Implementors in hierarchy                                                                                                             |
| -------------------- | ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `Deferrable<'a>`     | Lazy construction from thunk                            | `Thunk`, `SendThunk`, `Trampoline`, `RcLazy`, `ArcLazy`, `RcTryLazy`, `ArcTryLazy`, `TryThunk`, `TrySendThunk`, `Free<ThunkBrand, A>` |
| `SendDeferrable<'a>` | Thread-safe lazy construction (extends `Deferrable`)    | `SendThunk`, `TrySendThunk`, `ArcLazy`, `ArcTryLazy`                                                                                  |
| `RefFunctor`         | Mapping with `&A` input                                 | `LazyBrand<RcLazyConfig>`, `TryLazyBrand<E, RcLazyConfig>`                                                                            |
| `SendRefFunctor`     | Thread-safe mapping with `&A` input                     | `LazyBrand<ArcLazyConfig>`, `TryLazyBrand<E, ArcLazyConfig>`                                                                          |
| `LazyConfig`         | Infallible memoization strategy (pointer + cell choice) | `RcLazyConfig`, `ArcLazyConfig`                                                                                                       |
| `TryLazyConfig`      | Fallible memoization strategy (extends `LazyConfig`)    | `RcLazyConfig`, `ArcLazyConfig`                                                                                                       |

#### The "Why" of Multiple Types

Unlike lazy languages (e.g., Haskell) where the runtime handles everything, Rust requires us to choose our trade-offs:

1. **`Thunk` vs `Trampoline`**: `Thunk` is faster and supports borrowing (`&'a T`). Its `tail_rec_m` is stack-safe, but deep `bind` chains will overflow the stack. `Trampoline` guarantees stack safety for all operations via a trampoline (the `Free` monad) but requires types to be `'static`. Note that `!Send` types like `Rc<T>` are fully supported. A key distinction is that `Thunk` implements `Functor`, `Applicative`, and `Monad` directly, making it suitable for generic programming, while `Trampoline` does not.

2. **`Thunk` vs `SendThunk`**: `Thunk` wraps `Box<dyn FnOnce() -> A + 'a>` and is `!Send`. `SendThunk` wraps `Box<dyn FnOnce() -> A + Send + 'a>` and can cross thread boundaries. Use `SendThunk` when you need truly lazy `into_arc_lazy()` (converting to `ArcLazy` without eager evaluation), or when building deferred computation chains that will be consumed on another thread. `TrySendThunk` is the fallible counterpart.

3. **Computation vs Caching**: `Thunk` and `Trampoline` describe _computations_ that are not memoized. Each instance is consumed on `.evaluate()` (which takes `self` by value), so the computation runs exactly once per instance, but constructing a new instance re-executes the work. `Lazy`, by contrast, caches the result so that all clones share a single evaluation. If you have an expensive operation (like a DB call), convert it to a `Lazy` to guarantee it runs at most once.

4. **`RefFunctor` vs `Functor`**: `Lazy::evaluate()` returns `&A`, not `A`. The standard `Functor` trait expects owned `A`, so automatically cloning would violate the library's zero-cost abstraction principle. `RefFunctor` honestly represents what `Lazy` can do: mapping with `&A -> B`.

5. **`LazyConfig` vs `TryLazyConfig`**: The memoization strategy is split into two traits. `LazyConfig` covers infallible memoization (pointer type, lazy cell, thunk type). `TryLazyConfig` extends it with fallible variants (`TryLazy`, `TryThunk`). Third-party implementations can choose to implement only `LazyConfig` when fallible memoization is not needed.

#### Quick Decision Guide

- **Short deferred computation chains**: Use `Thunk`.
- **Deep recursion or long pipelines**: Use `Trampoline`.
- **Cache an expensive result (single-threaded)**: Use `RcLazy`.
- **Cache an expensive result (multi-threaded)**: Use `ArcLazy`.
- **Deferred computation that must be `Send`**: Use `SendThunk`.
- **Any of the above, but fallible**: Use the `Try*` counterpart.
- **Generic programming over lazy types**: Use `Deferrable` / `SendDeferrable` trait bounds.

#### Workflow Example: Expression Evaluator

A robust pattern is to use `TryTrampoline` for stack-safe, fallible recursion, `TryLazy` to memoize expensive results, and `TryThunk` to create lightweight views.

Consider an expression evaluator that handles division errors and deep recursion:

```rust
use fp_library::types::*;

#[derive(Clone)]
enum Expr {
	Val(i32),
	Add(Box<Expr>, Box<Expr>),
	Div(Box<Expr>, Box<Expr>),
}

// 1. Stack-safe recursion with error handling (TryTrampoline)
fn eval(expr: &Expr) -> TryTrampoline<i32, String> {
	let expr = expr.clone(); // Capture owned data for 'static closure
	TryTrampoline::defer(move || match expr {
		Expr::Val(n) => TryTrampoline::ok(n),
		Expr::Add(lhs, rhs) => eval(&lhs).bind(move |l| eval(&rhs).map(move |r| l + r)),
		Expr::Div(lhs, rhs) => eval(&lhs).bind(move |l| {
			eval(&rhs).bind(move |r| {
				if r == 0 {
					TryTrampoline::err("Division by zero".to_string())
				} else {
					TryTrampoline::ok(l / r)
				}
			})
		}),
	})
}

// Usage
fn main() {
	let expr = Expr::Div(Box::new(Expr::Val(100)), Box::new(Expr::Val(2)));

	// 2. Memoize result (TryLazy)
	// The evaluation runs at most once, even if accessed multiple times.
	let result = RcTryLazy::new(move || eval(&expr).evaluate());

	// 3. Create deferred view (TryThunk)
	// Borrow the cached result to format it.
	let view: TryThunk<String, String> = TryThunk::new(|| {
		let val = result.evaluate().map_err(|e| e.clone())?;
		Ok(format!("Result: {}", val))
	});

	assert_eq!(view.evaluate(), Ok("Result: 50".to_string()));
}
```
