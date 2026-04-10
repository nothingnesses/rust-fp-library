# Dispatch Expansion: Implementation Review 2

Semantic correctness review of the bi-prefixed dispatch traits against PureScript
reference implementations.

## Summary

The dispatch implementations are semantically correct. The type parameter ordering,
fold argument conventions, and Result/Either parameter swap are all handled
consistently and match PureScript's semantics when accounting for the
`ResultBrand::Of<A, B> = Result<B, A>` encoding. One argument-order concern in
`fold_left_with_index` is confirmed to be a non-issue. Several convenience functions
are absent from the dispatch layer but present in the trait modules, which is an
intentional design choice worth documenting.

## 1. Bifunctor: bimap parameter semantics

### PureScript reference

```purescript
class Bifunctor f where
  bimap :: (a -> b) -> (c -> d) -> f a c -> f b d

-- Either instance:
instance bifunctorEither :: Bifunctor Either where
  bimap f _ (Left a)  = Left (f a)    -- first function maps Left
  bimap _ g (Right b) = Right (g b)   -- second function maps Right
```

### Rust implementation

The Rust `Bifunctor` trait signature:

```rust
fn bimap<'a, A, B, C, D>(f: impl Fn(A) -> B, g: impl Fn(C) -> D, p: Of<A, C>) -> Of<B, D>
```

The `ResultBrand` encoding: `Of<A, B> = Result<B, A>`, so `Of<A, C> = Result<C, A>`.
This means `A` is the error type (first position) and `C` is the success type (second
position). The implementation confirms:

```rust
fn bimap(...) {
    match p {
        Ok(c) => Ok(g(c)),   // g maps Ok (second-position, type C)
        Err(a) => Err(f(a)), // f maps Err (first-position, type A)
    }
}
```

This is correct. In PureScript, `bimap f g (Left a) = Left (f a)` maps the first
type parameter with `f`. In Rust, `f` maps `A` (the error type), which occupies
the first position in the Kind encoding. The swap in `Result<B, A>` vs
`Either (Left a) (Right b)` is properly absorbed by the Kind definition.

### Dispatch layer

The `BimapDispatch` trait passes `self.0` as `f` and `self.1` as `g` to
`Brand::bimap(self.0, self.1, fa)`. The tuple convention `(f, g)` matches
the trait signature where `f` handles the first position and `g` handles the
second position. This is consistent with PureScript's `bimap f g`.

**Verdict: Correct.** No parameter swap issue.

### Ref variant

The `RefBifunctor` impl for `ResultBrand` follows the same pattern:
`f` maps `&A` (error), `g` maps `&C` (success). The dispatch passes
`self.0` and `self.1` in the same order. Correct.

## 2. Bifoldable: bi_fold_right argument order

### PureScript reference

```purescript
bifoldr :: (a -> c -> c) -> (b -> c -> c) -> c -> p a b -> c

-- Either instance:
bifoldr f _ z (Left a)  = f a z
bifoldr _ g z (Right b) = g b z
```

### Rust implementation

```rust
fn bi_fold_right(f: impl Fn(A, C) -> C, g: impl Fn(B, C) -> C, z: C, p: Of<A, B>) -> C
```

The `ResultBrand` implementation:

```rust
match p {
    Err(a) => f(a, z),  // f handles error (first position, type A)
    Ok(b) => g(b, z),   // g handles success (second position, type B)
}
```

This matches PureScript: `f` folds over the first type parameter (Left/Err),
`g` folds over the second (Right/Ok). The fold functions take `(element, acc)`
which matches PureScript's `(a -> c -> c)`.

### Dispatch layer

The `BiFoldRightDispatch` passes `self.0` as `f` and `self.1` as `g`, and the free
function signature is `bi_fold_right(fg, z, fa)` matching `bifoldr(f, g, z, p)`.

**Verdict: Correct.**

## 3. Bifoldable: bi_fold_left argument order

### PureScript reference

```purescript
bifoldl :: (c -> a -> c) -> (c -> b -> c) -> c -> p a b -> c

-- Either instance:
bifoldl f _ z (Left a)  = f z a
bifoldl _ g z (Right b) = g z b
```

### Rust implementation

```rust
fn bi_fold_left(f: impl Fn(C, A) -> C, g: impl Fn(C, B) -> C, z: C, p: Of<A, B>) -> C
```

The `ResultBrand` implementation:

```rust
match p {
    Err(a) => f(z, a),
    Ok(b) => g(z, b),
}
```

