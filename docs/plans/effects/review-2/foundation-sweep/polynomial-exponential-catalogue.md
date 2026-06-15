# Polynomial versus Exponential Effect Catalogue (E5 handoff)

This is the execution step E5 deliverable: a catalogue of the `data-effects` effect set split into the cases the foundation sweep scoped (first-order and polynomial higher-order) and the cases it deferred (exponential higher-order). It bounds the later exponential design round; it is not part of the FS-1 work this sweep validated.

## Criterion

From `data-effects-core/src/Data/Effect.hs`: each effect has an order (`OrderOf`: `FirstOrder` or `HigherOrder`) and, for higher-order effects, a form (`FormOf`: `Polynomial` or `Exponential`). In the effect modules the order shows up in the generator macro (`makeEffectF` for first-order, `makeEffectH` for higher-order). The form is structural: an effect is polynomial when the carrier `f` appears only in covariant (positive) positions (it wraps sub-computations), and exponential when the carrier appears in a function or otherwise negative position (it captures continuations or a run function). The sweep scoped first-order and polynomial higher-order effects; exponential higher-order effects (`PolyHFunctor` does not hold) are the later round.

## First-order effects (no carrier; fully in scope)

These have no sub-computation and are the common case the unified row plus brand-keyed dispatch handles directly: `Throw` (Except), `State`, `Reader`'s `Ask`, `Writer`'s `Tell`, `NonDet` (`Choose`/`Empty`), `Fresh`, `Input`, `Output`, `Fail`, `KVStore`, `Accum`, `Log`. (Generated with `makeEffectF`.)

## Polynomial higher-order effects (carrier covariant; in scope, elaboration or weave)

These wrap sub-computations in covariant positions and are the cases the sweep validated by elaboration (POC-4/POC-5/POC-9) or, failing that, weave: `Catch` (Except, `makeEffectH`), `Local` (Reader), `WriterH`'s `Listen` and `Censor` (Writer), `Bracket`/resource lifecycles, `Provider`. These are the FS-1 higher-order target; `PolyHFunctor` holds, so resources cannot escape the scope.

## Exponential higher-order effects (carrier in negative position; deferred to the later round)

These put the carrier in a function or negative position, so they capture continuations or a run function and are out of scope for this sweep; they need the later exponential design round (and likely the heftia `Shift`/`CC` continuation machinery, the W8/W13 reference):

- `Unlift` (`UnliftBase`/`WithRunInBase`): the constructor holds `f` taking a run function (`m ~> base`), the carrier in negative position.
- `CC` (call/cc) and `Shift` (delimited continuations): capture the continuation (`Shift ans ref` with `Call`/`Abort`), the defining exponential case.
- `Select` (backtracking search) and continuation-style `Coroutine`: continuation-capturing, exponential.

## Bearing on the sweep

FS-1 (the adopted design) covers the first-order and polynomial higher-order effects, which is the whole catalogue except the exponential group above. The exponential effects are explicitly the later design round; this catalogue is their starting scope. Where an effect's exact `FormOf` is ambiguous from its shape (for example whether a given `Coroutine` or `Provider` variant is polynomial or exponential), the later round should confirm it against the effect's `FormOf` instance in `data-effects` before designing its mechanism.
