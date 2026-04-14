# Ref Expansion: Open Questions Investigation 3

Testing strategy, documentation, and integration concerns.

## 1. Existing Ref Trait Test Coverage is Uneven

### Issue

What test patterns exist for Ref traits, and what gaps does the plan
need to address?

### Research Findings

Ref trait testing is inconsistent across types:

- **`ref_functor.rs` (inline):** Has two quickcheck property tests for
  `LazyBrand` (identity, composition). No other brands are tested in
  the trait module itself.
- **`option.rs` (type file):** Has quickcheck tests for `ref_functor_identity`,
  `ref_functor_composition`, `ref_semimonad_left_identity`,
  `ref_semimonad_right_identity`, `ref_semimonad_associativity`,
  `ref_foldable_fold_map`, `ref_lift_identity`, `ref_lift2_commutativity`,
  `ref_traversable_identity`, `ref_traversable_consistent_with_traverse`.
  This is the most comprehensive Ref test suite.
- **`vec.rs` (type file):** Has quickcheck tests for `ref_functor_identity`,
  `ref_functor_composition`, `ref_foldable_additive`,
  `ref_semimonad_left_identity`, `ref_semimonad_right_identity`,
  `ref_semimonad_associativity`, `ref_lift_identity`,
  `ref_traversable_identity`, `ref_traversable_consistent_with_traverse`,
  `par_ref_functor_equivalence`. Similarly comprehensive.
- **`result.rs` (type file):** Has quickcheck tests for `bifunctor_identity`
  and `bifunctor_composition` (by-value). Has NO property-based tests
  for any Ref traits, despite implementing RefFunctor, RefFoldable,
  and RefTraversable for both `ResultErrAppliedBrand` and
  `ResultOkAppliedBrand`.
- **`ref_foldable.rs`, `ref_traversable.rs` (trait modules):** No inline
  `#[cfg(test)]` modules at all. Testing is delegated entirely to the
  type files.
- **No compile-fail tests** exist specifically for Ref traits.

### Approaches

**A: Follow the existing pattern (tests in type files only).**
Property-based tests for new Ref traits go in the type files
(`result.rs`, `pair.rs`, `option.rs`, `vec.rs`, etc.). This is the
dominant pattern. Simple, keeps tests co-located with implementations.

**B: Add inline tests in new trait modules too.**
Add a few tests in `ref_bifunctor.rs`, `ref_compactable.rs`, etc.,
similar to `ref_functor.rs`. These serve as trait-level smoke tests
that verify at least one brand works. Then add comprehensive
per-brand tests in the type files.

**C: Also add compile-fail tests.**
For traits with `Clone` bounds (RefCompactable, RefAlt), add trybuild
tests verifying that non-Clone types produce clear errors.

### Recommendation

Approach B + C. Add one or two smoke tests per trait module (matching
the `ref_functor.rs` pattern), plus comprehensive quickcheck tests in
each type file. RefCompactable and RefAlt should have compile-fail
tests for non-Clone element types.

Additionally, result.rs is missing Ref trait property tests entirely;
this existing gap should be addressed as part of or alongside this
plan. Otherwise the new RefBifunctor tests exist but the foundational
RefFunctor tests for the same type are absent.

## 2. Documentation Standards for New Ref Traits

### Issue

What documentation pattern should new traits follow? Are there
attributes or conventions that must be applied?

### Research Findings

Existing Ref trait modules follow a consistent pattern:

- **Module-level doc comment:** Starts with a short description, user
  story ("I want to..."), explanation of when this trait is useful,
  and a module-level example using `#[fp_macros::document_module]`.
- **Trait-level doc comment:** Short description, comparison to the
  by-value counterpart, explanation of `ref_` semantics, minimal
  implementation notes, laws section, and law examples using
  `#[document_examples]`.
- **Method-level attributes:** `#[document_signature]`,
  `#[document_type_parameters(...)]`, `#[document_parameters(...)]`,
  `#[document_returns(...)]`, `#[document_examples]` with inline code
  examples.
