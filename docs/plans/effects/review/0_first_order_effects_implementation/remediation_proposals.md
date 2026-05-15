# Remediation Proposals for effects.rs Review

## Summary

The review filed **5 fundamental, 8 major, and 9 minor findings**.
Clustered by root cause, those 13 non-minor findings collapse into
**4 root-cause clusters**: (i) the mono-in-A interpreter shape with
no `M` in the return type (drives F1, F5, M6 and partially F2);
(ii) the structural absence of a callable continuation primitive
(F2, F4, partially M3); (iii) the unimplemented scoped-row dispatch
trait (F3, M8); (iv) per-effect cost and HRTB-over-types friction
(M2, M4, M5, M7). The single highest-leverage remediation is
**deleting the `run_accum` family entirely** (F1, since after
dropping `init` it would be a literal alias for `interpret`) plus
**delivering Phase 4 scoped dispatch via a parallel
`DispatchScopedHandlers` trait** (F3, M8); together they convert the
two largest API lies into either honest local APIs or honest
"deferred to Phase X" stubs. **One finding has no clean fix
in the current encoding: F2 (no callable continuation primitive).**
The honest answer is to accept that this is a freer-monad
encoding, not a delimited-continuation handler system, and to
revise plan claims accordingly.

## Fundamental Findings

### F1. `run_accum` and `run_accum_rec` ignore the `init` argument across all six wrappers

**Restated:** Every `run_accum<St>(handlers, init: St)` body across
the six wrappers reduces to `let _ = init; self.interpret(handlers)`.
The function accepts an initial state value, drops it, and delegates
to `interpret`. The doctests show users threading state via
captured `Rc<RefCell<S>>` cells, not via `init`. The function
signature is therefore a contract the implementation cannot keep.

