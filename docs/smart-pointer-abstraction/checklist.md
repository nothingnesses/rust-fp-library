# Smart Pointer Abstraction Implementation Checklist

This checklist tracks progress on implementing the `SmartPointer` type class and `SharedLazy` type. See [plan.md](./plan.md) for full context and design details.

---

## Phase 1: SmartPointer Foundation

### 1.1 Trait Definition

- [ ] Create `fp-library/src/classes/smart_pointer.rs`
- [ ] Define `SmartPointer` trait with:
  - [ ] Associated type `Of<T: ?Sized>: Clone + Deref<Target = T>`
  - [ ] Method `fn new<T>(value: T) -> Self::Of<T>`
- [ ] Define `SendSmartPointer` marker trait with appropriate bounds
- [ ] Add comprehensive documentation following `docs/architecture.md` standards
- [ ] Add module-level examples

### 1.2 Brand Definitions

- [ ] Add `RcBrand` struct to `fp-library/src/brands.rs`
- [ ] Add `ArcBrand` struct to `fp-library/src/brands.rs`
- [ ] Add documentation for both brands explaining their use cases

### 1.3 Rc Implementation

- [ ] Create `fp-library/src/types/rc_ptr.rs`
- [ ] Implement `SmartPointer` for `RcBrand`
  - [ ] `type Of<T: ?Sized> = Rc<T>`
  - [ ] `fn new<T>(value: T) -> Rc<T>` using `Rc::new`
- [ ] Add `impl_kind!` for `RcBrand` if needed
- [ ] Add documentation and examples
- [ ] Verify `RcBrand` does NOT implement `SendSmartPointer`

### 1.4 Arc Implementation

- [ ] Create `fp-library/src/types/arc_ptr.rs`
- [ ] Implement `SmartPointer` for `ArcBrand`
  - [ ] `type Of<T: ?Sized> = Arc<T>`
  - [ ] `fn new<T>(value: T) -> Arc<T>` using `Arc::new`
- [ ] Implement `SendSmartPointer` for `ArcBrand`
- [ ] Add `impl_kind!` for `ArcBrand` if needed
- [ ] Add documentation and examples

### 1.5 Free Functions & Re-exports

- [ ] Add free function `smart_pointer_new` to `fp-library/src/classes/smart_pointer.rs`
- [ ] Update `fp-library/src/classes.rs` to re-export `smart_pointer` module
- [ ] Update `fp-library/src/functions.rs` to re-export `smart_pointer_new` (prefixed)
- [ ] Update `fp-library/src/types.rs` to re-export `rc_ptr` and `arc_ptr` modules

### 1.6 Phase 1 Tests

- [ ] Unit test: `RcBrand::new` creates `Rc<T>`
- [ ] Unit test: `ArcBrand::new` creates `Arc<T>`
- [ ] Unit test: `Clone` works for `RcBrand::Of<T>`
- [ ] Unit test: `Clone` works for `ArcBrand::Of<T>`
- [ ] Unit test: `Deref` works correctly for both
- [ ] Compile-fail test: `RcBrand::Of<T>` is `!Send`
- [ ] Compile-success test: `ArcBrand::Of<T: Send + Sync>` is `Send + Sync`

---

## Phase 2: SharedLazy Type

### 2.1 Core Structure

- [ ] Create `fp-library/src/types/shared_lazy.rs`
- [ ] Define `SharedLazy<'a, PtrBrand, OnceBrand, FnBrand, A>` struct
  - [ ] Contains `<PtrBrand as SmartPointer>::Of<(OnceCell, Thunk)>`
- [ ] Add appropriate trait bounds on type definition
- [ ] Add comprehensive struct-level documentation

### 2.2 Core Implementation

- [ ] Implement `SharedLazy::new(thunk)` method
  - [ ] Creates new OnceCell
  - [ ] Wraps (OnceCell, thunk) in SmartPointer
- [ ] Implement `SharedLazy::force(&self)` method
  - [ ] Takes `&self` (unlike `Lazy::force` which takes `Self`)
  - [ ] Dereferences smart pointer
  - [ ] Uses `OnceCell::get_or_init`
  - [ ] Returns cloned value
- [ ] Add documentation with type signatures for all methods

### 2.3 Clone Implementation

- [ ] Implement `Clone` for `SharedLazy`
- [ ] Clone must be cheap (just smart pointer clone)
- [ ] Verify clones share memoization state (via test)

### 2.4 Type Class Implementations

- [ ] Implement `Semigroup` for `SharedLazy` where `A: Semigroup + Clone`
  - [ ] `append` creates new `SharedLazy` that forces both and appends
- [ ] Implement `Monoid` for `SharedLazy` where `A: Monoid + Clone`
  - [ ] `empty` creates `SharedLazy` returning `Monoid::empty()`
- [ ] Implement `Defer` for `SharedLazy`
  - [ ] `defer(f)` creates `SharedLazy` that calls f, then forces result

### 2.5 Kind Implementation

- [ ] Add `SharedLazyBrand<PtrBrand, OnceBrand, FnBrand>` to `fp-library/src/brands.rs`
- [ ] Add `impl_kind!` for `SharedLazyBrand`
- [ ] Verify kind signature matches expected HKT pattern

### 2.6 Free Functions

- [ ] Add `shared_lazy_new` free function
- [ ] Add `shared_lazy_force` free function
- [ ] Re-export in `fp-library/src/functions.rs`

### 2.7 Type Aliases (Convenience)

- [ ] Add `RcLazy<A>` as alias for `SharedLazy<'static, RcBrand, OnceCellBrand, RcFnBrand, A>`
- [ ] Add `ArcLazy<A>` as alias for `SharedLazy<'static, ArcBrand, OnceLockBrand, ArcFnBrand, A>`
- [ ] Document thread-safety characteristics of each alias

