# Lifetime Ablation Experiment

## Background
This experiment was conducted to explore the impact of removing explicit lifetime parameters from the library's Higher-Kinded Type (HKT) simulation. 

### Motivation
The primary driver for this change was **API Ergonomics**. The previous HKT model required every `Kind` and its associated `Of` type to carry a lifetime parameter (e.g., `type Of<'a, A: 'a>: 'a`). This complexity propagated into every HKT-aware trait (`Functor`, `Monad`, `Applicative`, etc.) and every function using them, resulting in:
*   **"Lifetime Soup"**: Signatures became difficult to read and write (e.g., `fn map<'a, Brand: Functor, A: 'a, B: 'a, Func>(f: Func, fa: Apply!(...))`).
*   **Macro Complexity**: The internal macro-generated code for `Kind` traits was significantly more complex to maintain and debug.
*   **Onboarding Friction**: New users faced a steep learning curve understanding why lifetimes were necessary for seemingly simple FP operations.

### Goal
The goal was to replace lifetime-aware `Kind` traits with a simplified version (e.g., `type Of<A>`), assuming that most common use cases could either be `'static` or that Rust's type system could infer the necessary bounds without explicit tracking in the HKT trait.

## Analyses and Findings

### 1. Ergonomic Gains
The experiment successfully simplified the public API. Trait definitions and function signatures became much cleaner, resembling those found in languages like Haskell or Scala, where lifetimes are not a first-class concern.

### 2. The "Closure Hurdle" (Technical Breakdown)
Removing lifetimes introduced **17 compiler errors (E0310)** across the library. These errors are not merely boilerplate issues but represent a fundamental limitation of the simplified model.

#### Root Cause: Trait Object Defaulting
Most HKTs that wrap functions or delayed computations (e.g., `RcFnBrand`, `LazyBrand`, `ThunkBrand`, `Free`) rely on type erasure via trait objects (`dyn Fn` or `dyn FnOnce`).
*   In Rust, a trait object `dyn Trait` has an implicit lifetime bound. If no lifetime is specified, it defaults to `+ 'static`.
*   Previously, the HKT system used the `'a` parameter to constrain these trait objects: `Rc<dyn 'a + Fn(A) -> B>`.
*   Without `'a`, the compiler defaults to `Rc<dyn 'static + Fn(A) -> B>`.

#### The Impact on Closures
When a user provides a closure to `map` or `bind`, that closure might capture local variables.
*   **Reference Captures**: If a closure captures a reference (`&T`), it is non-`'static`. It cannot be stored in a `+ 'static` trait object, resulting in an immediate E0310 error.
*   **Owned Captures (The "Move" Strategy)**: Even if a closure captures data by value (using the `move` keyword), the compiler cannot **prove** the closure is `'static` unless the trait bound explicitly requires it (e.g., `Func: Fn(A) -> B + 'static`).

### 3. The "Clone and Move" Strategy
One potential workaround discussed was asking users to clone data and move it into closures to satisfy the `'static` requirement.

**Example:**
```rust
let x = vec![1, 2, 3];
let x_cloned = x.clone();
map::<LazyBrand, _, _, _>(move |i| i + x_cloned.len(), fa);
```

**Findings on this strategy:**
*   **Pros**: It allows the simplified HKT model to work for computations that can afford to own their data. It pushes the library toward a "purer" functional model where data is not shared via short-lived references.
*   **Cons**:
    *   **Inexpressibility**: Many common Rust patterns (e.g., mapping over a `Vec<&str>`) become impossible to represent in the HKT system.
    *   **Boilerplate**: Users are forced into a verbose `clone`/`move` pattern.
    *   **Performance**: Mandatory cloning introduces significant overhead for large data structures.
    *   **Internal Trait Complexity**: To support this, the library must add `+ 'static` bounds to almost every closure-accepting method in its trait hierarchy, which is itself a form of "trait soup."

## Potential Approaches

### Approach A: Revert to Lifetime-Aware HKTs
Roll back the ablation and restore the `'a` parameter to the `Kind` trait and its implementors.
*   **Pros**:
    *   **Full Power**: Supports the entire spectrum of Rust types, including those with short-lived references.
    *   **Consistency**: Matches the behavior of other low-level Rust libraries that need to handle lifetimes.
*   **Cons**:
    *   **Complexity**: Reintroduces the "lifetime soup" that the experiment sought to eliminate.
    *   **Maintenance**: Increases the complexity of macro-generated code.

### Approach B: Mandatory 'static Ownership (Owned-Only FP)
Adopt the simplified HKT model but enforce `'static` bounds on all generic parameters used in HKT-aware traits.
*   **Pros**:
    *   **Simplicity**: The cleanest possible API signatures.
    *   **Safety**: Eliminates most lifetime-related borrow checker issues for the end-user.
*   **Cons**:
    *   **Restrictiveness**: Prevents the use of the library with any data containing non-`'static` references.
    *   **Fragmentation**: Users who need references will find the library unusable and may need to seek alternative abstractions.

### Approach C: Hybrid Bifurcated Hierarchy
Provide two parallel versions of the HKT traits: a simplified version for `'static` types and a lifetime-aware version for general use.
*   **Pros**: Offers the "best of both worlds."
*   **Cons**:
    *   **Massive Duplication**: Requires duplicating the entire trait hierarchy (`Functor` vs `StaticFunctor`).
    *   **User Confusion**: Users must constantly decide which version of the trait to implement or use.

### Approach D: GAT-based Lifetime Inference (Experimental)
Leverage more advanced Generic Associated Type (GAT) patterns, possibly using "bottom" lifetimes or default lifetime parameters if they become available in future Rust versions.
*   **Pros**: Potential for a high-power, low-complexity API.
*   **Cons**: Not feasible in stable Rust 1.80+ without significant and fragile macro trickery.

## Conclusion
The Lifetime Ablation Experiment has demonstrated that **lifetimes are not merely an aesthetic burden in a Rust HKT library; they are a functional necessity** for a system that aims to be general-purpose and support type-erased containers like `Rc` and `Lazy`.

While the "Clone and Move" strategy (Approach B) offers a path toward a simpler API, it fundamentally changes the nature of the library from a general-purpose tool to an "owned-only" framework. 

**Recommendation:**
The library should likely return to Approach A (Lifetime-Aware HKTs) to maintain full expressiveness, but focus on improving macro-driven documentation and helper types to hide as much of the "lifetime soup" as possible from the end-user.
