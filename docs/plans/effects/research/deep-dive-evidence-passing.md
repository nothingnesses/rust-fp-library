# Deep dive: evidence-passing dispatch in Rust

**Status:** complete
**Last updated:** 2026-04-24

## 1. Purpose and scope

Stage 1 flagged EvEff ([eveff.md](eveff.md)) and Koka
([koka.md](koka.md)) as genuinely novel relative to the five row
encodings in port-plan section 4.1. The synthesis
([\_classification.md](_classification.md), section 5.1) queued this
deep dive to answer one question: can Rust host a typed handler-vector
dispatch, as EvEff and Koka do in Haskell, that is ergonomic and
competitive with the nested-coproduct options 1, 2, and 4, without
requiring delimited continuations?

Scope: classify the dispatch mechanism (handler-vector lookup by
type identity) separately from the control substrate (multi-prompt
continuations). The control substrate is ruled out by port-plan
section 1.2. The dispatch mechanism is the candidate for a sixth
option or a refinement of an existing one. This dive reads EvEff's
`Context e` GADT and Koka's `OpenResolve` pass against the
fp-library Brand/Kind machinery and decides which, if either,
survives the translation.

## 2. Detailed mechanism

### 2.1 EvEff: runtime linear scan with compile-time type equality

EvEff represents an effect context as a GADT-linked list of handler
values rather than a coproduct of effect operations:

```haskell
-- src/Control/Ev/Eff.hs:121-126
data (h :: * -> * -> *) :* e
data Context e where
  CCons :: !(Marker ans) -> !(h e' ans) -> !(ContextT e e') -> !(Context e) -> Context (h :* e)
  CNil  :: Context ()
```

Each `CCons` node holds four runtime values: a `Marker` (prompt
identifier, used by the `Ctl` control monad only), a handler record
`h e' ans` typed against the type-level head of the row, a context
transformer, and the tail context. The `Eff` monad itself is a
continuation:

```haskell
-- src/Control/Ev/Eff.hs:154
newtype Eff e a = Eff (Context e -> Ctl a)
```

Membership is proved by a pair of classes using a closed type family
for type equality:

```haskell
-- src/Control/Ev/Eff.hs:256-275
class In h e where
  subContext :: Context e -> SubContext h

instance (InEq (HEqual h h') h h' w) => In h (h' :* w) where
  subContext = subContextEq

type family HEqual (h :: * -> * -> *) (h' :: * -> * -> *) :: Bool where
  HEqual h h  = 'True
  HEqual h h' = 'False

class (iseq ~ HEqual h h') => InEq iseq h h' e where
  subContextEq :: Context (h' :* e) -> SubContext h

instance (h ~ h') => InEq 'True h h' e where
  subContextEq ctx = SubContext ctx

instance ('False ~ HEqual h h', In h e) => InEq 'False h h' e where
  subContextEq ctx = subContext (ctail ctx)
```

`HEqual` is a closed type family: at instance-resolution time, GHC
checks whether the head `h'` of the context equals the desired `h`;
if so, the `'True` instance terminates the search; otherwise the
`'False` instance recurses on the tail. Dispatch proceeds at runtime
as a linear scan over the cons list, but the path through the list
is compile-time determined by the type-family reduction. `perform`
then extracts the right handler and calls its `Op`:

```haskell
-- src/Control/Ev/Eff.hs:312-315
perform :: (h :? e) => (forall e' ans. h e' ans -> Op a b e' ans) -> a -> Eff e b
perform selectOp x
  = withSubContext (\(SubContext (CCons m h g ctx)) -> applyOp (selectOp h) m (applyT g ctx) x)
```

Dispatch cost per operation: a linear walk through the context, N
pointer dereferences for N stacked handlers. Dispatch correctness is
proved statically by GHC via `HEqual`. Operation handlers are
ordinary Haskell records of function fields, stored as values (not
as constructors of a sum type) at each `CCons` node.

### 2.2 Koka: compile-time evidence-index linearisation

Koka addresses the same "handler-vector" semantics at compile time
rather than at runtime. The `OpenResolve` pass in
`src/Core/OpenResolve.hs:110-204` walks every effect-crossing
expression, extracts the list of _handled_ effects from the `from`
and `to` effect types via `extractHandledEffect`, and inserts an
`open` wrapper that carries the evidence indices as a vector:

```haskell
-- src/Core/OpenResolve.hs:164-170
evIndexOf l = let (htagTp, hndTp) = ...
              in App (makeTypeApp (resolve nameEvvIndex) [effTo, hndTp]) [htagTp]

-- src/Core/OpenResolve.hs:199
in let indices = makeVector typeEvIndex (map evIndexOfMask (addLevels lsFrom))
   in if (n <= 3) then wrapper (resolve (nameOpen n)) [indices]
                  else wrapperThunk (resolve (nameOpen 0)) [indices]
```

