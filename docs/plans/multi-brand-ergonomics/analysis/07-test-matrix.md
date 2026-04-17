---
title: Test Matrix
date: 2026-04-17
scope: Exhaustive list of test cases for Decision V (test-driven implementation)
---

# Test Matrix

This document enumerates every test case that Decision V requires.
Each row is a concrete call pattern with its expected outcome.
During implementation, each row becomes one `#[test]` function or
one compile-fail UI test file.

## 1. Non-regression tests (single-brand)

These exercise existing inference behaviour that must stay green
throughout the migration. Written at the start of phase 1 before
any changes. All use the library's public `map`, `bind`, etc.
inference wrappers (not `explicit::`).

### 1.1 Functor (map)

| #   | Call pattern                              | Expected outcome                    | Status  | File                                              | Lines |
| --- | ----------------------------------------- | ----------------------------------- | ------- | ------------------------------------------------- | ----- |
| 1   | `map(\|x: i32\| x + 1, Some(5))`          | `Some(6)`                           | Present | `fp-library/tests/non_regression_single_brand.rs` | 16-19 |
| 2   | `map(\|x: i32\| x + 1, vec![1, 2, 3])`    | `vec![2, 3, 4]`                     | Present | `fp-library/tests/non_regression_single_brand.rs` | 21-24 |
| 3   | `map(\|x: &i32\| *x + 1, &Some(5))`       | `Some(6)`, original preserved       | Present | `fp-library/tests/non_regression_single_brand.rs` | 26-31 |
| 4   | `map(\|x: &i32\| *x + 1, &vec![1, 2, 3])` | `vec![2, 3, 4]`, original preserved | Present | `fp-library/tests/non_regression_single_brand.rs` | 33-38 |
| 5   | `map(\|x: i32\| x.to_string(), Some(5))`  | `Some("5".to_string())`             | Present | `fp-library/tests/non_regression_single_brand.rs` | 40-43 |

### 1.2 Semimonad (bind)

| #   | Call pattern                                                   | Expected outcome            | Status  | File                                              | Lines |
| --- | -------------------------------------------------------------- | --------------------------- | ------- | ------------------------------------------------- | ----- |
| 6   | `bind(Some(5), \|x: i32\| if x > 3 { Some(x) } else { None })` | `Some(5)`                   | Present | `fp-library/tests/non_regression_single_brand.rs` | 47-50 |
| 7   | `bind(None::<i32>, \|x: i32\| Some(x + 1))`                    | `None`                      | Present | `fp-library/tests/non_regression_single_brand.rs` | 52-55 |
| 8   | `bind(vec![1, 2, 3], \|x: i32\| vec![x, x * 10])`              | `vec![1, 10, 2, 20, 3, 30]` | Present | `fp-library/tests/non_regression_single_brand.rs` | 57-60 |

### 1.3 Semimonad (join)

| #   | Call pattern              | Expected outcome | Status  | File                                              | Lines |
| --- | ------------------------- | ---------------- | ------- | ------------------------------------------------- | ----- |
| 9   | `join(Some(Some(5)))`     | `Some(5)`        | Present | `fp-library/tests/non_regression_single_brand.rs` | 71-74 |
| 10  | `join(Some(None::<i32>))` | `None`           | Present | `fp-library/tests/non_regression_single_brand.rs` | 76-79 |

### 1.4 Foldable

| #   | Call pattern                                                                         | Expected outcome | Status  | File                                              | Lines |
| --- | ------------------------------------------------------------------------------------ | ---------------- | ------- | ------------------------------------------------- | ----- |
| 11  | `fold_left::<RcFnBrand, _, _, _, _>(0, \|acc: i32, x: i32\| acc + x, vec![1, 2, 3])` | `6`              | Present | `fp-library/tests/brand_inference_integration.rs` | 85-89 |
| 12  | `fold_right::<RcFnBrand, _, _, _, _>(\|a: i32, b: i32\| a + b, 0, vec![1, 2, 3])`    | `6`              | Present | `fp-library/tests/non_regression_single_brand.rs` | 83-87 |

### 1.5 Filterable

