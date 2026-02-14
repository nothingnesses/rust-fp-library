Optics implementations across languages

What are the differences between the [https://pursuit.purescript.org/packages/purescript-profunctor-lenses/8.0.0](https://pursuit.purescript.org/packages/purescript-profunctor-lenses/8.0.0) library for PureScript and the [https://hackage.haskell.org/package/optics](https://hackage.haskell.org/package/optics) library for Haskell? Are there other popular forms/implementations of optics? What are the trade-offs of each? I'm building [https://raw.githubusercontent.com/nothingnesses/rust-fp-library/refs/heads/main/README.md](https://raw.githubusercontent.com/nothingnesses/rust-fp-library/refs/heads/main/README.md) and would like to add some form of optics. What form of optics would make the most sense to implement and why?

* * *

This is a rich topic that sits at the intersection of type theory, library design, and language constraints. Let me break it down systematically.

* * *

The Major Encodings of Optics
-----------------------------

Before comparing the specific libraries, it helps to understand the main ways optics can be represented, since each library is essentially a choice of encoding.

**1\. Concrete (get/set pair)** The simplest formulation: a `Lens s a` is literally a `{ view: s -> a, set: s -> a -> s }` struct. Easy to understand, but composition is non-trivial and requires manual plumbing. You must write a custom `compose` function, and extending the hierarchy to prisms, traversals, etc. requires defining separate composition rules for each combination.

**2\. Van Laarhoven (functor-polymorphic functions)** The encoding used by Haskell's dominant `lens` library:

haskell

    type Lens s t a b = forall f. Functor f => (a -> f b) -> s -> f t
    type Traversal s t a b = forall f. Applicative f => (a -> f b) -> s -> f t
    type Prism s t a b = forall p f. (Choice p, Applicative f) => p a (f b) -> p s (f t)

The killer feature is that every optic is just a function, so **composition is ordinary function composition** (`<<<` or `(.)`). This is why the `lens` library can have such a simple composition story. The tradeoff is that the types are rank-2 (impredicative in some cases), which causes confusing error messages — you see the raw functor-polymorphic type rather than something labelled "Lens". Also, some optic kinds (affine traversals, which focus on _at most one_ element) don't have a natural van Laarhoven form because there's no standard typeclass between `Functor` and `Applicative` that captures them cleanly.

**3\. Profunctor** The profunctor encoding represents optics as constraints on a profunctor `p`:

haskell

    type Iso   s t a b = forall p. Profunctor p            => p a b -> p s t
    type Lens  s t a b = forall p. Strong p                => p a b -> p s t
    type Prism s t a b = forall p. Choice p                => p a b -> p s t

Composition still works by function composition. The optic "kind" is determined by which profunctor constraint you require. Combining a `Lens` (requires `Strong`) with a `Prism` (requires `Choice`) gives you an `AffineTraversal` (requires both). This makes it very easy to generalize to new optic kinds: the type is determined entirely by the constraint on `p`.

**4\. Existential** The existential encoding makes the structure extremely explicit:

haskell

    data Lens s a = forall c. Lens (s -> (c, a)) ((c, a) -> s)
    data Prism s a = forall c. Prism (s -> Either c a) (Either c a -> s)

This says a `Lens s a` witnesses that `s` is isomorphic to some product `(c, a)`, and a `Prism s a` witnesses that `s` is isomorphic to some sum `Either c a`. This encoding has the deepest categorical clarity and connects most cleanly to category-theoretic treatments. The downside is that function composition no longer applies — you need a custom `compose` operation, making it more awkward as an API.

* * *

`purescript-profunctor-lenses` vs. `optics`
-------------------------------------------

These libraries share the same underlying encoding (profunctor), but differ significantly in design philosophy.

### `purescript-profunctor-lenses`

PureScript chose the profunctor encoding because it doesn't have the historical baggage of Haskell's `lens`. The library uses transparent type synonyms, so `Lens s t a b` is directly `forall p. Strong p => p a b -> p s t`.

Composition works via `<<<` (PureScript's backwards function composition), which is just ordinary function composition. Each optic in the library describes a relationship between a structure `s` and zero, one, or many values `a` (called the focus), and you compose small optics into larger ones by chaining them.

Key features:

*   Transparent types — you see the profunctor encoding directly in type signatures
*   Indexed optics supported
*   Composition via function composition (`<<<`)
*   Includes `Iso`, `Lens`, `Prism`, `Traversal`, `Fold`, `Getter`, `Setter`, `AffineTraversal`, and more
*   No separate "tag" mechanism — optic kinds are plain type aliases

The transparency is a double-edged sword. It means optics can be defined without depending on the library at all, but it also means type errors expose internal constraints rather than friendly optic names.

### `optics` (Haskell, well-typed)

The `optics` library uses an abstract/opaque `Optic` newtype: each kind of optic is identified by a "tag type" (like `A_Lens`), so that `Lens s t a b = Optic A_Lens NoIx s t a b`. The profunctor representation is an implementation detail hidden behind this abstraction boundary.

This has major consequences:

*   Type inference is better — the optic kind is preserved in the inferred type, so `:t traversed % to not` gives you `Traversable t => Optic A_Fold '[] (t Bool) (t Bool) Bool Bool` rather than a sprawling constraint mess.
*   Affine optics (at most one target) are supported naturally — something the van Laarhoven encoding struggles with.
*   The optic hierarchy is closed — you cannot add new optic kinds without modifying the library. This is the price paid for a clean, abstract interface.
*   Composition uses `%` instead of `(.)`, since you're composing opaque newtypes rather than raw functions.
*   OverloadedLabels work out of the box — you can use `#name` as a lens for any record with a `name` field.
*   A separate `optics-vl` package provides compatibility utilities for converting to/from the van Laarhoven representations used by `lens`.

The key design tension: `optics` sacrifices the open-world extensibility of the transparent profunctor encoding in exchange for much friendlier type errors and a cleaner user-facing API.

* * *

Other Notable Approaches
------------------------

**`lens` (Haskell)** — the dominant library, uses van Laarhoven. Enormously feature-rich but infamous for opaque error messages. Has `Getting`, `ALens`, `ATraversal` etc. as rank-1 variants needed to work around impredicativity. Notably, `lens` generalises `view` over any `MonadReader`, and allows `view` to work on folds; `optics` deliberately does not do this by default.

**`microlens`/`microlens-platform` (Haskell)** — a minimal subset of `lens` that covers the 90% use case. Only lenses and traversals, no prisms, much smaller dependency footprint.

**`prolens` (Haskell)** — a lightweight profunctor implementation focused on education, minimalism, and performance, while remaining beginner-friendly. It includes a mini-tutorial on optics within the documentation.

**Monocle (Scala)** — existential-based, large batteries-included library with derive macros. Heavily influenced Rust attempts.

**`lens-rs` (Rust)** — a Rust crate that uses proc-macros to derive optics, with trait-based dispatch. Avoids the HKT problem by making traversals concrete over specific types.

* * *

What This Means for `fp-library`
--------------------------------

Your library has already solved the hardest part of the HKT problem using the defunctionalization/brands approach. This is actually the foundation you need. Here's the core challenge and your options:

### The Fundamental Rust Obstacle

Van Laarhoven and profunctor encodings both rely on _universal quantification over a type constructor_:

*   Van Laarhoven: `forall f. Functor f => ...`
*   Profunctor: `forall p. Strong p => ...`

Rust's type system cannot express this natively. You can't write `for<F: Functor> (A -> F<B>) -> S -> F<T>` because `F` isn't a fully-applied type. Due to Rust's lack of native support for HKTs, it's not possible to implement a general `Functor<_>` trait, so any mapped implementation can only be tailored to a specific type constructor.

Your brands/defunctionalization machinery handles this for `Functor`, `Monad` etc., but applying it to the _profunctor encoding of optics_ would be extremely verbose — every optic would need to universally quantify over all `Strong` profunctor brands, which is a massive list of trait bounds.

### Recommended Approach: Concrete (Existential) Optics

Given your library's design priorities (zero-cost, uncurried, Rust-idiomatic), **concrete/existential optics are the right fit**. Here's why:

The existential encoding maps cleanly to Rust structs:

rust

    // Lens: s is isomorphic to (Context, A)
    struct Lens<S, A> {
        view: fn(&S) -> A,
        update: fn(S, A) -> S,
    }
    
    // Prism: s is isomorphic to Either<Context, A>
    struct Prism<S, A> {
        preview: fn(S) -> Option<A>,   // Left branch = not this variant
        review: fn(A) -> S,
    }
    
    // Traversal: generalises both
    struct Traversal<S, A> {
        to_list:   fn(&S) -> Vec<&A>,
        over:      fn(S, fn(A) -> A) -> S,
    }

Composition is a custom `compose` function (not automatic like function composition), but this is fine for a Rust library — you already have an uncurried, explicit-composition design philosophy with `compose`.

**Why not van Laarhoven?** Because the `forall f. Functor f =>` part requires HKT in a way that your brands approach would need to enumerate every concrete brand as a bound — not zero-cost and not ergonomic.

**Why not profunctor?** Same problem, doubled: you'd need to universally quantify over profunctor brands. This would require something like `for<P: StrongBrand>` where `StrongBrand` is a brand trait representing all strong profunctors, and the plumbing grows combinatorially.

**Why not an opaque-tag approach like `optics`?** This is actually viable and interesting in Rust. You could have:

rust

    struct Optic<Kind, S, A>(/* internal */, PhantomData<(Kind, S, A)>);
    struct LensTag;
    struct PrismTag;

This gives you better type errors (you see `Optic<LensTag, S, A>` not a complex bound) and matches the design philosophy of the Haskell `optics` library. The cost is that the hierarchy is closed. But since your library already has a clear, curated set of type classes, a closed optic hierarchy is probably appropriate and more maintainable.

### Practical Recommendation

Implement a **concrete optics layer** using the existential-inspired approach, integrated with your existing brands:

1.  Start with `Lens<S, A>` (concrete `view`/`set`), `Prism<S, A>` (concrete `preview`/`review`), and `Traversal<S, A>` (using your `Traversable` instances). These cover 95% of practical use cases.
2.  Use your existing `Functor`/`Traversable` brands to implement `over` for traversals — your `map` and `traverse` functions already do the heavy lifting. A `Traversal<S, A>` over a structure that implements your `Traversable` typeclass is essentially just `traverse` applied to a specific focus.
3.  Provide a `compose` function for each combination (`Lens ∘ Lens = Lens`, `Lens ∘ Prism = AffineTraversal`, etc.), possibly using marker traits to encode the subtyping lattice.
4.  Optionally use an opaque tag approach (`Optic<Kind, S, A>`) for cleaner types, at the cost of some internal complexity.

The existential encoding has one more advantage that aligns well with your zero-cost philosophy: concrete structs holding function pointers or `impl Fn` can be fully monomorphized and inlined by the compiler, whereas dynamic dispatch or type-erased optics would add indirection.

### Files hidden in shared chats

23:29

This is a genuinely interesting meta-question, and the critique makes some valid points while also having its own inaccuracies. Let me go through it carefully.

* * *

Where the Critique is Right
---------------------------

**The struct-with-fn-pointer criticism has merit.** My suggestion of `fn` pointers (`fn(&S) -> A`) was indeed too simplistic — raw function pointers can't capture environments, so you'd need `impl Fn` bounds or boxed closures in practice. The critique is correct that naive `Box<dyn Fn>` incurs heap allocation, and that consuming ownership on every `set` call (`fn(S, A) -> S`) is a real concern in Rust.

However, this is a solvable problem rather than a fatal flaw in concrete optics — you can use `impl Fn` with generic parameters on the struct, or use `&mut`\-based setters. The critique overstates "catastrophically slow" — many Rust FP patterns do use owned transformations, and your library explicitly adopts an owned/uncurried style throughout.

* * *

Where the Critique is Wrong or Overstated
-----------------------------------------

### On profunctor optics being "the best fit"

The sketch given is appealing but glosses over the hardest part. The claim is:

> "A Lens is simply a generic function! Universal quantification is just a generic function definition."

This is only partially true. The issue is **not** writing a single lens like `fn lens<P: Strong>(...)`. The issue is that a _value_ of type `Lens<S, A>` — something you can store, pass around, compose, and return — requires the universal quantification to be part of the _data_, not just the _function signature_. In Rust, a stored `Lens` would need to be something like:

rust

    struct Lens<S, T, A, B>(Box<dyn for<P: Strong> Fn(App2<P, A, B>) -> App2<P, S, T>>);

Rust **cannot express this** — `for<P: Strong>` is not valid syntax because Rust's HRTB (`for<'a>`) only quantifies over lifetimes, not types or traits. This is the actual blocker the critique waves away. You can write optics as _individual generic functions_ but you cannot easily store them as _first-class values_ with the full universally quantified type. This fundamentally limits composability at the value level.

The critique's own code sketch never shows a stored lens value or composition of two lenses — only a single-lens definition. That's the easy part.

### On van Laarhoven being merely "Good"

The critique says van Laarhoven is hard because of "GATs or complex lifetimes" for `Traversable`. But your library _already has `Traversable`_. The van Laarhoven traversal encoding:

rust

    fn over<F: Functor, S, A, B>(
        traversal: impl Fn(impl Fn(A) -> App<F, B>) -> App<F, S>,
        f: impl Fn(A) -> App<F, B>,
        s: S,
    ) -> App<F, S>

...is expressible using your existing brand machinery in a way closer to what the critique claims for profunctors. The lifetime concern is real but manageable.

* * *

My Revised Assessment
---------------------

Both responses have merit. The corrected, synthesised view:

1.  **Concrete/existential optics** are the most _Rust-idiomatic_ and easiest to implement, store, and compose — but the critique is right that naive ownership semantics are painful. Use `impl Fn` generics and `&mut` variants to address this. This is the pragmatic choice if ergonomics for _users of the library_ is the priority.
2.  **Van Laarhoven** is viable given your existing `Functor`/`Traversable` brands. Individual optics can be expressed as generic functions. The limitation is the same as profunctor: you can't easily store a `Lens<S, A>` as a first-class value with its quantification preserved.
3.  **Profunctor optics** are the most theoretically elegant fit for your brand architecture, but the critique's dismissal of the "universal quantification" problem is wrong — it applies equally to profunctor optics as to van Laarhoven. The `for<P: Strong>` higher-ranked trait bound on types (not lifetimes) is not expressible in stable Rust today.

**My original recommendation stands with a correction:** concrete optics are the most practical, but the implementation should use `impl Fn` generics (monomorphised) rather than `fn` pointers, and `&mut` or owned-update variants rather than purely copying semantics. The critique's profunctor proposal is theoretically elegant but does not resolve the core constraint it claims to resolve — it just moves the unresolvable quantification from van Laarhoven's `forall f` to profunctor's `forall p`.

I asked: `Is the response correct, or do you disagree? Does it have inaccuracies or mistakes?` Response: ``This response is **correct in its core analysis** and offers a critical, high-quality correction to the previous advice. It accurately identifies the "Composition Problem" caused by Rust's lack of Rank-2 polymorphism, which is the single biggest hurdle for optics in Rust. Here is a breakdown of why this response is accurate and how to apply its advice. ### 1. The Core Insight: The "Rank-2" Blocker The response correctly identifies that the previous critique ("Profunctor is just a generic function") fails when you try to **compose** optics. * **In Haskell:** `compose lens1 lens2` results in a new function that is *still* polymorphic over `P`. It remains `forall p. Strong p => ...`. * **In Rust:** When you compose two generic functions (e.g., via a closure or helper function), the compiler must decide what `P` is *at the moment of composition*. * You cannot write: `let composed = |p| lens1(lens2(p));` where `p` is generic. Rust closures cannot be generic. * This means you cannot create a reusable `composed` lens that works for both "Getting" (where `P=Forget`) and "Setting" (where `P=Function`). You would have to re-compose them every time you use them. The response is 100% correct that this makes "optics as functions" ergonomic for *defining* atomic lenses, but broken for *building* libraries of composed lenses. ### 2. The Solution: Concrete Optics (with a caveat) The response recommends "Concrete/Existential" optics (structs) as the practical solution. * **Correction on Syntax:** The response mentions using `impl Fn` generics" on the struct. To be clear, valid Rust syntax requires named generic parameters: ```rust // Correct "Concrete" implementation pub struct Lens<S, A, G, S2> where G: Fn(&S) -> A, S2: Fn(S, A) -> S, { get: G, set: S2, _marker: PhantomData<(S, A)>, } ``` This works perfectly and allows the compiler to inline the functions, verifying the response's claim about performance. ### 3. Missing Option: The "Trait Object" Profunctor There is one alternative the response missed, which specifically fits your `rust-fp-library` architecture (HKT simulation). You can solve the "Universal Quantification" storage problem by using a **Trait** instead of a Struct or Function. A trait method can be generic, allowing you to delay the choice of `P`. ```rust // This solves the storage AND composition problem pub trait Optic<S, T, A, B> { // P is generic on the METHOD, not the TRAIT fn run<P>(&self, p: App2<P, A, B>) -> App2<P, S, T> where P: Strong; } // Composition is a struct that implements the Trait struct Composed<O1, O2>(O1, O2); impl<S, A, B, ...> Optic<S, S, B, B> for Composed<O1, O2> { fn run<P>(&self, p: App2<P, A, B>) -> App2<P, S, T> where P: Strong { self.0.run(self.1.run(p)) // P stays polymorphic! } } ``` This is likely the "Holy Grail" for your specific library, as it preserves the Profunctor encoding you wanted while solving the storage issue the AI warned you about. ### Conclusion The response is **correct**. It saves you from implementing a "function-based" system that would hit a dead end as soon as you tried to compose lenses. **Recommendation:** 1. **Accept the response.** 2. If you want the absolute highest integration with your HKT library, try the **Trait-Based Profunctor** approach sketched above. 3. If you want the most ergonomic user experience, use the **Concrete (Struct)** approach recommended by the response.`` Is the response correct, or do you disagree? Does it have inaccuracies or mistakes?

Show more

23:34

This response makes a genuinely important and correct observation — the trait-object profunctor approach using a generic method on a trait — but it also contains a subtle but critical error in that very proposal, and some other issues worth examining.

* * *

What's Correct
--------------

**The Rank-2 closure problem is accurately described.** Rust closures cannot be generic, so `let composed = |p| lens1(lens2(p))` does not work when `p` needs to remain polymorphic. This is real and is the core blocker for function-based profunctor optics.

**The trait-based approach is the most important insight** in this thread so far. The key observation is that generics on a _method_ (rather than the _trait_) defer monomorphisation, which is exactly what you need:

rust

    pub trait Optic<S, T, A, B> {
        fn run<P: Strong>(&self, p: App2<P, A, B>) -> App2<P, S, T>;
    }

This is legitimate Rust — method-level generics work — and composition via a `Composed` struct that delegates to both optics does preserve the polymorphism. This is the key insight the previous two responses missed.

* * *

What's Wrong or Incomplete
--------------------------

### The `Composed` implementation has a type error

Look carefully at the sketch:

rust

    impl<S, A, B, ...> Optic<S, S, B, B> for Composed<O1, O2> {

The type parameters are garbled. For composing `O1: Optic<S, T, M, N>` and `O2: Optic<M, N, A, B>`, you get `Composed: Optic<S, T, A, B>`. The sketch writes `Optic<S, S, B, B>` which is nonsensical — it conflates the outer and inner type parameters. A correct signature would require something like:

rust

    impl<S, T, M, N, A, B, O1, O2> Optic<S, T, A, B> for Composed<O1, O2>
    where
        O1: Optic<S, T, M, N>,
        O2: Optic<M, N, A, B>,

### The trait approach has its own unsolved problem: `dyn Optic`

The response claims this solves "the storage AND composition problem." Composition via a concrete `Composed<O1, O2>` struct is fine, but _storing_ optics heterogeneously (e.g. in a `Vec`, or returning from a function without knowing the concrete type) still requires `dyn Optic<S, T, A, B>`. But `dyn` requires object-safety, and **a trait with a generic method is not object-safe in Rust**. You cannot write `Box<dyn Optic<S, T, A, B>>`. So "storage" is solved only for the monomorphic/static case, not for type-erased storage.

### The concrete struct syntax is overly complicated

The "corrected" syntax:

rust

    pub struct Lens<S, A, G, S2> where G: Fn(&S) -> A, S2: Fn(S, A) -> S { ... }

This is valid but makes the `Lens` type carry the concrete closure types as type parameters, meaning two lenses with different closure implementations have different types entirely. You cannot write a function that accepts any `Lens<S, A>` — you'd need `fn foo<G: Fn(&S) -> A, U: Fn(S, A) -> S>(lens: Lens<S, A, G, U>)` everywhere. This is verbose and limits abstraction. The alternative of boxing the closures (`Box<dyn Fn>`) restores uniformity at the cost of allocation, which the previous response correctly flagged.

* * *

The Honest Summary
------------------

The trait-based profunctor approach is the most promising direction for your library and the most important new idea in this conversation. But the sketch is incomplete and has type errors. The key remaining tension is:

*   **Static dispatch + monomorphic composition** (`Composed<O1, O2>` structs): zero-cost, fully inline-able, but the composed type grows structurally and you cannot store optics abstractly.
*   **Dynamic dispatch** (`Box<dyn Optic>`): uniform storage, but not object-safe due to the generic method, so it doesn't actually work as written.

The resolution is that most practical Rust optics libraries (like `lens-rs`) avoid this by either accepting the monomorphic type-growth, using proc-macros to generate specialised instances, or restricting to concrete optics for the user-facing API while using traits internally. The "Holy Grail" framing in the response is somewhat oversold — the trait approach is real and useful, but it doesn't cleanly resolve all the tensions, and the object-safety wall is a genuine ceiling.
