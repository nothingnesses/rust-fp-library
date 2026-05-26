# Audit: `document_examples(skip_call_check)` Cleanup

Generated 2026-05-26.

Command:

```bash
just document-examples --invalid-reasons --summary
```

The audit modes exclude intentional `tests/ui` compile-fail fixtures. The plain
`just document-examples` count is `1090` because it includes the intentional
bare `skip_call_check` fixture added for macro diagnostics. The cleanup surface
below covers production documentation examples.

This is the current baseline audit. Before effects cleanup starts, Step 5 must
migrate the script's `unnecessary_skip` detector to a `syn`-based parser that
matches the macro's nested helper function behavior, then this audit must be
refreshed.

## Summary

Total objective issues: `1026`.

Issues by kind:

| Issue                      | Count | Cleanup classification                                                                                                                                                                |
| -------------------------- | ----: | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `stale_placeholder_reason` |   781 | Replace the placeholder reason if the skip remains justified, or remove the skip when a direct example is practical.                                                                  |
| `unnecessary_skip`         |   245 | Remove `skip_call_check` when the existing example already calls the item, or rewrite the example if the direct call is only an unreachable stand-in and the skip is still justified. |

No `missing_reason`, `empty_reason`, `reason_without_skip`,
`skip_on_non_function_item`, or malformed-option issues are currently reported
outside intentional compile-fail fixtures.

## Directory Totals

| Directory                      | Issues |
| ------------------------------ | -----: |
| `fp-library/src/classes`       |    198 |
| `fp-library/src/dispatch`      |    134 |
| `fp-library/src/types/core`    |    529 |
| `fp-library/src/types/effects` |     56 |
| `fp-library/src/types/optics`  |    109 |

## Area Notes

`fp-library/src/types/effects` has zero stale placeholder reasons. Its `56`
issues are all `unnecessary_skip` findings and are queued for the effects
cleanup step.

Classes, dispatch, core types, and optics still carry the placeholder migration
reason and should be cleaned in the order defined by the plan. Files with both
placeholder and unnecessary-skip findings should remove the skip first when the
example already exercises the documented item; only remaining justified skips
need replacement reason text.

## File Checklist

Columns:

- `stale`: placeholder reason entries.
- `unnecessary`: examples where every Rust code block appears to call the
  documented function or method.
- `non_function`: skip on an item where direct-call validation has no function
  or method target.
- `other`: missing, empty, reason-without-skip, duplicate, malformed, or
  unsupported option issues.