| #   | Call pattern                                 | Expected outcome | Status  | File                                              | Lines |
| --- | -------------------------------------------- | ---------------- | ------- | ------------------------------------------------- | ----- |
| 13  | `filter(\|x: i32\| x > 2, vec![1, 2, 3, 4])` | `vec![3, 4]`     | Present | `fp-library/tests/non_regression_single_brand.rs` | 91-94 |

### 1.6 Lift

| #   | Call pattern                                        | Expected outcome | Status  | File                                              | Lines   |
| --- | --------------------------------------------------- | ---------------- | ------- | ------------------------------------------------- | ------- |
| 14  | `lift2(\|a: i32, b: i32\| a + b, Some(1), Some(2))` | `Some(3)`        | Present | `fp-library/tests/non_regression_single_brand.rs` | 98-101  |
| 15  | `lift2(\|a: i32, b: i32\| a + b, Some(1), None)`    | `None`           | Present | `fp-library/tests/non_regression_single_brand.rs` | 103-106 |

### 1.7 Alt

| #   | Call pattern                | Expected outcome | Status  | File                                              | Lines   |
| --- | --------------------------- | ---------------- | ------- | ------------------------------------------------- | ------- |
| 16  | `alt(None::<i32>, Some(5))` | `Some(5)`        | Present | `fp-library/tests/non_regression_single_brand.rs` | 110-113 |
| 17  | `alt(Some(3), Some(5))`     | `Some(3)`        | Present | `fp-library/tests/non_regression_single_brand.rs` | 115-118 |

### 1.8 Traversable

| #   | Call pattern                                                                                         | Expected outcome      | Status  | File                                              | Lines   |
| --- | ---------------------------------------------------------------------------------------------------- | --------------------- | ------- | ------------------------------------------------- | ------- |
| 18  | `traverse::<RcFnBrand, _, _, _, OptionBrand, _>(\|x: i32\| if x > 0 { Some(x) } else { None }, ...)` | `Some(vec![1, 2, 3])` | Present | `fp-library/tests/non_regression_single_brand.rs` | 122-129 |
| 19  | `traverse::<RcFnBrand, _, _, _, OptionBrand, _>(\|x: i32\| if x > 0 { Some(x) } else { None }, ...)` | `None`                | Present | `fp-library/tests/non_regression_single_brand.rs` | 131-138 |

Note: `traverse` and `fold_left`/`fold_right` require an explicit
`FnBrand` turbofish even with single-brand types. `traverse` also
requires an explicit applicative brand (`OptionBrand` above). These
are not fully "inferred" in the same way as `map` or `bind`, but
they do infer the traversable/foldable Brand from the container.

## 2. Multi-brand positive tests (new behaviour)

These test the new closure-directed inference for multi-brand types.
Grouped by phase; tests start `#[ignore]`d and are un-ignored as
each operation is migrated.

### 2.1 Phase 1: map (functor)

