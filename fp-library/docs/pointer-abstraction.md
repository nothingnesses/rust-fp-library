### Pointer Abstraction & Shared Semantics

The library uses a flat set of independent pointer traits to abstract over pointer strategies (`Box` vs `Rc` vs `Arc`) and to enable shared memoization semantics for lazy evaluation.

#### Pointer Traits

The pointer and coercion traits are organized as independent traits rather than a linear supertrait chain. Supertrait links exist only where a trait's methods reference the supertrait's associated type:

```mermaid
---
config:
  layout: elk
---
graph TD
    Pointer["Pointer (Deref)"] -.independent.-> RefCountedPointer["RefCountedPointer (Clone + Deref)"]
    Pointer -.independent.-> SendRefCountedPointer["SendRefCountedPointer (Clone + Send + Sync + Deref)"]
    RefCountedPointer --> ToDynCloneFn["ToDynCloneFn"]
    SendRefCountedPointer --> ToDynSendFn["ToDynSendFn"]
    Pointer --> ToDynFn["ToDynFn"]
    Pointer --> ToDynFnOnce["ToDynFnOnce"]
    BoxBrand -.implements.-> Pointer
    BoxBrand -.implements.-> ToDynFn
    BoxBrand -.implements.-> ToDynFnOnce
    RcBrand -.implements.-> Pointer
    RcBrand -.implements.-> RefCountedPointer
    RcBrand -.implements.-> ToDynFn
    RcBrand -.implements.-> ToDynCloneFn
    ArcBrand -.implements.-> Pointer
    ArcBrand -.implements.-> RefCountedPointer
    ArcBrand -.implements.-> SendRefCountedPointer
    ArcBrand -.implements.-> ToDynFn
    ArcBrand -.implements.-> ToDynCloneFn
    ArcBrand -.implements.-> ToDynSendFn
```

| Trait                   | Associated type                    | Bounds                        | Purpose                                                                 |
| :---------------------- | :--------------------------------- | :---------------------------- | :---------------------------------------------------------------------- |
| `Pointer`               | `Of`                               | `Deref`                       | Heap-allocated pointer                                                  |
| `RefCountedPointer`     | `Of`, `TakeCellOf`                 | `Clone + Deref`               | Clonable reference-counted pointer                                      |
| `SendRefCountedPointer` | `Of`                               | `Clone + Send + Sync + Deref` | Thread-safe reference-counted pointer                                   |
| `ToDynFn`               | (uses `Pointer::Of`)               |                               | Coerce `impl Fn` -> `dyn Fn` behind a pointer                           |
| `ToDynFnOnce`           | (uses `Pointer::Of`)               |                               | Coerce `impl FnOnce` -> `dyn FnOnce` behind a pointer (single-shot)     |
| `ToDynCloneFn`          | (uses `RefCountedPointer::Of`)     |                               | Coerce `impl Fn` -> `dyn Fn` behind a clonable pointer                  |
| `ToDynSendFn`           | (uses `SendRefCountedPointer::Of`) |                               | Coerce `impl Fn` -> `dyn Fn + Send + Sync` behind a thread-safe pointer |

| Brand      | Implements                                                                                                                                                      |
| :--------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `BoxBrand` | `Pointer`, `ToDynFn`, `ToDynFnOnce`                                                                                                                             |
| `RcBrand`  | `Pointer`, `RefCountedPointer`, `ToDynFn`, `ToDynCloneFn`                                                                                                       |
| `ArcBrand` | All six multi-shot traits (`Pointer`, `RefCountedPointer`, `SendRefCountedPointer`, `ToDynFn`, `ToDynCloneFn`, `ToDynSendFn`); does NOT implement `ToDynFnOnce` |

#### `ToDynFnOnce` is `BoxBrand`-only by structural necessity

The `(pointer-capability, closure-semantic)` matrix maps each pair to exactly the brands that legitimately host it:

| Closure semantic       | Owned (`BoxBrand`) | Refcounted (`RcBrand`)     | Thread-safe refcounted (`ArcBrand`) |
| :--------------------- | :----------------- | :------------------------- | :---------------------------------- |
| `Fn` (multi-shot)      | `ToDynFn`          | `ToDynFn` + `ToDynCloneFn` | All four `Fn`-family traits         |
| `FnOnce` (single-shot) | `ToDynFnOnce`      | (operationally broken)     | (operationally broken)              |

`Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` cannot be implemented because [`FnOnce::call_once`](https://doc.rust-lang.org/stable/core/ops/trait.FnOnce.html) consumes `self` (the trait object), which cannot be moved out of a shared pointer without invalidating other clones. The only legitimate `dyn FnOnce` carrier is `Box<dyn FnOnce>`, where the standard library's blanket [`impl<F: ?Sized + FnOnce<Args>> FnOnce<Args> for Box<F>`](https://doc.rust-lang.org/stable/core/ops/trait.FnOnce.html#impl-FnOnce%3CArgs%3E-for-Box%3CF,+A%3E) consumes the box on call, moving the underlying `FnOnce` out and dropping the `Box` allocation in one step.

**Use-case:** single-shot continuation cells in effect types such as `BoxState`, `BoxReader`, `BoxChoose`, and default `Run` scoped-effect closure storage, where the program-tree's single-shot semantics make `FnOnce` the precise type and `Box<dyn Fn>` would over-promise multi-shot semantics. Multi-shot wrappers (`RcRun` / `ArcRun` and their `Explicit` siblings) keep using the cloneable `dyn Fn` paths via `ToDynCloneFn` / `ToDynSendFn`.

#### Generic Function Brands

`FnBrand<P>` is parameterized over a `RefCountedPointer` brand `P`:

- `RcFnBrand` is a type alias for `FnBrand<RcBrand>`.
- `ArcFnBrand` is a type alias for `FnBrand<ArcBrand>`.

This allows a unified implementation of `CloneFn` while `SendCloneFn` is only implemented when `P: ToDynSendFn`. Both traits are parameterized by `ClosureMode` (`Val` or `Ref`), controlling whether the wrapped closure takes its input by value (`Fn(A) -> B`) or by reference (`Fn(&A) -> B`). The composable variant `Arrow` (for optics) adds `Category + Strong` supertraits. Code that is generic over the pointer brand can work with either `Rc` or `Arc` without duplication.

#### Shared Memoization

`Lazy` uses a configuration trait (`LazyConfig`) to abstract over the underlying storage and synchronization primitives, ensuring shared memoization semantics across clones:

- `Lazy<'a, A, Config>` is parameterized by a `LazyConfig` which defines the storage type.
- `RcLazy` uses `Rc<LazyCell>` for single-threaded, shared memoization.
- `ArcLazy` uses `Arc<LazyLock>` for thread-safe, shared memoization.

This ensures Haskell-like semantics where forcing one reference updates the value for all clones.
See [Lazy Evaluation](./lazy-evaluation.md) for the full type overview, trade-offs, and decision guide.

#### Design Rationale

- **Correctness:** Ensures `Lazy` behaves correctly as a shared thunk rather than a value that is re-evaluated per clone.
- **Performance:** Leverages standard library types (`LazyCell`, `LazyLock`) for efficient, correct-by-construction memoization.
- **Flexibility:** Separates the concern of memoization (`Lazy`) from computation (`Trampoline`/`Thunk`), allowing users to choose the right tool for the job.
