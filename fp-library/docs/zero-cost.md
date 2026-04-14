### Zero-Cost Abstractions & Uncurried Semantics

Unlike many functional programming libraries that strictly adhere to curried functions (e.g., `map(f)(fa)`), `fp-library` adopts **uncurried semantics** (e.g., `map(f, fa)`) for its core abstractions.

**Why?**
Traditional currying in Rust often requires:

- Creating intermediate closures for each partial application.
- Heap-allocating these closures (boxing) or wrapping them in reference counters (`Rc`/`Arc`) to satisfy type system constraints.
- Dynamic dispatch (`dyn Fn`), which inhibits compiler optimizations like inlining.

By using uncurried functions with `impl Fn` or generic bounds, `fp-library` achieves **zero-cost abstractions**:

- **No Heap Allocation:** Operations like `map` and `bind` do not allocate intermediate closures.
- **Static Dispatch:** The compiler can fully monomorphize generic functions, enabling aggressive inlining and optimization.
- **Ownership Friendly:** Better integration with Rust's ownership and borrowing system.

This approach ensures that using high-level functional abstractions incurs no runtime penalty compared to hand-written imperative code.

**Exceptions:**
While the library strives for zero-cost abstractions, some operations inherently require dynamic dispatch or heap allocation due to Rust's type system:

- **Functions as Data:** When functions are stored in data structures (e.g., inside a `Vec` for `Semiapplicative::apply`, or in `Lazy` thunks), they must often be "type-erased" (wrapped in `Rc<dyn Fn>` or `Arc<dyn Fn>`). This is because every closure in Rust has a unique, anonymous type. To store multiple different closures in the same container, or to compose functions dynamically (like in `Endofunction`), they must be coerced to a common trait object.
- **Lazy Evaluation:** The `Lazy` type relies on storing a thunk that can be cloned and evaluated later, which typically requires reference counting and dynamic dispatch.

For these specific cases, the library provides `Brand` types (like `RcFnBrand` and `ArcFnBrand`) to let you choose the appropriate wrapper (single-threaded vs. thread-safe) while keeping the rest of your code zero-cost. The library uses a unified `Pointer` hierarchy to abstract over these choices.