| #   | Call pattern                                                                                                                               | Expected outcome | Assumption validated                 | Status  | File                                          | Lines |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- | ------------------------------------ | ------- | --------------------------------------------- | ----- |
| 20  | `map(\|x: i32\| x + 1, Ok::<i32, String>(5))`                                                                                              | `Ok(6)`          | Val + multi-brand, Ok direction      | Present | `fp-library/tests/multi_brand_integration.rs` | 17-21 |
| 21  | `map(\|e: String\| e.len(), Err::<i32, String>("hi".into()))`                                                                              | `Err(2)`         | Val + multi-brand, Err direction     | Present | `fp-library/tests/multi_brand_integration.rs` | 23-27 |
| 22  | `map(\|x: i32\| x + 1, Err::<i32, String>("fail".into()))`                                                                                 | `Err("fail")`    | Val + multi-brand, passthrough       | Present | `fp-library/tests/multi_brand_integration.rs` | 29-33 |
| 23  | `map(\|x: &i32\| *x + 1, &Ok::<i32, String>(5))`                                                                                           | `Ok(6)`          | Ref + multi-brand, Ok direction      | Present | `fp-library/tests/multi_brand_integration.rs` | 35-40 |
| 24  | `map(\|e: &String\| e.len(), &Err::<i32, String>("hi".into()))`                                                                            | `Err(2)`         | Ref + multi-brand, Err direction     | Present | `fp-library/tests/multi_brand_integration.rs` | 42-47 |
| 25  | `map(\|x: &i32\| *x + 1, &Err::<i32, String>("fail".into()))`                                                                              | `Err("fail")`    | Ref + multi-brand, passthrough       | Present | `fp-library/tests/multi_brand_integration.rs` | 49-54 |
| 26  | `fn f<E: 'static>(r: Result<i32, E>) -> Result<i32, E> { map(\|x: i32\| x + 1, r) }` called with `Ok::<i32, String>(5)`                    | `Ok(6)`          | Generic fixed param (POC 9)          | Present | `fp-library/tests/multi_brand_integration.rs` | 56-62 |
| 27  | `fn f<E: 'static>(r: Result<i32, E>) -> Result<i32, E> { map(\|x: i32\| x + 1, r) }` called with `Err::<i32, String>("x".into())`          | `Err("x")`       | Generic fixed param, passthrough     | Present | `fp-library/tests/multi_brand_integration.rs` | 64-70 |
| 28  | `fn f<T: 'static>(r: Result<T, String>) -> Result<T, usize> { map(\|e: String\| e.len(), r) }` called with `Err("hi".into())`              | `Err(2)`         | Generic fixed param, other direction | Present | `fp-library/tests/multi_brand_integration.rs` | 72-78 |
| 29  | `fn f<T: 'static + Clone, E: 'static>(r: Result<T, E>) -> Result<T, E> { map(\|x: T\| x.clone(), r) }` called with `Ok::<i32, String>(10)` | `Ok(10)`         | Both params generic                  | Present | `fp-library/tests/multi_brand_integration.rs` | 80-86 |
| 30  | `fn f<E: 'static>(r: &Result<i32, E>) -> Result<i32, E> where E: Clone { map(\|x: &i32\| *x + 1, r) }` called with `&Ok::<i32, String>(5)` | `Ok(6)`          | Ref + generic fixed param            | Present | `fp-library/tests/multi_brand_integration.rs` | 88-96 |

### 2.2 Phase 2: bind (semimonad)

| #   | Call pattern                                                                              | Expected outcome | Assumption validated                | Status  | File                                          | Lines   |
| --- | ----------------------------------------------------------------------------------------- | ---------------- | ----------------------------------- | ------- | --------------------------------------------- | ------- |
| 31  | `bind(Ok::<i32, String>(5), \|x: i32\| Ok::<String, String>(x.to_string()))`              | `Ok("5")`        | Val + multi-brand bind              | Present | `fp-library/tests/multi_brand_integration.rs` | 100-104 |
| 32  | `bind(Err::<i32, String>("fail".into()), \|x: i32\| Ok::<String, String>(x.to_string()))` | `Err("fail")`    | Val + multi-brand bind, passthrough | Present | `fp-library/tests/multi_brand_integration.rs` | 106-111 |
| 33  | `bind(&Ok::<i32, String>(5), \|x: &i32\| Ok::<String, String>(x.to_string()))`            | `Ok("5")`        | Ref + multi-brand bind              | Present | `fp-library/tests/multi_brand_integration.rs` | 113-118 |

### 2.3 Phase 2: bimap (bifunctor, arity 2)

| #   | Call pattern                                                                    | Expected outcome | Assumption validated          | Status  | File                                          | Lines   |
| --- | ------------------------------------------------------------------------------- | ---------------- | ----------------------------- | ------- | --------------------------------------------- | ------- |
| 34  | `bimap((\|e: String\| e.len(), \|x: i32\| x + 1), Ok::<i32, String>(5))`        | `Ok(6)`          | Arity-2 multi-brand           | Present | `fp-library/tests/multi_brand_integration.rs` | 125-129 |
| 35  | `bimap((\|e: String\| e.len(), \|x: i32\| x + 1), Err::<i32, String>("hi"...))` | `Err(2)`         | Arity-2 multi-brand, Err side | Present | `fp-library/tests/multi_brand_integration.rs` | 131-135 |

### 2.4 Phase 2: fold (foldable)

| #   | Call pattern                                                                        | Expected outcome | Assumption validated | Status  | File                                          | Lines   |
| --- | ----------------------------------------------------------------------------------- | ---------------- | -------------------- | ------- | --------------------------------------------- | ------- |
| 36  | `fold_map::<RcFnBrand, _, _, _, _>(\|x: i32\| x.to_string(), Ok::<i32, String>(5))` | `"5"`            | Multi-brand fold     | Present | `fp-library/tests/multi_brand_integration.rs` | 153-157 |

