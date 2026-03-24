# Analysis: `Deferrable` trait

**File:** `fp-library/src/classes/deferrable.rs`
**PureScript reference:** `Control.Lazy` from `purescript-control`

## Summary

`Deferrable<'a>` is a single-method trait that provides `defer(f: impl FnOnce() -> Self + 'a) -> Self`, enabling lazy construction from a thunk. It is the Rust translation of PureScript's `Lazy` class (`defer :: (Unit -> l) -> l`). The trait has 13 implementations across the lazy evaluation hierarchy and one subtrait, `SendDeferrable`.

## PureScript comparison

### What translates well

- The core operation (`defer`) maps directly: PureScript's `(Unit -> l) -> l` becomes `(impl FnOnce() -> Self + 'a) -> Self`. The use of `FnOnce` is the right choice since `Unit -> l` in PureScript is a thunk that will only be called once for the purpose of constructing the value.
- The trait name change from `Lazy` to `Deferrable` is a good choice, since `Lazy` is already an overloaded term in Rust (there is `std::cell::LazyCell`, `std::sync::LazyLock`, and the library's own `Lazy` type). Using `Deferrable` avoids confusion.

### What is missing: `fix`

PureScript's `Control.Lazy` defines:

```purescript
fix :: forall l. Lazy l => (l -> l) -> l
fix f = go where go = defer \_ -> f go
```

The Rust trait documents why `fix` is absent: lazy self-reference requires shared ownership (`Rc`/`Arc`) and interior mutability, which only `Lazy` provides. The concrete `rc_lazy_fix` and `arc_lazy_fix` functions fill this gap. This is a sound design decision, and the documentation explains it clearly.

However, there is a subtlety worth noting. The PureScript `fix` works because `go` is a lazily-evaluated binding: `defer \_ -> f go` captures `go` by reference in a thunk, and the thunk is not forced until after `go` is bound. In Rust, the closest analogue is indeed the `OnceCell`/`OnceLock` pattern used in `rc_lazy_fix`/`arc_lazy_fix`. There is no way to express this generically across all `Deferrable` types since Rust lacks lazy bindings at the language level.

### What is missing: instances for function types and `()`

PureScript provides:

```purescript
instance lazyFn :: Lazy (a -> b) where
  defer f = \x -> f unit x

instance lazyUnit :: Lazy Unit where
  defer _ = unit
```

The Rust library has no `Deferrable` impl for `()` or for function types. These are worth considering:

- **`()` instance:** `defer(|_| ()) = ()`. This is trivial and would be useful in generic contexts. However, `()` does not normally carry a lifetime parameter, so the impl would be `impl Deferrable<'static> for ()`, or `impl<'a> Deferrable<'a> for ()` since `()` is `'static` and satisfies any lifetime bound.
- **Function type instances:** PureScript's `Lazy (a -> b)` defers evaluation of the function itself by eta-expanding. In Rust, the analogous concept would be harder to express due to the lack of a single canonical function type (there are `fn`, `Fn`, `FnMut`, `FnOnce`, closures, etc.), and trait impls cannot be written for `impl Fn(A) -> B` generically. This is a fundamental language limitation and not a flaw in the library.

## Design evaluation

### Trait definition: correct and minimal

The trait is well-designed:

- **Single method:** `defer` is the only method, matching PureScript's single-method class.
- **`FnOnce` is appropriate:** The thunk produces a value once; `FnOnce` is the minimal requirement.
- **Lifetime parameter on the trait:** `Deferrable<'a>` correctly captures that different implementors have different lifetime constraints. `Thunk<'a, A>` supports arbitrary lifetimes, while `Trampoline<A>` and `Free<ThunkBrand, A>` require `'static`. This is more expressive than a single fixed lifetime.
- **`where Self: Sized`:** Required because `defer` returns `Self` by value, which needs a known size. This is unavoidable.

### The eager-evaluation problem for `Send` types

Several implementations of `Deferrable::defer` call `f()` eagerly (i.e., the thunk is executed immediately rather than being deferred). This occurs for:

- `SendThunk`: `fn defer(f) -> Self { f() }`
- `TrySendThunk`: `fn defer(f) -> Self { f() }`
- `ArcLazy`: `fn defer(f) -> Self { f() }`
- `ArcTryLazy`: `fn defer(f) -> Self { f() }`

The reason is documented: `Deferrable::defer` does not require `Send` on the closure, but `SendThunk::new` and `ArcLazy::new` do (they store the closure in a `Send`-capable wrapper). Since the trait method signature cannot add `Send`, the only option is to evaluate eagerly and wrap the result.

This is a **genuine semantic issue**. The transparency law says `defer(|| x)` should be observationally equivalent to `x`, and in a pure setting, eager vs. lazy evaluation produces the same result. But in practice:

1. **Side effects in the thunk are executed at construction time, not at evaluation time.** If the thunk prints or mutates something, `defer(|| { println!("hi"); SendThunk::pure(42) })` prints immediately, whereas `Thunk::defer(|| { println!("hi"); Thunk::pure(42) })` prints only when evaluated.
2. **Performance characteristics change.** The whole point of deferral is to delay computation. Eager evaluation defeats this purpose for `Send` types.