- **Free function doc:** Mirrors method docs but adds "Free function
  version that dispatches to [the type class' associated function]"
  and includes its own `#[document_signature]` and parameter docs.
- **Kind annotation:** `#[kind(type Of<'a, A: 'a>: 'a;)]` on the
  trait (or the two-parameter variant for bifunctorial traits).

For bifunctorial Ref traits specifically, the by-value `Bifunctor`
trait provides the template: `#[kind(type Of<'a, A: 'a, B: 'a>: 'a;)]`
and type parameter docs listing both sides.

### Approaches

**A: Template from RefFunctor (for single-param) and Bifunctor (for
bi-param).**
Each new trait module gets the full documentation treatment:
module doc, trait doc with laws, method docs with all five document
attributes, free function docs. This is the established pattern.

**B: Minimal docs, fill in later.**
Add stubs and defer full documentation to step 7 in the plan.

### Recommendation

Approach A. The plan already lists documentation as step 7, but
implementing traits without proper doc comments will cause `just doc`
to produce warnings (violating the zero-warnings policy in
CLAUDE.md). All five document attributes (`#[document_signature]`,
`#[document_type_parameters]`, `#[document_parameters]`,
`#[document_returns]`, `#[document_examples]`) must be applied to
every public method and free function from the start.

The plan should specify the laws for each new trait explicitly:

- **RefBifunctor:** Identity (`ref_bimap(|x| x.clone(), |x| x.clone(), p)`
  is equivalent to a clone of `p` given `A: Clone, C: Clone`).
  Composition (`ref_bimap(|x| g(&f(x)), |x| j(&h(x)), p)` equals
  `ref_bimap(g, j, ref_bimap(f, h, p))`).
- **RefBifoldable:** Consistency between `ref_bi_fold_map` and
  `ref_bi_fold_right` (same as by-value).
- **RefBitraversable:** Traverse/sequence consistency
  (`ref_bi_traverse(f, g, p)` equals
  `ref_bi_sequence(ref_bimap(f, g, p))`).
- **RefCompactable:** If also RefFunctor, identity
  (`ref_compact(ref_map(Some, fa))` preserves values given Clone).
- **RefAlt:** Associativity
  (`ref_alt(ref_alt(x, y), z) = ref_alt(x, ref_alt(y, z))`).

## 3. Free Function Re-export Mechanics

### Issue

When new free functions are added (e.g., `ref_bimap`, `ref_compact`,
`ref_alt`), how do they get exported? Are there naming concerns?

### Research Findings

The `generate_function_re_exports!` macro in `functions.rs` scans
all `.rs` files in `src/classes/` at compile time, finds public
functions, and generates `pub use` re-exports. It handles naming
conflicts via an alias map.

The existing by-value names are: `bimap`, `alt`, `compact`, `separate`,
`bi_fold_right`, `bi_fold_left`, `bi_fold_map`, `bi_traverse`,
`bi_sequence`. The new Ref functions will be prefixed with `ref_`
(e.g., `ref_bimap`, `ref_compact`), so there are no naming collisions.

The macro only scans top-level `.rs` files in the directory; it does
not recurse into subdirectories. New trait modules added as
`src/classes/ref_bifunctor.rs` will be picked up automatically.

Functions defined inside a `mod inner { ... }` block with a
`pub use inner::*;` re-export are correctly detected by
`detect_re_export_pattern`.

Dispatch functions (in `src/classes/dispatch/`) are NOT scanned by
the macro. They are manually re-exported in `functions.rs`. Since the
new Ref traits do not use dispatch (no by-value/by-ref ambiguity for
`ref_bimap` etc.), this is not a concern.

### Approaches

**A: Rely on automatic scanning.**
Add new modules; the macro picks them up. No changes to
`functions.rs` needed unless there are name conflicts.

**B: Add explicit aliases proactively.**
Even if no conflicts exist now, add aliases in the alias map to
guard against future name collisions.

### Recommendation

Approach A. The `ref_` prefix naturally avoids collisions with
existing names. The macro's automatic scanning will detect the new
free functions without manual intervention. If a future trait module
introduces a conflicting name, an alias can be added at that time.

One concern: if `RefBitraversable` introduces functions like
`ref_traverse_left` and `ref_traverse_right`, these could conflict
with a hypothetical future `ref_traverse_left` in a different context.
The plan should use `ref_bi_traverse_left` and `ref_bi_traverse_right`
to maintain the `bi` prefix and avoid ambiguity.

## 4. Module Registration and Ordering in classes.rs

### Issue

New trait modules need to be registered in `classes.rs`. Are there
ordering concerns?

### Research Findings

`classes.rs` lists all submodules in alphabetical order (`pub mod alt;`,
`pub mod alternative;`, ..., `pub mod witherable;`). The trait
re-export macro `generate_trait_re_exports!` also scans the directory
automatically; it does not depend on the `pub mod` ordering.

However, Rust's module compilation order matters for trait resolution.
The `pub mod` declarations in `classes.rs` are processed in the order
listed. If `ref_bitraversable.rs` uses `RefBifunctor` and
`RefBifoldable` as supertraits via `use crate::classes::*`, the
traits must be available by the time `ref_bitraversable` is compiled.
Since all modules are declared in `classes.rs` and the compiler
resolves names across the full crate, alphabetical ordering is fine:
`ref_bifoldable` and `ref_bifunctor` come before
`ref_bitraversable` alphabetically.

### Approaches

**A: Alphabetical insertion.**
Add `pub mod ref_alt;`, `pub mod ref_bifoldable;`,
`pub mod ref_bifunctor;`, `pub mod ref_bitraversable;`, and
`pub mod ref_compactable;` in alphabetical position. This matches
the existing convention.

### Recommendation

Approach A. Alphabetical order is the established convention and
happens to satisfy the dependency order for these specific modules.
No special ordering is needed.

The plan should note that all five new `pub mod` declarations can be
added in a single commit at the start of implementation (even before
the files exist, using empty module files) or incrementally as each
trait is implemented.

## 5. Derived RefFunctor for BifunctorAppliedBrands vs. Existing Specific Brands

### Issue

The plan proposes adding generic `RefFunctor` impls for
`BifunctorFirstAppliedBrand<Brand, A>` and
`BifunctorSecondAppliedBrand<Brand, B>` that delegate to `ref_bimap`.
Could these conflict with or interfere with existing tests that use
specific applied brands like `ResultErrAppliedBrand<E>`?

### Research Findings

The by-value side already has this pattern:

- `impl<Brand: Bifunctor, A: 'static> Functor for BifunctorFirstAppliedBrand<Brand, A>`
- `impl<E: 'static> Functor for ResultErrAppliedBrand<E>`

These coexist because `BifunctorFirstAppliedBrand<ResultBrand, i32>`
and `ResultErrAppliedBrand<i32>` are different types, even though
they map to the same underlying `Result<A, i32>` type via `Kind`.

The same separation will apply on the Ref side:

- `impl<Brand: RefBifunctor, A: Clone + 'static> RefFunctor for BifunctorFirstAppliedBrand<Brand, A>`
- `impl<E: Clone + 'static> RefFunctor for ResultErrAppliedBrand<E>`

No E0119 conflict because the types are different.

**However,** the `BifunctorFirstAppliedBrand` RefFunctor impl
requires `Brand: RefBifunctor`, which means `RefBifunctor` must be
implemented for `ResultBrand` before
`BifunctorFirstAppliedBrand<ResultBrand, E>` gains `RefFunctor`.
This is a compile-time dependency that must be satisfied.

Existing tests use `ResultErrAppliedBrand`, not
`BifunctorFirstAppliedBrand`, so they will not be affected. No
interference.

### Approaches

**A: Proceed as planned.**
The types are distinct. No test interference will occur.

**B: Add tests verifying equivalence.**
Add property tests asserting that `ref_map` via
`BifunctorFirstAppliedBrand<ResultBrand, E>` produces the same
result as `ref_map` via `ResultErrAppliedBrand<E>`. This provides a
consistency check.

### Recommendation

Approach A, augmented with approach B's equivalence tests. The plan
should include at least one quickcheck test per bifunctorial type
verifying that the generic applied brand's `ref_map` matches the
specific applied brand's `ref_map`. This catches subtle implementation
differences (e.g., argument ordering in bimap vs. the specific impl).

## 6. Property-Based Test Laws for New Traits

### Issue

What laws should be tested, and what test patterns should be used?

### Research Findings

The existing codebase uses `quickcheck_macros::quickcheck` for all
property-based law tests. Tests are inline `#[quickcheck]` functions
returning `bool`. No `proptest` usage was found.

By-value bifunctor laws are tested in `result.rs`:

- `bifunctor_identity`: `bimap(identity, identity, x) == x`.
- `bifunctor_composition`: bimap of composed functions equals
  composed bimaps.

By-value alt laws are tested in `vec.rs`:

- `alt_associativity`: `alt(alt(x, y), z) == alt(x, alt(y, z))`.
- `alt_distributivity`: `map(f, alt(x, y)) == alt(map(f, x), map(f, y))`.

By-value compactable laws are tested in `vec.rs`:

- `compactable_functor_identity`: `compact(map(Some, fa)) == fa`.
- `compactable_plus_annihilation_map`: `compact(map(|_| None, xs)) == empty`.

Bifoldable and bitraversable laws are tested only via doc examples,
not via quickcheck.

### Recommended Laws to Test

**RefBifunctor:**

- Identity: `ref_bimap(Clone::clone, Clone::clone, &p) == p`, given
  `A: Clone, C: Clone`. Uses `Result<i32, i32>` and `Pair<i32, i32>`.
- Composition: `ref_bimap(|x| g(&f(x)), |x| j(&h(x)), &p)` equals
  `ref_bimap(g, j, &ref_bimap(f, h, &p))`.
- Per-type tests for all 5 implementors.

**RefBifoldable:**

- fold_map/fold_right consistency:
  `ref_bi_fold_map(f, g, &p) == ref_bi_fold_right(fold_f, fold_g, empty, &p)`.
- Per-type tests for all 5 implementors.

**RefBitraversable:**

- traverse/sequence consistency:
  `ref_bi_traverse(f, g, &p) == ref_bi_sequence(&ref_bimap(f, g, &p))`.
  Note: `ref_bi_sequence` takes `&P<F<A>, F<B>>`, which requires
  `ref_bimap` to produce `P<F<A>, F<B>>` first, then a reference is
  taken. This is testable.
- Per-type tests for all 4 implementors.

**RefCompactable:**

- If also RefFunctor: `ref_compact(&ref_map(Some, &fa))` equals a
  clone of `fa`, given `A: Clone`. This requires chaining `ref_map`
  and `ref_compact`, which works for Vec and CatList.
- Per-type tests for all 3 implementors.

**RefAlt:**

- Associativity: `ref_alt(&ref_alt(&x, &y), &z) == ref_alt(&x, &ref_alt(&y, &z))`.
- Distributivity with RefFunctor:
  `ref_map(f, &ref_alt(&x, &y)) == ref_alt(&ref_map(f, &x), &ref_map(f, &y))`.
- Per-type tests for all 3 implementors.

### Approaches

**A: Quickcheck tests in type files (established pattern).**
Add `ref_bifunctor_identity`, `ref_bifunctor_composition`, etc. in
`result.rs`, `pair.rs`, etc. Use `Result<i32, i32>` and similar
simple types for quickcheck generation.

**B: Also add doc-level law examples on traits.**
Embed law examples in the trait doc comment, matching the by-value
traits. These are compiled as doc tests.

### Recommendation

Both A and B. Quickcheck tests in type files provide randomized
coverage. Doc-level examples provide readable documentation and
serve as additional test vectors. This matches the established
pattern where, e.g., `Bifunctor` has law examples in its doc comment
AND quickcheck tests in `result.rs`.

The plan's step 6 ("Tests") is too vague. It says "Property-based
tests for RefBifunctor bimap identity and composition" but does not
mention laws for the other four traits. It should enumerate all laws
and specify which types get quickcheck tests. A reasonable minimum:
quickcheck tests for Result, Pair, and Vec (or Option for
Compactable/Alt), covering identity and composition (or
associativity) for each trait.

## 7. Implementation Order Dependencies and Compilation

### Issue

Does the plan specify the order of module additions clearly enough?
Are there implicit dependencies that could cause compilation failures
if implemented out of order?

### Research Findings

The plan specifies:

1. RefBifunctor (+ derived RefFunctor for applied brands).
2. RefBifoldable (+ derived RefFoldable for applied brands).
3. RefBitraversable (+ derived RefTraversable for applied brands).
4. RefCompactable.
5. RefAlt.
6. Tests.
7. Documentation.

**Dependency analysis:**

- **RefBitraversable** requires `RefBifunctor + RefBifoldable` as
  supertraits. Implemented at step 3, after steps 1-2. Correct.
- **RefAlt** requires `RefFunctor` as a supertrait (from the plan's
  signature: `pub trait RefAlt: RefFunctor`). RefFunctor already
  exists. No ordering issue.
- **Derived RefFoldable for applied brands** (step 2) requires the
  trait `RefBifoldable` (defined in step 2) and the Kind impl for
  `BifunctorFirstAppliedBrand` (already exists from the by-value
  side). No ordering issue within step 2.
- **Derived RefTraversable for applied brands** (step 3) requires
  `RefBifunctor + RefBifoldable + RefBitraversable` (all defined by
  step 3) and RefFunctor + RefFoldable for the applied brand (derived
  in steps 1-2). Correct ordering.

**Implicit dependency not called out:** The derived RefTraversable
for `BifunctorFirstAppliedBrand` requires the applied brand to
implement `RefFunctor` (from step 1) AND `RefFoldable` (from step 2)
as RefTraversable supertraits. These are satisfied by the derived
impls from steps 1-2. But if someone tries to implement step 3's
derived RefTraversable without first completing steps 1-2's derived
RefFunctor and RefFoldable, compilation will fail. The plan lists
these in the right order, but does not explicitly state this
dependency.

**Another implicit dependency:** The `FnBrand` type parameter in
RefBifoldable and RefBitraversable (from the plan's signatures)
requires `LiftFn` (or `CloneFn`). This trait must be in scope. The
existing `classes` module already exports it via `use crate::classes::*`,
so this is not an issue.

### Approaches

**A: Keep the current ordering, add explicit dependency notes.**
The plan's ordering is correct. Add a note that steps 1-3 must be
sequential because each derived applied-brand impl depends on the
previous step's derived impls.

**B: Restructure to implement all three bi-traits per type at once.**
Instead of "all types get RefBifunctor, then all get RefBifoldable,
then all get RefBitraversable," implement all three for Result, then
all three for Pair, etc. This avoids partial states.

### Recommendation

Approach A. The plan's current "trait-first" ordering (all types get
trait X, then all get trait Y) is cleaner than a "type-first"
ordering because it allows each PR/commit to be self-contained around
a single trait definition. The implicit dependency between steps 1-3
should be documented, noting that the derived applied-brand impls for
RefTraversable depend on the derived RefFunctor and RefFoldable impls
from steps 1 and 2.

Steps 4 and 5 (RefCompactable, RefAlt) are independent of steps 1-3
and independent of each other. They could be implemented in parallel
or in any order.

## Summary

| Issue                         | Finding                                                                                      | Recommendation                                                                                        |
| ----------------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| 1. Ref trait test coverage    | Uneven; result.rs has no Ref quickcheck tests despite having impls.                          | Add quickcheck tests in type files + smoke tests in trait modules. Fix existing gaps in result.rs.    |
| 2. Documentation standards    | Five document attributes required on every public method/function. Laws must be specified.   | Apply full doc template from the start. Enumerate laws for all 5 traits in the plan.                  |
| 3. Free function re-exports   | Automatic scanning handles new modules. `ref_` prefix avoids conflicts.                      | Rely on automatic scanning. Use `ref_bi_traverse_left` (not `ref_traverse_left`) to keep `bi` prefix. |
| 4. Module registration        | Alphabetical order in classes.rs. No compilation order concerns.                             | Insert modules alphabetically; this is the established pattern.                                       |
| 5. Applied brand interference | Generic and specific brands are different types; no E0119 conflict.                          | Proceed as planned. Add equivalence tests between generic and specific applied brands.                |
| 6. Property test laws         | Plan only mentions RefBifunctor laws; omits the other 4 traits.                              | Enumerate all laws for all 5 traits. Add quickcheck tests per type for each trait.                    |
| 7. Implementation ordering    | Steps 1-3 must be sequential (derived applied-brand impls chain). Steps 4-5 are independent. | Document the sequential dependency for steps 1-3. Steps 4-5 can be done in any order.                 |
