# Analysis of `fp-library` Optics vs PureScript's Profunctor Lenses

This document analyzes the design of `fp-library`'s profunctor optics implementation compared to PureScript's `purescript-profunctor-lenses`, highlighting architectural differences, successes, and significant flaws/mistakes in the Rust implementation.

## 1. Architectural Differences & The Impredicativity Workaround

### PureScript Approach

In PureScript, optics are simply type aliases for functions mapping profunctors:

```purescript
type Optic p s t a b = p a b -> p s t
type Lens s t a b = forall p. Strong p => Optic p s t a b
```

Because of **impredicativity** (the inability to easily store rank-N polymorphic functions inside records or state in the Haskell/PureScript type systems), PureScript must define specialized "A-prefixed" variants of optics backed by concrete profunctors. For instance, `ALens` is backed by the `Shop` profunctor, and `APrism` by the `Market` profunctor, enabling them to be passed as first-class values without typechecker errors.

### Rust (`fp-library`) Approach

Rust inherently lacks impredicative polymorphism, so `fp-library` takes a fundamentally different route. The core `Optic` is defined as a trait:

```rust
pub trait Optic<'a, P: Profunctor, S: 'a, T: 'a, A: 'a, B: 'a> {
    fn evaluate(&self, pab: Apply!(P::Of<'a, A, B>)) -> Apply!(P::Of<'a, S, T>);
}
```

Concrete optics like `Lens`, `Prism`, and `Iso` are implemented as **monomorphic structs** that store the underlying closures (`view` and `set` for a Lens). They then implement the `Optic` trait for all matching constraints.

**Verdict:** This is an elegant and successful divergence. Because `Lens` is already a concrete struct, `fp-library` does not need to expose `ALens`, `APrism`, or `AnIso`. The standard struct representations inherently sidestep the impredicativity issue while providing standard trait monomorphization upon evaluation.

---

## 2. Major Flaw: Complete Loss of Polymorphic Consumption

The primary value proposition of profunctor optics is that the consumer doesn't need to know _what_ optic they are holding—they only care about what capabilities (constraints) it provides.

### PureScript's Elegance

In PureScript, the `view` function can consume a `Lens`, an `Iso`, a `Getter`, or a `Fold` because it simply requires the `Forget` profunctor capability:

```purescript
view :: forall s t a b. Getter s t a b -> s -> a
-- (Where Getter is `forall r. Optic (Forget r) s t a b`)
```

### `fp-library`'s Mistake

The helper functions in `fp-library` are rigidly typed to specific structs:

```rust
pub fn optics_view<'a, P, S, A>(optic: &LensPrime<'a, P, S, A>, s: S) -> A
pub fn optics_preview<'a, P, S, A>(optic: &PrismPrime<'a, P, S, A>, s: S) -> Option<A>
```

Because of this rigid typing:

- You **cannot** use `optics_view` on an `IsoPrime`. You are forced to use `optics_from`.
- You **cannot** use `optics_view` on a generic `Getter`.
- You **cannot** use `optics_view` on a composed optic.

This completely breaks the abstraction of profunctor optics. Instead of relying on trait constraints (e.g., passing any `O: Optic<ForgetBrand>`), the API forces exact concrete type matching.

---

## 3. Major Flaw: Composition Ergonomics

### PureScript's Elegance

Because PureScript optics are just functions (`p a b -> p s t`), composing a Lens and a Prism is as simple as function composition using standard operators:

```purescript
myOptic = lens <<< prism
```

### `fp-library`'s Mistake

Because optics in Rust are structs implementing a trait, they cannot compose natively via function composition. `fp-library` introduces a `Composed` struct:

```rust
let composed = optics_compose(outer_lens, inner_lens);
```

This introduces two massive ergonomic hurdles:

1. **Nested Types:** Composing three or four optics results in deeply nested type signatures (e.g., `Composed<..., Composed<...>>`), which leak into the developer experience.
2. **Helper Incompatibility:** Because `optics_view` expects exactly a `LensPrime`, you **cannot** pass a `Composed` optic to it. To view a composed optic, the user is forced to bypass the helpers, drop down to the lowest level (`optics_eval`), and construct raw profunctor instances (`Forget::new(...)`) by hand.

