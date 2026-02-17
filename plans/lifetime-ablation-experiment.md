# Lifetime Ablation Experiment

## Background
This experiment explores the feasibility and impact of removing explicit lifetime parameters from the library's Higher-Kinded Type (HKT) simulation. 

The goal is to replace the arity-2 `Kind` trait:
`Kind_cdc7cd43dac7585f` (associated type `type Of<'a, A: 'a>: 'a`)

With a simpler arity-1 `Kind` trait:
`Kind_ad6c20556a82a1f0` (associated type `type Of<A>`)

This change simplifies the internal macro-generated code and the public signatures of HKT-aware traits like `Functor` and `Monad`.

## Current Status
The refactor has been successfully applied to the core type class definitions and several container types. However, a significant architectural hurdle has been encountered with types that wrap closures.

## Summary of Changes Made
- **Trait Redefinitions**: `Functor`, `Applicative`, `Semimonad`, `Foldable`, `Traversable`, `Bifunctor`, `Compactable`, `Filterable`, and `Witherable` have been updated to use the lifetime-free `Kind` traits.
- **Lifetime Removal in Signatures**: Explicit `'a` lifetimes and `A: 'a` bounds have been removed from the methods of the above traits.
- **Container Type Migrations**: The following types have been updated to the new model:
    - `Identity`, `Option`, `Vec`, `Result`, `Tuple1`, `Pair`, `Tuple2`, `CatList`, `String`.
- **Computation Type Migrations**: `Thunk`, `TryThunk`, `Lazy`, `TryLazy`, `Free`, `Trampoline`, `TryTrampoline`, `Step`.
    - These types now use a lifetime-free `Kind` and are forced to `'static` for HKT compatibility.
    - **Update**: `Step` has been fully migrated, including its `Foldable` and `ParFoldable` implementations.
- **HKT Branding Updates**: `impl_kind!` blocks for these types now use `type Of<A>` or `type Of<A, B>`.
- **Optics Refactor**: `Optic`, `Lens`, and `LensPrime` have been updated to use the lifetime-free profunctor brand.

## Files Edited
- `fp-library/src/classes/*.rs` (All trait definitions, including `Function`, `Profunctor`, `Category`, etc.)
- `fp-library/src/types/identity.rs`
- `fp-library/src/types/option.rs`
- `fp-library/src/types/vec.rs`
- `fp-library/src/types/result.rs`
- `fp-library/src/types/tuple_1.rs`
- `fp-library/src/types/pair.rs`
- `fp-library/src/types/tuple_2.rs`
- `fp-library/src/types/cat_list.rs`
- `fp-library/src/types/string.rs`
- `fp-library/src/types/arc_ptr.rs`
- `fp-library/src/types/rc_ptr.rs`
- `fp-library/src/types/endofunction.rs`
- `fp-library/src/types/send_endofunction.rs`
- `fp-library/src/types/endomorphism.rs`
- `fp-library/src/types/fn_brand.rs`
- `fp-library/src/types/thunk.rs`
- `fp-library/src/types/try_thunk.rs`
- `fp-library/src/types/lazy.rs`
- `fp-library/src/types/try_lazy.rs`
- `fp-library/src/types/free.rs`
- `fp-library/src/types/trampoline.rs`
- `fp-library/src/types/try_trampoline.rs`
- `fp-library/src/types/step.rs`
- `fp-library/src/types/optics.rs`

## Analyses and Findings

### 1. Successful Ablation for "Passive" Containers
For types that simply hold data (`Vec<A>`, `Option<A>`, etc.), ablation is highly successful. It results in much cleaner code and simplifies HKT integration. The loss of lifetime-parameterized application (e.g., `Option<&'a T>`) via HKT traits is a minor regression compared to the gain in simplicity.

### 2. The "Closure Hurdle"
The experiment reached an "inflection point" when attempting to migrate `Thunk<'a, A>`, `Lazy<'a, A>`, `Free<F, A>`, and especially the function-related traits like `Function` and `Profunctor`. These types wrap closures that may capture borrowed data.

**Key Finding**: Types that store `Box<dyn Fn... + 'a>` are fundamentally higher-arity in Rust. They require *both* a lifetime parameter and a type parameter to be safe and flexible.

