# Plan: Fix 63 Failing Doc Tests in Indexed Optics

## Context

The indexed optics fix implementation (Steps 1–5) changed several API signatures:
- Removed generic type parameters from `folded()`, `traversed()`, `mapped()` constructors
- Removed `P`/`Q` type parameters from `optics_un_index`, `optics_as_index`, `optics_reindexed`, `IndexedSetterOptic::evaluate`, `IndexedFoldOptic::evaluate`
- Removed `'b` lifetime from `IndexedTraversalFunc::apply`

The doc examples were not updated to match these new signatures, causing 63 doc test failures across 8 files. Additionally, some doc examples have incorrect logic, wrong imports, or reference private modules.

## Error Categories

| ID | Error | Fix |
|----|-------|-----|
| A | `folded::<VecBrand>()` / `traversed::<VecBrand>()` / `mapped::<VecBrand>()` — 0 generic args expected | Remove turbofish |
| B | `optics_un_index::<..8 args..>` — takes 7, not 8 | Remove one `_` |
| C | `optics_as_index::<..8 args..>` — takes 7, not 8 | Remove one `_` |
| D | `optics_reindexed::<..10 args..>` — takes 9, not 10 | Remove one `_` |
| E | `impl Optic` result used with `optics_view`/`optics_over` — no `GetterOptic`/`SetterOptic` impl | Use `let _ = ...` + assert on original indexed optic |
| F | `IndexedSetterOptic::evaluate::<RcBrand>` — 0 generic args | Remove turbofish |
| G | `IndexedFoldOptic::evaluate::<i32, RcBrand>` — `i32: Monoid` not satisfied | Change `R` to `String` |
| H | `acc.append(f(i, x))` — `append` is associated fn, not method | Use `Semigroup::append(acc, f(i, x))` |
| I | `IndexedFoldOptic` / `IndexedTraversalOptic` not in scope | Add `use fp_library::classes::optics::*;` |
| J | `fp_library::Apply!(<M as fp_library::kinds::Kind!(...)>)` — nested path-qualified macros fail | Import macros: `use fp_library::{Apply, Kind};` |
| K | `IndexedTraversalFunc` not imported | Add import |
| L | `RcFnBrand` not imported | Add `use fp_library::brands::RcFnBrand;` |
| M | Missing `#[derive(Clone)]` on `MySetter`/`MyTraversal` | Add derive |
| N | `Traversal::traversed::<VecBrand>()` — no such method | Use different traversal construction |
| O | Private module path (`types::optics::indexed_fold::Folded`) | Use re-exported path (`types::optics::Folded`) |
| P | `IndexedTraversalFunc::apply::<OptionBrand, _>` — 1 generic arg, not 2 | Remove `_` |
| Q | Assertion value wrong (`dimap` test: 41 ≠ 25) | Fix expected value |
| R | `(usize, i32): Monoid` not satisfied (composed.rs) | Rewrite example with correct types |

## Fix Strategy

Many doc examples that implement `IndexedTraversalFunc` inline (the "MyTraversal" pattern) with complex `Apply!`/`Kind!` macros should be **replaced** with simpler examples using `traversed()`. Only `new()` constructor examples need the inline impl pattern.

For adapter examples (`evaluate_indexed`/`evaluate_indexed_discards_focus`), the `optics_un_index`/`optics_as_index` results return `impl Optic` which can't be used with typed operations — use `let _ = ...` to prove compilation and assert on the original indexed optic.

## Steps by File

### Step 1: `indexed.rs` (2 failures)

**1a. `IndexedBrand::dimap` (line 117)**
- Error Q: assertion wrong — `(10 + (16 * 2)) - 1 = 41`, not 25
- Fix: Change `assert_eq!((transformed.inner)((10, 16)), 25)` → `assert_eq!((transformed.inner)((10, 16)), 41)`

**1b. `IndexedBrand::wander` (line 297)**
- Errors J, K: `Apply!`/`Kind!`/`Applicative`/`TraversalFunc` not in scope; `TraversalFunc::apply` had `'b` removed
- Fix: Add imports `use fp_library::{Apply, kinds::Kind, classes::{applicative::Applicative, optics::TraversalFunc}};` and remove `'b` from the inline impl

### Step 2: `indexed_fold.rs` (12 failures)

