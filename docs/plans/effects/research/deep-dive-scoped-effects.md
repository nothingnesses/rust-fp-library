# Deep dive: scoped-effect ergonomics for the Rust port

**Status:** complete
**Last updated:** 2026-04-24

## 1. Purpose and scope

This deep dive addresses the framing question from the Stage 1 classification (section 5.3):

> Which scoped-effect pattern minimises boilerplate for the Rust port while staying compatible with the chosen row encoding?

Constraint: the port commits to Option 4 (hybrid coproduct + macro sugar) with Peano indexing as the substrate (section 4.1, leaning), and to a four-variant Free family for first-class programs (section 4.4). Delimited continuations are ruled out by section 1.2.

Scoped effects in scope: `Reader::local` and `Error::catch`. These are canonical test cases because they (a) re-enter an effect handler with a modified capability, and (b) dynamically short-circuit a sub-computation. If a pattern supports these two cleanly, it covers `mask`, `bracket`, `interleave`, and similar control-flow effects.

Out of scope: multi-prompt delimiters (MpEff); evidence-passing dispatch (EvEff); delimited continuations or language-level capability syntax (Koka, Effekt). The focus is on four patterns implementable in a Free monad: heftia's dual-row elaboration, polysemy's Tactical continuation threading, in-other-words' primitive reformulation, and freer-simple's interposition layer.

## 2. The four patterns

### 2.1 Heftia: dual-row first-class elaboration

**Mechanism.** Scoped effects are reified as first-class constructors in a separate higher-order effect row. A `Catch e` effect is defined as `Catch :: f a -> Catch f a` (a value of type `f a` and a handler), not embedded in a first-order effect functor. The interpreter pattern-matches on the `Catch` constructor and uses `interposeInWith` (heftia/src/Control/Monad/Hefty/Interpret.hs:51-52, heftia-effects/src/Control/Monad/Hefty/Except.hs:50) to interpose an exception handler into the sub-computation (heftia/heftia-effects/src/Control/Monad/Hefty/Except.hs:50-51: `handleCatch (Catch action hdl) = action & interposeInWith \(Throw e) _ -> hdl e`). The `action` is a full `Eff` program, not an opaque continuation.

**Rust sketch:**

```rust
// Heftia pattern: dual rows, first-order + higher-order separated
type FirstOrderRow = Coproduct<State<i32>, Coproduct<Reader<String>, Void>>;
type HigherOrderRow = Coproduct<Catch<String>, Coproduct<Local<String>, Void>>;

// Higher-order effect constructor is first-class
enum HigherOrderOp<A> {
    Catch(
        Box<Free<VariantF<FirstOrderRow>, A>>,  // the action
        Box<dyn Fn(String) -> Free<VariantF<FirstOrderRow>, A>>,  // handler
    ),
    Local(
        Box<dyn Fn(String) -> String>,  // modifier
        Box<Free<VariantF<FirstOrderRow>, A>>,  // scoped computation
    ),
}

// Interpreter receives action as a data value, not a captured continuation
fn interpret_catch<A>(
    catch: HigherOrderOp<A>,
) -> Free<VariantF<FirstOrderRow>, A> {
    match catch {
        HigherOrderOp::Catch(action, handler) => {
            // action is inspectable, reinterpretable, clonable
            // Install a throw handler and re-run action
            // Handler sees the continuation (the rest of the program)
        }
        HigherOrderOp::Local(f, scoped) => {
            // Modify environment, run scoped, restore
        }
    }
}
```

**Type-error quality:** Excellent. A mismatch in the action type (first-order row size or handler signature) is caught at the point of construction. Row narrowing is explicit: the higher-order row is separate, so handlers are not fighting for space in the same union.

**Handler-composition complexity:** Low. The dual row sits orthogonally to the first-order machinery. A user writes one handler for `Catch` and one for `Throw`; they compose via standard `interpret` chaining.

**Row requirements:** Two separate rows (first-order and higher-order). Option 4 can accommodate this via two `VariantF` parameters or a two-tier macro `effects_first![...] + effects_higher![...]`. The row mechanism itself (Peano indexing, coproduct nesting) does not change.