**The Conflict**:
- The ablated `Functor` trait requires `ThunkBrand::Of<A>` to resolve to a concrete type.
- Since the trait provides no lifetime, we must fix it to `'static` in the `impl_kind!`.
- This prevents `map` from working with closures that capture local variables, as they would produce `Thunk<'a, B>` instead of the required `Thunk<'static, B>`.
- Similarly, for `Profunctor` and `Arrow`, removing the lifetime from `compose` and `arrow` forces the input closures to be `'static`.

### 3. Impact on Pointer Abstractions
Ablating lifetimes in `UnsizedCoercible` and `SendUnsizedCoercible` forces all coerced closures to be `'static`. While this simplifies the traits, it prevents the use of these pointers for temporary closures that borrow from the stack, significantly limiting the library's utility in non-long-lived scenarios.

### 4. Cascade of Incompatibilities
The transition to a lifetime-free `Kind` model for category-theoretic traits (`Semigroupoid`, `Category`, `Arrow`) created a massive wave of compilation errors (over 500 detected during the experiment).
- `Endomorphism` and `Endofunction` became unusable with non-`'static` closures.
- `Trampoline` and `TryTrampoline`, which are built on `Free` and `Thunk`, became disconnected from the HKT system because their underlying brands no longer matched the expected trait signatures.
- Many traits like `Arrow` and `Profunctor` had their internal logic and default implementations broken because they could no longer express the necessary lifetime relationships between input and output.

## Possible Next Steps

1.  **Strict 'static Policy**: 
    Force all HKT-compatible computations to be `'static`. This preserves the simplicity of the ablated model but restricts the library to "owned" data or `'static` references.
2.  **Hybrid Model**:
    Maintain two versions of HKT traits or two versions of "closure" types (e.g., `Thunk<A>` for `'static` HKT use and `BorrowedThunk<'a, A>` for non-HKT use).
3.  **Partial Ablation**:
    Acknowledge that `Kind<A>` and `Kind<'a, A>` are both necessary. The experiment would conclude that ablation is beneficial for containers but harmful for computation types.
4.  **Lifetime Erasure (Unsafe)**:
    Use `unsafe` to cast lifetimes. This is deemed unacceptable for a library prioritizing safety.

## Conclusion (Postponed)
The experiment has demonstrated that while lifetime-free HKTs are ideal for data containers, they struggle to represent the full power of Rust's borrowing system in the context of deferred execution and category-theoretic abstractions.

A complete migration to a lifetime-free `Kind` model has been performed across the `types/` directory. This involved:
1.  **Forcing `'static`**: Computation types like `Thunk`, `Lazy`, and `Free` now implement the lifetime-free `Kind` by fixing their internal lifetime to `'static`.
2.  **Constraint Propagation**: Forcing `'static` at the HKT level cascaded into many trait implementations (e.g., `Functor`, `MonadRec`, `Profunctor`), requiring them to add `'static` bounds to their generic parameters and closure types.
3.  **Optics Impact**: Optics like `Lens` and `LensPrime` now require `'static` data and functions to be compatible with the new profunctor branding.

**Finding**: While the ablation results in a significantly simpler API for the user (no turbofish lifetimes, cleaner signatures), it comes at the cost of **losing all support for non-static borrowing** in HKT-aware code. This is a severe limitation for a library intended for general-purpose Rust development.

**Current State**: The library is in a non-compiling state with approximately 140 errors remaining. These errors primarily consist of:
- `E0310`: Generic parameters (like `A`, `B`, `Func`) need explicit `'static` bounds because they are now used in contexts (like `Thunk<'static, A>`) that require them to be valid for `'static`.
- `E0053`: Trait implementation signatures no longer match the trait definitions because of missing or extra `'static` bounds.
- `E0195`: Lifetime parameters on associated functions (like `map`) no longer match the updated (lifetime-free) trait declarations.

The volume of errors and the architectural compromise (forcing everything to `'static`) suggests that a pure arity-1 HKT model, while elegant in languages with garbage collection, creates significant friction with Rust's ownership and borrowing model.
