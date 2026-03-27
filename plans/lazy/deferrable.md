# Analysis: `Deferrable` Trait

**File:** `fp-library/src/classes/deferrable.rs`

## 1. Design: Faithfulness to PureScript's `Lazy`

PureScript's `Control.Lazy` (lines 10-11 of the reference):

```purescript
class Lazy l where
  defer :: (Unit -> l) -> l
```

The Rust `Deferrable` trait (line 62-84 of `deferrable.rs`):

```rust
pub trait Deferrable<'a> {
    fn defer(f: impl FnOnce() -> Self + 'a) -> Self
    where
        Self: Sized;
}
```

### Correspondences

| PureScript | Rust | Notes |
|---|---|---|
| `class Lazy l` | `trait Deferrable<'a>` | Lifetime parameter added for Rust's ownership model. |
| `defer :: (Unit -> l) -> l` | `fn defer(f: impl FnOnce() -> Self + 'a) -> Self` | `Unit -> l` maps to `FnOnce() -> Self`. |
| `fix :: Lazy l => (l -> l) -> l` | `rc_lazy_fix` / `arc_lazy_fix` (concrete functions) | Cannot be generic; see below. |

### Key differences

- **Lifetime parameter `'a`:** PureScript's `Lazy` has no lifetime. Rust requires `'a` to bound how long the captured closure lives. This is a necessary adaptation. `Trampoline` and `Free<ThunkBrand, _>` implement `Deferrable<'static>` because their internal thunks require `'static`.

- **`FnOnce` vs `Unit ->`:** PureScript uses `Unit -> l` (a regular function). Rust's `FnOnce() -> Self` is the direct equivalent. Using `FnOnce` (not `Fn`) is correct since the thunk is consumed at most once.

- **`fix` is not generic:** The documentation (lines 26-37) explains this well. In PureScript, `fix` works because all values are lazily evaluated and can self-reference through thunks. In Rust, self-reference requires `Rc`/`Arc` + interior mutability, which is specific to `Lazy`, not all `Deferrable` types. The decision to provide `rc_lazy_fix` and `arc_lazy_fix` as concrete functions rather than forcing a `fix` method on the trait is sound.

- **Renamed from `Lazy` to `Deferrable`:** Good choice. `Lazy` is already used as a type name (`Lazy<'a, A, Config>`), so using the same name for the trait would cause confusion. "Deferrable" communicates the intent clearly.

- **No `Lazy (a -> b)` instance:** PureScript provides `instance lazyFn :: Lazy (a -> b) where defer f = \x -> f unit x`. There is no equivalent Rust impl for function types. This is reasonable since Rust functions are not lazily evaluated by default, and a blanket impl for `Fn` would conflict with the ownership model. This instance is mainly useful in PureScript for tying the knot with function values, which is not ergonomic in Rust anyway.

### Assessment

The adaptation is faithful and appropriate. The lifetime parameter, `FnOnce` choice, and exclusion of generic `fix` are all well-motivated by Rust's ownership model.

## 2. Implementation Quality

### The trait itself (lines 62-84)

The trait definition is clean and minimal. Two observations:

- **`where Self: Sized` bound:** Required because `impl FnOnce()` arguments use static dispatch. This prevents `dyn Deferrable<'a>` usage, but that is acceptable since deferred construction is inherently a concrete-type operation.

- **`+ 'a` on the closure:** Correctly ties the closure's lifetime to the trait's lifetime parameter, ensuring captured references do not outlive the deferred value.

### Implementor behavior varies significantly

The implementations fall into three categories:

1. **Truly deferred:** `Thunk`, `TryThunk`, `Trampoline`, `TryTrampoline`, `Free<ThunkBrand, _>`, `RcLazy`, `RcTryLazy` all wrap the closure and defer evaluation. These satisfy the spirit of `Deferrable`.

2. **Eagerly evaluated (due to `Send` gap):** `SendThunk` (line 400-426 of `send_thunk.rs`), `TrySendThunk` (line 842-872 of `try_send_thunk.rs`), `ArcLazy` (line 854-888 of `lazy.rs`), and `ArcTryLazy` (line 1319-1354 of `try_lazy.rs`) all call `f()` immediately in their `defer` implementation. This is because `Deferrable::defer` does not require `Send` on the closure, but these types need `Send` closures internally.

   This is a significant semantic issue: calling `defer(|| expensive())` on these types evaluates `expensive()` immediately, which violates the expected deferred-evaluation semantics. The transparency law (`defer(|| x) == x`) is technically satisfied since the value is the same, but the *purpose* of deferral (delaying computation) is lost.

   The documentation at lines 39-45 warns about this for `ArcLazy`, which is good. However, the warning is on the `Deferrable` trait docs, not on the individual `SendThunk`/`TrySendThunk` impls, where users are more likely to encounter it.

