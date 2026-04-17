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

| #   | Call pattern                              | Expected outcome                    |
| --- | ----------------------------------------- | ----------------------------------- |
| 1   | `map(\|x: i32\| x + 1, Some(5))`          | `Some(6)`                           |
| 2   | `map(\|x: i32\| x + 1, vec![1, 2, 3])`    | `vec![2, 3, 4]`                     |
| 3   | `map(\|x: &i32\| *x + 1, &Some(5))`       | `Some(6)`, original preserved       |
| 4   | `map(\|x: &i32\| *x + 1, &vec![1, 2, 3])` | `vec![2, 3, 4]`, original preserved |
| 5   | `map(\|x: i32\| x.to_string(), Some(5))`  | `Some("5".to_string())`             |

### 1.2 Semimonad (bind)

| #   | Call pattern                                                   | Expected outcome            |
| --- | -------------------------------------------------------------- | --------------------------- |
| 6   | `bind(Some(5), \|x: i32\| if x > 3 { Some(x) } else { None })` | `Some(5)`                   |
| 7   | `bind(None::<i32>, \|x: i32\| Some(x + 1))`                    | `None`                      |
| 8   | `bind(vec![1, 2, 3], \|x: i32\| vec![x, x * 10])`              | `vec![1, 10, 2, 20, 3, 30]` |

### 1.3 Semimonad (join)

| #   | Call pattern              | Expected outcome |
| --- | ------------------------- | ---------------- |
| 9   | `join(Some(Some(5)))`     | `Some(5)`        |
| 10  | `join(Some(None::<i32>))` | `None`           |

### 1.4 Foldable

| #   | Call pattern                                                 | Expected outcome |
| --- | ------------------------------------------------------------ | ---------------- |
| 11  | `fold_left(0, \|acc: i32, x: i32\| acc + x, vec![1, 2, 3])`  | `6`              |
| 12  | `fold_right(\|x: i32, acc: i32\| acc + x, 0, vec![1, 2, 3])` | `6`              |

### 1.5 Filterable

| #   | Call pattern                                   | Expected outcome |
| --- | ---------------------------------------------- | ---------------- |
| 13  | `filter(\|x: &i32\| *x > 2, vec![1, 2, 3, 4])` | `vec![3, 4]`     |

### 1.6 Lift

| #   | Call pattern                                        | Expected outcome |
| --- | --------------------------------------------------- | ---------------- |
| 14  | `lift2(\|a: i32, b: i32\| a + b, Some(1), Some(2))` | `Some(3)`        |
| 15  | `lift2(\|a: i32, b: i32\| a + b, Some(1), None)`    | `None`           |

### 1.7 Alt

| #   | Call pattern                | Expected outcome |
| --- | --------------------------- | ---------------- |
| 16  | `alt(None::<i32>, Some(5))` | `Some(5)`        |
| 17  | `alt(Some(3), Some(5))`     | `Some(3)`        |

### 1.8 Traversable

| #   | Call pattern                                                              | Expected outcome      |
| --- | ------------------------------------------------------------------------- | --------------------- |
| 18  | `traverse(\|x: i32\| if x > 0 { Some(x) } else { None }, vec![1, 2, 3])`  | `Some(vec![1, 2, 3])` |
| 19  | `traverse(\|x: i32\| if x > 0 { Some(x) } else { None }, vec![1, -1, 3])` | `None`                |

Note: exact signatures for traverse, fold, filter, lift2, alt may
need adjustment based on the library's actual free-function
signatures (FnBrand parameters, explicit Brand turbofish for some
operations, etc.). Verify against the current API before writing
tests.

## 2. Multi-brand positive tests (new behaviour)

These test the new closure-directed inference for multi-brand types.
Grouped by phase; tests start `#[ignore]`d and are un-ignored as
each operation is migrated.

### 2.1 Phase 1: map (functor)