**Fit with Option 4 + Peano.** Excellent fit. The dual-row architecture is orthogonal to the encoding choice. Option 4's macro can expand both rows independently. Peano indices work identically for both layers.

**Note on MpEff.** MpEff achieves the same scoped-effect first-classness without dual rows, by using multi-prompt delimiters. The effect row is unified, but scoped ops are marked specially at the type level and compiled to distinct prompt resets. Not portable to Rust without continuation support; the dual-row idea is the portable extraction.

### 2.2 Polysemy: Tactical state threading

**Mechanism.** Scoped effects are encoded as single first-order constructors with continuation-like payloads. A `Catch e` effect is defined as `Catch :: m a -> (e -> m a) -> Error e m a` (two monadic actions). To interpret, the handler uses the `Tactical` environment (polysemy/src/Polysemy/Internal/Tactics.hs:77-78) to extract the actions as reified `Sem (e ': r) (f x)` values, where `f` is an existential functor wrapping the stateful context of upstream effects. Combinators like `runT` (Tactics.hs:145-154) and `bindT` lift the actions into the Tactical environment, allowing the interpreter to manipulate them sequentially (polysemy/src/Polysemy/Internal/Union.hs:82-107, Weaving type).

**Rust sketch:**

```rust
// Polysemy pattern: scoped ops are first-order; handler uses Tactical-like wrapper
enum FirstOrderOp<A> {
    Catch {
        try_action: Box<Free<VariantF<R>, A>>,
        handler: Box<dyn Fn(String) -> Free<VariantF<R>, A>>,
    },
    Local {
        f: Box<dyn Fn(String) -> String>,
        action: Box<Free<VariantF<R>, A>>,
    },
}

// Tactical-like environment: re-entry point bundled with functor state
struct TacticalEnv<'a, F, R> {
    initial_state: F,
    // The key: can lift an Eff-shaped action (Sem r a) back out as Sem (e ': r) (F a)
    runT: &'a dyn Fn(Box<dyn Any>) -> Box<dyn Any>,  // Real: dyn Fn(Sem r a) -> Sem (e ': r) (F a)
    bindT: &'a dyn Fn(/* continuation */) -> Box<dyn Any>,
}

// Interpreter uses TacticalEnv to thread state through higher-order actions
fn interpret_catch<A>(
    env: &TacticalEnv<impl Functor, R>,
    catch: FirstOrderOp<A>,
) -> Free<VariantF<R>, A> {
    match catch {
        FirstOrderOp::Catch { try_action, handler } => {
            // Use env.runT(try_action) to lift it into Tactical, allowing state threading
            // Then bindT to lift the handler
            // The state threading is automatic
        }
        _ => todo!(),
    }
}
```

**Type-error quality:** Moderate. The Tactical wrapper hides the functor parameter `F`, which is existential. If an action has the wrong type (e.g., returns `i32` instead of `String`), the error happens when `runT` tries to unify types, and the message may be unclear because `F` is opaque.

**Handler-composition complexity:** Moderate-to-high. The Tactical machinery adds an extra layer of abstraction. Interpreters must learn to use `runT`, `bindT`, `pureT`, and understand why state threading is necessary. The "existential functor" idea is powerful but not intuitive.

**Row requirements:** Single row (first-order only). Higher-order effects are embedded as data inside the first-order effect constructors. Row size and encoding do not change. Peano indices suffice.

**Fit with Option 4 + Peano.** Good fit. The row encoding is unchanged; Tactical is a library-level abstraction on top of a standard Free monad. No special macro support is needed, though Rust's lack of a higher-rank `Fn` type makes the state-threading API harder to express safely. The Weaving type's state-threading infrastructure would need to be adapted to work with Rust's boxed closures and existential functors.

### 2.3 In-other-words: primitive reformulation

