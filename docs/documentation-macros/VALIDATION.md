# Document Module Validation

## Overview

The `document_module` attribute macro now includes built-in validation to ensure that impl blocks and methods have appropriate documentation attributes. This helps maintain consistent documentation across your codebase.

## Validation Rules

### Impl Block Validation

An impl block should have:
- `#[document_type_parameters]` if it has type parameters
- `#[document_parameters]` if it contains methods with receiver parameters (self, &self, &mut self, etc.)

### Method Validation

A method should have:
- `#[document_signature]` - always recommended for documenting the Hindley-Milner signature
- `#[document_type_parameters]` if it has type parameters
- `#[document_parameters]` if it has non-receiver parameters

## Validation Modes

### Default Mode (Warn)

By default, `document_module` emits compile-time errors for missing documentation attributes:

```rust
#[document_module]
mod my_module {
    pub struct MyType;
    
    // WARNING: Impl block contains methods with receiver parameters
    // but no #[document_parameters] attribute
    impl MyType {
        // WARNING: Method `new` should have #[document_signature] attribute
        pub fn new() -> Self {
            Self
        }
        
        // WARNING: Method `process` has type parameters but no #[document_type_parameters]
        // WARNING: Method `process` has parameters but no #[document_parameters]
        // WARNING: Method `process` should have #[document_signature] attribute
        pub fn process<T>(&self, value: T) -> T {
            value
        }
    }
}
```

### Disabling Validation

You can disable validation warnings using the `no_validation` option:

```rust
#[document_module(no_validation)]
mod my_module {
    pub struct MyType;
    
    // No warnings will be emitted
    impl MyType {
        pub fn new() -> Self {
            Self
        }
    }
}
```

### Explicit Warn Mode

You can explicitly enable validation (same as default):

```rust
#[document_module(warn)]
mod my_module {
    // ...
}
```

## Properly Documented Example

Here's an example of a fully documented module that won't produce warnings:

```rust
#[document_module]
mod my_module {
    pub struct MyType;
    
    #[document_parameters("Self" = "The MyType instance")]
    impl MyType {
        #[document_signature]
        pub fn new() -> Self {
            Self
        }
        
        #[document_signature]
        #[document_type_parameters("The value type")]
        #[document_parameters("value" = "The value to process")]
        pub fn process<T>(&self, value: T) -> T {
            value
        }
    }
}
```

## Rationale

This validation feature helps:

1. **Maintain Consistency**: Ensures all impl blocks and methods follow the same documentation conventions
2. **Catch Omissions**: Identifies missing documentation attributes early in development
3. **Improve Code Quality**: Encourages thorough documentation of type parameters and function parameters
4. **Better Documentation**: Helps generate more complete API documentation

## Migration Guide

If you have existing code that triggers validation warnings:

1. **Option 1**: Add the missing documentation attributes as suggested by the warnings
2. **Option 2**: Add `no_validation` to your `document_module` attributes to suppress warnings temporarily

Example migration:

```rust
// Before (will emit warnings)
#[document_module]
mod my_module {
    impl MyType {
        pub fn foo(&self) { }
    }
}

// Option 1: Add documentation
#[document_module]
mod my_module {
    #[document_parameters("Self" = "The MyType instance")]
    impl MyType {
        #[document_signature]
        pub fn foo(&self) { }
    }
}

// Option 2: Disable validation temporarily
#[document_module(no_validation)]
mod my_module {
    impl MyType {
        pub fn foo(&self) { }
    }
}
```

## Implementation Details

The validation system uses the [`ErrorCollector`](../src/core/error_handling.rs) to accumulate warnings and emit them as compile-time errors. It performs validation recursively on nested modules to ensure comprehensive coverage.

The validation logic is implemented in [`validation.rs`](../src/documentation/validation.rs) and integrated into the document_module macro in [`document_module.rs`](../src/documentation/document_module.rs).
