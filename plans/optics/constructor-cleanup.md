# Optics Constructor Cleanup

## Background

Commit `a9656e3` introduced "prime" constructors for `Lens`, `LensPrime`,
`AffineTraversal`, `AffineTraversalPrime`, and `PrismPrime` that avoid the
`S: Clone` requirement present in the old `new` constructors.  The internal
representation of these types was simultaneously changed: instead of storing
separate getter and setter fields, they now store a single `to` function that
returns both the focus and a setter closure together (matching PureScript's
`lens'` / `affineTraversal'` / `prism'` encodings).

The legacy `new` constructors were kept alongside the new prime constructors,
but they now just adapt the old two-function interface to the new internal `to`
field.  This leaves the API in an inconsistent transitional state.

## What the diff introduced

| Type | Legacy constructor | New prime constructor |
|------|-------------------|-----------------------|
| `Lens` | `new(view, set) where S: Clone` | `lens_prime(to)` |
| `LensPrime` | `new(view, set) where S: Clone` | `lens_prime(to)` |
| `AffineTraversal` | `new(preview, set) where S: Clone` | `affine_traversal_prime(to)` |
| `AffineTraversalPrime` | `new(preview, set) where S: Clone` | `affine_traversal_prime(to)` |
| `PrismPrime` | `new(preview: S->Option<A>, review) where S: Clone` | `new_with(preview: S->Result<A,S>, review)` |
| `Grate` / `GratePrime` | (no change — only one constructor, signature updated in place) | — |

The `Optic` trait impls and concrete method impls (`view`, `set`, `preview`,
`modify`, etc.) on all these types now work without `S: Clone`.

## Findings: do the new constructors subsume the legacy ones?

Yes, completely.  Every value constructable via the legacy `new` can be
constructed via the prime constructor — the legacy `new` body literally just
wraps the prime encoding with an `s.clone()`.  The new constructors accept
strictly more types (those without `Clone`) and produce identical results for
`Clone` types.

## Bugs introduced during the transition

### Bug 1 — `PrismPrime` trait impls still carry `S: Clone`

The `Optic for PrismPrime` impl (prism.rs:653) correctly has no `S: Clone`.
But five trait impls that delegate to it still require `S: Clone` even though
they perform no cloning themselves:

- `PrismOptic for PrismPrime`   (prism.rs:705)
- `TraversalOptic for PrismPrime` (prism.rs:748)
- `FoldOptic for PrismPrime`    (prism.rs:791)
- `SetterOptic for PrismPrime`  (prism.rs:835)
- `ReviewOptic for PrismPrime`  (prism.rs:878)

**Effect:** a `PrismPrime` created with `new_with` (no `S: Clone`) cannot be
used as a `TraversalOptic`, `FoldOptic`, etc., even though the underlying
`Optic` impl works fine.  The `S: Clone` constraint is simply vestigial.

**Fix:** remove `S: Clone` from all five impls.

### Bug 2 — `PrismPrime::modify` still requires `S: Clone`

```rust
// current (prism.rs:635)
pub fn modify(&self, s: S, f: impl Fn(A) -> A) -> S where S: Clone {
    match self.preview(s.clone()) {
        Some(a) => self.review(f(a)),
        None => s,
    }
}
```

The internal `preview_fn` now has type `S -> Result<A, S>`, so the structure
does not need to be cloned.  Compare with the already-correct
`AffineTraversalPrime::modify` (affine.rs:620):

```rust
pub fn modify(&self, s: S, f: impl Fn(A) -> A) -> S {
    match (self.to)(s) {
        Ok((a, set)) => set(f(a)),
        Err(s) => s,
    }
}
```

**Fix:** rewrite `PrismPrime::modify` using `self.preview_fn` directly:

```rust
pub fn modify(&self, s: S, f: impl Fn(A) -> A) -> S {
    match (self.preview_fn)(s) {
        Ok(a) => (self.review_fn)(f(a)),
        Err(s) => s,
    }
}
```

## API inconsistencies

### Inconsistency 1 — naming: `Lens::lens_prime` vs `LensPrime`

`Lens::lens_prime` is a constructor on `Lens` that returns a `Lens`, not a
`LensPrime`.  In PureScript `lens'` is simply a variant of `lens` with a
different interface, but in Rust the `'` suffix has been materialised as a
separate type (`LensPrime`).  Seeing `Lens::lens_prime` a reader expects it to
return `LensPrime`.

Same problem on `AffineTraversal::affine_traversal_prime` (returns
`AffineTraversal`, not `AffineTraversalPrime`).

