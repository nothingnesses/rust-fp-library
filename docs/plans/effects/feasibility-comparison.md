# Feasibility Comparison: Extensible Effects for rust-fp-lib

## Summary

This document compares the two candidate approaches for implementing extensible effects in this library, based on analysis of the Hasura `eff` (Haskell) and `purescript-run` (PureScript) codebases. It evaluates feasibility against what the library already provides and what would need to be built.

## Verdict

**The `eff` approach (delimited continuations) is not feasible for Rust.** Its core depends on GHC RTS primops (`prompt#`, `control0#`) that have no Rust equivalent. A faithful port would require unsafe stack manipulation with platform-specific assembly, far outside the scope of a library.

**The `purescript-run` approach (Free monad + extensible variants) is feasible.** It is a pure data structure approach that requires no runtime support. The main challenge is encoding PureScript's row polymorphism, which is the source of `purescript-run`'s ergonomics, in Rust's type system.

The recommended path: **port the `purescript-run` design**, using `eff`'s handler API and semantics as aspirational design targets where possible.

## What the Library Already Has

| Building Block                   | Status              | Location                                       | Notes                                                      |
| -------------------------------- | ------------------- | ---------------------------------------------- | ---------------------------------------------------------- |
| HKT encoding (Brands)            | Available           | `brands.rs`, `kinds.rs`                        | Brand pattern with `Kind` traits                           |
| Functor/Monad hierarchy          | Available           | `classes/functor.rs`, `classes/monad.rs`, etc. | Complete hierarchy                                         |
| Free monad                       | Partially available | `types/free.rs`                                | Has `'static` limitation from `Box<dyn Any>` continuations |
| Coyoneda                         | Available           | `types/coyoneda.rs`                            | Free functor; could help with functor requirement          |
| Profunctor optics                | Available           | `types/optics.rs`, `classes/profunctor/`       | Shows the library can handle complex HKT patterns          |
| Extensible variants / coproducts | **Missing**         | N/A                                            | Critical gap                                               |
| Row polymorphism                 | **Missing**         | N/A                                            | Must be approximated                                       |
| MonadRec / trampolining          | Available           | `types/free.rs` uses CatList                   | Stack-safe recursion exists                                |

## The Three Key Problems

### Problem 1: Extensible Variants (VariantF)

PureScript's `VariantF r a` is an open, row-indexed sum of functors. This is the heart of `purescript-run`: it allows a computation to declare which effects it uses via a row type, and handlers to peel off one effect at a time.

**Options for Rust:**

**A. Type-level list + nested Either (coproduct)**

Represent `VariantF (state :: State s, reader :: Reader e | r)` as a nested coproduct:

```rust
// Coproduct<State<S>, Coproduct<Reader<E>, Rest>>
enum Coproduct<H, T> {
    Here(H),
    There(T),
}
enum Void {} // empty coproduct tail
```

Injection/projection via traits:

```rust
trait Member<Effect, Index> {
    fn inject(e: Effect) -> Self;
    fn project(self) -> Result<Effect, Self>;
}
```

Where `Index` is a type-level Peano number (Z, S<Z>, S<S<Z>>, ...) disambiguating which position in the coproduct the effect lives at.

Pros: Fully extensible, type-safe, no macros required.
Cons: Deep nesting produces complex types. Error messages degrade with depth. Trait resolution overhead at compile time.

**B. Macro-generated closed enum**

A proc macro generates a concrete enum for each effect combination:

```rust
effects! {
    enum MyEffects {
        State(State<i32>),
        Reader(Reader<String>),
    }
}
```

Pros: Simple types, good error messages, fast compilation.
Cons: Not extensible; every effect combination needs its own enum. Cannot write code polymorphic over "any effect set containing State."

**C. Hybrid: nested coproduct with macro sugar**

Use the nested coproduct internally but provide macros for ergonomic construction:

```rust
type MyEffects = coprod![State<i32>, Reader<String>];
// Expands to: Coproduct<State<i32>, Coproduct<Reader<String>, Void>>
```

Pros: Combines extensibility with usability.
Cons: Still has deep-nesting issues, but macros hide most of the pain.

