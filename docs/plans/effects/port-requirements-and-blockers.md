# Porting `purescript-run` to `fp-library`: Requirements, Open Questions, and Blockers

## Scope

This document is the follow-up to [purescript-run-research.md](purescript-run-research.md). That document catalogued the PureScript implementation; this one inventories the Rust side (the current `fp-library` crate) and enumerates the concrete issues, open questions, and blockers that must be resolved before a port can proceed.

The goal is not to design the port. It is to produce a complete list of decisions that must be made, with enough context on each decision to start work. Where a design decision is obvious, it is flagged as such; where it is genuinely open, the tradeoffs are listed.

Sources cross-referenced:

- [purescript-run-research.md](purescript-run-research.md) for what the port must deliver.
- The following files in `fp-library` for what already exists:
  - [brands.rs](../../../fp-library/src/brands.rs), [kinds.rs](../../../fp-library/src/kinds.rs) (HKT machinery).
  - [functions.rs](../../../fp-library/src/functions.rs) (the user-facing `map`, `bind`, etc.).
  - [types/free.rs](../../../fp-library/src/types/free.rs) (Free monad).
  - [types/coyoneda.rs](../../../fp-library/src/types/coyoneda.rs), [types/coyoneda_explicit.rs](../../../fp-library/src/types/coyoneda_explicit.rs) (Coyoneda variants).
  - [types/control_flow.rs](../../../fp-library/src/types/control_flow.rs) (the `Step`/`ControlFlow` type).
  - [classes/monad_rec.rs](../../../fp-library/src/classes/monad_rec.rs), [classes/natural_transformation.rs](../../../fp-library/src/classes/natural_transformation.rs), and the rest of [classes/](../../../fp-library/src/classes/).
  - [Cargo.toml](../../../fp-library/Cargo.toml) (edition 2024, MSRV implied, optional `stacker` and `serde` features).

---

## 1. What the Port Must Deliver (Recap)

From the research document, a minimum usable port needs:

1. A row-polymorphic effect index `r`. (Open sum over labels and effect functors.)
2. `VariantF`: an open sum of functors indexed by `r`.
3. A monadic wrapper analogous to `Run r a = Free (VariantF r) a`.
4. Handler combinators: `inj`, `prj`, `on`, `case_`, `match`, `expand`, `contract`.
5. Run combinators: `lift`, `send`, `peel`, `resume`, `interpret`, `run`, `runRec`, `runAccum`, `runAccumPure`, `runPure`, `runCont`, `extract`, `expand`.
6. At least three concrete per-effect modules to validate the design (`Reader`, `State`, plus one more; `Writer` or `Except` is the obvious choice).
7. A base-monad bridge (the Rust analogue of `liftEffect`/`runBaseEffect`; presumably interop with `async` runtimes or plain `io::Result`).

---

## 2. Inventory: What `fp-library` Already Has

### 2.1 Already present and directly usable

