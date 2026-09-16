# Limitations and Workarounds

Sections are ordered from most fundamental (language-level constraints that shape the entire library) to most applied (specific implementation trade-offs with workarounds in place).

## The Brand Pattern (No Native HKT)

### The Issue

Rust does not support higher-kinded types natively. You cannot write `impl Functor for Option` because `Option` is a type constructor (`* -> *`), not a type (`*`), and Rust's trait system only operates on concrete types.

The library works around this using the Brand pattern (lightweight higher-kinded polymorphism / type-level defunctionalization): each type constructor has a zero-sized marker type (e.g., `OptionBrand`) that implements `Kind` traits mapping it back to the concrete type.

### Consequences

- **No method syntax.** Type class operations are free functions, not methods on the container. You write `bind(x, f)` not `x.bind(f)`.
- **Generated trait names in errors.** Compiler errors expose the macro-generated `Kind` trait names (e.g., `Kind_cdc7cd43dac7585f`) rather than human-readable names, making diagnostics harder to interpret.
- **Wrapping/unwrapping overhead in generic code.** Generic functions must use `Apply!` macro invocations to convert between the `Kind` associated type and the concrete type, adding syntactic noise.
- **Turbofish for ambiguous types.** Types reachable through multiple brands at a given arity (e.g., `Result` at arity 1) cannot use brand inference and require `explicit::` variants with turbofish: `explicit::map::<ResultErrAppliedBrand<E>, _, _, _, _>(f, x)`.

### Mitigation

**Brand inference:** For types with a single unambiguous brand (Option, Vec, Identity, Thunk, Lazy, etc.), the `InferableBrand` trait enables the compiler to infer the brand from the container type. No turbofish needed: `map(|x| x + 1, Some(5))`. At arity 2 (bifunctor operations), types like `Result` that are ambiguous at arity 1 become unambiguous: `bimap((f, g), Ok(5))`.

**Do-notation:** The `m_do!` and `a_do!` macros provide ergonomic do-notation. In inferred mode (`m_do!({ ... })`), the brand is inferred from container types. In explicit mode (`m_do!(Brand { ... })`), the brand is specified for ambiguous types or to use `pure()`.

The `Pipe` trait allows method-chaining syntax for some operations. The `impl_kind!` and `trait_kind!` macros automate the boilerplate of defining new brands and kind traits.

## Uncurried Semantics (No Zero-Cost Currying)

### The Issue

Most FP languages and libraries use curried functions: `map(f)(fa)`. In Rust, returning a closure from a function requires either boxing it (`Box<dyn Fn>`) or wrapping it in a reference-counted pointer (`Rc<dyn Fn>`, `Arc<dyn Fn>`). Both involve heap allocation and dynamic dispatch, defeating the library's zero-cost abstraction goal.

Every closure in Rust has a unique, anonymous type. A curried `map(f)` would need to return `impl Fn(F::Of<A>) -> F::Of<B>`, but `impl Trait` in return position captures the concrete closure type, making it impossible to store, pass around, or compose without type erasure.

### Consequence

The library uses uncurried semantics throughout: `map(f, fa)` instead of `map(f)(fa)`. This allows the compiler to monomorphize `f` at each call site, enabling inlining and zero heap allocation. The trade-off is that partial application is not directly supported; you must use explicit closures instead (e.g., `|fa| map(f, fa)`).

### Potential Future Resolution

