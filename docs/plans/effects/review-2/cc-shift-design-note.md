# CC/Shift design note

Written for item 19 of the remediation plan (step 1; adoption or rejection
is step 2). Targets heftia-shaped delimited continuations on the FS-1
substrate, per the item's adopted direction: answer type carried in the
effect type, handlers built on captured cloneable continuations, one-shot
`shift` on the `Box` store evaluated as a complementary capability.
References: heftia's `Control.Monad.Hefty.Shift` and data-effects'
`Data.Effect.Shift` (local checkouts under `effects/`), the E5 catalogue
([foundation-sweep/polynomial-exponential-catalogue.md](foundation-sweep/polynomial-exponential-catalogue.md)),
the W13 runtime policy, MpEff's `mpromptIORef` discipline, and
switch-resume's one-shot capture (external-ideas items 2 and 5).

## The reference shape

heftia's `Shift` is a first-order effect whose type carries both the
answer type and the continuation reification:

```haskell
data Shift ans ref :: Effect where
    SubShiftFork :: Shift ans ref f (Either (ref a) a)
    Call :: ref a -> a -> Shift ans ref f ans
    Abort :: ans -> Shift ans ref f a
```

`SubShiftFork` is the capture primitive: it resumes either with the
reified continuation (`Left (ref a)`) or with a resumed value
(`Right a`); `shift` derives from it. Call/cc is not a second mechanism
and carries no answer type of its own: `CC ref` (`SubFork`/`Jump`) leaves
`ans` to emerge only when `ref` pins at handling, and it interprets onto
`Shift` in a few lines (`runCCOnShift`). Prompts with different answer
types coexist as separate, distinctly typed row entries. The delimiter
pins the reification to the residual row and answer type:

```haskell
runShift :: (FOEs es) => (a -> Eff es ans)
         -> Eff (Shift ans (Op (Eff es ans)) ': es) a -> Eff es ans
handleShift = \case
    SubShiftFork -> \exit -> exit . Left . Op $ exit . Right
    Call (Op exit) x -> (exit x >>=)
    Abort ans -> const $ pure ans
```

Three load-bearing facts. First, the captured continuation
(`exit :: x -> Eff es ans`) is an ordinary first-class value, and the
fork handler calls it twice, which is what multi-shot means; this is free
in Haskell and is exactly what a store choice must buy in Rust. Second,
the answer type is monomorphic per prompt: `ans` is a parameter of the
effect type, fixed at the row entry, and heftia offers no answer-type
polymorphism within one delimiter. Third, `runShift` requires the
residual row to be first-order only (`FOEs es`); a delimiter does not
commute with uninterpreted higher-order cells. heftia enforces nothing
else statically: continuations carry no one-shot or linearity
restriction, and the library flags only performance, not soundness, as
the multi-continuation concern.

## The encoding on FS-1

The effect is an ordinary first-order row cell in the established
parameterise-and-pin convention, with the same nominal-row lazy
self-reference every higher-order pin already uses (the reification names
the row that contains the cell, legal through the row brand's kind
projection):

- `ShiftBrand<Ans, V>`: `Ans` is the prompt's answer type, `V` the
  captured value type. The reified continuation is the store's `Fn` form,
  `Stored<'static, V, Free<Row, Ans, Store>>` behind the `Rc`/`Arc`
  stores (`Rc<dyn Fn(V) -> Free<Row, Ans, RcStore>>`), the concrete path
  the W13 record already names.
- Operations: `shift(body: impl FnOnce(Cont<V, Row, Ans, Store>) ->
Program<Ans>) -> V` and `abort(ans: Ans) -> !`, with the fork primitive
  (`sub_shift_fork() -> Either<Cont, V>`) expressible but secondary;
  `Call` is a method on the reified continuation value rather than a row
  operation, which avoids threading `ref` through the row twice.
- The delimiter is a narrowing-tier runner in `run_cont`'s shape (the
  shipped whole-row CPS fold is `interpretBy`'s analogue): `run_shift`
  folds the program at result `Ans`, reifying each `Shift` cell's
  continuation as a store-`Fn` value handed to the arm, re-emitting every
  other cell into the residual row lazily, exactly as the accumulator
  runners do.
- CC derives as a thin layer over `Shift`, ported only if a consumer
  wants the jump vocabulary.

