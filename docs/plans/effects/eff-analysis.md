# Technical Analysis: Hasura `eff` Library

## Overview

`eff` is a high-performance extensible effect system for Haskell built on **delimited continuations** (Approach 3 from effects.md). Unlike free-monad-based approaches, `eff` directly manipulates GHC's call stack using RTS primops (`prompt#`, `control0#`), achieving near-zero overhead for effect dispatch.

Source: <https://github.com/hasura/eff>

## Core Architecture

### The Eff Virtual Machine

The library is best understood as an "embedded virtual machine" managing two pieces of state:

1. **Metacontinuation stack**: Handler frames, thread-local state, dynamic winders.
2. **Targets vector**: A cache mapping effect types to their handler implementations for O(1) dispatch.

These are bundled into a "registers" structure threaded through all computations:

```haskell
newtype Registers# = Registers# (# PromptId, Targets# #)
pattern Registers :: PromptId -> Targets -> Registers
```

### The EVM and Eff Types

```haskell
-- Raw machine type: takes registers, runs in IO, returns updated registers + result
newtype EVM a = EVM# { unEVM# :: Registers# -> IO (Result a) }
data Result a = Result Registers# ~a

-- User-facing type: adds type-level effect tracking
type Eff :: [Effect] -> Type -> Type
newtype Eff effs a = Eff# { unEff# :: EVM a }
```

`Eff` is a newtype over `EVM`, adding the `effs` type-level list that tracks which effects are available. Both are `Functor`, `Applicative`, `Monad`, and `MonadIO`.

### Effect Kind

Effects have kind `(Type -> Type) -> Type -> Type`, parameterized over the monad and the return type:

```haskell
type Effect = (Type -> Type) -> Type -> Type

data State s :: Effect where
  Get :: State s m s
  Put :: ~s -> State s m ()

data Error e :: Effect where
  Throw :: e -> Error e m a
  Catch :: Eff (Error e ': effs) a -> (e -> Eff effs a) -> Error e (Eff effs) a
```

## Delimited Continuations via GHC Primops

### The Two Core Primops

```haskell
prompt# :: ((State# s -> (# State# s, a #)) -> State# s -> (# State# s, a #))
         -> State# s -> (# State# s, a #)

control0# :: ((a -> State# s -> (# State# s, b #)) -> State# s -> (# State# s, b #))
           -> State# s -> (# State# s, a #)
```

- **`prompt#`**: Installs a delimiter (prompt) on the call stack. This marks the boundary up to which a continuation can be captured.
- **`control0#`**: Captures the continuation from the current point up to (but not including) the nearest enclosing prompt, packages it as a function, and passes it to the handler.

### Unwind Mechanism

Control operations are implemented via exceptions rather than direct stack capture:

```haskell
data Unwind
  = UnwindAbort PromptId ~Any            -- jump to handler, return value
  | UnwindControl (Capture Any)          -- capture continuation

data Capture a where
  Capture
    :: PromptId
    -> CaptureMode                       -- IncludePrompt | ExcludePrompt
    -> ((b -> EVM c) -> EVM d)           -- user's handler function
    -> (b -> EVM a)                      -- composed continuation
    -> Capture a
```

When a control operation is invoked, it throws an `UnwindControl` exception. The exception propagates up through `promptVM`, which catches it and dispatches accordingly.

### promptVM and controlVM

```haskell
promptVM
  :: IO (Registers, a)
  -> (a -> IO b)                         -- return handler
  -> (PromptId -> Any -> IO b)           -- abort handler
  -> (Capture a -> IO b)                 -- capture handler
  -> IO b
promptVM m onReturn onAbort onControl = IO.handle handleUnwind do
  Result _ a <- IO (prompt# (unIO (packIOResult m)))
  onReturn a

controlVM :: ((a -> EVM b) -> IO (Registers, b)) -> IO (Registers, a)
controlVM f = IO (control0# f#) ...
```

`promptVM` wraps `prompt#` and installs an exception handler for `Unwind`. `controlVM` wraps `control0#` and captures the current continuation.

## Effect Dispatch: O(1) via Targets Vector

### Type-Level Index Computation

