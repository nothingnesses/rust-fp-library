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

### 4.3 BLOCKER: How strong should stack-safety guarantees be

The existing `Free` is stack-safe (O(1) bind, iterative drop via `Extract`). That is sufficient for `Run`'s own stack-safety. But the PureScript library distinguishes two interpreter families:

- `interpret` / `run` / `runAccum`: assume the target monad is stack-safe.
- `interpretRec` / `runRec` / `runAccumRec`: require `MonadRec` on the target.

In Rust, this distinction is less useful: most target monads we'd write (`Option`, `Result`, `Thunk`) already implement `MonadRec` or trivially can. The open question:

- Do we ship both families, mirroring PureScript 1:1? Easier to document but doubles the surface area.
- Do we ship only the `MonadRec` family and make every interpreter stack-safe by default? Simpler, costs a few percent in common cases.

**Recommendation (not a decision):** ship only the `MonadRec` family. Revisit if we find target monads that cannot implement it.

### 4.4 DECISION: Ship a four-variant `Free` family

The existing `Free<F, A>` imposes three limitations, each driven by an independent implementation choice:

1. **`'static` only**, because `Box<dyn Any>` requires `'static`. Blocks effects that hold borrowed references (`State<&'a mut Vec<T>>`), handlers that close over non-`'static` environment data, and non-owned payloads (`&str`, `&[T]`).
2. **Single-owner, non-cloneable**, because `Box<dyn FnOnce>` consumes its callable once. Blocks multi-shot continuations — specifically the `Choose` effect from `purescript-run`, whose handler calls the continuation both for `true` and for `false`. The current `Free` cannot serve as Run's AST for multi-shot effects.
3. **Not thread-safe**, because `Box<dyn FnOnce>` has no `Send`/`Sync` bounds. Blocks effectful programs that need to cross thread boundaries.