The grammar gap, stated plainly: `SubShiftFork`'s capture type `a` is a
per-use free type variable in Haskell, and a Rust operations enum has no
per-variant generics, so the capture type must either be pinned in the
effect type (the `V` parameter above) or type-erased. The pinned encoding
is recommended per principle 4 (the continuation's type is fully visible,
illegal calls unrepresentable); a prompt needing several capture types
holds several cells under tags (`TaggedBrand<L, ShiftBrand<Ans, V>>`),
the shipped mechanism for same-effect multiplicity. The erased encoding
(`Cont<Erased, ...>` with a checked downcast at the call boundary)
recovers heftia's per-use polymorphism at the cost of a runtime-checked
boundary and is the documented fallback (principle 3) if the pin proves
too restrictive against real consumers.

## What mono-in-A makes impossible, and what the encoding gives up

This section discharges the item's standing requirement.

- Answer-type-polymorphic capture, a single cell serving arbitrary answer
  types per use, is not expressible: the operations enum fixes `Ans` at
  the cell. This matches the reference rather than falling short of it,
  since heftia's `ans` is likewise fixed per prompt, but the prior
  reviews' warning stands as the reason the pin must stay visible in the
  type: the W13 policy's "do not ship a mono-in-`A` compatibility surface
  without that proof" is honoured by carrying `Ans` (and `V`) explicitly,
  never erasing them behind a uniform interface.
- The capture-value type is pinned per cell (`V`), where heftia allows a
  fresh `a` per use. This is the encoding's genuine give-up; tags recover
  multiplicity, the erased fallback recovers full polymorphism at a
  downcast boundary.
- Multi-shot capture exists only on the `Rc`/`Arc` stores, whose queues
  and continuations are `Clone`/`Fn`; on the `Box` store the reified
  continuation is `FnOnce` and `shift` is one-shot by type. This is the
  store axis doing its job, not a defect.
- A delimiter over a residual row holding uninterpreted higher-order
  cells is out of scope, mirroring heftia's `FOEs` bound; the `OrderOf`
  markers give the type-level material to enforce the same restriction.

## Interactions

- Bracket and cancellation: the W13 policy line is restated as binding:
  prompt `Bracket` finalization and multi-shot resumption cannot both
  hold (a release that ran once cannot be un-run when a continuation
  re-enters the bracketed region). The design surfaces this as a
  documented exclusion, not a runtime surprise: multi-shot `shift` and
  `Bracket` do not share a row in the first round.
- Handler state under multi-shot capture: heftia's default is
  invocation-time threading (the state at the moment a continuation is
  re-invoked flows forward; no automatic rollback), with `transactState`
  as the opt-in rollback, and the ported `transact_state` already gives
  this design the same opt-in. Threaded accumulators (the `AccumStep`
  family) compose naturally, forking the accumulator per re-entry
  exactly as `handle_choose_accum` forks per branch; shared-by-reference
  cells are global across re-entries unless the handler adopts MpEff's
  snapshot-and-restore discipline (`mpromptIORef`), which becomes the
  documented pattern for cell-backed handlers that need per-entry
  isolation.
- One-shot `shift` on the `Box` store needs none of the multi-shot
  machinery and no async coupling: programs are already data, so a
  one-shot delimiter reifies the `FnOnce` continuation directly.
  switch-resume's async framing (the continuation as the rest of the
  `run_async` future) is thereby subsumed rather than adopted: capturing
  the Rust async stack is inherently one-shot and adds nothing over
  capturing the `Free` spine, which the W13 research already concluded.

## Staged implementation plan (on adoption)

1. One-shot `shift` on the `Box` store, proof of concept first: the
   `ShiftBrand<Ans, V>` cell, the `run_shift` delimiter as a narrowing
   fold, and an oracle exercising capture, abort, and resume-once,
   validating the encoding with zero new substrate machinery. Fallback
   per the standard discipline: if the pinned `V` blocks the oracle, the
   erased-capture boundary; surface if both fail.
2. Multi-shot `shift` on the `Rc`/`Arc` stores, inside the multi-shot
   interpretation round: the runner tier generalised over the store
   parameter, the reified continuation as the store's `Fn` form, and the
   MpEff snapshot discipline documented. The item 17 (`alt`) and OQ-18B
   (async) revisits ride the same round, and item 17's re-expression is
   literally upstream's derivation: heftia runs NonDet on `Shift` by
   calling the captured continuation once per branch, so multi-shot
   `shift` is the primitive `alt` falls out of.
3. CC as the derived jump vocabulary, on demand.

Implementation remains out of the remediation plan's scope until this
note is adopted (item 19 step 2).