| #   | Call pattern                                                                                                                               | Expected outcome | Assumption validated                 |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- | ------------------------------------ |
| 20  | `map(\|x: i32\| x + 1, Ok::<i32, String>(5))`                                                                                              | `Ok(6)`          | Val + multi-brand, Ok direction      |
| 21  | `map(\|e: String\| e.len(), Err::<i32, String>("hi".into()))`                                                                              | `Err(2)`         | Val + multi-brand, Err direction     |
| 22  | `map(\|x: i32\| x + 1, Err::<i32, String>("fail".into()))`                                                                                 | `Err("fail")`    | Val + multi-brand, passthrough       |
| 23  | `map(\|x: &i32\| *x + 1, &Ok::<i32, String>(5))`                                                                                           | `Ok(6)`          | Ref + multi-brand, Ok direction      |
| 24  | `map(\|e: &String\| e.len(), &Err::<i32, String>("hi".into()))`                                                                            | `Err(2)`         | Ref + multi-brand, Err direction     |
| 25  | `map(\|x: &i32\| *x + 1, &Err::<i32, String>("fail".into()))`                                                                              | `Err("fail")`    | Ref + multi-brand, passthrough       |
| 26  | `fn f<E: 'static>(r: Result<i32, E>) -> Result<i32, E> { map(\|x: i32\| x + 1, r) }` called with `Ok::<i32, String>(5)`                    | `Ok(6)`          | Generic fixed param (POC 9)          |
| 27  | `fn f<E: 'static>(r: Result<i32, E>) -> Result<i32, E> { map(\|x: i32\| x + 1, r) }` called with `Err::<i32, String>("x".into())`          | `Err("x")`       | Generic fixed param, passthrough     |
| 28  | `fn f<T: 'static>(r: Result<T, String>) -> Result<T, usize> { map(\|e: String\| e.len(), r) }` called with `Err("hi".into())`              | `Err(2)`         | Generic fixed param, other direction |
| 29  | `fn f<T: 'static + Clone, E: 'static>(r: Result<T, E>) -> Result<T, E> { map(\|x: T\| x.clone(), r) }` called with `Ok::<i32, String>(10)` | `Ok(10)`         | Both params generic                  |
| 30  | `fn f<E: 'static>(r: &Result<i32, E>) -> Result<i32, E> where E: Clone { map(\|x: &i32\| *x + 1, r) }` called with `&Ok::<i32, String>(5)` | `Ok(6)`          | Ref + generic fixed param            |

### 2.2 Phase 2: bind (semimonad)

| #   | Call pattern                                                                              | Expected outcome | Assumption validated                |
| --- | ----------------------------------------------------------------------------------------- | ---------------- | ----------------------------------- |
| 31  | `bind(Ok::<i32, String>(5), \|x: i32\| Ok::<String, String>(x.to_string()))`              | `Ok("5")`        | Val + multi-brand bind              |
| 32  | `bind(Err::<i32, String>("fail".into()), \|x: i32\| Ok::<String, String>(x.to_string()))` | `Err("fail")`    | Val + multi-brand bind, passthrough |
| 33  | `bind(&Ok::<i32, String>(5), \|x: &i32\| Ok::<String, String>(x.to_string()))`            | `Ok("5")`        | Ref + multi-brand bind              |

### 2.3 Phase 2: bimap (bifunctor, arity 2)

| #   | Call pattern                                                                      | Expected outcome | Assumption validated          |
| --- | --------------------------------------------------------------------------------- | ---------------- | ----------------------------- |
| 34  | `bimap(\|x: i32\| x + 1, \|e: String\| e.len(), Ok::<i32, String>(5))`            | `Ok(6)`          | Arity-2 multi-brand           |
| 35  | `bimap(\|x: i32\| x + 1, \|e: String\| e.len(), Err::<i32, String>("hi".into()))` | `Err(2)`         | Arity-2 multi-brand, Err side |

### 2.4 Phase 2: fold (foldable)