These are the same forces that produced the [four-variant Coyoneda family](../../../fp-library/docs/coyoneda.md) (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`, `CoyonedaExplicit`). The axes are identical here: sharing model (Box / Rc / Arc) and existentiality (erased continuation types via `Box<dyn Any>` + CatList, or concrete recursive enum). The Coyoneda story covered every useful combination with four siblings. The Free story should do the same.

**Decision: ship all four Free variants, mirroring the Coyoneda family exactly.**

| Variant        | Sharing | Erasure                  | `'static`? | Cloneable? | Thread-safe? | Bind | Purpose                                                              |
| -------------- | ------- | ------------------------ | ---------- | ---------- | ------------ | ---- | -------------------------------------------------------------------- |
| `Free` (today) | `Box`   | `Box<dyn Any>` + CatList | Yes        | No         | No           | O(1) | Default; fast single-shot effect programs.                           |
| `RcFree`       | `Rc`    | `Box<dyn Any>` + CatList | Yes        | Yes, O(1)  | No           | O(1) | Multi-shot continuations (`Choose`, nondeterminism).                 |
| `ArcFree`      | `Arc`   | `Box<dyn Any>` + CatList | Yes        | Yes, O(1)  | Yes          | O(1) | Effect programs that cross thread boundaries.                        |
| `FreeExplicit` | `Box`   | concrete recursive enum  | No         | No         | No           | O(N) | Effects with borrowed payloads (`Reader<&str>`, `State<&'a mut T>`). |

The POC at [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs) validated the fourth (`FreeExplicit`). The first (`Free`) already ships. The two new ones (`RcFree`, `ArcFree`) are mechanical copies of `Free`'s internals with the outer wrapper swapped (`Box` -> `Rc` / `Arc`) and appropriate bounds added on stored closures. The closure-storage pattern from [ArcCoyoneda](../../../fp-library/src/types/arc_coyoneda.rs) is the direct template — in particular, the associated-type-bound trick (`Kind<Of<'a, A>: Send + Sync>`) that lets the compiler auto-derive `Send`/`Sync` without unsafe.

**Why ship all four at once rather than incrementally.** The API of `Run<R, A>` is shaped by which Free variant underlies it. If Run starts on `Free` and later needs multi-shot (`Choose`), switching to `RcFree` is a breaking change: user-written handlers move from `FnOnce` to `Fn`-shaped continuations, effect functors that stored `FnOnce` payloads have to change, and any previously-compiled effect program stops type-checking. The cost of the Coyoneda-style "pick the variant that fits" API has already been paid once in the library; paying it again for Free keeps the design coherent and avoids a near-certain v2 migration.

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

All four variants need stack-safe `Drop`, but the mechanism differs:

- **`Free`, `RcFree`, `ArcFree`** share the current `Free`'s `Drop` strategy: iteratively dismantle `Suspend` layers via the `Extract` trait ([free.rs:218-225](../../../fp-library/src/types/free.rs#L218-L225)). `RcFree` and `ArcFree` only need to run the dismantling when the last reference is dropped (which `Rc::drop`/`Arc::drop` already gives them); the inner CatList dismantling is identical.
- **`FreeExplicit`** needs a custom iterative `Drop` that the POC deliberately leaves out of scope. Pattern:
  1. Add an `Extract`-style trait bound on `F` at the struct definition (`F: Extract + Functor + 'a`) so `Drop` can call it. Rust requires `Drop` impl bounds to match struct bounds, so this propagates.
  2. Implement `Drop` as a loop: repeatedly take the current `Wrap(f_inner)`, call `F::extract(f_inner)` to pull out the next `Box<FreeExplicit>`, then non-recursively drop the extracted layer. When a `Pure` is reached, let default drop handle it.
  3. Caveat: this forces every effect functor used with `FreeExplicit` to implement `Extract`. For functors that cannot (e.g., effects whose payload is a continuation function rather than a concrete value), users must go through `fold_free` into a `MonadRec` target instead. Same story `Free` already tells; the bound simply propagates.

**Cleanup tasks at promotion time** are enumerated in section 7.

#### Deferred: the Rc/Arc × Explicit intersections

The sharing and existentiality axes form a 2×3 matrix. The four variants above fill four of the six cells. The remaining two cells are the "shared + concrete" corners:

|          | Box            | Rc                              | Arc                              |
| -------- | -------------- | ------------------------------- | -------------------------------- |
| Erased   | `Free`         | `RcFree`                        | `ArcFree`                        |
| Explicit | `FreeExplicit` | **`RcFreeExplicit`** (deferred) | **`ArcFreeExplicit`** (deferred) |

The equivalent deferral applies to Coyoneda (`RcCoyonedaExplicit`, `ArcCoyonedaExplicit`) — noted here because the existing Coyoneda family has already made the same decision by omitting those two corners, and this plan deliberately inherits that shape.

**Capabilities each cell would add:**

- `RcFreeExplicit` would enable multi-shot continuations (`Choose`) over borrowed effect payloads (`Reader<&'a Config>`). Sound only for immutable borrows; mutable + multi-shot is a borrow-checker violation regardless of encoding.
- `ArcFreeExplicit` would enable thread-crossing effect programs that borrow from an enclosing `std::thread::scope`. Coherent but very narrow.

**Why deferred rather than shipped:**

The four primary variants (`Free`, `RcFree`, `ArcFree`, `FreeExplicit`) are each _enabling_: without any one of them, a category of Run program is impossible with no workaround. That's what justified shipping all four up front. The two intersections are _ergonomic_: they cover combinations of two capabilities already covered separately, and users who hit the combined case can work around it by choosing either primary variant that covers one axis (clone borrowed data into owned before invoking `Choose`, for instance).

The practical consequence: adding `RcFreeExplicit` or `ArcFreeExplicit` later is a non-breaking, additive change. Users who picked `FreeExplicit` or `RcFree` in v1 do not have to migrate; the new intersection variant just opens a new opt-in path. Contrast this with the four primary variants, where deferring any of them would force a breaking API change when added later because Run's handler shape depends on which Free underlies it.

**What must be true to revisit this decision:**

- A concrete user request for `Choose` + borrowed effect payloads, or a scoped-threads + borrowed-payloads program, that cannot be expressed tolerably by cloning to owned first.
- Or a demonstration that leaving two corners of the matrix empty produces surprising error messages or teaching problems that outweigh the implementation cost.

Without one of those, shipping the intersections now is speculative generality. The door is deliberately left open; we are not shipping them yet.

#### Open questions left after this decision

- Whether Run programs targeting thread-safe execution (`ArcFree`) need a parallel `Send`-constrained `Functor`/`Monad` trait hierarchy, or whether the existing `Send`-families in `classes/send_*.rs` already cover it.
- Whether `RcFree` and `ArcFree` should be behind cargo features so users of the `Free` fast path don't pay compile cost for the other variants. Defer until the port is closer to shipping.

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
Run<Effects, A>   = FreeFamily<VariantF<Effects>, A>

where
  FreeFamily        = one of { Free, RcFree, ArcFree, FreeExplicit } per section 4.4
  VariantF<Effects> = open sum of effect functors, encoded as a nested Coproduct
  Coproduct<H, T>   = Here(H) | There(T)
  Void              = empty-tail of the coproduct
  Member<E, Effects> = trait proving E is somewhere in the coproduct
```

The user-facing type constructor is `Run<Effects, A>`. The `Effects` parameter is a nested `Coproduct` (possibly produced by a `coprod!` macro). The underlying Free variant is selected at the `Run`-type-alias level: `Run<R, A>` is one alias per variant (`RcRun`, `ArcRun`, `RunExplicit`) so users can opt in to sharing or lifetime flexibility without rewriting effect code. Effect functors satisfy `Functor` (which the existing `Coyoneda` makes trivial to provide for any enum) — this is the trade-off for keeping the Free core instead of Freer.

### 5.2 Why Free + Coyoneda rather than Freer

The candidates were:

- **Standard Free + explicit `Functor` bound** on each effect (with `Coyoneda` as a helper for effects that aren't naturally functors).
- **Freer** (existential continuation, no `Functor` requirement).

The four-variant commitment in section 4.4 resolves this in favour of Free. Reasons:

- The existing `Free<F, A>` already implements the fast path. Building Run on top of it reuses the stack-safe CatList internals, the custom `Drop`, and the `fold_free`-via-`MonadRec` interpreter machinery. Rewriting all of that under a Freer encoding would duplicate work and introduce a divergent second set of tests.
- The POC demonstrated that `FreeExplicit` integrates with the Kind system cleanly; the remaining two siblings (`RcFree`, `ArcFree`) follow the Coyoneda playbook directly. Keeping the Free family cohesive is easier than maintaining parallel Free and Freer families.
- The per-effect `Functor` requirement that motivated Freer is cheap to satisfy in this library: `Coyoneda<F>` gives any type constructor a `Functor` instance for free. Effect enums that don't want to hand-derive `Functor` can wrap themselves in `Coyoneda` at lift time. Note that Coyoneda only addresses the "give one functor a map" half of the problem; it does not substitute for the row-polymorphic `VariantF` that `Run` still needs (see section 3.2). The split is deliberate: Coyoneda per-effect, `VariantF` across effects.

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

---

## 6. Implementation Roadmap

### Phase 1: Complete the Free family

Land the three missing Free variants (`RcFree`, `ArcFree`, `FreeExplicit`) before starting Run work. Doing this first locks in the substrate and lets Phases 2-4 treat the choice of variant as a user-level parameter.

1. `FreeExplicit<'a, F, A>` promoted from POC with custom iterative `Drop` (section 4.4). Delete the local copies in the POC test and bench files once the promotion lands.
2. `RcFree<F, A>` following the `Free` template with `Rc` swapped in for `Box`. Continuations become `Rc<dyn Fn>`; `lower_ref(&self)` / `peel_ref(&self)` analogues to match the `RcCoyoneda` pattern. No `Send`/`Sync` bounds.
3. `ArcFree<F, A>` following the `ArcCoyoneda` template: `Arc<dyn Fn + Send + Sync>`, associated-type bounds on the `Kind` trait (`Kind<Of<'a, A>: Send + Sync>`) to auto-derive `Send`/`Sync` without unsafe.
4. A shared test suite exercising the properties each variant promises (single-shot vs. multi-shot vs. borrow-carrying vs. thread-crossing) so future refactors do not silently regress one variant's behaviour.

### Phase 2: Core machinery

1. `Coproduct<H, T>` and `Void` types.
2. `Member<E, Index>` trait for injection/projection with type-level index.
3. `VariantF<Effects>` on top of Coproduct, carrying the per-functor `map` dictionary (or leaning on `Coyoneda` to lift non-functor effect enums).
4. `Run<Effects, A>` type aliases for each Free variant: `Run`, `RcRun`, `ArcRun`, `RunExplicit`.
5. `peel` / `send` / `pure` core operations, implemented once per Free variant (mostly delegation to the underlying Free family).
6. Convenience macros: `coprod![]` for type construction, `effects![]` if needed.

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
- **Smart constructors.** `ask`, `get`, `put`, `modify`, `tell`, `throw`, `catch`. Each is a thin wrapper over `inj + lift_f`/`send`.
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
  - Keep the `CatList` + `Box<dyn Any>` erasure for O(1) bind; only the outer container changes.
  - Add `lower_ref(&self)` / `peel_ref(&self)` so handlers can re-interpret the tree without consuming it. Require `F::Of<'a, A>: Clone` at those call sites, as `RcCoyoneda` does.
  - `RcFreeBrand<F>` registered with `impl_kind!`. Functor / Foldable at the brand level; `Pointed` / `Semimonad` as inherent methods, matching `RcCoyoneda`'s coverage.
  - Custom iterative `Drop` that inherits the existing `Free`'s `Extract`-driven dismantling when the last `Rc` reference is released.
- **Add `ArcFree<F, A>` at `fp-library/src/types/arc_free.rs`.** Follow the [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs) template. Steps:
  - Swap `Rc` for `Arc` and add `Send + Sync` bounds on the stored continuations and `F::Of<'a, A>: Send + Sync`.
  - Use the associated-type-bounds trick (`F: Kind<Of<'a, A>: Send + Sync>`) so the compiler auto-derives `Send + Sync` on `ArcFree` and `ArcFreeBrand` without unsafe.
  - `ArcFreeBrand<F>` registered with `impl_kind!`. Foldable at the brand level; everything else inherent, matching `ArcCoyoneda`'s coverage (the HKT `Functor::map` signature lacks `Send + Sync` on its closure, so `Functor` is not implementable at the brand level for `ArcFree` either).
  - Iterative `Drop`, same pattern as above.
- **Add a cross-variant `Free`-family test suite.** Co-locate with the per-variant test modules. Validate properties each variant promises: `Free` rejects multi-shot; `RcFree` supports multi-shot over `Choose`; `ArcFree` crosses a `spawn` boundary; `FreeExplicit` accepts `&'a str` payloads. The POC's `TalkF`/`DinnerF` example (see [test/Examples.purs](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106)) is the natural integration test to port against each variant.

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

| Aspect                   | `eff` (Hasura)                     | `purescript-run`                                 | Proposed Rust design                                  |
| ------------------------ | ---------------------------------- | ------------------------------------------------ | ----------------------------------------------------- |
| Core mechanism           | Delimited continuations            | Free monad                                       | Free monad (four-variant family, section 4.4)         |
| Effect dispatch          | O(1) array lookup                  | O(n) peel loop                                   | O(n) peel loop                                        |
| Open sum                 | Type-level list + array            | Row-polymorphic VariantF                         | Nested Coproduct                                      |
| Handler install          | `prompt#` + push target            | Recursive interpretation                         | Iterative loop                                        |
| Multi-shot continuations | Yes (via `control`)                | Yes (tree is re-interpretable; used by `Choose`) | Yes via `RcFree`/`ArcFree` (not available via `Free`) |
| Thread-crossing programs | Yes                                | Yes via `Aff`                                    | Yes via `ArcFree`                                     |
| Borrowed effect payloads | N/A                                | N/A (GC)                                         | Yes via `FreeExplicit`                                |
| Higher-order effects     | Natural (via `locally`, `control`) | Supported (via `locally`-like patterns)          | Needs design work                                     |
| Stack safety             | Native (RTS handles it)            | `MonadRec` / trampolining                        | Iterative loops (native)                              |
| Runtime dependency       | GHC RTS                            | None (pure data)                                 | None (pure data)                                      |
| Feasible in Rust?        | No                                 | Yes                                              | Yes (recommended)                                     |

---

## 11. Cross-Reference Table: PureScript Piece to Rust Status

| PureScript piece                                                                              | Rust counterpart in `fp-library` today                             | Status                                             |
| --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------- |
| `Free f a` (single-shot, `'static`)                                                           | [`Free<F, A>`](../../../fp-library/src/types/free.rs)              | Present, `'static`-only.                           |
| `Free f a` (multi-shot via shared continuations)                                              | `RcFree<F, A>`                                                     | Missing; see section 4.4 / Phase 1.                |
| `Free f a` (multi-shot, thread-safe)                                                          | `ArcFree<F, A>`                                                    | Missing; see section 4.4 / Phase 1.                |
| `Free f a` (lifetime-carrying payloads)                                                       | `FreeExplicit<'a, F, A>`                                           | POC only; promotion pending section 4.4 / Phase 1. |
| `liftF`                                                                                       | `Free::lift_f`                                                     | Present.                                           |
| `foldFree`                                                                                    | `Free::fold_free`                                                  | Present, requires `G: MonadRec + 'static`.         |
| `hoistFree`                                                                                   | `Free::hoist_free`                                                 | Present.                                           |
| `resume` / `resume'`                                                                          | `Free::resume`                                                     | Present.                                           |
| `MonadRec`, `tailRecM`, `Step`                                                                | `MonadRec`, `tail_rec_m`, `ControlFlow`                            | Present.                                           |
| `TypeEquals`, `to`, `from`                                                                    | Nothing direct. Rust generics + `PhantomData` cover it implicitly. | N/A by design.                                     |
| `Newtype` class                                                                               | Nothing. Rust newtypes need no abstraction.                        | N/A by design.                                     |
| `Natural transformation (~>)`                                                                 | `NaturalTransformation<F, G>` trait                                | Present.                                           |
| `Variant` (non-functor)                                                                       | Absent.                                                            | Missing, not needed for Run.                       |
| `VariantF`                                                                                    | Absent.                                                            | Missing; central blocker.                          |
| Row `Row (Type -> Type)`                                                                      | Absent.                                                            | Missing; central blocker.                          |
| `inj`, `prj`, `on`, `case_`, `match`, `expand`, `contract`                                    | Absent.                                                            | Missing.                                           |
| Row constraints (`Cons`, `Union`, `Lacks`)                                                    | Absent.                                                            | Missing; needs trait-based emulation.              |
| `IsSymbol`, `Proxy "label"`                                                                   | Absent.                                                            | Missing; options in blocker 4.1.                   |
| `Run r a`                                                                                     | Absent.                                                            | Missing.                                           |
| `lift`, `send`, `peel`, `resume` (Run level)                                                  | Absent (exists at Free level).                                     | Missing.                                           |
| `interpret`, `run`, `runRec`, `runAccum`, `runAccumRec`, `runPure`, `runAccumPure`, `runCont` | Absent.                                                            | Missing.                                           |
| `Run.Reader`, `Run.State`, `Run.Writer`, `Run.Except`, `Run.Choose`                           | Absent.                                                            | Missing (mechanical once `Run` exists).            |
| `liftEffect`, `runBaseEffect`, `liftAff`, `runBaseAff`                                        | Absent.                                                            | Missing; target choice is itself a question.       |

---

## 12. Summary

`fp-library` has everything it needs for the "free monad + stack-safe recursion + natural transformation" substrate. The Rust equivalents of `Free`, `MonadRec`, `Step`, and `NaturalTransformation` are already in place and close enough to the PureScript shape that `fold_free` is effectively `runRec` already.

What is missing is twofold: the **row-polymorphic open sum** (`VariantF` and its supporting type-level machinery), and **three additional `Free` siblings** that together unblock multi-shot continuations, thread-crossing programs, and borrowed effect payloads (see section 4.4).

The two remaining hard blockers:

1. **Row encoding** (section 4.1). HList / coproduct / tuple / `TypeId` dispatch. Every other piece of the port is shaped by this.
2. **Functor dictionary dispatch** (section 4.2). Static bound vs dynamic box. Choice follows from 4.1. `Coyoneda` covers the non-functor effect case.

Decided:

- **Free family (section 4.4).** Ship four variants: `Free` (existing), `RcFree`, `ArcFree`, `FreeExplicit`. The POC validated the fourth; the first already ships; the middle two are mechanical copies of existing Coyoneda siblings. This resolves the former third blocker ("`'static` bound on Free") in favour of covering the whole design space up front.

The other open questions (async story, macro design, exhaustiveness trade-offs) are secondary and can be deferred until a prototype exists.

**Recommended next action:** execute Phase 1 from section 6 — land the three missing Free siblings in `src/` before starting the Run machinery. Once all four variants exist and share a cross-variant test suite, the Row and Functor-dispatch blockers become the only remaining open questions and can be tackled with confidence that the substrate is solid.
