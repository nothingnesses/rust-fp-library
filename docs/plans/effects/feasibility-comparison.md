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

| Aspect                   | `eff` (Hasura)                     | `purescript-run`                                 | Proposed Rust Design     |
| ------------------------ | ---------------------------------- | ------------------------------------------------ | ------------------------ |
| Core mechanism           | Delimited continuations            | Free monad                                       | Freer monad              |
| Effect dispatch          | O(1) array lookup                  | O(n) peel loop                                   | O(n) peel loop           |
| Open sum                 | Type-level list + array            | Row-polymorphic VariantF                         | Nested Coproduct         |
| Handler install          | `prompt#` + push target            | Recursive interpretation                         | Iterative loop           |
| Multi-shot continuations | Yes (via `control`)                | Yes (tree is re-interpretable; used by `Choose`) | Yes (tree interpretable) |
| Higher-order effects     | Natural (via `locally`, `control`) | Supported (via `locally`-like patterns)          | Needs design work        |
| Stack safety             | Native (RTS handles it)            | `MonadRec` / trampolining                        | Iterative loops (native) |
| Runtime dependency       | GHC RTS                            | None (pure data)                                 | None (pure data)         |
| Feasible in Rust?        | No                                 | Yes                                              | Yes (recommended)        |

## Considered: Can the Missing Pieces Be Filled by `purescript-run`'s Approach? Can We Hybridise?

[eff-analysis.md](./eff-analysis.md) concludes that `switch-resume` can be extended to cover most of eff's semantics (multi-prompt, `control0`-like behavior, scoped operations), but cannot support multi-shot continuations in stable Rust. This raises two follow-up questions addressed here:

1. Does `purescript-run`'s approach actually cover what extended `switch-resume` cannot?
2. Would a hybrid (async-based fast path + data-structure-based multi-shot path) be worthwhile?

### `purescript-run`'s Approach Covers All the Gaps

The free-monad / `Run (Free (VariantF r) a)` design covers every gap identified for `switch-resume`, in stable Rust:

| Gap from extended `switch-resume` | Covered by `purescript-run`'s approach?                                                                                                                                                                                                                                               |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Multiple independent prompts      | Yes. Each handler peels one effect from the row; handlers nest arbitrarily.                                                                                                                                                                                                           |
| `control`/`control0` semantics    | Yes. The handler receives both the effect _and_ the continuation as data; it can inspect, modify, invoke zero/one/many times, or forward to outer handlers.                                                                                                                           |
| Scoped operations (`locally`)     | Yes. `runExcept`, `runReader`, etc. can launch a nested interpreter on a sub-expression and splice the result back.                                                                                                                                                                   |
| **Multi-shot continuations**      | **Yes.** The continuation is an AST node. Interpreting it twice = `k(True)` and `k(False)` in `runChoose`. This is exactly how `purescript-run`'s `Choose` handler works, and it is what makes the free-monad approach fundamentally different from `switch-resume`'s `FnOnce` model. |
| Async coloring                    | Not applicable. Users write effectful code in `Run<R, A>` (a different kind of coloring, but no `.await` required).                                                                                                                                                                   |

The critical row is multi-shot. In the free-monad encoding, the "captured continuation" is not a paused state machine — it is a data structure. Data structures are trivially clonable and re-interpretable. `purescript-run`'s `Choose` handler literally recurses on both the `true` and `false` sub-trees, which is semantically multi-shot.

**Conclusion for Q2**: Yes, `purescript-run`'s approach fully subsumes what extended `switch-resume` can do, and additionally covers the multi-shot case that extended `switch-resume` cannot reach.

### Would a Hybrid Be Worth It?

In principle, a library could expose both mechanisms:

- **Fast path (async, switch-resume-extended)** for tail-resumptive and abortive effects: State, Reader, Writer, Error, one-shot Coroutine. Uses real delimited continuations, zero AST allocation per bind.
- **Multi-shot path (free monad, purescript-run-like)** for NonDet and any other effect that requires re-invoking continuations. Allocates an AST, interprets it.

