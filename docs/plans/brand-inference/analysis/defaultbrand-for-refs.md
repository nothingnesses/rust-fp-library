# Analysis: Blanket `DefaultBrand` for References

## Summary

A blanket `impl<'a, T: DefaultBrand> DefaultBrand for &'a T` is feasible,
correct, and necessary for brand inference to work with the Ref dispatch
path. This analysis examines coherence, parameterized types, multi-brand
exclusion, double references, macro generation, mutable references, and
error messages.

## Background

The inference-based `map` function proposed in the plan has the signature:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>,
    fa: FA,
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
    FA: DefaultBrand + 'a,
    <FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>,
```

The `FunctorDispatch` trait has two impls:

- **Val dispatch:** `FA = <Brand as Kind>::Of<'a, A>` (owned, e.g., `Option<A>`).
- **Ref dispatch:** `FA = &'b <Brand as Kind>::Of<'a, A>` (borrowed, e.g., `&Lazy<'a, A, Config>`).

For Ref dispatch, `FA` is `&'b Lazy<'a, A, Config>`. The constraint
`FA: DefaultBrand` requires `DefaultBrand` to be implemented for
`&'b Lazy<'a, A, Config>`. Without the blanket impl, every type that
supports Ref dispatch would need a separate hand-written
`impl DefaultBrand for &T` alongside the owned impl, which is
boilerplate that scales poorly.

The proposed blanket impl:

```rust
impl<'a, T: DefaultBrand + ?Sized> DefaultBrand for &'a T {
    type Brand = T::Brand;
}
```

## 1. Coherence and Orphan Rules

The blanket impl `impl<'a, T: DefaultBrand + ?Sized> DefaultBrand for &'a T`
is a standard Rust pattern and does not violate coherence or orphan rules.

**Why it works:**

- `DefaultBrand` is defined in the fp-library crate.
- `&'a T` is a fundamental type. Rust allows blanket impls over reference
  types for locally-defined traits.
- This is the same pattern used by `std` traits like `Display`, `Debug`,
  `Hash`, `AsRef`, etc., which all have `impl<T: Trait> Trait for &T`
  blanket impls in the standard library.

**Coexistence with concrete impls:**

The blanket impl covers `&Option<A>`, `&Vec<A>`, `&Lazy<'a, A, Config>`,
etc. There must be NO concrete `impl DefaultBrand for &Option<A>` or
similar. This is fine because the blanket impl is the only impl needed
for references; no concrete reference impls should exist. If someone
accidentally writes `impl DefaultBrand for &Option<A>`, the compiler
produces a "conflicting implementations" error, which is the correct
behavior.

The blanket impl does not conflict with concrete impls for owned types
(`impl<A> DefaultBrand for Option<A>`, `impl<A> DefaultBrand for Vec<A>`,
etc.) because `&T` and `T` are distinct types. Rust's coherence checker
treats `impl Trait for T` and `impl Trait for &T` as non-overlapping.

**Conclusion:** No coherence issues. The blanket impl coexists cleanly
with all concrete owned-type impls.

## 2. Interaction with Parameterized Types

Consider `Lazy<'a, A, Config>` with brand `LazyBrand<Config>`:

```rust
impl<'a, A: 'a, Config: LazyConfig + 'a> DefaultBrand for Lazy<'a, A, Config> {
    type Brand = LazyBrand<Config>;
}
```

Through the blanket impl, `&'b Lazy<'a, A, Config>` resolves:

```
<&'b Lazy<'a, A, Config> as DefaultBrand>::Brand
    = <Lazy<'a, A, Config> as DefaultBrand>::Brand   // blanket delegates to T
    = LazyBrand<Config>                                // concrete impl
```

This correctly propagates the `Config` parameter through the reference.

**Lifetime considerations:**

- The reference lifetime `'b` appears only in `&'b T` and does not leak
  into `T::Brand`. The blanket impl's `'a` (the reference lifetime) is
  not used in the associated type, so it is erased cleanly.
- The inner lifetime `'a` in `Lazy<'a, A, Config>` is part of `T` itself
  and flows through the concrete impl normally.
- `LazyBrand<Config>` is `'static` (zero-sized marker type with no
  lifetime parameters), so there is no lifetime conflict between the
  reference's borrow lifetime and the brand.

**Other parameterized types work identically:**

| Reference type               | Resolves to brand         |
| ---------------------------- | ------------------------- |
| `&Lazy<'a, A, Config>`       | `LazyBrand<Config>`       |
| `&TryLazy<'a, A, E, Config>` | `TryLazyBrand<E, Config>` |
| `&Coyoneda<'a, F, A>`        | `CoyonedaBrand<F>`        |
| `&Const<'a, R, A>`           | `ConstBrand<R>`           |

**Conclusion:** The blanket impl correctly propagates parameterized brands
through references with no lifetime issues.

