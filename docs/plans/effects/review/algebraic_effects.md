# Algebraic Effect Systems

A comprehensive overview of what algebraic effects are, what they enable,
how they relate to monad transformers and other approaches to effectful
programming, and the design space of practical implementations.

## 1. What Is an Algebraic Effect System?

An **algebraic effect system** is a programming-language facility (or a
library encoding of one) that lets a program describe _which_ side
effects it performs, separately from _how_ those effects are
interpreted. A program written against such a system has two
ingredients:

1. **Effect signatures.** A set of named operations with typed
   parameters and return types. For example, a `State Int` signature
   provides `get : () -> Int` and `put : Int -> ()`. The program is
   written in terms of these operations as if they were ordinary
   functions, but their meaning is not yet fixed.
2. **Handlers.** Programs (or values, or modules) that interpret
   operations into some effectful target. A handler for `State Int`
   might thread an integer parameter through the computation; another
   might log every read and write to an external system; another might
   enumerate all possible histories nondeterministically. The same
   effectful program can be reinterpreted by simply choosing a
   different handler.

The "algebraic" qualifier comes from universal algebra. A signature is a
set of operation symbols with arities; a model is an interpretation of
those symbols into some carrier set. The free model on a signature is
the set of syntax trees built from the operations, and any model
induces a unique homomorphism out of the free model. In programming
terms: a computation tree built from operations is the free model, and
a handler is a homomorphism into a chosen interpretation.

The pioneering theoretical paper is Plotkin and Power's "Algebraic
Operations and Generic Effects" (2003); the "handlers" extension is
due to Plotkin and Pretnar (2009), "Handlers of Algebraic Effects."

## 2. What Do Algebraic Effects Allow?

The headline benefit is **decoupled interpretation**: a program's
description of _what_ it does is independent of any particular _how_.
Concretely, this enables:

- **Mocking and testing.** Replace a `Network` handler with one that
  serves canned responses; replace `Time` with a clock you control.
  No code under test needs to be parameterized by a "test mode"; the
  effect system already provides the seam.
- **Multiple interpretations of the same code.** A program written
  against `Logger` can be run with a console handler in development, a
  structured-JSON handler in production, and a no-op handler in a
  benchmark, all without modifying the program.
