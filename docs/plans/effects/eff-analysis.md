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

## Feasibility for Rust Port

### Blockers

**Delimited continuations**: The entire architecture depends on `prompt#` / `control0#`, which allow capturing and resuming slices of the call stack. Rust has no equivalent. Options:

1. Use `setjmp`/`longjmp` with stack copying (unsafe, platform-specific).
2. Use Rust's `async` machinery as a form of stackless coroutine (limited).
3. Abandon the continuation-based approach entirely and use a free monad instead.

### Conclusion

The `eff` approach is fundamentally tied to GHC's runtime support for delimited continuations. A faithful port to Rust is not feasible without either unsafe stack manipulation or a custom runtime. The library's _API design_ and _semantics_ are valuable references, but the _implementation strategy_ does not translate to Rust.

The recommended path is to use `eff`'s semantics and handler API as design targets while implementing via a free-monad-based approach (closer to `purescript-run`).