| File                                                                               | Total | Stale | Unnecessary | Non-function | Other |
| ---------------------------------------------------------------------------------- | ----: | ----: | ----------: | -----------: | ----: |
| `fp-library/src/classes/applicative.rs`                                            |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/category.rs`                                               |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/clone_fn.rs`                                               |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/classes/division_ring.rs`                                          |    12 |     6 |           6 |            0 |     0 |
| `fp-library/src/classes/euclidean_ring.rs`                                         |    28 |    17 |          11 |            0 |     0 |
| `fp-library/src/classes/extract.rs`                                                |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/heyting_algebra.rs`                                        |    34 |    18 |          16 |            0 |     0 |
| `fp-library/src/classes/lazy_config.rs`                                            |     2 |     1 |           1 |            0 |     0 |
| `fp-library/src/classes/monad.rs`                                                  |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/monoid.rs`                                                 |     2 |     1 |           1 |            0 |     0 |
| `fp-library/src/classes/optics.rs`                                                 |     6 |     6 |           0 |            0 |     0 |
| `fp-library/src/classes/plus.rs`                                                   |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/pointer.rs`                                                |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_alt.rs`                                                |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_apply_first.rs`                                        |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_apply_second.rs`                                       |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_bifoldable.rs`                                         |     6 |     6 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_bifunctor.rs`                                          |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_bitraversable.rs`                                      |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_compactable.rs`                                        |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_counted_pointer.rs`                                    |     6 |     6 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_filterable.rs`                                         |     8 |     8 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_filterable_with_index.rs`                              |     8 |     8 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_foldable.rs`                                           |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_foldable_with_index.rs`                                |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_functor_with_index.rs`                                 |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_semimonad.rs`                                          |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_traversable_with_index.rs`                             |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/ref_witherable.rs`                                         |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/classes/ring.rs`                                                   |     8 |     5 |           3 |            0 |     0 |
| `fp-library/src/classes/semigroupoid.rs`                                           |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/semiring.rs`                                               |    22 |    16 |           6 |            0 |     0 |
| `fp-library/src/classes/send_clone_fn.rs`                                          |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/classes/send_ref_counted_pointer.rs`                               |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/classes/to_dyn_clone_fn.rs`                                        |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/classes/to_dyn_send_fn.rs`                                         |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/classes/wrap_drop.rs`                                              |     2 |     1 |           1 |            0 |     0 |
| `fp-library/src/dispatch/alt.rs`                                                   |     5 |     4 |           1 |            0 |     0 |
| `fp-library/src/dispatch/apply_first.rs`                                           |     5 |     4 |           1 |            0 |     0 |
| `fp-library/src/dispatch/apply_second.rs`                                          |     5 |     4 |           1 |            0 |     0 |
| `fp-library/src/dispatch/bifoldable.rs`                                            |     9 |     9 |           0 |            0 |     0 |
| `fp-library/src/dispatch/bifunctor.rs`                                             |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/bitraversable.rs`                                         |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/compactable.rs`                                           |     8 |     7 |           1 |            0 |     0 |
| `fp-library/src/dispatch/contravariant.rs`                                         |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/dispatch/filterable.rs`                                            |    12 |    12 |           0 |            0 |     0 |
| `fp-library/src/dispatch/filterable_with_index.rs`                                 |    12 |    12 |           0 |            0 |     0 |
| `fp-library/src/dispatch/foldable.rs`                                              |     9 |     9 |           0 |            0 |     0 |
| `fp-library/src/dispatch/foldable_with_index.rs`                                   |     9 |     9 |           0 |            0 |     0 |
| `fp-library/src/dispatch/functor.rs`                                               |     5 |     4 |           1 |            0 |     0 |
| `fp-library/src/dispatch/functor_with_index.rs`                                    |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/lift.rs`                                                  |    12 |    12 |           0 |            0 |     0 |
| `fp-library/src/dispatch/map_first.rs`                                             |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/map_second.rs`                                            |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/semiapplicative.rs`                                       |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/semimonad.rs`                                             |    11 |    10 |           1 |            0 |     0 |
| `fp-library/src/dispatch/traversable.rs`                                           |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/traversable_with_index.rs`                                |     3 |     3 |           0 |            0 |     0 |
| `fp-library/src/dispatch/witherable.rs`                                            |     6 |     6 |           0 |            0 |     0 |
| `fp-library/src/types/additive.rs`                                                 |     3 |     2 |           1 |            0 |     0 |
| `fp-library/src/types/arc_cat_list.rs`                                             |     9 |     6 |           3 |            0 |     0 |
| `fp-library/src/types/arc_coyoneda.rs`                                             |    13 |     8 |           5 |            0 |     0 |
| `fp-library/src/types/arc_free.rs`                                                 |    18 |    15 |           3 |            0 |     0 |
| `fp-library/src/types/arc_free_explicit.rs`                                        |     8 |     6 |           2 |            0 |     0 |
| `fp-library/src/types/arc_ptr.rs`                                                  |     8 |     7 |           1 |            0 |     0 |
| `fp-library/src/types/cat_list.rs`                                                 |    42 |    31 |          11 |            0 |     0 |
| `fp-library/src/types/conjunctive.rs`                                              |     3 |     2 |           1 |            0 |     0 |
| `fp-library/src/types/control_flow.rs`                                             |    50 |    39 |          11 |            0 |     0 |
| `fp-library/src/types/coyoneda.rs`                                                 |    12 |     7 |           5 |            0 |     0 |
| `fp-library/src/types/coyoneda_explicit.rs`                                        |     2 |     2 |           0 |            0 |     0 |
| `fp-library/src/types/disjunctive.rs`                                              |     3 |     2 |           1 |            0 |     0 |
| `fp-library/src/types/dual.rs`                                                     |     3 |     2 |           1 |            0 |     0 |
| `fp-library/src/types/effects/interpreter.rs`                                      |     1 |     0 |           1 |            0 |     0 |
| `fp-library/src/types/effects/interpreter/first_order.rs`                          |     1 |     0 |           1 |            0 |     0 |
| `fp-library/src/types/effects/interpreter/scoped_resume.rs`                        |    32 |     0 |          32 |            0 |     0 |
| `fp-library/src/types/effects/rc_run_explicit.rs`                                  |     2 |     0 |           2 |            0 |     0 |
| `fp-library/src/types/effects/run.rs`                                              |     3 |     0 |           3 |            0 |     0 |
| `fp-library/src/types/effects/run/representation.rs`                               |     1 |     0 |           1 |            0 |     0 |
| `fp-library/src/types/effects/run_explicit.rs`                                     |     2 |     0 |           2 |            0 |     0 |
| `fp-library/src/types/effects/run_explicit/boundary.rs`                            |     1 |     0 |           1 |            0 |     0 |
| `fp-library/src/types/effects/standard_scoped_handlers/ref_local/raw_replacers.rs` |     1 |     0 |           1 |            0 |     0 |
| `fp-library/src/types/effects/standard_scoped_handlers/writer/raw_accumulators.rs` |     9 |     0 |           9 |            0 |     0 |
| `fp-library/src/types/effects/variant_f.rs`                                        |     3 |     0 |           3 |            0 |     0 |
| `fp-library/src/types/endofunction.rs`                                             |     5 |     5 |           0 |            0 |     0 |
| `fp-library/src/types/endomorphism.rs`                                             |     5 |     5 |           0 |            0 |     0 |
| `fp-library/src/types/first.rs`                                                    |     2 |     1 |           1 |            0 |     0 |
| `fp-library/src/types/fn_brand.rs`                                                 |     6 |     6 |           0 |            0 |     0 |
| `fp-library/src/types/free.rs`                                                     |    16 |    13 |           3 |            0 |     0 |
| `fp-library/src/types/free_explicit.rs`                                            |     6 |     5 |           1 |            0 |     0 |
| `fp-library/src/types/identity.rs`                                                 |    10 |     9 |           1 |            0 |     0 |
| `fp-library/src/types/last.rs`                                                     |     2 |     1 |           1 |            0 |     0 |
| `fp-library/src/types/lazy.rs`                                                     |    15 |    11 |           4 |            0 |     0 |
| `fp-library/src/types/multiplicative.rs`                                           |     3 |     2 |           1 |            0 |     0 |
| `fp-library/src/types/optics/affine.rs`                                            |    10 |     5 |           5 |            0 |     0 |
| `fp-library/src/types/optics/fold.rs`                                              |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/forget.rs`                                            |     3 |     2 |           1 |            0 |     0 |
| `fp-library/src/types/optics/functions.rs`                                         |    15 |    14 |           1 |            0 |     0 |
| `fp-library/src/types/optics/getter.rs`                                            |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/indexed.rs`                                           |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/types/optics/indexed_fold.rs`                                      |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/types/optics/indexed_getter.rs`                                    |     4 |     3 |           1 |            0 |     0 |
| `fp-library/src/types/optics/indexed_lens.rs`                                      |    16 |    10 |           6 |            0 |     0 |
| `fp-library/src/types/optics/indexed_setter.rs`                                    |    12 |     8 |           4 |            0 |     0 |
| `fp-library/src/types/optics/indexed_traversal.rs`                                 |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/types/optics/iso.rs`                                               |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/lens.rs`                                              |    10 |     5 |           5 |            0 |     0 |
| `fp-library/src/types/optics/prism.rs`                                             |    10 |     5 |           5 |            0 |     0 |
| `fp-library/src/types/optics/review.rs`                                            |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/setter.rs`                                            |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/option.rs`                                                   |    13 |    13 |           0 |            0 |     0 |
| `fp-library/src/types/pair.rs`                                                     |    52 |    44 |           8 |            0 |     0 |
| `fp-library/src/types/rc_cat_list.rs`                                              |     9 |     6 |           3 |            0 |     0 |
| `fp-library/src/types/rc_coyoneda.rs`                                              |    14 |     8 |           6 |            0 |     0 |
| `fp-library/src/types/rc_free.rs`                                                  |    18 |    15 |           3 |            0 |     0 |
| `fp-library/src/types/rc_free_explicit.rs`                                         |    10 |     8 |           2 |            0 |     0 |
| `fp-library/src/types/rc_ptr.rs`                                                   |     6 |     5 |           1 |            0 |     0 |
| `fp-library/src/types/result.rs`                                                   |    36 |    36 |           0 |            0 |     0 |
| `fp-library/src/types/send_endofunction.rs`                                        |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/types/send_thunk.rs`                                               |     4 |     3 |           1 |            0 |     0 |
| `fp-library/src/types/thunk.rs`                                                    |     6 |     4 |           2 |            0 |     0 |
| `fp-library/src/types/trampoline.rs`                                               |     9 |     5 |           4 |            0 |     0 |
| `fp-library/src/types/try_lazy.rs`                                                 |    11 |     9 |           2 |            0 |     0 |
| `fp-library/src/types/try_send_thunk.rs`                                           |     5 |     3 |           2 |            0 |     0 |
| `fp-library/src/types/try_thunk.rs`                                                |    11 |     9 |           2 |            0 |     0 |
| `fp-library/src/types/try_trampoline.rs`                                           |     9 |     5 |           4 |            0 |     0 |
| `fp-library/src/types/tuple_1.rs`                                                  |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/types/tuple_2.rs`                                                  |    36 |    36 |           0 |            0 |     0 |
| `fp-library/src/types/vec.rs`                                                      |    31 |    28 |           3 |            0 |     0 |