---

## 4. Major Flaw: Inconsistent Structs vs. Trait Objects

While `Lens`, `Prism`, and `Iso` are concrete structs, purely constraint-based optics like `Getter` and `Setter` are defined as type aliases to trait objects:

```rust
pub type Getter<'a, S, T, A, B> = dyn Optic<'a, ForgetBrand<A>, S, T, A, B>;
```

This splits the ecosystem in half:

1. "Concrete" optics (`Lens`, `Prism`) use static dispatch and structs.
2. "Abstract" optics (`Getter`, `Setter`) use dynamic dispatch (`dyn`).

In Rust, trait objects come with severe lifetime, object-safety, and performance implications. Furthermore, because a `Lens` struct isn't a `dyn Optic` trait object without explicit coercion, this design makes it exceedingly difficult to write APIs that generically accept "any optic that acts as a Getter".

---

## Conclusion

`fp-library` successfully ports the underlying category theory of PureScript's profunctor optics—providing correct implementations of `Strong`, `Choice`, and `Wander` for the base profunctors.

However, it sacrifices the elegant developer experience and polymorphism that make PureScript's lenses so powerful. By tying consumption functions (`view`, `set`) to concrete structs instead of the abstract `Optic` trait, and by failing to provide ergonomic, polymorphic composition, it creates an API that is verbose, brittle, and conceptually misaligned with the interoperability goals of profunctor optics.

## 5. Proposed Refactoring Approaches

To resolve these architectural flaws and restore the power of profunctor optics in Rust, several refactoring approaches can be taken.

### Approach 1: Generic Extension Traits (The "Profunctor Ergonomics" Fix)

Instead of tying helper functions to concrete structs, define traits representing optic operations that provide blanket implementations for any type implementing the `Optic` trait with the correct profunctor.

```rust
pub trait GetterOptic<'a, S: 'a, A: 'a> {
    fn view(&self, s: S) -> A;
}

impl<'a, O, S: 'a, A: 'a> GetterOptic<'a, S, A> for O
where
    O: Optic<'a, ForgetBrand<A>, S, S, A, A>,
{
    fn view(&self, s: S) -> A {
        // Evaluate the optic using the Forget profunctor
        // ...
    }
}
```

**Trade-offs:**

- **Pros:** Preserves the core theoretical model (the `Optic` trait and Profunctors). Fully restores polymorphism: you can now call `.view()` on a `Lens`, `Iso`, `Getter`, or a `Composed` optic. Highly ergonomic (method syntax `lens.view(s)`). Zero-cost abstraction via static dispatch.
- **Cons:** Does not solve the infinitely growing type signatures of composed optics (`Composed<O1, Composed<O2, O3>>`). Type inference errors may be slightly more complex when traits fail to resolve.

### Approach 2: Capability Traits (The "Idiomatic Rust" Hierarchy)

