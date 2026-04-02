# Implementing Algebraic Effects as a Library: Approaches, Prerequisites, and Trade-offs

This document surveys every major strategy for implementing algebraic effects as a library in an existing language, with concrete implementation details, what your host language needs to provide, and the trade-offs you'll face.

---

## Approach 1: Free Monad over Extensible Sums

**Reference implementations:** PureScript `Run`, Haskell `freer-simple`, Scala `Eff` (atnos-org)

### How it works

You represent an effectful computation as a data structure — a free monad — that records each effectful operation as a node in a tree. The tree is then _interpreted_ by a handler that walks the structure, pattern-matching on effect operations and producing concrete results.

The core type of the standard free monad is:

```
data Free f a
  = Pure a
  | Impure (f (Free f a))
```

This requires `f` to be a Functor. The _freer_ monad (used by `freer-simple` and related libraries) eliminates this constraint by existentially quantifying over the intermediate type:

```
data Freer f a
  = Pure a
  | forall x. Impure (f x) (x -> Freer f a)
```

This separates the effect operation (`f x`) from the continuation (`x -> Freer f a`), so individual effects no longer need to be functors — they are plain GADTs describing operations and their return types. PureScript's `Run` uses the standard free monad (with the Functor requirement), while `freer-simple` uses the freer encoding.

To make effects extensible, `f` is an _open sum_ (extensible variant/coproduct) of individual effect types. PureScript's `Run` uses `VariantF` (a row-polymorphic variant of functors); Haskell libraries typically use a type-level list with membership constraints.

In the standard free monad, each effect is a functor whose constructors encode the operation's parameters and a continuation slot:

```
data State s a
  = Get (s -> a)
  | Put s (() -> a)
```

In the freer encoding, effects are instead plain GADTs without a continuation slot:

```
data State s a where
  Get :: State s s
  Put :: s -> State s ()
```

An interpreter peels one layer of the free monad at a time, matching on the effect tag and either handling it or forwarding unhandled effects.

### Prerequisites from the host language

- **Higher-kinded types or a way to simulate them.** The standard free monad requires parameterizing over functors (`f` in `Free f a`), which needs HKTs. Languages with HKTs (Haskell, Scala, PureScript) make this natural. The freer encoding reduces this requirement somewhat (effects are plain GADTs, not functors), but you still need HKTs or equivalent for the `Freer` type itself. Languages without HKTs (Rust, TypeScript) require workarounds (defunctionalization, trait-based emulation, or giving up on some generality).
- **Sum types or extensible variants.** You need a way to combine multiple effect functors into one open sum. Row polymorphism (PureScript) is the cleanest. Type-level lists with membership type classes (Haskell) work but require more boilerplate and can produce confusing type errors. Tagged unions with runtime dispatch (dynamically typed languages) are the simplest but sacrifice static safety.
- **Tail-call optimization or explicit trampolining.** Interpreting a free monad is inherently recursive. Without TCO, you need to convert to a loop using a trampoline (explicit `Step`/`Done` constructors), as PureScript's `Run` does with its `*Rec` interpreter variants.

### Trade-offs

| Advantage                                                                 | Disadvantage                                                                        |
| ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| No runtime/compiler modifications needed                                  | Overhead from allocating and interpreting the tree structure                        |
| Portable across any language with the prerequisites                       | Performance degrades with effect-heavy code (every `bind` allocates)                |
| Effects are first-class data — you can serialize, inspect, or replay them | Boilerplate for defining each effect's functor                                      |
| Multi-shot continuations are trivial (just re-interpret the tree)         | Higher-order effects (like `local`, `catch`) are notoriously difficult to get right |
| Easy to reason about — the semantics is just a fold                       | Stack safety requires explicit care                                                 |

### Pitfall: Higher-order effects