**Root cause:** The mono-in-A handler dispatch returns `A`, with no
`M` in the return type. The interpreter has no slot to expose
threaded state to the caller. The plan
[resolutions.md Q3](../../resolutions.md#L360) confirmed
"closure-capture" as the chosen state-threading model and explicitly
deferred StateT-as-target to Phase 6+. Once that decision was
locked, `init` had no role inside the function but remained in the
signature for "PureScript Run parity".

**Option A: Drop the `init` parameter from the signature.**

- _What:_ Remove the `init: St` parameter and the `St` type
  parameter from `run_accum` / `run_accum_rec` on all six wrappers.
  Touches the 12 method signatures listed in the review report
  (e.g.,
  [run.rs:653-666](../../../../../fp-library/src/types/effects/run.rs#L653-L666)),
  the 12 doctests, and the brief mention in
  [plan.md "Out of scope" / decisions row](../../plan.md#L1207).
- _Cost:_ Small (a session). API breakage is in-scope (no public
  release yet per
  [plan.md API stability stance](../../plan.md#L1011)). No type-inference
  impact (removing a parameter strictly relaxes inference). One
  resolution-doc revision: amend
  [resolutions.md Q3](../../resolutions.md#L360) to record that the
  `init` parameter was vestigial and is now removed; the
  closure-capture pattern remains.
- _Benefit:_ The headline `run_accum` / `run_accum_rec` API stops
  lying. Users who wire `Rc<RefCell<S>>` see the function shape that
  matches their usage. Resolves F1 cleanly. Does not interact with
  any other finding except cosmetically.
- _Risks:_ The remaining function name (`run_accum`) is still
  somewhat misleading; without state threading at all, it is just
  an alias for `interpret`. See Option C for the alternative
  rename.

**Option B: Make `run_accum` actually thread state by interposing a
state-injection adapter.**

- _What:_ Implement a small adapter inside the body that captures
  `init` in a fresh `Rc<RefCell<St>>`, wraps each handler with a
  closure that has access to that cell, and reconstructs the
  handler list. Final result becomes `(St, A)` (matching PureScript
  Run's `runState`). Touches the 12 wrapper bodies plus a new
  helper module. Estimated 200-400 lines.
- _Cost:_ Medium. Requires the wrapping closure to dynamically
  re-tag handlers, which means handler closures must accept a
  reference to the state cell parameter (a new convention). Type
  inference suffers: every handler closure now needs an extra
  reference parameter, which Rust will not infer from the macro's
  emitted shape today. Existing tests using closure-capture state
  break (they would double-thread).
- _Benefit:_ The function name regains its meaning. PureScript Run
  parity at the API level. Does not require StateT.
- _Risks:_ The "wrap each handler" step has to walk the handler
  list at type-level, which is exactly the trait machinery
  [resolutions.md Q3](../../resolutions.md#L386) called "doubles the
  trait machinery" and rejected. Reproduces a rejected design.

**Option C: Rename `run_accum` to `run_with_cell` (or similar) and
keep `init` removed.**

- _What:_ Same as Option A but with a function rename to make the
  closure-capture convention explicit at the call site. Touches
  the same files plus call-site renames in tests and doctests.
- _Cost:_ Same as Option A plus the rename. Loses literal naming
  parity with PureScript Run (`runAccum`). Touches ~30 call sites
  (12 doctests + tests in
  [run_handle.rs](../../../../../fp-library/tests/run_handle.rs)
  and similar).
- _Benefit:_ Clearer API. The name advertises the closure-capture
  convention. Forecloses future confusion.
- _Risks:_ Departs from PureScript Run naming, which the project
  consistently honours elsewhere. May invite a future maintainer
  to "restore the original name" without understanding why it was
  renamed.

**Option D: Delete `run_accum` and `run_accum_rec` entirely; keep
only `interpret` / `interpret_rec`.**

- _What:_ Remove all 12 method definitions across the six wrappers,
  delete the corresponding doctests, and add a doctest to
  [`interpret`](../../../../../fp-library/src/types/effects/run.rs#L497)
  showing the closure-capture state pattern (the same pattern the
  current `run_accum` doctests show, just attached to `interpret`
  instead of to a function whose name implies state). Touches the
  same files as Option A minus the surviving signatures. Roughly
  the same effort as Option A; net deletes more lines than it adds.
- _Cost:_ Small (a session). API breakage is in-scope per
  [plan.md "API stability stance"](../../plan.md#L1011). Loses the
  PureScript Run name `runAccum` from the public surface; the
  closure-capture pattern is documented on `interpret` instead.
  Same plan revisions as Option A plus removing the `runAccum`
  bullet from
  [plan.md "Implementation phasing" line 1131](../../plan.md#L1131).
- _Benefit:_ Smallest possible API surface for the same
  capability. Two names for one operation is strictly worse than
  one. Future-proof: when StateT lands in Phase 6+, the new
  state-threading entry point gets a name that actually means
  state threading (e.g., `interpret_state` or
  `interpret_with::<StateT<S, IdBrand>>`), uncontaminated by the
  vestigial `run_accum` slot. Closes F1 without leaving an alias
  behind.
- _Risks:_ Loses the PureScript Run naming hook. A user who comes
  from PureScript and types `prog.run_accum(...)` gets a
  "method not found" error rather than a friendly redirect; the
  rustdoc on `interpret` should mention the closure-capture
  pattern by the keyword "state" so the rustdoc search picks it
  up.

**Recommendation: Option D.** After dropping `init`, `run_accum`
and `interpret` are the same function with two names; same for
`run_accum_rec` and `interpret_rec`. Two names for one operation
is a strict regression in API clarity. PureScript Run parity is a
naming convention, not a constraint, and the project has departed
from PureScript naming before when the Rust shape diverges (e.g.,
`im_do!` for inherent monadic do, with no PureScript analogue).
The forward-compatibility argument for keeping the slot is weak:
when StateT lands in Phase 6+, the natural entry point is
`interpret_rec::<StateT<S, IdBrand>>`, not a re-purposed
`run_accum`. Drop the function; document the closure-capture
pattern on `interpret`'s rustdoc with a "state" keyword for
discoverability.

**Dependencies and ordering:** Standalone. Can land before any
other Phase 3 / Phase 4 work. Should land before Phase 4 begins
so that scoped-effect handlers do not inherit the same parameter
mistake.

**Plan revision required:** Yes (small).
[Resolutions.md Q3](../../resolutions.md#L360)'s text needs updating
to "state threading is via user-side closure captures applied to
`interpret` / `interpret_rec`; no separate `run_accum` API". The
[plan.md "Implementation phasing" line 1131](../../plan.md#L1131)
mention of `runAccum` should be removed; the
[Phase 6+ deferred entry](../../plan.md#L2325) for "state-via-StateT"
should explicitly say the future entry point will be a new
function name, not a revival of `run_accum`.

### F2. Mono-in-A handler dispatch has no callable continuation

**Restated:** A handler clause receives the lowered effect layer
(e.g., `State<'_, P, S, NextProgram>` containing
`dyn Fn(S) -> NextProgram`), not a uniform callable continuation
`k : x -> Result`. The "continuation" is per-effect-type, embedded
in each effect constructor. There is no `resume(x)` primitive.

**Root cause:** This is a structural property of the freer-monad
encoding: the program is an AST, the handler folds the AST, and
"the rest of the program" is a sub-AST inside each operation
constructor, not a delimited continuation. Stable Rust has no
delimited continuations (no `prompt#` / `control0#`). The
[decisions.md section 1.2](../../decisions.md) explicitly rules out
Hasura-`eff`-style delimited continuations as not portable.

**Option A: Accept the limitation; document it in plan.md and
rustdoc.**

- _What:_ Add a "Continuations" section to
  [plan.md "Out of scope"](../../plan.md#L1287) stating that this is a
  freer-monad encoding, that handlers receive folded sub-programs
  not callable continuations `k`, and that callers wanting
  multi-shot resumption work through the per-effect closure (e.g.,
  `dyn Fn(Bool) -> NextProgram` in a `Choose` effect). Add a
  rustdoc paragraph to
  [interpreter.rs](../../../../../fp-library/src/types/effects/interpreter.rs)
  with the same framing. Touches two files.
- _Cost:_ Negligible. Documentation only.
- _Benefit:_ Honest with rubric Section 3 / Section 6.1 #4 readers
  about what this system is and is not. Removes the silent
  expectation that this is an algebraic-effects-with-handlers
  system in the Plotkin-Pretnar sense.
- _Risks:_ Some users may abandon the library on reading the
  disclaimer. The honest framing is the right tradeoff.

**Option B: Add a `resume` helper trait per effect type that
exposes the embedded closure as `k`.**

- _What:_ Define a `Resumable<NextProgram>` trait with method
  `fn resume(self, x: Self::Input) -> NextProgram` and implement
  it for every effect type that embeds a closure. Handlers can
  call `state_op.resume(current_state)` instead of pattern-matching
  on `State::Get(k)` and calling `(*k)(s)`. Touches every effect
  type's module (today only [state.rs](../../../../../fp-library/src/types/effects/state.rs);
  Phase 3 step 5 adds Reader, Except, Writer, Choose).
- _Cost:_ Small to medium. Per-effect trait impl plus one trait
  declaration. No type-inference impact (handler bodies still call
  `dispatch` the same way; the new `.resume()` is just sugar over
  the embedded closure). Adds one trait to the public API.
- _Benefit:_ Cosmetic uniformity inside handler bodies. Does not
  change the underlying encoding.
- _Risks:_ Effects with multiple "continuation slots" (e.g.,
  `Choose` with separate `true`/`false` branches) have no obvious
  single `resume`; the trait would need an associated type or
  multiple methods. The cosmetic benefit may not justify the
  per-effect boilerplate.

**Option C (rejected): Switch to evidence-passing or delimited-continuation
encoding.**

- _What:_ Replace the freer-monad encoding with EvEff-style
  evidence passing or with `switch-resume`-based delimited
  continuations.
- _Cost:_ Massive. Total rewrite.
- _Benefit:_ Real callable continuations.
- _Risks:_ Plan
  [decisions.md sections 1.2, 4.5](../../decisions.md) and
  [plan.md "Out of scope" line 1287](../../plan.md#L1287) explicitly
  rule out evidence-passing and delimited continuations on stable
  Rust. Reproducing a rejected approach. Listed only for
  completeness.

**Recommendation: Option A.** Accept the limitation and document
it. The freer-monad encoding was chosen with eyes open per
[decisions.md section 1.2](../../decisions.md); the rubric judges this
as a real loss but the loss was the price of stable-Rust
portability. Option B is a small ergonomic sweetener that can land
later if user feedback asks for it; not load-bearing now. Option C
is foreclosed by prior decisions. The right move is honesty in the
docs, not a fix.

**Dependencies and ordering:** Standalone. Should land alongside
F5 (also a mono-in-A consequence) so the documentation reads
coherently.

**Plan revision required:** Yes (small).
[Plan.md "Out of scope" line 1287](../../plan.md#L1287) should grow
a "Callable continuations in handler clauses (Plotkin-Pretnar `k`)"
bullet, with a one-paragraph reason citing the freer-monad encoding
and the absence of stable-Rust delimited continuations.

### F3. Scoped effects are reserved structurally but unimplemented and panic at runtime

**Restated:** The dual-row architecture is wired (a `Node::Scoped`
arm exists, the brand machinery routes through `S`), but no
scoped-effect constructors exist and every interpreter contains an
`unreachable!("... scoped effects ship in Phase 4")` arm. The
scoped row is currently always `CNilBrand`.

**Root cause:** Phase 4 has not started. The `DispatchHandlers`
trait's closure shape
`Fn(EBrand::Of<NextProgram>) -> NextProgram` cannot host scoped
operations whose argument is itself a `Run<R, S, A>`; scoped
dispatch needs a parallel trait whose closure takes a sub-program
and returns a program in the same row.

**Option A: Convert the `unreachable!` into a typed compile-time
constraint by tightening interpret bounds to `S = CNilBrand`.**

- _What:_ Today
  [run.rs:497-519](../../../../../fp-library/src/types/effects/run.rs#L497-L519)
  takes any `S: Kind + WrapDrop + Functor + 'static`. Tighten to
  `where S = CNilBrand` (or via a `ScopedRowEmpty` marker trait).
  The `Node::Scoped(_)` arm becomes
  `Node::Scoped(cnil) => match cnil {}` (statically uninhabited).
  Touches the 6 `interpret` bodies, the 6 `interpret_with` bodies,
  and the 6 `interpret_rec` bodies (~18 sites).
- _Cost:_ Small (a session). One new marker trait or a hard-coded
  `S = CNilBrand` bound. Removes `unreachable!` panics. Does not
  block Phase 4: a separate `interpret_scoped` family lands when
  scoped effects ship.
- _Benefit:_ Eliminates a runtime panic that is only "unreachable"
  per documentation, not per type system. Resolves F3's
  runtime-trap aspect immediately. Reduces clippy suppressions.
- _Risks:_ Users currently writing
  `Run<R, NonEmptyScopedRow, A>` and calling `interpret` would now
  fail to compile. There are no such users (no scoped constructors
  exist), so the breakage is theoretical.

**Option B: Ship Phase 4 (heftia-style scoped dispatch) before
removing the `unreachable!`.**

- _What:_ Per
  [plan.md "Phase 4: Scoped effects" line 2096](../../plan.md#L2096),
  build the `Catch`, `Local`, `Bracket`, `Span` constructors and a
  parallel `DispatchScopedHandlers` trait. The interpreter's
  `Node::Scoped(layer) => scoped_handlers.dispatch(layer)` arm
  replaces the panic.
- _Cost:_ Large (multi-week). Whole Phase 4 surface area.
- _Benefit:_ Resolves F3 fully and fulfils the plan's Phase 4
  Success criteria. Resolves M8 by giving `S` a real role.
- _Risks:_ Phase 4 has known design uncertainty (heftia's row
  architecture diverges from fp-library's per
  [resolutions.md "Heftia row architecture clarification"](../../resolutions.md#L626));
  rushing scoped dispatch to fix a runtime panic is the wrong
  motivation.

**Option C: Combine A and B sequentially: tighten now, deliver
Phase 4 later.**

- _What:_ Land Option A immediately to remove the panic; defer
  Option B to Phase 4. When Phase 4 ships, relax the bound and
  add a `interpret_scoped` family.
- _Cost:_ Same as Option A initially; Phase 4 cost paid later.
- _Benefit:_ Best of both: panic gone now, Phase 4 stays scoped to
  Phase 4.
- _Risks:_ Two-step transition has a transient phase where users
  who want both first-order and scoped effects in one program
  cannot compile. Acceptable given no scoped constructors exist
  today.

**Recommendation: Option C.** The runtime panic is the immediate
review concern; Option A removes it in a session. Option B is the
right long-term fix but is the entire Phase 4 effort. Sequencing
A-then-B converts an `unreachable!` lie into an honest
"not-yet-supported" type error today, then upgrades to the full
solution when Phase 4 lands. Per the convention in
[CLAUDE.md "executing actions with care"], pushing a panic out of
the runtime and into the type system is strictly safer.

**Dependencies and ordering:** Option A is standalone and should
land before Phase 4 begins. Option B is the existing Phase 4
plan; no change to its sequencing.

**Plan revision required:** Yes (small for A, none for B).
Option A requires noting in
[plan.md "Phase 3 step 2" entries](../../plan.md#L1957) that the
`interpret` family is `S = CNilBrand`-only until Phase 4 ships
the scoped dispatch.

### F4. The "single-shot vs multi-shot" property promised per wrapper is not enforced at the effect-instance level

**Restated:** The State effect's continuation slot is `dyn Fn`
behind an `Rc` or `Arc` regardless of which wrapper holds it. So
on the single-shot `Run` and `RunExplicit` wrappers, a handler can
still call the State continuation many times. The wrapper-level
"single-shot" property only applies to the outer Free spine, not
to per-effect closures.

**Root cause:** The plan's
[resolved 2026-05-03 sub-decision (3.a-1)](../../resolutions.md#L18)
locked "one effect type per operation across all wrappers" for
design simplicity, with continuations parameterised by the pointer
brand `P` ([state.rs:60-73](../../../../../fp-library/src/types/effects/state.rs#L60-L73)).
The choice trades the strict per-wrapper Fn-trait property for
"one State definition runs everywhere".

**Option A: Document that the per-wrapper Fn property applies to
the Free spine only, not to per-effect closures.**

- _What:_ Add a paragraph to
  [plan.md Success criteria line 2598](../../plan.md#L2598) clarifying
  that "single-shot vs multi-shot" describes the Free wrapper's
  spine consumption, and that effects with stored closure
  continuations (State, Reader, etc.) carry the multi-shot
  property at the effect-instance level on every wrapper.
  Mirror in [state.rs:24-29](../../../../../fp-library/src/types/effects/state.rs#L24-L29)
  rustdoc.
- _Cost:_ Negligible.
- _Benefit:_ Honest framing. Resolves F4's "API claim does not
  match reality" concern.
- _Risks:_ Some users may pick `Run` (single-shot) thinking it
  prevents multi-shot resource leaks; the doc fix puts the burden
  on users to read rustdoc.

**Option B: Split the State effect type into
`State<RcBrand>` and `StateOnce` variants, with Fn vs FnOnce
continuations.**

- _What:_ Add a parallel `StateOnce<S, A>` with
  `Get(Box<dyn FnOnce(S) -> A>)`. Use the FnOnce version on the
  single-shot wrappers; keep the Fn version on multi-shot wrappers.
  Macro-generate per-wrapper smart constructors. Touches every
  effect-with-closure type (State today; Reader, Except, Writer,
  Choose in Phase 3 step 5).
- _Cost:_ Medium. Doubles the per-effect type definitions.
  Reproduces the rejected design from
  [resolutions.md (3.a-2) sub-decision](../../resolutions.md#L209).
- _Benefit:_ Type-system enforcement of single-shot.
- _Risks:_ Doubles the documentation surface, doubles the macro
  complexity, doubles the brand registration. The plan locked
  (3.a-1) "single effect type" specifically to avoid this. Option
  B is therefore re-litigating a settled decision.

**Option C: Wrap the per-effect closure in a `OnceCell`-style
runtime guard on single-shot wrappers.**

- _What:_ Per-wrapper inherent `lift` constructors install a
  runtime-checked `OnceCell` around the stored closure on
  single-shot wrappers. A second call panics.
- _Cost:_ Small. One wrapper helper plus a per-wrapper smart
  constructor adjustment.
- _Benefit:_ Runtime detection of multi-shot misuse on single-shot
  wrappers.
- _Risks:_ Converts a silent semantic bug into a runtime panic,
  which is worse for resource safety than either honest
  documentation (Option A) or compile-time prevention (Option B).
  Adds runtime overhead for a check that is meaningless on
  multi-shot wrappers.

**Recommendation: Option A.** The plan
[(3.a-1) resolution](../../resolutions.md#L209) locked "single effect
type per operation". The review's F4 finding is a complaint about
the _advertised_ property, not the implementation. Documentation
resolves the mismatch without re-opening a settled decision.
Option B re-litigates that decision; Option C trades a
documentation gap for a runtime trap.

**Dependencies and ordering:** Standalone. Bundle with F2's
documentation revision for coherent rustdoc.

**Plan revision required:** Yes (small).
[Plan.md Success criteria line 2598](../../plan.md#L2598) text should
read "single-shot vs multi-shot of the Free wrapper's spine
consumption" rather than the unqualified claim today.

### F5. The natural-transformation handler shape that the rubric demands cannot be expressed by Rust closures, and the chosen workaround removes the property

**Restated:** PureScript Run's headline `interpret` is rank-2
`(VariantF r ~> m)` polymorphic over the program's result type.
Rust closures cannot be A-polymorphic, so the port adopted a
mono-in-A `Fn(EBrand::Of<NextProgram>) -> NextProgram` shape. Each
handler is bound to one program's `A`. The escape hatch
(`NaturalTransformation` consumed by `Free::fold_free`) does not
expose a `k` either.

**Root cause:** Same as F2: stable Rust lacks rank-2 closures and
delimited continuations. The choice was made under
[resolutions.md "Rust constraints that shaped the analysis"](../../resolutions.md#L592).

**Option A: Document the limitation; promote `NaturalTransformation +
fold_free` to a first-class API.**

- _What:_ Add a "Reusable handler libraries" section to plan.md
  and to the rustdoc on
  [interpreter.rs](../../../../../fp-library/src/types/effects/interpreter.rs)
  pointing users to
  [`NaturalTransformation`](../../../../../fp-library/src/classes/natural_transformation.rs)
  for cross-`A` reuse, with a worked example showing
  `fold_free(nt, prog)`. Touches plan.md and the interpreter
  rustdoc.
- _Cost:_ Negligible.
- _Benefit:_ Removes the silent expectation that the headline
  `interpret` API is reusable across `A`. Routes the use case
  (compositional handler libraries) to the API that handles it.
- _Risks:_ The `NaturalTransformation` path's ergonomics are
  worse (no `handlers!` macro yet); users may give up.

**Option B: Add a macro that emits a per-`A` impl of a handler
trait, so a "library handler" looks like `pure_state!{...}` and
expands to an impl over an `A`-generic trait.**

- _What:_ Build a `define_handler!` macro that emits, for a given
  effect, a struct + a generic impl block that satisfies
  `DispatchHandlers<'_, EBrand::Of<'a, A>, A>` for any `A`.
  Library authors ship the macro invocation; users include the
  emitted struct in their `handlers!{}` block. Touches
  [fp-macros/src/effects.rs](../../../../../fp-macros/src/effects.rs)
  and adds a new module.
- _Cost:_ Medium. New macro plus per-effect skeleton boilerplate.
  Type inference: users pass the struct value as a handler; the
  macro emits the `impl<A> for MyStruct`, so inference picks up
  `A` from the call-site program type.
- _Benefit:_ Approximates rank-2 NT via macro-generated rank-1
  impls. Closes the "compositional handler library" gap mentioned
  in
  [resolutions.md "Decision 1" line 516](../../resolutions.md#L516).
- _Risks:_ Macro complexity. The macro must correctly emit GAT
  bounds; non-trivial. Rust trait-resolution may surface
  ambiguity if multiple library-handler structs collide on the
  same effect.

**Option C: Re-encode handlers as trait objects with per-`A` vtables.**

- _What:_ Replace the `Handler<E, F>` newtype with a trait object
  `Box<dyn for<'a, A> ErasedHandler<E, ...>>`. Lifts the rank-2
  quantification to the trait-object boundary.
- _Cost:_ Large. Stable Rust does not allow `for<...>` over types
  in trait objects (this is the same HRTB-over-types limitation
  that blocks
  [state.rs SendFunctor (M5)](../../../../../fp-library/src/types/effects/state.rs#L142-L146)).
  Likely unimplementable today.
- _Benefit:_ Real rank-2 if it worked.
- _Risks:_ Reproduces an unsolved type-system problem.

**Recommendation: Option A now, Option B if user demand surfaces.**
Option A makes the limitation honest and points users to
`fold_free` for the few cross-`A` use cases. Option B is a
worth-building macro escape hatch but only earns its complexity if
multiple library handlers want to ship; today there are none.
Option C is foreclosed by the same constraint that already blocks
F4 and M5.

**Dependencies and ordering:** Option A is standalone. Option B
should follow Phase 3 step 5 (standard FO effects); without those
effects, there is nothing to compose.

**Plan revision required:** Yes (small for A; medium for B if it
ships). The
[plan.md Phase 6+ deferred items](../../plan.md#L2325) section already
mentions an `interpret_nt` deferred entry per
[resolutions.md "Decision 4" line 568](../../resolutions.md#L568); add
a cross-link to `NaturalTransformation` there. For B, add a new
phase entry "Phase 5+ optional: define_handler! macro".

## Major Findings

### M1. `interpret`'s "all handlers at once" form forces handler-list ordering by lexical sort, not by user intent

**Restated:** The `handlers!{...}` macro sorts effect brands
lexically; the all-at-once `interpret` requires the handler-list
cells to align with the row chain in the same order. For
non-commuting effects (`NonDet x Except`), users must pick the
order via the pipelined `interpret_with::<EBrand>` form. The
all-at-once form does not expose ordering.

**Root cause:** The handler-list/row chain alignment is the
mono-in-A dispatch trait's structural invariant
([interpreter.rs:36-46](../../../../../fp-library/src/types/effects/interpreter.rs#L36-L46)).
The `handlers!` macro chose lexical sort for canonicalisation
(matches the `effects!` row macro's sort).

**Option A: Document the constraint; require `interpret_with` for
non-commuting cases.**

- _What:_ Add a rustdoc warning to the all-at-once `interpret`
  body
  ([run.rs:497-519](../../../../../fp-library/src/types/effects/run.rs#L497-L519))
  pointing users at `interpret_with` for non-commuting effects.
  Already partly documented at
  [run.rs:933-935](../../../../../fp-library/src/types/effects/run.rs#L933-L935).
- _Cost:_ Negligible.
- _Benefit:_ Honest about the limitation.
- _Risks:_ Users may not read the warning.

**Option B: Add an `interpret_in_order!{...}` macro variant that
does not lex-sort.**

- _What:_ Parallel macro that preserves user-given handler order,
  emitting a non-canonicalised `HandlersCons` chain. Users pick
  the order; the row brand at the call site must match.
- _Cost:_ Small to medium. Macro variant plus row-brand
  permutation proof via existing `CoproductSubsetter`.
- _Benefit:_ Lets users control order without dropping to
  `interpret_with`.
- _Risks:_ Two macros for the same job; users have to remember
  which to use. Lexical-sort canonicalisation was deliberate per
  [decisions.md section 4.1 workaround 1](../../decisions.md).

**Recommendation: Option A.** The pipelined `interpret_with` is
the existing escape hatch, and the lexical-sort canonicalisation
buys real ergonomic value for the common case
([decisions.md section 4.1](../../decisions.md)). Option B duplicates
machinery for a rare case.

**Dependencies and ordering:** Standalone.

**Plan revision required:** No.

### M2. Stack safety only via `interpret_rec` / `tail_rec_m`; the bare `interpret_with` is host-stack recursive

**Restated:** `interpret`'s outer `loop` over `peel` is iterative,
but `interpret_with`'s `Functor::map` recursion is host-stack
([run.rs:1032-1054](../../../../../fp-library/src/types/effects/run.rs#L1032-L1054)).
Programs with deep eager-recursion blow the stack on
`interpret_with`.

**Root cause:** `interpret_with`'s recursion is via
`Functor::map(closure, layer)` where the closure recursively calls
`interpret_with`. The closure captures `handler` (must be `Clone`),
and the recursion lives in the closure body, not in a `while` loop.

**Option A: Add an `interpret_with_rec::<MBrand, EBrand>` family
parallel to `interpret_rec`.**

- _What:_ New per-wrapper inherent method that combines the
  pipeline shape with `tail_rec_m` over an external M target.
  Touches the 6 wrapper modules.
- _Cost:_ Medium. Six method bodies plus 6 doctests plus
  integration tests.
- _Benefit:_ Fully resolves M2 with stack safety and external M
  routing. Resolves the "step 4 vs step 3 capability gap"
  implicitly.
- _Risks:_ Adds a 4th interpreter family per wrapper. Users now
  pick from 4 shapes per wrapper.

**Option B: Document the limitation; recommend converting to
`interpret_rec` when depth becomes a concern.**

- _What:_ Rustdoc note on `interpret_with` pointing at
  `interpret_rec` for stack-safe variant. Users either flatten
  their pipeline into a single `interpret_rec` call or accept the
  stack risk.
- _Cost:_ Negligible.
- _Benefit:_ No new API surface.
- _Risks:_ The "flatten into `interpret_rec`" workaround forfeits
  the row-narrowing benefit (`interpret_with` returns
  `Run<RMinus, S, A>`; `interpret_rec` returns `M::Of<A>`). Not a
  full substitute.

**Recommendation: Option A.** Adds the missing fourth interpreter
shape (pipeline + stack-safe + external M target) so users do not
have to choose between row narrowing and stack safety. The 4th
family has a clean orthogonality story (simple, pipeline,
MonadRec, pipeline+MonadRec); naming it
`interpret_with_rec::<MBrand, EBrand>` keeps the convention from
the existing `interpret_with` and `interpret_rec` siblings. Per
the principle from
[resolutions.md "Decision 2"](../../resolutions.md#L528), each
interpreter shape uniquely enables a use case the others cannot
subsume; pipeline-plus-stack-safe is a real combination today
that no current method serves.

**Dependencies and ordering:** Sequence after Phase 3 step 5
(standard FO effects) so the integration tests can use State,
Reader, Except chains rather than IdentityBrand toy effects.

**Plan revision required:** Yes (small). Add a Phase 3 step
between current step 4 (`interpret_rec`) and step 5 (standard
effects), or fold into step 4 as a sub-step. The
[plan.md Phase 6+ deferred items](../../plan.md#L2325) section
already mentions axis combinations not yet shipped; this fills
one of the slots.

### M3. Handler closures must be `Clone + 'static` (and `Send + Sync` on Arc) for `interpret_with`

**Restated:** Each `Functor::map` over a layer's content clones
the handler. Users cannot capture unique resources (e.g., a
`BufWriter`) in `interpret_with` handlers.

**Root cause:** The recursive narrowing applies the handler to
each inner sub-program; each application needs a fresh handler
because the closure may move-capture in the recursion.

**Option A: Wrap the handler in `Rc` (or `Arc`) per-wrapper inside
`interpret_with`'s body.**

- _What:_ Replace the per-recursion `handler.clone()` with one
  upfront `Rc::new(handler)` followed by `Rc::clone(&rc_handler)`
  inside the recursion. Each wrapper hard-codes the right
  pointer type (`Rc` for the four non-Arc wrappers, `Arc` for
  the two Arc wrappers). Touches the 6 wrapper bodies.
- _Cost:_ Small. The Rc/Arc allocation is one per top-level
  `interpret_with` call instead of one per sub-program. Six
  parallel implementations.
- _Benefit:_ Drops the `Clone` bound on the handler; the bound
  becomes only `Fn + 'static` (plus `Send + Sync` on Arc
  wrappers). Resource captures (non-Clone) work.
- _Risks:_ Six near-duplicate bodies; the per-wrapper choice
  (`Rc` vs `Arc`) is hard-coded.

**Option B: Document the constraint; suggest users wrap unique
resources in `Rc<RefCell<_>>` themselves.**

- _What:_ Rustdoc note. No code change.
- _Cost:_ Negligible.
- _Benefit:_ No machinery change.
- _Risks:_ Users still hit the Clone bound; ergonomics unchanged.

**Option C: Parameterise `interpret_with` over a pointer brand
`P: RefCountedPointer`, with each wrapper picking its canonical
brand at the call site.**

- _What:_ Use the existing
  [`RefCountedPointer`](../../../../../fp-library/src/classes/ref_counted_pointer.rs)
  trait that already abstracts over `RcBrand` / `ArcBrand`. Each
  wrapper's `interpret_with` body becomes:

  ```rust
  let shared: <P as RefCountedPointer>::Of<'_, F> = P::new(handler);
  // recursive body uses shared.clone() (cheap refcount bump)
  ```

  The wrapper's public method picks `P`: the four non-Arc
  wrappers default to `RcBrand`, the two Arc wrappers default to
  `ArcBrand`, parallel to the choice
  [state.rs:60-73](../../../../../fp-library/src/types/effects/state.rs#L60-L73)
  already makes for State's continuation slot. The implementation
  body could live in a shared helper function generic over
  `P: RefCountedPointer` so the six wrappers do not duplicate the
  recursion logic. Touches the 6 wrapper bodies plus optionally
  one new `interpret_with_via<P>` shared helper.

- _Cost:_ Small to medium. Slightly more machinery than Option A
  (one trait bound on the helper), but the existing
  `RefCountedPointer` trait does the heavy lifting; no new
  abstraction is invented. Type inference: the wrapper-level
  public method fixes `P`, so users see no extra parameter at
  call sites. Internal `interpret_with_via<P>` requires `P` at
  the call site (one turbofish per wrapper definition).
- _Benefit:_ Drops the `Clone` bound exactly like Option A. Adds
  a single shared implementation that the six wrappers delegate
  to, reducing the per-wrapper code duplication that currently
  motivates [m9 (interpreter dispatch impl duplication)](#minor-findings).
  The Arc wrappers automatically get `Arc<F>` instead of `Rc<F>`
  via the brand choice, so `Send + Sync` falls out structurally
  rather than being a manual per-wrapper concern. Matches the
  established convention in
  [state.rs](../../../../../fp-library/src/types/effects/state.rs)
  of parameterising per-effect machinery over `P: RefCountedPointer`
  exactly so one definition serves both refcount families.
- _Risks:_ The shared helper's signature gets one extra type
  parameter and a `where P: RefCountedPointer` bound. Existing
  `interpret_with` callers see no change because the public
  method threads the brand internally. Some risk that
  `RefCountedPointer::new`'s `Of<'a, T>: Sized` bound clashes
  with handler types that contain unsized internal state, but
  handler closures are always `Sized`, so this is theoretical.

**Recommendation: Option C.** The
[`RefCountedPointer`](../../../../../fp-library/src/classes/ref_counted_pointer.rs)
trait is the project's existing answer for "abstract over Rc vs
Arc with a single brand parameter"; this is precisely its job.
Option A hard-codes the brand per wrapper and reproduces six
near-identical bodies; Option C threads `P` through one shared
helper and uses the wrapper-level brand choice as the only
difference. The Arc wrappers gain `Send + Sync` for free via the
brand, mirroring the pattern
[state.rs](../../../../../fp-library/src/types/effects/state.rs)
already establishes for per-effect continuations. Option C is
also a cleaner foundation for Phase 4 scoped handlers, which will
face the same Clone-vs-shared-handler choice.

**Dependencies and ordering:** Standalone. Should land after F1
and F3 to avoid touching the same wrappers in three commits.
Naturally pairs with M9 (interpreter dispatch impl deduplication)
since both push toward a shared helper.

**Plan revision required:** No (uses an existing trait per the
established convention; documenting in the per-step deviations
entry suffices).

### M4. State-effect `Functor` allocates a fresh `Rc` / `Arc` per `map` call

**Restated:** Each `<StateBrand as Functor>::map` rebuilds the
continuation by composing `f` over the existing `Rc<dyn Fn>`,
producing a fresh `Rc` per `map` call. A long bind chain over a
State program does N allocations.

**Root cause:** The State effect's `Functor::map` composes inside
the State variant directly rather than letting Coyoneda fuse
externally. Each layer reaches the State `Functor::map` because
the row's `Functor` impl recurses through the Coproduct chain into
the leaf brand.

**Option A: Rely on Coyoneda fusion; never call State's
Functor::map directly during composition.**

- _What:_ Audit the call sites of
  [state.rs:130-139](../../../../../fp-library/src/types/effects/state.rs#L130-L139).
  `Functor::map` on State is only called by interpreters that
  lower the Coyoneda before dispatch
  ([interpreter.rs:253](../../../../../fp-library/src/types/effects/interpreter.rs#L253)
  calls `coyo.lower()`). The lowering already fuses
  `Coyoneda::map` into one closure composition. So State's
  `Functor::map` runs once per dispatch, not per bind.
- _Cost:_ Investigation, not code. Verify with LSP findReferences
  on `<StateBrand as Functor>::map` and confirm zero callers
  outside the lowering path.
- _Benefit:_ If verified, the finding is a non-issue; document
  that the per-effect `Functor::map` cost is amortised by Coyoneda
  fusion.
- _Risks:_ If callers outside the lowering path exist, the fix is
  Option B.

**Option B: Add Coyoneda fusion at the smart-constructor layer.**

- _What:_ Smart constructors emit `Coyoneda::lift_map` (or similar)
  that pre-composes `f` into the Coyoneda's stored `f` slot,
  avoiding a fresh State allocation. Touches
  [run.rs](../../../../../fp-library/src/types/effects/run.rs) and
  the smart-constructor sites.
- _Cost:_ Medium. Per-smart-constructor refactor.
- _Benefit:_ Per-bind allocation cost moves into Coyoneda's
  internal queue.
- _Risks:_ Coyoneda's internal `f` composition is already O(1) per
  bind (the whole point of Coyoneda); duplicating it at the smart-
  constructor layer adds machinery for marginal gain.

**Recommendation: Option A (audit and document).** The Coyoneda
abstraction is precisely the per-effect-allocation amortiser; if
the State `Functor::map` is only called once per dispatch (which
it should be), this is not a real issue. If audit shows otherwise,
revisit.

**Dependencies and ordering:** Audit is standalone.

**Plan revision required:** No for Option A; Yes (small) for B.

### M5. SendFunctor for `StateBrand` is deferred; multi-thread State is unimplemented

**Restated:** `<StateBrand as SendFunctor>::send_map` cannot be
written because the bound
`<P as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
must be expressed per-`A` and stable Rust does not support
HRTB-over-types
([state.rs:142-146](../../../../../fp-library/src/types/effects/state.rs#L142-L146)).
Plan
[active blocker](../../plan.md#L599) records this. `ArcRun::get` and
`ArcRun::put` cannot ship.

**Root cause:** Same family as F5 / Option C: stable Rust's HRTB
quantifies over lifetimes only. The bound must hold for every `A`
the user might choose.

**Option A: Add per-`A` `SendFunctor` impl by forwarding through a
helper trait that requires `A: Send + Sync` at the impl site.**

- _What:_ Define a trait
  `SendFunctorAt<'a, A>: Functor` with method
  `fn send_map_at(...)` and an impl over State that requires
  `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A>: Send + Sync`.
  The wrapper machinery dispatches through
  `SendFunctorAt::<A>::send_map_at` instead of
  `SendFunctor::send_map` for State. Touches
  [state.rs](../../../../../fp-library/src/types/effects/state.rs),
  [send_functor.rs](../../../../../fp-library/src/classes/send_functor.rs),
  and the Arc wrapper bodies.
- _Cost:_ Medium. New trait variant. The wrapper bodies must
  pick between `SendFunctor` and `SendFunctorAt` based on whether
  the row carries State.
- _Benefit:_ Unblocks ArcRun State without a stable-Rust
  type-system change.
- _Risks:_ The new trait may itself hit HRTB-over-types if its
  impls quantify over `A` in non-trivial ways. Worth a feasibility
  prototype before committing.

**Option B: Wait for stable HRTB-over-types (or a Rust nightly
feature) and defer ArcRun State indefinitely.**

- _What:_ Plan
  [active blocker section](../../plan.md#L599) accepts this as the
  current state. Defer until rustc lands `for<T>` over types.
- _Cost:_ None.
- _Benefit:_ No effort spent.
- _Risks:_ The Phase 3 step 5 Success criterion ("State on every
  wrapper") cannot be met for an unknown duration. Users wanting
  multi-threaded State must use a workaround
  (closure-capture state with an `Arc<Mutex<S>>`).

**Option C: Refactor State to not embed a closure; encode it as a
discriminator + result type, with the interpreter supplying state
externally.**

- _What:_ Make `State<S, A>` a tag-only enum
  (`State::Get`, `State::Put(S)`); the result-type relationship
  with `A` lives in a separate trait that the interpreter uses to
  thread state. No `dyn Fn` slot inside the State variant.
- _Cost:_ Large. State no longer has the same shape as
  PureScript Run's `State`; doesn't generalise to other effects
  (Reader, Except) which also have closure slots.
- _Benefit:_ Bypasses the HRTB-over-types issue entirely (no
  `dyn Fn` to quantify over).
- _Risks:_ Departs from PureScript Run's per-effect closure
  pattern; adopts a one-off encoding.

**Recommendation: Option A (with prototype) plus Option B as
fallback.** Spike a `SendFunctorAt` trait against State; if it
type-checks, ship it. If the prototype hits the same HRTB
limitation, accept Option B and document Phase 3 step 5's "ArcRun
State" criterion as deferred.

**Dependencies and ordering:** Spike before Phase 3 step 5a.4
(ArcRun State). If spike fails, revise plan to defer.

**Plan revision required:** Maybe (depends on spike outcome).

### M6. No async / IO story; `Future`-as-`MonadRec` is deferred to "Phase 6+"

**Restated:** Real-world IO inside handlers and async/await
integration are absent. Plan
[line 1308-1312](../../plan.md#L1308-L1312) commits to `Future` as a
MonadRec target later but ships only `Thunk`, `Option`, `Result`
today.

**Root cause:** `Future`-as-`MonadRec` requires `MonadRec` to
accommodate a `Future` carrier, which involves runtime selection
(`tokio` vs `async-std`), polling state, and waker management.
Plan
[Phase 6+ deferred entry](../../plan.md#L2325) is correct that this
is a separate effort.

**Option A: Accept the Phase 6+ deferral; document async usage
pattern via `spawn_blocking`.**

- _What:_ Add a "Async usage" section to plan.md and the run
  rustdocs explaining that for now async users wrap interpreter
  calls in `spawn_blocking`. Mirror in the interpreter rustdoc.
- _Cost:_ Negligible.
- _Benefit:_ Honest about the gap. Gives users the workaround.
- _Risks:_ The workaround does not support cancellation or
  structured concurrency.

**Option B: Bring the `Future`-as-`MonadRec` work forward into
Phase 5.**

- _What:_ Implement a `MonadRec` impl for `Pin<Box<dyn Future>>`
  (or similar). Touches
  [`monad_rec.rs`](../../../../../fp-library/src/classes/monad_rec.rs)
  (which exposes the `tail_rec_m` method on the `MonadRec`
  trait), and adds a `FutureBrand`. Substantial.
- _Cost:_ Large. Whole sub-phase.
- _Benefit:_ Closes M6 fully. Users can `interpret_rec::<FutureBrand>`.
- _Risks:_ Async Rust ecosystem fragmentation (tokio vs async-std)
  may force runtime-specific brands.

**Recommendation: Option A.** Phase 6+ deferral is the existing
plan; bringing async forward is multi-week work for a use case
not yet driven by user demand. Document the workaround and stay
the course.

**Dependencies and ordering:** Standalone (Option A).

**Plan revision required:** Yes (small) for A.

### M7. `bind` is `FnOnce` while handler closures are `Fn`; conversions across the API are awkward

**Restated:** `Run::bind` takes `FnOnce(A) -> Run<R, S, B>` while
`DispatchHandlers`'s closure is `Fn(...) -> NextProgram`. A user
helper that forwards a closure to both APIs cannot pick a single
bound.

**Root cause:** `bind` is consumed once (the program is moved into
bind); handlers may be called many times by `tail_rec_m`'s `Fn`
step closure (per
[resolutions.md Q2 line 321](../../resolutions.md#L321)). The bounds
are correct for their respective callers; they just do not
co-compose.

**Option A: Accept the asymmetry; document it.**

- _What:_ One-line rustdoc note on `bind` and on
  `DispatchHandlers::dispatch` mentioning the asymmetry.
- _Cost:_ Negligible.
- _Benefit:_ Surfaces the constraint.
- _Risks:_ Users still face the awkwardness.

**Option B: Relax `bind` to accept `Fn`.**

- _What:_ Change `bind` to take `Fn(A) -> Run<R, S, B>`.
- _Cost:_ Small. But `bind` consumes `self` and produces a single
  `Run` ; if the closure is `Fn`, the surrounding `Free::bind`
  body must accommodate calling it multiple times (which it does
  not today; bind is single-use semantically).
- _Benefit:_ Symmetric `Fn` bound.
- _Risks:_ `Free::bind`'s implementation in
  [`free.rs`](../../../../../fp-library/src/types/free.rs) builds the
  CatList queue assuming single-use bind closures; relaxing the
  bound has implications across the Free family.

**Recommendation: Option A.** The asymmetry is correct per the
respective caller's needs (`bind` is single-use; handlers are
multi-call). The friction is real but localised; documentation
suffices.

**Dependencies and ordering:** Standalone.

**Plan revision required:** No.

### M8. The "scoped" row in every public type signature is currently always `CNilBrand`

**Restated:** Every wrapper carries `Run<R, S, A>` with `S` only
ever inhabitable as `CNilBrand`. Users pay the type parameter cost
in every signature for no current benefit.

**Root cause:** Plan
[decisions.md section 4.5](../../decisions.md) committed the dual-row
architecture before any scoped constructor existed; the parameter
is structurally reserved for Phase 4.

**Option A: Hide `S` behind a default type parameter.**

- _What:_ Default `S = CNilBrand` on the six wrapper structs.
  Users writing `Run<R, A>` get the empty scoped row; users
  writing `Run<R, S, A>` opt in. Touches every wrapper struct
  declaration.
- _Cost:_ Small. Stable Rust supports defaults on struct generics
  but not on inherent-method generics, so the default applies
  only at struct sites.
- _Benefit:_ Day-to-day type signatures get shorter. Users who
  never use scoped effects never see the parameter.
- _Risks:_ Inference may resolve `S` to the default at sites
  where the user wanted to leave it open. Requires audit.

**Option B: Wait for Phase 4; accept the parameter noise as a
forward-compatibility tax.**

- _What:_ Status quo. When Phase 4 ships, every signature already
  has the parameter slot.
- _Cost:_ None.
- _Benefit:_ Zero migration when Phase 4 lands.
- _Risks:_ Continued parameter-noise complaints.

**Recommendation: Option B.** Phase 4 is on the roadmap, so the
parameter has a real future role. Adding defaults now plus
removing them at Phase 4 is more work than leaving them in place.

**Dependencies and ordering:** Standalone.

**Plan revision required:** No.

## Minor Findings

- **m1.** `clippy::unreachable` suppression in
  [run.rs:493-496](../../../../../fp-library/src/types/effects/run.rs#L493-L496):
  remove the suppression after F3 Option A lands (the
  `unreachable!` arm becomes structurally impossible via
  `match cnil {}`).
- **m2.** `&self` on `HandlersNil::dispatch`
  ([interpreter.rs:189-194](../../../../../fp-library/src/types/effects/interpreter.rs#L189-L194)):
  acceptable; an alternative `fn dispatch(layer: CNil) -> NextProgram`
  free function would avoid materialising `HandlersNil`, but
  duplicates the trait object surface. Leave as-is.
- **m3.** `Handler<E, F>: Clone, Copy` derive
  ([handlers.rs:75-81](../../../../../fp-library/src/types/effects/handlers.rs#L75-L81)):
  remove `Copy` (closures are not `Copy`), keep `Clone`.
- **m4.** Builder ordering enforced only by docs
  ([handlers.rs:140-180](../../../../../fp-library/src/types/effects/handlers.rs#L140-L180)):
  add a compile-time check via a marker trait `HandlerListAlignedWith<RowBrand>`
  that the dispatch impl requires; failures surface at the
  builder call site rather than at dispatch.
- **m5.** Aliases `run` / `run_rec`
  ([run.rs:824-839](../../../../../fp-library/src/types/effects/run.rs#L824-L839)):
  retain for PureScript parity; tag the rustdoc with
  `#[doc(alias = "interpret")]` so search elides the duplication.
- **m6.** `Node` HRTB-poisoning helpers in arc_run.rs: extract a
  shared private `node_helpers` submodule to deduplicate the
  three workaround helpers across arc_run.rs and arc_run_explicit.rs.
- **m7.** State module docs reference plan phases by identifier
  ([state.rs:1-30](../../../../../fp-library/src/types/effects/state.rs#L1-L30)):
  rewrite to be self-contained per
  [feedback_no_history_in_text.md memory note]. Touches several
  effects-module docs.
- **m8.** `Member` facade over `CoprodInjector` /
  `CoprodUninjector`
  ([member.rs:81-139](../../../../../fp-library/src/types/effects/member.rs#L81-L139)):
  retain; the `Remainder` projection is the reason the facade
  exists. Document this in the module docs.
- **m9.** Three near-duplicate `DispatchHandlers` impls
  ([interpreter.rs:329-388](../../../../../fp-library/src/types/effects/interpreter.rs#L329-L388)):
  factor into one impl over a `Lower<NextProgram>` trait
  (consuming for Coyoneda, by-ref for Rc/Arc), or accept the
  duplication and add a comment cross-linking the three.

## Root Cause Clusters

The 13 non-minor findings collapse into 4 root-cause clusters.

**Cluster 1: Mono-in-A interpreter shape, no `M` in return type
(F1, F5, M6, partially F2 and M2).** The mono-in-A choice
([resolutions.md Decision 1, Q1](../../resolutions.md#L466)) was
forced by Rust's no-rank-2-closures constraint. Consequence: the
return type is `A`, not `M<A>`, so anything that needs to thread
an effect monadically (state, async/IO, library-handler reuse)
must work around the absence of `M`. Single remediation
**(landing F1's Option A drop-`init`)** does not unify the
cluster but makes the headline API honest about the limitation.
Full unification would require StateT (deferred to Phase 6+).

**Cluster 2: No callable continuation primitive (F2, F4 partially,
M3 partially).** The freer-monad encoding stores per-effect
closures inside variants; there is no uniform `k`. F2 is the
purest expression; F4 is the per-wrapper Fn-trait property leaking
through; M3 is the handler-Clone bound that the per-effect
closures impose. Cluster remediation: **document honestly (F2
Option A)** and accept the encoding's identity.

**Cluster 3: Scoped row is structural-only (F3, M8).** The
`Node::Scoped` arm has no constructors and no dispatch trait. F3
is the runtime panic; M8 is the parameter noise. Cluster
remediation: **F3 Option C** (tighten now, deliver Phase 4 later)
plus M8 Option B (wait for Phase 4) collapses both into a single
"deliver Phase 4" workstream.

**Cluster 4: Per-effect cost and HRTB-over-types friction (M2, M4,
M5, M7).** Stable Rust's lifetime-only HRTB plus the per-effect
allocation pattern. M5 is the active blocker; M2/M4/M7 are softer
ergonomic costs. Cluster remediation: **prototype `SendFunctorAt`
(M5 Option A)**; if that succeeds, several friction points ease.

## Sequencing Plan

**Must do before continuing the WIP** (Phase 3 step 5 / Phase 4):

1. **F1 Option D: Delete `run_accum` and `run_accum_rec`
   entirely.** Small. Removes 12 method definitions + 12
   doctests; adds one closure-capture-state doctest to
   `interpret`. Lands first because every Phase 4 scoped-effect
   interpreter would otherwise inherit the same parameter mistake
   if `run_accum` survived.
2. **F3 Option A: Tighten `interpret` family to `S = CNilBrand`.**
   Small. Removes runtime panic; converts to compile-time error
   for currently-uninhabited bad programs. Pairs with F1.
3. **F2 Option A + F5 Option A + F4 Option A: documentation
   pass.** Small (a session). Aligns plan claims with rubric
   reality. Bundle as one commit.

**Do before next milestone** (Phase 3 step 5 completion):

4. **M5 Option A: Prototype `SendFunctorAt` for State.** Medium
   (a few days). If successful, unblocks ArcRun State; if not,
   defer and revise plan accordingly.
5. **M3 Option C: Parameterise `interpret_with` over
   `P: RefCountedPointer`; wrap handler in `P::Of<F>`.** Small.
   Drops Clone bound on user closures; reuses the existing
   pointer-brand abstraction. Pairs naturally with M9 (shared
   helper deduplication).
6. **M4 Option A: Audit State `Functor::map` call sites; document
   Coyoneda fusion.** Small. Confirms the per-`map` allocation
   is amortised.
7. **M2 Option A: Add `interpret_with_rec::<MBrand, EBrand>`
   family.** Medium. Six method bodies + integration tests.
   Sequence after Phase 3 step 5 so the standard FO effects can
   drive the integration tests.

**Do before Phase 4 lands** (or at Phase 4 kickoff):

8. **F3 Option B: Build `DispatchScopedHandlers` trait and ship
   `Catch`, `Local`, `Bracket`, `Span`.** Large. Whole Phase 4.
   Resolves F3 and M8 fully.

**Defer with rationale:**

9. **M6 Option A: async usage documentation.** Phase 6+
   deferred per existing plan; document the `spawn_blocking`
   workaround now.
10. **M7 Option A: bind/handler asymmetry documentation.**
    Cosmetic. Bundle with the F2/F5/F4 doc pass if it slips.
11. **F5 Option B (`define_handler!` macro): defer until library
    handlers exist.** No current consumers.
12. **All minor (m1-m9) findings: bundle into a "polish" commit
    before the next public release.** None block downstream work.

## Findings with No Clean Fix

**F2 (no callable continuation primitive).** Stable Rust has no
delimited continuations; the freer-monad encoding cannot expose a
uniform `k`. The honest answer is documentation, not a fix. Plan
revision: amend
[plan.md "Out of scope" line 1287](../../plan.md#L1287) to add:

> Callable continuation primitives in handler clauses (Plotkin-Pretnar
> `k : x -> Result`). The freer-monad encoding stores per-effect
> closures inside variant constructors; handlers fold sub-programs
> via the `DispatchHandlers` trait but do not receive a uniform
> resumable continuation. Multi-shot semantics are achievable via
> the per-effect closure (e.g., `Choose`'s embedded
> `dyn Fn(Bool) -> NextProgram` can be invoked twice), but the
> shape is per-effect, not per-handler. This is a structural
> property of the encoding, not a defect of the implementation.

**F5's rank-2 NT gap.** Same root cause as F2; Rust closures
cannot be A-polymorphic. Plan revision: extend the same
"Out of scope" entry to mention rank-2 natural transformations,
and cross-link to
[`NaturalTransformation`](../../../../../fp-library/src/classes/natural_transformation.rs)
plus
[`Free::fold_free`](../../../../../fp-library/src/types/free.rs) as
the escape hatches.

**M5 (HRTB-over-types) if the prototype fails.** If
`SendFunctorAt` cannot be expressed without HRTB-over-types
either, then ArcRun State is structurally blocked until rustc
ships the feature. Plan revision: revise
[Success criteria line 2610](../../plan.md#L2610) to qualify "State
on every wrapper" as "State on every wrapper except Arc family
(deferred pending HRTB-over-types in stable Rust)".

These three findings (F2, F5, M5-if-blocked) are the honest
"accept and document" set. The remediation is plan-revision text,
not code.
