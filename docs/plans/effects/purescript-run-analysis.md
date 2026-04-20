# Technical Analysis: PureScript `purescript-run` Library

## Overview

`purescript-run` implements extensible algebraic effects using **Free monad over row-polymorphic extensible variants** (Approach 1 from effects.md). It is the cleanest existing implementation of this approach, leveraging PureScript's native row polymorphism for ergonomic, type-safe effect composition.

Source: <https://github.com/natefaubion/purescript-run>

## Core Architecture

### The Run Type

```purescript
newtype Run r a = Run (Free (VariantF r) a)
```

`Run` is a newtype over `Free` parameterized by `VariantF r`, where:

- `r` is a **row type** representing an extensible set of effect functors.
- `a` is the return type.
- `Free` provides the monadic structure (bind, sequencing).
- `VariantF r` is the extensible sum type carrying effect operations.

`Run` derives `Functor`, `Apply`, `Applicative`, `Bind`, `Monad` from the underlying `Free`.

### VariantF: The Extensible Sum

From `purescript-variant`, `VariantF` is a row-indexed sum of functors:

```purescript
-- Conceptual structure:
data VariantF r a  -- where r is a row of functors

-- Injection:
inj :: forall sym f r1 r2 a. Row.Cons sym f r1 r2 => IsSymbol sym
    => Proxy sym -> f a -> VariantF r2 a

-- Projection:
on :: forall sym f r_rest r a b. Row.Cons sym f r_rest r => IsSymbol sym
   => Proxy sym -> (f a -> b) -> (VariantF r_rest a -> b) -> VariantF r a -> b
```

Each effect functor is labeled with a string symbol (e.g., `"state"`, `"reader"`). `inj` injects a functor value into the variant at a given label. `on` pattern-matches on a label, peeling it off and either handling it or delegating to a handler for the remaining row.

### Row Polymorphism

PureScript's row types enable open-world effect sets:

```purescript
-- Type aliases for effect rows
type STATE s r = (state :: State s | r)
type READER e r = (reader :: Reader e | r)
type EXCEPT e r = (except :: Except e | r)

-- Programs are polymorphic in the "rest" of the row
myProgram :: forall r. Run (STATE Int + READER String + r) Unit
```

Key row constraints from `Prim.Row`:

- `Row.Cons sym f r1 r2`: Row `r2` equals `sym :: f` prepended to `r1`.
- `Row.Union r1 r2 r3`: Row `r3` is the union of `r1` and `r2`.

These constraints are resolved at compile time with no runtime cost.

## Effect Injection and Projection

### lift: Injecting Effects

```purescript
lift :: forall sym r1 r2 f a
      . Row.Cons sym f r1 r2 => IsSymbol sym => Functor f
     => Proxy sym -> f a -> Run r2 a
lift p = Run <<< liftF <<< inj p
```

Flow: `f a` -> `inj` into `VariantF r2 a` -> `liftF` into `Free (VariantF r2) a` -> wrap in `Run`.

### send: Re-injecting Unhandled Effects

```purescript
send :: forall a r. VariantF r a -> Run r a
send = Run <<< liftF
```

Used in handlers to pass through effects that the current handler does not match.

### peel and resume: Inspecting the Next Step

```purescript
peel :: forall a r. Run r a -> Either (VariantF r (Run r a)) a
peel = resume Left Right

resume :: forall a b r
        . (VariantF r (Run r a) -> b) -> (a -> b) -> Run r a -> b
```

`peel` / `resume` inspect one layer of the Free structure:

- `Right a`: The computation is done, producing value `a`.
- `Left v`: The next effect operation `v`, where the functor parameter is the continuation `Run r a` (the rest of the computation).

## Interpretation Functions

### interpret / interpretRec: Natural Transformation

```purescript
interpret :: forall m a r. Monad m
          => (VariantF r ~> m) -> Run r a -> m a

interpretRec :: forall m a r. MonadRec m
             => (VariantF r ~> m) -> Run r a -> m a
```

These interpret the entire effect row at once via a natural transformation. `interpretRec` uses `MonadRec` for guaranteed stack safety.

### run / runRec: Full Control

```purescript
run :: forall m a r. Monad m
    => (VariantF r (Run r a) -> m (Run r a)) -> Run r a -> m a
run k = loop where
  loop = resume (\a -> loop =<< k a) pure

runRec :: forall m a r. MonadRec m
       => (VariantF r (Run r a) -> m (Run r a)) -> Run r a -> m a
```

Unlike `interpret`, the handler receives both the effect and the continuation (`Run r a`), allowing it to inspect or modify the rest of the computation.

### runPure: Pure Effect Elimination

```purescript
runPure :: forall r1 r2 a
         . (VariantF r1 (Run r1 a) -> Step (Run r1 a) (VariantF r2 (Run r1 a)))
        -> Run r1 a -> Run r2 a
```

