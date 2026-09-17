# FS-0 Baseline Metrics (foundation sweep, Tier 0)

Captured at the start of the foundation sweep (charter setup step S6), from the effects subsystem at branch `feat/effects` commit `0204d802`. These are the control numbers the assessment rubric scores candidate designs against (rubric items 4 and 6). Line counts include doctests and inline tests, so they measure source volume to maintain, not logic density.

## Code-volume baseline

| Surface                                                                                                                                                     | Lines | Measurement                                                                                                                                |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ----: | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Six wrapper families (the six wrapper files plus their `representation`/`boundary`/`raw_scoped`/`smart_constructors` submodules)                            | 40792 | `wc -l` over `run.rs`, `rc_run.rs`, `arc_run.rs`, `run_explicit.rs`, `rc_run_explicit.rs`, `arc_run_explicit.rs` and their `*/` submodules |
| Boundary-frame subsystem (all `*/boundary.rs` and `*/raw_scoped.rs`, `interpreter/scoped_resume.rs`, and the `standard_scoped_handlers/*/carrier.rs` files) | 17205 | `wc -l` over those files                                                                                                                   |
| `standard_scoped_handlers/` (whole subtree)                                                                                                                 | 18594 | `wc -l` over the subtree                                                                                                                   |
| Result-polymorphic protocol trait definitions                                                                                                               |    22 | `rg 'pub trait \w*FirstOrder(Handler\|Replacer\|Rewriter\|Accumulator\|PreservingAccumulator)\b'` count                                    |
| Effects subsystem total (`types/effects/**` plus `brands/effects.rs`)                                                                                       | 87137 | `wc -l` over the subtree                                                                                                                   |
| Effects macros (`fp-macros/src/effects/**`)                                                                                                                 |  1371 | `wc -l` over the subtree                                                                                                                   |

What each candidate design would aim to remove, for rubric item 4:

- FS-1 (elaboration): the boundary-frame subsystem (~17.2k), the 22 result-polymorphic protocol-trait definitions and their bodies, and most of the per-wrapper duplication within the six families (~40.8k).
- FS-2 (weave): the same boundary-frame subsystem, replaced by a single weave mechanism; the protocol traits largely go too.
- FS-3 (facade): deletes none of the above; adds a type-level partition layer.
- The substrate unification (Axis 4, all designs) targets the duplication within the six wrapper families.

## Performance baseline

Deferred. The existing `effect_rows` and `scoped_operations` Criterion benches are the performance control for rubric item 6, which is only exercised at POC-10. Capturing the numbers now risks them drifting before POC-10 and being measured in a different build environment, so they are captured immediately before POC-10 instead, against whatever `feat/effects` commit is current then, and recorded here as an addendum. This is a deliberate narrowing of step S6's "run the benches now" wording: the line-count baseline (needed at every gate for the debt-reduction rubric row) is captured now; the perf baseline (needed only at the final gate) is captured when it is actually compared.