- **HKT brand system** in [brands.rs](../../../fp-library/src/brands.rs) and [kinds.rs](../../../fp-library/src/kinds.rs). Zero-sized marker brands (`OptionBrand`, `ResultBrand`, etc.) plus a `trait_kind!` macro that generates deterministic hash-named kind traits. The kinds support several signatures, most importantly `Of<'a, A: 'a>: 'a`, which is what effect functors will need.
- **Functor/Monad hierarchy** in [classes/](../../../fp-library/src/classes/): `Functor`, `Pointed`, `Semiapplicative`, `Applicative`, `Semimonad`, `Monad`, plus many auxiliary classes.
- **Natural transformations** in [classes/natural_transformation.rs](../../../fp-library/src/classes/natural_transformation.rs) as a trait `NaturalTransformation<F, G>` with `fn transform<'a, A: 'a>(&self, fa: F<A>) -> G<A>`. This is what `interpret`, `foldFree`, and `hoist` need.
- **`MonadRec` and `ControlFlow`** in [classes/monad_rec.rs](../../../fp-library/src/classes/monad_rec.rs) and [types/control_flow.rs](../../../fp-library/src/types/control_flow.rs). `tail_rec_m(func, initial)` with `ControlFlow::Continue(next)` vs. `ControlFlow::Break(done)` is the direct analogue of PureScript's `tailRecM` with `Step Loop/Done`. `OptionBrand`, `ThunkBrand`, `IdentityBrand` implement `MonadRec`.
- **Free monad** in [types/free.rs](../../../fp-library/src/types/free.rs). "Reflection without remorse" style with a `CatList<Continuation<F>>`, a `FreeView<F>` of `Return(TypeErasedValue)` or `Suspend(...)`, and `Box<dyn Any>`-based erasure. Exposes `pure`, `bind`, `map`, `wrap`, `lift_f`, `resume`, `evaluate`, `hoist_free`, `fold_free`. The `fold_free` signature ([free.rs:774-798](../../../fp-library/src/types/free.rs#L774-L798)) takes a `NaturalTransformation<F, G>` and requires `G: MonadRec + 'static`, which is exactly the shape `runRec` needs.
- **Coyoneda, two variants** in [types/coyoneda.rs](../../../fp-library/src/types/coyoneda.rs) and [types/coyoneda_explicit.rs](../../../fp-library/src/types/coyoneda_explicit.rs). The trait-object version implements `Functor` via the brand system and hides the intermediate type parameter behind `Box<dyn CoyonedaInner>`. The explicit version exposes the intermediate type and fuses maps at compile time. Both support `hoist`.
- **Lazy evaluation primitives**: `Thunk`, `Trampoline`, `Lazy`, memoized versions. These are candidate targets for the interpreter bridge (analogous to PureScript's `Aff`/`Effect`).

### 2.2 Present but insufficient

- **Free is restricted to `'static`.** Because `TypeErasedValue = Box<dyn Any>` and `Any: 'static`, every type that flows through a `Free` value must be `'static` ([free.rs:46-53](../../../fp-library/src/types/free.rs#L46-L53)). This is documented. For effect programs that close over non-`'static` borrowed state (e.g., a `&mut Vec<T>` held by a handler), this is a hard limit.
- **Free is not itself a `Kind`.** It cannot be used as a brand, because its `Of<'a, A>` would have to be `Free<F, A>` for any `'a`, but `Free`'s `'static` bound means there is no sensible `'a` parameter. This blocks nice type signatures like `Free<F>: Monad` for the Functor/Monad trait hierarchy. In the Run port, `Run<R>` will inherit this restriction unless we use a different base.
- **Coyoneda is closed-sum, not open-sum.** It gives us a free `Functor` instance for any type constructor, but does not provide row-polymorphic labelling or partial interpretation. It is adjacent to `VariantF` (both are functor machinery), but solves a different problem.
- **`NaturalTransformation` is not rank-N in the PureScript sense.** It's a trait with `fn transform<'a, A: 'a>(...)` as a method, and the trait itself is a concrete value at the call site. PureScript's `f ~> g = forall x. f x -> g x` is a first-class value; Rust's version is a dispatched trait-object or a generic parameter. This works for `fold_free` but may bite when building handler combinators that need to store or pass natural transformations around.

### 2.3 Entirely missing

- **Row types / type-level open rows.** Nothing in the crate corresponds to PureScript's `Row Type` or `Row (Type -> Type)`. No HList, no tuple-of-brands, no type-level string map. Grepping for `variant`, `row`, `hlist`, `polysum` returns no hits in `fp-library/src`.
- **Open-sum-of-functors (`VariantF`).** No type that holds one of N possible functors with a runtime tag. This is the central missing piece.
- **Type-level labels.** No use of symbol reflection, no const-generic strings, no per-effect marker-trait convention. `TypeId` is used inside `Free` for erasure but is not exposed as a labelling primitive.
- **Handler combinators.** `inj`, `prj`, `on`, `case_`, `match`, `expand`, `send`, `peel`, `resume` (at the Run level), `interpret`, `run`, `runRec`, `runAccum`, `runAccumRec`, `runAccumPure`, `runPure`, `runCont`, `extract` are all absent. Some of these (e.g., `peel`/`resume` for `Free`) exist at the `Free` level under different names (`resume`), but are not lifted to a `Run`-shaped API.
- **Per-effect functors.** No `Reader<E>`, `State<S>`, `Writer<W>`, `Except<E>`, `Choose`. All need to be written.
- **`Run` newtype.** No wrapper type, no handler-composition pipeline, no derived Monad for it.
- **Base-monad bridge.** No equivalent of `liftEffect`/`runBaseEffect`/`runBaseAff`. The Rust choice for "the base" is itself an open question.

---

## 3. Blockers: Decisions That Must Be Made Before Coding

Each blocker below must be resolved before writing any new types. They are ordered by how much downstream code they influence.

### 3.1 BLOCKER: How to represent the effect row

This is _the_ decision. Every other question flows from it. Rust has no row types. There are four plausible encodings, each with a different cost profile:

1. **Type-level heterogeneous list (HList) of brand-label pairs.**
   Example: `Run<(StateBrand<i32>, (ReaderBrand<Env>, Nil))>`. Needs trait-based membership and removal (`Contains<Brand>`, `Remove<Brand>` traits). Gives open composition, but error messages degrade with depth. Similar to `frunk::HList`.

2. **Closed tuple of brands with a fixed arity.**
   Example: `Run<(StateBrand<i32>, ReaderBrand<Env>)>`. Simple, good error messages, but loses openness: you cannot write `speak :: forall r. String -> Run (TALK + r) Unit` because there is no way to say "a tuple with at least this entry". You'd need a sum-of-supertypes or macro wrappers.

3. **Single wide enum with one variant per effect.**
   Example: `enum AppEffect { State(...), Reader(...) }`. Users must define this enum up front, closing the world. Violates the first principle of `Run` (open composition). Rejected on design grounds but mentioned for completeness.

4. **Trait-object dispatch with `TypeId` tags.**
   Example: `Run` holds `Box<dyn Any>` plus a `TypeId` for tag. Allows full openness and dynamic dispatch. Costs: boxing per effect invocation, runtime type lookup, loss of static exhaustiveness checks. Closest match to the PureScript runtime representation (which is literally `{ type: String, value, map }`).

**Open questions under this blocker:**

- Is it acceptable to require the user to write their effect set as a type-level list (option 1) even for a 3-effect program? PureScript uses row sugar `(state :: State s, reader :: Reader e | r)`; there is no sugar in Rust.
- Can a macro (`effects![State<i32>, Reader<Env>]`) make option 1 or 2 ergonomic enough?
- Is exhaustiveness checking (compile-time "you forgot to handle the `state` effect") a hard requirement? Option 4 can't give that; options 1-3 can.
- Do we need a `Lacks` constraint (prevents duplicate labels in a row)? PureScript's row system has this built in; Rust needs trait-based emulation.

**Recommended first step:** build a minimal prototype of both options 1 and 4 with two effects and see which feels tolerable. Do not commit to the full port until this is chosen.

### 3.2 BLOCKER: Functor dictionary for VariantF

PureScript's `VariantFRep` stores `map :: Mapper f` alongside the value. This lets `VariantF`'s own `map` dispatch to whichever effect functor is currently active. In Rust:

- **Static option:** every effect type in the row must satisfy a `Functor` bound. The row representation must preserve this bound. This is natural for option 1 above (`HList` with a `AllFunctors` witness trait), awkward for option 4.
- **Dynamic option:** store a `Box<dyn Fn(Box<dyn FnOnce(A) -> B>, ...) -> ...>` in the `VariantF` value. Works for any row encoding, but is the most boxed possible implementation.

Related: does the Rust `Functor` trait's `map` signature (`fn map<'a, A, B>(f: impl Fn(A) -> B + 'a, fa: F<A>) -> F<B>`) let us store the function pointer without knowing `A` and `B` ahead of time? Probably not without erasure, which defeats the point of `VariantF`'s generic parameter.

**Open question:** is it acceptable to force all effect functors to implement a supplementary `DynFunctor` trait that hides the type parameters behind `dyn Any` or `Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>`? This is the practical consequence of the dynamic option and needs explicit acknowledgment.

### 3.3 BLOCKER: How strong should stack-safety guarantees be

The existing `Free` is stack-safe (O(1) bind, iterative drop via `Extract`). That is sufficient for `Run`'s own stack-safety. But the PureScript library distinguishes two interpreter families:

- `interpret` / `run` / `runAccum`: assume the target monad is stack-safe.
- `interpretRec` / `runRec` / `runAccumRec`: require `MonadRec` on the target.

In Rust, this distinction is less useful: most target monads we'd write (`Option`, `Result`, `Thunk`) already implement `MonadRec` or trivially can. The open question is:

- Do we ship both families, mirroring PureScript 1:1? Easier to document but doubles the surface area.
- Do we ship only the `MonadRec` family and make every interpreter stack-safe by default? Simpler, costs a few percent in common cases.

**Recommendation (not a decision):** ship only the `MonadRec` family. Revisit if we find target monads that cannot implement it.

### 3.4 BLOCKER: The `'static` bound inherited from `Free`

Every effect functor and every effect value that flows through `Run` will be `'static` if we keep the current `Free`. Consequences:

- Users cannot define effects that hold borrowed references (e.g., `State<&'a mut Vec<T>>`).
- Handlers cannot close over non-`'static` environment data.
- All effect payloads become owned (`String` not `&str`, `Vec<T>` not `&[T]`).

This is a real tax on ergonomics but is consistent with other Free-monad-style effect systems in Rust (e.g., `freer-simple`-like crates, if any existed). Options:

- **Accept the tax.** Document prominently that `Run` is for `'static` effects. This is the simplest path and matches the existing `Free`.
- **Write a non-`'static` Free.** Possible if we replace `Box<dyn Any>` with an existential that carries the lifetime. Very hard in practice; would need custom unsafe type erasure or a completely different encoding.
- **Parameterize `Run` over a lifetime.** `Run<'a, R, A>` where each `F in R` has `F: Functor<'a>`. Trades API complexity for expressive power. Compatible with the explicit Coyoneda encoding already in the crate ([coyoneda_explicit.rs](../../../fp-library/src/types/coyoneda_explicit.rs) supports `'a` lifetimes).

**Open question:** is there enough demand for non-`'static` effects to justify the cost? Probably not for a first release; revisit based on users.

### 3.5 BLOCKER: Natural transformations as values

`interpret` takes a natural transformation `VariantF r ~> m` as a runtime value. In PureScript this is just a polymorphic function. In Rust:

- The existing `NaturalTransformation<F, G>` trait works for `F` with a statically-known type. But `VariantF r` is an _open_ sum; its concrete representation changes with `r`.
- A natural transformation from `VariantF r` must, by construction, handle every case in `r`. In PureScript this is assembled with `case_ # on _reader handleReader # on _state handleState`. The `on` combinator threads the "smaller row" through the type of the remaining fallback.
- In Rust, the equivalent is probably a tuple-of-closures (one per effect) indexed by the same type-level structure as the row, produced by something like `handlers! { state: handleState, reader: handleReader }` (a macro).

**Open question:** will users build natural transformations by hand, or only via a macro? A macro-based DSL is the realistic answer, but it pushes complexity into the macro layer.

---

## 4. Non-Blocking Tasks (Mostly Mechanical)

The following can be written once the blockers are resolved. Each is a straightforward adaptation.

- **`Run` newtype** wrapping `Free<VariantFBrand<R>, A>` once the row encoding is fixed.
- **Per-effect functors.** Small enums with a `Functor` impl:
  ```rust
  enum State<S, A> { State(Box<dyn Fn(S) -> S>, Box<dyn Fn(S) -> A>) }
  ```
  Direct translation from PureScript's `data State s a = State (s -> s) (s -> a)`.
- **Smart constructors.** `ask`, `get`, `put`, `modify`, `tell`, `throw`, `catch`. Each is a thin wrapper over `inj + lift_f`.
- **`extract :: Run () a -> a`.** Trivial once the empty row type is defined.
- **`expand`.** One-line `unsafe fn` using `mem::transmute` once the row constraints prove subsetting.
- **Base-monad bridge.** A `liftEffect`-analog for any target monad we care about. The first target should probably be `Thunk` or `Identity` (pure), with `async fn` as a followup.
- **Error messages.** Rust's error messages on trait-heavy type machinery are legendary. Budget time for macro-generated human-readable errors.

---

## 5. Open Questions That Should Be Answered Before Starting

These are not full blockers (they don't prevent the first line of code) but they should have answers recorded before the port is attempted.

1. **What is the target audience?** A library author building effect-heavy internal tooling has different needs than an application developer. The row-encoding choice in 3.1 may change based on this.
2. **Do we want partial interpretation?** The ability to write `runReader: Run (READER + r) a -> Run r a` is the whole point of `Run`. If we relax this (only full interpretation), we can drop the row polymorphism and the problem becomes much easier. Confirm we want the full version.
3. **How does `Run` interact with `async`?** PureScript's `Aff` maps onto async Rust. Do we want an `async`-aware interpreter from day one, or is a synchronous interpreter acceptable?
4. **What's the story for `IO`/effects-with-side-effects?** The PureScript `Effect` monad has no direct Rust analog. Options: `Thunk`, a custom `Effect` type, `std::io::Result`, or lean on `async` from the start. Needs a named owner.
5. **Macro infrastructure.** Will the port introduce a `define_effect!` macro that generates the functor, smart constructors, and label type? Almost certainly yes. Who owns the macro design?
6. **Testing strategy.** The research doc points at [test/Examples.purs](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106) as the canonical end-to-end example (`TalkF` + `DinnerF`). Port that example as an integration test to validate the design.

---

## 6. Cross-Reference Table: PureScript Piece to Rust Status

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
| `IsSymbol`, `Proxy "label"`                                                                   | Absent.                                                            | Missing; options above.                      |
| `Run r a`                                                                                     | Absent.                                                            | Missing.                                     |
| `lift`, `send`, `peel`, `resume` (Run level)                                                  | Absent (exists at Free level).                                     | Missing.                                     |
| `interpret`, `run`, `runRec`, `runAccum`, `runAccumRec`, `runPure`, `runAccumPure`, `runCont` | Absent.                                                            | Missing.                                     |
| `Run.Reader`, `Run.State`, `Run.Writer`, `Run.Except`, `Run.Choose`                           | Absent.                                                            | Missing (mechanical once `Run` exists).      |
| `liftEffect`, `runBaseEffect`, `liftAff`, `runBaseAff`                                        | Absent.                                                            | Missing; target choice is itself a question. |

---

## 7. Summary

`fp-library` has everything it needs for the "free monad + stack-safe recursion + natural transformation" substrate. The Rust equivalents of `Free`, `MonadRec`, `Step`, and `NaturalTransformation` are already in place and close enough to the PureScript shape that `fold_free` is effectively `runRec` already.

What is missing is the **row-polymorphic open sum** (`VariantF` and its supporting type-level machinery). Nothing in the crate today solves that problem, and Rust does not give us the solution for free.

The three hard blockers in order:

1. **Row encoding** (section 3.1). HList vs tuple vs `TypeId` dispatch. Every other piece of the port is shaped by this.
2. **Functor dictionary dispatch** (section 3.2). Static bound vs dynamic box. Choice follows from 3.1.
3. **`'static` bound on Free** (section 3.4). Accept, replace, or parameterize. Independent of 3.1 but bounds the scope of what users can do.

The other open questions (async story, macro design, exhaustiveness trade-offs) are secondary and can be deferred until a prototype of 3.1 + 3.2 exists.

**Recommended next action:** build a proof-of-concept with exactly two effects (`Reader<Env>` and `State<i32>`), using the simplest plausible row encoding (HList, no labels, no `Lacks`), no macros, and validate whether the resulting API is recognizable as "extensible algebraic effects". If yes, proceed to flesh out; if no, revisit section 3.1 before anything else.