The free monad approach struggles with "scoped" or higher-order effects — operations that take effectful computations as arguments (e.g., `local` for Reader, `catchError` for exceptions). The problem is that the continuation in a free monad is _the rest of the computation_, not a delimited scope. Various workarounds exist (weaving/threading in `fused-effects`, Tactics in `polysemy`, the "hefty algebras" approach in `heftia`) but all add complexity. If your use cases are purely first-order/algebraic effects, this isn't an issue. If you need `bracket`, `local`, `mask`, etc., budget significant design effort here.

---

## Approach 2: ReaderT IO / Evidence-in-Environment

**Reference implementations:** Haskell `effectful`, Haskell `cleff`

> **Note:** Haskell `bluefin` is sometimes grouped with these libraries, but it uses a distinct capability-passing design rather than the ReaderT IO pattern. It is listed separately in the implementations table at the end of this document.

### How it works

Instead of building a data structure, you run effects directly in IO (or your language's native side-effect mechanism), threading an _environment_ that maps effect labels to their current handler implementations. The `Eff` monad is essentially `ReaderT Env IO` where `Env` is a mutable, indexed collection of handler records.

When you "perform" an effect, you look up the handler in the environment and call it directly — there's no intermediate data structure. When you install a handler, you push a new entry into the environment (shadowing the old one for that effect), run the inner computation, then pop it.

State effects are implemented by allocating an `IORef` (mutable reference) and storing read/write functions in the environment. Error effects are implemented as native exceptions. This means effects compose with IO naturally and have predictable semantics in the presence of concurrency and asynchronous exceptions.

### Prerequisites from the host language

- **A native side-effect mechanism** (IO monad, mutable state, exceptions). This approach is fundamentally impure — you're using real mutation and exceptions under the hood, with a typed wrapper on top.
- **Mutable references** (IORef, Ref, AtomicReference, etc.) for state effects.
- **An indexable, mutable environment.** Typically implemented as a mutable array or vector indexed by effect ID (often derived from a type-level list position). Fast random access is important.
- **Native exceptions** for error effects (optional but strongly recommended for performance and correct interaction with IO).
- **A way to track effects in types.** Type-level lists with membership constraints (Haskell), intersection types (Scala), or phantom type parameters. Without this, you lose static effect safety, though you can still get the performance benefits.

### Trade-offs

| Advantage                                                                                                         | Disadvantage                                                                                                             |
| ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Extremely fast — effect dispatch is a direct function call via a mutable lookup, no data structure interpretation | Cannot support true algebraic effects (multi-shot continuations, nondeterminism) because you can't capture and replay IO |
| Predictable semantics with concurrency, exceptions, and resource management                                       | You're effectively doing dependency injection with extra steps — philosophically less "algebraic"                        |
| Natural interop with existing IO-based code                                                                       | Higher-order effects require `MonadUnliftIO`-style machinery, which has its own limitations                              |
| Minimal allocations, GHC optimizes well                                                                           | Tied to a specific runtime (IO); pure interpretation (for testing) requires a separate code path                         |
| The "ReaderT IO" pattern is already widely understood                                                             | Effect handlers can't inspect or transform the continuation — they're just functions                                     |

### Key insight: What you give up

This approach cannot implement effects that require capturing the continuation. You cannot implement nondeterminism (calling resume multiple times to explore branches), cooperative scheduling via `yield` (suspending and later resuming a computation), or any effect where the handler needs to "pause" the performer and do something else before resuming. If your use case is primarily State, Reader, Error, Writer, and IO-wrapping, this approach is ideal. If you need generators, coroutines, backtracking, or async-as-an-effect, you need a different approach.

---

## Approach 3: Delimited Continuations (Native)

**Reference implementations:** OCaml 5 effect handlers, Hasura `eff` (GHC), Racket, Scheme

### How it works

The language runtime provides primitives for capturing and reinstating slices of the call stack. The two fundamental operations are:

- **`perform`** (or `shift`): Capture the continuation from the current point up to the nearest enclosing handler (delimiter/`reset`), package it as a first-class value, and transfer control to the handler.
- **`continue`** (or `resume`): Reinstate the captured continuation, splicing its stack frames back onto the current stack, and feed it a value.

In OCaml 5, this is implemented with _fibers_ — heap-allocated, dynamically growing stack segments. The program stack is a linked list of fibers. Installing a handler allocates a new fiber. `perform` detaches everything above the handler's fiber and packages it as a continuation object. `continue` reattaches it. No stack frames are copied for one-shot continuations.

### Prerequisites from the host language

- **Runtime support for stack capture/reinstatement.** This is the hard part. You need the ability to snapshot a range of stack frames and later splice them back. This typically requires either:
  - Modifying the language runtime (as OCaml 5 and the proposed GHC changes do)
  - Using platform-specific tricks (like `setjmp`/`longjmp` combined with stack copying, as Koka's `libhandler` does in C)
  - Using OS-level fibers or green threads (e.g., Project Loom on the JVM)
  - CPS-transforming the entire program (a compiler approach, not a library approach)
- **If one-shot only:** Much simpler — you can use coroutines or fibers without copying. OCaml enforces one-shot dynamically (`Continuation_already_resumed` exception).
- **If multi-shot:** You need the ability to copy/clone stack frames, which is significantly more complex and expensive.

### Trade-offs

| Advantage                                                                                            | Disadvantage                                                                                                           |
| ---------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Highest performance for effect dispatch — no encoding overhead                                       | Requires runtime modifications or platform-specific tricks                                                             |
| Supports true algebraic effects including suspension, resumption, and (if multi-shot) nondeterminism | Not implementable as a pure library in most languages                                                                  |
| Direct-style code (no monadic wrapping, no CPS, no generators)                                       | One-shot restriction (OCaml, eff) precludes some effects                                                               |
| Composable — multiple handlers compose naturally via nesting                                         | Continuation objects interact subtly with resources (file handles, locks) — must `discontinue` abandoned continuations |
| Enables concurrency patterns (fibers, coroutines, schedulers) as user-space libraries                | Debugging stack traces can be confusing with captured continuations                                                    |

### Implementing without runtime modifications

If you can't modify the runtime, you can approximate delimited continuations using:

- **Coroutines/fibers** (see Approach 4)
- **CPS transformation** (see Approach 5)
- **`setjmp`/`longjmp` + manual stack management** (C/C++ only; see Koka's `libhandler`)
- **Platform continuations** (JVM Project Loom virtual threads, .NET fibers)

---

## Approach 4: Coroutines / Asymmetric Coroutines

**Reference implementations:** Lua and Ruby libraries from "One-shot Algebraic Effects as Coroutines" (Kawahara & Kameyama, 2020), JavaScript `effects.js`, Python implementations using generators

### How it works

The key insight from the paper "One-shot Algebraic Effects as Coroutines" is that **one-shot algebraic effects can be directly embedded in any language with asymmetric (stackful) coroutines.** The rest of a coroutine's execution _is_ the one-shot continuation.

The translation works as follows:

1. Each handler installs a coroutine. The handled computation runs inside this coroutine.
2. When the computation performs an effect, it `yield`s the effect operation (tag + arguments) to the handler coroutine.
3. The handler receives the yielded effect, inspects it, and decides what to do.
4. To resume, the handler sends a value back into the coroutine via `resume(value)`.
5. To abort (like an exception), the handler simply doesn't resume — the coroutine is abandoned.

The handler dispatches by checking if the yielded effect matches its handled effect. If not, it re-yields the effect outward (forwarding to an outer handler), which naturally gives you the stack-search semantics of dynamic handler lookup.

### Prerequisites from the host language

- **Stackful (asymmetric) coroutines.** The coroutine must be able to yield from _any depth_ in the call stack, not just from the top-level coroutine body. This rules out JavaScript generators (which are stackless — `yield` can only appear directly in the generator function body, not in functions it calls). Languages with stackful coroutines include: Lua, Ruby (Fibers), Kotlin (coroutines), Python (with greenlet or similar), Go (goroutines, though not directly usable this way).
- **If you only have stackless generators** (JavaScript, Python native generators): You can still implement effects, but every intermediate function in the call chain must also be a generator and must `yield*` / `yield from` through the effects. This is the "coloring problem" — your entire call chain becomes infected with generator syntax. Some libraries (like `effects.js` in JavaScript) accept this cost and use generators as a "do-notation" for the effect monad.
- **Some form of tagged values** for effect operations (objects, tagged tuples, etc.)

### Stackful vs. Stackless: A critical distinction

| Feature                   | Stackful (Lua, Ruby Fiber, Kotlin)          | Stackless (JS generators, Python generators)           |
| ------------------------- | ------------------------------------------- | ------------------------------------------------------ |
| Yield from nested calls   | Yes — yield captures the full call chain    | No — yield only works at the generator's top level     |
| Call chain "coloring"     | No — ordinary functions can perform effects | Yes — all intermediate functions must be generators    |
| Implementation complexity | Lower for the library author                | Higher — must propagate yields manually                |
| Performance               | Generally better                            | Overhead from generator allocation at each level       |
| Multi-shot                | No (coroutine state can't be cloned)        | Possible with replay (re-run the generator from start) |

### Trade-offs

| Advantage                                                     | Disadvantage                                                                                       |
| ------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Works as a library in many existing languages                 | One-shot only (can't clone coroutine state)                                                        |
| Simple implementation (the paper's Lua library is very small) | Stackless coroutines require "coloring" the entire call chain                                      |
| Direct-style code (with stackful coroutines)                  | Performance overhead from coroutine creation and context switching                                 |
| No compiler modifications needed                              | Effect forwarding (re-yielding unhandled effects) adds latency proportional to handler stack depth |
| Natural fit for languages that already have coroutines        | Debugging can be confusing — stack traces go through coroutine boundaries                          |

### Practical example (Lua sketch)

```lua
-- Create a new effect
local Read = eff.inst("Read")
local Write = eff.inst("Write")

-- Perform effects in user code
function add_and_get(n)
  local x = eff.perform(Read)
  eff.perform(Write, x + n)
  return x + n
end

-- Handle effects
local result = eff.handler({
  [Read] = function(resume)
    return resume(current_state)
  end,
  [Write] = function(resume, val)
    current_state = val
    return resume()
  end,
}, add_and_get, 5)
```

---

## Approach 5: CPS Transformation (Selective or Global)

**Reference implementations:** Koka (compiler-level), Scala `Effekt` library

> **Note:** The TypeScript library commonly known as "Effect" (formerly `effect-ts` / `@effect-ts/core`) is sometimes mentioned in this context, but it uses a ZIO-inspired fiber-based runtime architecture, not CPS transformation. It belongs more naturally alongside Approach 2 (environment-passing with a typed effect channel) than here.

### How it works

Continuation-Passing Style transforms direct-style code so that instead of returning a value, every function takes an extra argument — the continuation — representing "what to do next." An effect handler is then just a function that receives the continuation and decides whether/how to call it.

The key insight from Koka's design is that **you don't need to CPS-transform everything** — only code that actually uses resumable effects needs CPS. This is called _selective CPS transformation_. Code that only uses "tail-resumptive" effects (where the handler's last action is to resume) can be compiled to direct function calls with zero overhead.

As a library approach (rather than a compiler approach), you can implement CPS effects using:

1. **Monadic CPS:** Wrap computations in a continuation monad. The `Effekt` Scala library uses a _multi-prompt delimited continuation monad_ — each handler installs a "prompt" (delimiter), and performing an effect captures the continuation up to the matching prompt.

2. **Callback-style:** In languages without monadic syntax, use callbacks explicitly. This is essentially what `async/await` does for a single effect (Promise), generalized to multiple effects.

### Prerequisites from the host language

- **For the monadic approach:** First-class functions, and ideally some form of for-comprehension / do-notation / monad syntax to make CPS code readable. Without syntactic support, CPS code becomes deeply nested callbacks ("callback hell").
- **For selective CPS at the library level:** You need a way to distinguish "needs CPS" from "doesn't need CPS" computations, which requires either type-level tracking or a code transformation tool.
- **For the capability-passing variant (Effekt):** The language should support some form of implicit/contextual parameter passing. Scala's `implicit` / `given` / `using` is ideal. Kotlin's context receivers work. Without implicit passing, you must thread capabilities manually, which is verbose but functional.

### The Effekt / Capability-Passing variant

The `Effekt` library (and the Effekt language) takes a distinctive approach:

- Effect handlers are _capabilities_ — objects that are passed (often implicitly) to effectful functions.
- Performing an effect = calling a method on the capability object.
- The capability carries a reference to its handler's prompt, so **handler lookup is lexical, not dynamic.** This avoids the "handler hijacking" problem where a re-raised effect accidentally matches a different handler than intended.
- Under the hood, calling an effect operation captures a multi-prompt delimited continuation using the capability's prompt marker.
- Effect safety is achieved through the type system: capabilities carry abstract type members that appear in the effect type, and handlers introduce capabilities whose types are existentially quantified, so they can only be used within the handler's scope.

### Trade-offs

| Advantage                                                                | Disadvantage                                                                                 |
| ------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| Supports multi-shot continuations (CPS naturally allows re-invocation)   | Without syntactic support, code is deeply nested / requires monadic notation                 |
| Selective CPS can achieve near-zero overhead for tail-resumptive effects | Global CPS transforms every function, which destroys stack traces and makes debugging harder |
| Capability-passing gives lexical (predictable) handler resolution        | Implementation of multi-prompt delimited continuations is non-trivial                        |
| Works as a library (Effekt in Scala proves this)                         | Performance of the continuation monad can be poor without optimization                       |
| Avoids the "handler hijacking" problem (capability-passing variant)      | Interop with existing non-CPS code requires wrapping/lifting                                 |
| Theoretically clean — backed by well-studied CPS semantics               | Implicit/contextual parameter passing is needed for ergonomic capability passing             |

---

## Approach 6: Evidence Passing (Compile-time Resolved)

**Reference implementation:** Koka (compiler-level), partially approximated by `cleff` and `effectful` in Haskell

### How it works

This is Koka's signature compilation strategy, described in the paper "Generalized Evidence Passing for Effect Handlers." Instead of dynamically searching the stack for a handler at runtime, the compiler transforms effectful code to explicitly pass handler "evidence" as hidden function parameters. Each effect operation is dispatched by directly calling the corresponding handler via this evidence — O(1) dispatch with no runtime search.

The transformation works as follows:

1. Effect types are tracked via row polymorphism.
2. For each effect in a function's effect row, the compiler adds a hidden parameter carrying the handler's evidence (essentially a vtable/record of handler functions + a prompt marker for the continuation).
3. When an effect is performed, the compiler emits a direct call through the evidence parameter.
4. When a handler is installed, the compiler creates a new evidence value and passes it to the handled computation.

### Prerequisites for a library approximation

- **Type-level effect tracking** (row polymorphism or type-level lists) so you know _which_ evidence to pass.
- **Compiler plugin or code generation** to automate the evidence threading. Without automation, you're manually passing extra arguments everywhere, which is what `cleff` and `effectful` effectively do (using a mutable environment as a runtime approximation of evidence passing).
- **Ideally, implicit parameters or type-class-based dispatch** to make the evidence passing ergonomic.

### Trade-offs

| Advantage                                                      | Disadvantage                                                           |
| -------------------------------------------------------------- | ---------------------------------------------------------------------- |
| O(1) effect dispatch — no runtime search, no dynamic lookup    | Requires a compiler or preprocessor, not a pure library technique      |
| Amenable to aggressive optimization (inlining, specialization) | Adds complexity to the compilation pipeline                            |
| Works well with strict/eager evaluation                        | The monadic transformation needed internally is complex                |
| Interacts cleanly with type-level effect information           | Approximations (mutable environment) lose some theoretical guarantees  |
| No runtime needed — compiles to plain C (in Koka's case)       | Higher-order effects require additional "evidence-threading" machinery |

---

## Approach 7: Monad Transformers / MTL-style

**Reference implementations:** Haskell `mtl`, Scala `cats-mtl`

### How it works

Each effect is a monad transformer that wraps another monad. State is `StateT`, errors are `ExceptT`, etc. Effects are composed by stacking transformers. Type classes (e.g., `MonadState`, `MonadError`) provide an abstract interface, and instances are derived for each transformer stack.

This is the oldest approach and predates algebraic effects. It's included here because it's the baseline against which all other approaches are compared, and because understanding its limitations motivates the alternatives.

### Prerequisites

- **Monad transformers** or equivalent (monad type classes, higher-kinded types).
- **Type class machinery** for deriving instances through transformer stacks (`lift`/`liftIO`).

### Trade-offs

| Advantage                                                 | Disadvantage                                                                                                |
| --------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Extremely mature, well-understood, widely available       | O(n) lifting cost for n transformer layers                                                                  |
| Excellent tooling and documentation                       | Transformer ordering matters and affects semantics (e.g., `StateT s (ExceptT e)` vs `ExceptT e (StateT s)`) |
| No special runtime support needed                         | N² instance problem: each transformer needs instances for every type class                                  |
| Good performance for simple stacks (GHC specializes well) | Adding/removing effects from the middle of the stack is painful                                             |
|                                                           | Higher-order effects have surprising semantics (e.g., `catchError` discards state)                          |

---

## Decision Matrix

### "What does my host language have?" → "What approach should I use?"

| Host Language Features                                            | Best Approach                                        | Runner-up                         |
| ----------------------------------------------------------------- | ---------------------------------------------------- | --------------------------------- |
| **Stackful coroutines** (Lua, Ruby, Kotlin)                       | Coroutines (Approach 4)                              | Free monad (Approach 1)           |
| **Stackless generators + HKTs** (PureScript, Haskell)             | Free monad (Approach 1)                              | ReaderT IO (Approach 2)           |
| **Native IO + mutable refs + type-level tricks** (Haskell)        | ReaderT IO (Approach 2)                              | Free monad (Approach 1)           |
| **Implicit parameters + delimited continuations monad** (Scala 3) | Capability-passing CPS (Approach 5)                  | Free monad (Approach 1)           |
| **Runtime-level continuations** (OCaml 5, Scheme, Racket)         | Native delimited continuations (Approach 3)          | —                                 |
| **Only closures + basic types** (Python, Go, C)                   | Coroutines if available; otherwise CPS callbacks     | Free monad (with manual encoding) |
| **JavaScript (generators only)**                                  | Generator-based CPS (Approach 5 + Approach 4 hybrid) | Free monad via generators         |

### "What effects do I need?" → "What approaches support them?"

| Effect Pattern                        | Free Monad   | ReaderT IO     | Native Delimited Cont. | Coroutines    | CPS |
| ------------------------------------- | ------------ | -------------- | ---------------------- | ------------- | --- |
| State, Reader, Writer                 | ✅           | ✅ (best perf) | ✅                     | ✅            | ✅  |
| Exceptions (non-resumable)            | ✅           | ✅ (native)    | ✅                     | ✅            | ✅  |
| Resumable exceptions                  | ✅           | ❌             | ✅                     | ✅ (one-shot) | ✅  |
| Generators / yield                    | ✅           | ❌             | ✅                     | ✅            | ✅  |
| Cooperative concurrency               | ✅           | ❌             | ✅ (best)              | ✅            | ✅  |
| Nondeterminism / backtracking         | ✅ (natural) | ❌             | ✅ (if multi-shot)     | ❌            | ✅  |
| Async / await                         | ✅           | ✅ (native)    | ✅                     | ✅            | ✅  |
| Higher-order effects (local, bracket) | ⚠️ (hard)    | ✅ (natural)   | ✅                     | ⚠️            | ✅  |

---

## Further Reading: Essential Papers and Implementations

### Papers

- **"An Introduction to Algebraic Effects and Handlers"** (Pretnar, 2015) — The best tutorial-style introduction. Start here.
- **"Freer Monads, More Extensible Effects"** (Kiselyov & Ishii, 2015) — The freer monad approach that eliminates the Functor constraint.
- **"Type Directed Compilation of Row-typed Algebraic Effects"** (Leijen, 2017) — Koka's selective CPS and evidence passing.
- **"Generalized Evidence Passing for Effect Handlers"** (Xie, Brachthäuser, Hillerström, Schuster, Leijen, 2020) — The O(1) dispatch strategy.
- **"One-shot Algebraic Effects as Coroutines"** (Kawahara & Kameyama, 2020) — The coroutine embedding.
- **"Capability-passing Style for Zero-cost Effect Handlers"** (Schuster, Brachthäuser, Ostermann, ICFP 2020) — Effekt's approach.
- **"Effects, Capabilities, and Boxes"** (Brachthäuser et al., OOPSLA 2022) — Extends capability passing with scope safety.
- **"Efficient Compilation of Algebraic Effects"** (Pretnar et al., 2020) — Source-to-source optimization of effect handlers.
- **"Effects for Less"** (Alexis King) — The blog post / talk that motivated Hasura's `eff` and GHC's delimited continuations proposal.

### Implementations to study

| Implementation          | Language   | Approach                                            | Why study it                                                                                                    |
| ----------------------- | ---------- | --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `purescript-run`        | PureScript | Free monad + row polymorphism                       | Cleanest free monad implementation                                                                              |
| `freer-simple`          | Haskell    | Freer monad                                         | Simplest Haskell effect library                                                                                 |
| `effectful`             | Haskell    | ReaderT IO                                          | Best-performing Haskell library                                                                                 |
| `cleff`                 | Haskell    | ReaderT IO (lighter weight)                         | Simpler API than effectful, good source to read                                                                 |
| `polysemy`              | Haskell    | Free monad + higher-order weaving                   | Ambitious attempt at higher-order effects                                                                       |
| `fused-effects`         | Haskell    | Church-encoded free monad + weaving                 | Higher-order effects via weaving; good middle ground between free monads and ReaderT IO                         |
| `heftia`                | Haskell    | Hefty algebras                                      | Implements correct higher-order + algebraic effect semantics (based on Poulsen & van der Rest's hefty algebras) |
| `bluefin`               | Haskell    | Capability passing (no algebraic effects)           | Shows how far you can go without continuations                                                                  |
| Effekt (Scala library)  | Scala      | Multi-prompt delimited continuations + capabilities | Best library-level CPS implementation                                                                           |
| ZIO                     | Scala      | Fiber-based runtime + typed error channels          | Most widely deployed effect system in industry; not algebraic effects per se, but an influential alternative    |
| `effects.js`            | JavaScript | Generators as do-notation                           | Practical JS implementation with multi-shot via replay (re-executes from start; not true continuation cloning)  |
| Koka's `libhandler`     | C          | `setjmp`/`longjmp` + stack capture                  | How to implement effects in C                                                                                   |
| OCaml 5 `Effect` module | OCaml      | Native fibers                                       | Reference for runtime-level implementation                                                                      |
| Yelouafi's gist         | JavaScript | Generators as delimited continuations               | Excellent pedagogical implementation                                                                            |