The nightly feature `unboxed_closures` ([rust-lang/rust#29625](https://github.com/rust-lang/rust/issues/29625)) combined with `fn_traits` ([rust-lang/rust#29625](https://github.com/rust-lang/rust/issues/29625)) and particularly `impl_trait_in_fn_trait_return` ([rust-lang/rust#99697](https://github.com/rust-lang/rust/issues/99697)) could enable zero-cost currying by allowing functions to return `impl Fn` without boxing. If stabilized, the library could offer curried variants alongside the uncurried API.

## No Rank-N Types

### The Issue

Rust cannot express rank-2 (or higher) types. You cannot write a type alias or data type that is universally quantified over a trait-bounded type parameter. In PureScript/Haskell, rank-2 types are used pervasively in FP abstractions. Their absence in Rust forces workarounds throughout the library.

### Consequences

#### Profunctor optics

In PureScript, an optic is a rank-2 polymorphic function:

```purescript
type Lens s t a b = forall p. Strong p => p a b -> p s t
```

Composition is ordinary function composition (`<<<`), and the profunctor is chosen at the use site. Rust cannot express this, so the library uses concrete structs (`Lens`, `Prism`, `Iso`, etc.) storing reified internal representations (equivalent to PureScript's `ALens`/`APrism`/`AnIso`), composed via a `Composed` struct with static dispatch rather than function composition. This results in deeply nested types for long composition chains, and generic code must be bounded by optic traits (e.g., `O: LensOptic`) rather than profunctor constraints.

See [Optics Comparison](optics-analysis.md) and [Profunctor Classes Analysis](profunctor-analysis.md) for detailed comparisons.

#### `Wander` trait

PureScript's `wander` takes `forall f. Applicative f => (a -> f b) -> s -> f t`. Rust encodes this via the `TraversalFunc` trait, which provides a concrete `apply` method that the `Wander` implementation calls with specific applicative functors.

#### No `Yoneda` type

PureScript's `Yoneda f a` is `forall b. (a -> b) -> f b`, which requires rank-2 quantification to store as a data type. This cannot be represented in Rust.

#### No `unCoyoneda` eliminator

In Haskell/PureScript, `unCoyoneda :: (forall b. (b -> a) -> f b -> r) -> Coyoneda f a -> r` provides access to the existential intermediate type `b` via a rank-2 continuation. Without this, `Coyoneda::hoist` must lower first (requiring `F: Functor`), transform, then re-lift. PureScript's `hoistCoyoneda` has no `Functor` constraint. `CoyonedaExplicit` avoids this because `B` is an explicit type parameter, not existential.

## Unexpressible Bounds in Trait Method Signatures

### The Issue

The reference-counted stores of the `Store`-parameterised types (`Coyoneda<'a, F, A, Store>` and `FreeExplicit<'a, F, A, Store>` at `RcBrand`/`ArcBrand`) cannot implement some type class traits at the brand level, because their operations require bounds (like `Clone` or `Send + Sync`) on the `Kind` associated type `F::Of<'a, A>` or on the value type `A` that cannot be expressed in the trait method signatures.

For example, `Coyoneda::lift` at the Rc store requires `F::Of<'a, A>: Clone` because the base layer must be clonable for `lower_ref` to work. But the `Pointed` trait's `pure` method has no way to express this:

```rust,ignore
// Pointed::pure signature - no Clone bound on the return type's contents
fn pure<'a, A: 'a>(value: A) -> Self::Of<'a, A>;
```

The same problem affects `Semimonad::bind`, `Semiapplicative::apply`, and `Lift::lift2` at the Rc and Arc stores.

At the Arc store, the problem is compounded: the plain `Functor::map` signature lacks `Send + Sync` bounds on the closure parameter, so closures passed to `map` cannot be stored inside `Arc`-wrapped layers. The `SendFunctor`/`SendFoldable` hierarchy carries those bounds and closes that particular gap.

### Consequences

| Form                 | Brand-level `Functor`   | Brand-level `Pointed` | Brand-level `Semimonad` | Reason                                                                                                                                                                                                                               |
| :------------------- | :---------------------- | :-------------------- | :---------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Coyoneda` (Box)     | Yes                     | Yes                   | Yes                     | The one-shot `Box<dyn CoyonedaInner>` cell has no extra bounds.                                                                                                                                                                      |
| `Coyoneda` (Rc)      | Yes                     | No                    | No                      | Construction needs `F::Of: Clone`.                                                                                                                                                                                                   |
| `Coyoneda` (Arc)     | Yes (via `SendFunctor`) | No                    | No                      | Construction needs `F::Of: Clone + Send + Sync` and closures `Send + Sync`. `SendFunctor` (closure has `Send + Sync`) closes the by-value `Functor` gap; the `Pointed`/`Semimonad` analogues stay blocked by the construction bound. |
| `FreeExplicit` (Box) | Yes                     | Yes                   | Yes                     | Concrete recursive enum; `bind` has no `Clone` bound. `Lift` / `Semiapplicative` / `Applicative` / `Monad` blocked: `lift2` consumes `fb` multiply, and the Box store is not `Clone`.                                                |
| `FreeExplicit` (Rc)  | No                      | Yes                   | No                      | `bind` requires per-`A` `Clone` bounds for the shared-inner-state recovery path; `pure` does not.                                                                                                                                    |
| `FreeExplicit` (Arc) | No                      | Yes                   | No                      | Same as the Rc store, plus the `Send + Sync` auto-derive bound is per-`A` (no HRTB-over-types in stable Rust).                                                                                                                       |

By reference, the Box store of `FreeExplicit` additionally implements `RefFunctor`, `RefPointed`, and `RefSemimonad` (a recursive helper walks `&fa` via `F::ref_map`; `RefLift` is blocked because the closure captures a shorter-lifetime `&A` and cannot satisfy `+ 'a`). The Rc and Arc stores currently expose no by-reference surface; their by-value `bind` and `evaluate` are inherent methods carrying the per-`A` `Clone` bound explicitly.

### Workaround: Inherent Methods

Operations the brand level cannot carry are provided as per-store inherent methods with the necessary bounds stated explicitly:

```rust,ignore
// Coyoneda's Arc store: map requires Send + Sync closures and targets
impl<'a, F, A: Send + Sync + 'a> Coyoneda<'a, F, A, ArcBrand> {
	pub fn map<B: Send + Sync + 'a>(self, f: impl Fn(A) -> B + Send + Sync + 'a) -> Coyoneda<'a, F, B, ArcBrand> { ... }
}

// FreeExplicit's Rc store: bind carries the per-A Clone bound
impl<'a, F, A: Clone + 'a> FreeExplicit<'a, F, A, RcBrand> {
	pub fn bind<B: 'a>(self, f: impl Fn(A) -> FreeExplicit<'a, F, B, RcBrand> + 'a) -> FreeExplicit<'a, F, B, RcBrand> { ... }
}
```

Construction-flavoured operations with no current per-store consumer (`pure`, `bind`, `apply`, and `lift2` on `Coyoneda`) exist only at the Box store; the Rc and Arc stores provide `lift`, `map`, `lower_ref`, `collapse`, and an O(1) structural `Clone`.

This means the reference-counted store arms cannot be used generically (e.g., passed to a function expecting `F: Pointed`), but the provided operations work when used directly on the concrete type. See [Coyoneda Implementations](coyoneda.md) for the full comparison.

### Root Cause

Rust's trait system does not support conditional bounds on associated types. There is no way to write "when `A: Clone`, then `Self::Of<'a, A>` supports `pure`." Each trait method signature is fixed for all implementors. This is a fundamental Rust limitation, not a library design issue.