This is superficially attractive because eff's main performance win over free monads comes from _not_ allocating a tree for every bind. If 95% of real-world effect use is tail-resumptive/abortive, we could get close to eff's performance for the common case while retaining multi-shot for the rare case.

But the hybrid runs into a fundamental technical problem: **you cannot move between encodings at runtime.** When a `NonDet::Choose` is encountered inside async code, the rest of the async computation would need to become a free-monad AST for the handler to re-interpret it. But Rust async state machines are not introspectable and cannot be converted to an AST. The transition has to happen at a _static boundary_ declared in the source code.

What this would look like in practice:

```rust
// Fast-path async: no multi-shot allowed.
async fn fast_computation(ctx: &Ctx) -> i32 { ... }

// Multi-shot region: must be written in Run<R, A> style.
fn multishot_computation() -> Run<Choose, i32> { ... }

// Crossing the boundary requires explicit conversion:
async fn mixed(ctx: &Ctx) -> Vec<i32> {
    let async_result = fast_computation(ctx).await;
    // Switch encodings. The rest of this function cannot call back into async effects.
    run_choose(multishot_computation().with_input(async_result))
}
```

This has several negatives:

1. **User-facing complexity**: programmers must choose which encoding each piece of code uses, based on whether multi-shot is needed downstream.
2. **Viral boundary**: if a function _might_ be called from a multi-shot context, it must be written in the free-monad encoding, even when used in async contexts. Functions that might multi-shot propagate upward through call graphs.
3. **Double maintenance**: built-in effects like `State` would need two implementations (one async, one AST), or one implementation that covers both (loses the performance benefit of the fast path).
4. **Two sets of handler APIs**: documentation and mental model double in size.
5. **Partial feature set in the fast path**: scoped operations like `local` and `catch` are doable but require care in each path.

Given these trade-offs, a hybrid is worth serious consideration only if a specific use case justifies the complexity. For a general-purpose effects library in this codebase, a single approach is far more valuable:

- Users get one consistent mental model.
- Maintenance burden is one code path.
- Documentation covers one system.
- Performance characteristics are predictable.

### Decision: Single-Approach, purescript-run-Style

The recommended path remains: implement a `purescript-run`-style free-monad effects system, as described in [Proposed Architecture](#proposed-architecture) and the [Implementation Roadmap](#implementation-roadmap) above. Reasons:

1. **Covers everything**: including multi-shot, which extended `switch-resume` cannot.
2. **Stable Rust only**: no nightly features, no platform-specific stack manipulation.
3. **No async coloring**: the "coloring" is `Run<R, A>`, which is ordinary monadic code using the library's existing HKT hierarchy.
4. **Existing foundation**: the library already has `Free`, `Functor`/`Monad` traits, and `Coyoneda`. Adding `Coproduct` + `Member` + `Freer` is the main missing piece.
5. **Fits the library's style**: this is a functional-programming library that already leans into "everything in a monad"; free monads are a natural fit.

### Where Does `switch-resume` Still Fit?

`switch-resume` remains useful in two narrow roles:

1. **As an optional interpreter backend.** Users who want their `Run<R, A>` computation to execute with real async semantics (rather than synchronous tree-walking) could run it through a `switch-resume`-based interpreter. This would work fine for effect sets that _don't_ include multi-shot effects. The library could expose `run_async : Run<R, A> -> impl Future<Output = A>` as an alternative to `run_pure`.

2. **For a specific async-native effect** (e.g., a Coroutine handler that wants to be genuinely suspendable in async code rather than producing an AST status). This would be an internal implementation choice of one handler, not a public API concern.

Neither of these changes the core design. They are optional conveniences.

### What About Eventually Supporting Fast-Path Effects?

A future optimisation, after the `purescript-run`-style library is working and benchmarked: identify tail-resumptive effects that do not need the full AST machinery, and compile them to direct calls. `cleff` and `effectful` in Haskell do something similar (storing handler records in a `ReaderT IO` environment and dispatching directly for first-order effects). This is an optimisation, not a design, and should be revisited only if benchmarks show the AST overhead is a real problem for the library's use cases.
