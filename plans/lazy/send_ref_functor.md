# SendRefFunctor Analysis

## Overview

`SendRefFunctor` is a type class for types that can be mapped over by receiving references to their contents, with the additional constraint that the mapping function must be `Send`. It lives at `/fp-library/src/classes/send_ref_functor.rs` and currently has two implementors: `LazyBrand<ArcLazyConfig>` (for `ArcLazy`) and `TryLazyBrand<E, ArcLazyConfig>` (for `ArcTryLazy`).

## Trait Definition

```rust
#[kind(type Of<'a, A: 'a>: 'a;)]
pub trait SendRefFunctor {
    fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
        func: impl FnOnce(&A) -> B + Send + 'a,
        fa: Apply!(<Self as Kind!(...)>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
}
```

## 1. Trait Design: The Send/Non-Send Split

### Intentional Independence from RefFunctor

The most notable design choice is that `SendRefFunctor` does **not** extend `RefFunctor` as a supertrait. This directly contrasts with `SendDeferrable`, which **does** extend `Deferrable`:

| Trait pair | Relationship | Reason |
|---|---|---|
| `SendDeferrable: Deferrable` | Supertrait | `Deferrable::defer` takes `FnOnce() -> Self + 'a`; `SendDeferrable::send_defer` adds `Send`. An `ArcLazy` can implement both because the non-Send `defer` can simply evaluate eagerly (documented caveat). |
| `SendRefFunctor` vs `RefFunctor` | Independent | `RefFunctor::ref_map` takes `impl FnOnce(&A) -> B + 'a` (no `Send`). `ArcLazy::new` requires the closure to be `Send`, so a `RefFunctor` impl for `ArcLazy` would accept closures it cannot actually store. |

This independence is well-motivated. The `RefFunctor` contract promises callers they can pass non-`Send` closures. If `ArcLazy` implemented `RefFunctor`, the implementation would need to either:
- Reject non-`Send` closures at impl level (violating the trait contract), or
- Add hidden `Send` bounds that defeat the purpose of having a non-`Send` trait.

The result is a clean partition:
- `RcLazy` implements `RefFunctor` only.
- `ArcLazy` implements `SendRefFunctor` only.
- Neither implements both.

This is the correct design. Making `SendRefFunctor: RefFunctor` would be unsound for `ArcLazy` because `Arc<LazyLock<...>>` requires `Send` on the initializing closure.

### Trade-off: No Generic Code Over Both

The downside is that there is no way to write a single generic function that works with both `RcLazy` and `ArcLazy` via a shared trait bound. Code written against `RefFunctor` cannot accept `ArcLazy`, and vice versa. This is an inherent limitation of Rust's type system: there is no way to express "this trait's method takes a closure that may or may not need to be `Send`, depending on the implementor." In Haskell, this would be handled by a type class with a constraint parameter; in Rust, the two traits must remain distinct.

## 2. Consistency with the Send Pattern

### Where It Matches SendDeferrable

- **Naming convention:** `SendRefFunctor` / `send_ref_map` mirrors `SendDeferrable` / `send_defer`. Consistent.
- **Free function pattern:** Both provide a free function alongside the trait method. `send_ref_map` dispatches to `Brand::send_ref_map`, just as `send_defer` dispatches to `D::send_defer`.
- **FnOnce usage:** Both use `FnOnce` rather than `Fn`, justified by at-most-once evaluation semantics. Both document why this is the case.
- **Send on the closure, not Sync:** Both require `Send` but not `Sync` on the closure. `SendDeferrable` documents this explicitly (contrasting with `SendCloneableFn` which needs `Send + Sync` for `Fn`). `SendRefFunctor` documents it implicitly through the signature.
- **Module structure:** Both follow the `document_module` / `mod inner` / `pub use inner::*` pattern with tests below.
- **Property-based tests:** Both have QuickCheck tests for their laws.
- **Documentation quality:** Both document laws, rationale, and examples thoroughly.

### Where It Diverges from SendDeferrable

| Aspect | SendDeferrable | SendRefFunctor |
|---|---|---|
| Supertrait | `Deferrable<'a>` | None (standalone) |
| HKT encoding | None (method on `Self`) | `#[kind(type Of<'a, A: 'a>: 'a;)]` |
| Lifetime parameter | On the trait: `SendDeferrable<'a>` | On the method: `fn send_ref_map<'a, ...>` |
| Type parameters on A/B | None (operates on `Self`) | `A: Send + Sync + 'a, B: Send + Sync + 'a` |