This matches PureScript: step functions take `(acc, element)`, which corresponds
to `(c -> a -> c)`.

### Dispatch layer

The `BiFoldLeftDispatch` routes `self.0` to `f` and `self.1` to `g`, passing
through to `Brand::bi_fold_left`. The free function takes `(fg, z, fa)`.

**Verdict: Correct.**

## 4. Bifoldable: bi_fold_map

### PureScript reference

```purescript
bifoldMap :: Monoid m => (a -> m) -> (b -> m) -> p a b -> m

-- Either instance:
bifoldMap f _ (Left a)  = f a
bifoldMap _ g (Right b) = g b
```

### Rust implementation

```rust
fn bi_fold_map(f: impl Fn(A) -> M, g: impl Fn(B) -> M, p: Of<A, B>) -> M
```

The `ResultBrand` implementation:

```rust
match p {
    Err(a) => f(a),
    Ok(b) => g(b),
}
```

**Verdict: Correct.** Matches PureScript exactly (modulo the Kind swap).

## 5. Bitraversable: bi_traverse

### PureScript reference

```purescript
class (Bifunctor t, Bifoldable t) <= Bitraversable t where
  bitraverse :: Applicative f => (a -> f c) -> (b -> f d) -> t a b -> f (t c d)

-- Either instance:
bitraverse f _ (Left a)  = Left <$> f a
bitraverse _ g (Right b) = Right <$> g b
```

### Rust implementation

```rust
fn bi_traverse(f, g, p: Of<A, B>) -> F::Of<Of<C, D>>
```

The `ResultBrand` implementation:

```rust
match p {
    Err(a) => F::map(|c| Err(c), f(a)),
    Ok(b) => F::map(|d| Ok(d), g(b)),
}
```

This matches PureScript: `f` handles the first position (Left/Err),
`g` handles the second (Right/Ok), and the result is wrapped in the
applicative context.

### Dispatch layer

The `BiTraverseDispatch` passes `self.0` to `f` and `self.1` to `g`, which is
correct.

**Verdict: Correct.**

## 6. Laws

### Bifunctor laws (documented in Rust)

- Identity: `bimap(identity, identity, p) = p`
- Composition: `bimap(compose(f, g), compose(h, i), p) = bimap(f, h, bimap(g, i, p))`

These match PureScript's Bifunctor laws exactly.

### Bifoldable laws (documented in Rust)

- Consistency: `bi_fold_map(f, g, x) = bi_fold_right(|a, c| append(f(a), c), |b, c| append(g(b), c), empty(), x)`

This matches PureScript's `bifoldMapDefaultR` definition:
`bifoldMapDefaultR f g = bifoldr (append <<< f) (append <<< g) mempty`

### Bitraversable laws (documented in Rust)

- Traverse/sequence consistency: `bi_traverse(f, g, x) = bi_sequence(bimap(f, g, x))`

This matches PureScript's `bitraverseDefault`:
`bitraverseDefault f g t = bisequence (bimap f g t)`

**Verdict: All documented laws are consistent with PureScript.**

## 7. Convenience functions

### PureScript provides

In `Data.Bitraversable`:

- `bisequence` - Yes, present in Rust trait module.
- `ltraverse` (traverse left only) - Present as `traverse_left` in Rust.
- `rtraverse` (traverse right only) - Present as `traverse_right` in Rust.
- `bifor` (flipped bitraverse) - Present as `bi_for` in Rust.
- `lfor` (flipped ltraverse) - Present as `for_left` in Rust.
- `rfor` (flipped rtraverse) - Present as `for_right` in Rust.

In `Data.Bifoldable`:

- `bifold` - Not present in Rust. Minor; easily added as `bi_fold_map(identity, identity, x)`.
- `bitraverse_` (effectful fold ignoring result) - Not present in Rust.
- `bifor_` (flipped bitraverse\_) - Not present in Rust.
- `bisequence_` (sequence ignoring result) - Not present in Rust.
- `biany` / `biall` - Not present in Rust.

### Dispatch layer coverage

The dispatch layer covers only the five core operations:

- `bimap` (BimapDispatch)
- `bi_fold_right` (BiFoldRightDispatch)
- `bi_fold_left` (BiFoldLeftDispatch)
- `bi_fold_map` (BiFoldMapDispatch)
- `bi_traverse` (BiTraverseDispatch)