| #   | Call pattern                                                                      | Expected outcome   | Assumption validated |
| --- | --------------------------------------------------------------------------------- | ------------------ | -------------------- |
| 36  | `fold_map(\|x: i32\| x.to_string(), Ok::<i32, String>(5))` with a suitable Monoid | Value from Ok side | Multi-brand fold     |

Note: exact fold signature for Result depends on whether
`ResultErrAppliedBrand<E>` has a `Foldable` instance. Verify
against the library's actual type class instances.

### 2.5 Phase 2: filter (filterable)

| #   | Call pattern                                   | Expected outcome | Assumption validated |
| --- | ---------------------------------------------- | ---------------- | -------------------- |
| 37  | Multi-brand filter on a type with `Filterable` | Filtered result  | Multi-brand filter   |

Note: verify which multi-brand types have `Filterable` instances.

### 2.6 Phase 2: traverse (traversable)

| #   | Call pattern                                             | Expected outcome | Assumption validated                |
| --- | -------------------------------------------------------- | ---------------- | ----------------------------------- |
| 38  | `traverse(\|x: i32\| Some(x + 1), Ok::<i32, String>(5))` | `Some(Ok(6))`    | Multi-brand traverse                |
| 39  | `traverse(\|x: i32\| None::<i32>, Ok::<i32, String>(5))` | `None`           | Multi-brand traverse, inner failure |

### 2.7 Phase 2: lift2

| #   | Call pattern                                                                            | Expected outcome | Assumption validated             |
| --- | --------------------------------------------------------------------------------------- | ---------------- | -------------------------------- |
| 40  | `lift2(\|a: i32, b: i32\| a + b, Ok::<i32, String>(1), Ok::<i32, String>(2))`           | `Ok(3)`          | Multi-brand lift2                |
| 41  | `lift2(\|a: i32, b: i32\| a + b, Ok::<i32, String>(1), Err::<i32, String>("x".into()))` | `Err("x")`       | Multi-brand lift2, short-circuit |

### 2.8 Phase 2: apply (semiapplicative)

| #   | Call pattern                                              | Expected outcome | Assumption validated                   |
| --- | --------------------------------------------------------- | ---------------- | -------------------------------------- |
| 42  | `apply(Ok::<CloneFn, String>(f), Ok::<i32, String>(5))`   | `Ok(f(5))`       | Dual-Slot-bound inference (Decision H) |
| 43  | `apply(&Ok::<CloneFn, String>(f), &Ok::<i32, String>(5))` | `Ok(f(5))`       | Ref + dual-Slot-bound                  |

Note: exact `apply` call syntax depends on FnBrand and CloneFn
types. Adjust based on the actual dispatch module signature.

### 2.9 Phase 2: closureless multi-brand (explicit only)

| #   | Call pattern                                                            | Expected outcome | Assumption validated                 |
| --- | ----------------------------------------------------------------------- | ---------------- | ------------------------------------ |
| 44  | `explicit::join::<ResultErrAppliedBrand<String>, _, _, _>(Ok(Ok(5)))`   | `Ok(5)`          | Closureless multi-brand via explicit |
| 45  | `explicit::alt::<ResultErrAppliedBrand<String>, _, _, _>(Ok(1), Ok(2))` | `Ok(1)`          | Closureless multi-brand via explicit |

### 2.10 Phase 2: closureless single-brand (inference)

| #   | Call pattern                | Expected outcome | Assumption validated                           |
| --- | --------------------------- | ---------------- | ---------------------------------------------- |
| 46  | `join(Some(Some(5)))`       | `Some(5)`        | Closureless single-brand still infers via Slot |
| 47  | `alt(None::<i32>, Some(5))` | `Some(5)`        | Closureless single-brand still infers via Slot |

Note: tests 46-47 overlap with non-regression tests 9, 16. They
are included here explicitly to confirm these still work AFTER the
Slot migration (not just before).

## 3. Compile-fail UI tests (negative cases)

Each is a separate `.rs` file under `tests/ui/` with a
corresponding `.stderr` snapshot.