The handler returns `Step`:

- `Loop r'`: Continue interpreting `r'` (handled the effect).
- `Done v`: Emit `v` into the output effect type `r2` (forwarded the effect).

This is the key combinator for building handlers that peel off one effect at a time.

### runAccumPure: Pure with State

```purescript
runAccumPure :: forall r1 r2 a b s
              . (s -> VariantF r1 (Run r1 a)
                    -> Step (Tuple s (Run r1 a)) (VariantF r2 (Run r1 a)))
             -> (s -> a -> b) -> s -> Run r1 a -> Run r2 b
```

Threads an accumulator `s` through each step. This is how `State`, `Writer`, and similar effects are interpreted: the handler carries state and produces the final result by combining the accumulator with the return value.

### runCont / runAccumCont: Continuation-Passing Style

```purescript
runCont :: forall m a b r
         . (VariantF r (m b) -> m b) -> (a -> m b) -> Run r a -> m b

runAccumCont :: forall m r s a b
              . (s -> VariantF r (s -> m b) -> m b)
             -> (s -> a -> m b) -> s -> Run r a -> m b
```

CPS variants where the handler receives the continuation already applied (as `m b`), avoiding the need for a `Monad` constraint.

### extract: Eliminate Empty Row

```purescript
extract :: forall a. Run () a -> a
extract = unwrap >>> runFree \_ -> unsafeCrashWith "Run: the impossible happened"
```

When all effects have been handled (row is `()`), extract the pure value. The crash case is unreachable.

### expand: Widen Effect Row

```purescript
expand :: forall r1 r2 rx a. Row.Union r1 rx r2 => Run r1 a -> Run r2 a
expand = unsafeCoerce
```

Widens a narrower effect set to a larger one. Safe because the runtime representation is identical; only the phantom row type changes.

## Built-in Effects

### State

```purescript
data State s a = State (s -> s) (s -> a)
derive instance functorState :: Functor (State s)

type STATE s r = (state :: State s | r)
```

The `State` functor bundles a state transformation `s -> s` with a continuation `s -> a`.

**Operations:**

```purescript
get    :: forall s r. Run (STATE s + r) s
put    :: forall s r. s -> Run (STATE s + r) Unit
modify :: forall s r. (s -> s) -> Run (STATE s + r) Unit
gets   :: forall s r. (s -> t) -> Run (STATE s + r) t
```

**Handler (runState):**

```purescript
runState :: forall s r a. s -> Run (STATE s + r) a -> Run r (Tuple s a)
```

Implementation (via `runAccumPure`):

1. Peels the next instruction.
2. If `State t k`: applies transformation `t s` to get `s'`, calls continuation `k s'`.
3. If unrelated effect: forwards via `Done`.
4. Returns `Tuple finalState value`.

**Multi-label support:** `runStateAt` / `getAt` / `putAt` accept a `Proxy sym` to distinguish multiple independent State effects in the same row.

### Reader

```purescript
newtype Reader e a = Reader (e -> a)
derive newtype instance functorReader :: Functor (Reader e)

type READER e r = (reader :: Reader e | r)
```

**Operations:**

```purescript
ask   :: forall e r. Run (READER e + r) e
asks  :: forall e r a. (e -> a) -> Run (READER e + r) a
local :: forall e a r. (e -> e) -> Run (READER e + r) a -> Run (READER e + r) a
```

**Handler:** When encountering `Reader k`, applies `k` to the environment value `e` and continues.

### Writer

```purescript
data Writer w a = Writer w a
derive instance functorWriter :: Functor (Writer w)

type WRITER w r = (writer :: Writer w | r)
```

**Operations:**

```purescript
tell   :: forall w r. w -> Run (WRITER w + r) Unit
censor :: forall w a r. (w -> w) -> Run (WRITER w + r) a -> Run (WRITER w + r) a
```

**Handler (foldWriter / runWriter):**

```purescript
foldWriter :: forall w b a r. (b -> w -> b) -> b -> Run (WRITER w + r) a -> Run r (Tuple b a)
runWriter  :: forall w a r. Monoid w => Run (WRITER w + r) a -> Run r (Tuple w a)
```

Threads an accumulator, folding each `Writer w _` into the accumulated value.

### Except

```purescript
newtype Except e a = Except e
derive instance functorExcept :: Functor (Except e)

type EXCEPT e r = (except :: Except e | r)
```

**Operations:**

```purescript
throw   :: forall e a r. e -> Run (EXCEPT e + r) a
catch   :: forall e a r. (e -> Run r a) -> Run (EXCEPT e + r) a -> Run r a
rethrow :: forall e a r. Either e a -> Run (EXCEPT e + r) a
note    :: forall e a r. e -> Maybe a -> Run (EXCEPT e + r) a
```

