# Lifetime Ablation Experiment

## Background
This experiment explores the feasibility and impact of removing explicit lifetime parameters from the library's Higher-Kinded Type (HKT) simulation. 

The goal is to replace `Kind` traits containing lifetime constraints, e.g.:
`Kind_cdc7cd43dac7585f` (associated type `type Of<'a, A: 'a>: 'a`)

With a simpler `Kind` trait not containing lifetime constraints:
`Kind_ad6c20556a82a1f0` (associated type `type Of<A>`)

This change simplifies the internal macro-generated code and the public signatures of HKT-aware traits like `Functor` and `Monad`.

## Current Status
The experiment has been expanded to cover almost all types and traits in the library. All explicit lifetime parameters (`'a`) and `'static` constraints have been removed from the core data structures and trait implementations in `types/` and `classes/`.

The project is currently in a non-compiling state. This is expected, as the removal of lifetimes from types that wrap closures (like `Thunk`, `Lazy`, `Free`) forces those closures to be `'static`.

## Summary of Changes Made
- **Trait Redefinitions**: Core traits (`Functor`, `Applicative`, `Monad`, etc.) were previously updated to use lifetime-free `Kind` traits.
- **Data Type Migrations**: The following types have had their lifetime parameters and `'static` bounds removed from their definitions and implementations:
    - `Thunk`, `Lazy`, `TryThunk`, `TryLazy`, `Step`, `Trampoline`, `Free`.
    - `Pair`, `Tuple2`, `Result` (and their respective brands).
    - `Optics` (`Optic`, `Lens`, `LensPrime`, `Composed`).
    - `FnBrand`, `ArcBrand`, `RcBrand`.
- **HKT Branding Updates**: `impl_kind!` blocks and `Apply!` macro calls have been simplified to use the `type Of<A>` or `type Of<A, B>` model without lifetimes.

## Files Edited
- `fp-library/src/classes/*.rs` (All trait definitions)
- `fp-library/src/types/thunk.rs`
- `fp-library/src/types/lazy.rs`
- `fp-library/src/types/try_thunk.rs`
- `fp-library/src/types/try_lazy.rs`
- `fp-library/src/types/step.rs`
- `fp-library/src/types/trampoline.rs`
- `fp-library/src/types/free.rs`
- `fp-library/src/types/pair.rs`
- `fp-library/src/types/tuple_2.rs`
- `fp-library/src/types/result.rs`
- `fp-library/src/types/optics.rs`
- `fp-library/src/types/fn_brand.rs`
- `fp-library/src/types/arc_ptr.rs`
- `fp-library/src/types/rc_ptr.rs`

## Analyses and Findings

### 1. Simplified HKT Model
The removal of lifetimes significantly simplifies the `Kind` trait and all downstream traits (`Functor`, `Monad`, etc.). The code is much cleaner and easier to read.

### 2. The "Closure Hurdle" (Confirmed)
Removing lifetimes from types that store trait objects (e.g., `Box<dyn Fn()>`) makes them implicitly `'static`. 
- **Pros**: Matches the model of GC-based functional languages; simplifies type signatures.
- **Cons**: Prevents these types from capturing local, non-static data. This is a significant restriction for a Rust library.

## Instructions for Next Agent

### 1. Review and Clean Up Documentation
Documentation and comments in the modified files still contain references to lifetimes and `'static` requirements (e.g., in `lib.rs`, `thunk.rs`, `free.rs`). These should be updated to reflect the new lifetime-free model.

### 2. Resolve Compilation Errors
Run `cargo check -p fp-library`. Most errors fall into these categories:
- **`'static` Requirement**: Closures passed to `Thunk::new`, `Lazy::new`, etc., now need to be `'static`. You may need to add `+ 'static` bounds to some generic functions or examples.
- **Trait Implementation Mismatches**: Some manual trait implementations might still have residual lifetime parameters or use the wrong `Kind` trait ID.
- **`Apply!` Macro Inconsistency**: Ensure all `Apply!` calls are using the ablated signature.

### 3. Verify Remaining Types
Double-check simple container types like `Identity`, `Option`, `Vec`, `String`, `Tuple1` to ensure their implementations are fully consistent with the ablated model (though most are already done).

### 4. Categorical Optics
The `optics.rs` file was heavily modified. Pay special attention to how `evaluate` and the lens constructors work without lifetimes.

## Conclusion (Postponed)
The structural ablation is complete. The next phase is to achieve compilation and evaluate whether the resulting `'static`-only limitation for closure-based types is an acceptable trade-off for the increased simplicity.