### Potential correctness issue: `RcLazy` flattening

The `RcLazy` impl (lines 819-851 of `lazy.rs`):

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    RcLazy::new(move || f().evaluate().clone())
}
```

This creates a new `RcLazy` that, when evaluated, evaluates the inner `RcLazy` and clones its value. The `Clone` bound on `A` is required for this flattening. This is semantically `join` (monadic flattening) rather than simple deferral. A simpler implementation could have been:

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    RcLazy::new(move || f().evaluate().clone())
}
```

This is actually the only option given `RcLazy`'s API (it stores `A`, not `RcLazy<A>`), so the flattening is a necessary consequence. The `Clone` bound is the price.

## 3. Laws

### Stated law: Transparency (line 23-24)

> The value produced by `defer(|| x)` is identical to `x`. This law does not constrain *when* evaluation occurs.

This is the only law stated, and it corresponds to PureScript's implied semantics. However:

### Missing laws and considerations

1. **Idempotency / nesting:** `defer(|| defer(|| x))` should be observationally equivalent to `defer(|| x)`. This is implied by transparency but worth stating explicitly, especially since some impls (e.g., `RcLazy`) do non-trivial flattening.

2. **No side-effect duplication:** For memoized types (`Lazy`), `defer` should not cause a side effect in the thunk to execute more than once. This is an implementation property, not a law per se, but it is worth documenting.

3. **PureScript has no explicit laws either:** The PureScript `Lazy` class does not state formal laws beyond the implicit expectation that `defer` is transparent. So the Rust version is at parity.

### Law testing

There are no dedicated law tests for `Deferrable` in the test suite. The doc examples demonstrate transparency but do not use property-based testing (QuickCheck). Other type classes in the library (e.g., `Functor`, `Foldable`) have QuickCheck law tests. `Deferrable` should have them too, at minimum for:

- Transparency: `defer(|| pure(x)).evaluate() == pure(x).evaluate()` for each implementor.
- Nesting: `defer(|| defer(|| pure(x))).evaluate() == pure(x).evaluate()`.

## 4. API Surface

### Current API

- `Deferrable::defer` (trait method)
- `defer` (free function, line 111-113)
- `SendDeferrable::send_defer` (separate trait, `send_deferrable.rs`)
- `send_defer` (free function)
- `rc_lazy_fix` / `arc_lazy_fix` (concrete fixed-point combinators)

### Assessment

The API is minimal and well-designed. Each piece has a clear role.

### Possible additions

1. **`defer_with` or `delay`:** A convenience that takes `impl FnOnce() -> A` instead of `impl FnOnce() -> Self`. Currently, users must write `defer(|| Thunk::new(|| 42))` or `defer(|| Thunk::pure(42))`. A method like `delay(|| 42)` that combines `new`/`pure` with `defer` would reduce boilerplate. However, this conflates two distinct operations (construction and deferral), so omitting it is defensible.

2. **`force` counterpart:** PureScript has no `force` in the `Lazy` class, so its absence here is consistent. The library uses `evaluate()` as an inherent method on each type instead, which is idiomatic Rust. Adding `force` to `Deferrable` would require an associated output type, adding complexity. The `Evaluable` trait already serves this role.

## 5. Consistency with Other Type Classes

### Positive consistency

- **Module structure:** `deferrable.rs` follows the same `#[fp_macros::document_module] mod inner { ... } pub use inner::*;` pattern as `functor.rs`, `foldable.rs`, etc.
- **Free function pattern:** The `defer` free function mirrors `map`, `pure`, `bind`, etc.
- **Documentation macros:** Uses `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]` consistently.
- **Supertrait pattern:** `SendDeferrable: Deferrable` follows the same pattern as `SendCloneableFn: CloneableFn`.

### Inconsistencies