The `SendDeferrable` trait exists as the proper fix: `send_defer` requires `Send + Sync` on the closure and can actually defer execution. This is a correct design, but it means that `Deferrable::defer` for `Send` types is misleading, since calling it on `ArcLazy` or `SendThunk` does not actually defer anything.

**Possible mitigations:**
- Add a note in the `Deferrable` trait documentation warning that some implementations may evaluate eagerly when the closure does not meet `Send` requirements.
- Consider whether these types should implement `Deferrable` at all, or whether the eager-evaluation implementations should be removed in favor of requiring users to use `SendDeferrable` directly for `Send` types. The tradeoff is that removing these impls would mean generic code bounded by `Deferrable` could not accept `ArcLazy` or `SendThunk`.

### The trait is not used in generic algorithms

`Deferrable` is only used as a bound in two places:
1. The free function `defer<'a, D: Deferrable<'a>>(f) -> D`.
2. The supertrait `SendDeferrable<'a>: Deferrable<'a>`.

No algorithm or combinator in the library is generic over `Deferrable`. This means the trait currently serves primarily as a vocabulary type, an interface contract rather than a mechanism for code reuse. This is not necessarily a problem; in PureScript, `Lazy` is similarly used mostly for `fix` and for signaling laziness capability. But it does mean the trait's value is somewhat limited in practice, since users will call `Thunk::defer(f)` directly rather than the trait method.

### Relationship to `Evaluable`

`Evaluable` is the inverse of `Deferrable` conceptually: `Deferrable` wraps a computation, `Evaluable` unwraps it. However, there is no formal relationship between them (no supertrait connection, no law connecting them). This is fine, since `Evaluable` is an HKT-level trait (on brands) while `Deferrable` is a value-level trait (on concrete types). They operate at different abstraction levels.

## Documentation quality

The documentation is good overall:

- **Laws section:** The transparency law is stated clearly.
- **`fix` rationale:** Well-explained with links to concrete alternatives.
- **Examples:** Present for both the trait and the free function.

### Minor issues

1. The module-level doc example (`let eval: Thunk<i32> = defer(|| Thunk::new(|| 42))`) creates a `Thunk::new` inside `defer`, which is a thunk-of-a-thunk. It would be slightly clearer to use `Thunk::pure(42)` inside the `defer` call, as the trait-level example does, since that better demonstrates the transparency law.
2. The free function's doc says "Creates a value from a computation that produces the value," which is identical to the trait method's doc. This is fine for consistency but could be slightly differentiated (e.g., "Convenience wrapper for `Deferrable::defer`").

## Trait bounds assessment

The trait bounds are minimal and correct:

- `impl FnOnce() -> Self + 'a` is the weakest possible closure bound. `Fn` or `FnMut` would be unnecessarily restrictive.
- The `+ 'a` bound ensures the closure lives long enough for the deferred type's lifetime.
- No unnecessary bounds like `Clone`, `Send`, or `Debug` are imposed.

## Edge cases and ergonomics

1. **Type inference:** `defer(|| Thunk::pure(42))` requires a type annotation on the binding (e.g., `let x: Thunk<i32> = ...`) because the return type `D` is not constrained by the argument. This is inherent to the design and unavoidable without turbofish syntax.

2. **Nested deferral:** `defer(|| defer(|| x))` should be equivalent to `defer(|| x)` by the transparency law. For `Thunk`, this creates `Thunk::new(|| Thunk::new(|| x.evaluate()).evaluate())`, which is correct but adds unnecessary indirection. There is no `join`-like flattening optimization. This is unlikely to matter in practice.

3. **The `'a` parameter can be annoying.** When writing generic code bounded by `Deferrable<'a>`, the lifetime must be threaded through, and callers must specify it. For `'static` types like `Trampoline`, this means writing `Deferrable<'static>` explicitly. This is a Rust ergonomics issue, not a design flaw.

## Recommendations

1. **Document the eager-evaluation caveat.** Add a note to the trait-level documentation explaining that some implementations (those requiring `Send` closures internally) may evaluate the thunk eagerly, and that `SendDeferrable` should be preferred for thread-safe types when true deferral is needed.

2. **Consider a `()` instance.** `impl<'a> Deferrable<'a> for () { fn defer(_f: impl FnOnce() -> () + 'a) -> () { () } }` is trivially correct and could be useful in generic contexts.

3. **Improve the module-level example.** Change `Thunk::new(|| 42)` to `Thunk::pure(42)` inside the `defer` call for consistency with other examples and to avoid the thunk-of-a-thunk pattern.

4. **No structural changes needed.** The trait design is sound. The lifetime parameterization, `FnOnce` choice, and separation from `SendDeferrable` are all correct decisions. The eager-evaluation issue for `Send` types is an inherent tension in Rust's type system, and the `SendDeferrable` subtrait is the right mitigation.