### 2.5 Phase 2: filter (filterable)

| #   | Call pattern                                   | Expected outcome | Assumption validated | Status | File | Lines |
| --- | ---------------------------------------------- | ---------------- | -------------------- | ------ | ---- | ----- |
| 37  | Multi-brand filter on a type with `Filterable` | Filtered result  | Multi-brand filter   | N/A    |      |       |

Note: Result does not have a `Filterable` instance. No multi-brand
type in the library currently implements `Filterable`, so this test
case is not applicable.

### 2.6 Phase 2: traverse (traversable)

| #   | Call pattern                                                                                    | Expected outcome | Assumption validated                | Status  | File                                          | Lines   |
| --- | ----------------------------------------------------------------------------------------------- | ---------------- | ----------------------------------- | ------- | --------------------------------------------- | ------- |
| 38  | `traverse::<RcFnBrand, _, _, _, OptionBrand, _>(\|x: i32\| Some(x + 1), Ok::<i32, String>(5))`  | `Some(Ok(6))`    | Multi-brand traverse                | Present | `fp-library/tests/multi_brand_integration.rs` | 161-166 |
| 39  | `traverse::<RcFnBrand, _, _, _, OptionBrand, _>(\|_x: i32\| None::<i32>, Ok::<i32, String>(5))` | `None`           | Multi-brand traverse, inner failure | Present | `fp-library/tests/multi_brand_integration.rs` | 168-173 |

### 2.7 Phase 2: lift2

| #   | Call pattern                                                                            | Expected outcome | Assumption validated             | Status  | File                                          | Lines   |
| --- | --------------------------------------------------------------------------------------- | ---------------- | -------------------------------- | ------- | --------------------------------------------- | ------- |
| 40  | `lift2(\|a: i32, b: i32\| a + b, Ok::<i32, String>(1), Ok::<i32, String>(2))`           | `Ok(3)`          | Multi-brand lift2                | Present | `fp-library/tests/multi_brand_integration.rs` | 139-143 |
| 41  | `lift2(\|a: i32, b: i32\| a + b, Ok::<i32, String>(1), Err::<i32, String>("x".into()))` | `Err("x")`       | Multi-brand lift2, short-circuit | Present | `fp-library/tests/multi_brand_integration.rs` | 145-149 |

### 2.8 Phase 2: apply (semiapplicative)

| #   | Call pattern                                                                                   | Expected outcome | Assumption validated                   | Status  | File                                          | Lines   |
| --- | ---------------------------------------------------------------------------------------------- | ---------------- | -------------------------------------- | ------- | --------------------------------------------- | ------- |
| 42  | `ResultErrAppliedBrand::<String>::apply::<RcFnBrand, i32, i32>(Ok(lift_fn_new(...)), Ok(5))`   | `Ok(6)`          | Val multi-brand apply (explicit Brand) | Present | `fp-library/tests/multi_brand_integration.rs` | 183-190 |
| 43  | `ResultErrAppliedBrand::<String>::ref_apply::<RcFnBrand, i32, i32>(&Ok(Rc::new(...)), &Ok(5))` | `Ok(6)`          | Ref multi-brand apply (explicit Brand) | Present | `fp-library/tests/multi_brand_integration.rs` | 192-200 |

Note: the `apply` inference wrapper cannot disambiguate multi-brand
types like Result because Brand is inferred from the value container
via InferableBrand, and Result has two impls (ResultErrAppliedBrand
and ResultOkAppliedBrand). Multi-brand apply requires calling
`Semiapplicative::apply` directly with an explicit Brand. The test
matrix originally described these as using the inference wrapper, but
the actual tests use the type class method directly.

### 2.9 Phase 2: closureless multi-brand (explicit only)

| #   | Call pattern                                                            | Expected outcome | Assumption validated                 | Status  | File                                          | Lines   |
| --- | ----------------------------------------------------------------------- | ---------------- | ------------------------------------ | ------- | --------------------------------------------- | ------- |
| 44  | `explicit::join::<ResultErrAppliedBrand<String>, _, _>(Ok(Ok(5)))`      | `Ok(5)`          | Closureless multi-brand via explicit | Present | `fp-library/tests/multi_brand_integration.rs` | 204-209 |
| 45  | `explicit::alt::<ResultErrAppliedBrand<String>, _, _, _>(Ok(1), Ok(2))` | N/A              | Closureless multi-brand via explicit | N/A     |                                               |         |

