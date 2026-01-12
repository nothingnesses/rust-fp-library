# Edge Cases Checklist

This checklist tracks the implementation of unit tests for edge cases.

## Option

- [x] `map` on `None`
- [x] `bind` on `None`
- [x] `bind` returning `None`
- [x] `fold_right` on `None`
- [x] `fold_left` on `None`
- [x] `traverse` on `None`
- [x] `traverse` returning `None`

## Vec

- [x] `map` on empty vector
- [x] `bind` on empty vector
- [x] `bind` returning empty vector
- [x] `fold_right` on empty vector
- [x] `fold_left` on empty vector
- [x] `traverse` on empty vector
- [x] `traverse` returning empty vector
- [x] `construct` with empty tail
- [x] `deconstruct` on empty vector

## Identity

- [x] `map`
- [x] `bind`
- [x] `fold_right`
- [x] `fold_left`
- [x] `traverse`

## Result

- [x] `map` on `Err`
- [x] `bind` on `Err`
- [x] `bind` returning `Err`
- [x] `fold_right` on `Err`
- [x] `fold_left` on `Err`
- [x] `traverse` on `Err`
- [x] `traverse` returning `Err`

## Lazy

- [x] `force` memoization (ensure computation runs only once)
- [x] `defer` execution order (ensure computation is deferred)

## Pair

- [x] `map`
- [x] `bind`
- [x] `fold_right`
- [x] `fold_left`
- [x] `traverse`

## Endomorphism

- [x] `append` associativity
- [x] `empty` identity

## Endofunction

- [x] `append` associativity
- [x] `empty` identity

## OnceCell

- [x] `new`
- [x] `set`
- [x] `get`
- [x] `get_or_init`

## OnceLock

- [x] `new`
- [x] `set`
- [x] `get`
- [x] `get_or_init`
