# Experimentation: Using `Kind!` Macro in Trait Definitions

## Objective

The goal was to replace the hardcoded, generated `Kind_*` trait names (e.g., `Kind_c3c3610c70409ee6`) in the `fp-library` codebase with invocations of the `Kind!` macro. This would improve readability and maintainability by making the kind signature explicit at the point of use.

## Hypothesis

Since the `Kind!` macro expands to the identifier of the trait (e.g., `Kind!(('a), (A: 'a), ('a))` -> `Kind_c3c3610c70409ee6`), it was hypothesized that this macro could be used directly in the supertrait position of a trait definition.

## Experiments

### 1. Direct Supertrait Usage

We attempted to use the macro directly in the supertrait list:

```rust
pub trait Functor: Kind!(('a), (A: 'a), ('a)) { ... }
```

**Result:** Failed.
**Error:** `error: expected one of '(', '+', '::', '<', '=', 'where', or '{', found '!'`

The Rust parser does not allow macro invocations in the supertrait bounds position. It expects a path or a lifetime.

### 2. Where Clause Usage

We attempted to move the bound to a `where` clause:

```rust
pub trait Functor where Self: Kind!(('a), (A: 'a), ('a)) { ... }
```

**Result:** Failed.
**Error:** `error: expected one of '(', '+', ',', '::', '<', or '{', found '!'`

Similar to the supertrait list, the parser does not allow macro invocations in the type path of a `where` clause bound.

### 3. Macro Wrapper

We attempted to wrap the entire trait definition in a `macro_rules!` macro, hoping that the inner `Kind!` macro would be expanded or handled during the outer macro's expansion:

```rust
macro_rules! with_kind {
    ($($tokens:tt)*) => {
        $($tokens)*
    }
}

with_kind! {
    pub trait Functor: Kind!(('a), (A: 'a), ('a)) { ... }
}
```

**Result:** Failed.
**Error:** `error: expected one of '(', '+', '::', '<', '=', 'where', or '{', found '!'`

The `macro_rules!` expansion simply emits the tokens, and the parser still encounters the `Kind!` invocation in an invalid position. Procedural macros that manually parse and expand the inner macro might work, but that would require significant changes to the macro infrastructure.

### 4. Type and Trait Aliases

We also investigated if aliases could serve as a workaround:

#### Type Alias
```rust
type MyKind = Kind!(('a), (A: 'a), ('a));
pub trait TestTrait1: MyKind { ... }
```
**Result:** Failed.
**Error:** `expected a type, found a trait` (for the alias definition) and `expected trait, found type alias` (for the usage).
Type aliases cannot alias traits directly, nor can they be used as supertraits.

#### Trait Alias (Unstable)
```rust
#![feature(trait_alias)]
trait MyKind2 = Kind!(('a), (A: 'a), ('a));
pub trait TestTrait2: MyKind2 { ... }
```
**Result:** Failed.
**Error:** `expected one of '(', '+', '::', ';', '<', or 'where', found '!'`
Even with the unstable `trait_alias` feature, the parser does not allow macro invocations on the right-hand side of the alias definition.

## Conclusion

It is **not possible** to use the `Kind!` macro directly in trait definitions (supertrait bounds, where clauses) or indirectly via type/trait aliases due to current Rust syntax limitations. The parser consistently rejects macro invocations in these positions.

To achieve the desired outcome, one would likely need a procedural macro that generates the entire trait definition (e.g., `def_trait!`), which would handle the expansion of the kind signature internally. However, this is a more invasive change than simply using the existing `Kind!` macro.
