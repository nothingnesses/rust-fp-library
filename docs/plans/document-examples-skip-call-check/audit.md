# Audit: `document_examples(skip_call_check)` Cleanup

Generated 2026-05-26. Refreshed after the Step 10 macro validation.

Command:

```bash
just document-examples --invalid-reasons --summary
```

The audit modes exclude intentional `tests/ui` compile-fail fixtures. The plain
`just document-examples` count is `705` because it includes intentional
`tests/ui` `skip_call_check` fixtures added for macro diagnostics. The cleanup
surface below covers documentation examples outside those compile-fail
fixtures.

This is the current parser-aligned baseline audit. The script uses the same
doctest normalization and `syn` call-detection semantics as the macro for
`unnecessary_skip` reporting, including assertion macro arguments and nested
helper body exclusions.

## Summary

Total objective issues: `0`.

Issues by kind: none.

No `missing_reason`, `empty_reason`, `reason_without_skip`,
`stale_placeholder_reason`, `unnecessary_skip`,
`skip_on_non_function_item`, or malformed-option issues are currently reported
outside intentional compile-fail fixtures.

## Directory Totals

No directories currently have objective invalid entries.

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
`identity.rs` and `option.rs`, is also clean. The thunk/send-thunk batch,
covering `thunk.rs` and `send_thunk.rs`, is also clean. The
trampoline/try-trampoline batch, covering `trampoline.rs` and
`try_trampoline.rs`, is also clean. The fallible-thunk batch, covering
`try_thunk.rs` and `try_send_thunk.rs`, is also clean. The lazy/try-lazy
batch, covering `lazy.rs` and `try_lazy.rs`, is also clean. The free-explicit
batch, covering `free_explicit.rs`, `rc_free_explicit.rs`, and
`arc_free_explicit.rs`, is also clean. The free family batch, covering
`free.rs`, `rc_free.rs`, and `arc_free.rs`, is also clean. The Coyoneda family
batch, covering `coyoneda.rs`, `rc_coyoneda.rs`, and `arc_coyoneda.rs`, is also
clean. The CatList family batch, covering `cat_list.rs`, `rc_cat_list.rs`, and
`arc_cat_list.rs`, is also clean. The Vec batch, covering `vec.rs`, is also
clean. The Result/Tuple2 batch, covering `result.rs` and `tuple_2.rs`, is also
clean. The final core batch, covering `control_flow.rs` and `pair.rs`, is also
clean. The optics batch, covering `fp-library/src/types/optics`, is also clean.

The repo-wide objective cleanup surface is clean. The Step 10 macro validation
now rejects stale `skip_call_check` usage when every Rust code block already
calls the documented item, and rejects `skip_call_check` on non-function items
where direct-call validation has no target.

## File Checklist

Columns:

- `stale`: placeholder reason entries.
- `unnecessary`: examples where every Rust code block appears to call the
  documented function or method.
- `non_function`: skip on an item where direct-call validation has no function
  or method target.
- `other`: missing, empty, reason-without-skip, duplicate, malformed, or
  unsupported option issues.

No files currently have objective invalid entries.