Abandon the raw `Optic` trait as the primary public interface, and instead model optics through native Rust "capability traits" (similar to standard Rust lens crates like `lens-rs` or Scala's `Monocle` v1).

```rust
pub trait Getter<'a, S: 'a, A: 'a> {
    fn view(&self, s: S) -> A;
}

pub trait Setter<'a, S: 'a, A: 'a> {
    fn set(&self, s: S, a: A) -> S;
}

// A Lens is simply anything that is both a Getter and a Setter
pub trait Lens<'a, S: 'a, A: 'a>: Getter<'a, S, A> + Setter<'a, S, A> {}
```

**Trade-offs:**

- **Pros:** Extremely idiomatic Rust. Excellent compiler error messages. Very easy to write functions that take `&impl Getter` or `Box<dyn Getter>`. Removes the conceptual overhead of Profunctors for library consumers.
- **Cons:** Deviates entirely from the PureScript profunctor encoding. You lose the unified mathematical foundation of profunctor optics. Extending the hierarchy requires writing explicit blanket implementations between all compatible traits instead of relying on the profunctor typeclass hierarchy to magically unify them.

### Approach 3: Operator Overloading & Concrete Optics (The "Syntactic Sugar" Fix)

Retain the current struct-based approach but radically improve the composition ergonomics using Rust's operator overloading (e.g., `std::ops::Shr` or `BitOr`), combined with rewriting the helpers to accept generic types.

```rust
impl<'a, S, T, M, N, A, B, O1, O2> std::ops::Shr<O2> for O1
where
    // Simplified trait bounds for illustration
    O1: Optic<'a, P, S, T, M, N>,
    O2: Optic<'a, P, M, N, A, B>,
{
    type Output = Composed<'a, S, T, M, N, A, B, O1, O2>;

    fn shr(self, rhs: O2) -> Self::Output {
        Composed::new(self, rhs)
    }
}
```

Helper functions like `optics_view` would then be updated to accept `&impl Optic<ForgetBrand<A>, ...>`.

**Trade-offs:**

- **Pros:** Makes composition as elegant as PureScript (`lens1 >> lens2 >> lens3`). Low refactoring effort compared to Approach 2, as it builds on the existing `Composed` struct.
- **Cons:** Operator overloading on complex generic types can stretch the Rust compiler's trait solver, potentially leading to slow compile times. The `Composed` type nesting remains a problem for debug signatures.

### Approach 4: Rank-2 Trait Encoding (Direct PureScript Emulation)

The most theoretically accurate way to emulate PureScript's `forall p. Strong p => p a b -> p s t` in Rust is to shift the Profunctor generic parameter `P` from the _trait_ level down to the _method_ level. This creates a "Rank-2" encoding.

Instead of a single `Optic` trait, we define a trait for each level of the optic hierarchy, constraining the generic `P` on the `evaluate` method:

```rust
// Matches: type Iso s t a b = forall p. Profunctor p => p a b -> p s t
pub trait IsoOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
    fn evaluate<P: Profunctor>(&self, pab: Apply!(P::Of<'a, A, B>)) -> Apply!(P::Of<'a, S, T>);
}

// Matches: type Lens s t a b = forall p. Strong p => p a b -> p s t
pub trait LensOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
    fn evaluate<P: Strong>(&self, pab: Apply!(P::Of<'a, A, B>)) -> Apply!(P::Of<'a, S, T>);
}

// Matches: type Prism s t a b = forall p. Choice p => p a b -> p s t
pub trait PrismOptic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
    fn evaluate<P: Choice>(&self, pab: Apply!(P::Of<'a, A, B>)) -> Apply!(P::Of<'a, S, T>);
}
```

To resolve the composition issue, `optics_view` and `optics_set` would no longer take concrete structs like `LensPrime`. Instead, they would take `&impl LensOptic` or `&impl GetterOptic`, instantly restoring true polymorphism.

**Trade-offs:**

- **Pros:** This is the closest mathematical equivalent to PureScript's Profunctor optics possible in Rust. It cleanly separates the optic capabilities. By removing `P` from the struct and trait definition, optics become significantly easier to store and pass around.
- **Cons:** Rust's trait system does not support automatic subtyping for constraints. In PureScript, an `Iso` (`forall p. Profunctor p`) is automatically accepted wherever a `Lens` (`forall p. Strong p`) is expected because `Strong` implies `Profunctor`. In Rust, you must manually define this hierarchy using blanket implementations:
  ```rust
  // Manually wiring the hierarchy
  impl<'a, O, S, T, A, B> LensOptic<'a, S, T, A, B> for O
  where O: IsoOptic<'a, S, T, A, B> { ... }
  ```
  Building the full lattice of blanket implementations for the entire optic hierarchy (Iso -> Lens/Prism -> AffineTraversal -> Traversal -> Fold/Setter) is extremely boilerplate-heavy and risks triggering Rust's overlapping implementation errors (`E0119`).