`evIndexOf` computes a stable index for each effect label; the row
is erased at runtime and replaced with a flat vector of indices
(`KK_TAG_EVV_VECTOR` in `kklib/include/kklib.h`). Duplicate labels
(the same effect handled twice, for `handlerLocal`-style scoped
state) get mask levels via `addLevels`
(`src/Core/OpenResolve.hs:208-218`), which computes a level for each
label based on how many identical labels precede it. The runtime can
then distinguish "the `State<Int>` at level 0" from "the `State<Int>`
at level 1" without any runtime type inspection.

The dispatch path at runtime is therefore an array lookup by index,
not a linear scan. EvEff's O(n) per-operation cost becomes O(1) at
the cost of running the `OpenResolve` pass on every effect-crossing
call site at compile time.

## 3. Rust portability assessment

The fp-library HKT machinery
([`fp-library/src/kinds.rs`](../../../../fp-library/src/kinds.rs),
[`fp-library/src/brands.rs`](../../../../fp-library/src/brands.rs))
provides a stable mechanical scheme: every brand is a zero-sized
`PhantomData`-wrapper struct (e.g., `struct OptionBrand`), and each
`Kind` trait generated by the `trait_kind!` macro maps a brand plus
type arguments to a concrete type via an associated type `Of<...>`.
A typed `Context` over the fp-library HKT system would look roughly
like:

```rust
// sketch, not production code
struct ContextCons<H: HandlerBrand, E: Context> {
    handler: <H as Kind_2>::Of<'static, Tail, Ans>,
    tail: E,
}
struct ContextNil;

trait In<H: HandlerBrand, E: Context> {
    fn sub_context(ctx: &E) -> &SubContextOf<H>;
}
```

Three questions determine whether this is a genuine sixth encoding
option or a recombination of existing ones.

### 3.1 What replaces `HEqual`?

Rust has no closed type family analogue. There is no built-in way
to write "if `H == H'` then found else recurse" as a trait-instance
cascade. Three approximations exist, each of which collapses the
design back into one of the plan's existing options:

1. **Peano-indexed dispatch.** The `In<H, E>` trait carries an
   index parameter `Idx` that the user (or inference) supplies as
   `Here`, `There<Here>`, `There<There<Here>>`, and so on. This is
   exactly Option 1. The compile-time search EvEff does via
   `HEqual` is shifted to the frunk-style instance chain.
2. **`TypeId` runtime comparison.** Each handler is tagged at
   construction with its `TypeId`, and dispatch does runtime
   comparison on the cons list. This is Option 3 with an added
   type-level row for static exhaustiveness.
3. **`min_specialization`.** On nightly Rust, specialization allows
   "if the head matches, terminate, else recurse" to be expressed
   as overlapping impls. This is closest to the Haskell mechanism
   but ties the port to nightly and is specifically flagged as a
   brittle feature.

The Haskell "evidence passing is novel relative to coproduct"
argument depends entirely on `HEqual`; in Rust, removing `HEqual`
removes the distinction. The runtime shape (linked list of handler
values) can stay, but the dispatch mechanism necessarily becomes
Option 1 or Option 3 in disguise.

### 3.2 What survives the translation?

The runtime handler-as-value shape survives with two concrete
benefits over a coproduct baseline: (a) handlers are addressed by
position in the context, so adding a new handler does not change
the type of existing handler records, and (b) operations dispatch
to a direct function-pointer call on the handler record, avoiding
the pattern-match branch structure of coproduct dispatch. In
practice this is a wash: Rust's LLVM-backed inlining flattens both
patterns, and the coproduct match is usually a single jump-table.
No compile-time evidence suggests the handler-as-value shape is
faster in Rust; Haskell's box-per-node heap representation makes
the linked-list form cheap there, but Rust pays an allocation cost
per `CCons`-equivalent node unless the entire context is
stack-allocated via GATs.

### 3.3 What about Koka's compile-time indexing?

