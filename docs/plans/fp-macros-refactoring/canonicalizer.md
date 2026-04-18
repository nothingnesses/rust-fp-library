# Canonicalizer improvements

All items in this document change the hash output and require a
coordinated version bump. They should be landed together in a single
commit to avoid multiple Kind trait name changes.

## Current Kind trait names

These are the current `trait_kind!` signatures and their generated
`Kind_` / `InferableBrand_` trait names. After applying the changes
below, every hash will change. The old names must be replaced with
the new ones across the codebase (source, tests, snapshots, docs).

| Signature                       | Hash               |
| ------------------------------- | ------------------ |
| `type Of<A>`                    | `ad6c20556a82a1f0` |
| `type Of<'a>`                   | `090267395a66af5f` |
| `type Of<'a, A>`                | `7d15082b8d266693` |
| `type Of<'a, A: 'a>: 'a`        | `cdc7cd43dac7585f` |
| `type Of<A, B>`                 | `5b1bcedfd80bdc16` |
| `type Of<'a, A, B>`             | `140eb1e35dc7afb3` |
| `type Of<'a, A, B>: 'a`         | `f910c70f664f876a` |
| `type Of<'a, A: 'a, B: 'a>: 'a` | `266801a817966495` |

The most commonly used hash is `cdc7cd43dac7585f` (`type Of<'a, A: 'a>:
'a`), which appears in most dispatch traits and type class
implementations.

Both `Kind_{hash}` and `InferableBrand_{hash}` use the same hash for
a given signature.

## Correctness fixes

### 1. visit_path ignores qself

A qualified type like `<T as Iterator>::Item` has `qself = T` and
`path = Iterator::Item`. The canonicalizer only processes the path
segments, producing `Iterator::Item` and losing the `T` part. Two
different qualified types could collide if they share the same path
suffix.

**Fix:** Canonicalize the qself type and include it in the output,
e.g., `<T0 as Iterator>::Item`.

### 2. visit_array doesn't canonicalize the length expression

`quote!(#array.len)` interpolates the entire `TypeArray` followed by
literal `.len` tokens, producing strings like `[T;5].len` instead of
just `5`. This also leaks un-canonicalized type parameter names into
the output, breaking equivalence for renamed params.

**Fix:** Extract the length expression properly:

```rust
let len_expr = &array.len;
let len = quote!(#len_expr).to_string().replace(" ", "");
```

### 3. Const expressions aren't canonicalized

`GenericArgument::Const(expr)` at line 188 stringifies the expression
via `quote!(#expr).to_string()` without mapping type parameters through
the canonicalizer. If a const expression referenced a type parameter,
the raw name would leak through.

**Fix:** Walk the expression tokens and substitute mapped type parameter
names. Since expressions can be arbitrarily complex, a pragmatic
approach is to tokenize, scan for idents that match the type_map, and
replace them with their canonical form.

## Structural improvements

### 4. Duplicated path segment processing

`canonicalize_bound` (lines 117-148) and `visit_path` (lines 224-251)
both iterate path segments with nearly identical
`None`/`AngleBracketed`/`Parenthesized` handling. The only difference
is `canonicalize_bound` prefixes the result with `"t"`.

**Fix:** Extract a shared `canonicalize_path_segments` method and call
it from both sites.

### 5. canonicalize_type is a redundant wrapper

```rust
fn canonicalize_type(&mut self, ty: &Type) -> Result<String> {
    self.visit(ty)
}
```

This adds no value. Replace all call sites with `self.visit(ty)` and
remove the method.

### 6. BTreeMap where HashMap suffices

The `lifetime_map` and `type_map` fields use `BTreeMap<String, usize>`.
The ordering guarantee is never used (maps are only used for lookup, not
iteration order). `HashMap` would be more appropriate and slightly
faster for lookups.

## Missing type support

These type variants currently fall through to `default_output` (which
returns an error) but could realistically appear in Kind signature
bounds or type arguments.

### 7. BareFn (fn(A) -> B)

Structurally identical to `Parenthesized` path arguments, which the
canonicalizer already handles. Would canonicalize inputs and output the
same way.

### 8. TraitObject (dyn Iterator<Item = A>)

Would canonicalize each trait bound using the existing
`canonicalize_bound` method. Needs to sort bounds for determinism (same
as `canonicalize_bounds`).

### 9. ImplTrait (impl Clone)

Structurally identical to trait objects for canonicalization. Same
approach as TraitObject.

### 10. Paren ((T))

Parenthesized type. Trivially delegates to the inner type.
