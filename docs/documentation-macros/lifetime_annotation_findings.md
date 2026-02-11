# Lifetime Annotation in Rust Function Calls: Findings

## Summary

**Question:** Can you explicitly annotate lifetimes when calling a function that has a lifetime as one of its type parameters?

**Answer:** **It depends** on whether the lifetime is "late bound" or "early bound".

## Late Bound vs Early Bound Lifetimes

### Late Bound Lifetimes ❌ Cannot Be Explicitly Annotated

Late bound lifetimes only appear in function parameters and return types. These **cannot** be explicitly specified when calling the function.

```rust
// Late bound lifetime
fn identity<'a>(x: &'a str) -> &'a str {
    x
}

// ❌ This fails with error E0794:
// "cannot specify lifetime arguments explicitly if late bound lifetime parameters are present"
// let result = identity::<'_>(&s);

// ✅ This works - lifetime is inferred
let result = identity(&s);
```

### Early Bound Lifetimes ✅ Can Be Explicitly Annotated

Early bound lifetimes appear in positions that require them to be known at the call site, such as:
- In `where` clauses
- In trait bounds
- With const generics
- In type-level positions

```rust
// Early bound lifetime (appears in where clause)
fn early_bound<'a, T>(x: &'a T) -> &'a T 
where
    T: 'a,
{
    x
}

// ✅ This works - lifetime can be explicitly specified
let result = early_bound::<'_, i32>(&x);

// ✅ Can also just specify type, let lifetime be inferred
let result = early_bound::<i32>(&x);

// ✅ Can even specify 'static explicitly
let result = early_bound::<'static, &str>(&s);
```

## What Makes a Lifetime Early Bound?

A lifetime becomes early bound when it appears in:

1. **Where clauses:**
   ```rust
   fn func<'a, T>(x: &'a T) -> &'a T where T: 'a { x }
   ```

2. **Trait bounds:**
   ```rust
   fn func<'a, T: 'a>(x: &'a T) -> &'a T { x }
   ```

3. **Combined with const generics:**
   ```rust
   fn func<'a, T, const N: usize>(x: &'a [T; N]) -> &'a [T; N] where T: 'a { x }
   ```

4. **Multiple constraints:**
   ```rust
   fn func<'a, T>(x: &'a T) -> &'a T where T: Debug + 'a { x }
   ```

## Test Results

All tests in [`lifetime_turbofish_test.rs`](../../fp-library/tests/lifetime_turbofish_test.rs) pass successfully:

```
running 12 tests
test tests::test_early_bound_complex_explicit ... ok
test tests::test_early_bound_const_explicit ... ok
test tests::test_early_bound_static ... ok
test tests::test_early_bound_const_implicit ... ok
test tests::test_early_bound_trait_explicit ... ok
test tests::test_early_bound_trait_implicit ... ok
test tests::test_early_bound_where_explicit_type_only ... ok
test tests::test_early_bound_where_explicit_underscore ... ok
test tests::test_early_bound_where_implicit ... ok
test tests::test_late_bound_implicit ... ok
test tests::test_late_bound_multiple_implicit ... ok
test tests::test_late_bound_type_explicit ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

## Practical Implications

1. **Most common case:** Function lifetimes are usually late bound, so you typically cannot explicitly annotate them
2. **Type parameters:** You can always explicitly specify type parameters even when lifetimes are late bound
3. **Workaround:** If you need to explicitly specify lifetimes, add a `where T: 'a` clause to make the lifetime early bound
4. **Inference:** In most cases, Rust's lifetime inference works perfectly, so explicit annotation is rarely needed

## References

- Rust compiler error E0794: "cannot specify lifetime arguments explicitly if late bound lifetime parameters are present"
- The distinction between late and early bound lifetimes is a Rust implementation detail related to higher-ranked trait bounds and borrow checking