**Mechanism.** Scoped effects are defined as derived effects in terms of primitive building blocks. A derived `Local i m a` effect is reformulated into primitive `ReaderPrimLocal f m` (src/Control/Effect/Internal/Reader.hs:58). The handler receives a monad value (not a continuation) that carries the rest of the computation. When the handler calls the primitive, it threads the modified environment into the monad using a helper like `R.local` from the standard library (Reader.hs:48: `ReaderPrimLocal f (ReaderC m) -> ReaderC (R.local f m)`). This avoids the quadratic instance problem: a derived + primitive pair requires one reformulate instance, not one per interpreter.

**Rust sketch:**

```rust
// In-other-words pattern: derived effects reformulate to primitives
enum DerivedOp<A> {
    Local(Box<dyn Fn(String) -> String>, Box<dyn Any>),  // f and the action monad
}

enum PrimitiveOp<A> {
    ReaderPrimLocal(
        Box<dyn Fn(String) -> String>,
        Box<dyn Any>,  // the ReaderC monad value
    ),
}

// Reformulation: derived -> primitive
fn reformulate_local<R>(
    local_f: Box<dyn Fn(String) -> String>,
    action_monad: Box<dyn Any>,  // Monad<VariantF<R>, A>
) -> PrimitiveOp<A> {
    // Re-express Local in terms of ReaderPrimLocal
    // The key: action_monad is a value, not a continuation
    PrimitiveOp::ReaderPrimLocal(local_f, action_monad)
}

// Primitive handler uses monad combinators (like R.local from stdlib)
fn interpret_primitive(
    prim: PrimitiveOp<A>,
) -> Free<VariantF<R>, A> {
    match prim {
        PrimitiveOp::ReaderPrimLocal(f, action_monad) => {
            // threading via stdlib R.local is transparent
            // No existential state; the monad value carries everything
        }
    }
}
```

**Type-error quality:** Good. The separation of derived from primitive means errors happen at the reformulation boundary, which is user-invisible once the library is set up. The primitive API is smaller and more regular.

**Handler-composition complexity:** Low. The reformulation is boilerplate (write one instance per derived effect), but it is local and straightforward. No existential functors or complex state threading. Once the primitives are in place, the derived layer is stable.

**Row requirements:** Single row. Derived effects do not require a separate layer; they are compiled down via type-level computation. Primitives are first-order. Peano indices work identically.

**Fit with Option 4 + Peano.** Good fit. The reformulation strategy is independent of the row encoding. It is a layering idea on top of option 4's macro-sugar coproduct. The only wrinkle: in-other-words uses a `Carrier` typeclass to mediate reformulation; Rust would need a similar abstraction at the Handler level, which is sketched in section 4.2 of port-plan.md.

### 2.4 Freer-simple: interposition

**Mechanism.** Scoped effects are not special; they are library-level abstractions built from first-order primitives. A `local` combinator (freer-simple/src/Control/Monad/Freer/Reader.hs:62-69) uses `interpose` (src/Control/Monad/Freer/Internal.hs:310-325) to temporarily install a handler that intercepts `Reader` operations. The `interpose` function sits "in the middle" of the effect stack: it pattern-matches on each operation, decides whether to intercept or relay, and continues. This is runtime dynamic dispatch, not AST rewriting.

```haskell
-- Freer-simple example from Reader.hs:62-69
local :: forall r effs a. Member (Reader r) effs
      => (r -> r)
      -> Eff effs a
      -> Eff effs a
local f m = do
  r <- asks f
  interpose @(Reader r) (\Ask -> pure r) m
```

**Rust sketch:**

```rust
// Freer-simple pattern: no special scoped ops; use runtime dispatch (interpose)
enum ReaderOp {
    Ask,  // first-order; no continuation
}

// local is a combinator, not an effect
fn local<A>(
    f: impl Fn(String) -> String,
    m: Free<VariantF<R>, A>,
) -> Free<VariantF<R>, A> {
    // Compute the modified environment
    let r = asks(f);  // Read and apply f

    // Interpose: install a temporary Ask handler
    interpose::<ReaderOp, _>(
        |op| match op {
            ReaderOp::Ask => Free::pure(r.clone()),
        },
        m,
    )
}

// Interpose does pattern matching on the fly
fn interpose<Op, A>(
    handler: impl Fn(Op) -> Free<VariantF<R>, A>,
    program: Free<VariantF<R>, A>,
) -> Free<VariantF<R>, A> {
    // Walk the program; when you see Op, intercept and call handler
    // Otherwise, relay
    match program.resume() {
        FreeView::Return(a) => Free::pure(a),
        FreeView::Suspend(var) => {
            if var.is::<Op>() {
                let op = var.downcast::<Op>().unwrap();
                handler(op)
            } else {
                // Relay to the next layer
                Free::wrap(var)
            }
        }
    }
}
```