## 3. Multi-Brand Exclusion

Types like `Result<A, E>` deliberately do NOT implement arity-1
`DefaultBrand` because they have multiple brands at that arity.

With the blanket impl:

```
&Result<A, E>: DefaultBrand
    requires Result<A, E>: DefaultBrand    // blanket's T: DefaultBrand bound
    Result<A, E> does NOT implement DefaultBrand
    therefore &Result<A, E> does NOT implement DefaultBrand
```

This is the correct behavior. The blanket impl's `T: DefaultBrand` bound
ensures that `&T` only implements `DefaultBrand` when `T` does. Types
excluded from brand inference remain excluded when referenced.

This applies to all multi-brand types:

| Type                  | `DefaultBrand`? | `&Type: DefaultBrand`?  |
| --------------------- | --------------- | ----------------------- |
| `Option<A>`           | Yes             | Yes (via blanket)       |
| `Vec<A>`              | Yes             | Yes (via blanket)       |
| `Lazy<'a, A, Config>` | Yes             | Yes (via blanket)       |
| `Result<A, E>`        | No              | No (correctly excluded) |
| `(First, Second)`     | No              | No (correctly excluded) |
| `ControlFlow<B, C>`   | No              | No (correctly excluded) |

**Conclusion:** Multi-brand exclusion propagates correctly through the
blanket impl. No additional opt-out is needed for references.

## 4. Double References

With the blanket impl, `&&Option<A>` resolves through two layers:

```
<&&Option<A> as DefaultBrand>::Brand
    = <&Option<A> as DefaultBrand>::Brand     // outer blanket
    = <Option<A> as DefaultBrand>::Brand      // inner blanket
    = OptionBrand                              // concrete impl
```

The blanket impl stacks, and `&&Option<A>` resolves to `OptionBrand`.

**Is this a problem?**

In practice, no. The `FunctorDispatch` Ref impl takes
`&'b <Brand as Kind>::Of<'a, A>`, which is a single reference. The
compiler infers `FA = &'b Option<A>`, not `FA = &&Option<A>`. Double
references would only appear if a user explicitly wrote
`map(f, &&some_option)`, which is not a natural calling convention.

Even if it did appear, the resolution is correct (still `OptionBrand`),
so no unsoundness arises. The `Kind` constraint
`Brand: Kind<Of<'a, A> = FA>` would fail for `FA = &&Option<A>` because
`OptionBrand::Of<'a, A> = Option<A>`, not `&&Option<A>`. This means
the type error would appear at the `Kind` constraint, not at
`DefaultBrand`, which is acceptable.

**Conclusion:** Double references resolve correctly but are rejected
downstream by the `Kind` equality constraint. This is harmless and
produces a reasonable compile error.

## 5. Interaction with `impl_kind!` Generation

The plan says `impl_kind!` should auto-generate `DefaultBrand` impls.
With the blanket `&T` impl, only the owned-type impl needs to be
generated. For example:

```rust
impl_kind! {
    for OptionBrand {
        type Of<'a, A: 'a>: 'a = Option<A>;
    }
}
// Generates:
// impl Kind_cdc7cd43dac7585f for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }
// impl<A> DefaultBrand_cdc7cd43dac7585f for Option<A> { type Brand = OptionBrand; }
```

The macro does NOT need to generate:

```rust
impl<'b, A> DefaultBrand_cdc7cd43dac7585f for &'b Option<A> { ... }
```

The blanket impl covers all reference cases automatically.

**For parameterized brands:**

```rust
impl_kind! {
    impl<Config: LazyConfig> for LazyBrand<Config> {
        type Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>;
    }
}
// Generates the owned impl only:
// impl<'a, A: 'a, Config: LazyConfig> DefaultBrand for Lazy<'a, A, Config> {
//     type Brand = LazyBrand<Config>;
// }
// &Lazy<'a, A, Config> is covered by the blanket impl
```

This means the `impl_kind!` macro logic stays simple: it generates exactly
one `DefaultBrand` impl per `impl_kind!` invocation (unless
`#[no_default_brand]` is specified), and the blanket impl handles all
reference forms.

**Conclusion:** The blanket impl eliminates the need for `impl_kind!` to
generate reference impls. Only owned-type impls are needed. This
simplifies the macro and reduces generated code.

## 6. Mutable References

Should `&mut T` also get a blanket impl?

```rust
impl<'a, T: DefaultBrand + ?Sized> DefaultBrand for &'a mut T {
    type Brand = T::Brand;
}
```

**Recommendation: No, at least not initially.**

Reasons:

1. **The library does not use `&mut` containers in its functional API.**
   All operations are either by-value (consuming the container) or by
   shared reference. The dispatch system has `Val` (owned) and `Ref`
   (shared borrow) markers; there is no `RefMut` marker. Adding
   `DefaultBrand for &mut T` enables nothing in the current design.

