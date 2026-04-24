# Porting `purescript-run` to `fp-library`: Plan, Blockers, and Open Questions

## Scope

This document is the follow-up to [purescript-run-research.md](purescript-run-research.md). That document catalogued the PureScript implementation; this one does three jobs:

1. Records the **feasibility verdict** (Run-style free-monad effects over competing approaches such as `eff`-style delimited continuations).
2. Inventories the **Rust side** (what `fp-library` already supplies, what is insufficient, what is missing).
3. Enumerates the **blockers, draft architecture, and roadmap** that must be resolved or executed before the port can ship.

The goal is not to write code here; it is to produce a complete list of decisions and a sketch of where the implementation is heading. Where a design decision is obvious, it is flagged as such; where it is genuinely open, the tradeoffs are listed.

Sources cross-referenced:

- [purescript-run-research.md](purescript-run-research.md) for what the port must deliver.
- [effects.md](effects.md) for the space of possible approaches.
- [eff-analysis.md](eff-analysis.md) for the delimited-continuations alternative that is ruled out below.
- The following files in `fp-library` for what already exists:
  - [brands.rs](../../../fp-library/src/brands.rs), [kinds.rs](../../../fp-library/src/kinds.rs) (HKT machinery).
  - [functions.rs](../../../fp-library/src/functions.rs) (the user-facing `map`, `bind`, etc.).
  - [types/free.rs](../../../fp-library/src/types/free.rs) (Free monad).
  - [types/coyoneda.rs](../../../fp-library/src/types/coyoneda.rs), [types/coyoneda_explicit.rs](../../../fp-library/src/types/coyoneda_explicit.rs) (Coyoneda variants).
  - [types/control_flow.rs](../../../fp-library/src/types/control_flow.rs) (the `Step`/`ControlFlow` type).
  - [classes/monad_rec.rs](../../../fp-library/src/classes/monad_rec.rs), [classes/natural_transformation.rs](../../../fp-library/src/classes/natural_transformation.rs), and the rest of [classes/](../../../fp-library/src/classes/).
  - [Cargo.toml](../../../fp-library/Cargo.toml) (edition 2024, optional `stacker` and `serde` features).

---

## 1. Feasibility Verdict

Two approaches to extensible effects were considered as porting targets: Hasura's `eff` (delimited continuations) and `purescript-run` (Free monad over extensible variants). The verdict:

- **`eff` is not feasible in stable Rust.** Its core depends on GHC RTS primops (`prompt#`, `control0#`) that have no Rust equivalent. A faithful port would require unsafe stack manipulation with platform-specific assembly, far outside the scope of a library.
- **`purescript-run` is feasible.** It is a pure data-structure approach that requires no runtime support. The main challenge is encoding PureScript's row polymorphism in Rust's type system.

The recommended path: **port the `purescript-run` design**, using `eff`'s handler API and semantics as aspirational design targets where possible.

A key payoff of the Run-style approach is **multi-shot continuations**. Because the "captured continuation" is a data structure (an AST node), it is trivially clonable and re-interpretable. `purescript-run`'s `Choose` handler literally recurses on both the `true` and `false` sub-trees, which is semantically multi-shot. The `switch-resume`-style approach (see [eff-analysis.md](eff-analysis.md)) cannot reach this case in stable Rust because its continuations are paused `FnOnce` state machines.

---

## 2. What the Port Must Deliver

From the research document, a minimum usable port needs:

1. A row-polymorphic effect index `r`. (Open sum over labels and effect functors.)
2. `VariantF`: an open sum of functors indexed by `r`.
3. A monadic wrapper analogous to `Run r a = Free (VariantF r) a`.
4. Handler combinators: `inj`, `prj`, `on`, `case_`, `match`, `expand`, `contract`.
5. Run combinators: `lift`, `send`, `peel`, `resume`, `interpret`, `run`, `runRec`, `runAccum`, `runAccumPure`, `runPure`, `runCont`, `extract`, `expand`.
6. At least three concrete per-effect modules to validate the design (`Reader`, `State`, plus one more; `Writer` or `Except` is the obvious choice).
7. A base-monad bridge (the Rust analogue of `liftEffect`/`runBaseEffect`; presumably interop with `async` runtimes or plain `io::Result`).

---