### 2.8 Phase 2 Tests

- [ ] Unit test: `SharedLazy::new` + `SharedLazy::force` returns correct value
- [ ] Unit test: Thunk is only called once (counter test)
- [ ] Unit test: Clones share memoization (key differentiator from `Lazy`)
- [ ] Unit test: `Semigroup::append` works correctly
- [ ] Unit test: `Monoid::empty` works correctly
- [ ] Unit test: `Defer::defer` works correctly
- [ ] Property test: Semigroup associativity law
- [ ] Property test: Monoid identity laws
- [ ] Compile-fail test: `RcLazy` is `!Send`
- [ ] Compile-success test: `ArcLazy` is `Send + Sync` when `A: Send + Sync`

---

## Phase 3: Integration & Polish

### 3.1 Documentation

- [ ] Add comparison section in `shared_lazy.rs` explaining difference from `Lazy`
- [ ] Add "when to use which" guidance
- [ ] Ensure all public items have examples
- [ ] Doc tests pass for all examples

### 3.2 Module Structure Updates

- [ ] Update `fp-library/src/types.rs` to include `shared_lazy`
- [ ] Update `fp-library/src/brands.rs` with all new brands
- [ ] Verify no circular dependencies introduced

### 3.3 Documentation Files

- [ ] Update `docs/std-coverage-checklist.md` with:
  - [ ] `SmartPointer` entry in Type Classes table
  - [ ] `RcBrand` entry in Data Types table
  - [ ] `ArcBrand` entry in Data Types table
  - [ ] `SharedLazyBrand` entry in Data Types table
- [ ] Update `docs/architecture.md` if SmartPointer introduces new patterns
- [ ] Update `docs/todo.md` to mark Lazy memoization item as addressed

### 3.4 Final Verification

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `cargo doc` builds without warnings
- [ ] Review generated documentation for clarity

---

## Phase 4: Optional ClonableFn Refactor

> **Note**: This phase is optional and can be deferred. The SmartPointer abstraction provides value even without this refactoring.

### 4.1 Investigation

- [ ] Investigate CoerceUnsized for unsized coercion
- [ ] Test if `Rc::new(f)` → `Rc<dyn Fn>` can be abstracted
- [ ] Determine if specialization would help
- [ ] Document findings in this checklist

### 4.2 Implementation (if viable)

- [ ] Create `GenericFnBrand<PtrBrand>` struct
- [ ] Implement `Function` for `GenericFnBrand<PtrBrand>`
- [ ] Implement `ClonableFn` for `GenericFnBrand<PtrBrand>`
- [ ] Create type aliases:
  - [ ] `type RcFnBrand = GenericFnBrand<RcBrand>`
  - [ ] `type ArcFnBrand = GenericFnBrand<ArcBrand>`
- [ ] Ensure backward compatibility (existing code compiles)
- [ ] Update `Semigroupoid` implementation
- [ ] Update `Category` implementation

### 4.3 Phase 4 Tests

- [ ] All existing `RcFnBrand` tests pass with alias
- [ ] All existing `ArcFnBrand` tests pass with alias
- [ ] No regression in benchmark performance

---

## Release Preparation

### Changelog

- [ ] Add entry to `fp-library/CHANGELOG.md` under `[Unreleased]`:
  ```markdown
  ### Added
  - `SmartPointer` type class for abstracting over `Rc` and `Arc`
  - `RcBrand` and `ArcBrand` brands implementing `SmartPointer`
  - `SendSmartPointer` marker trait for thread-safe smart pointers
  - `SharedLazy` type with Haskell-like shared memoization semantics
  - `RcLazy` and `ArcLazy` type aliases for common configurations
  - Free functions: `smart_pointer_new`, `shared_lazy_new`, `shared_lazy_force`
  ```

### Optional: Macros Changelog

- [ ] If `fp-macros` changes required, update `fp-macros/CHANGELOG.md`

---

## Open Questions

Track open questions and decisions here:

| Question | Status | Decision |
|----------|--------|----------|
| Should `RcBrand`/`ArcBrand` have different names to avoid confusion with `RcFnBrand`/`ArcFnBrand`? | Open | Consider `RcPtrBrand`/`ArcPtrBrand` |
| Should Phase 4 (ClonableFn refactor) be done at all? | Open | Depends on CoerceUnsized investigation |
| Should `SharedLazy` be the only lazy type, or keep both? | Decided | Keep both - different use cases |
| Feature flag for new functionality? | Open | Probably not needed - additive change |

---

## Notes for Implementors

### Key Files to Create

1. `fp-library/src/classes/smart_pointer.rs` - New type class
2. `fp-library/src/types/rc_ptr.rs` - Rc implementation
3. `fp-library/src/types/arc_ptr.rs` - Arc implementation  
4. `fp-library/src/types/shared_lazy.rs` - SharedLazy type

### Key Files to Modify

1. `fp-library/src/brands.rs` - Add new brands
2. `fp-library/src/classes.rs` - Re-export smart_pointer
3. `fp-library/src/types.rs` - Re-export new type modules
4. `fp-library/src/functions.rs` - Re-export free functions

### Testing Strategy

1. **Unit tests**: In each module's `#[cfg(test)]` block
2. **Property tests**: In `fp-library/tests/property_tests.rs`
3. **Compile-fail tests**: In `fp-library/tests/ui/` directory
4. **Doc tests**: In documentation examples

### Reference Implementations

- Look at `lazy.rs` for structure of Lazy type
- Look at `rc_fn.rs` / `arc_fn.rs` for brand implementation pattern
- Look at `clonable_fn.rs` for type class pattern
- Look at `once.rs` for Once type class pattern