All of these divergences are justified by the different natures of the traits:
- `Deferrable` operates on concrete types (`Self`), so the lifetime parameter goes on the trait. `RefFunctor`-family traits operate on type constructors (brands), so the lifetime is on the method and the HKT machinery handles type application.
- The `Send + Sync` bounds on `A` and `B` are necessary because `ArcLazy` stores values inside `Arc<LazyLock<...>>`, which requires `Send + Sync` for its contents.
- The lack of a supertrait is the key divergence, explained above.

## 3. Method Signatures and Bounds

### The `send_ref_map` Signature

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
    func: impl FnOnce(&A) -> B + Send + 'a,
    fa: Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
```

**Bounds analysis:**

- `A: Send + Sync + 'a` and `B: Send + Sync + 'a`: Required because `ArcLazy` wraps `Arc<LazyLock<A, ...>>`, and `Arc<T>` is `Send + Sync` only when `T: Send + Sync`. The `Sync` bound on `A` is specifically needed because the `ArcLazy` may be shared across threads (via `Arc::clone`), and `LazyLock`'s `Deref` impl requires `Sync` on the stored value for the once-cell semantics.
- `func: impl FnOnce(&A) -> B + Send + 'a`: `FnOnce` because the memoized cell calls it at most once. `Send` because the closure is moved into an `Arc<LazyLock<...>>` which may be sent to another thread before evaluation. The `'a` bound ties the closure's lifetime to the lazy value.
- No `Sync` on `func`: Correct. The closure is consumed by `FnOnce`, so it never needs shared references. Only `Send` is needed to move it across thread boundaries.

**Comparison with RefFunctor:**

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl FnOnce(&A) -> B + 'a,
    fa: Apply!(...),
) -> Apply!(...);
```

The only additions in `SendRefFunctor` are `Send + Sync` on `A` and `B`, and `Send` on `func`. This is the minimal set of additional bounds required for thread safety, which is clean.

### The Free Function

```rust
pub fn send_ref_map<'a, Brand: SendRefFunctor, A: Send + Sync + 'a, B: Send + Sync + 'a>(
    func: impl FnOnce(&A) -> B + Send + 'a,
    fa: Apply!(<Brand as Kind!(...)>::Of<'a, A>),
) -> Apply!(<Brand as Kind!(...)>::Of<'a, B>) {
    Brand::send_ref_map(func, fa)
}
```

Straightforward delegation. Matches the pattern used by `ref_map`, `send_defer`, `defer`, and other free functions in the library.

## 4. Documentation Quality

### Strengths

- **Rationale section ("Why a Separate Trait?"):** Clearly explains why `SendRefFunctor` is not a supertrait of `RefFunctor`, with concrete references to `RcLazy` and `ArcLazy`.
- **Laws:** Both identity and composition laws are stated and demonstrated with code examples.
- **Cache chain behavior:** Documents the memory retention chain when chaining `send_ref_map` calls, warning about potential memory accumulation.
- **"Why FnOnce?":** Explains the closure bound choice.
- **Examples:** Module-level example, trait-level law examples, method-level usage example, and free function example. All are self-contained and correct.
- **`#[document_signature]`, `#[document_type_parameters]`, etc.:** Full documentation macro annotations on both the trait method and free function.

### Issues

- **Composition law phrasing inconsistency:** The identity law says `send_ref_map(|x| x.clone(), fa)`, but the composition law says `send_ref_map(|x| f(&g(x)), fa)`. In the identity law, the closure receives `&A` and calls `.clone()` to produce `A`, which assumes `A: Clone`. However, the trait does not require `Clone` on `A`. The law is stated as a conceptual property (if `Clone` were available, identity should hold), but this could be confusing. `RefFunctor` handles this more carefully by noting "given `A: Clone`" in its identity law.
- **Cross-referencing:** The `RefFunctor` docs reference `SendRefFunctor` and explain the independence. The `SendRefFunctor` docs reference `RefFunctor`. This bidirectional linking is good.
- **No explicit mention of `Sync` requirement:** The docs explain `Send` on the closure but do not call out why `A` and `B` need `Sync` (not just `Send`). This is derivable from `Arc` semantics but could be stated for clarity.

## 5. Issues and Limitations

### No Supertrait Relationship (Intentional but Costly)

As discussed, the independence of `RefFunctor` and `SendRefFunctor` is correct but means generic code cannot abstract over both. If a library consumer wants to write a function that accepts either `RcLazy` or `ArcLazy`, they must either:
- Write two versions (one for each trait), or
- Use a custom trait that wraps both, or
- Use inherent methods directly (losing the brand-level abstraction).