**Type-error quality:** Moderate. The interpose function uses downcast-like runtime checks, so mismatches are caught at runtime or very late at type-check time. No compile-time guarantee that the handler covers all cases.

**Handler-composition complexity:** Low to moderate. `local` and `catch` are simple library functions; no boilerplate. However, the implementation of `interpose` is intricate (it must walk the program tree correctly, handle async monad boundaries, etc.). Once `interpose` is in the library, users do not see the complexity.

**Row requirements:** Single row. No special effect types; `Ask` and `Throw` are plain first-order operations. Scoped behaviour is a runtime combinator, not a type-level distinction. Peano indices are unchanged.

**Fit with Option 4 + Peano.** Good fit. The interposition strategy is a library-level pattern that does not depend on the row encoding. Option 4's macro is unaware of scoped effects; they are handled entirely at the combinator layer. The tradeoff is that Rust's pattern matching on a union of unknowns (a `VariantF` that could contain any effect) is less ergonomic than Haskell's; the `interpose` implementation would need careful handling of the open union's type erasure.

## 3. Comparison matrix

| Axis                        | Heftia (dual-row)  | Polysemy (Tactical)      | In-other-words (primitive)  | Freer-simple (interpose) |
| --------------------------- | ------------------ | ------------------------ | --------------------------- | ------------------------ |
| Type-error quality          | Excellent          | Moderate                 | Good                        | Moderate                 |
| Handler-composition         | Low boilerplate    | Moderate boilerplate     | Low-to-moderate boilerplate | Low boilerplate          |
| Row requirements            | Two rows (no cost) | One row                  | One row                     | One row                  |
| Fit with Option 4 + Peano   | Excellent          | Good                     | Good                        | Good                     |
| Macro-integration           | Dual-row macros    | Single macro             | Single macro                | Single macro             |
| Continuation signature      | `F -> Eff a`       | `F -> Sem (e:r) (f a)`   | `Monad m => m a`            | Runtime dispatch         |
| Scoped ops first-class      | Yes                | Yes (opaque)             | Yes (via monad)             | No (combinators)         |
| Stack-safe reinterpretation | Yes (AST)          | Yes (AST)                | Yes (monad)                 | Yes (combinator)         |
| Needs existential functor   | No                 | Yes (`f`)                | No                          | No                       |
| Risk of quadratic instances | No                 | Yes (Weaving per (e, m)) | No (reformulate)            | No                       |
| Runtime overhead            | AST walk           | AST walk + state thread  | Monad bind                  | Interpose scan           |

## 4. Recommendation

**Adopt heftia's dual-row first-class elaboration pattern.** One sentence: scoped effects as first-class constructors in a separate higher-order row minimize boilerplate and align perfectly with a Free-AST design.

**Justification.** Heftia's pattern wins on the most critical axes for a Rust port:

1. **Type-error quality.** Actions and handlers are typed directly in the constructor; mismatches are caught at construction time, not buried in Tactical's existential state.
2. **Macro simplicity.** Dual-row macros are no harder than single-row macros; they expand to two independent `VariantF` parameters. No special Tactical or reformulation boilerplate needed.
3. **Composability.** Interpreters compose via standard `interpret` chaining; no need to juggle existential functors or reformulate specs.
4. **Future-proof for multi-shot.** If the port ships `RcFree` or `ArcFree` later (for `Choose`-style effects), the dual-row approach scales naturally. Polysemy's Weaving would require rewiring the state-threading infrastructure.
5. **Aligns with Option 4 philosophy.** Option 4 is about macro sugar hiding complexity. The dual-row macro is just one more tier of sugar; it is transparently nested.