```haskell
class eff :< effs where
  reifyIndex :: Int

instance {-# OVERLAPPING #-} eff :< (eff ': effs) where
  reifyIndex = 0

instance eff :< effs => eff :< (eff' ': effs) where
  reifyIndex = reifyIndex @eff @effs + 1
```

The `:<` class computes a compile-time index for each effect in the type-level list. This becomes a runtime `Int` used for array lookup.

### Targets Vector

```haskell
newtype Targets = Targets (SmallArray Any)

lookupTarget :: (eff :< effs) => Targets -> Handler eff
lookupTarget (Targets ts) = case indexSmallArray ts (reifyIndex @eff @effs) of
  (# Any h #) -> h

pushTarget :: Handler eff -> Targets -> Targets
pushTarget h (Targets ts1) = Targets $ runSmallArray do
  let len = sizeofSmallArray ts1
  ts2 <- newSmallArray (len + 1) null#
  writeSmallArray ts2 0 (Any h)
  copySmallArray ts2 1 ts1 0 len
  pure ts2
```

Handlers are stored in a `SmallArray` indexed by their type-level position. Installing a handler pushes it to position 0 and shifts existing handlers right.

### send: Effect Dispatch

```haskell
send :: forall eff a effs. eff :< effs => eff (Eff effs) a -> Eff effs a
send !e = Eff \rs@(Registers _ ts) -> unEff (runHandler (lookupTarget @effs ts) e) rs
```

Dispatch is: compute index (compile-time) -> array lookup (O(1)) -> call handler. No stack walking or dynamic search.

## Handler Installation

### The Handler and Handle Types

```haskell
newtype Handler eff = Handler#
  { runHandler# :: forall effs a. eff :<# effs
      -> eff (Eff effs) a -> Registers# -> State# RealWorld
      -> (# State# RealWorld, Registers#, a #)
  }

type Handle :: Effect -> [Effect] -> Type -> Type -> [Effect] -> Type -> Type
newtype Handle eff effs i r effs' a = Handle# { runHandle# :: Registers# -> Eff effs' a }
```

`Handler` is the stored handler function. `Handle` is the monad in which handler code runs, with type parameters tracking the effect being handled, available effects, intermediate/final return types, and the originating `send` site.

### handle: The Core Handler Function

```haskell
handle
  :: forall eff a r effs
   . (a -> Eff effs r)                   -- exit handler (onReturn)
  -> (forall effs' b. eff :< effs'
        => eff (Eff effs') b
        -> Handle eff effs a r effs' b)  -- operation handler
  -> Eff (eff ': effs) a                 -- computation
  -> Eff effs r                          -- result (effect removed)
```

The handler function receives:

1. An exit handler applied to the computation's final value.
2. An operation handler that interprets each effect operation.
3. The computation to handle.