Koka's approach is different: it does not require a type-level
equality test because it resolves dispatch entirely at compile
time. A Rust proc-macro (sibling of corophage's `Effects![...]`)
could assign each effect in a row a stable integer index by
lexical sort order at expansion time, emit a const `[usize; N]`
table, and generate code that reads the handler from an array slot
rather than walking a list. This is a genuine implementation-level
improvement: O(1) dispatch, no runtime type inspection, no
nightly features. It is a refinement of Option 4, not a new
encoding.

Blockers for a Rust simulation:

- **Heap allocation per context node** unless the context is built
  on a stack-allocated tuple with GATs. Acceptable cost.
- **Brand erasure.** The fp-library `Of<'a, A>` kind introduces a
  lifetime parameter that requires any type parameter baked into a
  brand to outlive `'a`, effectively meaning `'static`. Non-
  `'static` handlers (capturing local state) would need the same
  treatment corophage uses for non-`'static` effects, namely a
  lifetime-parameterised brand. No new blocker specific to this
  design.
- **Type-error quality.** An `In<H, E>` trait with an inferred
  index parameter produces frunk-class error messages. No worse
  than Option 1; no better.

## 4. Comparison against the port-plan's five options

The dispatch mechanism that EvEff calls "evidence passing" does
not, in Rust, constitute a sixth row encoding. Once `HEqual` is
removed from the picture, the remaining design space is spanned by
the existing options:

- A `Context` cons-list with Peano `In<H, E, Idx>` dispatch is
  Option 1 with handler values instead of effect-operation values.
  Row shape identical; substrate different in name only.
- A `Context` cons-list with `TypeId`-tagged handlers is Option 3
  with an added static type-level row. Dispatch overhead same.
- A macro that emits stable compile-time indices over a flat row
  is a refinement of Option 4.

Koka's compile-time indexing is the one genuinely new idea: it
shows that the macro layer in Option 4 can take on more work than
corophage's macro currently does. Specifically, the macro can emit
a const index table alongside the coproduct type, and handler
dispatch can bypass the coproduct pattern-match in favour of an
array index. This saves a branch per dispatch and produces a fixed
runtime representation regardless of the type-level row ordering.

Conclusion: **no sixth option is warranted.** The evidence-passing
contribution to the port should be recorded as a refinement of
Option 4, not as a separate encoding.

## 5. Concrete plan-edit recommendations

This deep dive recommends three specific edits to
`../port-plan.md` section 4.1.

**5.1 Close evidence passing as a candidate encoding.** Section
4.1 should explicitly note that evidence passing, as implemented
by EvEff and MpEff, was evaluated in Stage 2 and does not
constitute a sixth row encoding for Rust. The dispatch mechanism
in Haskell depends on the `HEqual` closed type family; Rust has no
analogue outside of unstable specialization. A Rust simulation
reduces to Option 1 (Peano index) or Option 3 (`TypeId`) depending
on which substitute is chosen.

**5.2 Refine Option 4.** The Option 4 description should add a
note that a sufficiently rich proc-macro can, in addition to
expanding the coproduct, emit a const `[usize; N]` index table
assigning each effect a stable integer index by lexical sort
order. Dispatch through this table is O(1) regardless of row
size, matches Koka's compile-time index strategy, and does not
require runtime `TypeId` inspection. This is additive to the
existing macro layer; corophage's existing `Effects![...]` does
not currently do this, so the note should frame it as an
achievable optimisation rather than a present feature.

**5.3 Note mask levels for duplicate effects.** If the port ever
supports `handlerLocal`-style duplicate handlers for the same
effect (for example, scoped state with nested `local`), it must
implement the equivalent of Koka's `addLevels`
(`src/Core/OpenResolve.hs:208-218`) to distinguish the nested
handlers. This is not an encoding question but an implementation
detail that matters when multiple entries in a row share the same
effect type. The port plan does not currently address this; the
issue should be raised under section 4.1 or section 4.2 once
scoped operations are designed.

Stage 2 follow-ups worth considering:

- `deep-dive-compile-time-indexing.md`: prototype the
  index-emitting proc-macro described in 5.2 on top of corophage's
  existing `Effects![...]` expansion. Measure compile-time cost
  and dispatch overhead against a coproduct-pattern-match
  baseline.

The priority-2 and priority-3 deep dives from
[\_classification.md](_classification.md) (coroutine-vs-Free and
scoped-effect ergonomics) remain on the Stage 2 queue unchanged.

## 6. References

- `src/Control/Ev/Eff.hs:121-126`: `Context` GADT and `:*` context
  cons.
- `src/Control/Ev/Eff.hs:154`: `Eff` continuation-style monad.
- `src/Control/Ev/Eff.hs:256-275`: `In h e` class, `HEqual` type
  family, `InEq` dispatch instances.
- `src/Control/Ev/Eff.hs:312-315`: `perform` dispatch via
  `withSubContext`.
- `src/Core/OpenResolve.hs:110-204`: `resOpen` pass; inserts
  `open` wrappers with evidence-index vectors.
- `src/Core/OpenResolve.hs:164-170`: `evIndexOf` computing a
  compile-time index for each effect label.
- `src/Core/OpenResolve.hs:208-218`: `addLevels` mask-level
  computation for duplicate effects.
- `kklib/include/kklib.h:72,442`: `KK_TAG_EVV_VECTOR` runtime
  evidence-vector representation.
- `fp-library/src/kinds.rs:15-53`: fp-library `Kind` trait
  variants via `trait_kind!` macro.
- `fp-library/src/brands.rs:39-394`: Brand struct definitions and
  HKT encoding conventions.
- [eveff.md](eveff.md): Stage 1 classification of EvEff.
- [koka.md](koka.md): Stage 1 classification of Koka.
- [\_classification.md](_classification.md), section 5.1: Stage 2
  question this dive answered.
