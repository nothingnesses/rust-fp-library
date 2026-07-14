# Async runtime-policy extension

Written for item 18 of the remediation plan, extending the adopted W13
policy record on the shipped FS-1 async surface (the OQ-18A terminal
composition; the OQ-18B single-shot-only decision). W13 established the
baseline: a continuation-as-data async driver, a runtime-agnostic public
surface, and no `MonadRec`-over-`Future`. This record adds the per-effect
eligibility criteria for the deferred runtime-sensitive effects and
settles the `Io` question's current round.

## Eligibility criteria

A runtime-sensitive effect is eligible for the polynomial async family
when all four criteria hold:

1. **Programs-as-data.** The effect is an ordinary row cell interpreted
   by handlers or lowered to the async base; it changes no substrate type
   and no driver loop.
2. **Runtime-agnostic constructors.** No constructor names a runtime.
   Binding to a concrete scheduler, clock, or spawner happens handler
   side, in arms, steps, or the future passed to `await_future`, so the
   public surface stays executor-neutral.
3. **Single async base.** The effect suspends only by lowering to the one
   `Await` cell; no second suspension primitive enters the row.
4. **Polynomial shape.** The effect neither captures nor re-enters
   interpretation contexts; effects that do are the E5-catalogued
   exponentials and belong to item 19's round.

## Per-effect verdicts

- **Timer**: eligible, and already expressible without a dedicated
  effect, since `await_future` embeds any runtime's sleep future
  directly. A dedicated `Timer` cell earns its place only when a consumer
  needs interposable or mockable time; it is then an ordinary first-order
  effect whose handler supplies the clock, passing all four criteria.
- **Subprocess**: eligible on the same shape (spawn-and-wait lowers to a
  future; the handler supplies the spawner), with the same trigger: a
  concrete consumer, not speculative surface.
- **Parallel**: not eligible in this family. Concurrency inside one cell
  is already free, since one embedded future may internally join or race
  whatever it likes; what a `Parallel` effect would add is racing or
  joining programs (each with its own effects), and that requires the
  driver to hold multiple pending futures at once, a driver capability
  rather than a row cell, with the interleaving decision being scheduling
  rather than row semantics. Deferred behind its own design note when a
  consumer appears.
- **Unlift**: exponential per the E5 catalogue (it captures the
  interpreter as a value), so it fails criterion 4 and belongs to item
  19's round.
- **Provider**: exponential likewise (scoped provision of interpretation
  contexts); item 19's round.
- **Io** (a general base-lift for synchronous effects): rejected for this
  round. The captured-cell idiom remains the blessed mechanism: handler
  arms and steps are arbitrary Rust closures over captured resources,
  which is the shape every guide example already uses, so a general `Io`
  cell would duplicate what arms provide while weakening interposability
  (an opaque `Io` cell cannot be selected by brand the way a named effect
  can). Revisit together with `Unlift` at item 19, where "run this IO
  with the interpreter in hand" becomes expressible and the trade
  changes.
