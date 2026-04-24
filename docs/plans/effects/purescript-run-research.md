# Extensible Algebraic Effects in PureScript: A Deep Dive into `purescript-run`

## Purpose of This Document

This document is a research and reference note for a future port of the PureScript `Run` effects system into Rust. It catalogs every type, type class, function, and language feature that the PureScript implementation relies on, and explains how those pieces combine to produce a row-polymorphic, extensible effects monad. The goal is not to design the Rust port here, but to provide a complete, verified picture of what must be ported (or replaced with a Rust-appropriate substitute).

Libraries covered:

- `purescript-run` at [github.com/natefaubion/purescript-run](https://github.com/natefaubion/purescript-run) (the effects library itself).
- `purescript-variant` at [github.com/natefaubion/purescript-variant](https://github.com/natefaubion/purescript-variant) (open sum types, including `VariantF`).
- `purescript-free` at [github.com/purescript/purescript-free](https://github.com/purescript/purescript-free) (the `Free` monad wrapper).
- `purescript-tailrec` at [github.com/purescript/purescript-tailrec](https://github.com/purescript/purescript-tailrec) (stack-safe monadic recursion via `MonadRec`).
- `purescript-type-equality` at [github.com/purescript/purescript-type-equality](https://github.com/purescript/purescript-type-equality) (zero-cost type equality proofs used for row manipulation).

---

## 1. Motivation: Why `Run` Exists

From the `purescript-run` README:

> "Run is an implementation of extensible, algebraic effects for PureScript. This means we can write composable programs using normal PureScript data types, and then provide interpreters for those data types when we actually want to run them. Our effect descriptions naturally compose with others, so we don't need to write a large encompassing data type, or explicitly lift things through transformer stacks."

The library addresses the classical composability problem of effectful code. The three competing approaches that `Run` is designed to improve on are:

1. **Monolithic effect types.** If every effect lives in one large sum type, adding a new effect requires editing that type and every interpreter. Code becomes coupled to the union.
2. **Monad transformer stacks (mtl-style).** Transformers compose, but require explicit `lift` calls to reach deeper layers, depend on `newtype` wrappers that can leak through instances, and expose a fragile instance-resolution story (overlapping `MonadState s`, etc.).
3. **Typeclass-based effects (mtl).** Works for small programs but pollutes signatures with many constraints, and adding a new effect means adding a new class plus instances for every transformer.

`Run` avoids all of these by representing the _set of effects available to a program_ as an open **row type** parameter `r`. Programs then have signatures like `Run (READER Int + STATE String + r) a`. Interpreting an effect rewrites the row by removing it:

```purescript
runReader :: forall e r a. e -> Run (READER e + r) a -> Run r a
```

This gives three big practical wins:

- **Open composition.** Any two effects can be combined without prior coordination. The compiler tracks the union at the type level.
- **Partial interpretation.** Handlers can peel off a single effect while leaving the rest of the row intact. The README stresses this ([README.md:306](https://github.com/natefaubion/purescript-run/blob/master/README.md#L306)): "Oftentimes we only want to handle [effects] in isolation. That is, we want to interpret one effect in terms of other effects at our convenience."
- **Stack safety as an interpreter choice.** The library exposes both `interpret`/`run` (fast, but depend on target stack safety) and `interpretRec`/`runRec` (stack-safe via `MonadRec`).

The fundamental trick: `newtype Run r a = Run (Free (VariantF r) a)`. It is the composition of two orthogonal pieces; `VariantF` gives row-polymorphic sums of functors, and `Free` turns any functor into a monad. The rest of the library is handler combinators and ergonomic wrappers.

---

## 2. Architectural Overview: How the Pieces Fit

```
   User code (do-notation)
           |
           v
        Run r a                 <-- newtype wrapper with derived Monad
           |
           v
     Free (VariantF r) a        <-- Free provides Monad; VariantF provides choice of effect
           |                         (Free: "Reflection without Remorse" sequence representation)
           v
       VariantF r x             <-- one of N effect functors, tagged by a string label
           |                        (represented at runtime as { type, value, map })
           v
      effect functor f_i        <-- e.g., State s, Reader e, Writer w, Except e, Choose, TalkF, ...
```

Each layer has a single responsibility:

- `Free` gives monadic sequencing: `bind`, `pure`, stack-safe interpretation, `foldFree`, `hoistFree`, `substFree`.
- `VariantF` gives the open-sum-of-functors: `inj`, `prj`, `on`, `case_`, `match`, `expand`, `contract`.
- `Run` is the user-facing wrapper. It derives `Monad` via newtype deriving, adds a `MonadRec` instance, and supplies the combinator zoo (`lift`, `send`, `peel`, `resume`, `interpret`, `run`, `runRec`, `runAccum`, `runAccumRec`, `runCont`, `runPure`, `runAccumPure`, `extract`, `expand`, plus `Effect`/`Aff` bridges).
- Per-effect modules (`Run.Reader`, `Run.State`, `Run.Writer`, `Run.Except`, `Run.Choose`) each define one functor shape, its constructors, and an interpreter.

A user writes a program in the `Run` monad, adding each required effect to the row. At the boundary (e.g., `main`), they stack interpreters in any order: each interpreter strips one label from the row and produces a new `Run` over the reduced row. Eventually the row becomes `()` and `extract :: Run () a -> a` yields the result.

---

## 3. Core Data Types

### 3.1 `Run`

Defined at [Run.purs:56-93](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs#L56-L93):

```purescript
newtype Run r a = Run (Free (VariantF r) a)

derive instance newtypeRun :: Newtype (Run r a) _
derive newtype instance functorRun :: Functor (Run r)
derive newtype instance applyRun :: Apply (Run r)
derive newtype instance applicativeRun :: Applicative (Run r)
derive newtype instance bindRun :: Bind (Run r)
derive newtype instance monadRun :: Monad (Run r)
```

All instances for `Functor`, `Apply`, `Applicative`, `Bind`, `Monad` are inherited via newtype deriving from `Free`. The only hand-written typeclass instance is `MonadRec`:

```purescript
instance monadRecRun :: MonadRec (Run r) where
  tailRecM f = loop
    where
    loop a = do
      b <- f a
      case b of
        Done r -> pure r
        Loop n -> loop n
```

`Run`'s own `tailRecM` is written in terms of `Run`'s own `bind`, so stack safety is deferred to whatever monad the program is finally interpreted into.

### 3.2 `VariantF`

Defined at [Data/Functor/Variant.purs:58-59](https://github.com/natefaubion/purescript-variant/blob/master/src/Data/Functor/Variant.purs#L58-L59):

```purescript
data VariantF :: Row (Type -> Type) -> Type -> Type
data VariantF f a
```

The kind `Row (Type -> Type) -> Type -> Type` is crucial: `VariantF` is indexed by a row whose entries are themselves type constructors (kind `Type -> Type`, i.e., functors). The second parameter `a` is the value carried inside the chosen functor.

Runtime representation (used internally via `unsafeCoerce`):

```purescript
newtype VariantFRep f a = VariantFRep
  { type :: String       -- the label, e.g. "reader"
  , value :: f a         -- the actual functor value
  , map :: Mapper f      -- the functor's fmap, stored alongside
  }
```

Where `Mapper f = Mapper (forall a b. (a -> b) -> f a -> f b)` (a rank-2 newtype wrapping `fmap`). This is how `VariantF` can have a `Functor` instance without knowing `f` statically at the point of use: the `fmap` dictionary is carried in the value.

Parallel `Variant` (non-functor version used outside `Run`):

```purescript
foreign import data Variant :: Row Type -> Type

newtype VariantRep a = VariantRep
  { type :: String
  , value :: a
  }
```

`Variant` does not carry a `map` field because it holds a plain `a`, not an `f a`.

### 3.3 `Free`

Defined at [Control/Monad/Free.purs:38-42](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs#L38-L42):

```purescript
data Free f a = Free (FreeView f Val Val) (CatList (ExpF f))

newtype ExpF f = ExpF (Val -> Free f Val)

data FreeView f a b = Return a | Bind (f b) (b -> Free f a)

data Val
```

Notable features:

- `Val` is an opaque phantom type. The accumulated continuations are stored with `Val` in place of their real types. `unsafeCoerce` moves between `Val` and real types when values are extracted. This is how a heterogeneous continuation list can be typed homogeneously.
- `CatList` is a catenable list with O(1) append. Each `>>=` appends the continuation, rather than building a nested `Bind`. This is the **"Reflection without Remorse"** optimization from Van der Ploeg and Kiselyov's paper, and it removes the classic quadratic-blowup problem of left-associated binds.
- `FreeView` is the logical view constructor the user sees after running `toView`: either a `Return` or a single `Bind` whose continuation is the flattened `CatList`.

The bind implementation at [Free.purs:68-72](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs#L68-L72):

```purescript
instance freeBind :: Bind (Free f) where
  bind (Free v s) k = Free v (snoc s (ExpF (unsafeCoerceBind k)))
    where
    unsafeCoerceBind :: forall a b. (a -> Free f b) -> Val -> Free f Val
    unsafeCoerceBind = unsafeCoerce
```

`snoc s (ExpF k)` appends the new continuation to the `CatList`. When interpretation reaches this node, `toView` [Free.purs:223-245](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs#L223-L245) walks and re-associates the list into a single normalized `FreeView`.

### 3.4 `Step` (from `purescript-tailrec`)

[Control/Monad/Rec/Class.purs](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs):

```purescript
data Step a b = Loop a | Done b
```

A binary sum used as the return of each iteration of `tailRecM`. `Loop a` means "continue with new accumulator `a`"; `Done b` means "stop with final value `b`". It is the driver for all stack-safe iteration in the ecosystem.

### 3.5 Per-Effect Functor Shapes

Each effect module defines a small ADT that is the functor parameter of `VariantF`.

```purescript
-- Run.Reader
newtype Reader e a = Reader (e -> a)
type READER e r = (reader :: Reader e | r)

-- Run.State
data State s a = State (s -> s) (s -> a)
type STATE s r = (state :: State s | r)

-- Run.Writer
data Writer w a = Writer w a
type WRITER w r = (writer :: Writer w | r)

-- Run.Except
newtype Except e a = Except e
type EXCEPT e r = (except :: Except e | r)
type Fail = Except Unit
type FAIL r = EXCEPT Unit r

-- Run.Choose
data Choose a = Empty | Alt (Boolean -> a)
type CHOOSE r = (choose :: Choose | r)
```

Every effect is a `Functor` whose `a` parameter represents "the rest of the program". This is the standard free-monad encoding: `Reader e a = Reader (e -> a)` means "I need an environment of type `e` and then I can continue with the result of type `a`".

---

## 4. Core Type Classes

### 4.1 `MonadRec`

[Control/Monad/Rec/Class.purs:58-59](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs#L58-L59):

```purescript
class Monad m <= MonadRec m where
  tailRecM :: forall a b. (a -> m (Step a b)) -> a -> m b
```

Instances drive `tailRecM` without using the host-language stack. For `Effect` [lines 124-137]:

```purescript
instance monadRecEffect :: MonadRec Effect where
  tailRecM f a = do
    r <- Ref.new =<< f a
    untilE do
      Ref.read r >>= case _ of
        Loop a' -> do
          e <- f a'
          _ <- Ref.write e r
          pure false
        Done _ -> pure true
    fromDone <$> Ref.read r
```

This uses a mutable reference and a JavaScript `while`-loop primitive (`untilE`) to iterate without recursion. Other instances (`Identity`, `Either`, `Maybe`, `Function`) are defined analogously, each avoiding stack growth.

### 4.2 `MonadFree`

Defined in `Control.Monad.Free.Class` as a typeclass for monads that can be interpreted as Free. Used for `liftFree` abstractions over transformers.

### 4.3 `TypeEquals`

[Type/Equality.purs:20-24](https://github.com/purescript/purescript-type-equality/blob/master/src/Type/Equality.purs#L20-L24):

```purescript
class TypeEquals :: forall k. k -> k -> Constraint
class Coercible a b <= TypeEquals a b | a -> b, b -> a where
  proof :: forall p. p a -> p b

instance refl :: TypeEquals a a where
  proof a = a
```

Two features make this work:

- **Functional dependencies** `a -> b, b -> a` force the compiler to unify. If a context requires `TypeEquals (Proxy r1) (Proxy (EFFECT r2))`, then `r1 ~ EFFECT r2` must hold.
- **Only one instance (`refl`).** Any derivation must go through `TypeEquals a a`, which is the identity.

This is used in `Run` to make instances like:

```purescript
instance runMonadEffect :: (TypeEquals (Proxy r1) (Proxy (EFFECT r2))) => MonadEffect (Run r1)
```

conditional on the row containing exactly the `effect` label.

### 4.4 Row Constraints (compiler-provided)

Not type classes in the user-defined sense, but constraints provided by `Prim.Row` and `Prim.RowList` that appear throughout:

- `Row.Cons sym a r1 r2`: "row `r2` equals row `r1` with label `sym : a` added."
- `Row.Union l m r`: "row `r` is the disjoint union of rows `l` and `m`."
- `Row.Lacks sym r`: "row `r` does not contain label `sym`."
- `RowToList r rl`: converts a row to a type-level list for inductive reasoning.
- `IsSymbol sym`: the symbol literal has a runtime `reflectSymbol :: Proxy sym -> String`.

These are what allow `lift`'s signature to say "adding `sym : f` to row `r1` gives row `r2`":

```purescript
lift
  :: forall sym r1 r2 f a
   . Row.Cons sym f r1 r2
  => IsSymbol sym
  => Functor f
  => Proxy sym
  -> f a
  -> Run r2 a
```

---

## 5. Key Functions in Detail

### 5.1 Variant / VariantF Operations

From [Data/Variant.purs:52-101](https://github.com/natefaubion/purescript-variant/blob/master/src/Data/Variant.purs#L52-L101):

**`inj`**: inject a value at a label.

```purescript
inj :: forall sym a r1 r2.
  R.Cons sym a r1 r2 => IsSymbol sym =>
  Proxy sym -> a -> Variant r2
```

Implementation: build `VariantRep { type: reflectSymbol p, value }` and `unsafeCoerce` to `Variant r2`.

**`prj`**: project a specific case, returning an `Alternative` (usually `Maybe`).

```purescript
prj :: forall sym a r1 r2 f.
  R.Cons sym a r1 r2 => IsSymbol sym => Alternative f =>
  Proxy sym -> Variant r2 -> f a
```

Defined as `prj p = on p pure (const empty)`.

**`on`**: peel one case off.

```purescript
on :: forall sym a b r1 r2.
  R.Cons sym a r1 r2 => IsSymbol sym =>
  Proxy sym -> (a -> b) -> (Variant r1 -> b) -> Variant r2 -> b
```

At runtime, compares the stored `type` string to `reflectSymbol p`. If equal, call the success handler on the value; otherwise coerce the variant to the smaller row and pass it to the fallback.

**`case_`**: exhaustiveness terminator.

```purescript
case_ :: forall a. Variant () -> a
case_ r = unsafeCrashWith case unsafeCoerce r of
  VariantRep v -> "Data.Variant: pattern match failure [" <> v.type <> "]"
```

Only typechecks when the variant row has been fully consumed.

**`match`**: exhaustive record-of-handlers.

```purescript
match :: forall rl r r1 r2 b.
  RL.RowToList r rl => VariantMatchCases rl r1 b => R.Union r1 () r2 =>
  Record r -> Variant r2 -> b
```

`R.Union r1 () r2` enforces that the handler record covers every case.

**`expand`**: widen to a larger row.

```purescript
expand :: forall lt a gt. R.Union lt a gt => Variant lt -> Variant gt
expand = unsafeCoerce
```

Safe because the runtime data does not change.

**`contract`**: narrow to a subset, fallibly.

```purescript
contract :: forall lt gt f. Alternative f => Contractable gt lt =>
  Variant gt -> f (Variant lt)
```

Checks at runtime whether the current tag is in the subset.

`VariantF` has the same surface with `f a` instead of `a` and an extra `Functor` requirement on `f` (so that the stored `Mapper f` can be constructed).

### 5.2 Free Monad Operations

From [Control/Monad/Free.purs](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs):

- **`liftF :: forall f. f ~> Free f`** [lines 123-130]: wrap a single `f a` into `Free f a`.
- **`resume :: Functor f => Free f a -> Either (f (Free f a)) a`** [lines 199-204]: observe one layer.
- **`resume' :: forall f a r. (forall x. f x -> (x -> Free f a) -> r) -> (a -> r) -> Free f a -> r`** [lines 207-215]: richer `resume` that exposes the existentially-typed intermediate.
- **`foldFree :: forall f m. MonadRec m => (f ~> m) -> Free f ~> m`** [lines 154-160]: stack-safe natural-transformation-based interpreter. Uses `tailRecM` under the hood.
- **`runFree :: forall f a. Functor f => (f (Free f a) -> Free f a) -> Free f a -> a`** [lines 174-180]: interpret by returning the next `Free` each step.
- **`hoistFree :: forall f g. (f ~> g) -> Free f ~> Free g`** [lines 148-149]: change the effect functor.
- **`substFree :: forall f g. (f ~> Free g) -> Free f ~> Free g`** [lines 164-170]: inline-substitute one effect for another without a `MonadRec` pass.

### 5.3 Run Combinator Zoo

All from [Run.purs](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs):

- **`lift`** [lines 116-124]: lift a raw effect functor into `Run`.
- **`send :: forall a r. VariantF r a -> Run r a`** [lines 144-148]: used inside handlers to re-emit an unhandled effect to the next interpreter.
- **`peel`** [lines 128-132]: observe one step of a `Run` program, returning `Either (VariantF r (Run r a)) a`.
- **`resume`** [lines 135-141]: elimination principle taking two continuations (effect vs. value).
- **`expand`** [lines 165-170]: widen the effect row (`unsafeCoerce` gated by `Row.Union`).
- **`extract :: forall a. Run () a -> a`** [lines 173-174]: extract a value once the row is empty.
- **`interpret` / `run`** [lines 178-197]: interpret into a `Monad`; caller's responsibility to ensure stack safety.
- **`interpretRec` / `runRec`** [lines 201-221]: interpret into a `MonadRec`, using `runFreeM` (Free's stack-safe fold) internally.
- **`runCont`** [lines 224-233]: continuation-passing interpreter; the handler receives instructions that already contain the rest of the program as a continuation.
- **`runAccum` / `runAccumRec`** [lines 237-261]: monadic interpreters that thread an accumulator.
- **`runPure`** [lines 279-292]: pure, non-monadic interpreter. Handler returns `Step (Run r1 a) (VariantF r2 (Run r1 a))`; `Loop r'` means "keep interpreting `r'`", `Done a'` means "emit this (different) effect to the output row".
- **`runAccumPure`** [lines 296-311]: pure interpreter plus accumulator.

The `Run.Reader`, `Run.State`, etc. modules are thin wrappers: each defines a label proxy, smart constructors (e.g., `ask`, `put`, `tell`, `throw`), and an interpreter built from `runPure` or `runAccumPure`. For example:

```purescript
-- Run.State
modify :: forall s r. (s -> s) -> Run (STATE s + r) Unit
put    :: forall s r. s -> Run (STATE s + r) Unit
get    :: forall s r. Run (STATE s + r) s
runState :: forall s r a. s -> Run (STATE s + r) a -> Run r (Tuple s a)
```

Base-monad bridges [lines 313-345](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs#L313-L345):

```purescript
type EFFECT r = (effect :: Effect | r)
liftEffect   :: forall r. Effect ~> Run (EFFECT + r)
runBaseEffect :: Run (EFFECT + ()) ~> Effect
runBaseEffect = runRec $ match { effect: \a -> a }

type AFF r = (aff :: Aff | r)
liftAff      :: forall r. Aff ~> Run (AFF + r)
runBaseAff   :: Run (AFF + ()) ~> Aff
runBaseAff'  :: Run (AFF + EFFECT + ()) ~> Aff
```

Note that `runBaseEffect` is forced to use `runRec` because `Effect` is not stack-safe in its `bind`; `Aff` is stack-safe, so `runBaseAff` can use the cheaper `run`.

---

## 6. Language Features Relied Upon

The list below is the complete set of PureScript features the implementation uses. Each item is a thing the Rust port must either reproduce or replace.

### 6.1 Higher-Kinded Types

Pervasive. `VariantF :: Row (Type -> Type) -> Type -> Type` is higher-kinded in two senses: the row contains type constructors (kind `Type -> Type`), and `VariantF` itself takes a row-of-functors. `Free :: (Type -> Type) -> Type -> Type`. Every natural transformation signature (`f ~> g` for `forall x. f x -> g x`) requires HKT.

### 6.2 Row Polymorphism

The central feature. `Run (STATE s + r) a` is a row-polymorphic type. `r` is an open row variable that can unify with any row that doesn't already contain `state`. The compiler resolves `Row.Cons`, `Row.Union`, and `Row.Lacks` constraints to decide whether a program is well-typed. This is not present in Rust.

### 6.3 Rank-N Types

Multiple places:

- `foldFree :: forall f m. MonadRec m => (f ~> m) -> Free f ~> m` takes a rank-2 function (the inner `forall x. f x -> m x`).
- `Mapper f = Mapper (forall a b. (a -> b) -> f a -> f b)` stored in `VariantFRep`.
- `Unvariant r = Unvariant (forall x. Unvariant' r x -> x)`.
- `runPure`, `runAccumPure` take callbacks that are polymorphic over the continuation's row and result type.

### 6.4 Type-Level String Literals (Symbols)

`Proxy "reader"`, `Proxy "state"`. The `IsSymbol` class reifies them to `String` at runtime via `reflectSymbol`. Critical for `inj`, `prj`, `on`, and for the stored `type :: String` field in the runtime `VariantRep`.

### 6.5 Functional Dependencies

`TypeEquals a b | a -> b, b -> a` uses fundeps to force bidirectional unification. Without them the `TypeEquals` class would be useless, because any instance could be ambiguous.

### 6.6 Newtype Deriving

`Run` derives `Functor`, `Apply`, `Applicative`, `Bind`, `Monad`, and `Newtype` entirely from `Free`. No hand-written code.

The `Newtype` class itself comes from `purescript-newtype` ([github.com/purescript/purescript-newtype](https://github.com/purescript/purescript-newtype)) and is a method-less class whose entire definition is `class Coercible t a <= Newtype t a | t -> a`. It exists only to enable the `wrap`/`unwrap` coercion pair (both implemented as `coerce`) and to let the compiler derive newtype instances. `purescript-run` uses it at two points only: the `derive instance newtypeRun :: Newtype (Run r a) _` line at [Run.purs:95](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs#L95), and `unwrap` calls inside `peel`, `resume`, `runRec`, and friends to pull the inner `Free (VariantF r) a` out of `Run`. For the Rust port this library has no counterpart: a Rust newtype `struct Run<R, A>(Free<VariantF<R>, A>)` exposes its inner via `self.0` or a `Deref` impl, and no trait machinery is required.

### 6.7 `unsafeCoerce`

Used at three critical sites:

1. `Free`'s `bind` (erasing continuation types into `Val`).
2. `VariantF`/`Variant` injection and projection (mapping to and from `VariantRep`).
3. `Run.expand` (widening the row after the `Row.Union` constraint has validated safety).

In every case the surrounding type system constraints make the coercion safe.

### 6.8 FFI

`purescript-variant` uses **no** JavaScript FFI. Variant is `foreign import data Variant :: Row Type -> Type` (an opaque type), and the runtime representation is a plain PureScript record coerced through `unsafeCoerce`. At runtime a variant is literally a JavaScript object `{ type: "label", value: ... }` (plus `map` for `VariantF`). This matters for the Rust port: the design is expressible entirely within the language, so there is no hidden host-language magic to replicate.

### 6.9 Garbage Collection and Laziness

`Free`'s `CatList` assumes persistent, shared, garbage-collected lists. `Trampoline = Free ((->) Unit)` depends on PureScript thunks being lazy. These are not language features of Rust, so the port will need ownership and allocation decisions.

### 6.10 Quick Reference: PureScript Feature to Rust Counterpart

The table below is a fast lookup of each PureScript mechanism used by `purescript-run` and the most plausible Rust counterpart for each. It is informational, not prescriptive; design tradeoffs live in [port-plan.md](port-plan.md).

| PureScript feature                                | Where used in `purescript-run`           | Plausible Rust counterpart                                                   |
| ------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------- |
| Row polymorphism (`r` in `Run r a`)               | Open-ended effect sets                   | No direct equivalent. Approximated via type-level lists or macros.           |
| `VariantF` (row-indexed functor sum)              | Open union of effect functors            | Nested coproduct (`Coproduct<H, T>`), closed tuple, or dynamic trait object. |
| Native higher-kinded types (`Functor f`, `~>`)    | Instance quantification                  | Brand-based HKT encoding already in `fp-library`.                            |
| `Row.Cons` / `Row.Union` / `Row.Lacks`            | Compile-time row arithmetic              | Custom traits (`Member<E, Row>`, `Remove<E, Row>`) on an HList encoding.     |
| `unsafeCoerce` (expand, coerceM, toRows/fromRows) | Zero-cost row widening                   | `std::mem::transmute` or pointer casts, each gated by a trait bound.         |
| Newtype deriving                                  | Zero-cost `Run` wrapper                  | `#[repr(transparent)]` plus manual impls (no trait analog needed).           |
| `IsSymbol` + `Proxy "label"`                      | Label-indexed effect access              | Per-effect zero-sized marker types, `TypeId`, or const generics (limited).   |
| `TypeEquals` witness                              | Instance-resolution equality constraints | Implicit via Rust's unification; occasional `PhantomData` wrapper.           |
| Rank-N types (`foldFree`, `Mapper f`)             | Handler callbacks polymorphic in `x`     | Generic fn parameters, trait objects (`dyn for<'a> Fn`), or GATs.            |

---

## 7. How It All Works Together: A Walkthrough

Reading the canonical example from [test/Examples.purs:13-106](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106):

```purescript
data TalkF a
  = Speak String a
  | Listen (String -> a)

derive instance functorTalkF :: Functor TalkF

type TALK r = (talk :: TalkF | r)
_talk = Proxy :: Proxy "talk"

speak :: forall r. String -> Run (TALK + r) Unit
speak str = Run.lift _talk (Speak str unit)

listen :: forall r. Run (TALK + r) String
listen = Run.lift _talk (Listen identity)
```

What happens step by step when a user writes `speak "hi"`:

1. `Speak "hi" unit` is a value of type `TalkF Unit`.
2. `Run.lift _talk (Speak "hi" unit)` calls `VariantF.inj _talk (Speak "hi" unit)`, producing `VariantF (talk :: TalkF | r) Unit` at runtime `{ type: "talk", value: <TalkF>, map: <functor dict for TalkF> }`.
3. `Run.lift` then calls `liftF` from `Free`, producing `Free (VariantF (talk :: TalkF | r)) Unit`.
4. The result is wrapped in `Run`, giving `Run (talk :: TalkF | r) Unit`.

A handler has signature like:

```purescript
handleTalk :: forall r. TalkF ~> Run (EFFECT + r)
handleTalk = case _ of
  Speak str next -> do
    liftEffect $ Console.log str
    pure next
  Listen reply -> pure (reply "I am Groot")

runTalk :: forall r. Run (EFFECT + TALK + r) ~> Run (EFFECT + r)
runTalk = interpret (on _talk handleTalk send)
```

`on _talk handleTalk send` is the pattern for a handler: "if the effect is `talk`, run `handleTalk`; otherwise (`send`) re-emit it unchanged into the same row." `interpret` then walks the `Free (VariantF r) a` structure, applying this handler at each `Bind` node, yielding a new `Run` with the `talk` label removed from its row.

The program's "main" is a chain of interpretations:

```purescript
main :: Effect (Tuple Bill Unit)
main =
  program3
    # runDinnerPure { stock: 10, bill: 0 }  -- removes DINNER
    # runTalk                                -- removes TALK, uses EFFECT
    # runBaseEffect                          -- collapses into Effect
```

Each `#` is a pipe that rewrites the row. The type system tracks the row shrinking at each step; the compiler rejects the pipeline if any effect is left unhandled.

**Stack safety.** Because all interpreters go through `Free`'s stack-safe fold (`runFreeM` via `runRec`), and `Free`'s bind uses `CatList` to avoid left-associated quadratic blowup, programs of arbitrary iteration depth (e.g., `forever`, `tailRecM`) run in constant stack.

---

## 8. What a Rust Port Must Supply

This section is an inventory, not a design. The Rust port must answer, or consciously skip, each of these.

### 8.1 Must Be Supplied

1. **A row-polymorphic effect index.** Rust has no row types. The port needs a substitute: type-level lists via traits (à la `frunk`, `generic-array`), const-generic tuples, or an enum-of-enums. Each option has tradeoffs for partial interpretation and inference.
2. **An open sum of functors (`VariantF`).** The runtime representation can be a tagged union: `{ tag: &'static str, value: Box<dyn Any>, map: Box<dyn Fn...> }`, or a generic enum. The `Mapper f` dictionary must be carried somehow; Rust traits can supply it statically via trait bounds or dynamically via vtables.
3. **A `Free`-equivalent monad.** Must be `Functor`, `Applicative`, `Monad` for the chosen HKT encoding (this project already has HKT machinery via Brand types). Must be stack-safe, meaning either (a) eager left-associated construction with efficient continuation sequencing, or (b) a catenable-list sequence with phantom type erasure via `Box<dyn Any>`, or (c) a trampoline loop in the interpreter.
4. **A `MonadRec`-equivalent.** Either a trait with a method `tail_rec_m`, or iterator-style interpreters that drive continuations in a `while let` loop.
5. **Natural transformations.** Rust can encode `f ~> g` as a trait like `trait Nat<F, G> { fn apply<A>(x: F<A>) -> G<A>; }`, but it requires HKT support. The existing Brand-based HKT system should suffice.
6. **Per-effect functors.** `Reader<E>`, `State<S>`, `Writer<W>`, `Except<E>`, `Choose`. Each is a small enum with one of the fields being a continuation.
7. **Handler combinators.** `inj`, `prj`, `on`, `case_`, `match`, `expand`, `send`, `peel`, `resume`. Plus the Run combinators: `lift`, `interpret`, `run`, `runRec`, `runAccum`, `runAccumPure`, `runPure`, `runCont`, `extract`, `expand`.
8. **Label encoding.** Rust has no type-level strings. Options: const generic strings (unstable/limited), zero-sized marker types per effect (one struct per effect acting as its label), or `TypeId` of the effect functor (simplest; labels become types).

### 8.2 Can Be Skipped or Simplified

1. **`Aff`/`Effect` bridges.** The Rust port will bridge into whatever async runtime or direct-effect model makes sense (e.g., `async fn`, `std::io`). There is no analog to Aff.
2. **`Variant` (non-functor version).** `purescript-run` only needs `VariantF`. `Variant` is used elsewhere in the PureScript ecosystem, not by Run directly.
3. **`Trampoline`, `Cofree`, `Yoneda`, `Coyoneda`.** These live in `purescript-free` but are not used by `purescript-run`.
4. **`unsafeCoerce`-based row widening.** Rust's equivalent will use `transmute` or a dedicated conversion trait; either is fine as long as the type-level constraints prove safety.

### 8.3 Design Questions That Need Resolution Before Coding

1. **How are effect rows represented?** The expressive power of row polymorphism versus the ergonomics of the Rust type system is the central tradeoff. Closed tuple-based rows are easy to implement but prevent open-ended programs. HList-style encodings allow openness but make type errors worse. Const-generic strings remain unstable.
2. **Where does the `Functor` dictionary live?** In PureScript, it's carried in `VariantFRep.map`. In Rust with static dispatch, the constraint can be part of the trait bound at injection time; with dynamic dispatch, it becomes a vtable field.
3. **Is the `Free` monad stack-safe via `CatList` or via iterative interpretation?** The first requires an efficient persistent catenable list in Rust (possible, but non-trivial). The second is simpler: every interpreter loops with `while let` / `tailRecM`, and `bind` builds a left-associated tree, accepting the quadratic cost during interpretation but not during construction. Given that Rust programs rarely build multi-million-node effect trees, the iterative approach is probably sufficient.
4. **How much should this align with the repo's existing HKT/Brand system?** The existing `higher-kinded-types.rs` and the optics system show the project has already made design decisions about HKTs. The port should reuse those primitives.
5. **How much type erasure is acceptable?** PureScript uses `unsafeCoerce` liberally. Rust can do the same with `transmute` plus `Box<dyn Any>`, but every such use is a safety audit. A fully-statically-typed alternative is possible but may require more HKT machinery than exists today.

---

## 9. Dependency Summary

From [purescript-run/bower.json](https://github.com/natefaubion/purescript-run/blob/master/bower.json):

- `purescript-aff`, `purescript-effect` (base monad bridges; not needed in Rust).
- `purescript-free`, `purescript-variant` (the two pieces the Rust port must reproduce).
- `purescript-tailrec`, `purescript-type-equality` (infrastructure; both reproducible as Rust traits plus helpers).
- `purescript-unsafe-coerce`, `purescript-newtype`, `purescript-prelude`, `purescript-either`, `purescript-maybe`, `purescript-tuples`, `purescript-profunctor`, `purescript-typelevel-prelude` (standard PureScript surface; most are already mirrored in this repo's `fp-library`). `purescript-newtype` in particular is used only for the `Newtype (Run r a) _` derivation and for `unwrap` calls; it has no Rust counterpart since Rust newtypes need no trait-level machinery.

The `purescript-free` dependency on `purescript-catenable-lists` is the one non-trivial piece that is not obviously present in Rust. A Rust catenable list can be built from two stacks (Okasaki-style persistent deque) or from a rope, if the CatList-based `Free` approach is adopted.

---

## 10. Verified Source References

Every signature and claim in this document is traceable to a specific file and line range in the PureScript sources. The most load-bearing references:

- `Run` newtype: [purescript-run/src/Run.purs:56-93](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs#L56-L93).
- `MonadRec Run` instance: [purescript-run/src/Run.purs:105-112](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs#L105-L112).
- `lift` signature: [purescript-run/src/Run.purs:116-124](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs#L116-L124).
- `runPure` signature: [purescript-run/src/Run.purs:279-292](https://github.com/natefaubion/purescript-run/blob/master/src/Run.purs#L279-L292).
- `VariantF` definition: [purescript-variant/src/Data/Functor/Variant.purs:49-59](https://github.com/natefaubion/purescript-variant/blob/master/src/Data/Functor/Variant.purs#L49-L59).
- `Variant` definition and `VariantRep`: [purescript-variant/src/Data/Variant.purs:45](https://github.com/natefaubion/purescript-variant/blob/master/src/Data/Variant.purs#L45) and [purescript-variant/src/Data/Variant/Internal.purs:44](https://github.com/natefaubion/purescript-variant/blob/master/src/Data/Variant/Internal.purs#L44).
- `Free` definition and bind: [purescript-free/src/Control/Monad/Free.purs:38-72](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs#L38-L72).
- `MonadRec` class and `Effect` instance: [purescript-tailrec/src/Control/Monad/Rec/Class.purs:58-137](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs#L58-L137).
- `TypeEquals` class: [purescript-type-equality/src/Type/Equality.purs:20-24](https://github.com/purescript/purescript-type-equality/blob/master/src/Type/Equality.purs#L20-L24).

---

## 11. Summary

The `Run` library is a composition of three minimal ideas:

1. Use a **row type** to enumerate effects, giving open, compositional union of effects with exhaustiveness checked at the type level.
2. Use an **open sum of functors** (`VariantF`) as the effect representation, so any functor can be lifted into any effect row without prior coordination.
3. Use the **`Free` monad** to provide monadic sequencing over that sum, with stack-safe interpretation via `MonadRec`.

Everything else (the per-effect modules, the interpreter combinators, the base-monad bridges) is a thin ergonomic layer. A Rust port that reproduces these three cores, even in a simplified closed-row form, will deliver most of the value.

The hardest design question for the Rust port is how to represent the row of effects. Every other question (`Free` encoding, `MonadRec` equivalent, functor dictionaries, handler combinators) has a reasonable Rust answer that follows directly from that choice. It should be the first thing decided.