Note: test 45 is not applicable because Result does not implement
the `Alt` trait. The `explicit::alt` function requires an `Alt`
bound on the Brand.

### 2.10 Phase 2: closureless single-brand (inference)

| #   | Call pattern                | Expected outcome | Assumption validated                           | Status  | File                                          | Lines   |
| --- | --------------------------- | ---------------- | ---------------------------------------------- | ------- | --------------------------------------------- | ------- |
| 46  | `join(Some(Some(5)))`       | `Some(5)`        | Closureless single-brand still infers via Slot | Present | `fp-library/tests/multi_brand_integration.rs` | 213-216 |
| 47  | `alt(None::<i32>, Some(5))` | `Some(5)`        | Closureless single-brand still infers via Slot | Present | `fp-library/tests/multi_brand_integration.rs` | 218-221 |

Note: tests 46-47 overlap with non-regression tests 9, 16. They
are included here explicitly to confirm these still work AFTER the
Slot migration (not just before).

## 3. Compile-fail UI tests (negative cases)

Each is a separate `.rs` file under `tests/ui/` with a
corresponding `.stderr` snapshot.

### 3.1 Phase 1

| #   | File                         | Call pattern                               | Expected error                                | Status  | File (full path)                                             |
| --- | ---------------------------- | ------------------------------------------ | --------------------------------------------- | ------- | ------------------------------------------------------------ |
| 48  | `multi_brand_diagonal.rs`    | `map(\|x: i32\| x + 1, Ok::<i32, i32>(5))` | Ambiguous Slot impls; suggest `explicit::map` | Present | `fp-library/tests/ui/multi_brand_diagonal.rs` + `.stderr`    |
| 49  | `multi_brand_unannotated.rs` | `map(\|x\| x + 1, Ok::<i32, String>(5))`   | Cannot infer Brand; annotate closure input    | Present | `fp-library/tests/ui/multi_brand_unannotated.rs` + `.stderr` |
| 50  | `double_ref.rs`              | `map(\|x: &i32\| *x + 1, &&Some(5))`       | FunctorDispatch Ref impl does not match `&&T` | Present | `fp-library/tests/ui/double_ref.rs` + `.stderr`              |

## 4. Other multi-brand types

The plan lists five multi-brand types. Tests above focus on Result.
At least one test per other multi-brand type confirms the pattern
generalises:

### 4.1 Pair

| #   | Call pattern                                 | Expected outcome   | Status  | File                                          | Lines   |
| --- | -------------------------------------------- | ------------------ | ------- | --------------------------------------------- | ------- |
| 51  | `map(\|x: i32\| x + 1, Pair(5, "hello"))`    | `Pair(6, "hello")` | Present | `fp-library/tests/multi_brand_integration.rs` | 225-230 |
| 52  | `map(\|s: &str\| s.len(), Pair(5, "hello"))` | `Pair(5, 5)`       | Present | `fp-library/tests/multi_brand_integration.rs` | 232-237 |

### 4.2 Tuple2

| #   | Call pattern                                                                                   | Expected outcome | Status  | File                                          | Lines   |
| --- | ---------------------------------------------------------------------------------------------- | ---------------- | ------- | --------------------------------------------- | ------- |
| 53  | `explicit::map::<Tuple2SecondAppliedBrand<&str>, _, _, _, _>(\|x: i32\| x + 1, (5, "hello"))`  | `(6, "hello")`   | Present | `fp-library/tests/multi_brand_integration.rs` | 244-250 |
| 54  | `explicit::map::<Tuple2FirstAppliedBrand<i32>, _, _, _, _>(\|s: &str\| s.len(), (5, "hello"))` | `(5, 5)`         | Present | `fp-library/tests/multi_brand_integration.rs` | 252-260 |

Note: Tuple2 cannot use brand inference even with distinct types
because it has multiple arity-1 brands. The compile-fail test
`tuple2_no_inferable_brand.rs` confirms this. Use `explicit::map`
with a turbofish instead.

### 4.3 ControlFlow

