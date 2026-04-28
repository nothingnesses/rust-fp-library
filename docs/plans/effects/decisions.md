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

- **The current `Free<F, A>` cannot, on its own, serve as Run's substrate.** Three limitations, all traceable to `TypeErasedValue = Box<dyn Any>` plus `Box<dyn FnOnce>` continuations ([free.rs:46-53](../../../fp-library/src/types/free.rs#L46-L53)): it requires `A: 'static` (so effect payloads cannot borrow), it is single-owner (so multi-shot effects like `Choose` cannot consume their continuation twice), and it is not `Send`/`Sync` (so thread-crossing effect programs are impossible). As a consequence of the `'static` bound, `Free` also cannot implement the library's `Kind` trait directly, which would otherwise let it participate in HKT-polymorphic code. Section 4.4 addresses all three by shipping three sibling variants (`RcFree`, `ArcFree`, `FreeExplicit`) alongside the existing `Free`.
- **Coyoneda operates over one functor, not many.** Coyoneda's structure is `exists B. { F<B>, B -> A }` — a single functor `F` plus a deferred map. It has no tag, no label, no injection/projection, and no mechanism for "one of several functors". It solves the "give any single type constructor a cheap `Functor` instance" problem, which is different from the "hold one of an extensible set of functors" problem that Run needs `VariantF` to solve. Coyoneda will still earn a supporting role (providing `Functor` for effect enums that don't have one natively), but it cannot substitute for `VariantF`; the latter must be built from scratch (see section 3.3 and section 4.1).
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

This is _the_ decision. Every other question flows from it. Rust has no row types. Any encoding we pick must preserve **openness** — the ability to write `fn speak<R>(...) -> Run<TALK + R, ()>` where `R` is an unknown extension of the effect set. Closed encodings (fixed tuples, monolithic enums) are rejected up front because they break the core premise of Run.

An ecosystem survey (see the research note below) surfaced five distinct open encodings. Not every one is compatible with the Free-family commitment in section 4.4, but each is worth understanding. The subsection that follows defines key vocabulary used throughout the options list.

#### Background: key concepts for the options below

**First-class program.** In a Free-monad design, a "program using effects A and B" is a concrete value, typically a tree of `Bind` and `Pure` nodes. You can store it in a variable, clone it (if the underlying types allow), walk it multiple times, substitute one effect for another via hoisting, or re-interpret it into a different target monad. In the mtl-style (trait-bound-set) design, a "program" is a generic Rust function: calling it runs it, and that is all. There is no value representing "the program" that you can inspect or replay. The property of being a value rather than a function is what enables multi-shot interpretation, handler composition via `peel`/`send`, and pure-data interpreters like `runPure`. Our section 4.4 commitment to the Free family requires this property.

**Row ordering.** In PureScript, `(a :: A, b :: B)` and `(b :: B, a :: A)` are the same row type; labels form a set, not a list. Rust tuples and nested coproducts are ordered: `Coproduct<A, Coproduct<B, Void>>` is a distinct type from `Coproduct<B, Coproduct<A, Void>>`. When two functions return programs with different orderings of the same effects, Rust's type checker does not see them as compatible without an explicit conversion.

**How the type-level list is actually encoded.** Options 1 and 2 share the same runtime shape (a right-nested enum) and differ only in how they index positions in the list. The enum:

```rust
enum Void {} // empty type, no values
enum Coproduct<Head, Tail> {
    Here(Head),
    There(Tail),
}
```

A row of three effects is `Coproduct<E1, Coproduct<E2, Coproduct<E3, Void>>>`. Membership ("is E2 in this row?") is proved by a trait whose second parameter is a type-level index naming the position:

- **Peano indices (option 1, frunk):** `Here`, `There<Here>`, `There<There<Here>>`, and so on. The nth effect has a type n layers deep. Trait resolution walks the coproduct once per `There`, so compile time and error-message size scale O(n).
- **Binary indices (option 2, typenum):** `UInt<UInt<UTerm, B1>, B0>` for index 2, that is, a binary encoding. The nth effect has a type O(log n) deep. Compile time and error-message size scale O(log n).

Both are real type-level numbers; Rust resolves the right trait impl at compile time. The index is inferred at every call site, so users never type one directly, but they do see them in error messages when a membership claim fails.

**Embedder and Subsetter (frunk terminology).** Two traits that translate between orderings or sizes of a coproduct. `CoproductEmbedder<Larger, Idxs>` proves "my coproduct can be embedded into a larger one by mapping each variant to the correct position in the larger one." `CoproductSubsetter<Subset, Idxs>` proves "I can project out a subset of my variants into a smaller coproduct." They compose, but their invocation is user-visible: every time you widen or narrow an effect row, there is an explicit (even if inference-elided) call. These are the machinery that papers over the row-ordering problem above.

**Type-level list machinery.** Options 1 and 2 encode the effect row as a recursively-nested type constructor such as `Coproduct<A, Coproduct<B, Void>>` or `HList![A, B]`. Working with such a list requires recursive traits (`Member<E, List, Idx>`), type-level indices (Peano `Here`/`There` or typenum naturals), and usually macros to hide the nesting. Rust trait bounds (`T: A + B`) are _not_ a type-level list; they are an unordered, duplicate-free set of constraints the compiler resolves directly. The trait-bound-set approach (option 5) uses this native mechanism instead of building list machinery in user space.

#### The five open encodings

**1. Type-level heterogeneous list / nested coproduct.**

```rust
// An effect set written as a nested coproduct:
type MyEffects = Coproduct<State<i32>, Coproduct<Reader<Env>, Void>>;

// A program polymorphic in additional effects R:
fn my_program<R>(...) -> Run<Coproduct<State<i32>, R>, i32> {
    // polymorphic in the tail R; composes with any extension the caller supplies.
}
```

How openness works: a function generic over the tail `R` of the coproduct can be called with any extension the caller supplies. Adding a new effect adds a new layer to the nesting; no existing program changes.

Known concrete issues:

- **Row ordering.** `Coproduct<State, Coproduct<Reader, Void>>` is a different type from `Coproduct<Reader, Coproduct<State, Void>>`. Composing two functions with different orderings requires an `Embedder` or `Subsetter` (see background). For a 3-effect program there are 6 orderings, so in practice users rely on handler APIs and helper traits that hide the conversion machinery.
- **Error messages.** Trait resolution on deeply nested coproducts produces errors like `expected CoprodInjector<State<i32>, There<There<There<Here>>>> for Coproduct<Reader, Coproduct<Writer, Coproduct<Except, Coproduct<State, Void>>>>`. Every crate using this approach (`effing-mad`, `corophage`) publicly acknowledges this as a known pain point.
- **Compile time.** Each callsite monomorphises the coproduct to its specific index, producing a distinct instance per use.

Real-world references: `frunk::Coproduct`, `effing-mad`, `corophage`; analogous to Haskell's `freer-simple` and `polysemy`.

**Nightly-toolchain caveat.** `effing-mad`'s design relies on Rust's unstable `Coroutine` trait (requires `feature(coroutines, coroutine_trait)`). The trait has no stabilisation timeline. Any port that takes `effing-mad` as its implementation model inherits this blocker. `corophage` sidesteps the issue by building on `fauxgen`-backed async coroutines that compile on stable Rust; see [research/effing-mad.md](research/effing-mad.md) and [research/corophage.md](research/corophage.md).

**2. Typenum-indexed sum list.**

Same user-facing encoding as option 1, but membership indices are binary type-level naturals (`typenum::UInt`) instead of Peano `Here` / `There<T>` wrappers.

```rust
// The 5th effect's index becomes roughly UInt<UInt<UTerm, B1>, B0>  (O(log n) depth)
// instead of There<There<There<There<Here>>>>  (O(n) depth).
```

Same openness story, same row-ordering issue. Theoretical improvements:

- Binary naturals scale better; index-type depth grows logarithmically with effect count.
- Error messages improve proportionally because the printed index types are shallower.
- Compile time improves because trait resolution recurses fewer times.

A refinement of option 1 at the implementation level; it does not change the user-facing surface.