### 3.1 Phase 1

| #   | File                         | Call pattern                               | Expected error                                |
| --- | ---------------------------- | ------------------------------------------ | --------------------------------------------- |
| 48  | `multi_brand_diagonal.rs`    | `map(\|x: i32\| x + 1, Ok::<i32, i32>(5))` | Ambiguous Slot impls; suggest `explicit::map` |
| 49  | `multi_brand_unannotated.rs` | `map(\|x\| x + 1, Ok::<i32, String>(5))`   | Cannot infer Brand; annotate closure input    |
| 50  | `double_ref.rs`              | `map(\|x: &i32\| *x + 1, &&Some(5))`       | FunctorDispatch Ref impl does not match `&&T` |

## 4. Other multi-brand types

The plan lists five multi-brand types. Tests above focus on Result.
At least one test per other multi-brand type confirms the pattern
generalises:

### 4.1 Pair

| #   | Call pattern                                      | Expected outcome         |
| --- | ------------------------------------------------- | ------------------------ |
| 51  | `map(\|x: i32\| x + 1, Pair::new(5, "hello"))`    | Maps over first element  |
| 52  | `map(\|s: &str\| s.len(), Pair::new(5, "hello"))` | Maps over second element |

### 4.2 Tuple2

| #   | Call pattern                              | Expected outcome         |
| --- | ----------------------------------------- | ------------------------ |
| 53  | `map(\|x: i32\| x + 1, (5, "hello"))`     | Maps over first element  |
| 54  | `map(\|s: &&str\| s.len(), (5, "hello"))` | Maps over second element |

### 4.3 ControlFlow

| #   | Call pattern                                      | Expected outcome                 |
| --- | ------------------------------------------------- | -------------------------------- |
| 55  | `map(\|x: i32\| x + 1, ControlFlow::Continue(5))` | Maps based on closure input type |

### 4.4 TryThunk

| #   | Call pattern                                | Expected outcome           |
| --- | ------------------------------------------- | -------------------------- |
| 56  | `map(\|x: i32\| x + 1, try_thunk_ok_value)` | Maps over the success slot |

Note: tests 51-56 depend on which type class instances exist for
each multi-brand type. Verify against the library's actual
instances. Some types may only have Functor (not Monad, Foldable,
etc.), limiting the operations that can be tested.

## 5. Do-notation (Decision K audit)

| #   | Call pattern                                                                                            | Expected outcome | Assumption validated                 |
| --- | ------------------------------------------------------------------------------------------------------- | ---------------- | ------------------------------------ |
| 57  | `m_do!(ResultErrAppliedBrand<String> { x: i32 <- Ok::<i32, String>(5); pure(x + 1) })`                  | `Ok(6)`          | Explicit-mode m_do! with multi-brand |
| 58  | `a_do!(ResultErrAppliedBrand<String> { x <- Ok::<i32, String>(5); y <- Ok::<i32, String>(10); x + y })` | `Ok(15)`         | Explicit-mode a_do! with multi-brand |

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
| Val + multi-brand inference works via closure         | 20-22, 26-29, 31-32, 34-42       |
| Ref + multi-brand inference works via closure         | 23-25, 30, 33, 43                |
| Generic fixed parameter resolves correctly            | 26-30                            |
| Diagonal Result<T,T> is a compile error               | 48                               |
| Unannotated closure on multi-brand is a compile error | 49                               |
| Double reference &&T is a compile error               | 50                               |
| Closureless single-brand infers via Slot              | 46-47                            |
| Closureless multi-brand needs explicit::              | 44-45                            |
| Pattern generalises to all 5 multi-brand types        | 51-56                            |
| Do-notation explicit mode works with multi-brand      | 57-58                            |
| Existing single-brand calls are not regressed         | 1-19                             |
| Marker-via-Slot resolves Val/Ref correctly            | 3-4, 23-25, 30, 33, 43           |
| Arity-2 (bimap) works for multi-brand                 | 34-35                            |
