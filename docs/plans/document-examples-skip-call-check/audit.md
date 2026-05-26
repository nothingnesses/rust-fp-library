# Audit: `document_examples(skip_call_check)` Cleanup

Generated 2026-05-26. Refreshed after the Step 8 identity/option cleanup.

Command:

```bash
just document-examples --invalid-reasons --summary
```

The audit modes exclude intentional `tests/ui` compile-fail fixtures. The plain
`just document-examples` count is `967` because it includes the intentional bare
`skip_call_check` fixture added for macro diagnostics. The cleanup surface below
covers production documentation examples.

This is the current parser-aligned baseline audit. The script uses the same
doctest normalization and `syn` call-detection semantics as the macro for
`unnecessary_skip` reporting, including assertion macro arguments and nested
helper body exclusions.

## Summary

Total objective issues: `683`.

Issues by kind:

| Issue                      | Count | Cleanup classification                                                                                                                                                                |
| -------------------------- | ----: | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `stale_placeholder_reason` |   431 | Replace the placeholder reason if the skip remains justified, or remove the skip when a direct example is practical.                                                                  |
| `unnecessary_skip`         |   252 | Remove `skip_call_check` when the existing example already calls the item, or rewrite the example if the direct call is only an unreachable stand-in and the skip is still justified. |

No `missing_reason`, `empty_reason`, `reason_without_skip`,
`skip_on_non_function_item`, or malformed-option issues are currently reported
outside intentional compile-fail fixtures.

## Directory Totals

| Directory                     | Issues |
| ----------------------------- | -----: |
| `fp-library/src/types/core`   |    563 |
| `fp-library/src/types/optics` |    120 |

## Area Notes

`fp-library/src/types/effects`, `fp-library/src/classes`, and
`fp-library/src/dispatch` have zero objective invalid entries. The first core
newtype-wrapper batch, covering `additive.rs`, `conjunctive.rs`,
`disjunctive.rs`, `dual.rs`, `first.rs`, `last.rs`, and
`multiplicative.rs`, also has zero objective invalid entries. The
endofunction/endomorphism wrapper batch, covering `endofunction.rs`,
`endomorphism.rs`, and `send_endofunction.rs`, has zero objective invalid
entries as well. The function-brand and pointer wrapper batch, covering
`fn_brand.rs`, `arc_ptr.rs`, and `rc_ptr.rs`, is also clean. The small
tuple/Coyoneda-explicit batch, covering `tuple_1.rs` and
`coyoneda_explicit.rs`, is also clean. The identity/option batch, covering
`identity.rs` and `option.rs`, is also clean.

Core types and optics still carry the placeholder migration reason and should be
cleaned in the order defined by the plan. Files with both placeholder and
unnecessary-skip findings should remove the skip first when the example already
exercises the documented item; only remaining justified skips need replacement
reason text.

## File Checklist

Columns:

- `stale`: placeholder reason entries.
- `unnecessary`: examples where every Rust code block appears to call the
  documented function or method.
- `non_function`: skip on an item where direct-call validation has no function
  or method target.
- `other`: missing, empty, reason-without-skip, duplicate, malformed, or
  unsupported option issues.

| File                                               | Total | Stale | Unnecessary | Non-function | Other |
| -------------------------------------------------- | ----: | ----: | ----------: | -----------: | ----: |
| `fp-library/src/types/arc_cat_list.rs`             |     9 |     6 |           3 |            0 |     0 |
| `fp-library/src/types/arc_coyoneda.rs`             |    13 |     8 |           5 |            0 |     0 |
| `fp-library/src/types/arc_free.rs`                 |    18 |    15 |           3 |            0 |     0 |
| `fp-library/src/types/arc_free_explicit.rs`        |     8 |     6 |           2 |            0 |     0 |
| `fp-library/src/types/cat_list.rs`                 |    44 |    31 |          13 |            0 |     0 |
| `fp-library/src/types/control_flow.rs`             |    74 |    39 |          35 |            0 |     0 |
| `fp-library/src/types/coyoneda.rs`                 |    12 |     7 |           5 |            0 |     0 |
| `fp-library/src/types/free.rs`                     |    16 |    13 |           3 |            0 |     0 |
| `fp-library/src/types/free_explicit.rs`            |     6 |     5 |           1 |            0 |     0 |
| `fp-library/src/types/lazy.rs`                     |    15 |    11 |           4 |            0 |     0 |
| `fp-library/src/types/optics/affine.rs`            |    10 |     5 |           5 |            0 |     0 |
| `fp-library/src/types/optics/fold.rs`              |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/forget.rs`            |     3 |     2 |           1 |            0 |     0 |
| `fp-library/src/types/optics/functions.rs`         |    26 |    14 |          12 |            0 |     0 |
| `fp-library/src/types/optics/getter.rs`            |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/indexed.rs`           |     1 |     1 |           0 |            0 |     0 |
| `fp-library/src/types/optics/indexed_fold.rs`      |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/types/optics/indexed_getter.rs`    |     4 |     3 |           1 |            0 |     0 |
| `fp-library/src/types/optics/indexed_lens.rs`      |    16 |    10 |           6 |            0 |     0 |
| `fp-library/src/types/optics/indexed_setter.rs`    |    12 |     8 |           4 |            0 |     0 |
| `fp-library/src/types/optics/indexed_traversal.rs` |     4 |     4 |           0 |            0 |     0 |
| `fp-library/src/types/optics/iso.rs`               |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/lens.rs`              |    10 |     5 |           5 |            0 |     0 |
| `fp-library/src/types/optics/prism.rs`             |    10 |     5 |           5 |            0 |     0 |
| `fp-library/src/types/optics/review.rs`            |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/optics/setter.rs`            |     4 |     2 |           2 |            0 |     0 |
| `fp-library/src/types/pair.rs`                     |    78 |    44 |          34 |            0 |     0 |
| `fp-library/src/types/rc_cat_list.rs`              |     9 |     6 |           3 |            0 |     0 |
| `fp-library/src/types/rc_coyoneda.rs`              |    14 |     8 |           6 |            0 |     0 |
| `fp-library/src/types/rc_free.rs`                  |    18 |    15 |           3 |            0 |     0 |
| `fp-library/src/types/rc_free_explicit.rs`         |    10 |     8 |           2 |            0 |     0 |
| `fp-library/src/types/result.rs`                   |    60 |    36 |          24 |            0 |     0 |
| `fp-library/src/types/send_thunk.rs`               |     4 |     3 |           1 |            0 |     0 |
| `fp-library/src/types/thunk.rs`                    |     7 |     4 |           3 |            0 |     0 |
| `fp-library/src/types/trampoline.rs`               |     7 |     5 |           2 |            0 |     0 |
| `fp-library/src/types/try_lazy.rs`                 |    11 |     9 |           2 |            0 |     0 |
| `fp-library/src/types/try_send_thunk.rs`           |     5 |     3 |           2 |            0 |     0 |
| `fp-library/src/types/try_thunk.rs`                |    15 |     9 |           6 |            0 |     0 |
| `fp-library/src/types/try_trampoline.rs`           |     7 |     5 |           2 |            0 |     0 |
| `fp-library/src/types/tuple_2.rs`                  |    60 |    36 |          24 |            0 |     0 |
| `fp-library/src/types/vec.rs`                      |    43 |    28 |          15 |            0 |     0 |