**No known Rust reference implementation.** Earlier drafts of this section cited [`reffect`](https://github.com/js2xxx/reffect) as the Option 2 reference. Stage 1 research (see [research/reffect.md](research/reffect.md)) confirmed that reffect's tag type is a single-parameter `UInt<U>` phantom wrapper defined in `src/util/tag.rs`, not a two-parameter `typenum`-style binary natural `UInt<U, B0|B1>`. Its index-type depth is therefore O(n), structurally equivalent to frunk's `There<T>`; the logarithmic advantages above are unrealised in any surveyed Rust crate. Option 2 is kept as a design point but has no production validation.

**3. Trait-object dispatch with `TypeId` tags.**

Effects are erased to `Box<dyn Any>` plus a `TypeId`. The effect set is not tracked in types at all.

```rust
// Handler peels by downcasting at runtime:
if effect.type_id() == TypeId::of::<State<i32>>() {
    let state_op = effect.downcast::<State<i32>>().unwrap();
    // handle it
} else if effect.type_id() == TypeId::of::<Reader<Env>>() {
    // ...
}
```

How openness works: fully dynamic. Adding a new effect just means inserting a new downcast branch; the program's type signature does not change.

Known concrete issues:

- **No static exhaustiveness.** The compiler cannot tell you "you forgot to handle the State effect"; a missing branch falls through at runtime.
- **Boxing cost.** Every effect invocation involves a `Box<dyn Any>` allocation and a `TypeId` lookup. PureScript pays this cost (its runtime representation is literally `{ type: String, value, map }`) and gets away with it because JavaScript already boxes everything; Rust does not, so this is a real per-invocation tax.
- **Loses the first-class-program-indexed-by-row property** (see background). The program's type no longer records which effects it uses; that information lives only in which handlers the user chose to write. Signatures like `fn prog() -> Run<A>` lose their documentation value and cannot be used for static analysis.

Real-world references: `anymap`, `typemap`, `typedmap`, `axum::Extension`, and PureScript's own runtime representation of `Run`.

**4. Hybrid: coproduct + macro sugar.**

Use option 1 or 2 internally; expose user-facing macros that accept a flat effect list:

```rust
// User writes a flat list:
type MyEffects = coprod![State<i32>, Reader<Env>];
// Macro expands to: Coproduct<State<i32>, Coproduct<Reader<Env>, Void>>

// Usage looks like row-sugar in type positions:
fn my_program<R: Embed<coprod![State<i32>]>>(...) -> Run<R, i32> { ... }
```

Same fundamental encoding as options 1/2; macros hide the nesting. `corophage`'s `Effects![E1, ...Alias]` (with spread syntax) is the reference implementation and the approach closest to PureScript's row sugar.

Inherits options 1/2's row-ordering issue. Note that `corophage`'s macro does _not_ currently sort or deduplicate orderings (see [research/corophage.md](research/corophage.md)); it preserves user-written order and expands to a right-nested `Coproduct<A, Coproduct<B, CNil>>` verbatim. Row-ordering mismatches are instead mediated at composition sites by `CoproductSubsetter`; see "Ordering mitigations" below. Macro-based lexical normalisation (workaround 1) is compatible with this design but is not what `corophage` itself implements today. Inherits compile-time cost. Error messages can be worse than raw coproduct because the user sees the macro expansion in the error, not their original code. Currently the default direction for the draft architecture in section 5.

**Per-effect lifetime parameter (corophage pattern).** `corophage` attaches a lifetime parameter `'a` to every effect type (`trait Effect<'a>` per `corophage/src/effect.rs:83`), allowing effect payloads to borrow non-`'static` data. This is a direct, production-validated match for the port's `FreeExplicit` variant (see section 4.4), which must support non-`'static` effect payloads. The port should adopt the same `'a` pattern on effect traits; doing so is what makes `FreeExplicit` useful in practice.

**Compile-time index-table refinement (Koka-inspired).** A sufficiently rich proc-macro can do more than expand the coproduct: it can also emit a const `[usize; N]` table assigning each effect in the row a stable integer index by lexical sort order. Handler dispatch then reads the handler from an array slot by index rather than pattern-matching the coproduct, giving O(1) dispatch independent of the row size and no runtime `TypeId` inspection. This is a portable adaptation of Koka's `OpenResolve` pass (see [research/koka.md](research/koka.md) and [research/deep-dive-evidence-passing.md](research/deep-dive-evidence-passing.md)). `corophage`'s current `Effects![...]` macro does _not_ emit such a table; this is flagged as an achievable Phase 2 optimisation, not a Phase 1 requirement.

**Type-level row-narrowing handler API (coroutine-crate pattern).** The three Rust coroutine crates surveyed in Stage 1 (corophage, reffect, effing-mad) converge on a handler-attachment API shape worth adopting on top of whichever Free variant the port ships. The `Program::handle` method (or equivalent) consumes the effectful value and returns a new value typed against `<Remaining as CoproductSubsetter<HandledEffects, SubsetIdx>>::Remainder`, with an HList of accumulated handlers extended on the right. The effect row narrows in the type system with each attachment, letting callers attach handlers in any order and receive a type-checked intermediate value at each step; see `corophage/src/program.rs:133-156` for the canonical shape. This is a _handler-API_ decision independent of the row-encoding choice and is compatible with any of options 1, 2, 3, or 4. The type-level row narrowing is a partial analogue of PureScript's `peel`/`send`; the data-level counterpart (continuation exposed as a first-class value) requires the Free AST that section 4.4 commits to. See [research/deep-dive-coroutine-vs-free.md](research/deep-dive-coroutine-vs-free.md) section 3.2.

**Content-addressed canonical naming (tstr_crates refinement).** A second Phase 2 refinement on top of the macro layer: each effect carries a `tstr::TS!("name")` as its canonical identifier (either as an associated type on a `NamedEffect` trait or as a marker), and the macro sorts by that name rather than by the type's stringified path. This fixes a real failure mode of stringify-based sorting: when an effect is written `crate::a::Reader` in one place and `Reader` (after import) elsewhere, the two stringified forms differ and would sort to different positions. With `TS!("name")`, the canonical identifier is content-addressed and stable across import paths. See [poc-effect-row/tests/feasibility.rs](../../../poc-effect-row/tests/feasibility.rs) tests t14-t16 for the data shape (a `NamedEffect` trait with `type Name: IsTStr + Copy; const NAME: Self::Name;`) and a working compile-time comparison via `tstr::cmp`. What this refinement does NOT enable on stable Rust: type-level auto-canonicalisation of hand-written coproducts, because the `Ordering` returned by `tstr::cmp` is not a stable const generic kind; the const result cannot drive a recursive type-level sort. Lifting it requires nightly's `feature(adt_const_params)` plus `feature(generic_const_exprs)`; on stable, the macro remains the canonicaliser and `CoproductSubsetter` remains the fallback. tstr_crates is a Phase 2 refinement of Option 4, available on stable Rust 1.88+; see [../type-level-sorting/research/tstr-crates.md](../type-level-sorting/research/tstr-crates.md) and the type-level-sorting [research synthesis](../type-level-sorting/research/_classification.md) section 4 for the wider context.

**5. Trait-bound set (one-trait-per-effect, mtl-style).**

Each effect is a trait; a program is a generic function constrained by the trait bounds that name its effects.

```rust
trait State<S> { fn get(&mut self) -> S; fn put(&mut self, s: S); }
trait Reader<E> { fn ask(&self) -> E; }

// The "effect set" of this program is its trait-bound set.
fn my_program<M: State<i32> + Reader<Env>>(m: &mut M) -> i32 {
    let env = m.ask();
    let n = m.get();
    n + env.config_value()
}
```

How openness works: a function's effect set _is_ its trait-bound set. `fn foo<M: A + B>` composes with `fn bar<M: B + C>` into `fn baz<M: A + B + C>`. No type-level list machinery (see background) because Rust trait bounds are _already_ an unordered, duplicate-free, compositional set; the compiler knows how to merge and resolve them. You do not build this in user space.

This is the cleanest native Rust encoding of row polymorphism.

**Fundamental blocker for this port:** programs are not first-class values (see background). Concretely:

- `my_program` above is a function, not a value. You cannot clone it, print it, or inspect which operations it performs without running it.
- Handler methods run "in tail position": when `my_program` calls `m.get()`, the `M::get` implementation returns a value, computation continues, and `M::get` never sees what the program does afterwards. There is no "rest of the program" handle for the handler to manipulate.
- Multi-shot interpretation is impossible. The `Choose` effect wants to handle `Alt(k)` by running `k(true)` _and_ `k(false)`, but in this encoding there is no `k`; there is only a method call that returns one boolean. You would have to restructure the entire program in continuation-passing style to recover a continuation handle, which amounts to building a Free monad in disguise.
- Pure-data interpreters like `runPure` are impossible. `runPure` inspects the AST to decide what to emit for each effect; there is no AST here.

In PureScript terms, this is the distinction between `MonadState` (a constraint, runs methods) and `Run (STATE s + r)` (a value, builds an AST). The library deliberately ships Run because some effects need AST access. Section 4.4 commits to Run-style via the Free family, which forecloses on this encoding as the substrate.

Kept in the plan not as a candidate but as a benchmark: this is what good row-ergonomics looks like in Rust. Options 1 to 4 should aim to match option 5's error-message quality and compositional feel, even though they will pay the type-level-list-machinery tax along the way. Real-world references: [shtsoft effect-trait pattern](https://blog.shtsoft.eu/2022/12/22/effect-trait-dp.html), `karpal-effect`, `higher`.

#### Ordering mitigations

Options 1, 2, and 4 share a problem: `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>` are distinct types despite denoting the same effect set. Three workarounds exist, each with distinct costs.

**Sorting types at the type level is not directly available in stable Rust.** The trait system can unify types but cannot compare them by any total order; `TypeId::of::<T>()` is a runtime value and cannot drive trait resolution. Haskell's `row-types` library does this via the built-in `Symbol` kind (type-level strings with a canonical ordering); Rust has no analogous kind. Any workaround therefore simulates ordering from outside the type system.

The three workarounds:

1. **Macro-based canonicalisation by textual tag.** A proc macro (such as the `coprod![...]` or `effects![...]` in option 4) sorts effect names lexically at expansion time and emits a `Coproduct` in canonical order.

   ```rust
   type E1 = effects![State<i32>, Reader<Env>];
   type E2 = effects![Reader<Env>, State<i32>];
   // Both expand to the same Coproduct, e.g.:
   //   Coproduct<Reader<Env>, Coproduct<State<i32>, Void>>
   ```

   Pros: simple, no language extensions, no per-effect boilerplate. Folds into option 4's macro layer for free, so the row-ordering issue disappears from user code as long as users go through the macro. Cons: hand-written `Coproduct<...>` types bypass the sort. Textual names include generic parameters, so different parameterisations sort into different positions (usually correct, occasionally surprising). Fully-generic effect types may not have a canonical name at macro-expansion time.

2. **Tag-based type-level sorting.** Each effect implements a trait providing a type-level numeric tag (`trait EffectTag { type Tag: typenum::Unsigned; }`), and a recursive trait performs insertion sort at type-resolution time.

   Pros: works without a macro; users can define a `Coproduct` by hand and the type system still sorts it. Cons: every effect definition requires a tag impl; tag collisions produce confusing errors and demand a coordination mechanism (a global registry or hash-of-type-name); compile time worsens because type-level sorting is quadratic trait resolution in the worst case. No Rust effect crate currently ships this approach.

3. **Permutation proofs (`CoproductSubsetter`).** Don't sort. Instead, generalise every API so that a function accepting an effect row also accepts any permutation of it; the trait machinery (`CoproductSubsetter<Target, Indices>` from frunk) proves the permutation at the call site.

   Pros: works with any coproduct shape; no macro discipline required. This is what `effing-mad` and `corophage` already do. Cons: user-visible machinery; error messages carry full permutation indices; compile time scales with permutation size (factorial worst case, though inference usually resolves quickly); users pay the cost at every composition site.

**Recommendation.** Adopt workaround 1 as a Phase 1 implementation detail of option 4. The macro layer already exists in the hybrid design; making it canonicalise is additive and cheap. Workaround 3 stays as a fallback for the rare cases where a user bypasses the macro and composes raw Coproducts; existing frunk machinery handles it. Workaround 2 is rejected on complexity and compile-time grounds unless a compelling case for hand-authored coproducts emerges.

**POC validation.** The hybrid is validated empirically in [poc-effect-row/](../../../poc-effect-row/), with the findings written up in [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md). Seventeen tests on stable Rust 1.94 cover the cases this section flagged as risky: generic effect parameters (`Reader<Env>` vs `State<S>`), same-root-different-params (`Reader<i32>` vs `Reader<i64>`), lifetime parameters, 5- and 7-effect rows for trait-inference scaling, and hand-written non-canonical coproducts mediated through `.subset()` (with an explicit `<_ as CoproductSubsetter<_, _>>::subset(...)` test confirming the trait is the load-bearing piece even though `.subset()` is an inherent method). The macro implementation is ~30 lines; the proc-macro sorts by `quote!{}.to_string()` of each parsed `syn::Type`, which is sufficient for the cases tested. The remaining open question (rare): how the macro should treat fully-generic effect type parameters (e.g., a row over `R` where `R` is itself a generic type parameter); the POC did not probe that case because it requires a richer call-site setup to be meaningful.

**Note on trait-bound sets (option 5 above).** The trait-bound-set approach is trivially order-invariant: `T: A + B` and `T: B + A` are the same constraint. This is the only unordered container at the type level in stable Rust, but it lives in the _constraint_ world, not the _data_ world, so it cannot serve as a data container for the Free family. It is not a mitigation for coproduct ordering; it is an entirely different encoding that sidesteps the problem at the cost of giving up first-class programs.

#### Evaluated and declined

**Evidence passing (EvEff / Koka dispatch).** Stage 2 research evaluated the handler-vector dispatch mechanism used by EvEff (Xie and Leijen, ICFP 2020) and Koka as a candidate sixth row encoding. Finding: the mechanism is not portable to Rust as a distinct encoding. In Haskell, dispatch relies on the closed type family `HEqual` (`src/Control/Ev/Eff.hs:263-265`) to drive instance resolution by type identity. Rust has no closed-type-family analogue outside of unstable `min_specialization`. Any Rust simulation reduces to Option 1 (Peano index) or Option 3 (`TypeId` runtime comparison) at the implementation level. See [research/eveff.md](research/eveff.md), [research/koka.md](research/koka.md), and [research/deep-dive-evidence-passing.md](research/deep-dive-evidence-passing.md). The one genuinely portable idea from this family, Koka's compile-time effect-index vector, has been folded into Option 4 above as a refinement of the macro layer.

#### Honourable mentions (not direct candidates)

- **Coroutine-frame row (Abubalay).** The entire row disappears at runtime because the stackless coroutine's state machine is already a sum of "currently suspended at effect X" variants. An optimisation target for any option 1-based implementation rather than a separate encoding.
- **Const-generic effect axes (Wuyts' `#[maybe(async)]` proposal).** Each effect becomes an independent const-bool parameter (`T: Trait<ASYNC = E>`), which is row-polymorphism in a different shape. Useful for a finite, known-at-language-level effect set (like async/sync); cannot express "arbitrary tail `r`".
- **Extractor-tuple pattern (axum `FromRequest`, bevy `QueryFilter`).** Provides extensibility by macro-implementing a trait for tuples of length 0..16. Works well for closed-world dispatch but does not give true row polymorphism. Good precedent for ergonomic "user adds an effect type" workflows.

**Openness-preserving options remaining as candidates:** 1, 2, 3, 4. Option 5 is ruled out by section 4.4's Free-family commitment but is the benchmark for what good row-ergonomics looks like.

**Open questions under this blocker:**

- Is it acceptable to require the user to write their effect set as a type-level list even for a 3-effect program? PureScript uses row sugar `(state :: State s, reader :: Reader e | r)`; there is no sugar in Rust.
- Can a macro (`effects![State<i32>, Reader<Env>]` / `coprod![...]`) make the coproduct approach ergonomic enough? `corophage`'s `Effects![...]` syntax is the benchmark to match or beat.
- Is exhaustiveness checking (compile-time "you forgot to handle the `state` effect") a hard requirement? Option 3 can't give that; options 1, 2, and 4 can.
- Do we need a `Lacks` constraint (prevents duplicate labels in a row)? PureScript's row system has this built in; Rust needs trait-based emulation, cost similar to `Member`.
- Should the implementation use frunk's Peano-indexed Coproduct or typenum-indexed SumList? Both are open; the choice affects error-message quality and compile time but not the user-facing API.
- How should duplicate entries in a row be distinguished? PureScript disallows them via row's `Lacks` constraint. Koka handles the duplicate case via mask levels (`src/Core/OpenResolve.hs:208-218`), computing a level for each label based on how many identical labels precede it, so a nested `handlerLocal`-style handler for the same effect is addressable distinctly from its parent. If the port supports scoped handlers that re-introduce an already-handled effect, it needs an equivalent; see [research/deep-dive-evidence-passing.md](research/deep-dive-evidence-passing.md) section 5.3.

**Leaning:** the hybrid (option 4) remains the default, with Peano indexing (option 1 substrate, as `corophage` implements) as the starting point. True typenum-binary indexing (option 2) is an acknowledged future optimisation but has no Rust reference implementation to copy from; adopting it would require fresh design work. Trait-objects (option 3) are the fallback if the static-dispatch routes become unmanageable. Build a minimal prototype before committing. Track `corophage` as the primary concrete reference implementation; `effing-mad` is a secondary reference but requires nightly Rust.

### 4.2 BLOCKER: Functor dictionary for VariantF

PureScript's `VariantFRep` stores `map :: Mapper f` alongside the value. This lets `VariantF`'s own `map` dispatch to whichever effect functor is currently active. In Rust:

- **Static option:** every effect type in the row must satisfy a `Functor` bound. The row representation must preserve this bound. Natural for the HList/coproduct encoding (option 1/5 above); awkward for option 4.
- **Dynamic option:** store a `Box<dyn Fn(Box<dyn FnOnce(A) -> B>, ...) -> ...>` in the `VariantF` value. Works for any row encoding, but is the most boxed possible implementation.
- **Freer option:** drop the functor requirement altogether. See section 5.2.

Does the Rust `Functor` trait's `map` signature (`fn map<'a, A, B>(f: impl Fn(A) -> B + 'a, fa: F<A>) -> F<B>`) let us store the function pointer without knowing `A` and `B` ahead of time? Probably not without erasure, which defeats the point of `VariantF`'s generic parameter.

**Open question:** is it acceptable to force all effect functors to implement a supplementary `DynFunctor` trait that hides the type parameters behind `dyn Any`? This is the practical consequence of the dynamic option and needs explicit acknowledgment.

**Resolution lean: static option via Coyoneda.** Section 5.2's commitment to Free + Coyoneda already supplies what each row variant needs. An effect that is naturally a `Functor` satisfies the bound directly; an effect that is not is wrapped in `Coyoneda<E>` at lift time, and `Coyoneda` is trivially a `Functor` for any `E`. `VariantF<...>`'s `map` then dispatches via type-level pattern-matching on the coproduct rather than via a runtime dictionary. The four-variant Coyoneda family pairs with the six-variant Free family (see section 5.2): a user picking `RcFree` (or `RcFreeExplicit`) automatically pairs with `RcCoyoneda` for the wrap and inherits the matching sharing / `Send` / `Sync` properties; the same pairing logic applies across the rest of the matrix. The dynamic option (`DynFunctor` + `Box<dyn Any>`) is retained as a fallback only if a future use case surfaces an effect type that genuinely cannot be Coyoneda-wrapped; Stage 1 effects research surveyed every Haskell Free library and found none in that situation. The Freer option is foreclosed by section 5.2.

**POC validation.** The static option is validated end-to-end in [poc-effect-row/](../../../poc-effect-row/) tests c01-c08, written up in [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md) section 4.6. Eight tests on stable Rust 1.94 confirm: (a) the macro integrates with Coyoneda wrapping, with two orderings producing the same canonical type; (b) `Coyoneda<F, A>` implements `Functor` for any `F`, including effects with no Functor impl of their own; (c) `Coproduct<H, T>` implements `Functor` via recursive trait dispatch (`Coproduct<H, T>: Functor where H: Functor + T: Functor`, with `CNil` as the base case), so the active variant's `fmap` is selected by trait resolution alone with no runtime dictionary, no specialization, no nightly features. One implementation note worth recording: `Coyoneda<F, A>` has two type parameters, so the wrapping macro must thread the answer type explicitly (`effects_coyo![A; F1, F2]` in the POC); in production this would be hidden inside `Run<Effs, A>`'s definition so users would not see it.

**Scope narrowing from the dual-row decision in section 4.5.** The Functor-dictionary problem applies only to the _first-order_ effect row (the `VariantF` of algebraic effects). The higher-order row (scoped effects such as `Catch<E>` or `Local<E>`; see section 4.5) is not a functor: it is a closed set of constructors, each holding its own action and handler payload, and is interpreted by manual case dispatch rather than by `map`. This halves the surface area of the problem. Whichever option is adopted here (static bound, dynamic `DynFunctor`, or freer-style erasure) applies only to first-order effects; scoped effects do not require a dictionary at all. See [research/deep-dive-scoped-effects.md](research/deep-dive-scoped-effects.md).

### 4.3 DECISION: Ship both interpreter families (PureScript-mirroring)

The existing `Free` is stack-safe (O(1) bind, iterative drop via `Extract`). That is sufficient for `Run`'s own stack-safety. The PureScript library distinguishes two interpreter families:

- `interpret` / `run` / `runAccum`: assume the target monad is stack-safe.
- `interpretRec` / `runRec` / `runAccumRec`: require `MonadRec` on the target.

In Rust, this distinction is less useful at the design level: most target monads we'd write (`Option`, `Result`, `Thunk`) already implement `MonadRec` or trivially can. But the documentation and pedagogical advantages of mirroring PureScript 1:1 are real, and the implementation cost of shipping the second family is mostly mechanical (the iterative-via-`MonadRec` path is the harder one and is already required for the six-variant Free family in section 4.4).

**Decision: ship both families, mirroring PureScript.** Two interpreter families:

- `interpret` / `run` / `runAccum`: assume the target monad is stack-safe (recursive interpretation).
- `interpretRec` / `runRec` / `runAccumRec`: require `MonadRec` on the target (iterative via trampolining).

This doubles the public-API surface but matches the upstream PureScript naming, which makes the library easier to teach to PureScript users and easier to cross-reference against `purescript-run` source. The few-percent runtime cost concern that motivated "MonadRec only" is real but small; users who care can reach for the recursive family explicitly.

### 4.4 DECISION: Ship a six-variant `Free` family with Erased/Explicit dispatch split

The existing `Free<F, A>` imposes three limitations, each driven by an independent implementation choice:

1. **`'static` only**, because `Box<dyn Any>` requires `'static`. Blocks effects that hold borrowed references (`State<&'a mut Vec<T>>`), handlers that close over non-`'static` environment data, and non-owned payloads (`&str`, `&[T]`).
2. **Single-owner, non-cloneable**, because `Box<dyn FnOnce>` consumes its callable once. Blocks multi-shot continuations — specifically the `Choose` effect from `purescript-run`, whose handler calls the continuation both for `true` and for `false`. The current `Free` cannot serve as Run's AST for multi-shot effects.
3. **Not thread-safe**, because `Box<dyn FnOnce>` has no `Send`/`Sync` bounds. Blocks effectful programs that need to cross thread boundaries.

These are the same forces that produced the [four-variant Coyoneda family](../../../fp-library/docs/coyoneda.md) (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`, `CoyonedaExplicit`). The axes are identical here: sharing model (Box / Rc / Arc) and existentiality (erased continuation types via type-erased `dyn Any` cells + CatList, or concrete recursive enum). Unlike Coyoneda, the Free port ships **all six** cells of the 2x3 matrix: the two intersection variants (`RcFreeExplicit`, `ArcFreeExplicit`) are required to carry Brand dispatch over multi-shot and thread-crossing effect programs, which the Erased family cannot satisfy (the `dyn Any` erasure forces `A: 'static`, incompatible with the `Kind` trait's `Of<'a, A: 'a>: 'a` signature).

**Decision: ship all six Free variants. The Erased family (`Free`, `RcFree`, `ArcFree`) is inherent-method only; the Explicit family (`FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) carries the full Brand dispatch hierarchy.**

| Variant           | Sharing | Erasure                                | `'static`? | Cloneable? | Thread-safe? | Bind | Brand dispatch                 | Purpose                                                                    |
| ----------------- | ------- | -------------------------------------- | ---------- | ---------- | ------------ | ---- | ------------------------------ | -------------------------------------------------------------------------- |
| `Free` (today)    | `Box`   | `Box<dyn Any>` + CatList               | Yes        | No         | No           | O(1) | None (inherent-only)           | Default; fast single-shot effect programs.                                 |
| `RcFree`          | `Rc`    | `Rc<dyn Any>` + CatList                | Yes        | Yes, O(1)  | No           | O(1) | None (inherent-only)           | Multi-shot continuations fast path (`Choose`, nondeterminism).             |
| `ArcFree`         | `Arc`   | `Arc<dyn Any + Send + Sync>` + CatList | Yes        | Yes, O(1)  | Yes          | O(1) | None (inherent-only)           | Thread-crossing effect programs fast path.                                 |
| `FreeExplicit`    | `Box`   | concrete recursive enum                | No         | No         | No           | O(N) | Yes (`Functor`/`Monad`/...)    | Brand-dispatched single-shot Run programs; effects with borrowed payloads. |
| `RcFreeExplicit`  | `Rc`    | concrete recursive enum                | No         | Yes, O(1)  | No           | O(N) | Yes (`Functor`/`Monad`/...)    | Brand-dispatched multi-shot Run programs.                                  |
| `ArcFreeExplicit` | `Arc`   | concrete recursive enum                | No         | Yes, O(1)  | Yes          | O(N) | Yes (via `SendFunctor` family) | Brand-dispatched thread-crossing Run programs.                             |

The POC at [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs) validated the existential-free shape (`FreeExplicit`); steps 1-3 of Phase 1 promoted it and added the Erased Rc/Arc siblings. `RcFreeExplicit` and `ArcFreeExplicit` are mechanical extensions of `FreeExplicit` with the outer wrapper swapped to `Rc<...>` / `Arc<... + Send + Sync>` respectively, mirroring the closure-storage pattern from [ArcCoyoneda](../../../fp-library/src/types/arc_coyoneda.rs) — including the associated-type-bound trick (`Kind<Of<'a, A>: Send + Sync>`) that lets the compiler auto-derive `Send`/`Sync` without unsafe.

**Why the Erased/Explicit dispatch split.** Each Free variant in the Erased family stores continuations and intermediate values in `dyn Any` cells (`Box<dyn Any>`, `Rc<dyn Any>`, `Arc<dyn Any + Send + Sync>`). `dyn Any`'s downcast is sound only when the contained type is `'static`, so the entire family pins the result type `A` to `'static`. The library's `Kind` trait has signature `type Of<'a, A: 'a>: 'a;` and `Functor::map` takes `fn map<'a, A: 'a, B: 'a>(...)`; neither admits tightening to `A: 'static` at the impl site, so the Erased family cannot participate in Brand dispatch under the existing trait hierarchy. The Explicit family keeps the functor structure as a concrete recursive enum (no `dyn Any`, so no `'static` requirement) and fits the existing trait hierarchy directly. The split lets the user choose: O(1) bind via inherent methods on the Erased variant when typeclass-generic dispatch isn't needed, or O(N) bind with full Brand dispatch on the Explicit variant when it is. Both halves of the matrix are first-class commitments, not deprecation candidates; conversion between them (`Run -> RunExplicit`) walks the structure once and rebuilds in the other shape.

**Why ship all six at once rather than incrementally.** The API of `Run<R, S, A>` is shaped by which Free variant underlies it. If Run starts on a subset and later needs another (e.g., adds multi-shot via `Choose`, or adds Brand dispatch over a previously inherent-method-only variant), the change is breaking: user-written handlers move from one continuation shape to another, effect functors that stored particular payload shapes have to change, any previously-compiled effect program stops type-checking. The cost of the Coyoneda-style "pick the variant that fits" API has already been paid once in the library; paying it again for Free keeps the design coherent and avoids a near-certain v2 migration. Shipping the Explicit intersections (`RcFreeExplicit`, `ArcFreeExplicit`) up front is what makes Brand-dispatched multi-shot and thread-crossing Run programs available without breaking the simpler users in the Erased fast path.

#### POC status (FreeExplicit only)

The POC covered the one variant that required novel work (the non-erased recursive enum). Files:

- [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs) — 6 passing tests, 1 intentionally `#[ignore]`d.
- [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs) — Criterion bench at 4 depths.
- [fp-library/tests/ui/free_requires_static.rs](../../../fp-library/tests/ui/free_requires_static.rs) — compile-fail documenting that `Free` rejects non-`'static` payloads, motivating `FreeExplicit`.

Findings per question:

| #   | Question                                  | Finding                                                                                                                                                                                                                                |
| --- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | Compiles as `Kind` with the brand system? | Yes. `FreeExplicitBrand<F>` registers via `impl_kind! { impl<F: Kind_... + 'static> for FreeExplicitBrand<F> { ... } }` and satisfies the Kind trait for `IdentityBrand` and `OptionBrand`.                                            |
| Q2  | Supports non-`'static` payload?           | Yes. Both `FreeExplicit<'_, IdentityBrand, &str>` with a borrowed payload and a borrowed value inside a `Wrap` layer compile and round-trip.                                                                                           |
| Q3  | Bind overhead at increasing depths?       | Approximately linear in the spine walk. Single bind + evaluate over a pre-built `Wrap` spine: 273 ns / 3.95 μs / 28.5 μs / 277 μs at depths 10 / 100 / 1 000 / 10 000. Per-node cost ~27 ns. Acceptable for realistic effect programs. |
| Q4a | Iterative evaluate on deep chains?        | Yes. 100 000-deep chain evaluates without stack overflow via a `while let` loop in `evaluate_identity`.                                                                                                                                |
| Q4b | Naive `Drop` on deep chains?              | **Overflows.** Must ship a custom iterative `Drop` (see below). Test is `#[ignore]`d to document the behaviour without crashing normal runs.                                                                                           |
| Q5  | Two-effect Run-shaped example?            | Yes. `OptionBrand` short-circuits cleanly via `Wrap(None)`; `IdentityBrand` chained binds compose.                                                                                                                                     |

#### What to do about `Drop`

All six variants need stack-safe `Drop`, with two strategies split along the Erased/Explicit axis:

- **`Free`, `RcFree`, `ArcFree`** share the current `Free`'s `Drop` strategy: iteratively dismantle `Suspend` layers via the `Extract` trait ([free.rs:218-225](../../../fp-library/src/types/free.rs#L218-L225)). `RcFree` and `ArcFree` only run the dismantling when the last reference is dropped (which `Rc::drop`/`Arc::drop` already gives them, with `Rc::try_unwrap` / `Arc::try_unwrap` used inside the loop to take owned access to nested layers); the inner CatList dismantling is identical.
- **`FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`** each ship a custom iterative `Drop` that the original POC left out of scope. Pattern:
  1. Add an `Extract`-style trait bound on `F` at the struct definition (`F: Extract + Functor + 'a`) so `Drop` can call it. Rust requires `Drop` impl bounds to match struct bounds, so this propagates.
  2. Implement `Drop` as a loop: repeatedly take the current `Wrap(f_inner)`, call `F::extract(f_inner)` to pull out the next boxed/refcounted `FreeExplicit`-shaped value, then non-recursively drop the extracted layer. When a `Pure` is reached, let default drop handle it.
  3. The Rc/Arc variants additionally use `Rc::try_unwrap` / `Arc::try_unwrap` inside the dismantling loop: when the last reference is held the inner is moved out and walked iteratively, otherwise the variant defers cleanup to whichever holder eventually drops the last reference.
  4. Caveat: this forces every effect functor used with the Explicit family to implement `Extract`. For functors that cannot (e.g., effects whose payload is a continuation function rather than a concrete value), users must go through `fold_free` into a `MonadRec` target instead. Same story `Free` already tells; the bound simply propagates.

**Cleanup tasks at promotion time** are enumerated in section 7.

#### Why all six cells: the role each plays

The sharing and existentiality axes form a 2x3 matrix. All six cells ship under this decision, with each row carrying a distinct dispatch story:

|          | Box            | Rc               | Arc               |
| -------- | -------------- | ---------------- | ----------------- |
| Erased   | `Free`         | `RcFree`         | `ArcFree`         |
| Explicit | `FreeExplicit` | `RcFreeExplicit` | `ArcFreeExplicit` |

**Top row (Erased family): inherent-method-only fast path.** All three Erased variants pin `A: 'static` because of `dyn Any` erasure. They expose the same surface (`pure`, `wrap`, `bind`, `map`, `lift_f`, `to_view`, `resume`, `evaluate`, `hoist_free`) via inherent methods only — no Brand, no `Functor`/`Monad` impl. They give O(1) bind via the "Reflection without Remorse" CatList trick, which is the win that justifies them; users who don't need typeclass-generic dispatch (the common case for Run programs that are a single coherent pipeline, not a generic library combinator) accept the `'static` payload restriction in exchange for this speed.

**Bottom row (Explicit family): Brand-dispatched general path.** All three Explicit variants use a concrete recursive enum (`Pure | Wrap`) instead of `dyn Any` erasure, so payloads are `A: 'a` (no `'static` restriction). They participate in Brand dispatch via `FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>` and implement `Functor` / `Pointed` / `Semimonad` / `Monad` (and the by-reference siblings). The cost is O(N) bind because each `bind` walks the spine via `F::map`. The `ArcFreeExplicit` variant additionally requires the new `SendFunctor` family of traits (by-value + `Send + Sync`-bounded) since the existing `Functor` trait's closure parameters lack `Send + Sync` bounds; this gap is the same one that prevents `ArcCoyonedaBrand` from implementing `Functor`, and `SendFunctor` resolves both at once.

**Conversion between Erased and Explicit.** Each Erased variant gets an `into_explicit()` method that walks the structure once and rebuilds in Explicit form. Cost is O(N) per conversion in the structure depth. The expected pattern: build with the Erased variant during the construction phase (cheap O(1) binds accumulating), call `into_explicit()` at the boundary where typeclass-generic code begins, hand the resulting Explicit value to the consumer. Reverse conversion (`from_explicit`) walks the recursion and rebuilds via Erased binds; same cost, opposite direction. Both directions preserve the multi-shot/thread-safe properties of the underlying substrate (`RcFree -> RcFreeExplicit` keeps multi-shot via `Rc<dyn Fn>` continuations; `ArcFree -> ArcFreeExplicit` keeps `Send + Sync`).

**Why both halves are first-class.** Earlier drafts of this section deferred `RcFreeExplicit` and `ArcFreeExplicit` on the rationale that they were ergonomic intersections of capabilities already covered separately. The Brand-dispatch analysis in Phase 1 step 4 disproved this: the Erased family cannot satisfy the existing `Kind` / `Functor` hierarchy under any encoding, so any Brand-dispatched multi-shot or thread-crossing Run program needs the corresponding Explicit variant. The deferred-intersection framing collapsed once Brand dispatch was held as a requirement; the resolution is to ship the full matrix and let the user pick between the speed of inherent O(1) bind and the flexibility of Brand-dispatched O(N) bind.

**Compile-cost mitigation deferred.** Cargo feature gates that let downstream crates opt out of compiling individual variants remain a deferred follow-up (Phase 6+) until benchmark or compile-time evidence justifies the added complexity.

**Deferred Coyoneda parallel intersections.** The Coyoneda family today ships four variants ([`Coyoneda`](../../../fp-library/src/types/coyoneda.rs), [`RcCoyoneda`](../../../fp-library/src/types/rc_coyoneda.rs), [`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs), [`CoyonedaExplicit`](../../../fp-library/src/types/coyoneda_explicit.rs)). The two missing intersections, `RcCoyonedaExplicit` and `ArcCoyonedaExplicit`, are deferred follow-ups whose rationale is preserved here so the contrast with the Free decision stays explicit:

- _What `RcCoyonedaExplicit<'a, F, B, A, Func>` would be:_ structurally identical to `CoyonedaExplicit` (intermediate type `B` exposed as a type parameter, function `Func: Fn(B) -> A` stored inline, single `F::map` at lower time), but with the outer wrapper swapped from `Box` to `Rc<RcCoyonedaExplicitInner>` and the boxed-function form (`.boxed()`) replaced by an `Rc<dyn Fn(B) -> A + 'a>`-shaped equivalent. `Clone` is O(1) refcount bump, matching `RcCoyoneda`. The brand `RcCoyonedaExplicitBrand<F, B>` would mirror [`CoyonedaExplicitBrand<F, B>`](../../../fp-library/src/brands.rs#L171) and provide `Functor` plus `Foldable` impls under the same `B: 'static` brand-level constraint.

- _What `ArcCoyonedaExplicit<'a, F, B, A, Func>` would be:_ thread-safe sibling of `RcCoyonedaExplicit`. `Arc<ArcCoyonedaExplicitInner>` outer wrapper, `Arc<dyn Fn(B) -> A + Send + Sync + 'a>` boxed-function form, `Send + Sync` propagation via the same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick used by `ArcCoyoneda` and `ArcFreeExplicit`. Brand integration would land via the `SendFunctor` family from Phase 1 step 6 once `ArcCoyonedaBrand`'s parallel `SendFunctor` impls are in place.

- _What they would be for:_ zero-cost compile-time map fusion in `Rc`/`Arc`-shared per-effect Coyoneda values, the same advantage `CoyonedaExplicit` already provides for the `Box` case. When a `RcRun` / `ArcRun` (or their Explicit Run siblings) program lifts an effect through `RcCoyoneda::map` / `ArcCoyoneda::map` and the per-effect map chain is deep enough that the per-layer trait-object dispatch becomes a measurable cost, the Explicit form would let the compiler inline the function chain into a single `F::map` call. The fusion advantage is identical in shape to what `CoyonedaExplicit` provides for the default `Coyoneda`; only the sharing model changes.

- _Why deferred (and why this differs from the Free intersections that were un-deferred):_ the Coyoneda Hidden trio (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`) already supports both `A: 'a` and Brand dispatch, because Coyoneda's existential is a single trait object hiding one intermediate type rather than `dyn Any` over heterogeneous values; there is no `'static` restriction to escape. The Explicit row therefore offers an _optimization_ (compile-time fusion) rather than an _enabling capability_ (Brand dispatch eligibility). The Free intersections (`RcFreeExplicit`, `ArcFreeExplicit`) were un-deferred because they were enabling — without them, Brand-dispatched multi-shot or thread-crossing Run programs were impossible. The Coyoneda intersections remain deferred because nothing impossible without them; the existing four variants already cover every capability, just without the per-effect fusion knob.

- _Trigger:_ promote one or both into the next plan revision when any of the following hold. (1) A benchmark on a real Run program backed by `RcRun` / `ArcRun` / `RcRunExplicit` / `ArcRunExplicit` shows that per-effect `RcCoyoneda::map` / `ArcCoyoneda::map` dispatch is a measurable bottleneck whose cost the missing fusion form would remove. (2) A user (internal or external) requests a `Send + Sync`-friendly equivalent of the existing `CoyonedaExplicit::map` chain that they can pair with `ArcRunExplicit`-shaped effect programs. (3) The Free family adds further variants in a future revision that demand symmetric Coyoneda support, in which case shipping the Coyoneda intersections at the same time keeps the matrix coherent.

The four-vs-six asymmetry between Coyoneda and Free is therefore intentional and structurally justified, not an oversight: Coyoneda's hidden form is genuinely sufficient at every cell where Free's erased form would lose a capability.

#### Open questions left after this decision

- Whether the `SendFunctor` family (the by-value + `Send + Sync` parallel of the existing `SendRefFunctor` family) should be added in Phase 1 as a unified set (`SendFunctor`, `SendPointed`, `SendSemimonad`, `SendMonad`) or staged across multiple phases. Phase 1 step 6 currently commits to the unified addition because `ArcFreeExplicitBrand` needs the full hierarchy; revisit only if compile time grows uncomfortable.
- Whether the six Free variants (and their six Run wrappers in Phase 2) should sit behind cargo feature gates so users of the default `Free` fast path don't pay compile cost for the others. Defer until the port is closer to shipping and downstream feedback indicates whether the cost is real.
- Whether the `Run -> RunExplicit` conversion API supports partially-handled programs (where some handlers have already narrowed the row) or restricts conversion to fully-unhandled programs only. Phase 2 step 4 commits to a concrete conversion shape; subtle invariants around handler-pipeline state may surface during implementation and inform whether to relax or tighten.

### 4.5 DECISION: Scoped-effect representation via heftia's dual row

#### Dual-row scoped-effect integration with the Free family

The port adopts heftia's dual-row architecture for scoped effects (full rationale below). Concretely, `Run` is typed against _two_ rows rather than one: a first-order algebraic row (`VariantF` of ordinary effect functors, subject to section 4.2's functor-dictionary question) and a higher-order row (a coproduct of scoped-effect constructors such as `Catch<E>` and `Local<E>`, holding their action and handler payloads as first-class values). The higher-order row does _not_ require a `Functor` instance; it is interpreted via manual case dispatch, which simplifies section 4.2 and keeps scoped effects visible as data rather than hidden in Tactical-style state threading. The six-variant Free decision in section 4.4 is orthogonal to the dual-row split: any of `Free`, `RcFree`, `ArcFree`, `FreeExplicit`, `RcFreeExplicit`, or `ArcFreeExplicit` can carry the dual row via its Free-family wrapper. See [research/deep-dive-scoped-effects.md](research/deep-dive-scoped-effects.md) for the pattern comparison and recommendation.

#### Comparison and decision

Stage 2 priority-3 research ([research/deep-dive-scoped-effects.md](research/deep-dive-scoped-effects.md)) compared four patterns for representing scoped effects (`Reader.local`, `Error.catch`, `mask`, `bracket`) under the port's encoding:

- Heftia's dual-row elaboration (scoped effects in a separate row, reified as constructors).
- Polysemy's `Tactical` state threading.
- In-other-words' Derivs/Prims reformulation with `Effly` continuation wrapper.
- Freer-simple's flat interposition (first-order only).

Plus MpEff's native multi-prompt delimited continuations as an aspirational, non-portable reference (GHC RTS primitives; ruled out by section 1.2).

**Decision: adopt heftia's dual-row architecture.**

Scoped effects live in a second row, separate from the first-order algebraic row. Each scoped effect is a concrete constructor holding its action and handler(s) as first-class values. Interpreters dispatch on the constructor via a trait with one method per variant; no Tactical state-threading, no existential functor, no reformulation pass.

**Standard scoped-effect constructors shipped with the port:**

| Constructor                             | Payload                                                                                                                                               | Models                                                                                                     |
| --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `Catch<'a, E>`                          | `action: Run<R, A>`, `handler: Box<dyn FnOnce(E) -> Run<R, A>>`                                                                                       | `Error.catch`, `try`/`catch`.                                                                              |
| `Local<'a, E>` (Val flavour)            | `modify: Box<dyn FnOnce(E) -> E>`, `action: Run<R, A>`                                                                                                | `Reader.local` (closure consumes the env).                                                                 |
| `RefLocal<'a, E>` (Ref flavour)         | `modify: Box<dyn FnOnce(&E) -> E>`, `action: Run<R, A>`                                                                                               | `Reader.local` (closure borrows the env).                                                                  |
| `Bracket<'a, A, B>` (Val flavour)       | `acquire: Run<R, A>`, `body: Box<dyn FnOnce(A) -> Run<R, (A, B)>>`, `release: Box<dyn FnOnce(A) -> Run<R, ()>>`                                       | Resource management (`Run` / `RunExplicit` users; non-refcounted substrate).                               |
| `RefBracket<'a, P, A, B>` (Ref flavour) | `acquire: Run<R, A>`, `body: Box<dyn FnOnce(P::Of<A>) -> Run<R, B>>`, `release: Box<dyn FnOnce(P::Of<A>) -> Run<R, ()>>` where `P: RefCountedPointer` | Resource management (`RcRun` / `ArcRun` / `RcRunExplicit` / `ArcRunExplicit` users; refcounted substrate). |
| `Span<'a, Tag>`                         | `tag: Tag`, `action: Run<R, A>`                                                                                                                       | Instrumentation / tracing.                                                                                 |

The exact payload shapes depend on the chosen Free variant (boxed `dyn FnOnce` for `Free`; shared `Rc<dyn Fn>` for `RcFree`, and so on). Users can define their own higher-order effects by implementing the same interpreter trait.

**Why heftia over the alternatives.**

- _vs polysemy's Tactical_: Tactical threads state through an existential functor `f ()` that must be carried end-to-end; every interpreter has to thread it via `runT`/`bindT`/`pureT`. In Rust, this pattern produces a forest of `Box<dyn Any>` and an existential `F` that `rustc` cannot infer. Heftia's dual row avoids the existential entirely; the action payload is a concrete `Run<R, A>`.
- _vs in-other-words' Effly_: Effly requires a Derivs/Prims reformulation layer that rewrites each scoped effect into a primitive effect before interpretation. This doubles the effect-definition burden and requires writing two interpreters per effect. The dual-row approach has one pass.
- _vs freer-simple's interposition_: interposition only supports _first-order_ rewrites (swap `Reader.ask` results for a modified environment during a sub-computation). It does not support true higher-order effects like `Error.catch` that need to pattern-match on exceptions thrown by the sub-computation. The port needs `Error.catch`, so interposition is insufficient.

**Why this is structurally a decision, not a blocker.** Unlike section 4.1 (where five encoding options compete on rough parity) or section 4.2 (where the functor-dictionary tradeoff is still open), the dual-row pattern is the clear winner once the port commits to first-class programs (section 4.4) and declines delimited continuations (section 1.2). The alternatives are either strictly more complex (Tactical, Effly) or strictly less expressive (interposition). This subsection is therefore titled DECISION rather than BLOCKER.

**Sub-decisions under this section:**

- _Lifetime parameter `'a` from day one._ Decision: yes. Each scoped-effect constructor is generic over a lifetime `'a` (`Catch<'a, E>`, `Local<'a, E>`, etc.) from day one, mirroring corophage's per-effect `'a` already adopted by section 4.1 Option 4 for first-order effects. This keeps the design uniform across both rows and avoids a breaking-change retrofit when `FreeExplicit` use cases want non-`'static` scoped actions.
- _Interpreter continuation type._ Decision: fixed `Run<R, A>`. The trait method that interprets a scoped effect returns a `Run<R, A>` parameterised generically by `R` and `A`; the answer type and effect row can change via the trait method's signature, but the wrapper stays `Run`. Handlers that escape to non-`Run` values (e.g., a `runPure: Run<Void, A> -> A` analogue) live as separate free functions outside the trait, mirroring PureScript Run's split between `interpret` (Run-to-Run) and `run`/`runPure` (Run-to-bare-value). This matches every Haskell library the port can credibly imitate (heftia, in-other-words, PureScript Run): row-narrowing is expressed via the handler's signature, not via a per-impl associated type. An associated continuation type is deferred to a v2 if a real use case surfaces a handler that genuinely needs different per-impl output kinds inside the trait; until then, the simpler shape is preferred.
- _User-defined extension shape._ Decision: coproduct-of-constructors (heftia-style). The higher-order row is a coproduct of concrete struct types (`Catch<'a, E>`, `Local<'a, E>`, user-defined `MyScoped<'a, ...>`, etc.); each struct gets its own interpreter impl. This mirrors the first-order row's structure (also a coproduct of effect functor structs per section 4.1 Option 4), keeping both rows uniform: programs are trees of `Pure`/`Suspend` over coproducts of constructor values, which is the data shape section 4.4 commits to as "first-class programs." A single-registration trait would invert this for the higher-order row only, breaking the symmetry; declined. Per-effect boilerplate (one struct + one impl per scoped effect) can be reduced later via a `define_scoped_effect!` macro mirroring section 9's planned `define_effect!` for first-order effects. The macro is a Phase 2 ergonomic improvement, not a Phase 1 design choice.
- _Concrete payload shapes for the standard scoped constructors._ Decision: `Catch<'a, E>` and `Span<'a, Tag>` carry the day-one `'a` parameter from the previous sub-decision; the table above lists every constructor's full payload. `Bracket` and `Local` ship in two parallel flavours (Val and Ref) that mirror the library's existing Val/Ref dispatch pattern documented at [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md); a single user-facing smart constructor (`bracket` / `local`) uses closure-driven dispatch to pick the right flavour at the call site, exactly the way `dispatch::functor::map` already does for `Functor` versus `RefFunctor`.

  **Bracket: Val and Ref flavours.** The Val flavour `Bracket<'a, A, B>` has `body: Box<dyn FnOnce(A) -> Run<R, (A, B)>>` and `release: Box<dyn FnOnce(A) -> Run<R, ()>>`. The body consumes `A`, threads it back to the interpreter via `(A, B)`, and the interpreter moves the returned `A` into `release`. The Ref flavour `RefBracket<'a, P, A, B>` has `body: Box<dyn FnOnce(P::Of<A>) -> Run<R, B>>` and `release: Box<dyn FnOnce(P::Of<A>) -> Run<R, ()>>` where `P: RefCountedPointer` is `RcBrand` for `RcRun`/`RcRunExplicit` users and `ArcBrand` for `ArcRun`/`ArcRunExplicit` users. Body and release both receive a pointer clone, the resource lives until the last clone drops, and PureScript's GC-aliased semantics from [`bracket`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs#L308) port directly.

  **Why two flavours rather than one Val flavour for everyone.** PureScript's `bracket :: Run r a -> (a -> Run r Unit) -> (a -> Run r b) -> Run r b` takes `a` by value in both `body` and `release`, relying on GC to alias the resource between the two closures. Rust has no GC, so something has to carry the aliasing. The Val flavour uses the user's return type (`(A, B)`) to thread it back. The Ref flavour uses a refcount. Both are honest; neither subsumes the other. Users of the default `Run` get the Val flavour (no extra allocation, just an explicit thread-back); users of `RcRun` / `ArcRun` are already paying for refcounting on the substrate, so refcounting the resource is in-budget and gives them the closer-to-PureScript semantics.

  **Why not the originally drafted `body: Box<dyn FnOnce(&A) -> Run<R, B>>` shape.** That shape looked like the Rust-faithful translation of PureScript's `(a -> Aff b)`, but it collapses into a useless signature on closer inspection: the returned `Run<R, B>` outlives the `&A` borrow (it is stored in a scoped-effect node and walked by the interpreter long after the body call has returned), so any closure inside the `Run` that captured `&A` would dangle. Effective bodies would be restricted to "look at `&A` synchronously, copy out `Copy` fields, return a `Run` whose closures reference nothing." Almost every real bracketed body wants the resource live throughout the body's effectful tree (e.g., reading from a `BufReader<File>` across nested binds), so this shape is rejected.

  **Local: Val and Ref flavours.** Same axis, lighter justification. `Local<'a, E>` (Val) holds `modify: Box<dyn FnOnce(E) -> E>` and matches PureScript's `local :: (e -> e) -> Run r a -> Run r a` literally. `RefLocal<'a, E>` (Ref) holds `modify: Box<dyn FnOnce(&E) -> E>`, removing the `E: Clone` requirement that the Val flavour imposes on users who want to derive a sub-scope environment from the parent without owning it. The output `E` is owned, so there is no lifetime trap (the borrow only needs to live across the synchronous call to `modify`). This is structurally the same shape as `Functor::map`'s Val/Ref split and reuses the same dispatch machinery.

  **Why `Catch` does not get a Ref flavour.** A symmetric `RefCatch<'a, E>` with `handler: Box<dyn FnOnce(&E) -> Run<R, A>>` would hit the same lifetime trap as the rejected `Bracket` shape: the returned `Run<R, A>` outlives the borrow. Errors in the surveyed languages (Haskell, PureScript) are owned values and rarely the kind of multi-ton handle a bracket guards. The Val-only signature `Catch<'a, E>` with `handler: Box<dyn FnOnce(E) -> Run<R, A>>` is correct here; users with non-`Clone` errors that they want to inspect without consuming can wrap them in `Rc` themselves.

  **Why `Span` does not split.** No closure to dispatch over.

  **Smart-constructor dispatch.** `bracket(acquire, body, release)` and `local(modify, action)` are the user-facing functions. Each is a thin wrapper over a `BracketDispatch` / `LocalDispatch` trait with two impls: `Val` for closures of the by-value shape, `Ref<P>` for closures of the by-reference / by-pointer shape. The `Ref<P>` marker carries the pointer brand so that `Ref<RcBrand>` and `Ref<ArcBrand>` are distinct dispatch impls for `bracket` (the pointer kind is part of the variant identity for the Ref Bracket flavour); `local`'s Ref impl does not need the pointer parameter because `RefLocal`'s payload is just `&E`, not `P::Of<E>`. The dispatch traits and their `Val` / `Ref` markers reuse the existing types from [`fp-library/src/dispatch/`](../../../fp-library/src/dispatch/) so the pattern is uniform across Functor, Bracket, and Local. (See "Why this isn't speculative scope creep" below.)

  **Why this isn't speculative scope creep.** The library already commits to parallel Val/Ref hierarchies for every existing operation that takes a closure ([`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md)). Shipping `Bracket` and `Local` as Val-only would invert that policy specifically for the scoped-effect surface, forcing users with non-`Clone` resources or environments to either work around it or wait for a v2 follow-up. Adding the Ref flavour now is a one-time cost (one extra struct + one extra interpreter trait method per scoped effect) and matches the rest of the library's existing surface. (A `Mask<'a, E>` constructor was considered for the standard set and is deferred; see the next sub-decision for the full options analysis and revisit triggers.)

  **Worked example: a file-reading body across nested binds.** This is the use case that drove the Val/Ref split. With `RefBracket` (Ref flavour, refcounted), the closure can capture the file handle freely:

  ```rust,ignore
  // Pseudocode using RcRun (Erased family, inherent-method only).
  // FILE is the Reader/Writer effect for opening/reading; HANDLE is Rc<File>.
  let program: RcRun<FILE, NoScoped, String> = bracket(
      open_file_for_read("data.txt"),                     // acquire: RcRun<FILE, _, File>
      |handle: Rc<File>| {                                // body: closure-driven dispatch
                                                          //   picks RefBracket<'a, RcBrand, File, String>
          run_do! {
              line1 <- read_line(Rc::clone(&handle));
              line2 <- read_line(Rc::clone(&handle));
              line3 <- read_line(handle);                 // last clone moves into the read
              pure(format!("{line1}{line2}{line3}"))
          }
      },
      |handle: Rc<File>| close_file(handle),              // release: consumes the final clone
  );
  ```

  No lifetime annotations leak to the user. The closure's `Rc<File>` argument selects `Ref<RcBrand>`, the dispatch impl emits a `RefBracket<'a, RcBrand, File, String>` node, and the interpreter knows to clone the handle once for body and once for release. Switching to `ArcRun` flips the brand to `ArcBrand` and the program type-checks unchanged. (Brand-dispatched callers using `RcRunExplicit` / `ArcRunExplicit` would write `m_do!(RcRunExplicitBrand { ... })` instead of `run_do!`; both desugar to the same chained-bind shape.)

  By contrast, the Val flavour for the default `Run` looks like:

  ```rust,ignore
  // Pseudocode using Run (the default, Box-based Free; Erased family).
  // The body threads the resource back via its return type.
  let program: Run<FILE, NoScoped, String> = bracket(
      open_file_for_read("data.txt"),                     // acquire: Run<FILE, _, File>
      |file: File| {                                      // body: closure-driven dispatch
                                                          //   picks Bracket<'a, File, String> (Val)
          run_do! {
              line1 <- read_line(&file);                  // assume read_line(&File) is a Run-shaped op
              line2 <- read_line(&file);
              pure((file, format!("{line1}{line2}")))     // thread the File back to the interpreter
          }
      },
      |file: File| close_file(file),                      // release: receives the threaded-back file
  );
  ```

  The thread-back is explicit but kept entirely inside the body closure; no special syntax, just a tuple in the body's return type. Users who prefer the implicit-aliasing semantics of `RcRun` pay one allocation and skip the tuple plumbing.

- _Deferred to a future revision: `Mask` scoped-effect constructor._ Decision: the v1 standard scoped-effect set ships as `Catch`, `Local`, `Bracket`, `Span` only. `Mask` is not included in the initial surface; users who need duplicated-effect masking can roll their own via the `define_scoped_effect!` macro until concrete demand justifies promoting one to the standard set. The full design space surveyed below is preserved so this decision can be revisited.

  **Background: two unrelated meanings of "mask".** The word "mask" carries two distinct meanings in the effects literature, and the design space differs sharply between them:
  - _Meaning 1: GHC `mask` / `mask_`(asynchronous-exception masking)._`mask :: ((forall a. IO a -> IO a) -> IO b) -> IO b`. Used inside `bracket`to guarantee that asynchronous exceptions cannot be delivered between resource-acquire and saving the handle. Requires (a) an asynchronous-cancellation mechanism in the runtime, and (b) a`restore`callback to temporarily unmask. _This is not what the heftia tradition means by "mask"._ The port's v1 is sync (section 9 item 3), so there is no asynchronous interrupt to mask against; even shipping a constructor with this name would carry no runtime semantics until Phase 6+ when`Future`-targeted async lands.
  - _Meaning 2: heftia / Eff / in-other-words "mask one layer of a duplicated effect"._ When the same effect appears twice in a handler stack, `mask::<E>(action)` makes operations of effect `E` inside `action` skip the innermost handler and reach the next one out. This is the meaning a `Mask<'a, E>` constructor with `effect: PhantomData<E>` would implement in this port. The semantics are: the interpreter sees the `Mask<E>` node, decrements its "skip count" for effect `E`, and recurses; once inside `action`, any operation of effect `E` reaches one layer deeper than usual.

  **Why "mask one layer" was considered for the standard set.** Heftia, eff, and in-other-words all ship something equivalent. Without it, programs that stack the same effect twice (often unintentionally, via library composition) hit "the inner handler ate my operation" bugs that are awkward to work around with only `Catch` / `Local` / `Bracket` / `Span`. Including `Mask` from day one would have given the port heftia parity at the cost of introducing a primitive whose name collides with GHC's unrelated `mask`.

  **Options considered.**
  - _Option A: ship as designed; rename to clarify._ Call the constructor `Skip<'a, E>` or `MaskLayer<'a, E>` instead of `Mask`. Same semantics (skip one handler layer), less name collision with GHC's async-exception `mask`. _Pros:_ reserves the bare name "Mask" for a possible future GHC-style variant that arrives with async; user docs do not need a permanent disambiguation paragraph. _Cons:_ diverges from heftia's vocabulary enough that PureScript / Haskell migrants will hunt for the term; the rename has no precedent in the surveyed Haskell ecosystem.

  - _Option B: ship as designed; keep the name `Mask`; document loudly._ _Pros:_ aligns directly with heftia's vocabulary; minimal doc surface to maintain across libraries; users coming from heftia find the expected primitive under the expected name. _Cons:_ first-time readers (especially those coming from Haskell IO or async-Rust patterns) will conflate it with GHC `mask`; needs a "this is not GHC mask" disclaimer everywhere it appears, including the rustdoc, the user guide, and any tutorials.

  - _Option C (selected): defer until a concrete test program needs duplicated-effect masking._ Ship the v1 standard set as `Catch`, `Local`, `Bracket`, `Span`; users who hit duplicated-effect bugs roll their own via `define_scoped_effect!`. _Pros:_ matches the plan's deferral pattern (e.g., `generalBracket` deferred until the async target monad lands, see Phase 6+); avoids shipping a primitive without a use case; smaller Phase 4 surface; defers the naming question (Option A vs Option B) until a real user makes the case for it; non-breaking to add later. _Cons:_ users who hit duplicated-effect bugs early have to roll their own scoped effect, which carries its own learning curve; the port is briefly less feature-complete than heftia.

  - _Option D: ship as designed but add a compile-time check that prevents using `Mask<E>` unless `E` actually appears at least twice in the row._ A type-level `CountOccurrences<E, R>: TypeNum >= 2` constraint, similar in spirit to the `Member<E, R>` trait but counting rather than asserting presence. _Pros:_ removes the failure mode where `Mask<E>` silently does nothing because there is only one handler for `E`; turns a silent surprise into a compile-time error. _Cons:_ adds significant trait-level machinery (Peano arithmetic on the row at the type level); error messages for the count constraint are notoriously hard to make legible on stable Rust; over-engineered for a v1 primitive whose use cases are not yet validated.

  **Why Option C now.**
  - The port's headline use cases (TalkF + DinnerF integration test, Reader + State + Logger sample programs, the per-effect unit tests in Phase 3) do not stack the same effect twice in any reviewed example. Shipping `Mask` from day one would mean shipping dead code with respect to the v1 test suite.
  - The naming choice (Option A's `Skip` / `MaskLayer` vs Option B's `Mask`) is non-trivial and benefits from a real user trying both labels in code before being locked in. Deferring sidesteps a coin-flip naming commitment.
  - When concrete demand surfaces, promoting a user-defined `define_scoped_effect!`-built constructor into the standard set is a non-breaking addition. Users who need it sooner can write it themselves with a few dozen lines of boilerplate.
  - The deferral aligns with the established pattern in section 9 item 8 (separate `fp-effects-macros` crate split deferred until concrete need surfaces) and Phase 6+ entries like `generalBracket` and `BracketConditions` (deferred until the async target monad lands).

  **Revisit triggers.** Promote one of Options A, B, or D to the standard set when any of the following hold:
  - A user (internal or external) reports a duplicated-effect bug that is awkward to express with `Catch` / `Local` / `Bracket` / `Span` alone.
  - The TalkF + DinnerF Phase 4 milestone or a follow-up integration test naturally wants two stacked handlers for the same effect.
  - The port adds a second standard library or framework integration (HTTP middleware, transaction wrappers, structured logging) where duplicated effects arise from layered composition rather than from direct user choice.
  - The async target monad lands in Phase 6+, at which point both meanings of "mask" (heftia "skip one layer" and GHC "asynchronous-exception mask") become live design questions and benefit from being decided together rather than in sequence.

  When the trigger fires, re-read this sub-decision: Options A, B, D remain on the table; Option C falls off; pick one, write the constructor, and move the chosen option's pros / cons inline as the new "decision" while leaving the other options here as the historical alternatives.

---

### 4.6 DECISION: Natural transformations as values

`interpret` takes a natural transformation `VariantF r ~> m` as a runtime value. In PureScript this is a polymorphic function. In Rust:

- The existing `NaturalTransformation<F, G>` trait works for `F` with a statically-known type. But `VariantF r` is an _open_ sum; its concrete representation changes with `r`.
- A natural transformation from `VariantF r` must, by construction, handle every case in `r`. In PureScript this is assembled with `case_ # on _reader handleReader # on _state handleState`. The `on` combinator threads the "smaller row" through the type of the remaining fallback.
- In Rust, the equivalent is a tuple-of-closures (one per effect) indexed by the same type-level structure as the row.

**Decision: macro DSL primary, builder fallback.** Users assemble natural transformations via a macro (`handlers!{ state: handle_state, reader: handle_reader }`) that expands to the appropriate per-effect tuple-of-closures matching the row's type-level structure. The builder pattern (`nt().on::<State<i32>>(handle_state).on::<Reader<Env>>(handle_reader)`) remains available as the fallback for users who want to bypass the macro or compose handlers programmatically. This mirrors section 4.1's workaround 1 + workaround 3 hybrid (macro primary, mechanical fallback for hand-authoring) and section 4.1's row-narrowing handler API note: the consistent design across the plan is "macro for ergonomics, type-level building blocks for escapes".

The macro design is non-trivial; ownership of the macro rests with the same crate that hosts `effects!` (per section 4.1, fp-macros or a dedicated effects sub-crate). Error-message quality is the main implementation risk; the builder fallback exists partly to give users a path with cleaner errors when the macro expansion is the source of confusion.

**Macro layer split (`effects!`, `scoped_effects!`, raw forms).** The first-order row is constructed via `effects!` (public, emits Coyoneda-wrapped Coproduct) with an internal `crate::__internal::raw_effects!` available to fp-library itself for cases that need the un-wrapped Coproduct directly (test fixtures, lower-level combinators not exposed to users). The scoped row is constructed via `scoped_effects!`, which shares the lexical-sort canonicalisation helper with `effects!` but emits a `ScopedCoproduct` shape rather than a Coyoneda-wrapped one. The shared helper means sort-correctness fixes land in one place; the two thin entry points keep the first-order vs scoped distinction visible at the call site rather than threading both through one overloaded macro. The raw `effects!` companion is internal-only so the public surface stays single-purpose; if a user-facing need for the un-wrapped form surfaces later, promoting it from `__internal` is a non-breaking addition.

---

## 5. Draft Architecture (Recommended Direction)

The blockers above resolve, for the current working hypothesis, to the following shape. This is a **draft**; prototype first.

### 5.1 Core types

```
Run<Effects, ScopedEffects, A>  = FreeFamily<Node<Effects, ScopedEffects>, A>

Node<Effects, ScopedEffects>    = First(VariantF<Effects>)
                                | Scoped(ScopedCoproduct<ScopedEffects>)

where
  FreeFamily                    = one of { Free, RcFree, ArcFree, FreeExplicit, RcFreeExplicit, ArcFreeExplicit } per section 4.4
  VariantF<Effects>             = open sum of FIRST-ORDER effect functors, encoded as a nested Coproduct
  ScopedCoproduct<ScopedEffects>= open sum of HIGHER-ORDER scoped constructors (per section 4.5)
  Coproduct<H, T>               = Here(H) | There(T)
  Void                          = empty-tail of the coproduct
  Member<E, Effects>            = trait proving E is somewhere in the coproduct
```

The user-facing type constructor is `Run<Effects, ScopedEffects, A>`. Both parameters are nested `Coproduct`s (possibly produced by `effects![...]` and `scoped_effects![...]` macros). The underlying Free variant is selected at the Run-type level: there are six concrete Run types (`Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`, `ArcRunExplicit`), one per Free variant, so users can opt in to sharing, lifetime flexibility, or Brand-dispatched typeclass-generic code without rewriting effect logic. The Erased trio is inherent-method only and used via the `run_do!` macro; the Explicit trio is Brand-dispatched and used via `m_do!` over the corresponding `*RunExplicitBrand`. Conversion between paired variants is O(N) and one-directional per call. First-order effect functors satisfy `Functor` (which the existing `Coyoneda` makes trivial to provide for any enum) — this is the trade-off for keeping the Free core instead of Freer. Scoped constructors do NOT require `Functor` (section 4.2, dual-row scope narrowing); they are interpreted via manual case dispatch per section 4.5.

### 5.2 Why Free + Coyoneda rather than Freer

The candidates were:

- **Standard Free + explicit `Functor` bound** on each effect (with `Coyoneda` as a helper for effects that aren't naturally functors).
- **Freer** (existential continuation, no `Functor` requirement).

The six-variant commitment in section 4.4 resolves this in favour of Free. Reasons:

- The existing `Free<F, A>` already implements the fast path. Building Run on top of it reuses the stack-safe CatList internals, the custom `Drop`, and the `fold_free`-via-`MonadRec` interpreter machinery. Rewriting all of that under a Freer encoding would duplicate work and introduce a divergent second set of tests.
- The POC demonstrated that `FreeExplicit` integrates with the Kind system cleanly; `RcFreeExplicit` and `ArcFreeExplicit` extend that integration with `Rc`/`Arc` outer wrappers, and the Erased siblings (`RcFree`, `ArcFree`) follow the Coyoneda playbook directly. Keeping the Free family cohesive is easier than maintaining parallel Free and Freer families.
- The per-effect `Functor` requirement that motivated Freer is cheap to satisfy in this library: `Coyoneda<F>` gives any type constructor a `Functor` instance for free. Effect enums that don't want to hand-derive `Functor` can wrap themselves in `Coyoneda` at lift time. Note that Coyoneda only addresses the "give one functor a map" half of the problem; it does not substitute for the row-polymorphic `VariantF` that `Run` still needs (see section 3.2). The split is deliberate: Coyoneda per-effect, `VariantF` across effects.
- The Coyoneda family pairs with the Free family at every Rc/Arc/Box cell: `Coyoneda` / `RcCoyoneda` / `ArcCoyoneda` / `CoyonedaExplicit`. An effect lifted into a Coyoneda matching the chosen Free variant inherits the same sharing, lifetime, and `Send`/`Sync` properties as the surrounding program, so users do not need to think about which Coyoneda to pair with which Free.

The user-facing effect definition pattern therefore looks like plain Rust enums plus one `derive` or one `Coyoneda::lift`, not a new core encoding.

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

The example above assumes `State<S>` is a natural `Functor`. For an effect that is not a natural `Functor`, the user lifts it through `Coyoneda` at construction time (section 4.2 resolution lean) and the handler lowers before pattern-matching:

```rust
fn run_logger<R, A>(
    program: Run<Coprod![Coyoneda<Logger>, ...R], A>,
) -> Run<R, (Vec<String>, A)> {
    let mut log = Vec::new();
    let mut current = program;
    loop {
        match current.peel() {
            RunStep::Pure(a) => return Run::pure((log, a)),
            RunStep::Impure(Coproduct::Here(coyo), k) => {
                // Lower the Coyoneda first to recover the original
                // Logger value, then pattern-match.
                let logger_op: Logger = coyo.lower(/* ... */);
                match logger_op {
                    Logger::Log(msg) => { log.push(msg); current = k(()); }
                }
            }
            RunStep::Impure(Coproduct::There(other), k) => {
                current = Run::impure(other, k);   // forward
            }
        }
    }
}
```

The two patterns differ only at the `Coproduct::Here(...)` line: natural-Functor effects are pattern-matched directly; Coyoneda-wrapped effects are lowered first. The rest of the handler shape is identical.

---

## 6. Implementation Roadmap

### Phase 1: Complete the Free family

Land the five missing Free variants (`RcFree`, `ArcFree`, `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) plus the `SendFunctor` trait family before starting Run work. Doing this first locks in the substrate and lets Phases 2-4 treat the choice of variant as a user-level parameter.

1. `FreeExplicit<'a, F, A>` promoted from POC with custom iterative `Drop` (section 4.4). Delete the local copies in the POC test and bench files once the promotion lands.
2. `RcFree<F, A>` following the `Free` template with `Rc` swapped in for `Box`. Continuations become `Rc<dyn Fn>`; `lower_ref(&self)` / `peel_ref(&self)` analogues to match the `RcCoyoneda` pattern. No `Send`/`Sync` bounds. Inherent-method only (no Brand).
3. `ArcFree<F, A>` following the `ArcCoyoneda` template: `Arc<dyn Fn + Send + Sync>`, associated-type bounds on the `Kind` trait (`Kind<Of<'a, A>: Send + Sync>`) to auto-derive `Send`/`Sync` without unsafe. Inherent-method only (no Brand).
4. `RcFreeExplicit<'a, F, A>` extending `FreeExplicit` with an outer `Rc<RcFreeExplicitInner>` wrapper plus `Rc<dyn Fn>` continuations: O(N) bind, multi-shot, `A: 'a`, Brand-compatible.
5. `ArcFreeExplicit<'a, F, A>` extending `FreeExplicit` with an outer `Arc<ArcFreeExplicitInner>` wrapper plus `Arc<dyn Fn + Send + Sync>` continuations and the same associated-type-bound trick as `ArcFree`: O(N) bind, thread-crossing, `A: 'a`, Brand-compatible.
6. `SendFunctor` / `SendPointed` / `SendSemimonad` / `SendMonad` trait family — by-value parallels of the existing `SendRef*` family, with `Send + Sync` bounds on the closure parameters. Required by `ArcFreeExplicitBrand` (and resolves the equivalent gap that prevents `ArcCoyonedaBrand` from implementing `Functor` today).
7. Brand registrations and `Functor`/`Pointed`/`Semimonad`/`Monad` impls (plus the by-reference siblings) for `FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`, and `ArcFreeExplicitBrand<F>` (the last via `SendFunctor` etc.). The Erased family does not get brands.
8. Per-variant Criterion benches for all six variants (bind-deep / bind-wide / peel-and-handle), documenting the O(1) vs O(N) bind-cost asymmetry.
9. A shared test suite exercising the properties each variant promises (single-shot vs. multi-shot vs. borrow-carrying vs. thread-crossing vs. Brand-dispatched) plus `compile_fail` cases (Brand-dispatched call against an Erased variant, missing `Send + Sync` on a closure passed to `ArcFreeExplicit::bind`, etc.) so future refactors do not silently regress one variant's behaviour.

### Phase 2: Core machinery

1. `Coproduct<H, T>` and `Void` types.
2. `Member<E, Index>` trait for injection/projection with type-level index.
3. `VariantF<Effects>` on top of Coproduct, carrying the per-functor `map` dictionary (or leaning on `Coyoneda` to lift non-functor effect enums).
4. Six `Run` types, one per Free variant: `Run`, `RcRun`, `ArcRun` (Erased family, inherent-method only) and `RunExplicit`, `RcRunExplicit`, `ArcRunExplicit` (Explicit family, Brand-dispatched). Each Run type is a thin wrapper over its underlying Free variant, parameterised by the first-order effect row `R` and scoped-effect row `S`.
5. `peel` / `send` / `pure` core operations, implemented once per Run variant (mostly delegation to the underlying Free family).
6. `into_explicit()` / `from_explicit()` conversion methods between paired Erased and Explicit Run variants (`Run <-> RunExplicit`, `RcRun <-> RcRunExplicit`, `ArcRun <-> ArcRunExplicit`). Walks the structure once and rebuilds in the other shape; preserves multi-shot / `Send + Sync` properties of the underlying substrate.
7. Brand registrations and trait impls for the three Explicit Run brands (`RunExplicitBrand`, `RcRunExplicitBrand`, `ArcRunExplicitBrand`), delegating to the underlying `*FreeExplicitBrand` impls from Phase 1.
8. Convenience macros: `coprod![]` for type construction, `effects![]` for first-order effect rows, `run_do!` for Erased-family monadic do-notation (desugars to inherent method calls — bypasses the Brand hierarchy that the Erased family doesn't implement). `m_do!` over the Explicit Run brands continues to work via Brand dispatch as it does for any other Brand-dispatched monad.

### Phase 3: Interpretation

1. `run` / `runPure` (iterative interpretation loop; already stack-safe in Rust).
2. `runAccum` (interpretation with threaded state).
3. `interpret` (natural-transformation-style).
4. Stack-safe variants only if an actual target monad needs them.

### Phase 4: Built-in effects

1. `State` (get, put, modify, runState).
2. `Reader` (ask, asks, local, runReader).
3. `Except` (throw, catch, runExcept).
4. `Writer` (tell, censor, runWriter).
5. `Choose` (empty, alt, runChoose; validates multi-shot).

### Phase 5: Integration

1. Bridge to existing Monad/Functor hierarchy if the `'static` limitation is resolved.
2. Brand for `Run` to enable use with existing HKT-polymorphic code.
3. Consider whether optics can be used as effect accessors (profunctor-based effect projection).

---

## 7. Non-Blocking Tasks (Mostly Mechanical)

Once the blockers in section 4 are resolved and the Phase 1 / Phase 2 machinery from section 6 exists, the following are straightforward.

- **`Run` newtype** wrapping the chosen core.
- **Per-effect enums.** Direct translation from PureScript's `data State s a = ...`.
- **Smart constructors.** `ask`, `get`, `put`, `modify`, `tell`, `throw`, `catch`. Each is a thin wrapper over `inj + lift_f`/`send`. The concrete signature for the `lift_f` combinator (inherent associated function on each Run wrapper, raw-effect input, full Coyoneda-lift / inject / `Node::First` / `send` chain inside the body) is locked in by [plan.md Phase 2 step 9](plan.md) and the [2026-04-28 resolution](resolutions.md#resolved-2026-04-28-phase-2-step-9-scope-is-under-specified).
- **`extract :: Run () a -> a`.** Trivial once the empty row type is defined.
- **`expand`.** One-line `unsafe fn` using `mem::transmute` once the row constraints prove subsetting.
- **Base-monad bridge.** A `liftEffect`-analog for any target monad we care about. The first target should probably be `Thunk` or `Identity` (pure), with `async fn` as a followup.
- **Error messages.** Rust's error messages on trait-heavy type machinery are legendary. Budget time for macro-generated human-readable errors.
- **Promote `FreeExplicit` from POC to `src/`.** See section 4.4 for the POC findings.
  - Move `FreeExplicit<'a, F, A>` and `FreeExplicitBrand<F>` from the POC test file to a new `fp-library/src/types/free_explicit.rs` module, exported from [fp-library/src/types/mod.rs](../../../fp-library/src/types.rs).
  - Implement iterative `Drop` as described in section 4.4 (the `Extract`-driven dismantling pattern borrowed from the existing [free.rs:218-225](../../../fp-library/src/types/free.rs#L218-L225)). Without this step, deep `Wrap` chains stack-overflow on drop.
  - Replace the local `FreeExplicit` definition in [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs) with `use fp_library::types::FreeExplicit;`. Decide whether to keep the file as `free_explicit_integration.rs` or supersede it with dedicated tests in `src/types/free_explicit.rs`.
  - Replace the local `FreeExplicit` definition in [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs) with the same import. The bench's local copy exists only because no `src/` module was available at POC time; it must not be retained once the real type ships.
  - Port the POC's `evaluate_identity` / `evaluate_option` helpers into a generic `evaluate` backed by `Extract`, matching the shape `Free::evaluate` already has.
  - Add class instances (`Functor`, `Pointed`, `Semimonad`, `Monad`, `NaturalTransformation`-aware `fold_free`) for `FreeExplicitBrand<F>`, matching what `Free` exposes where applicable.
- **Add `RcFree<F, A>` at `fp-library/src/types/rc_free.rs`.** Follow the [rc_coyoneda.rs](../../../fp-library/src/types/rc_coyoneda.rs) template. Steps:
  - Swap `Box<dyn FnOnce>` continuations for `Rc<dyn Fn>` so the structure can be cloned and each continuation invoked multiple times (required for multi-shot effects like `Choose`).
  - Keep the `CatList` + `Rc<dyn Any>` erasure for O(1) bind; only the outer container changes (`Rc<dyn Any>` rather than `Box<dyn Any>` so the inner state participates in `Clone`).
  - Add `lower_ref(&self)` / `peel_ref(&self)` so handlers can re-interpret the tree without consuming it. Require `A: Clone` at those call sites because the type-erased cell may be shared between branches.
  - **No `RcFreeBrand`.** The `'static` requirement from `dyn Any` erasure is incompatible with `Kind`'s `Of<'a, A: 'a>: 'a` signature; `RcFree` is inherent-method only. Brand-dispatched multi-shot programs use `RcFreeExplicit` instead.
  - Custom iterative `Drop` that inherits the existing `Free`'s `Extract`-driven dismantling when the last `Rc` reference is released.
- **Add `ArcFree<F, A>` at `fp-library/src/types/arc_free.rs`.** Follow the [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs) template. Steps:
  - Swap `Rc` for `Arc`, the type-erased cell to `Arc<dyn Any + Send + Sync>`, and add `Send + Sync` bounds on stored continuations.
  - Use the associated-type-bounds trick (`F: Kind<Of<'a, A>: Send + Sync>`) so the compiler auto-derives `Send + Sync` on `ArcFree` without unsafe.
  - **No `ArcFreeBrand`.** Same `'static` reason as `RcFree`. Brand-dispatched thread-crossing programs use `ArcFreeExplicit` instead.
  - Iterative `Drop`, same pattern as above.
- **Add `RcFreeExplicit<'a, F, A>` and `ArcFreeExplicit<'a, F, A>`.** Extend `FreeExplicit`'s concrete recursive enum with `Rc<RcFreeExplicitInner>` / `Arc<ArcFreeExplicitInner>` outer wrappers and `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>` continuations. `A: 'a` (no `'static` requirement) because the structure has no `dyn Any`. O(N) bind via spine recursion through `F::map`. **Brand-compatible:** register `RcFreeExplicitBrand<F>` and `ArcFreeExplicitBrand<F>`, implement the by-value `Functor`/`Pointed`/`Semimonad`/`Monad` hierarchy (`ArcFreeExplicitBrand` requires the new `SendFunctor` family) plus the by-reference siblings.
- **Add the `SendFunctor` trait family.** New files under `fp-library/src/classes/`: `send_functor.rs`, `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` (and any further by-value Send-aware classes the `ArcFreeExplicitBrand` impls need). Each is the by-value parallel of the existing `send_ref_*` files, with `Send + Sync` bounds on the closure parameters. Resolves the gap that today prevents `ArcCoyonedaBrand` from implementing `Functor`; that brand should also gain `SendFunctor` impls as a bonus integration.
- **Add a cross-variant `Free`-family test suite.** Co-locate with the per-variant test modules. Validate properties each variant promises: `Free` rejects multi-shot; `RcFree` supports multi-shot over `Choose`; `ArcFree` crosses a `spawn` boundary; the Explicit row accepts `&'a str` payloads and integrates with Brand-dispatched typeclass code. The POC's `TalkF`/`DinnerF` example (see [test/Examples.purs](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106)) is the natural integration test to port against each variant.

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

## 9. Pre-implementation Decisions (formerly open questions)

These items were originally open questions; each is now answered. Recorded here so the implementer has explicit decisions to refer to. They are not full blockers (they don't prevent the first line of code) but they shape phase planning and the v1 / Phase 2 / Phase 3 split. Each item is written to be readable on its own; cross-references to other sections of this document are pointers for full detail, not prerequisites for understanding.

1. **Target audience: both library authors and application developers.** Two audiences are served simultaneously: application developers who want a clean `Run<R, A>` API and never look under the hood, and library authors who need to reach into the substrate to compose effects in non-standard ways. The dual approach is achieved by a layered API where macros (`effects!` for declaring rows, `handlers!` for natural transformations as values, `define_effect!` for declaring effect types) handle the ergonomic surface for the common case, while the raw substrate types (a six-variant `Free` family with an Erased/Explicit dispatch split for monadic AST sharing and lifetime variation, a separate row of higher-order constructors for scoped operations like `Catch` and `Local`, and per-effect `Coyoneda` wrapping for the `Functor` instance the substrate needs) remain available when users need to reason about substrate behaviour. This audience choice has no direct design consequences — it is recorded as the tiebreaker for future scoping decisions, where a feature appealing to only one audience needs explicit justification.

2. **Partial interpretation: yes.** Run programs must support handlers that remove a single effect from a row while leaving others untouched. Concretely, a handler with signature `runReader: Run<R + READER, A> -> Run<R, A>` takes a program with a `Reader` effect plus other effects `R`, handles the `Reader` part, and returns a program with only `R` remaining for downstream handlers. Without partial interpretation, the system would only support full `run: Run<Void, A> -> A` (handle-everything-at-once) handlers, which would simplify the row machinery dramatically but would no longer be `Run` in any meaningful sense — the entire premise of an extensible effect system depends on incrementally peeling effects off a row. This confirms the row-polymorphism design at the core of section 4.1 and every downstream commitment.

3. **Async interaction: sync interpreters in v1, async via target-monad in Phase 3.** The library's interpreter functions (`interpret` and `interpretRec`) stay purely synchronous in v1; users who want async execution achieve it not by changing the interpreter but by choosing a `Future`-shaped target monad as the result of interpretation. Concretely, `interpret` is generic over a target monad `G: MonadRec`; if the user picks `G = Identity` or `G = Thunk` they get a sync result, if Phase 3 ships a `MonadRec` impl for a `Future`-shaped wrapper they get an async result, all without changing which interpreter function they call. This mirrors PureScript Run's approach (interpret into `Aff` for async, into `Effect` for sync). The benefit is that sync users do not pay the async tax (pinned futures, executor coupling, `async fn in trait` complications, multi-shot continuation friction), and async users do not need a separate parallel `AsyncRun<...>` family or special async-aware handler API.

4. **IO and side-effects story: `Thunk` for v1, `Future` as a `MonadRec` target for Phase 3.** The library does not introduce an `Effect`-monad analogue to PureScript's `Effect`. v1 represents deferred / IO-shaped computations as `Thunk` values (lazy unevaluated computations, already shipping in fp-library); a handler that needs to perform real IO does so at the handler boundary by running standard library functions when peeling the IO effect from the row, returning a `Thunk` containing the result. Phase 3 adds `Future` as a `MonadRec` target via the same target-monad mechanism described in item 3, giving users an async path for IO-heavy programs without changing the interpreter. The IO story is therefore unified across sync and async: it is always "user picks the target monad," whether the target is `Thunk` (lazy / sync), `Identity` (eager / sync), or `Future` (async). This avoids the design risk of inventing a Rust-specific Effect monad that might duplicate or conflict with `async`.

5. **Higher-order effects: closed by section 4.5's heftia dual-row decision.** Higher-order effects are operations that take effectful computations as arguments rather than plain values: `Reader.local` runs a sub-program with a modified environment, `Error.catch` runs a sub-program and recovers from exceptions, `Bracket` acquires a resource, runs a body, and releases. PureScript Run handles these via interposition patterns; this port goes further by reifying scoped operations as a separate row of struct constructors (`Catch<'a, E>`, `Local<'a, E>`, `Bracket<A>`, `Span<Tag>`), keeping them visible as data rather than hiding them inside `Tactical`-style existential state threading or `Effly`-style continuation wrapping. The dual-row decision is the cleanest answer found across the surveyed Haskell ecosystem (heftia is the source); see section 4.5 DECISION for the full rationale plus the sub-decisions (dual-row architecture, day-one `'a` lifetime parameter, fixed `Run<R, A>` continuation type, coproduct-of-constructors extension shape, plus the deferral analysis for `Mask` covering the four options preserved for future revisit).

6. **Performance: Criterion benches per phase.** Each phase establishes baseline measurements before adding new code, so regressions are caught early. The template is the existing `FreeExplicit` POC's Criterion bench at four depths (10, 100, 1000, 10000 nested binds), already shipping at `fp-library/benches/benchmarks/free_explicit.rs`. Phase 1 adds matching benches for the five other Free variants (`Free`, `RcFree`, `ArcFree`, `RcFreeExplicit`, `ArcFreeExplicit`) covering the standard scenarios: bind-deep (left-associated chains), bind-wide (sequential composition), and peel-and-handle (full handler walk). Per-variant benches also document the O(1) vs O(N) bind cost asymmetry between the Erased and Explicit families. Phase 2 adds row-canonicalisation benches (the macro-time sort path vs the `CoproductSubsetter` permutation-proof fallback path), handler-composition benches, and `Run -> RunExplicit` conversion benches. Approximately 10-14 benches total. The original concern that motivated this item — that Freer-style encodings allocate a closure per bind — is addressed at the design level by the Erased Free family (which avoids per-bind closure allocation by using `CatList` to store continuations as a heterogeneous list), and the benches confirm the bound holds in practice; the Explicit family pays a known O(N) cost in exchange for Brand dispatch compatibility.

7. **Lifetime constraints: closed by sections 4.4 and 4.5.** The default Free implementation requires `'static` payloads because it uses `Box<dyn Any>` for type erasure of continuation arguments, which blocks effects that hold borrowed references like `Reader<&str>`. The six-variant Free family decision in section 4.4 includes the Explicit row (`FreeExplicit<'a, F, A>`, `RcFreeExplicit<'a, F, A>`, `ArcFreeExplicit<'a, F, A>`) that uses a concrete recursive enum instead of `dyn Any`, supporting non-`'static` payloads at the cost of slower bind (concrete enum walk vs erased `CatList`). Users who need borrowed effect data — or Brand-dispatched typeclass-generic code — opt into the Explicit family; users who want O(1) bind and don't need either property stay on the Erased family. Section 4.5's day-one `'a` parameter on every scoped-effect constructor (`Catch<'a, E>`, `Local<'a, E>`, etc.) extends the same approach to the higher-order row, so non-`'static` works uniformly across the first-order and higher-order rows.

8. **Macro infrastructure: single owner, the existing `fp-macros` crate.** All effects-related macros (`effects!` for first-order row declarations, emitting Coyoneda-wrapped Coproduct as the public surface, with an `__internal::raw_effects!` companion for fp-library-internal un-wrapped construction; `scoped_effects!` for higher-order row declarations, sharing the lexical-sort helper with `effects!` but emitting `ScopedCoproduct`; `handlers!` for natural transformations as values; `define_effect!` for declaring first-order effect types; `define_scoped_effect!` for declaring scoped-effect types) ship in the existing `fp-macros` crate alongside the HKT-system macros (`Kind!`, `impl_kind!`, `Apply!`) and the do-notation macros (`m_do!`, `a_do!`). One crate, one release cadence, one place to coordinate macro semantics — the alternative (a separate `fp-effects-macros` crate) would multiply release coordination and add a parallel macro-resolution path to debug. If `fp-macros` becomes too large in Phase 3+ (e.g., compile time grows uncomfortably), split into a separate crate at that point; defer until then. See section 4.6's "Macro layer split" paragraph for the public-vs-internal surface rationale.

9. **Testing strategy: per-phase unit + integration tests, with TalkF + DinnerF as the headline integration test.** Each phase's tests cover both correctness (the code does what the type signatures promise) and regression (previously-working behaviour stays working). Phase 1 ships per-Free-variant unit tests for the six variants (`Free`, `RcFree`, `ArcFree`, `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) plus `compile_fail` tests for negative cases (handler missing an effect, wrong type ascription on an effect's payload, Brand-dispatched call against an Erased variant, etc.). Phase 2 promotes the existing standalone `poc-effect-row/` test suites — 24 tests across the feasibility suite (workaround 1 macro, workaround 3 `CoproductSubsetter` fallback, generic effects, lifetime parameters, `tstr_crates` integration) and the Coyoneda integration suite (Coyoneda is a Functor for any inner type, Coproduct dispatches Functor recursively, end-to-end row canonicalisation under Coyoneda wrapping) — into the production crate as the row-canonicalisation regression baseline. Phase 4 ports the canonical `TalkF` + `DinnerF` example from [`purescript-run/test/Examples.purs`](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106): a multi-effect program demonstrating `Reader`, `State`, `Talk`, and `Dinner` effects composed and handled in turn, ported as-faithfully-as-possible to validate that the Rust port behaves like PureScript Run for a realistic worked example. Criterion benches (item 6 above) provide the performance baseline. Together, these layers cover correctness via unit + integration tests and performance via benches.

---

## 10. Comparison Table (Approaches and Proposed Rust Design)

| Aspect                                  | `eff` (Hasura)                     | `purescript-run`                                           | Proposed Rust design                                                                                                                      |
| --------------------------------------- | ---------------------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Core mechanism                          | Delimited continuations            | Free monad                                                 | Free monad (six-variant family, section 4.4)                                                                                              |
| Effect dispatch                         | O(1) array lookup                  | O(n) peel loop                                             | O(n) peel loop                                                                                                                            |
| Open sum                                | Type-level list + array            | Row-polymorphic VariantF                                   | Nested Coproduct                                                                                                                          |
| Handler install                         | `prompt#` + push target            | Recursive interpretation                                   | Iterative loop                                                                                                                            |
| Multi-shot continuations                | Yes (via `control`)                | Yes (tree is re-interpretable; used by `Choose`)           | Yes via `RcFree`/`ArcFree` (Erased, inherent O(1)) or `RcFreeExplicit`/`ArcFreeExplicit` (Explicit, Brand O(N)); not available via `Free` |
| Thread-crossing programs                | Yes                                | Yes via `Aff`                                              | Yes via `ArcFree` (Erased) or `ArcFreeExplicit` (Brand-dispatched)                                                                        |
| Borrowed effect payloads                | N/A                                | N/A (GC)                                                   | Yes via the Explicit family (`FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`)                                                         |
| Brand-dispatched typeclass-generic code | N/A                                | Yes (via Functor / Apply / Bind / Monad instances on Free) | Yes via the Explicit family only; Erased family is inherent-method only (section 4.4)                                                     |
| Higher-order effects                    | Natural (via `locally`, `control`) | Supported (via `locally`-like patterns)                    | Needs design work                                                                                                                         |
| Stack safety                            | Native (RTS handles it)            | `MonadRec` / trampolining                                  | Iterative loops (native)                                                                                                                  |
| Runtime dependency                      | GHC RTS                            | None (pure data)                                           | None (pure data)                                                                                                                          |
| Feasible in Rust?                       | No                                 | Yes                                                        | Yes (recommended)                                                                                                                         |

---

## 11. Cross-Reference Table: PureScript Piece to Rust Status

| PureScript piece                                                                              | Rust counterpart in `fp-library` today                             | Status                                                                           |
| --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| `Free f a` (single-shot, `'static`, inherent-method)                                          | [`Free<F, A>`](../../../fp-library/src/types/free.rs)              | Present, `'static`-only, Erased family.                                          |
| `Free f a` (multi-shot via shared continuations, inherent-method)                             | `RcFree<F, A>`                                                     | Phase 1 step 2 (in progress); Erased family.                                     |
| `Free f a` (multi-shot, thread-safe, inherent-method)                                         | `ArcFree<F, A>`                                                    | Phase 1 step 3 (in progress); Erased family.                                     |
| `Free f a` (lifetime-carrying payloads, Brand-dispatched)                                     | `FreeExplicit<'a, F, A>`                                           | Phase 1 step 1 (done) + step 7 (brand impls); Explicit family.                   |
| `Free f a` (multi-shot + lifetime-carrying, Brand-dispatched)                                 | `RcFreeExplicit<'a, F, A>`                                         | Phase 1 step 4 + step 7; Explicit family.                                        |
| `Free f a` (multi-shot, thread-safe, Brand-dispatched)                                        | `ArcFreeExplicit<'a, F, A>`                                        | Phase 1 step 5 + step 7 (via `SendFunctor` family from step 6); Explicit family. |
| `liftF`                                                                                       | `Free::lift_f`                                                     | Present.                                                                         |
| `foldFree`                                                                                    | `Free::fold_free`                                                  | Present, requires `G: MonadRec + 'static`.                                       |
| `hoistFree`                                                                                   | `Free::hoist_free`                                                 | Present.                                                                         |
| `resume` / `resume'`                                                                          | `Free::resume`                                                     | Present.                                                                         |
| `MonadRec`, `tailRecM`, `Step`                                                                | `MonadRec`, `tail_rec_m`, `ControlFlow`                            | Present.                                                                         |
| `TypeEquals`, `to`, `from`                                                                    | Nothing direct. Rust generics + `PhantomData` cover it implicitly. | N/A by design.                                                                   |
| `Newtype` class                                                                               | Nothing. Rust newtypes need no abstraction.                        | N/A by design.                                                                   |
| `Natural transformation (~>)`                                                                 | `NaturalTransformation<F, G>` trait                                | Present.                                                                         |
| `Variant` (non-functor)                                                                       | Absent.                                                            | Missing, not needed for Run.                                                     |
| `VariantF`                                                                                    | Absent.                                                            | Missing; central blocker.                                                        |
| Row `Row (Type -> Type)`                                                                      | Absent.                                                            | Missing; central blocker.                                                        |
| `inj`, `prj`, `on`, `case_`, `match`, `expand`, `contract`                                    | Absent.                                                            | Missing.                                                                         |
| Row constraints (`Cons`, `Union`, `Lacks`)                                                    | Absent.                                                            | Missing; needs trait-based emulation.                                            |
| `IsSymbol`, `Proxy "label"`                                                                   | Absent.                                                            | Missing; options in blocker 4.1.                                                 |
| `Run r a`                                                                                     | Absent.                                                            | Missing.                                                                         |
| `lift`, `send`, `peel`, `resume` (Run level)                                                  | Absent (exists at Free level).                                     | Missing.                                                                         |
| `interpret`, `run`, `runRec`, `runAccum`, `runAccumRec`, `runPure`, `runAccumPure`, `runCont` | Absent.                                                            | Missing.                                                                         |
| `Run.Reader`, `Run.State`, `Run.Writer`, `Run.Except`, `Run.Choose`                           | Absent.                                                            | Missing (mechanical once `Run` exists).                                          |
| `liftEffect`, `runBaseEffect`, `liftAff`, `runBaseAff`                                        | Absent.                                                            | Missing; target choice is itself a question.                                     |

---

## 12. Summary

`fp-library` has everything it needs for the "free monad + stack-safe recursion + natural transformation" substrate. The Rust equivalents of `Free`, `MonadRec`, `Step`, and `NaturalTransformation` are already in place and close enough to the PureScript shape that `fold_free` is effectively `runRec` already.

What is missing is twofold: the **row-polymorphic open sum** (`VariantF` and its supporting type-level machinery), and **five additional `Free` siblings** plus the **`SendFunctor` trait family** that together unblock multi-shot continuations, thread-crossing programs, borrowed effect payloads, and Brand-dispatched typeclass-generic code via the Erased/Explicit dispatch split (see section 4.4).

The two remaining hard blockers:

1. **Row encoding** (section 4.1). HList / coproduct / tuple / `TypeId` dispatch. Every other piece of the port is shaped by this.
2. **Functor dictionary dispatch** (section 4.2). Static bound vs dynamic box. Choice follows from 4.1. `Coyoneda` covers the non-functor effect case.

Decided:

- **Free family (section 4.4).** Ship six variants: `Free` (existing), `RcFree`, `ArcFree`, `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`, with an Erased/Explicit dispatch split where the Erased family is inherent-method only and the Explicit family carries Brand dispatch (plus the new `SendFunctor` trait family for the `Arc`-affected variant). The POC validated the existential-free shape; the Erased Rc/Arc siblings ship in Phase 1 steps 2-3; the Explicit Rc/Arc siblings ship in Phase 1 steps 4-5; the new trait family lands in Phase 1 step 6. This resolves the former third blocker ("`'static` bound on Free") in favour of covering the whole design space up front.

The other open questions (async story, macro design, exhaustiveness trade-offs) are secondary and can be deferred until a prototype exists.

**Recommended next action:** execute Phase 1 from section 6 — land the five missing Free siblings plus the `SendFunctor` family in `src/` before starting the Run machinery. Once all six variants exist and share a cross-variant test suite, the Row and Functor-dispatch blockers become the only remaining open questions and can be tackled with confidence that the substrate is solid.