### Inconsistency 2 — no consistent naming convention across prime constructors

| Type | Prime constructor name |
|------|------------------------|
| `Lens` | `lens_prime` |
| `LensPrime` | `lens_prime` |
| `AffineTraversal` | `affine_traversal_prime` |
| `AffineTraversalPrime` | `affine_traversal_prime` |
| `PrismPrime` | `new_with` |

`PrismPrime::new_with` follows an entirely different convention.

### Inconsistency 3 — `PrismPrime::new` uses `Option`, `Prism::new` uses `Result`

`Prism::new` takes `S -> Result<A, T>` — already the correct encoding with no
`S: Clone` required.  But `PrismPrime::new` takes `S -> Option<A>` and
requires `S: Clone` to reconstruct the `Err(s)` case.  Users of the
polymorphic `Prism` and the monomorphic `PrismPrime` face different mental
models for the same concept.

### Inconsistency 4 — legacy `new` bodies duplicate logic needlessly

All legacy `new` bodies now manually construct the internal `to` / `preview_fn`
encoding instead of delegating to the prime constructor.  They could be
one-liners:

```rust
// e.g. Lens::new could be:
pub fn new(view: impl 'a + Fn(S) -> A, set: impl 'a + Fn((S, B)) -> T) -> Self
where S: Clone {
    let view = <FnBrand<P> as CloneableFn>::new(view);
    let set  = <FnBrand<P> as CloneableFn>::new(set);
    Self::lens_prime(move |s: S| {
        let s2  = s.clone();
        let set = set.clone();
        (view(s), <FnBrand<P> as CloneableFn>::new(move |b| set((s2.clone(), b))))
    })
}
```

This is a code-quality issue rather than a behaviour bug, but it also means
the legacy constructors are harder to audit.

## Recommendations

### Recommended action (API breaking changes are acceptable)

1. **Remove all legacy `new` constructors** from `Lens`, `LensPrime`,
   `AffineTraversal`, `AffineTraversalPrime`, and `PrismPrime`.

2. **Rename the prime constructors to `new`** so they become the sole
   constructor for each type.  Concretely:
   - `Lens::lens_prime` → `Lens::new`
   - `LensPrime::lens_prime` → `LensPrime::new`
   - `AffineTraversal::affine_traversal_prime` → `AffineTraversal::new`
   - `AffineTraversalPrime::affine_traversal_prime` → `AffineTraversalPrime::new`
   - `PrismPrime::new_with` → `PrismPrime::new`

3. **Optionally** add back ergonomic two-argument convenience constructors
   under a clearly distinct name, e.g.:
   - `Lens::from_view_set(view, set) where S: Clone`
   - `LensPrime::from_view_set(view, set) where S: Clone`
   - `AffineTraversal::from_preview_set(preview, set) where S: Clone`
   - `AffineTraversalPrime::from_preview_set(preview, set) where S: Clone`
   - `PrismPrime::from_option(preview: S->Option<A>, review) where S: Clone`

   The key point is that these are named as convenience helpers, not as the
   primary constructor `new`.

4. **Fix Bug 1**: remove `S: Clone` from `PrismOptic`, `TraversalOptic`,
   `FoldOptic`, `SetterOptic`, and `ReviewOptic` impls for `PrismPrime`.

5. **Fix Bug 2**: rewrite `PrismPrime::modify` to not require `S: Clone`.

### Order of operations

Fix the bugs first (steps 4–5) since they are correctness issues independent
of the naming cleanup.  Then do the rename/remove (steps 1–3) in a single
commit so documentation and examples stay consistent.

## Files to touch

- `fp-library/src/types/optics/lens.rs`
- `fp-library/src/types/optics/affine.rs`
- `fp-library/src/types/optics/prism.rs`
- `docs/optics-analysis.md` (update the issues table and constructor notes)
- Any doc-test examples that use the old `new` / `lens_prime` / `affine_traversal_prime` / `new_with` names

## What is already correct and does not need changes

- `Prism::new(preview: S->Result<A,T>, review)` — no legacy issue, uses the
  right encoding already.
- `Grate::new` and `GratePrime::new` — only one constructor each, no legacy
  parallel; the signature was updated in place.
- All `Optic` / `LensOptic` / `TraversalOptic` / etc. impls on `Lens`,
  `LensPrime`, `AffineTraversal`, `AffineTraversalPrime` — `S: Clone` was
  already removed correctly.
- `AffineTraversalPrime::modify` — already uses the internal encoding without
  `Clone`.