**Recommendation:** Option C (hybrid). The nested coproduct is the standard encoding used by Haskell libraries (`freer-simple`, `polysemy`) that lack PureScript's row polymorphism. Macros can provide comparable ergonomics.

### Problem 2: Row Polymorphism / Effect Subsets

PureScript's row polymorphism allows:

```purescript
myFunc :: forall r. Run (STATE Int + r) a
```

This says "myFunc uses State, plus whatever other effects `r` contains." The `r` is universally quantified, so `myFunc` works with _any_ effect set that includes State.

In Rust, the equivalent is a trait bound:

```rust
fn my_func<Effects>() -> Run<Effects, A>
where
    Effects: Member<State<i32>>,
```

The `Member` trait proves that `State<i32>` exists somewhere in the `Effects` coproduct. This is how `freer-simple` and similar Haskell libraries work (using the `:>` / `Member` type class).

For handler composition (peeling off one effect), we need:

```rust
fn run_state<S, Effects, A>(
    initial: S,
    program: Run<Coproduct<State<S>, Effects>, A>,
) -> Run<Effects, (S, A)>
```

This works naturally with the nested coproduct: the handler matches `Coproduct::Here(state_op)` and wraps unhandled effects as `Coproduct::There(rest)`.

**Key insight:** The nested coproduct encoding gets "peel off one effect" for free via pattern matching on `Here` / `There`. This is the main reason it works well for effects despite lacking true row polymorphism.

### Problem 3: Free Monad + HKT Integration

The existing `Free<F, A>` implementation uses `Box<dyn Any>` for type-erased continuations, which requires `'static` bounds. This conflicts with the library's HKT Brand system, which uses lifetime-parameterized associated types.

**Options:**

**A. Fix the existing Free to work with Brands**

Rework the continuation queue to avoid `Box<dyn Any>` or find a way to coexist with lifetime parameters.

**B. Build a new Free specifically for effects**

A simpler `Free` that does not need to integrate with the general Brand/Kind system:

```rust
enum Free<F, A> {
    Pure(A),
    Impure(F, Box<dyn FnOnce(???) -> Free<F, A>>),
}
```

The challenge is the `???`: in the standard free monad, `F` is a functor and `Impure` holds `F(Free<F, A>)`. In the freer encoding, it holds `F(X)` for some existential `X` plus a continuation `X -> Free<F, A>`.

**C. Use the Freer encoding**

The freer monad eliminates the Functor requirement:

```rust
enum Freer<F, A> {
    Pure(A),
    Impure {
        effect: F,  // type-erased effect operation
        continuation: Box<dyn FnOnce(Box<dyn Any>) -> Freer<F, A>>,
    },
}
```

This is what `freer-simple` uses. Effects are plain data types (no Functor needed). The existential intermediate type is erased via `Box<dyn Any>`.

Pros: Simpler effect definitions (no functor instance needed). Closer to `freer-simple` than `purescript-run`, but the API can still follow `purescript-run`'s handler patterns.
Cons: Requires `Box<dyn Any>` + downcasting, losing some type safety at the boundary.

**D. Use the standard Free with Coyoneda**

Wrap each effect functor in `Coyoneda` to satisfy the Functor requirement automatically:

```rust
// Free<Coyoneda<VariantF<Effects>>, A>
```