**Runners-up and their tradeoffs:**

- **Polysemy (Tactical):** Excellent library design; the state-threading abstraction is powerful. But Rust's constraint on rank-N types and the existence of the existential `F` functor make it harder to implement safely. Error messages would be less clear. Recommended as a fallback if the dual-row approach proves hard to integrate with port-plan section 4.2's Functor dictionary requirement.
- **In-other-words (primitive reformulation):** Sound and elegant. The derived/primitive split solves the quadratic-instance problem. However, it adds a tier of abstraction that Rust developers would need to learn. Better as a Phase 2 optimization, once the dual-row layer is stable.
- **Freer-simple (interpose):** Lightest-weight, most idiomatic for Rust (runtime dispatch, no exotic type machinery). Good fit for a minimal MVP, but lacks the AST-level access that makes scoped effects reinterpretable and machine-readable. Recommended as a fallback if static dispatch becomes a blocker.

## 5. Rust portability wrinkles

At least three significant challenges emerge when porting heftia's approach to Rust:

### 5.1 Type inference for the higher-order constructor's action payload

In Haskell, a `Catch e` constructor carries an `f a` where `f` is automatically applied by the kind system. Rust has no equivalent automatic instantiation. The constructor must be written:

```rust
struct CatchOp<A> {
    action: Box<dyn Any>,  // Really: Free<VariantF<FirstOrder>, A>, but type-erased
    handler: Box<dyn Fn(String) -> Box<dyn Any>>,
}
```

The `dyn Any` boxes force type erasure, which defeats static checking. The alternative is to use a generic parameter:

```rust
struct CatchOp<F, A> {
    action: Free<F, A>,
    handler: Box<dyn Fn(String) -> Free<F, A>>,
}
```

But then the higher-order row itself becomes generic: `Coproduct<CatchOp<F, A>, ...>`. This couples the two rows when ideally they would remain independent. Haskell's higher-order type constructors resolve this; Rust does not have them cleanly at runtime.

**Mitigation:** Accept type erasure in the higher-order row and use `downcast_ref::<Free<VariantF<FirstOrder>, A>>` when needed. The cost is one `dyn Any` per scoped effect, amortized across all operations; it is acceptable.

### 5.2 HFunctor-equivalent machinery

Heftia's `HFunctor` or `PolyHFunctor` trait (heftia/src/Control/Monad/Hefty/Interpret.hs) provides a uniform `hmap :: (a ~> b) -> f a ~> f b` that works for higher-order functors. Rust's `Functor` trait does not generalize to this; there is no notion of a "higher-order" functor that takes a continuation transformer and produces a new one.

Workaround: define a manual `HigherOrderInterpreter` trait that dispatches on each higher-order constructor:

```rust
trait HigherOrderInterpreter {
    fn interpret_catch(&mut self, action: Box<dyn Any>, handler: Box<dyn Fn(String) -> Box<dyn Any>>) -> Free<VariantF<R>, A>;
    fn interpret_local(&mut self, f: Box<dyn Fn(String) -> String>, action: Box<dyn Any>) -> Free<VariantF<R>, A>;
    // ... one method per higher-order effect
}
```

This is boilerplate, but it is unavoidable. The dual-row pattern is not a workaround; it is the reason this boilerplate exists in every Haskell library that supports scoped effects.

### 5.3 The `'static` constraint on boxed continuations

When storing `Box<dyn Fn(String) -> Free<VariantF<R>, A>>` as the handler in a `CatchOp`, the closure must be `'static` because it will be boxed and stored in the union. If the handler closes over borrowed references (e.g., `&str`, `&mut Vec<T>`), the port must use `FreeExplicit` (section 4.4), not `Free`. The port-plan already commits to shipping `FreeExplicit` for this reason; the dual-row approach is compatible with it.