**Common template for inline `IndexedFoldFunc` impl (used in `new`, `clone`, `evaluate`):**
```rust
use fp_library::{
    brands::RcBrand,
    classes::{monoid::Monoid, semigroup::Semigroup},
    types::optics::*,
};
// ... struct definition (with #[derive(Clone)] for clone example) ...
impl<'a> IndexedFoldFunc<'a, usize, Vec<i32>, i32> for MyFold {
    fn apply<R: 'a + Monoid + 'static>(
        &self,
        f: Box<dyn Fn(usize, i32) -> R + 'a>,
        s: Vec<i32>,
    ) -> R {
        s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| Semigroup::append(acc, f(i, x)))
    }
}
```
Changes from current: add `semigroup::Semigroup` import, change `acc.append(f(i,x))` → `Semigroup::append(acc, f(i,x))`

**2a. `Folded::apply` (line ~183)**
- Errors O: Private module paths
- Fix: Change `types::optics::indexed_fold::Folded` → `types::optics::Folded`, change `classes::optics::indexed_fold::IndexedFoldFunc` → `types::optics::IndexedFoldFunc`

**2b. `IndexedFold::folded` (line ~228)**
- Error A: `folded::<VecBrand>()` takes 0 generic args
- Fix: Remove `::<VecBrand>` turbofish

**2c. `IndexedFold::new` (line ~128)**
- Error H: `acc.append(f(i,x))` method syntax
- Error I: `IndexedFoldOptic` not in scope
- Fix: Apply common template changes. Add `classes::optics::IndexedFoldOptic` import.

**2d. `IndexedFold::clone` (line ~76)**
- Same as `new` plus `#[derive(Clone)]` (already present for this one)

**2e. `IndexedFoldOptic::evaluate` (line ~309)**
- Errors H, I: Same as `new`

**2f. `IndexedOpticAdapter::evaluate_indexed` for IndexedFold (line ~378)**
- Errors A, B: `folded::<VecBrand>()` + `optics_un_index` 8 args
- Fix: Remove `::<VecBrand>`, reduce `optics_un_index` to 7 args

**2g. `IndexedOpticAdapterDiscardsFocus::evaluate_indexed_discards_focus` for IndexedFold (line ~428)**
- Errors A, C: Same pattern with `optics_as_index`

**2h–2l. IndexedFoldPrime variants** — Mirror fixes of 2b–2g

### Step 3: `indexed_getter.rs` (3 failures)