**Handler:** When encountering `Except e`, short-circuits with `Left e`. Otherwise returns `Right a`.

### Choose (NonDeterminism)

```purescript
data Choose a = Empty | Alt (Boolean -> a)
derive instance functorChoose :: Functor Choose

type CHOOSE r = (choose :: Choose | r)
```

**Operations:**

```purescript
cempty :: forall r a. Run (CHOOSE + r) a
calt   :: forall r a. Run (CHOOSE + r) a -> Run (CHOOSE + r) a -> Run (CHOOSE + r) a
```

**Handler:**

```purescript
runChoose :: forall f a r. Alternative f => Run (CHOOSE + r) a -> Run r (f a)
```

On `Empty`, returns `empty`. On `Alt k`, recursively interprets both `k true` and `k false`, combining with `<|>`.

## Handler Composition Pattern

Effects are eliminated one at a time from the outside in:

```purescript
program             -- Run (STATE Int + EXCEPT String + EFFECT + ()) Unit
  # catch handler   -- Run (STATE Int + EFFECT + ()) Unit
  # runState 0      -- Run (EFFECT + ()) (Tuple Int Unit)
  # runBaseEffect   -- Effect (Tuple Int Unit)
```

Each handler removes its effect from the row and returns a `Run` with the remaining effects. The final handler (`runBaseEffect` or `extract`) requires an empty or base-only row.

The `on` / `send` pattern builds a single-step interpreter:

```purescript
myInterpreter :: VariantF (MY_EFFECT + r) ~> Run r
myInterpreter = on _myEffect handleMyEffect send
```

`on` matches the target effect; `send` forwards everything else.

## Stack Safety

PureScript compiles to JavaScript, which has no TCO. Three strategies are used:

1. **`MonadRec` + `runRec` / `interpretRec`**: Uses `tailRecM` internally, converting recursive interpretation into a loop.
2. **`Step` + `runPure` / `runAccumPure`**: Handler returns `Loop` or `Done`, which the interpreter loops over.
3. **`Run`'s own `MonadRec` instance**: Allows stack-safe recursion within `Run` programs.

## PureScript-Specific Features

| Feature                              | Where Used                               | Rust Equivalent                            |
| ------------------------------------ | ---------------------------------------- | ------------------------------------------ |
| Row polymorphism                     | Effect set extensibility                 | No direct equivalent; see feasibility doc  |
| `VariantF` (row-indexed sum)         | Open union of effect functors            | No direct equivalent; requires encoding    |
| Native HKTs                          | `Functor f`, `Monad m`, `~>`             | Brand-based HKT encoding (existing in lib) |
| `Row.Cons` / `Row.Union` constraints | Type-level row manipulation              | Trait bounds on HLists/tuples              |
| `unsafeCoerce` (4 uses)              | `expand`, `coerceM`, `toRows`/`fromRows` | `transmute` or pointer casts               |
| Newtype deriving                     | Zero-cost `Run` wrapper                  | `#[repr(transparent)]`                     |
| `IsSymbol` + `Proxy`                 | Label-indexed effect access              | Phantom types + const generics             |
| `TypeEquals` witness                 | Instance resolution for effect rows      | Trait-based type equality                  |

## Key Dependencies

| Package                    | Purpose                                     |
| -------------------------- | ------------------------------------------- |
| `purescript-free`          | `Free` monad (core structure)               |
| `purescript-variant`       | `VariantF` (extensible functorial variants) |
| `purescript-tailrec`       | `MonadRec`, `Step` (stack safety)           |
| `purescript-type-equality` | `TypeEquals` witness                        |

## Design Observations for Rust Port

### What Translates Well

1. **Free monad structure**: The library already has `Free<F, A>` in `fp-library/src/types/free.rs`.
2. **Effect functors as data types**: `State s a`, `Reader e a`, etc. are plain Rust enums.
3. **Handler pattern (peel/match/send)**: Iterative interpretation loop maps naturally to Rust.
4. **Accumulator-based handlers**: `runAccumPure` is just a fold, straightforward in Rust.

### What Needs New Machinery

1. **Extensible variants (VariantF)**: The library has no coproduct/variant type. This is the critical missing piece.
2. **Row polymorphism**: PureScript's `(state :: State s | r)` syntax has no Rust equivalent. Must be approximated via type-level lists, tuples, or macros.
3. **`expand` (row widening)**: Requires proving that one effect set is a subset of another at the type level.

### What Is Already Partially Available

1. **HKT encoding**: Brand-based system exists but the `Free` type has lifetime constraints (`'static` required for `Box<dyn Any>` continuations) that may conflict.
2. **Functor/Monad hierarchy**: Complete trait hierarchy exists.
3. **Coyoneda**: Available as `fp-library/src/types/coyoneda.rs`, could help with functor requirements if using the standard (non-freer) encoding.