1. **Not HKT-based:** Most type classes in the library (`Functor`, `Monad`, etc.) are implemented on Brand types and use `Kind` machinery. `Deferrable` is implemented directly on concrete types (`Thunk<'a, A>`, `Lazy<'a, A, Config>`, etc.), not on brands. This is necessary because `Deferrable` does not involve mapping over a type parameter; it is about constructing a value from a thunk. The trait's signature `fn defer(f: ...) -> Self` has no `Kind` types. This is correct but worth noting as a design difference.

2. **Lifetime parameter on the trait:** Most type classes do not have a lifetime parameter; they use `for<'a>` bounds within their methods. `Deferrable<'a>` puts the lifetime on the trait itself, which means each impl specifies a concrete lifetime (e.g., `impl<'a, A: 'a> Deferrable<'a> for Thunk<'a, A>`). This is forced by the `+ 'a` bound on the closure needing to match the type's own lifetime. It is the right choice but makes `Deferrable` feel different from other traits in the hierarchy.

## 6. Limitations

1. **Eager evaluation for `Send` types:** As discussed in section 2, `SendThunk`, `TrySendThunk`, `ArcLazy`, and `ArcTryLazy` evaluate eagerly under `Deferrable::defer`. Users who want true deferral with thread-safe types must use `SendDeferrable::send_defer` instead. This is a footgun that the documentation mitigates but does not eliminate.

2. **`Clone` bound on `Lazy` impls:** `RcLazy`'s `Deferrable` impl requires `A: Clone` (line 821). `ArcLazy`'s `SendDeferrable` impl requires `A: Clone + Send + Sync` (line 897). These bounds come from the flattening (`f().evaluate().clone()`). Types that are expensive to clone pay a hidden cost.

3. **No `Deferrable` for `Option`, `Vec`, etc.:** The trait is only implemented for lazy evaluation types. In PureScript, `Lazy` can be implemented for any type where deferral makes sense. In Rust, deferral only makes sense for types that wrap closures, so the narrow scope is appropriate.

4. **`Sized` bound prevents trait objects:** `where Self: Sized` on `defer` means you cannot call `defer` through a `dyn Deferrable<'a>` reference. This is fine in practice since `defer` is a constructor (no `self` parameter), so trait objects would not help.

5. **No blanket impl for newtypes:** If you create a newtype `struct MyThunk(Thunk<'a, A>)`, you must manually implement `Deferrable`. There is no derive macro or blanket impl for delegating. This is a minor ergonomic limitation.

## 7. Documentation

### Strengths

- The trait-level docs (lines 18-61) are thorough. They explain the law, the rationale for omitting `fix`, and warn about eager evaluation in `Send` types.
- Cross-references to `Lazy`, `Thunk`, `rc_lazy_fix`, `arc_lazy_fix`, and `SendDeferrable` are all present and linked.
- Doc examples compile and demonstrate the transparency law.

### Issues

1. **Module-level docs (lines 1-13) are minimal:** The module doc just says "Types that can be constructed lazily from a computation" with a single example. Compare with `functor.rs` which has a richer module-level explanation. A sentence or two about when to use `Deferrable` vs constructing types directly would help.

2. **Warning placement (lines 39-45):** The warning about eager evaluation for `ArcLazy` is on the `Deferrable` trait itself. It would be more discoverable on the individual impl docs for `SendThunk` and `TrySendThunk` as well. The `SendThunk` impl (line 403-404 of `send_thunk.rs`) does mention it in a brief comment, but the `ArcLazy` impl (line 860-864 of `lazy.rs`) is more thorough.

3. **The law name "Transparency" is non-standard:** PureScript does not name this law. Other functional programming literature sometimes calls it "identity" or "delay-force identity." The name "Transparency" is fine but could benefit from a brief explanation of why it is called that (the deferral should be transparent to the consumer).

4. **Missing doc for the `'a` lifetime parameter:** The `#[document_type_parameters]` on line 46 says "The lifetime of the computation." This is accurate but could be more precise: it is the lifetime of the closure captured by `defer`, which bounds how long the deferred value can live.

## Summary

`Deferrable` is a clean, well-motivated adaptation of PureScript's `Lazy` class. The core design is sound: the lifetime parameter, `FnOnce` usage, and exclusion of generic `fix` are all correct for Rust. The main concern is that several implementations (`SendThunk`, `TrySendThunk`, `ArcLazy`, `ArcTryLazy`) evaluate eagerly, which undermines the trait's stated purpose. This is documented but remains a potential source of confusion. Adding QuickCheck-based law tests and slightly enriching the module-level documentation would improve the trait's standing in the library.