## 3. Inventory: What `fp-library` Already Has

### 3.1 Already present and directly usable

- **HKT brand system** in [brands.rs](../../../fp-library/src/brands.rs) and [kinds.rs](../../../fp-library/src/kinds.rs). Zero-sized marker brands (`OptionBrand`, `ResultBrand`, etc.) plus a `trait_kind!` macro that generates deterministic hash-named kind traits. The kinds support several signatures, most importantly `Of<'a, A: 'a>: 'a`, which is what effect functors will need.
- **Functor/Monad hierarchy** in [classes/](../../../fp-library/src/classes/): `Functor`, `Pointed`, `Semiapplicative`, `Applicative`, `Semimonad`, `Monad`, plus many auxiliary classes.
- **Natural transformations** in [classes/natural_transformation.rs](../../../fp-library/src/classes/natural_transformation.rs) as a trait `NaturalTransformation<F, G>` with `fn transform<'a, A: 'a>(&self, fa: F<A>) -> G<A>`. This is what `interpret`, `foldFree`, and `hoist` need.
- **`MonadRec` and `ControlFlow`** in [classes/monad_rec.rs](../../../fp-library/src/classes/monad_rec.rs) and [types/control_flow.rs](../../../fp-library/src/types/control_flow.rs). `tail_rec_m(func, initial)` with `ControlFlow::Continue(next)` vs. `ControlFlow::Break(done)` is the direct analogue of PureScript's `tailRecM` with `Step Loop/Done`. `OptionBrand`, `ThunkBrand`, `IdentityBrand` implement `MonadRec`.
- **Free monad** in [types/free.rs](../../../fp-library/src/types/free.rs). "Reflection without remorse" style with a `CatList<Continuation<F>>`, a `FreeView<F>` of `Return(TypeErasedValue)` or `Suspend(...)`, and `Box<dyn Any>`-based erasure. Exposes `pure`, `bind`, `map`, `wrap`, `lift_f`, `resume`, `evaluate`, `hoist_free`, `fold_free`. The `fold_free` signature ([free.rs:774-798](../../../fp-library/src/types/free.rs#L774-L798)) takes a `NaturalTransformation<F, G>` and requires `G: MonadRec + 'static`, which is exactly the shape `runRec` needs.
- **Coyoneda, two variants** in [types/coyoneda.rs](../../../fp-library/src/types/coyoneda.rs) and [types/coyoneda_explicit.rs](../../../fp-library/src/types/coyoneda_explicit.rs). The trait-object version implements `Functor` via the brand system and hides the intermediate type parameter behind `Box<dyn CoyonedaInner>`. The explicit version exposes the intermediate type and fuses maps at compile time. Both support `hoist`.
- **Lazy evaluation primitives**: `Thunk`, `Trampoline`, `Lazy`, memoized versions. These are candidate targets for the interpreter bridge (analogous to PureScript's `Aff`/`Effect`).

### 3.2 Present but insufficient

- **Free is restricted to `'static`.** Because `TypeErasedValue = Box<dyn Any>` and `Any: 'static`, every type that flows through a `Free` value must be `'static` ([free.rs:46-53](../../../fp-library/src/types/free.rs#L46-L53)). This is documented. For effect programs that close over non-`'static` borrowed state (e.g., a `&mut Vec<T>` held by a handler), this is a hard limit.
- **Free is not itself a `Kind`.** It cannot be used as a brand, because its `Of<'a, A>` would have to be `Free<F, A>` for any `'a`, but `Free`'s `'static` bound means there is no sensible `'a` parameter. This blocks nice type signatures like `Free<F>: Monad` for the Functor/Monad trait hierarchy. In the Run port, `Run<R>` will inherit this restriction unless we use a different base.
- **Coyoneda is closed-sum, not open-sum.** It gives us a free `Functor` instance for any type constructor, but does not provide row-polymorphic labelling or partial interpretation. It is adjacent to `VariantF` (both are functor machinery), but solves a different problem.
- **`NaturalTransformation` is not rank-N in the PureScript sense.** It's a trait with `fn transform<'a, A: 'a>(...)` as a method, and the trait itself is a concrete value at the call site. PureScript's `f ~> g = forall x. f x -> g x` is a first-class value; Rust's version is a dispatched trait-object or a generic parameter. This works for `fold_free` but may bite when building handler combinators that need to store or pass natural transformations around.

### 3.3 Entirely missing

- **Row types / type-level open rows.** Nothing in the crate corresponds to PureScript's `Row Type` or `Row (Type -> Type)`. No HList, no tuple-of-brands, no type-level string map. Grepping for `variant`, `row`, `hlist`, `polysum` returns no hits in `fp-library/src`.
- **Open-sum-of-functors (`VariantF`).** No type that holds one of N possible functors with a runtime tag. This is the central missing piece.
- **Type-level labels.** No use of symbol reflection, no const-generic strings, no per-effect marker-trait convention. `TypeId` is used inside `Free` for erasure but is not exposed as a labelling primitive.
- **Handler combinators.** `inj`, `prj`, `on`, `case_`, `match`, `expand`, `send`, `peel`, `resume` (at the Run level), `interpret`, `run`, `runRec`, `runAccum`, `runAccumRec`, `runAccumPure`, `runPure`, `runCont`, `extract` are all absent. Some of these (e.g., `peel`/`resume` for `Free`) exist at the `Free` level under different names (`resume`), but are not lifted to a `Run`-shaped API.
- **Per-effect functors.** No `Reader<E>`, `State<S>`, `Writer<W>`, `Except<E>`, `Choose`. All need to be written.
- **`Run` newtype.** No wrapper type, no handler-composition pipeline, no derived Monad for it.
- **Base-monad bridge.** No equivalent of `liftEffect`/`runBaseEffect`/`runBaseAff`. The Rust choice for "the base" is itself an open question.

---

## 4. Blockers: Decisions That Must Be Made Before Coding

Each blocker below must be resolved before writing any new types. They are ordered by how much downstream code they influence.

### 4.1 BLOCKER: How to represent the effect row

This is _the_ decision. Every other question flows from it. Rust has no row types. There are four plausible encodings, each with a different cost profile:

1. **Type-level heterogeneous list (HList) of brand-label pairs / nested coproduct.**
   Example: `Run<(StateBrand<i32>, (ReaderBrand<Env>, Nil))>` or, as a nested coproduct, `Coproduct<State<i32>, Coproduct<Reader<Env>, Void>>`. Needs trait-based membership and removal (`Member<Effect, Index>`, `Remove<Brand>`). Gives open composition. Similar to `frunk::HList`; similar to `freer-simple` and `polysemy` in Haskell. Pros: fully extensible, type-safe, no macros required. Cons: deep nesting produces complex types; error messages degrade with depth; trait-resolution overhead at compile time.
2. **Closed tuple of brands with a fixed arity.**
   Example: `Run<(StateBrand<i32>, ReaderBrand<Env>)>`. Simple, good error messages, but loses openness: you cannot write `speak :: forall r. String -> Run (TALK + r) Unit` because there is no way to say "a tuple with at least this entry". You'd need a sum-of-supertypes or macro wrappers.
3. **Single wide enum with one variant per effect.**
   Example: `enum AppEffect { State(...), Reader(...) }`. Users must define this enum up front, closing the world. Violates the first principle of `Run` (open composition). Rejected on design grounds but mentioned for completeness.
4. **Trait-object dispatch with `TypeId` tags.**
   Example: `Run` holds `Box<dyn Any>` plus a `TypeId` for tag. Allows full openness and dynamic dispatch. Costs: boxing per effect invocation, runtime type lookup, loss of static exhaustiveness checks. Closest match to the PureScript runtime representation (which is literally `{ type: String, value, map }`).
5. **Hybrid: nested coproduct + macro sugar for construction.**
   Use the nested coproduct internally but expose ergonomic macros: `type MyEffects = coprod![State<i32>, Reader<String>];` expanding to the nested form. This is the default direction for the draft architecture in section 5.

**Open questions under this blocker:**

- Is it acceptable to require the user to write their effect set as a type-level list even for a 3-effect program? PureScript uses row sugar `(state :: State s, reader :: Reader e | r)`; there is no sugar in Rust.
- Can a macro (`effects![State<i32>, Reader<Env>]` / `coprod![...]`) make the coproduct approach ergonomic enough?
- Is exhaustiveness checking (compile-time "you forgot to handle the `state` effect") a hard requirement? Option 4 can't give that; options 1-3 can.
- Do we need a `Lacks` constraint (prevents duplicate labels in a row)? PureScript's row system has this built in; Rust needs trait-based emulation.

**Leaning:** the hybrid (option 5) is the default; options 1 and 4 remain viable fallbacks. Build a minimal prototype before committing.

### 4.2 BLOCKER: Functor dictionary for VariantF

PureScript's `VariantFRep` stores `map :: Mapper f` alongside the value. This lets `VariantF`'s own `map` dispatch to whichever effect functor is currently active. In Rust:

- **Static option:** every effect type in the row must satisfy a `Functor` bound. The row representation must preserve this bound. Natural for the HList/coproduct encoding (option 1/5 above); awkward for option 4.
- **Dynamic option:** store a `Box<dyn Fn(Box<dyn FnOnce(A) -> B>, ...) -> ...>` in the `VariantF` value. Works for any row encoding, but is the most boxed possible implementation.
- **Freer option:** drop the functor requirement altogether. See section 5.2.

Does the Rust `Functor` trait's `map` signature (`fn map<'a, A, B>(f: impl Fn(A) -> B + 'a, fa: F<A>) -> F<B>`) let us store the function pointer without knowing `A` and `B` ahead of time? Probably not without erasure, which defeats the point of `VariantF`'s generic parameter.

**Open question:** is it acceptable to force all effect functors to implement a supplementary `DynFunctor` trait that hides the type parameters behind `dyn Any`? This is the practical consequence of the dynamic option and needs explicit acknowledgment.

### 4.3 BLOCKER: How strong should stack-safety guarantees be

The existing `Free` is stack-safe (O(1) bind, iterative drop via `Extract`). That is sufficient for `Run`'s own stack-safety. But the PureScript library distinguishes two interpreter families:

- `interpret` / `run` / `runAccum`: assume the target monad is stack-safe.
- `interpretRec` / `runRec` / `runAccumRec`: require `MonadRec` on the target.

In Rust, this distinction is less useful: most target monads we'd write (`Option`, `Result`, `Thunk`) already implement `MonadRec` or trivially can. The open question:

- Do we ship both families, mirroring PureScript 1:1? Easier to document but doubles the surface area.
- Do we ship only the `MonadRec` family and make every interpreter stack-safe by default? Simpler, costs a few percent in common cases.

**Recommendation (not a decision):** ship only the `MonadRec` family. Revisit if we find target monads that cannot implement it.

### 4.4 BLOCKER: The `'static` bound inherited from `Free`

Every effect functor and every effect value that flows through `Run` will be `'static` if we keep the current `Free`. Consequences:

- Users cannot define effects that hold borrowed references (e.g., `State<&'a mut Vec<T>>`).
- Handlers cannot close over non-`'static` environment data.
- All effect payloads become owned (`String` not `&str`, `Vec<T>` not `&[T]`).

Options:

- **Accept the tax.** Document prominently that `Run` is for `'static` effects. Simplest; matches the existing `Free`.
- **Write a non-`'static` Free.** Replace `Box<dyn Any>` with an existential that carries the lifetime. Very hard in practice.
- **Parameterize `Run` over a lifetime.** `Run<'a, R, A>` where each `F in R` has `F: Functor<'a>`. Trades API complexity for expressive power. Compatible with [coyoneda_explicit.rs](../../../fp-library/src/types/coyoneda_explicit.rs), which already supports `'a`.

**Open question:** is there enough demand for non-`'static` effects to justify the cost? Probably not for a first release.

### 4.5 BLOCKER: Natural transformations as values

`interpret` takes a natural transformation `VariantF r ~> m` as a runtime value. In PureScript this is a polymorphic function. In Rust:

- The existing `NaturalTransformation<F, G>` trait works for `F` with a statically-known type. But `VariantF r` is an _open_ sum; its concrete representation changes with `r`.
- A natural transformation from `VariantF r` must, by construction, handle every case in `r`. In PureScript this is assembled with `case_ # on _reader handleReader # on _state handleState`. The `on` combinator threads the "smaller row" through the type of the remaining fallback.
- In Rust, the equivalent is probably a tuple-of-closures (one per effect) indexed by the same type-level structure as the row, produced by a macro (`handlers! { state: handleState, reader: handleReader }`).

**Open question:** will users build natural transformations by hand, or only via a macro? A macro-based DSL is the realistic answer, but it pushes complexity into the macro layer.

---

## 5. Draft Architecture (Recommended Direction)

The blockers above resolve, for the current working hypothesis, to the following shape. This is a **draft**; prototype first.

### 5.1 Core types

```
Run<Effects, A> = Freer<Coproduct<...effects...>, A>

where
  Freer<F, A>        = Pure(A) | Impure(F_erased, continuation)
  Coproduct<H, T>    = Here(H) | There(T)
  Void               = empty-tail of the coproduct
  Member<E, Effects> = trait proving E is somewhere in the coproduct
```

The user-facing type constructor is `Run<Effects, A>`. The `Effects` parameter is a nested `Coproduct` (possibly produced by a `coprod!` macro). `Freer` replaces `Free` to eliminate the per-effect `Functor` requirement; the cost is internal use of `Box<dyn Any>` for the existential intermediate.

### 5.2 Why Freer rather than Free + Coyoneda

The decision between:

- **Standard Free + explicit `Functor` bound** on each effect (with `Coyoneda` as a helper for effects that aren't naturally functors).
- **Freer** (existential continuation, no `Functor` requirement).

favours Freer for a first cut. Reasons: effects become plain enums with no derive gymnastics; it aligns with `freer-simple` (a known-working Haskell library); the `Box<dyn Any>` cost is acceptable for a first implementation. Coyoneda is still available as a later optimisation if the functor route proves more ergonomic with the Brand system.

### 5.3 Effect definition pattern

```rust
// Effects are plain enums (no functor instance needed with Freer).
enum State<S> {
    Get,        // returns S
    Put(S),     // returns ()
}

enum Reader<E> {
    Ask,        // returns E
}

enum Except<E> {
    Throw(E),   // returns !
}
```

### 5.4 Handler pattern

```rust
fn run_state<S, R, A>(
    initial: S,
    program: Run<Coprod![State<S>, ...R], A>,
) -> Run<R, (S, A)> {
    let mut state = initial;
    let mut current = program;
    loop {
        match current.peel() {
            RunStep::Pure(a) => return Run::pure((state, a)),
            RunStep::Impure(Coproduct::Here(state_op), k) => {
                match state_op {
                    State::Get    => current = k(state.clone()),
                    State::Put(s) => { state = s; current = k(()); }
                }
            }
            RunStep::Impure(Coproduct::There(other), k) => {
                current = Run::impure(other, k);   // forward
            }
        }
    }
}
```

Handler composition then works as a pipeline that removes one effect from the row at each stage, mirroring PureScript's `# runReader env # runState 0 # extract`.

---

## 6. Implementation Roadmap

### Phase 1: Core machinery

1. `Coproduct<H, T>` and `Void` types.
2. `Member<E, Index>` trait for injection/projection with type-level index.
3. `Freer<F, A>` with existential continuation.
4. `Run<Effects, A>` as `Freer<Coproduct<...>, A>`.
5. `peel` / `send` / `pure` core operations.
6. Convenience macros: `coprod![]` for type construction, `effects![]` if needed.

### Phase 2: Interpretation

1. `run` / `runPure` (iterative interpretation loop; already stack-safe in Rust).
2. `runAccum` (interpretation with threaded state).
3. `interpret` (natural-transformation-style).
4. Stack-safe variants only if an actual target monad needs them.

### Phase 3: Built-in effects

1. `State` (get, put, modify, runState).
2. `Reader` (ask, asks, local, runReader).
3. `Except` (throw, catch, runExcept).
4. `Writer` (tell, censor, runWriter).
5. `Choose` (empty, alt, runChoose; validates multi-shot).

### Phase 4: Integration

1. Bridge to existing Monad/Functor hierarchy if the `'static` limitation is resolved.
2. Brand for `Run` to enable use with existing HKT-polymorphic code.
3. Consider whether optics can be used as effect accessors (profunctor-based effect projection).

---

## 7. Non-Blocking Tasks (Mostly Mechanical)

Once the blockers in section 4 are resolved and the core machinery from 6.1 exists, the following are straightforward.

- **`Run` newtype** wrapping the chosen core.
- **Per-effect enums.** Direct translation from PureScript's `data State s a = ...`.
- **Smart constructors.** `ask`, `get`, `put`, `modify`, `tell`, `throw`, `catch`. Each is a thin wrapper over `inj + lift_f`/`send`.
- **`extract :: Run () a -> a`.** Trivial once the empty row type is defined.
- **`expand`.** One-line `unsafe fn` using `mem::transmute` once the row constraints prove subsetting.
- **Base-monad bridge.** A `liftEffect`-analog for any target monad we care about. The first target should probably be `Thunk` or `Identity` (pure), with `async fn` as a followup.
- **Error messages.** Rust's error messages on trait-heavy type machinery are legendary. Budget time for macro-generated human-readable errors.

---

## 8. Hybrid and `switch-resume` Considerations

A natural question: could we expose a "fast path" based on `switch-resume`-style extended async (no AST allocation per bind) for tail-resumptive effects, plus the Freer/Run path for multi-shot? In principle yes. In practice the hybrid runs into a fundamental technical problem: **you cannot move between encodings at runtime.** When a `NonDet::Choose` is encountered inside async code, the rest of the async computation would need to become a free-monad AST for the handler to re-interpret it. Rust async state machines are not introspectable and cannot be converted to an AST. The transition has to happen at a static boundary declared in source code.

Consequences of pursuing the hybrid anyway:

1. **User-facing complexity.** Programmers must choose which encoding each piece of code uses, based on whether multi-shot is needed downstream.
2. **Viral boundary.** If a function _might_ be called from a multi-shot context, it must be written in the free-monad encoding, propagating upward.
3. **Double maintenance.** Built-in effects like `State` would need two implementations, or one implementation that covers both (losing the fast-path performance benefit).
4. **Two handler APIs.** Documentation and mental model double in size.

For a general-purpose effects library in this codebase, a single approach is far more valuable: one consistent mental model, one maintenance path, one documentation story, predictable performance.

**Decision: single-approach, `purescript-run`-style.** `switch-resume`-like machinery retains two narrow optional roles:

1. As an optional interpreter backend (`run_async : Run<R, A> -> impl Future<Output = A>`) for effect sets that do not include multi-shot effects.
2. As the internal implementation of a specific async-native effect (e.g., a genuinely suspendable Coroutine handler). This is a handler choice, not a public API change.

Neither changes the core design.

---

## 9. Open Questions That Should Be Answered Before Starting

These are not full blockers (they don't prevent the first line of code) but they should have answers recorded before the port is attempted.

1. **What is the target audience?** A library author building effect-heavy internal tooling has different needs than an application developer. The row-encoding choice in 4.1 may change based on this.
2. **Do we want partial interpretation?** The ability to write `runReader: Run (READER + r) a -> Run r a` is the whole point of `Run`. If we relax this (only full interpretation), we can drop row polymorphism and the problem becomes much easier. Confirm we want the full version.
3. **How does `Run` interact with `async`?** PureScript's `Aff` maps onto async Rust. Do we want an `async`-aware interpreter from day one, or is a synchronous interpreter acceptable?
4. **What's the story for `IO`/effects-with-side-effects?** The PureScript `Effect` monad has no direct Rust analog. Options: `Thunk`, a custom `Effect` type, `std::io::Result`, or lean on `async` from the start. Needs a named owner.
5. **Higher-order effects.** `purescript-run` supports `local` (Reader) and `catch` (Error), which take effectful computations as arguments. In the Freer encoding these require special handling. `eff` solves this with `locally` / `control`; a Freer-based system needs a different approach (possibly "hefty algebras" or explicit scoping).
6. **Performance.** Freer allocates a closure per bind. Benchmark against a direct non-effectful implementation to establish a baseline.
7. **Lifetime constraints.** Can the Freer monad's `Box<dyn FnOnce(...) -> ...>` work without `'static`? If not, effects carrying references (e.g., `Reader<&str>`) will not work. May force all effect data to be owned.
8. **Macro infrastructure.** Will the port introduce a `define_effect!` macro that generates the enum, smart constructors, and label type? Almost certainly yes. Who owns the macro design?
9. **Testing strategy.** Port the canonical `TalkF` + `DinnerF` example from [test/Examples.purs](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106) as an integration test to validate the design.

---

## 10. Comparison Table (Approaches and Proposed Rust Design)

| Aspect                   | `eff` (Hasura)                     | `purescript-run`                                 | Proposed Rust design     |
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

---

## 11. Cross-Reference Table: PureScript Piece to Rust Status

| PureScript piece                                                                              | Rust counterpart in `fp-library` today                             | Status                                       |
| --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- |
| `Free f a`                                                                                    | [`Free<F, A>`](../../../fp-library/src/types/free.rs)              | Present, `'static`-only.                     |
| `liftF`                                                                                       | `Free::lift_f`                                                     | Present.                                     |
| `foldFree`                                                                                    | `Free::fold_free`                                                  | Present, requires `G: MonadRec + 'static`.   |
| `hoistFree`                                                                                   | `Free::hoist_free`                                                 | Present.                                     |
| `resume` / `resume'`                                                                          | `Free::resume`                                                     | Present.                                     |
| `MonadRec`, `tailRecM`, `Step`                                                                | `MonadRec`, `tail_rec_m`, `ControlFlow`                            | Present.                                     |
| `TypeEquals`, `to`, `from`                                                                    | Nothing direct. Rust generics + `PhantomData` cover it implicitly. | N/A by design.                               |
| `Newtype` class                                                                               | Nothing. Rust newtypes need no abstraction.                        | N/A by design.                               |
| `Natural transformation (~>)`                                                                 | `NaturalTransformation<F, G>` trait                                | Present.                                     |
| `Variant` (non-functor)                                                                       | Absent.                                                            | Missing, not needed for Run.                 |
| `VariantF`                                                                                    | Absent.                                                            | Missing; central blocker.                    |
| Row `Row (Type -> Type)`                                                                      | Absent.                                                            | Missing; central blocker.                    |
| `inj`, `prj`, `on`, `case_`, `match`, `expand`, `contract`                                    | Absent.                                                            | Missing.                                     |
| Row constraints (`Cons`, `Union`, `Lacks`)                                                    | Absent.                                                            | Missing; needs trait-based emulation.        |
| `IsSymbol`, `Proxy "label"`                                                                   | Absent.                                                            | Missing; options in blocker 4.1.             |
| `Run r a`                                                                                     | Absent.                                                            | Missing.                                     |
| `lift`, `send`, `peel`, `resume` (Run level)                                                  | Absent (exists at Free level).                                     | Missing.                                     |
| `interpret`, `run`, `runRec`, `runAccum`, `runAccumRec`, `runPure`, `runAccumPure`, `runCont` | Absent.                                                            | Missing.                                     |
| `Run.Reader`, `Run.State`, `Run.Writer`, `Run.Except`, `Run.Choose`                           | Absent.                                                            | Missing (mechanical once `Run` exists).      |
| `liftEffect`, `runBaseEffect`, `liftAff`, `runBaseAff`                                        | Absent.                                                            | Missing; target choice is itself a question. |

---

## 12. Summary

`fp-library` has everything it needs for the "free monad + stack-safe recursion + natural transformation" substrate. The Rust equivalents of `Free`, `MonadRec`, `Step`, and `NaturalTransformation` are already in place and close enough to the PureScript shape that `fold_free` is effectively `runRec` already.

What is missing is the **row-polymorphic open sum** (`VariantF` and its supporting type-level machinery). Nothing in the crate today solves that problem, and Rust does not give us the solution for free.

The three hard blockers in order:

1. **Row encoding** (section 4.1). HList / coproduct / tuple / `TypeId` dispatch. Every other piece of the port is shaped by this.
2. **Functor dictionary dispatch** (section 4.2). Static bound vs dynamic box vs Freer (which sidesteps the problem). Choice follows from 4.1.
3. **`'static` bound on Free** (section 4.4). Accept, replace, or parameterize. Independent of 4.1 but bounds the scope of what users can do.

The other open questions (async story, macro design, exhaustiveness trade-offs) are secondary and can be deferred until a prototype exists.

**Recommended next action:** build a proof-of-concept with exactly two effects (`Reader<Env>` and `State<i32>`) using the nested-coproduct + Freer encoding described in section 5, no macros, and validate whether the resulting API is recognizable as "extensible algebraic effects". If yes, proceed to flesh out; if no, revisit blocker 4.1 before anything else.