**Mitigation:** Document clearly that scoped handlers that close over non-`'static` environment must use `FreeExplicit<'a, ...>` and include a lifetime parameter on the higher-order row itself: `HigherOrderRow<'a>`. This couples the two rows at the lifetime level, which is acceptable because lifetimes are usually inferred.

## 6. Concrete plan-edit recommendations

Section 4.1 is complete. The following edits to section 4.2 and 4.4 are recommended:

1. **Section 4.2 (Functor dictionary for VariantF).** Add a note: "The dual-row approach defers the Functor-dictionary problem: the first-order row is responsible for providing `Functor` dispatch (via `DynFunctor` or `Box<dyn Fn>` erasure); the higher-order row is not a functor itself (it is a set of constructors), so it does not require a dictionary. This simplifies the decision: implement `DynFunctor` for the first-order row's `VariantF`; the higher-order row uses manual case dispatch."

2. **Section 4.4 (Free family).** Add after the "Four-variant Free family" decision: "The port adopts heftia's dual-row architecture for scoped effects. Programs are typed against `Run<FirstOrderRow, HigherOrderRow, A>`, where `FirstOrderRow` is a `VariantF` coproduct (affected by the choice in 4.1) and `HigherOrderRow` is a separate coproduct of higher-order constructors. The `HigherOrderRow` does not require a `Functor` instance; it is interpreted via manual dispatch. This aligns with the Free-AST philosophy: scoped effects are first-class values, not hidden in Tactical-style state threading."

3. **New subsection 4.5 (scoped-effect constructors).** "The higher-order row ships standard scoped-effect constructors: `Catch<E>`, `Local<E>`, `Mask`, `Bracket<A>`, `Span<Tag>`. Each is defined as a struct holding the action (a `Box<dyn Any>` or lifetime-qualified value for non-`'static` cases) and handler(s). Interpreters are written as methods on a trait that dispatches on each constructor. This is the Heftia-inspired pattern and is lower-boilerplate than Polysemy's Tactical threading or in-other-words' reformulation."

4. **Section 5.1 (handler-composition pipeline).** Note that the dual-row approach does not require per-effect handler attachment machinery (the coroutine-crate pattern). Instead, users attach handlers to the two rows independently, in a two-stage pipeline. This is simpler but less fine-grained than per-effect narrowing. If the use case for row-narrowing emerges, it can be layered on top of the dual-row foundation in Phase 2.

5. **New subsection in section 3 (inventory).** "The port's `HigherOrderRow` includes implementations for `Catch<E>`, `Local<E>`, and `Mask`, demonstrating the pattern. These serve as worked examples; users can define their own higher-order effects by implementing the same trait."

## 7. References

**Heftia sources:**

- heftia/src/Control/Monad/Hefty.hs (high-level architecture)
- heftia/src/Control/Monad/Hefty/Interpret.hs (interpreter pattern)
- heftia-effects/src/Control/Monad/Hefty/Except.hs:50-51 (Catch/Throw example with `interposeInWith`)
- heftia/README.md (dual-row motivation)

**Polysemy sources:**

- src/Polysemy/Internal/Tactics.hs:77-78 (Tactical type alias)
- src/Polysemy/Internal/Tactics.hs:145-216 (runT, bindT, state threading)
- src/Polysemy/Internal/Union.hs:82-107 (Weaving type)

**In-other-words sources:**

- src/Control/Effect/Internal/Reader.hs:18-59 (Local effect, ReaderPrimLocal, reformulation)
- src/Control/Effect/Internal.hs:48-82 (Derivs, Prims, reformulate)
- src/Control/Effect/Type/Regional.hs (Regional helper)

**Freer-simple sources:**

- src/Control/Monad/Freer/Reader.hs:62-69 (local combinator)
- src/Control/Monad/Freer/Internal.hs:310-325 (interpose)

**Port-plan references:**

- docs/plans/effects/port-plan.md:4.1 (row encoding options)
- docs/plans/effects/port-plan.md:4.2 (Functor dictionary)
- docs/plans/effects/port-plan.md:4.4 (Free family decision)
- fp-library/src/types/free.rs (current Free implementation)