The convenience functions (`bi_sequence`, `traverse_left`, `traverse_right`,
`bi_for`, `for_left`, `for_right`) exist in the trait modules
(`bitraversable.rs`, `ref_bitraversable.rs`) as free functions but do NOT have
dispatch variants. This means users cannot call `traverse_left` with `&x` to get
automatic ref dispatch; they must use the explicit `ref_bi_traverse_left` function
instead.

### Ref convenience functions

The `ref_bitraversable.rs` module provides ref variants of all convenience
functions:

- `ref_bi_traverse`, `ref_bi_sequence`
- `ref_bi_traverse_left`, `ref_bi_traverse_right`
- `ref_bi_for`, `ref_bi_for_left`, `ref_bi_for_right`

These are comprehensive and match the owned variants.

**Assessment:** The convenience functions are not missing; they are just not
dispatch-unified. Users must choose between `traverse_left` (owned) and
`ref_bi_traverse_left` (ref) explicitly. This is consistent with how the
dispatch layer works for the core operations, where the dispatch is on the
primary operations only. Adding dispatch for convenience functions would be
possible but adds complexity for functions that are trivially composed from
the dispatched primitives.

**Recommendation:** No action needed. The current design is sound. If dispatch
for convenience functions is desired later, it can be added incrementally.

## 8. Result/Either parameter mapping

### The swap

PureScript: `Either a b` has `Left a | Right b`.
Rust: `ResultBrand::Of<A, B> = Result<B, A>`, meaning `A` maps to Err and `B` maps to Ok.

This swap is consistently handled across all implementations:

| Operation  | PureScript `Left a` | Rust `Err(a)` | Function |
| ---------- | ------------------- | ------------- | -------- |
| bimap      | `f a`               | `f(a)`        | `self.0` |
| bifoldr    | `f a z`             | `f(a, z)`     | `self.0` |
| bifoldl    | `f z a`             | `f(z, a)`     | `self.0` |
| bifoldMap  | `f a`               | `f(a)`        | `self.0` |
| bitraverse | `f a`               | `f(a)`        | `self.0` |

In all cases, the first function in the tuple handles the first type parameter (A),
which maps to Err in `Result<B, A>` and Left in `Either a b`. The second function
handles the second type parameter (B/Ok/Right).

**Verdict: The swap is correctly absorbed. No semantic mismatch.**

## 9. FoldableWithIndex: fold_left_with_index argument order

### PureScript reference

```purescript
foldlWithIndex :: (i -> b -> a -> b) -> b -> f a -> b
```

The step function takes `(index, accumulator, element)`.

### Rust implementation

```rust
fn fold_left_with_index(func: impl Fn(Self::Index, B, A) -> B, initial: B, fa: Of<A>) -> B
```

The dispatch layer's closure type:

```rust
F: Fn(Brand::Index, B, A) -> B + 'a
```

This matches PureScript exactly: `(index, accumulator, element) -> accumulator`.

**Verdict: Correct.** The argument order `(index, acc, elem)` matches PureScript's
`(i -> b -> a -> b)`.

## 10. Default implementation consistency

### bi_fold_right default (via bi_fold_map + Endofunction)

The Rust default:

```rust
fn bi_fold_right(f, g, z, p) -> C {
    let endo = Self::bi_fold_map(
        |a| Endofunction::new(|c| f(a, c)),
        |b| Endofunction::new(|c| g(b, c)),
        p,
    );
    endo.0(z)
}
```

PureScript's `bifoldrDefault`:

```purescript
bifoldrDefault f g z p = unwrap (bifoldMap (Endo <<< f) (Endo <<< g) p) z
```

These are semantically equivalent: both convert step functions into endofunctions
via `bifoldMap`, then apply the composed endofunction to the initial value.

### bi_fold_left default (via bi_fold_right + Endofunction)

The Rust default uses `bi_fold_right` with `Semigroup::append(k, current)` to
build up a left-to-right endofunction chain. PureScript's `bifoldlDefault` uses
`Dual <<< Endo <<< flip`. Both achieve the same result of reversing the
right-fold into a left-fold.

**Verdict: Default implementations are consistent with PureScript.**

## Conclusion

All five dispatch traits (BimapDispatch, BiFoldRightDispatch, BiFoldLeftDispatch,
BiFoldMapDispatch, BiTraverseDispatch) are semantically correct against
PureScript's reference implementations. The Result/Either parameter swap is
properly handled through the Kind encoding. Laws are consistent. Convenience
functions are present in trait modules but not dispatch-unified, which is an
acceptable design choice.

No issues requiring code changes were found.
