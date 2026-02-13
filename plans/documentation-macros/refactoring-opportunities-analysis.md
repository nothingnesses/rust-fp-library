# Code Architecture Analysis & Refactoring Opportunities

**Date:** 2026-02-11  
**Scope:** `fp-macros/src/core`, `fp-macros/src/support`, `fp-macros/src/analysis`, `fp-macros/src/documentation/generation.rs`, `fp-macros/src/conversion/patterns.rs`

## Executive Summary

This document analyzes repeated architectures, logic, and code patterns across the fp-macros codebase, identifying refactoring opportunities with detailed trade-off analysis. The codebase shows good separation of concerns but has several patterns that could benefit from consolidation.

**Key Findings:**

- 7 refactoring opportunities identified
- 2 high-priority improvements recommended
- 2 medium-priority consolidations suggested
- 3 areas already well-factored (no changes needed)

---

## Table of Contents

1. [Error Collection Pattern](#1-error-collection-pattern-critical)
2. [Attribute Parsing Pattern](#2-attribute-parsing-pattern-high-priority)
3. [Documentation Generation Pattern](#3-documentation-generation-pattern-medium-priority)
4. [Type Visitor Pattern](#4-type-visitor-pattern-consolidation-low-priority)
5. [Config Passing Pattern](#5-config-passing-pattern-medium-priority)
6. [Validation Logic Duplication](#6-validation-logic-duplication-medium-priority)
7. [Pattern Detection](#7-pattern-detection-duplication-low-priority)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Trade-off Matrix](#trade-off-matrix)

---

## 1. Error Collection Pattern (CRITICAL)

### Current State

Multiple modules independently implement error accumulation with similar patterns.

**Locations:**

- `fp-macros/src/core/error_handling.rs:229-270` - `ErrorCollector` struct defined
- `fp-macros/src/documentation/generation.rs:6-7` - `ErrorCollector` imported and used
- Manual error accumulation scattered across multiple files

**Current Pattern:**

```rust
// Manual approach (scattered)
let mut errors = Vec::new();
// ... collect errors ...
if errors.is_empty() {
    Ok(())
} else {
    Err(combine_errors(errors))
}

// ErrorCollector approach (better, but limited use)
let mut errors = ErrorCollector::new();
errors.push(error);
errors.extend(other_errors);
errors.finish()
```

**Problem:** Error collection logic is duplicated, and the `ErrorCollector` utility is underutilized.

### Refactoring Approaches

#### Option A: Centralized Error Collection Trait ⭐ RECOMMENDED

Create a trait for error-collecting operations:

```rust
// In fp-macros/src/core/error_handling.rs

pub trait CollectErrors {
    /// Execute a fallible operation, collecting any errors
    fn collect<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce() -> Result<T, syn::Error>;

    /// Execute a fallible operation with context
    fn collect_with_context<F, T>(&mut self, context: &str, f: F) -> Option<T>
    where
        F: FnOnce() -> Result<T, syn::Error>;
}

impl CollectErrors for ErrorCollector {
    fn collect<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce() -> Result<T, syn::Error>
    {
        match f() {
            Ok(value) => Some(value),
            Err(e) => {
                self.push(e);
                None
            }
        }
    }

    fn collect_with_context<F, T>(&mut self, context: &str, f: F) -> Option<T>
    where
        F: FnOnce() -> Result<T, syn::Error>
    {
        match f() {
            Ok(value) => Some(value),
            Err(e) => {
                let contextualized = syn::Error::new(
                    e.span(),
                    format!("{}: {}", context, e)
                );
                self.push(contextualized);
                None
            }
        }
    }
}

// Usage example
let mut errors = ErrorCollector::new();
if let Some(sig) = errors.collect(|| parse_signature(&item)) {
    if let Some(docs) = errors.collect(|| generate_docs(&sig)) {
        // Use docs
    }
}
errors.finish()?;
```

**Pros:**

- Eliminates repeated error collection logic
- Type-safe and composable
- Easy to test in isolation
- Natural Rust idiom (Result -> Option with side effects)
- Chainable operations

**Cons:**

- Adds one abstraction layer
- May be slightly more verbose for simple cases
- Requires understanding trait-based error collection

**Impact:**

- ~50 lines of duplicated error handling can be eliminated
- Improved consistency across modules
- Better error context handling

---

#### Option B: Error Accumulator Monad

Build a monadic error accumulator:

```rust
pub struct ErrorAccumulator<T> {
    value: Option<T>,
    errors: Vec<syn::Error>,
}

impl<T> ErrorAccumulator<T> {
    pub fn new(value: T) -> Self {
        Self { value: Some(value), errors: Vec::new() }
    }

    pub fn map<U, F>(self, f: F) -> ErrorAccumulator<U>
    where
        F: FnOnce(T) -> Result<U, syn::Error>
    {
        match self.value {
            Some(v) => match f(v) {
                Ok(new_val) => ErrorAccumulator {
                    value: Some(new_val),
                    errors: self.errors,
                },
                Err(e) => ErrorAccumulator {
                    value: None,
                    errors: {
                        let mut errs = self.errors;
                        errs.push(e);
                        errs
                    },
                },
            },
            None => ErrorAccumulator {
                value: None,
                errors: self.errors,
            },
        }
    }

    pub fn finish(self) -> Result<T, syn::Error> {
        if self.errors.is_empty() {
            Ok(self.value.unwrap())
        } else {
            Err(ErrorCollector::combine_errors(self.errors))
        }
    }
}
```

**Pros:**

- Composable operations
- Functional style matches HM signature generation philosophy
- Clear separation of success/failure paths
- Elegant chaining

**Cons:**

- Higher learning curve for team members
- More complex implementation
- May be overkill for procedural macro context
- Difficult to integrate with existing code

**Impact:**

- Requires significant refactoring
- May confuse contributors unfamiliar with monadic patterns

---

### Recommendation

**Choose Option A** (CollectErrors Trait)

**Rationale:**

1. Simpler to understand and implement
2. More Rust-idiomatic (traits over monads)
3. Easier incremental adoption
4. Lower risk, immediate value
5. Natural fit for existing `ErrorCollector` struct

**Implementation Priority:** HIGH (Phase 3 in roadmap)

---

## 2. Attribute Parsing Pattern (HIGH PRIORITY)

### Current State

Repeated attribute extraction and parsing patterns across multiple modules.

**Locations:**

- `fp-macros/src/documentation/generation.rs:138-148` - `parse_attr_or_none` helper
- `fp-macros/src/support/parsing.rs:107-131` - `parse_unique_attr_value`
- `fp-macros/src/support/attributes.rs:91-120` - `remove_attribute_tokens`
- `fp-macros/src/support/attributes.rs:146-158` - `remove_and_parse_attribute`

**Current Pattern:**

```rust
// Pattern 1: Find, remove, parse
let attr_pos = find_attribute(&attrs, "document_use")?;
let tokens = remove_attribute_tokens(&mut attrs, attr_pos)?;
let parsed = syn::parse2::<T>(tokens)?;

// Pattern 2: Find, extract value
let value = parse_unique_attr_value(&attrs, "document_use")?;

// Pattern 3: Combined in generation.rs
fn parse_attr_or_none(
    attrs: &[syn::Attribute],
    name: &str,
    errors: &mut ErrorCollector,
) -> Option<String> {
    parse_unique_attr_value(attrs, name).unwrap_or_else(|e| {
        errors.push(syn::Error::from(e));
        None
    })
}
```

**Problem:**

- Similar logic in 4+ different places
- Inconsistent error handling approaches
- Repeated "find then parse" pattern

### Refactoring Approaches

#### Option A: Unified AttributeParser Struct

```rust
// In fp-macros/src/support/attributes.rs

pub struct AttributeParser<'a> {
    attrs: &'a mut Vec<Attribute>,
}

impl<'a> AttributeParser<'a> {
    pub fn new(attrs: &'a mut Vec<Attribute>) -> Self {
        Self { attrs }
    }

    /// Extract attribute and parse its contents
    pub fn extract_and_parse<T: Parse>(&mut self, name: &str) -> Result<Option<T>> {
        let Some(index) = find_attribute(self.attrs, name) else {
            return Ok(None);
        };

        let tokens = remove_attribute_tokens(self.attrs, index)?;
        let parsed = syn::parse2::<T>(tokens)?;
        Ok(Some(parsed))
    }

    /// Extract a name-value attribute's string value
    pub fn extract_value(&mut self, name: &str) -> Result<Option<String>> {
        parse_unique_attr_value(self.attrs, name)
    }

    /// Check if attribute exists (without removing)
    pub fn has(&self, name: &str) -> bool {
        has_attr(self.attrs, name)
    }
}

// Usage
let mut parser = AttributeParser::new(&mut item.attrs);
if let Some(args) = parser.extract_and_parse::<FieldDocArgs>("document_fields")? {
    // Process args
}
```

**Pros:**

- Single responsibility
- Clear ownership model
- Centralized error handling
- Chainable operations possible

**Cons:**

- Requires mutable reference juggling
- May complicate ownership in some contexts
- Less discoverable than extension trait

**Impact:**

- Moderate refactoring effort
- Clearer attribute manipulation code

---

#### Option B: Extension Trait Pattern ⭐ RECOMMENDED

```rust
// In fp-macros/src/support/attributes.rs

/// Extension trait for attribute manipulation
pub trait AttributeExt {
    /// Find, remove, and parse an attribute in one operation
    fn find_and_remove<T: Parse>(&mut self, name: &str) -> Result<Option<T>>;

    /// Find and extract a name-value attribute's string value
    fn find_value(&self, name: &str) -> Result<Option<String>>;

    /// Find, remove, and extract value (mutable)
    fn find_and_remove_value(&mut self, name: &str) -> Result<Option<String>>;

    /// Check for attribute existence
    fn has_attribute(&self, name: &str) -> bool;
}

impl AttributeExt for Vec<Attribute> {
    fn find_and_remove<T: Parse>(&mut self, name: &str) -> Result<Option<T>> {
        let Some(index) = find_attribute(self, name) else {
            return Ok(None);
        };

        let tokens = remove_attribute_tokens(self, index)?;
        if tokens.is_empty() {
            return Ok(None);
        }

        let parsed = syn::parse2::<T>(tokens)?;
        Ok(Some(parsed))
    }

    fn find_value(&self, name: &str) -> Result<Option<String>> {
        parse_unique_attr_value(self, name)
    }

    fn find_and_remove_value(&mut self, name: &str) -> Result<Option<String>> {
        let Some(index) = find_attribute(self, name) else {
            return Ok(None);
        };

        let attr = self.remove(index);
        if let syn::Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s), ..
            }) = &nv.value
        {
            Ok(Some(s.value()))
        } else {
            Ok(None)
        }
    }

    fn has_attribute(&self, name: &str) -> bool {
        has_attr(self, name)
    }
}

// Usage (very clean!)
use crate::support::attributes::AttributeExt;

if let Some(args) = item.attrs.find_and_remove::<FieldDocArgs>("document_fields")? {
    // Process args
}

let document_use = method.attrs.find_value("document_use")?;
```

**Pros:**

- Natural Rust pattern (like `Iterator` extensions)
- No ownership issues
- Easy to discover via IDE autocomplete
- Familiar pattern for Rust developers
- Works seamlessly with existing code

**Cons:**

- Extension trait pollution (one more trait in scope)
- Requires `use` statement everywhere it's used
- Slightly less flexible than struct-based approach

**Impact:**

- Minimal refactoring effort
- Immediate code clarity improvement
- ~100+ lines of repetitive code eliminated

---

### Recommendation

**Choose Option B** (Extension Trait Pattern)

**Rationale:**

1. Most idiomatic Rust approach
2. Lowest friction for adoption
3. Natural fit with existing `Vec<Attribute>` usage
4. Easy to discover and use
5. Minimal risk

**Implementation Priority:** HIGH (Phase 1 in roadmap)

---

## 3. Documentation Generation Pattern (MEDIUM PRIORITY)

### Current State

All documentation macros follow a similar structure but with unique quirks:

**Locations:**

- `fp-macros/src/documentation/document_signature.rs`
- `fp-macros/src/documentation/document_parameters.rs`
- `fp-macros/src/documentation/document_type_parameters.rs`
- `fp-macros/src/documentation/document_fields.rs`

**Common Structure:**

```rust
pub fn document_X_worker(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // 1. Parse input item
    let mut item = parse_item(item)?;

    // 2. Extract targets (parameters/fields/types)
    let targets = extract_targets(&item)?;

    // 3. Parse documentation arguments
    let args = parse_args(attr)?;

    // 4. Validate counts match
    validate_match(targets, args)?;

    // 5. Generate and insert doc comments
    for (target, arg) in zip(targets, args) {
        let doc = format_parameter_doc(&target, &arg);
        insert_doc_comment(&mut item.attrs, doc, span);
    }

    // 6. Return modified item
    Ok(quote!(#item))
}
```

**Unique Quirks:**

- `document_parameters`: Extracts curried parameters from return types
- `document_fields`: Handles both named and unnamed (tuple) fields
- `document_type_parameters`: Works with generics (lifetimes, types, consts)
- `document_signature`: Complex HM type conversion and Self resolution

### Refactoring Approaches

#### Option A: Template Method Pattern

```rust
// In fp-macros/src/documentation/templates.rs

pub trait DocGenerator {
    type Item: Parse;
    type Target;
    type Args: Parse;

    /// Extract documentation targets from the item
    fn extract_targets(&self, item: &Self::Item) -> Result<Vec<Self::Target>>;

    /// Validate targets against provided arguments
    fn validate(&self, targets: &[Self::Target], args: &Self::Args) -> Result<()> {
        // Default implementation with count checking
        let target_count = targets.len();
        let arg_count = self.arg_count(args);
        if target_count != arg_count {
            return Err(Error::validation(
                Span::call_site(),
                format!("Expected {} arguments, found {}", target_count, arg_count)
            ));
        }
        Ok(())
    }

    /// Generate doc comments from targets and arguments
    fn generate_docs(
        &self,
        targets: Vec<Self::Target>,
        args: Self::Args
    ) -> Result<Vec<Attribute>>;

    /// Template method - the main workflow
    fn generate(
        &self,
        attr: TokenStream,
        item: TokenStream,
    ) -> Result<TokenStream> {
        let mut item = syn::parse2::<Self::Item>(item)?;
        let targets = self.extract_targets(&item)?;
        let args = syn::parse2::<Self::Args>(attr)?;

        self.validate(&targets, &args)?;

        let docs = self.generate_docs(targets, args)?;
        let attrs = self.get_attrs_mut(&mut item);
        attrs.splice(0..0, docs);

        Ok(quote!(#item))
    }

    // Helper methods
    fn arg_count(&self, args: &Self::Args) -> usize;
    fn get_attrs_mut(&self, item: &mut Self::Item) -> &mut Vec<Attribute>;
}

// Example implementation
struct FieldDocGenerator;

impl DocGenerator for FieldDocGenerator {
    type Item = ItemStruct;
    type Target = FieldInfo;
    type Args = FieldDocArgs;

    fn extract_targets(&self, item: &Self::Item) -> Result<Vec<Self::Target>> {
        // Field extraction logic
    }

    fn generate_docs(
        &self,
        targets: Vec<Self::Target>,
        args: Self::Args,
    ) -> Result<Vec<Attribute>> {
        // Doc generation logic
    }

    // ... other methods
}
```

**Pros:**

- DRY principle applied
- Centralized validation logic
- Easier to add new documentation types
- Clear contract for implementations

**Cons:**

- Each doc type has **significant** unique quirks
- Forces awkward abstractions for diverse use cases
- Loss of clarity for straightforward cases
- Generic type gymnastics (`Self::Item`, `Self::Target`, etc.)
- Harder to understand for newcomers

**Impact:**

- High refactoring effort
- Questionable maintainability gain
- May introduce bugs during migration

---

#### Option B: Keep Separate, Extract Common Utilities ⭐ RECOMMENDED

Keep separate worker functions but extract truly shared logic:

```rust
// In fp-macros/src/support/validation.rs (expand existing)

/// Validate that documentation argument count matches target count
pub fn validate_doc_argument_count(
    expected: usize,
    provided: usize,
    span: Span,
    context: &str,
) -> Result<()> {
    if expected != provided {
        return Err(Error::validation(
            span,
            format!(
                "Expected {} {} description{}, found {}",
                expected,
                context,
                if expected == 1 { "" } else { "s" },
                provided
            )
        ));
    }
    Ok(())
}

/// Validate that no documentable items exist (error case for empty functions)
pub fn validate_has_documentable_items(
    count: usize,
    span: Span,
    attr_name: &str,
    item_type: &str,
) -> Result<()> {
    if count == 0 {
        return Err(Error::validation(
            span,
            format!(
                "Cannot use #{attr_name} on {item_type} with no items to document"
            )
        ));
    }
    Ok(())
}

// In fp-macros/src/support/syntax.rs (expand existing)

/// Generate a batch of doc comments and insert them
pub fn insert_doc_comment_batch(
    attrs: &mut Vec<Attribute>,
    docs: Vec<(String, String)>,
    base_index: usize,
) {
    for (i, (name, desc)) in docs.into_iter().enumerate() {
        let doc_comment = format_parameter_doc(&name, &desc);
        let doc_attr: Attribute = parse_quote!(#[doc = #doc_comment]);
        attrs.insert(base_index + i, doc_attr);
    }
}
```

**Pros:**

- Flexibility for each macro's unique needs
- Clear, straightforward code
- Easy to understand and modify
- Low risk refactoring
- Incremental improvement

**Cons:**

- Some duplication remains (but manageable)
- Validation logic still somewhat scattered
- No grand unifying abstraction

**Impact:**

- Low refactoring effort
- Moderate code reuse improvement
- Clear wins without architectural complexity

---

### Recommendation

**Choose Option B** (Extract Common Utilities)

**Rationale:**

1. The unique quirks justify separate implementations
2. Template Method pattern would force awkward abstractions
3. Extracting utilities gives 80% of benefit with 20% of effort
4. Maintains code clarity and understandability
5. Lower risk of introducing bugs

**Implementation Priority:** MEDIUM (Phase 4 in roadmap)

---

## 4. Type Visitor Pattern Consolidation (LOW PRIORITY)

### Current State

Multiple implementations of the `TypeVisitor` trait with different purposes:

**Locations:**

- `fp-macros/src/support/type_visitor.rs` - Trait definition
- `fp-macros/src/conversion/hm_ast_builder.rs` - `HMTypeBuilder` implementation
- `fp-macros/src/support/syntax.rs:350-441` - `CurriedParamExtractor` implementation

**Analysis:**

The `TypeVisitor` trait is well-designed for its purpose:

```rust
pub trait TypeVisitor {
    type Output;

    fn default_output(&self) -> Self::Output;
    fn visit(&mut self, ty: &Type) -> Self::Output;
    fn visit_path(&mut self, type_path: &syn::TypePath) -> Self::Output;
    fn visit_macro(&mut self, type_macro: &syn::TypeMacro) -> Self::Output;
    // ... other visit methods
}
```

**Current Implementations:**

1. **HMTypeBuilder** - Transforms `syn::Type` → `HmAst` (for signature generation)
2. **CurriedParamExtractor** - Collects implicit parameters (side-effect based, Output = `()`)

### Refactoring Approaches

#### Option A: Visitor Combinator Library

```rust
// Hypothetical combinator approach
pub struct ChainVisitor<V1, V2> {
    first: V1,
    second: V2,
}

impl<V1: TypeVisitor, V2: TypeVisitor> TypeVisitor for ChainVisitor<V1, V2> {
    type Output = (V1::Output, V2::Output);
    // ... implementation
}

pub struct FilterVisitor<V, F> {
    visitor: V,
    filter: F,
}

// Usage
let visitor = ChainVisitor::new(
    HMTypeBuilder::new(config),
    CurriedParamExtractor::new(&mut params)
);
```

**Pros:**

- Composable visitors
- Reusable visitor logic
- Functional programming style

**Cons:**

- **Overkill** for current use cases (only 2 implementations)
- Complex type signatures
- Not clear this is actually needed
- Harder to debug
- Over-engineering risk

---

#### Option B: Keep As-Is ⭐ RECOMMENDED

The current pattern is clean and each visitor has a distinct, well-defined purpose.

**Rationale:**

- Only 2 implementations, both with very different needs
- `TypeVisitor` trait is already a good abstraction
- No evidence of duplication or pain points
- Adding combinators would be premature optimization

---

### Recommendation

**Choose Option B** (Keep As-Is)

**Rationale:**

1. Current abstraction level is appropriate
2. No evidence of duplication
3. Each visitor has distinct purpose
4. YAGNI principle (You Aren't Gonna Need It)

**Implementation Priority:** NONE (no changes needed)

---

## 5. Config Passing Pattern (MEDIUM PRIORITY)

### Current State

Config is passed explicitly through many function signatures:

**Locations:**

- Most functions in `conversion/` module
- Most functions in `analysis/` module
- Many functions in `documentation/` module

**Pattern:**

```rust
fn type_to_hm(
    ty: &Type,
    fn_bounds: &HashMap<String, HmAst>,
    generic_names: &HashSet<String>,
    config: &Config,  // <-- Passed explicitly
) -> HmAst

fn analyze_fn_bounds(
    sig: &Signature,
    config: &Config,  // <-- Passed explicitly
) -> HashMap<String, HmAst>

fn generate_signature(
    sig: &Signature,
    config: &Config,  // <-- Passed explicitly
) -> String
```

### Refactoring Approaches

#### Option A: Context Object Pattern

```rust
// In fp-macros/src/core/context.rs (new file)

pub struct MacroContext {
    config: Config,
    errors: ErrorCollector,
    span: Span,
    // Potential caching
    type_cache: HashMap<String, HmAst>,
}

impl MacroContext {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            errors: ErrorCollector::new(),
            span: Span::call_site(),
            type_cache: HashMap::new(),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    // Provide context-aware conversion
    pub fn type_to_hm(&mut self, ty: &Type) -> Result<HmAst> {
        // Use self.config internally
        // Could cache results in self.type_cache
    }

    pub fn finish(self) -> Result<()> {
        self.errors.finish()
    }
}

// Usage
let mut ctx = MacroContext::new(config);
let hm_type = ctx.type_to_hm(&return_type)?;
let signature = ctx.generate_signature(&sig)?;
ctx.finish()?;
```

**Pros:**

- Reduces parameter passing
- Centralizes contextual data
- Natural place for caching/memoization
- Could reduce function signatures

**Cons:**

- Adds **significant** state management complexity
- Makes pure functions stateful (breaks functional paradigm)
- Harder to reason about lifetimes
- Threading context through call stack is tedious
- Loss of functional purity (testing harder)
- Not clear caching is needed

---

#### Option B: Keep Explicit Passing ⭐ RECOMMENDED

Current approach with explicit config passing.

**Rationale:**

- Configuration is essentially global read-only state
- Explicit passing makes dependencies clear
- Easier to test (pure functions)
- Familiar pattern in Rust ecosystem
- No evidence of pain from current approach

---

### Recommendation

**Choose Option B** (Keep Explicit Passing)

**Rationale:**

1. Explicit is better than implicit for configuration
2. Pure functions are easier to test and reason about
3. No evidence that parameter passing is a problem
4. Context objects add complexity without clear benefit
5. Current approach is idiomatic Rust

**Implementation Priority:** NONE (no changes needed)

---

## 6. Validation Logic Duplication (MEDIUM PRIORITY)

### Current State

Similar validation patterns appear across multiple modules:

**Locations:**

- `fp-macros/src/support/validation.rs` - Generic validators
- `fp-macros/src/support/field_docs.rs:106-246` - Field-specific validation
- `fp-macros/src/support/parsing.rs:135-173` - Parameter validation

**Patterns:**

1. **Count Validation** - Ensure provided count matches expected count
2. **Duplicate Detection** - Check for duplicate entries in HashMaps
3. **Completeness Checking** - Verify all expected items are documented
4. **Zero-Size Validation** - Error on empty structs/variants

**Current Implementation:**

```rust
// validation.rs - Generic
pub fn validate_entry_count(...) -> Result<()>
pub fn validate_named_entries(...) -> Result<()>
pub fn check_duplicate_entry(...) -> Result<()>
pub fn validate_not_zero_sized(...) -> Result<()>

// parsing.rs - Parameters (specialized)
pub fn parse_parameter_documentation_pairs(...) -> Result<Vec<(String, DocArg)>> {
    // Contains embedded validation
    if expected == 0 { return Err(...) }
    if expected != found { return Err(...) }
}

// field_docs.rs - Fields (specialized)
impl FieldDocumenter {
    fn process_named_fields(...) -> Result<()> {
        // Contains embedded validation
        validation::validate_named_entries(...)
        validation::check_duplicate_entry(...)
    }
}
```

### Refactoring Approaches

#### Option A: Generic Validation Builder

```rust
// In fp-macros/src/support/validation.rs

pub struct Validator<T> {
    expected: Vec<T>,
    span: Span,
    context: &'static str,
}

impl<T: Eq + Hash + Display> Validator<T> {
    pub fn new(expected: Vec<T>, span: Span, context: &'static str) -> Self {
        Self { expected, span, context }
    }

    pub fn validate_complete<V>(
        &self,
        provided: &HashMap<T, V>
    ) -> Result<()> {
        // Check all expected items present
        for expected_item in &self.expected {
            if !provided.contains_key(expected_item) {
                return Err(Error::validation(
                    self.span,
                    format_missing_doc_error(self.context, &expected_item.to_string())
                ));
            }
        }

        // Check no extra items provided
        for provided_item in provided.keys() {
            if !self.expected.contains(provided_item) {
                return Err(Error::validation(
                    provided_item.span(), // Assumes T has span()
                    format_nonexistent_item_error(
                        self.context,
                        &provided_item.to_string(),
                        &self.expected
                    )
                ));
            }
        }

        Ok(())
    }

    pub fn validate_count(&self, provided: usize) -> Result<()> {
        validate_entry_count(
            self.expected.len(),
            provided,
            self.span,
            self.context
        )
    }
}

// Usage
let validator = Validator::new(
    field_names,
    span,
    "field"
);
validator.validate_complete(&provided_docs)?;
```

**Pros:**

- DRY validation logic
- Type-safe
- Easy to extend
- Centralized error messages

**Cons:**

- Generic constraints may be limiting (e.g., `T: Eq + Hash + Display`)
- Error messages less customizable
- Slightly more complex API
- May not fit all use cases

---

#### Option B: Consolidate into validation.rs ⭐ RECOMMENDED

Move all validation helpers to `validation.rs` and standardize error messages.

**Approach:**

```rust
// In fp-macros/src/support/validation.rs

// Add new validators

/// Validate that documentation arguments are provided (not empty)
pub fn validate_has_arguments(
    count: usize,
    span: Span,
    attr_name: &str,
    item_description: &str,
) -> Result<()> {
    if count == 0 {
        return Err(CoreError::Parse(syn::Error::new(
            span,
            format!(
                "Cannot use #{attr_name} on {item_description} with no items to document"
            ),
        )));
    }
    Ok(())
}

/// Validate parameter documentation count with custom error message
pub fn validate_parameter_doc_count(
    expected: usize,
    provided: usize,
    span: Span,
) -> Result<()> {
    if expected != provided {
        return Err(CoreError::Parse(syn::Error::new(
            span,
            format!(
                "Expected {} description argument{}, found {}",
                expected,
                if expected == 1 { "" } else { "s" },
                provided
            ),
        )));
    }
    Ok(())
}

// Standardize error message formatting

/// Standard error for missing documentation
pub fn error_missing_documentation(
    span: Span,
    item_name: &str,
    context: &str,
) -> CoreError {
    CoreError::Parse(syn::Error::new(
        span,
        format_missing_doc_error(context, item_name)
    ))
}

/// Standard error for duplicate documentation
pub fn error_duplicate_documentation(
    span: Span,
    item_name: &str,
    context: &str,
) -> CoreError {
    CoreError::Parse(syn::Error::new(
        span,
        format_duplicate_doc_error(context, item_name)
    ))
}
```

**Then consolidate usage:**

```rust
// In parsing.rs - Simplify
pub fn parse_parameter_documentation_pairs(
    targets: Vec<String>,
    entries: Vec<DocArg>,
    span: Span,
) -> Result<Vec<(String, DocArg)>> {
    // Use standardized validation
    validate_has_arguments(
        targets.len(),
        span,
        "document_parameters",
        "functions with no parameters"
    )?;

    validate_parameter_doc_count(
        targets.len(),
        entries.len(),
        span
    )?;

    Ok(targets.into_iter().zip(entries).collect())
}
```

**Pros:**

- Keep it simple
- Improve current module
- Standardize error messages
- Easy to maintain
- Low risk

**Cons:**

- Some duplication remains
- No fancy abstraction
- Manual consolidation work

---

### Recommendation

**Choose Option B** (Consolidate into validation.rs)

**Rationale:**

1. Simple, pragmatic approach
2. Improves consistency without over-engineering
3. Easy to review and test
4. Incremental improvement
5. Maintains clarity

**Implementation Priority:** MEDIUM (Phase 2 in roadmap)

---

## 7. Pattern Detection Duplication (LOW PRIORITY)

### Current State

FnBrand and Apply! pattern detection is well-factored:

**Location:**

- `fp-macros/src/conversion/patterns.rs:18-87`

**Implementation:**

```rust
pub struct FnBrandInfo {
    pub inputs: Vec<syn::Type>,
    pub output: syn::Type,
}

pub fn extract_fn_brand_info(
    type_path: &syn::TypePath,
    config: &Config,
) -> Option<FnBrandInfo>

pub fn extract_apply_macro_info(
    type_macro: &syn::TypeMacro
) -> Option<(syn::Type, Vec<syn::Type>)>
```

**Usage:**

- `fp-macros/src/conversion/hm_ast_builder.rs` - Uses both extractors
- `fp-macros/src/support/syntax.rs` - Uses `extract_fn_brand_info`

### Analysis

**This is already well-factored:**

- Pattern detection extracted into dedicated module
- Clear, reusable functions
- No duplication
- Appropriate abstraction level

### Recommendation

**No changes needed** - This code is exemplary.

---

## Implementation Roadmap

### Phase 1: Attribute Extensions

**Priority:** HIGH  
**Risk:** LOW

1. Create `AttributeExt` trait in `support/attributes.rs`
2. Add methods: `find_and_remove()`, `find_value()`, `find_and_remove_value()`
3. Update `generation.rs` to use new trait
4. Update other call sites incrementally
5. Add tests

**Expected Impact:**

- ~100 lines of code eliminated
- Improved attribute manipulation clarity
- Better discoverability

---

### Phase 2: Validation Consolidation

**Priority:** MEDIUM  
**Risk:** LOW

1. Add new validators to `support/validation.rs`:
   - `validate_has_arguments()`
   - `validate_parameter_doc_count()`
   - Standardized error constructors
2. Update `parsing.rs` to use consolidated validators
3. Update `field_docs.rs` to use consolidated validators
4. Standardize error messages across modules
5. Add tests

**Expected Impact:**

- Consistent error messages
- ~50 lines of duplication removed
- Easier to maintain validation logic

---

### Phase 3: Error Collection Trait

**Priority:** HIGH  
**Risk:** LOW

1. Add `CollectErrors` trait to `core/error_handling.rs`
2. Implement for `ErrorCollector`
3. Update `generation.rs` to use trait methods
4. Add context handling methods
5. Add comprehensive tests

**Expected Impact:**

- ~50 lines of error handling eliminated
- Better error context
- More consistent error collection

---

### Phase 4: Documentation Helper Extraction

**Priority:** MEDIUM
**Risk:** LOW

1. Extract common validation helpers (if not done in Phase 2)
2. Extract doc comment batch generation helper
3. Update documentation workers to use helpers
4. Add tests

**Expected Impact:**

- Moderate code reuse improvement
- Clearer documentation generation code

---

## Trade-off Matrix

| Refactoring                  | Code Reduction | Maintainability | Risk   | Learning Curve |
| ---------------------------- | -------------- | --------------- | ------ | -------------- |
| **Error Collection Trait**   | ⭐⭐⭐         | ⭐⭐⭐⭐        | Low    | Low            |
| **Attribute Extension**      | ⭐⭐⭐⭐       | ⭐⭐⭐⭐⭐      | Low    | Low            |
| **Validation Consolidation** | ⭐⭐⭐         | ⭐⭐⭐⭐        | Low    | Low            |
| **Doc Template Pattern**     | ⭐⭐⭐⭐⭐     | ⭐⭐            | Medium | Medium         |
| **Context Object**           | ⭐⭐⭐         | ⭐⭐            | High   | Medium         |
| **Validation Builder**       | ⭐⭐⭐⭐       | ⭐⭐⭐          | Low    | Low            |

### Legend

- ⭐ = Poor
- ⭐⭐ = Below Average
- ⭐⭐⭐ = Average
- ⭐⭐⭐⭐ = Good
- ⭐⭐⭐⭐⭐ = Excellent

---

## Summary & Recommendations

### DO IMPLEMENT (High Value, Low Risk)

1. **Attribute Extension Trait** (Phase 1)

   - Highest value-to-effort ratio
   - Immediate clarity improvement
   - Natural Rust idiom

2. **Validation Consolidation** (Phase 2)

   - Standardizes error messages
   - Low risk, clear benefits
   - Improves maintainability

3. **Error Collection Trait** (Phase 3)
   - Eliminates repetitive error handling
   - Type-safe and composable
   - Natural extension of existing code

### CONSIDER (Medium Value)

4. **Documentation Helper Extraction** (Phase 4)
   - Moderate code reuse benefit
   - Low risk
   - Only if patterns clearly emerge during other refactoring

### DO NOT IMPLEMENT (Low Value or High Risk)

5. **Documentation Template Pattern**

   - Quirks justify separate implementations
   - High complexity for questionable benefit
   - Risk of over-abstraction

6. **Context Object Pattern**

   - Adds complexity without clear benefit
   - Current explicit passing is cleaner
   - No evidence of pain points

7. **Type Visitor Combinators**
   - Only 2 implementations
   - YAGNI - premature optimization
   - Current abstraction is sufficient

---

## Conclusion

The codebase is **well-structured** with good separation of concerns. The recommended refactorings focus on:

1. **Reducing boilerplate** (attribute parsing, error collection)
2. **Improving consistency** (validation, error messages)
3. **Maintaining clarity** (avoid over-abstraction)

The phased implementation roadmap delivers incremental value with minimal disruption.

**Key Principle:** Favor pragmatic improvements over architectural purity. The goal is better maintainability, not maximum abstraction.