This is what `purescript-run` effectively does (PureScript's `VariantF` is already a functor via derived instances, but Coyoneda generalizes this).

**Recommendation:** Start with option C (Freer encoding). It is the simplest to implement, avoids functor boilerplate, and the `Box<dyn Any>` cost is acceptable for a first implementation. Option D is a possible optimization later if the functor encoding proves more ergonomic with the existing Brand system.

## Proposed Architecture

```
Run<Effects, A> = Freer<Coproduct<...effects...>, A>

where:
  Freer<F, A> = Pure(A) | Impure(F_erased, continuation)
  Coproduct<H, T> = Here(H) | There(T)
  Member<E, Effects> = trait proving E is in the coproduct
```

### Handler Pattern

```rust
fn run_state<S, R, A>(initial: S, program: Run<Coprod![State<S>, ...R], A>) -> Run<R, (S, A)> {
    let mut state = initial;
    let mut current = program;
    loop {
        match current.peel() {
            RunStep::Pure(a) => return Run::pure((state, a)),
            RunStep::Impure(Coproduct::Here(state_op), k) => {
                match state_op {
                    State::Get => { current = k(state.clone()); }
                    State::Put(s) => { state = s; current = k(()); }
                }
            }
            RunStep::Impure(Coproduct::There(other), k) => {
                current = Run::impure(other, k);  // forward
            }
        }
    }
}
```

### Effect Definition Pattern

```rust
// Effects are plain enums (no functor instance needed with Freer)
enum State<S> {
    Get,       // returns S
    Put(S),    // returns ()
}

enum Reader<E> {
    Ask,       // returns E
}

enum Except<E> {
    Throw(E),  // returns ! (never)
}
```

## Implementation Roadmap

### Phase 1: Core Machinery

1. **Coproduct type** with `Here`/`There` variants.
2. **Member trait** for injection/projection with type-level index.
3. **Freer monad** with existential continuation.
4. **Run type** as `Freer<Coproduct<...>, A>`.
5. **peel / send / pure** core operations.
6. Convenience macros: `coprod![]` for type construction, `effects![]` if needed.

### Phase 2: Interpretation

1. **run / runPure** (iterative interpretation loop).
2. **runAccum** (interpretation with threaded state).
3. **interpret** (natural-transformation-style).
4. Stack-safe variants if needed (Rust's iterative loops are already stack-safe).

### Phase 3: Built-in Effects

1. **State** (get, put, modify, runState).
2. **Reader** (ask, asks, local, runReader).
3. **Except** (throw, catch, runExcept).
4. **Writer** (tell, censor, runWriter).
5. **Choose** (empty, alt, runChoose).

### Phase 4: Integration

1. Bridge to existing Monad/Functor hierarchy if the `'static` limitation is resolved.
2. Brand for `Run` to enable use with existing HKT-polymorphic code.
3. Consider whether optics can be used as effect accessors (profunctor-based effect projection).

## Open Questions

1. **Lifetime constraints**: Can the Freer monad's `Box<dyn FnOnce(...) -> ...>` work without `'static`? If not, effects carrying references (e.g., `Reader<&str>`) will not work. This may force all effect data to be owned.

2. **Higher-order effects**: `purescript-run` supports `local` (Reader) and `catch` (Error), which take effectful computations as arguments. In the Freer encoding, these require special handling. `eff` solves this elegantly with `locally` and `control`; a Freer-based system needs a different approach (possibly "hefty algebras" or explicit scoping).

3. **Performance**: The Freer encoding allocates a closure per bind. For effect-heavy code this may be significant. Benchmarking against a direct (non-effectful) implementation will be needed.

4. **Interaction with async**: Rust's async ecosystem is pervasive. Can `Run` computations be interpreted into `Future`s? This would be a major ergonomic win.

5. **Macro ergonomics**: How much macro sugar is needed to make the system pleasant to use? PureScript's row polymorphism does a lot of heavy lifting that macros would need to approximate.

## Comparison Table

| Aspect                   | `eff` (Hasura)                     | `purescript-run`                        | Proposed Rust Design     |
| ------------------------ | ---------------------------------- | --------------------------------------- | ------------------------ |
| Core mechanism           | Delimited continuations            | Free monad                              | Freer monad              |
| Effect dispatch          | O(1) array lookup                  | O(n) peel loop                          | O(n) peel loop           |
| Open sum                 | Type-level list + array            | Row-polymorphic VariantF                | Nested Coproduct         |
| Handler install          | `prompt#` + push target            | Recursive interpretation                | Iterative loop           |
| Multi-shot continuations | Yes (via `control`)                | No (Free is one-shot)                   | No                       |
| Higher-order effects     | Natural (via `locally`, `control`) | Supported (via `locally`-like patterns) | Needs design work        |
| Stack safety             | Native (RTS handles it)            | `MonadRec` / trampolining               | Iterative loops (native) |
| Runtime dependency       | GHC RTS                            | None (pure data)                        | None (pure data)         |
| Feasible in Rust?        | No                                 | Yes                                     | Yes (recommended)        |
