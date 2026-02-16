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
- **HKT Branding Updates**: `impl_kind!` blocks for these types now use `type Of<A>` or `type Of<A, B>`.

## Files Edited
- `fp-library/src/classes/*.rs` (All trait definitions)
- `fp-library/src/types/identity.rs`
- `fp-library/src/types/option.rs`
- `fp-library/src/types/vec.rs`
- `fp-library/src/types/result.rs`
- `fp-library/src/types/tuple_1.rs`
- `fp-library/src/types/pair.rs`
- `fp-library/src/types/tuple_2.rs`
- `fp-library/src/types/cat_list.rs`
- `fp-library/src/types/string.rs`

## Analyses and Findings

### 1. Successful Ablation for "Passive" Containers
For types that simply hold data (`Vec<A>`, `Option<A>`, etc.), ablation is highly successful. It results in much cleaner code and simplifies HKT integration. The loss of lifetime-parameterized application (e.g., `Option<&'a T>`) via HKT traits is a minor regression compared to the gain in simplicity.

### 2. The "Closure Hurdle"
The experiment reached an "inflection point" when attempting to migrate `Thunk<'a, A>`, `Lazy<'a, A>`, and `Free<F, A>`. These types wrap closures that may capture borrowed data.

**Key Finding**: Types that store `Box<dyn Fn... + 'a>` are fundamentally higher-arity in Rust. They require *both* a lifetime parameter and a type parameter to be safe and flexible.

**The Conflict**:
- The ablated `Functor` trait requires `ThunkBrand::Of<A>` to resolve to a concrete type.
- Since the trait provides no lifetime, we must fix it to `'static` in the `impl_kind!`.
- This prevents `map` from working with closures that capture local variables, as they would produce `Thunk<'a, B>` instead of the required `Thunk<'static, B>`.

## Possible Next Steps

1.  **Strict 'static Policy**: 
    Force all HKT-compatible computations to be `'static`. This preserves the simplicity of the ablated model but restricts the library to "owned" data or `'static` references.
2.  **Hybrid Model**:
    Maintain two versions of HKT traits or two versions of "closure" types (e.g., `Thunk<A>` for `'static` HKT use and `BorrowedThunk<'a, A>` for non-HKT use).
3.  **Partial Ablation**:
    Acknowledge that `Kind<A>` and `Kind<'a, A>` are both necessary. The experiment would conclude that ablation is beneficial for containers but harmful for computation types.
4.  **Lifetime Erasure (Unsafe)**:
    Use `unsafe` to cast lifetimes. This is deemed unacceptable for a library prioritizing safety.

## Conclusion (Work in Progress)
The experiment has demonstrated that while lifetime-free HKTs are ideal for data containers, they struggle to represent the full power of Rust's borrowing system in the context of deferred execution.