- **Custom control flow.** Because handlers receive the _continuation_
  of the operation (i.e., what the program will do next, given the
  operation's result), they can implement nondeterminism (call the
  continuation many times), exceptions (don't call it), generators and
  coroutines (call it once, later), backtracking search, async/await,
  and so on, all from a single primitive: the typed delimited
  continuation that handlers receive.
- **Composability.** Handlers compose: you can layer a `Logger`
  handler over a `State` handler over a `Reader` handler, and the
  layers can be reordered (subject to subtle laws about commutativity
  with the continuation). This composability is structurally simpler
  than monad transformer stacks because it is not encoded as a tower
  of newtype wrappers.
- **Effect-tracked types.** A function's type lists which effects it
  may perform. `read : Path -> {File} String` is statically distinct
  from `pure : Int -> {} Int`. The compiler enforces that any call
  site has handlers (or further effect rows in scope) for every effect
  the callee may use.
- **Resource-safe abstractions.** Patterns like `with-resource`,
  transactions, and bracket-style cleanup are expressible directly as
  handlers, and they integrate with whatever other effects are in
  scope without manual lifting.

A useful slogan: _algebraic effects are to monads what async/await is
to callbacks._ They give you direct-style code that is nonetheless
statically tracked and reinterpretable.

## 3. The Core Vocabulary

Across implementations the terminology varies, but the concepts are
shared:

- **Operation (or "effect operation").** A typed primitive request the
  program can make. Often written as a constructor of a GADT-like
  signature type.
- **Signature (or "effect").** A collection of related operations.
  `State s`, `Exception e`, `NonDet`, `IO`.
- **Effect row (or "effect set").** A list or set of signatures that
  appear in a computation's type, e.g., `Eff [State Int, Logger] a`.
- **Handler.** A value that consumes a computation in some effect row
  and produces a computation in a smaller effect row (the handled
  effect has been removed) plus a transformed return type.
- **Continuation.** Inside a handler clause, the suspended rest of the
  program, exposed as a callable function `k : x -> Result`. Whether
  this continuation is single-shot, multi-shot, or escape-only depends
  on the system.
- **Discharge / interpret / run.** The act of applying handlers until
  the effect row is empty (or contains only `IO`), yielding a runnable
  value.

## 4. Relation to Monad Transformers

Monad transformers (MTL-style in Haskell, `purescript-transformers`,
etc.) and algebraic effects address the same problem: how to combine
multiple effects in one program. They differ in encoding and in
ergonomics.

### 4.1 What MTL does

In MTL, each effect is a typeclass parameterised by the monad:
`MonadState s m`, `MonadReader r m`, `MonadError e m`. Each effect
also has at least one concrete carrier (e.g., `StateT s m`) that
implements its class. A stack like `ReaderT r (StateT s (ExceptT e
IO))` is a tower of newtypes; class instances `lift` automatically
across the tower so that, e.g., a `MonadReader` operation works even
when `ReaderT` is buried under three other transformers.

### 4.2 Where MTL and algebraic effects coincide

For a fixed handler choice, an algebraic effect system _is_
isomorphic to a particular monad transformer stack. Running `State Int`
with the standard state-passing handler gives you exactly `StateT Int`.
Running `Exception e` with the standard handler gives you `ExceptT e`.
The free monad over the union of signatures, quotiented by the laws
that handlers must respect, is the monad you would have built by hand.

So: algebraic effects do not let you do anything that monad transformers
fundamentally cannot. They let you do those things with different
ergonomics and different performance profiles.

### 4.3 Where they differ

**Reinterpretation.** With MTL, the meaning of `get` and `put` is fixed
by which `StateT` you stacked. To swap interpretations (e.g., to log
every state mutation, or to use a database-backed cell), you change
the carrier and re-derive the stack, often touching every layer.
With algebraic effects, you swap a handler and the program is
unchanged. The program's type may not even change, since the effect
row is the same; only the handler that discharges it differs.

**O(n^2) instances.** MTL is famous for the "n^2 instances" problem.
For `n` effects you need roughly `n*n` typeclass instances to lift
each effect through each transformer. Library authors must anticipate
combinations. Adding a new transformer requires writing instances for
every existing effect class. Algebraic effect systems sidestep this by
indexing into an effect row directly: a `State Int` operation finds
its handler by its position in the row, not by chains of `lift`.

**Order matters, sometimes invisibly.** In MTL, the order of
transformers is semantically meaningful. `StateT s (ExceptT e Identity)`
keeps state changes when an exception is thrown? No: it discards them.
`ExceptT e (StateT s Identity)` keeps them. This is correct but
surprising. In algebraic effect systems, handler order is also
significant for the same reasons (handlers compose noncommutatively),
but the order is explicit at the call site of the runner, not buried
in a type alias far away from the program logic.

**Continuation flexibility.** Monad transformers give you the
expressive power of whatever the bottom monad allows. Algebraic
effects (especially those with first-class delimited continuations)
let _handlers themselves_ introduce continuation-shaped control:
multi-shot resumption, async resumption, backtracking, etc., without
the bottom monad needing to support it. This is the most genuine
expressiveness gap, and it shows up in practice when implementing
nondeterminism, async, generators, or backtracking search.

**Scoped (higher-order) operations.** MTL handles `local`, `catch`,
`mask`, and similar operations that take _computations_ as arguments
naturally, because the typeclass method takes an `m a`. Algebraic
effects, in their pure form, cannot type these operations directly:
the operation argument is required to be a value, not a continuation
of the same effect row. Workarounds are the subject of active
research; see Section 9.

### 4.4 Summary table

| Property                        | MTL / transformers          | Algebraic effects             |
| ------------------------------- | --------------------------- | ----------------------------- |
| Effect tracked in types         | Yes (typeclasses)           | Yes (effect row)              |
| Reinterpret without recompiling | Hard                        | Easy                          |
| n^2 instance burden             | Yes                         | No                            |
| Order of effects matters        | Yes                         | Yes                           |
| Multi-shot continuations        | Only via `ContT` or similar | Often native                  |
| Higher-order ops (`local` etc.) | Native                      | Awkward; needs scoped effects |
| Performance (idiomatic)         | Predictable, can be fused   | Varies by implementation      |
| Inference burden                | Modest                      | Often heavier                 |

## 5. Relation to Other Approaches

### 5.1 Free monads and freer monads

A **free monad** over a functor `F` is the syntax tree of programs
built from `F`-shaped operations. Interpreting a free monad means
folding it into a target monad. This is exactly the algebraic-effect
recipe, restricted to a single signature.

The **freer monad** (Kiselyov and Ishii, "Freer Monads, More
Extensible Effects", 2015) generalizes free monads so that the
signature does not need to be a `Functor` instance: it can be any
GADT-indexed family. This makes encoding effect operations easier and
more general, because operations naturally have GADT-shaped types
(input parameters and return type indexed differently).

Multi-effect free-monad libraries (`extensible-effects`, `polysemy`,
`freer-simple`, PureScript's `purescript-run`) use a freer monad over
an _open union_ of effect signatures, plus type-level row machinery to
add and remove effects from the row. The payoff is exactly the
algebraic-effect interface: handlers are folds, programs are syntax,
reinterpretation is free.

The cost is a constant overhead per operation: every operation
allocates a constructor, and every `>>=` builds a tree node. Naive
free-monad interpretation is O(n^2) in computation length because of
left-associated binds; the standard fix is the "reflection without
remorse" trick (codensity / type-aligned sequences) which makes it
O(n).

### 5.2 Tagless final / "MTL-style" without transformers

The **tagless final** style encodes effect signatures as typeclasses
(or PureScript records, or first-class modules) but keeps the carrier
abstract: `forall m. (MonadState s m, MonadReader r m) => m a`. There
is no transformer stack; instead, callers choose a single concrete
monad that implements all the constraints.

This is closer to MTL than to algebraic effects in spirit, but it
shares with algebraic effects the ability to choose the carrier at the
top level. Reinterpretation is possible (write a different
`MonadState` instance for a different carrier), but the carrier-choice
happens once globally, not handler-by-handler.

`fused-effects` and `in-other-words` are libraries that combine the
tagless-final encoding with handler-style interpretation: each effect
has both a class (for use in programs) and a carrier (for handling),
and a "carrier transformer" composes carriers without the standard
n^2 instance burden.

### 5.3 Capabilities and effect systems

In **capability-based** designs (e.g., Effekt, Scala 3 with capture
checking), an effect is a _capability_ held by a value, and the type
system tracks which capabilities a value can use. Capabilities are
typically **second-class**: they cannot be stored in long-lived data,
returned from functions, etc., which prevents capability leaks and
makes some performance optimizations easier (capabilities can compile
to direct function calls). The tradeoff is reduced expressiveness;
some programs that algebraic-effect systems accept are rejected by
capability systems.

### 5.4 Delimited continuations

The most powerful algebraic effects are direct sugar over **delimited
continuations**: `shift` and `reset`, or `prompt` and `control`. A
handler is a `reset`, an operation invocation is a `shift` that
captures up to the nearest enclosing handler, and the captured
continuation is what handler clauses get as `k`. Languages with native
delimited continuations (Koka, OCaml 5, Multicore OCaml, Racket) can
implement algebraic effects very efficiently, because the runtime
already supports the underlying mechanism.

## 6. What an Algebraic Effect System Should Be Able To Do

A "complete" algebraic effect system, judged against the published
research and against what practitioners want, should support:

### 6.1 Core capabilities

1. **First-class effect signatures.** Users define their own
   signatures. The system is not limited to a fixed set of built-in
   effects.
2. **Open effect rows.** A computation's effect row is an extensible
   set, not a closed enumeration. New effects can be added; handlers
   can subtract specific effects.
3. **Handlers as ordinary values.** Handlers are first-class: they can
   be passed as arguments, stored in data, parameterized over types,
   and composed.
4. **Access to continuations.** Handler clauses receive the
   continuation of the operation as a callable value. The system
   should make clear whether continuations are single-shot, multi-shot,
   or escape-only.
5. **Static effect tracking.** Effects in a function's type are
   checked at compile time. Calling a function with an unhandled effect
   is a type error.
6. **Effect polymorphism.** Functions that take other effectful
   functions (e.g., `map`, `traverse`, `bracket`) should be
   parameterizable over the callee's effect row, ideally without
   syntactic noise.
7. **Subsumption.** A computation in row `R1` should be usable where
   row `R2 >= R1` is expected. This is what "open" rows enable.

### 6.2 Higher-order capabilities

8. **Scoped operations.** Operations like `local : (r -> r) -> Eff R a
-> Eff R a`, `catch : Eff R a -> (e -> Eff R a) -> Eff R a`, and
   `mask : Eff R a -> Eff R a` take _computations_ as arguments. Plain
   algebraic effects cannot type these as ordinary operations because
   the operation's argument carries effects. A complete system needs a
   story for these. Approaches include scoped-effect calculi (e.g.,
   Hefty algebras / heftia), explicit "operation parameter is a thunk"
   conventions, and second-class capabilities.
9. **Resource safety.** `bracket`, `finally`, and other cleanup
   patterns must compose with arbitrary handlers, including handlers
   that drop or duplicate continuations (e.g., nondeterminism). This
   is genuinely subtle: nondeterministic backtracking past a `bracket`
   is a known semantic minefield.
10. **Async and concurrency.** Async/await, structured concurrency,
    cancellation, and timeouts should be expressible as handlers (or
    at least as effects with reasonable handlers).

### 6.3 Pragmatic capabilities

11. **Performance.** The cost per operation should be small and
    predictable. The cost of handler stacking should not be quadratic
    in the number of handlers. Common simplifications (state passing,
    reader environments) should be optimizable to no allocation.
12. **Good error messages.** When effect rows do not unify, the
    compiler should report which effect was unhandled, not just
    "could not match `Eff [A, B] a` with `Eff [A, B, C] a`".
13. **Interoperation with the host language.** The system should not
    require rewriting the world. It should integrate with existing
    `IO`, futures, async runtimes, and FFI without arcane glue.
14. **Debuggability.** Stack traces inside handlers should be
    intelligible. This is hard when handlers capture continuations,
    but partial solutions exist (Koka invests heavily here).
15. **Handler combinators.** Lifting, masking, interposing, and
    composing handlers should all be expressible without manual
    plumbing.

## 7. How Implementations Differ

The `effects/` directory next to this document contains thirteen
projects, which between them illustrate every major implementation
strategy. The differences matter: they determine performance,
expressiveness, and the kinds of programs that compile cleanly.

### 7.1 Free-monad encodings

Examples: `polysemy`, `freer-simple`, `extensible-effects`, PureScript's
`purescript-run`.

The computation is a tree of operation constructors plus a `Pure`
leaf. The bind operation extends the tree. Handlers fold the tree.
Effect rows are open unions encoded with type-level lists.

Pros: pure, no runtime extensions needed, portable across compilers.
Cons: per-operation allocation, sometimes deep type-inference work,
left-bind quadratic blowup unless mitigated. `polysemy` mitigates with
GHC plugin assistance for inference; `freer-simple` keeps the
implementation small; `purescript-run` keeps the encoding readable but
relies on `purescript-free` and a type-aligned queue for the `>>=`
fix.

### 7.2 Evidence-passing translations

Examples: `EvEff`, `MpEff`, the Koka backend.

Daan Leijen's "Evidence Passing Semantics" line of work compiles
algebraic-effect programs into ordinary functions that take an
evidence vector at runtime. Each operation looks up its handler in the
vector by static index and jumps to it; the handler captures the
delimited continuation up to its installation point. This is the
fastest known pure-library encoding for algebraic effects, and it is
what makes Koka's effects competitive with hand-written code.

Pros: very fast; supports multi-shot continuations cleanly; integrates
with native delimited continuations when available.
Cons: complex implementation; understanding error messages requires
understanding the translation; without compiler support the encoding
is heavy in the type system.

### 7.3 Tagless-final / fused

Examples: `fused-effects`, `in-other-words`.

Each effect has a class (a la MTL) and a _carrier_ type that
implements the class. Carriers compose by stacking, but the libraries
provide combinators to fuse the stack so that operations compile to
direct method calls without intermediate allocation. The effect row is
implicit in the typeclass constraints.

Pros: very fast (often as fast as MTL), small constant factors,
familiar to MTL users.
Cons: less reinterpretable at runtime (the carrier is fixed at
compile-time); inference can be tricky; still suffers some n^2
instance pain, mitigated by clever defaulting.

### 7.4 Capability / second-class

Examples: `effekt` (Effekt language), Scala 3 capture checking.

An effect is a capability passed implicitly. Capabilities are
second-class, so they cannot escape their introduction site. This
yields strong static guarantees and good performance, at the cost of
expressiveness.

Pros: easy to compile efficiently; excellent for resource safety.
Cons: cannot store handlers in data; some patterns require workarounds.

### 7.5 Higher-order / scoped effects

Examples: `heftia`, `eff` (effectful), `reffect`.

These extend the algebraic-effect model to handle operations whose
arguments themselves carry effects. `heftia` (Higashikawa) implements
"hefty algebras" in Haskell; `eff` and `reffect` use other techniques
(monad-transformer-like delegation, or closed type-level handling) to
cope with `local` and `catch`.

This is the active research frontier; the design space is not yet
settled.

### 7.6 Macro-DSL approaches in Rust

Examples: `effing-mad`, `fx-rs`, `corophage`.

In Rust, the lack of GHC-style higher-rank polymorphism and the
borrow-checker's interaction with continuations make idiomatic
algebraic-effect encodings hard. These libraries use procedural
macros, generators, or coroutines (`switch-resume` is a related
primitive) to provide effect-like ergonomics within Rust's
constraints. The ergonomics gap with Haskell/Koka is real but
narrowing.

## 8. Concrete Examples

### 8.1 The same program, four ways

Suppose we have a program that increments a counter, logs the new
value, and may throw on overflow.

**Algebraic effect (Koka-like pseudocode):**

```
fun bump() : <state<int>, log, exn> int
  val n = get()
  if n >= 100 then throw("overflow")
  put(n + 1)
  info("counter is now " ++ show(n + 1))
  return n + 1
```

The effect row `<state<int>, log, exn>` is part of the type. To run,
choose handlers:

```
fun main() : io int
  with handler-state(0)
  with handler-log-console
  with handler-exn-default(-1)
  bump()
```

**Free-monad library (Haskell, polysemy):**

```haskell
bump :: Members '[State Int, Log, Error String] r => Sem r Int
bump = do
  n <- get
  when (n >= 100) (throw "overflow")
  put (n + 1)
  info ("counter is now " ++ show (n + 1))
  return (n + 1)

main :: IO ()
main = do
  result <- runM
          . runError @String
          . runLogStdout
          . evalState (0 :: Int)
          $ bump
  print result
```

**MTL transformer stack:**

```haskell
bump :: (MonadState Int m, MonadLog m, MonadError String m) => m Int
bump = do
  n <- get
  when (n >= 100) (throwError "overflow")
  put (n + 1)
  logInfo ("counter is now " ++ show (n + 1))
  return (n + 1)

main :: IO ()
main = do
  result <- flip evalStateT 0
          . runExceptT
          . runStdoutLogger
          $ bump
  print result
```

**Tagless final (no transformers):**

```haskell
class Monad m => Counter m where
  bumpOnce :: m Int

instance Counter MyApp where
  bumpOnce = ...
```

The bodies are nearly identical. The differences are:

- Reinterpretation: the polysemy version can replace `runLogStdout`
  with `runLogToStream` without touching `bump`. The MTL version needs
  a different concrete monad (different `Logger` carrier) and
  potentially a refactor of the runner.
- Adding a new effect: in polysemy, append it to the row; in MTL,
  thread the new transformer through the stack and ensure all classes
  lift through it.
- Performance: well-tuned MTL is faster than naive polysemy; Koka and
  EvEff close the gap; fused-effects is competitive with MTL.

### 8.2 Multi-shot nondeterminism

This is the canonical example where algebraic effects shine.

```
fun choose(xs : list<a>) : <nondet> a
  perform Choose(xs)

fun pyth() : <nondet> (int, int, int)
  val a = choose([1, ..., 20])
  val b = choose([a, ..., 20])
  val c = choose([b, ..., 20])
  if a*a + b*b == c*c then return (a, b, c)
  else perform Fail()

handler nondet-list
  return x  -> [x]
  Choose(xs) -> flatten(xs.map(fn(x) resume(x)))
  Fail()    -> []
```

The handler clause for `Choose` calls `resume(x)` once per choice and
flattens. This works because `resume` is multi-shot: the same
suspended continuation is invoked many times with different values.
In MTL, this requires `ListT` or `LogicT`; either way it is a special
monad. With algebraic effects the same operational pattern works for
_any_ effect that needs to fork a computation into alternatives.

## 9. Open Problems and Active Research

- **Scoped/higher-order operations.** As noted, the cleanest known
  treatment is "Hefty Algebras" (Bach Poulsen and Van Der Rest, POPL
  2023). `heftia` is one Haskell encoding. The tradeoffs against
  alternatives (`eff`, `polysemy-plugin`, `reffect`) are ongoing
  research.
- **Effect rows and inference.** Open effect rows interact
  uncomfortably with type inference: there is often ambiguity about
  which "tail" of the row a particular operation belongs to. Solutions
  include row polymorphism with explicit row variables, set-based
  rows, and compiler plugins.
- **Resource safety with multi-shot continuations.** When a handler
  clones a continuation that has acquired a resource, who owns the
  resource? Answers range from "resources cannot escape a single-shot
  scope" (Effekt) to "the user must be careful" (most libraries).
- **Interaction with linear or affine types.** Handlers want to call
  continuations zero, one, or many times, which is exactly the domain
  of multiplicities. Linear Haskell, Idris 2, and various dependent
  systems are exploring this.
- **Native runtime support.** OCaml 5 ships effect handlers in the
  runtime; WasmFX is bringing them to WebAssembly; languages like
  Koka and Eff have them as primitives. The library/runtime divide
  determines what is possible.
- **Algebraic effects for parallelism.** The classical theory is
  sequential. Parallel algebraic effects are the subject of current
  work (e.g., on parallel handlers and effect-row commutativity).

## 10. Reading and References

The papers that best repay reading, in roughly the order I would
recommend them:

1. Plotkin and Power, "Algebraic Operations and Generic Effects"
   (2003). The starting point.
2. Plotkin and Pretnar, "Handlers of Algebraic Effects" (2009). Adds
   handlers and gives them semantics.
3. Kammar, Lindley, and Oury, "Handlers in Action" (2013). Practical
   handler programming, with examples.
4. Kiselyov and Ishii, "Freer Monads, More Extensible Effects" (2015).
   The library-encoded version that motivated the freer-simple line.
5. Leijen, "Type Directed Compilation of Row-Typed Algebraic Effects"
   (POPL 2017). Koka's compilation strategy.
6. Xie, Brachthauser, Hillerstrom, Schuster, Leijen, "Effect Handlers,
   Evidently" (ICFP 2020). Foundations of evidence-passing.
7. Wu, Schrijvers, Hinze, "Effect Handlers in Scope" (Haskell 2014).
   The original scoped-effect paper.
8. Bach Poulsen and Van Der Rest, "Hefty Algebras: Modular Elaboration
   of Higher-Order Algebraic Effects" (POPL 2023). The current
   front-runner for higher-order effects.
9. Brachthauser, Schuster, Ostermann, "Effects as Capabilities: Effect
   Handlers and Lightweight Effect Polymorphism" (OOPSLA 2020). The
   Effekt language design.
10. Sivaramakrishnan et al., "Retrofitting Effect Handlers onto OCaml"
    (PLDI 2021). What it took to add handlers to a mature compiler.

Local source trees worth grepping when something in this document is
unclear:

- `koka/` for a language with native handlers and evidence-passing
  compilation.
- `polysemy/` and `freer-simple/` for free-monad encodings in
  Haskell.
- `EvEff/` and `MpEff/` for Leijen's evidence-passing libraries.
- `fused-effects/` and `in-other-words/` for tagless-final / fused
  carriers.
- `effekt/` for capability-based second-class effects.
- `heftia/` and `reffect/` for higher-order / scoped effects.
- `effing-mad/`, `fx-rs/`, `corophage/` for Rust attempts.

## 11. Glossary

- **Algebraic operation.** A typed primitive request that can appear
  in a program; corresponds to a constructor of a signature.
- **Carrier.** In tagless-final / fused style, the concrete monad type
  that implements an effect class.
- **Continuation.** The remainder of a program, suspended and
  reified as a value.
- **Delimited continuation.** A continuation captured up to a specific
  enclosing prompt or handler installation, not the whole rest of the
  program.
- **Effect row.** A type-level set or list of effect signatures
  appearing in a computation's type.
- **Handler.** A homomorphism from the free model on a signature into
  some target; in code, a value that consumes operations of one or
  more effects and discharges them.
- **Hefty algebra.** An extension of algebraic theories that supports
  higher-order operations whose arguments are themselves
  computations.
- **Open union.** A type-level encoding of a sum type whose summands
  can be added at compile time.
- **Multi-shot.** A continuation that may be invoked more than once.
  Required for nondeterminism, backtracking, generators with state
  reset, etc.
- **Scoped (or higher-order) operation.** An operation whose argument
  is a computation in the same effect row, e.g., `local`, `catch`,
  `mask`.
- **Signature.** A collection of operations defining one effect.
- **Subsumption.** The rule that lets a computation in row `R1` be
  used where row `R2 >= R1` is expected.

## 12. A Practical Decision Framework

If you are choosing between approaches for a real project, the
useful axes of comparison are:

1. **Do you need to swap interpretations at runtime?** If yes, lean
   toward free-monad encodings or evidence-passing. If no, MTL or
   tagless final may be simpler.
2. **Do you need multi-shot continuations?** If yes, you need
   algebraic effects (free-monad, evidence-passing, or native).
   Otherwise any approach works.
3. **How important is performance?** From fastest to slowest, on
   typical microbenchmarks: native handlers (Koka, OCaml 5) ~
   evidence-passing ~ fused-effects ~ MTL > polysemy > naive freer.
   The gap shrinks as compilers add support.
4. **Do you have higher-order operations?** If you need many `local`,
   `catch`, `mask` style operations, MTL or hefty-algebra-based
   systems are easier than plain algebraic effects.
5. **What is your host language?** In Haskell, you have many choices.
   In OCaml 5, use native handlers. In Koka or Eff, the language has
   them. In Rust, you are choosing between macro DSLs and emerging
   coroutine support; the ergonomics are not yet at parity with
   Haskell or Koka.
6. **How important is debuggability?** Native runtime support helps
   most here; library encodings produce stack traces that reflect the
   encoding, not the source.

There is no single "best" system. The design space is a real tradeoff
space, and the right answer depends on what you are building, in
which language, and against which constraints.