The computation's type changes from `Eff (eff ': effs) a` to `Eff effs r`, removing the handled effect.

### Handle Operations

```haskell
liftH   :: Eff (eff ': effs) a -> Handle eff effs i r effs' a
abort    :: r -> Handle eff effs i r effs' a
control  :: ((a -> Eff effs r) -> Eff effs r) -> Handle eff effs i r effs' a
control0 :: ((a -> Eff (eff ': effs) i) -> Eff effs r) -> Handle eff effs i r effs' a
locally  :: Eff effs' a -> Handle eff effs i r effs' a
```

- **`liftH`**: Run an action in the outer effect context.
- **`abort`**: Jump to the handler, returning a value directly (used for exceptions).
- **`control`**: Capture continuation _including_ the handler prompt. The continuation has the same return type as the final result.
- **`control0`**: Capture continuation _excluding_ the handler prompt. Allows reinstalling the same handler on the captured continuation.
- **`locally`**: Run an action in the context of the original `send` call (used for scoped operations like `catch`, `local`).

## Built-in Effects

### State

```haskell
evalState :: s -> Eff (State s ': effs) a -> Eff effs a
evalState s0 (Eff m0) = Eff \rs -> do
  ref <- newIORef s0
  promptVM_ (m0 (pushHandler ref rs)) rs \(Capture ...) -> ...
  where
    pushHandler ref (Registers pid ts) =
      let h = Handler \case
            Get   -> Eff# $ liftIO $ readIORef ref
            Put s -> Eff# $ liftIO $ writeIORef ref s
      in Registers pid (pushTarget h ts)
```

State uses `IORef` for mutable storage. On continuation capture, the current state value is read and a _new_ `IORef` is created when the continuation is resumed, giving transactional semantics.

### Error

```haskell
runError :: Eff (Error e ': effs) a -> Eff effs (Either e a)
runError = handle (pure . Right) \case
  Throw e   -> abort $ Left e
  Catch m f -> locally (either f pure =<< runError m)
```

`Throw` uses `abort` to jump to the handler. `Catch` uses `locally` to run the recovery in the original context.

### Reader

```haskell
runReader :: r -> Eff (Reader r ': effs) a -> Eff effs a
runReader r = handle pure \case
  Ask       -> liftH $ pure r
  Local f m -> locally $ let !r' = f r in runReader r' m
```

### Writer

Implemented via `State` internally, using `IORef` to accumulate output. Supports `tell`, `listen`, and `censor`.

### NonDet

```haskell
runNonDetAll :: Alternative f => Eff (NonDet ': effs) a -> Eff effs (f a)
runNonDetAll = handle (pure . pure) \case
  Empty  -> abort empty
  Choose -> control \k -> liftA2 (<|>) (k True) (k False)
```

Uses `control` (not `control0`) to fork execution. The continuation `k` is invoked twice, once with `True` and once with `False`, and results are combined with `<|>`. This is a multi-shot continuation.

### Coroutine

```haskell
runCoroutine :: Eff (Coroutine a b ': effs) c -> Eff effs (Status effs a b c)
runCoroutine = handle (pure . Done) \case
  Yield a -> control0 \k -> pure $! Yielded a k
```

Uses `control0` to capture the continuation excluding the handler prompt, allowing the coroutine to be resumed externally.

## Semantics: Consistent Composition

A key advantage of `eff` over `mtl`-family libraries is _consistent semantics regardless of handler order_:

- **State + Error**: State modifications inside `catch` are always visible after error recovery, regardless of whether State or Error is handled first.
- **NonDet + Error**: All branches are always executed, even if created within a `catch`.

This falls out naturally from delimited control semantics and state transactionality (capturing/restoring `IORef` state on continuation capture).

## GHC-Specific Features Used

| Feature                          | Where Used                          | Rust Equivalent                              |
| -------------------------------- | ----------------------------------- | -------------------------------------------- |
| `prompt#` / `control0#` primops  | Core continuation machinery         | No equivalent; requires alternative approach |
| Unboxed types (`Int#`, `State#`) | Performance optimization            | N/A (Rust values are unboxed by default)     |
| `RankNTypes`                     | Handler type signatures             | Trait objects or HRTB (`for<'a>`)            |
| `TypeFamilies`                   | `DictRep` for constraint reflection | Associated types                             |
| `GADTs`                          | Effect operation definitions        | Rust enums                                   |
| `DataKinds` / type-level lists   | Effect tracking                     | Trait-based HLists or tuples                 |
| `OverlappingInstances`           | `:<` index computation              | Trait specialization (unstable)              |
| `unsafeCoerce`                   | `Any` wrapping in targets vector    | `std::mem::transmute`                        |
| `IORef`                          | State effect implementation         | `RefCell` / `Cell`                           |
| IO exceptions                    | Unwind mechanism                    | `panic`/`catch_unwind` or explicit `Result`  |
| `SmallArray#`                    | Targets vector                      | `Vec` or `SmallVec`                          |

## What Form of Delimited Continuations Does `eff` Need?

### The Delimited Continuation Zoo

Delimited continuation operators differ along two axes (following Dybvig, Peyton-Jones, and Sabry's "A Monadic Framework for Delimited Continuations"):

1. **Does capturing remove the delimiter from the stack?** `control0` / `shift0` remove it; `control` / `shift` leave it in place.
2. **When the captured continuation is invoked, is a fresh delimiter installed around it?** `shift` / `shift0` reinstall; `control` / `control0` do not.

This gives four operators:

| Operator   | Delimiter removed on capture? | Delimiter reinstalled on invoke? | Origin                       |
| ---------- | ----------------------------- | -------------------------------- | ---------------------------- |
| `shift`    | No                            | Yes                              | Danvy & Filinski (1990)      |
| `control`  | No                            | No                               | Sitaram & Felleisen (1990)   |
| `shift0`   | Yes                           | Yes                              | Danvy-Filinski CPS hierarchy |
| `control0` | Yes                           | No                               | Kiselyov / Dybvig et al.     |

Wikipedia's "Delimited continuation" article covers only `shift` / `reset`; the other variants come from the research literature (Dybvig et al. 2007; Kiselyov 2012; Kiselyov and Shan's work on delimited control in Haskell).

### eff's Primitive: GHC's `control0#`

The GHC RTS provides `prompt#` (installs a delimiter) and `control0#` (captures up to but excluding the delimiter, removes the delimiter, does not reinstall on invocation). In terms of the zoo above, `control0#` implements the `control0` operator. This is the most expressive of the four: `shift`, `control`, and `shift0` can all be implemented on top of `control0` by manually inserting or not inserting `prompt` calls around appropriate points. The reverse is not true. Kiselyov's paper "Delimited Control in OCaml, Abstractly and Concretely" establishes this expressiveness hierarchy.

### eff's User-Level Operators

On top of `control0#`, eff exposes three operators in the `Handle` monad:

1. **`abort :: r -> Handle eff effs i r effs' a`**

   A non-local exit. Implemented via an `UnwindAbort` exception caught by the enclosing `promptVM`. No continuation capture; the continuation is simply discarded. Used for `Throw` (Error) and `Empty` (NonDet).

2. **`control :: ((a -> Eff effs r) -> Eff effs r) -> Handle eff effs i r effs' a`**

   Captures the continuation _with_ the prompt included (`CaptureMode = IncludePrompt`). When the continuation is invoked, the handler is effectively reinstalled because the prompt was never removed from the continuation. This is `shift`-like behavior: composable, re-enterable, and crucially **multi-shot** — the continuation can be invoked zero, one, or many times. Used for `Choose` in NonDet:

   ```haskell
   Choose -> control \k -> liftA2 (<|>) (k True) (k False)
   ```

   `k True` and `k False` each run the rest of the computation under the same NonDet handler, producing `f a` values that are combined with `<|>`.

3. **`control0 :: ((a -> Eff (eff ': effs) i) -> Eff effs r) -> Handle eff effs i r effs' a`**

   Captures the continuation _without_ the prompt (`CaptureMode = ExcludePrompt`). The user receives a continuation whose type still mentions the handled effect (`Eff (eff ': effs) i`), meaning they must reinstall a handler for it if they want to invoke the continuation. This is `control0`-like behavior. Used for `Yield` in Coroutine:

   ```haskell
   Yield a -> control0 \k -> pure $! Yielded a k
   ```

   The continuation `k` is handed to the external caller as part of a `Yielded` value. To resume, the caller passes a new value and implicitly reinstalls the coroutine handler. This gives coroutines/generators their characteristic API.

4. **`liftH`** and **`locally`** are not continuation captures at all — they are adjustments to the registers (targets vector) used to run a sub-computation under a different effect view. `liftH` runs an action with the outer effect list visible (used by `Ask` to return a plain value without re-sending). `locally` runs an action with the _caller's_ effect list visible (used by `Catch` and `Local`, where the inner computation should bypass the current handler).

### Summary of What eff Needs

| Use Site                                             | Operation Needed          | Multi-Shot? | Notes                                                        |
| ---------------------------------------------------- | ------------------------- | ----------- | ------------------------------------------------------------ |
| `Throw` (Error), `Empty` (NonDet)                    | Abortive jump             | N/A         | No continuation needed; just unwind to handler.              |
| `Ask` (Reader), `Get`/`Put` (State), `Tell` (Writer) | None (tail-resumptive)    | N/A         | Handler produces value directly; continuation is implicit.   |
| `Choose` (NonDet)                                    | Shift-like (control)      | **Yes**     | Invoke continuation twice to fork.                           |
| `Yield` (Coroutine)                                  | Control0                  | No          | Continuation handed to external code.                        |
| `Catch` (Error), `Local` (Reader)                    | Delimited dynamic binding | N/A         | Run sub-computation with different handlers (via `locally`). |
| `Listen`, `Censor` (Writer)                          | Nested handlers           | N/A         | Install a fresh Writer handler for the scoped region.        |

The hardest-to-replicate capability is **multi-shot continuations** (needed for NonDet). Everything else can be approximated with simpler mechanisms.

## Can `switch-resume` Help?

`switch-resume` (<https://crates.io/crates/switch-resume>) is a Rust crate by @kuviman that implements delimited continuations on top of Rust's stable async/await. It is the closest thing in Rust's ecosystem to a real delimited-control primitive.

### What `switch-resume` Provides

```rust
pub async fn run<'a, T, Fut>(f: impl FnOnce(Task<'a, T>) -> Fut) -> T
where Fut: Future<Output = T> + 'a;

impl<'a, T> Task<'a, T> {
    pub async fn switch<ResumeArg, Fut, F>(&self, f: F) -> ResumeArg
    where
        Fut: Future<Output = T> + 'a,
        F: FnOnce(Resume<'a, ResumeArg, T>) -> Fut + 'a;
}

pub type Resume<'a, Arg, T> = Box<dyn FnOnce(Arg) -> Continuation<'a, T> + 'a>;
```

Semantics:

- **`run(f)`** installs a delimiter and drives the body future `f(task)` to completion.
- **`task.switch(g)`** pauses the current future, captures the continuation (the rest of the body up to the `run` boundary), and transfers control to `g(resume)`. The closure `g` produces a new future of the same result type `T`.
- **`resume(arg)`** is a `FnOnce` — one-shot. Awaiting it feeds `arg` back into the pause point and runs the captured continuation to completion, returning the final `T` into the switching closure.

### Example (from the project's README)

```rust
switch_resume::run(|task| async move {
    println!("begin");
    task.switch(|resume| async move {
        println!("before");
        resume(()).await;
        println!("after");
    })
    .await;
    println!("end");
})
// Output: begin, before, end, after
```

The continuation (everything from `switch` to the end of `run`) runs between "before" and "after" when `resume` is awaited.

### Comparison: `switch-resume` vs. `shift`/`reset` vs. eff's Operators

| Aspect                           | Wikipedia `shift`/`reset`     | eff's `control`               | eff's `control0`              | `switch-resume`                         |
| -------------------------------- | ----------------------------- | ----------------------------- | ----------------------------- | --------------------------------------- |
| Delimiter primitive              | `reset`                       | `prompt#`                     | `prompt#`                     | `run`                                   |
| Capture primitive                | `shift`                       | (`control0#` + IncludePrompt) | (`control0#` + ExcludePrompt) | `switch`                                |
| Delimiter removed on capture?    | No                            | No                            | Yes                           | No (one delimiter per `run`)            |
| Delimiter reinstalled on invoke? | Yes                           | Yes (still present)           | No                            | No (the original `run` is still active) |
| Multi-shot?                      | Yes (pure)                    | Yes                           | Yes                           | **No** (`FnOnce`)                       |
| Direct style?                    | Yes (in supported langs)      | Yes (in `Eff`)                | Yes (in `Eff`)                | No (requires `.await` everywhere)       |
| Multiple independent prompts?    | Yes (prompt tags / hierarchy) | Yes (one per handler)         | Yes (one per handler)         | **No** (one per `run`)                  |
| First-class continuation type    | Pure function                 | `a -> Eff effs r`             | `a -> Eff (eff ': effs) i`    | `Box<dyn FnOnce(Arg) -> Future>`        |

Closest match: `switch-resume`'s `switch` is roughly equivalent to `shift` at the _semantic_ level (delimiter stays, continuation composable), but operationally it is one-shot because Rust's `FnOnce` in the `Resume` type prevents multiple invocations, and it supports only a single global delimiter per `run` invocation.

### What `switch-resume` Enables for Our Effects System

`switch-resume` alone is not a complete substrate for eff's full feature set, but it could support a meaningful subset:

**Supported:**

- **Tail-resumptive effects** (Reader `ask`, State `get`/`put`, Writer `tell`): implement as plain function calls against a handler stored in the task environment. No continuation capture needed — these don't use `switch` at all.
- **Abortive effects** (Error `throw`, NonDet `empty`): implement via `switch` with a closure that ignores `resume` and returns the error/empty value directly, effectively aborting the task.
- **One-shot resumption** (Coroutine-like `yield` where the consumer only resumes once): the `switch` closure hands `resume` to external code. Since `FnOnce` allows exactly one call, this fits.

**Not supported without additional machinery:**

- **Multi-shot continuations** (NonDet `choose`, `Alternative <|>`): fundamentally impossible with `FnOnce`. Supporting this would require cloning the entire async state machine, which Rust does not expose.
- **Multiple independent handlers** (nested `catch`, nested `local`, nested `runState`): `switch-resume` has one delimiter per `run` invocation. You could nest `run`s, but then the inner `switch` only captures up to the inner `run`, losing the ability to cross handler boundaries in the way eff's `control0` + targets-vector approach does.
- **Lexical/scoped operations** (`Catch`, `Local`, `Listen`, `Censor`): require running a sub-computation with a different handler context. With `switch-resume`, you could simulate this by manually threading an effect environment, but it no longer resembles `eff`'s design.

**Implementation cost:**

- Rust's "function coloring" (async vs. sync) infects all effectful code: every function that might perform an effect must be `async`. This is a significant ergonomic tax.
- `Box<dyn ...>` on every `switch` call and every resume adds allocation overhead that is absent from eff's unboxed `control0#`.
- The `'a` lifetime bound on `Resume` means effects cannot easily carry borrowed data; most effect payloads need to be owned.

### Assessment

`switch-resume` is a genuine delimited continuation implementation and is closer to eff's primitive than any other pure-Rust option. It could serve as the substrate for a partial Rust port of eff that supports:

- State, Reader, Writer, Error (all first-order or abortive)
- Simple Coroutine-style yielding (one-shot)

But it cannot support:

- NonDet with `Alternative` (multi-shot)
- Multiple independent prompts (each effect handler as its own delimiter)
- Scoped operations (`catch`, `local`) in their full generality
- Sync (non-async) code paths

This is enough to cover the "boring but useful 80%" of an effects library, but falls short of being a full eff port. The "interesting" cases — the ones that justify algebraic effects as a framework rather than a dependency-injection pattern — are precisely the ones `switch-resume` cannot handle.

## Feasibility for Rust Port

### Blockers for a Full Port

**Multi-shot delimited continuations**: The fundamental blocker. eff's `control` requires invoking a captured continuation multiple times. Rust has no mechanism to clone a paused async state machine or a stack slice, and is unlikely to gain one without substantial language changes.

**Multiple independent prompts with O(1) dispatch**: eff's targets vector gives constant-time effect dispatch because each prompt corresponds to a specific handler stored at a known index. Replicating this in Rust would require either (a) a custom stackful runtime with explicit prompt management, or (b) accepting O(n) dispatch via a free monad.

### Partial-Port Options

1. **`switch-resume`-based partial port**: Supports the tail-resumptive and one-shot subset. Would require users to write async code and accept that NonDet and multi-shot operations are unavailable. Lightweight but incomplete.

2. **Custom stackful runtime (`libhandler`-style)**: Use `setjmp`/`longjmp` or platform-specific fiber APIs (e.g., `ucontext`, Windows Fibers) to implement full delimited continuations. Platform-specific, unsafe, not portable to all targets (notably WASM). This is how Koka's C runtime works.

3. **Free monad approach (abandon continuations)**: Build effects as a data structure and interpret them with explicit stack-safe loops. Supports multi-shot via re-interpretation (the tree can be walked multiple times). Slower than eff but portable and pure. This is what `purescript-run` does and what our [feasibility-comparison.md](./feasibility-comparison.md) recommends.

### Conclusion

The `eff` approach is fundamentally tied to GHC's runtime support for delimited continuations, specifically `control0#` with multi-shot capability. A faithful port to Rust is not feasible without either unsafe stack manipulation (option 2) or accepting a weaker feature set (option 1). The library's **API design** and **semantics** are valuable references and should inform the handler API of whatever approach we choose.

`switch-resume` is worth keeping in mind as a potential building block for experiments or for a future "async effects" variant, but it is not a substitute for a full delimited-continuation primitive. Its one-shot limitation and single-prompt-per-`run` design rule out the most interesting uses of `eff`.

The recommended path remains: use `eff`'s semantics and handler API as design targets while implementing via a free-monad-based approach (closer to `purescript-run`). See [feasibility-comparison.md](./feasibility-comparison.md) for the proposed Rust design.