### `Send + Sync` Bounds on A and B

The `Send + Sync` bounds on `A` and `B` are required by `Arc`, but they are arguably too strict for some use cases. A `LazyLock<A>` only requires `A: Send` for the `Send` impl and `A: Sync` for the `Sync` impl. Since `ArcLazy` wraps `Arc<LazyLock<A>>`, and `Arc<T>` requires `T: Send + Sync`, both bounds propagate. This is an inherent constraint of using `Arc`, not a design flaw.

### Only Two Implementors

Currently, only `LazyBrand<ArcLazyConfig>` and `TryLazyBrand<E, ArcLazyConfig>` implement `SendRefFunctor`. This makes the trait highly specialized. It exists primarily to give `ArcLazy` some HKT story, since `ArcLazy` cannot implement the regular `Functor` trait (which requires owned `A -> B` mapping, not `&A -> B`).

### Method Name Shadowing with Inherent Methods

`ArcLazy` has an inherent `ref_map` method (lines 879-886 of lazy.rs) that performs the same operation as `SendRefFunctor::send_ref_map` but with different ergonomics (method syntax, no brand parameter). Users working directly with `ArcLazy` will likely use the inherent method. The `SendRefFunctor` trait is primarily useful for brand-level generic programming, which is a narrow use case for memoized types.

### FnOnce Prevents Reuse of the Mapping Function

Using `FnOnce` means the mapping function cannot be shared across multiple `send_ref_map` calls without cloning it. This is correct for the single-evaluation semantics of memoized types, but it means patterns like "map the same function over a collection of lazy values" require the function to be `Clone` or to be re-created for each call. This is documented in the "Why FnOnce?" section and is a deliberate trade-off.

## 6. Alternatives Considered

### Alternative: SendRefFunctor as Supertrait of RefFunctor

```rust
pub trait SendRefFunctor: RefFunctor { ... }
```

This would require `ArcLazy` to implement `RefFunctor`, which is impossible without accepting non-`Send` closures. Ruled out for the reasons documented in the trait.

### Alternative: A Single RefFunctor with a Marker Trait for Send

```rust
pub trait RefFunctor {
    type RequiresSend: SendMarker; // associated type to control bound
    fn ref_map<'a, A: 'a, B: 'a>(...) -> ...;
}
```

This is theoretically possible via GAT-like tricks or conditional bounds, but Rust's type system does not support conditional trait bounds on method parameters based on an associated type. The two-trait approach is the pragmatic solution.

### Alternative: A Generic Trait Parameterized by Send Requirement

```rust
pub trait RefFunctorWith<S: SendBound> { ... }
type RefFunctor = RefFunctorWith<NoSend>;
type SendRefFunctor = RefFunctorWith<RequireSend>;
```

This could theoretically unify the two traits, but Rust has no way to conditionally add `Send` bounds to `impl FnOnce(...)` based on a type parameter. The bound must be syntactically present or absent in the trait definition. This alternative is not feasible in current Rust.

### Alternative: Remove the Trait, Keep Only Inherent Methods

Since the only implementors are lazy types with inherent `ref_map` methods, one could argue the trait is unnecessary. However, the trait provides:
1. A standardized interface documented by laws.
2. Brand-level polymorphism for generic code over `ArcLazy` vs `ArcTryLazy`.
3. A free function `send_ref_map` for consistent style with the rest of the library.

The trait's value is modest given only two implementors, but it maintains consistency with the library's design philosophy.

## 7. Summary

`SendRefFunctor` is a well-designed, narrowly-scoped trait that solves a real problem: giving `ArcLazy` and `ArcTryLazy` a brand-level mapping interface despite the `Send` constraints imposed by `Arc`. The deliberate independence from `RefFunctor` (rather than a supertrait relationship) is the correct choice given Rust's type system constraints, even though it sacrifices generic abstraction over both Rc-based and Arc-based lazy types.

The documentation is thorough, with clear rationale sections, law statements, examples, and caveats about cache chains and `FnOnce` semantics. The method signatures carry the minimal additional bounds (`Send + Sync` on values, `Send` on the closure) required for thread safety.

The main limitation is architectural: the trait has only two implementors and exists primarily to maintain the library's convention that type-level operations are expressed as type class traits with brands, even for types where the HKT story is "partial." This is a reasonable consistency trade-off rather than a design flaw.