| #   | Call pattern                                                     | Expected outcome           | Status  | File                                          | Lines   |
| --- | ---------------------------------------------------------------- | -------------------------- | ------- | --------------------------------------------- | ------- |
| 55  | `map(\|x: i32\| x + 1, ControlFlow::<String, i32>::Continue(5))` | `ControlFlow::Continue(6)` | Present | `fp-library/tests/multi_brand_integration.rs` | 264-269 |

### 4.4 TryThunk

| #   | Call pattern                               | Expected outcome | Status  | File                                          | Lines   |
| --- | ------------------------------------------ | ---------------- | ------- | --------------------------------------------- | ------- |
| 56  | `map(\|x: i32\| x + 1, TryThunk::pure(5))` | `Ok(6)`          | Present | `fp-library/tests/multi_brand_integration.rs` | 273-278 |

## 5. Do-notation (Decision K audit)

| #   | Call pattern                                                                                            | Expected outcome | Assumption validated                 | Status  | File                               | Lines   |
| --- | ------------------------------------------------------------------------------------------------------- | ---------------- | ------------------------------------ | ------- | ---------------------------------- | ------- |
| 57  | `m_do!(ResultErrAppliedBrand<String> { x: i32 <- Ok::<i32, String>(5); pure(x + 1) })`                  | `Ok(6)`          | Explicit-mode m_do! with multi-brand | Present | `fp-library/tests/do_notation.rs`  | 347-355 |
| 58  | `a_do!(ResultErrAppliedBrand<String> { x <- Ok::<i32, String>(5); y <- Ok::<i32, String>(10); x + y })` | `Ok(15)`         | Explicit-mode a_do! with multi-brand | Present | `fp-library/tests/ado_notation.rs` | 418-426 |

Note: the exact macro syntax for annotated bind patterns (`x: i32 <-`)
may differ. Verify against the actual `m_do!`/`a_do!` expansion.
Inferred-mode multi-brand `m_do!` is expected to fail at `pure`
(which needs Brand via turbofish).

## 6. Assumptions encoded

The tests above collectively validate these assumptions from the
plan:

| Assumption                                            | Covered by tests                 |
| ----------------------------------------------------- | -------------------------------- |
| Val + single-brand inference works                    | 1-5, 6-8, 11-19 (non-regression) |
| Ref + single-brand inference works                    | 3-4 (non-regression)             |
| Val + multi-brand inference works via closure         | 20-22, 26-29, 31-32, 34-41       |
| Ref + multi-brand inference works via closure         | 23-25, 30, 33                    |
| Generic fixed parameter resolves correctly            | 26-30                            |
| Diagonal Result<T,T> is a compile error               | 48                               |
| Unannotated closure on multi-brand is a compile error | 49                               |
| Double reference &&T is a compile error               | 50                               |
| Closureless single-brand infers via Slot              | 46-47                            |
| Closureless multi-brand needs explicit::              | 44                               |
| Pattern generalises to other multi-brand types        | 51-52, 55-56                     |
| Do-notation explicit mode works with multi-brand      | 57-58                            |
| Existing single-brand calls are not regressed         | 1-19                             |
| Marker-via-Slot resolves Val/Ref correctly            | 3-4, 23-25, 30, 33               |
| Arity-2 (bimap) works for multi-brand                 | 34-35                            |
| Multi-brand apply needs explicit Brand                | 42-43                            |
| Tuple2 cannot use inference (needs explicit::)        | 53-54                            |

## 7. Coverage summary

| Category                              | Total  | Present | N/A   |
| ------------------------------------- | ------ | ------- | ----- |
| 1. Non-regression single-brand (1-19) | 19     | 19      | 0     |
| 2. Multi-brand positive (20-47)       | 28     | 25      | 3     |
| 3. Compile-fail UI (48-50)            | 3      | 3       | 0     |
| 4. Other multi-brand types (51-56)    | 6      | 6       | 0     |
| 5. Do-notation (57-58)                | 2      | 2       | 0     |
| **Total**                             | **58** | **55**  | **3** |

N/A tests: 37 (Result has no Filterable), 45 (Result has no Alt).
Tests 42-43 and 53-54 required adjusting the call pattern from what
the matrix originally specified (inference wrapper) to what actually
works (explicit Brand or type class method), due to limitations of
multi-brand inference for closureless operations.
