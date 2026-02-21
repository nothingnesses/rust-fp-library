# Lifetime Ablation Experiment

## Background
This experiment was conducted to explore the impact of removing explicit lifetime parameters from the library's Higher-Kinded Type (HKT) simulation. 

### Motivation
The primary driver for this change was **API Ergonomics**. The previous HKT model required every `Kind` and its associated `Of` type to carry a lifetime parameter (e.g., `type Of<'a, A: 'a>: 'a`). This complexity propagated into every HKT-aware trait (`Functor`, `Monad`, `Applicative`, etc.) and every function using them, resulting in:
*   **"Lifetime Soup"**: Signatures became difficult to read and write (e.g., `fn map<'a, Brand: Functor, A: 'a, B: 'a, Func>(f: Func, fa: Apply!(...))`).
*   **Macro Complexity**: The internal macro-generated code for `Kind` traits was significantly more complex to maintain and debug.
*   **Onboarding Friction**: New users faced a steep learning curve understanding why lifetimes were necessary for seemingly simple FP operations.

### Goal
The goal was to replace lifetime-aware `Kind` traits with a simplified version (e.g., `type Of<A>`).

## Analyses and Findings

### 1. Ergonomic Gains
The experiment successfully simplified the public API. Trait definitions and function signatures became much cleaner, resembling those found in languages like Haskell or PureScript, where lifetimes are not a first-class concern.

### 2. The "Closure Hurdle" (Technical Breakdown)
Removing lifetimes introduced **17 compiler errors (E0310)** across the library. These errors are not merely boilerplate issues but represent a fundamental limitation of the simplified model.

#### Affected Files
The errors were confined to the following files, which all implement types relying on type erasure (trait objects):
*   `fp-library/src/types/arc_ptr.rs`
*   `fp-library/src/types/free.rs`
*   `fp-library/src/types/lazy.rs`
*   `fp-library/src/types/rc_ptr.rs`
*   `fp-library/src/types/thunk.rs`
*   `fp-library/src/types/try_lazy.rs`
*   `fp-library/src/types/try_thunk.rs`

#### Error Analysis
All errors encountered were **E0310**, indicating that a generic parameter was not known to live long enough to satisfy the implicit `'static` bound of a trait object.

**Example Error (from `fp-library/src/types/thunk.rs`):**
```text
error[E0310]: the parameter type `F` may not live long enough
   --> fp-library/src/types/thunk.rs:110:10
    |
110 |             Thunk(Box::new(f))
    |                   ^^^^^^^^^^^
    |                   |
    |                   the parameter type `F` must be valid for the static lifetime...
    |                   ...so that the type `F` will meet its required lifetime bounds
help: consider adding an explicit lifetime bound
    |
108 |             F: FnOnce() -> A + 'static,
    |                              +++++++++
```
This error occurs because `Thunk` stores a `Box<dyn FnOnce() -> A>`. Since no lifetime is specified on the box, Rust infers `Box<dyn FnOnce() -> A + 'static>`. However, the constructor accepts a generic `F`, which might contain non-static references (e.g., a closure capturing a local variable). The compiler rightly complains that `F` might die before the static lifetime required by the box.

#### Root Cause: Trait Object Defaulting
Most HKTs that wrap functions or delayed computations (e.g., `RcFnBrand`, `LazyBrand`, `ThunkBrand`, `Free`) rely on type erasure via trait objects (`dyn Fn` or `dyn FnOnce`).
*   In Rust, a trait object `dyn Trait` has an implicit lifetime bound. If not specified, it defaults to `'static` (e.g., `Box<dyn Trait + 'static>`).
*   Previously, the HKT system used the `'a` parameter to constrain these trait objects: `Rc<dyn 'a + Fn(A) -> B>`. This told the compiler "this object is valid for lifetime `'a`".
*   By removing `'a` from `Kind`, these implementations lost the ability to express a non-static lifetime, forcing them to default to `'static`.

#### The Impact on Closures
When a user provides a closure to `map` or `bind`, that closure often captures variables from the surrounding scope.
*   **With `'a`**: A closure capturing `&'a x` has lifetime `'a`. The HKT system could store this closure because `Kind` accepted `'a`.
*   **Without `'a`**: The HKT system (defaulting to `'static`) rejects any closure that captures local references. It only accepts closures that:
    1.  Capture nothing (functions).
    2.  Capture only `'static` data.
    3.  Own all captured data (via `move`).

This effectively disables standard FP patterns like:
```rust
let x = 10;
let res = list.map(|y| x + y); // Error: closure captures `x` by reference, not static.
```

### 3. Impact Distribution
Interestingly, simple container types like `Vec`, `Option`, and `Result` did **not** error.
*   These types use **Concrete Generics** (e.g., `Vec<T>`), which naturally inherit the lifetime of `T`. They do not use type erasure (trait objects) to hide the type of `T`.
*   The errors were exclusively found in types that use **Type Erasure** (`Thunk`, `Lazy`, `Free`, `RcFn`, `ArcFn`), proving that the lifetime parameter is strictly necessary for "computational" data structures in Rust, even if "storage" data structures can infer it.

## Potential Approaches

### Approach 1: Revert to Explicit Lifetimes (Recommended)
Restore the lifetime parameter `'a` to the `Kind` trait and its associated types.
*   **Pros**: Fully supports Rust's ownership/borrowing model. Correctly handles closures capturing local state.
*   **Cons**: Reintroduces "Lifetime Soup" and API complexity.
*   **Verdict**: Necessary for a general-purpose FP library.

### Approach 2: The "Owned Environment" Model
Accept the limitation and strictly require all HKT computations to be `'static`.
*   **Pros**: Keeps the clean, simple API.
*   **Cons**: Users cannot capture local references. They must `move` owned data (or `Rc`/`Arc` clones) into every closure.
*   **Verdict**: Viable only if the library pivots to a specific niche (e.g., async-focused or "data-only" FP) and abandons general-purpose ergonomics.

### Approach 3: Split the `Kind` Trait
Create separate traits for static and borrowed kinds (e.g., `StaticKind` vs `BorrowedKind`).
*   **Pros**: Simple types stay simple.
*   **Cons**: Bifurcates the ecosystem. Functions like `map` would require complex or duplicate implementations.
*   **Verdict**: Likely adds more confusion than it solves.

## Conclusion
The experiment demonstrates that while removing lifetimes simplifies the API, it fundamentally conflicts with Rust's memory model for **functional abstractions involving closures**.

The library relies heavily on "computational" types (`Lazy`, `Thunk`, `Free`) that wrap functions. In Rust, functions (closures) often have lifetimes. Erasing these lifetimes via trait objects without a mechanism to propagate them (`'a`) forces them to be `'static`.

Therefore, to support standard functional programming patterns (like capturing local context in `map`), the **explicit lifetime parameter `'a` must be retained**, despite the ergonomic cost. The "Lifetime Soup" is the price of admission for safe, zero-cost functional abstractions in Rust.