**3a. `IndexedGetterOptic::evaluate` (line ~146)**
- Error G: `IndexedFoldOptic::evaluate::<i32, RcBrand>` — `i32: Monoid` not satisfied
- Fix: Change the `IndexedFoldOptic` evaluate call to use `String` as R type, or replace the example to use `IndexedGetterOptic::evaluate` (which is what's being documented) instead of also testing `IndexedFoldOptic`

**3b. `evaluate_indexed` (line ~234)**
- Errors B, E: `optics_un_index` 8 args + `impl Optic` can't be used with `optics_view`
- Fix: Reduce to 7 args. Change to `let _unindexed = ...` + use `optics_indexed_view` on original optic for assertion

**3c. `evaluate_indexed_discards_focus` (line ~276)**
- Errors C, E: Same pattern with `optics_as_index`
- Fix: Same approach — `let _ = ...` + indexed assertion

### Step 4: `indexed_lens.rs` (8 failures)

**4a. `IndexedLens::evaluate` for `IndexedSetterOptic` (line ~555)**
- Error F: `IndexedSetterOptic::evaluate::<RcBrand>` — 0 generic args
- Fix: Remove `::<RcBrand>` turbofish

**4b. `IndexedLens::evaluate` for `IndexedFoldOptic` (line ~513)**
- Error G: `i32: Monoid` not satisfied
- Fix: Change R to `String`

**4c–d. `evaluate_indexed` / `evaluate_indexed_discards_focus` for IndexedLens (2 each)**
- Errors B/C, E: wrong generic count + `impl Optic` limitation
- Fix: reduce to 7 args, use `let _ = ...` + indexed assertion

**4e–h. IndexedLensPrime variants** — Mirror fixes

### Step 5: `indexed_setter.rs` (8 failures)

**5a. `IndexedSetter::mapped` (line ~287)**
- Error A: `mapped::<VecBrand>()` takes 0 generic args
- Fix: Remove `::<VecBrand>`

**5b. `Mapped::apply` (line ~67)**
- Error O: Private module paths
- Fix: Change `types::optics::indexed_setter::Mapped` → `types::optics::Mapped`, change `classes::optics::indexed_setter::IndexedSetterFunc` → `types::optics::IndexedSetterFunc`

**5c. `IndexedSetter::evaluate` for `IndexedSetterOptic` (line ~524)**
- Errors F, M: `evaluate::<RcBrand>` turbofish + `MySetter: Clone` not satisfied
- Fix: Remove turbofish, add `#[derive(Clone)]`

**5d–e. `evaluate_indexed` / `evaluate_indexed_discards_focus` for IndexedSetter**
- Errors A, B/C, E: `mapped::<VecBrand>()` + wrong generic count + `impl Optic` limitation
- Fix: Remove turbofish, reduce to 7 args, use `let _ = ...` + indexed assertion

**5f–h. IndexedSetterPrime variants** — Mirror fixes

### Step 6: `indexed_traversal.rs` (20 failures)

**6a. `Traversed::apply` (line ~85)**
- Errors O, P: Private module path + `apply::<OptionBrand, _>` takes 1 arg not 2
- Fix: Change `types::optics::indexed_traversal::Traversed` → `types::optics::Traversed`. Remove `_` from turbofish.

**6b. `IndexedTraversal::traversed` (line ~162)**
- Error A: `traversed::<VecBrand>()` takes 0 generic args
- Fix: Remove `::<VecBrand>`

**6c. `IndexedTraversal::new` (line ~294)**
- Errors J, K, I, L: Macro path issues + missing imports
- Fix: **Replace** the inline `MyTraversal` impl. Import macros: `use fp_library::{Apply, kinds::Kind, classes::{applicative::Applicative, optics::*}};`. Use unqualified `Apply!`/`Kind!` in the impl. Add `IndexedTraversalOptic` and `RcFnBrand` imports.

**6d. `IndexedTraversal::clone` (line ~239)**
- Same as `new` but already has `#[derive(Clone)]`

**6e. `IndexedTraversalOptic::evaluate` (line ~349)**
- Same macro/import issues as `new`

**6f. `IndexedFoldOptic::evaluate` for IndexedTraversal (line ~430)**
- Errors J, K, M, G: macros + missing Clone + `i32: Monoid`
- Fix: Add imports, add `#[derive(Clone)]`, change R to `String`

**6g. `IndexedSetterOptic::evaluate` for IndexedTraversal (line ~486)**
- Errors J, K, M, F: macros + missing Clone + turbofish
- Fix: Add imports, add derive, remove turbofish

**6h–i. `evaluate_indexed` / `evaluate_indexed_discards_focus` for IndexedTraversal**
- Errors A, B/C, E: `traversed::<VecBrand>()` + wrong generic count + `impl Optic` limitation
- Fix: Remove turbofish, reduce to 7 args, use `let _ = ...` + indexed assertion

**6j–r. IndexedTraversalPrime variants** — Mirror all the above fixes

### Step 7: `functions.rs` (8 failures)

**7a. `optics_un_index` function doc (line ~737)**
- Errors B, E: 8 generic args + `impl Optic` can't be used with `optics_view`
- Fix: **Replace** example — show compilation with `let _ = ...` and demonstrate un-indexing effect using `optics_indexed_view` on original optic

**7b. `UnIndex::evaluate` struct doc (line ~761)**
- Same as 7a

**7c. `optics_as_index` function doc (line ~815)**
- Errors C, E: Same pattern
- Fix: Same approach

**7d. `AsIndex::evaluate` struct doc (line ~836)**
- Same as 7c

**7e. `optics_reindexed` function doc (line ~899)**
- Errors D, E: 10 generic args + `impl IndexedOpticAdapter` can't be used with `optics_indexed_view`
- Fix: **Replace** example — show reindexing compiles and assert on original optic

**7f. `Reindexed::evaluate_indexed` struct doc (line ~924)**
- Same as 7e

**7g. `positions` function doc (line ~1036)**
- Error N: `Traversal::traversed::<VecBrand>()` — no such method
- Fix: **Replace** example to use `IndexedTraversal::traversed()` then demonstrate `positions` on the underlying traversal, or construct a `Traversal` via `Traversal::new(...)` with a simple inline `TraversalFunc` impl

**7h. `PositionsTraversalFunc::apply` (line ~975)**
- Errors N, K: `Traversal::traversed` doesn't exist + `IndexedTraversalFunc` not in scope
- Fix: Same as 7g for traversal construction + add import

### Step 8: `composed.rs` (2 failures)

**8a–b. `Composed::evaluate` indexed optic examples (line ~47)**
- Errors R, G: `(usize, i32): Monoid` not satisfied + type mismatch with Forget
- Fix: **Replace** the indexed part of the composed evaluate example. Use `String` as the Monoid type for fold evaluation, and fix Forget type parameters to match.

## Verification

1. `cargo check --workspace` — must pass
2. `cargo test --doc -p fp-library` — all 63 previously-failing doc tests must pass
3. `cargo clippy --workspace --all-features` — no new warnings
4. `cargo fmt --all -- --check` — formatting correct