2. **Functional programming semantics discourage mutation.** The library's
   design philosophy is value-oriented. `RefFunctor::ref_map` takes `&self`
   on the container, not `&mut self`. There is no `MutFunctor` trait.

3. **If needed later, it can be added without breaking changes.** Adding
   `impl DefaultBrand for &mut T` is a purely additive change. Existing
   code is unaffected because `&mut T` currently does not implement
   `DefaultBrand`, so no existing code depends on it being absent.

4. **Potential for confusion.** If `&mut Option<A>` resolved to
   `OptionBrand`, users might expect `map(f, &mut some_option)` to work,
   but it would fail at the `FunctorDispatch` constraint because there is
   no `MutRef` dispatch impl. The resulting error message would be
   confusing.

**Conclusion:** Do not add `DefaultBrand for &mut T`. It enables nothing
in the current design and could produce confusing errors. Add it later
only if a `MutRef` dispatch path is introduced.

## 7. Error Messages

### When `DefaultBrand` is not implemented

For `Result<A, E>`, which does not implement arity-1 `DefaultBrand`:

```rust
let y = map(|x: i32| x + 1, Ok::<i32, String>(5));
// Error: DefaultBrand is not implemented for Result<i32, String>
```

With the blanket impl, `&Result<A, E>` also fails:

```rust
let y = map(|x: &i32| *x + 1, &Ok::<i32, String>(5));
// Error: DefaultBrand is not implemented for &Result<i32, String>
```

The error mentions `&Result<i32, String>` rather than `Result<i32, String>`.
This is slightly less clear but still accurate.

### Does `#[diagnostic::on_unimplemented]` propagate?

The `#[diagnostic::on_unimplemented]` attribute on `DefaultBrand` applies
to types that directly fail to implement the trait. For `&Result<A, E>`,
the compiler checks whether `&Result<A, E>: DefaultBrand` holds. It finds
the blanket impl `impl<T: DefaultBrand> DefaultBrand for &T` and then
checks whether `Result<A, E>: DefaultBrand` holds. When that inner check
fails, the compiler reports the error on the inner type.

The behavior depends on the Rust compiler version and error reporting:

- **Rust reports the root cause:** The error message says `DefaultBrand`
  is not implemented for `Result<i32, String>` (the inner `T`), and the
  `#[diagnostic::on_unimplemented]` message from the `DefaultBrand` trait
  applies. The note about using `map_explicit` appears. This is the
  ideal case.

- **Rust reports the outer failure:** The error message says `DefaultBrand`
  is not implemented for `&Result<i32, String>`. The
  `#[diagnostic::on_unimplemented]` attribute still fires because
  `&Result<i32, String>` does not implement `DefaultBrand` directly (it
  only would via the blanket, which requires `T: DefaultBrand`). However,
  the `{Self}` placeholder in the diagnostic message expands to
  `&Result<i32, String>`, which includes the reference sigil but is
  still understandable.

In either case, the `#[diagnostic::on_unimplemented]` message provides
useful guidance. The message text should be written to handle both
owned and reference types:

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` has multiple brands and cannot use brand inference",
    note = "use `map_explicit::<YourBrand, _, _, _>(f, x)` to specify the brand explicitly"
)]
pub trait DefaultBrand { ... }
```

This wording works for both `Result<i32, String>` and
`&Result<i32, String>`.

**Testing recommendation:** Before finalizing the diagnostic message,
write a compile-fail test with `trybuild` that verifies the exact error
output for both `map(f, Ok(5))` and `map(|x: &i32| *x, &Ok(5))`. This
ensures the diagnostic propagates correctly through the blanket impl
on the target Rust version.

**Conclusion:** Error messages are adequate. The `#[diagnostic::on_unimplemented]`
attribute fires for both owned and referenced types. The message text
should be written to be natural for both cases.

## Summary of Findings

| Question                        | Answer                                             |
| ------------------------------- | -------------------------------------------------- |
| Coherence/orphan rules?         | No issues. Standard pattern, no conflicts.         |
| Parameterized type propagation? | Works correctly. Config/E params flow through.     |
| Multi-brand exclusion?          | Correctly excluded. `T: DefaultBrand` bound gates. |
| Double references?              | Resolves correctly, rejected by `Kind` constraint. |
| `impl_kind!` interaction?       | Only owned impl needed. Blanket covers references. |
| Mutable references?             | Not needed. No `MutRef` dispatch path exists.      |
| Error messages?                 | Adequate. `on_unimplemented` propagates.           |

**Recommendation:** Implement the blanket `impl<'a, T: DefaultBrand + ?Sized> DefaultBrand for &'a T`
as part of step 2 (Define `DefaultBrand` trait) of the implementation
plan. It is a single line of code that eliminates the need for per-type
reference impls and makes Ref dispatch work with brand inference
automatically.
