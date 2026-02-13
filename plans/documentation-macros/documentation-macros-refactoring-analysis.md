# Documentation Macros Refactoring Analysis

## Overview

This document analyzes the three documentation macro implementations:
- [`document_module.rs`](../fp-macros/src/documentation/document_module.rs)
- [`document_fields.rs`](../fp-macros/src/documentation/document_fields.rs)
- [`document_parameters.rs`](../fp-macros/src/documentation/document_parameters.rs)

All three follow a similar pattern: when applied to an outer scope, they process items within their scope and output documentation in place of other attributes within their scope.

## Repeated Patterns

### 1. Multi-Format Parsing with Fallbacks

All three macros need to handle multiple input formats and try parsing in a specific order.

**document_module.rs (lines 87-110):**
```rust
fn parse_document_module_input(item: TokenStream) -> Result<ParsedInput, syn::Error> {
    if let Some(parsed) = try_parse_module_wrapper(item.clone()) {
        return Ok(parsed);
    }
    if let Some(parsed) = try_parse_direct_items(item.clone()) {
        return Ok(parsed);
    }
    if let Ok(parsed) = try_parse_const_block(item) {
        return Ok(parsed);
    }
    Err(syn::Error::new(...))
}
```

**document_fields.rs (lines 49-82):**
```rust
pub fn document_fields_worker(...) -> Result<TokenStream> {
    if let Ok(item_enum) = syn::parse2::<ItemEnum>(item_tokens.clone()) {
        // Enum handling with validation
        return document_enum_fields(item_enum);
    }
    // Fall back to struct handling
    let mut item_struct = syn::parse2::<ItemStruct>(item_tokens)?;
    // ...
}
```

**document_parameters.rs (lines 181-214):**
```rust
pub fn document_parameters_worker(...) -> Result<TokenStream> {
    if let Ok(item_impl) = syn::parse2::<syn::ItemImpl>(item_tokens.clone()) {
        return process_impl_block(attr, item_impl);
    }
    // Otherwise, process as a function
    crate::support::syntax::generate_doc_comments(...)
}
```

### 2. Visitor Pattern for Nested Items

**document_module.rs (lines 186-240):** Has TWO nearly identical visitor implementations that differ only in config mutability:

```rust
// Visitor 1: Mutable config
struct ModuleVisitor<'a, F>
where
    F: Fn(&[Item], &mut Config) -> syn::Result<()>,
{
    operation: F,
    config: &'a mut Config,
    errors: &'a mut ErrorCollector,
}

impl<'a, F> VisitMut for ModuleVisitor<'a, F>
where
    F: Fn(&[Item], &mut Config) -> syn::Result<()>,
{
    fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
        if let Some((_, ref items)) = module.content {
            if let Err(e) = (self.operation)(items, self.config) {
                self.errors.push(e);
            }
            visit_mut::visit_item_mod_mut(self, module);
        }
    }
}

// Visitor 2: Immutable config (nearly identical!)
struct ModuleVisitorImmut<'a, F>
where
    F: Fn(&mut [Item], &Config) -> syn::Result<()>,
{
    operation: F,
    config: &'a Config,
    errors: &'a mut ErrorCollector,
}

impl<'a, F> VisitMut for ModuleVisitorImmut<'a, F>
where
    F: Fn(&mut [Item], &Config) -> syn::Result<()>,
{
    fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
        if let Some((_, ref mut items)) = module.content {
            if let Err(e) = (self.operation)(items, self.config) {
                self.errors.push(e);
            }
            visit_mut::visit_item_mod_mut(self, module);
        }
    }
}
```

**Duplication:** ~50 lines of nearly identical code that differs only in whether `items` is `&[Item]` vs `&mut [Item]` and whether `config` is `&mut Config` vs `&Config`.

### 3. Attribute Finding, Removal, and Parsing

**document_fields.rs (lines 26-33):** Uses the new unified helper:
```rust
let Some((_attr_idx, args)) =
    remove_and_parse_attribute::<FieldDocArgs>(&mut variant.attrs, DOCUMENT_FIELDS)?
else {
    return Ok(());
};
```

**document_parameters.rs (lines 32-38):** Uses older manual pattern:
```rust
let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_PARAMETERS) else {
    return Ok(());
};
let attr = method.attrs.remove(attr_pos);
// ... manual parsing
```

**Inconsistency:** `document_parameters.rs` could be updated to use `remove_and_parse_attribute`.

### 4. Validation Patterns

All three use validation helpers but with slightly different patterns:

**document_fields.rs:**
```rust
let documenter = FieldDocumenter::new(field_info, attr_span, "struct");
documenter.validate_and_generate(args, &mut item_struct.attrs)?;
```

**document_parameters.rs:**
```rust
validation::validate_entry_count(
    logical_params.len(),
    entries.len(),
    attr.span(),
    "method parameter",
)?;
```

**document_module.rs:** Validation is embedded in the resolution and generation phases.

### 5. Documentation Generation and Insertion

All three ultimately insert doc attributes:

**document_fields.rs:** Uses `FieldDocumenter` which internally calls helpers
**document_parameters.rs:** Uses `insert_doc_comments_batch`
**document_module.rs:** Delegates to `generate_documentation` function

## Shared Code Already Extracted

The codebase already has good shared utilities:

1. **[`support/attributes.rs`](../fp-macros/src/support/attributes.rs):**
   - `find_attribute`, `remove_attribute_tokens`, `remove_and_parse_attribute`

2. **[`support/validation.rs`](../fp-macros/src/support/validation.rs):**
   - `validate_entry_count`, `validate_named_entries`, `check_duplicate_entry`

3. **[`support/syntax.rs`](../fp-macros/src/support/syntax.rs):**
   - `generate_doc_comments`, `insert_doc_comment`, `insert_doc_comments_batch`

4. **[`core/error_handling.rs`](../fp-macros/src/core/error_handling.rs):**
   - `ErrorCollector` for accumulating errors

## Refactoring Approaches

### Approach 1: Generic Visitor Pattern (High Abstraction)

**Goal:** Unify the two visitor implementations in `document_module.rs` into a single generic visitor.

#### Implementation Strategy

Create a trait to abstract over operations:

```rust
// In support/visitors.rs
pub trait NestedOperation<Config> {
    fn apply(&mut self, items: &[Item], config: Config) -> syn::Result<()>;
}

// For mutable config operations
impl<F> NestedOperation<&mut Config> for F
where
    F: FnMut(&[Item], &mut Config) -> syn::Result<()>,
{
    fn apply(&mut self, items: &[Item], config: &mut Config) -> syn::Result<()> {
        self(items, config)
    }
}

// For immutable config operations
impl<F> NestedOperation<&Config> for F
where
    F: FnMut(&mut [Item], &Config) -> syn::Result<()>,
{
    fn apply(&mut self, items: &mut [Item], config: &Config) -> syn::Result<()> {
        self(items, config)
    }
}

// Single unified visitor
struct ModuleVisitor<'a, Op, Cfg> {
    operation: Op,
    config: Cfg,
    errors: &'a mut ErrorCollector,
}

impl<'a, Op, Cfg> VisitMut for ModuleVisitor<'a, Op, Cfg>
where
    Op: NestedOperation<Cfg>,
{
    fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
        if let Some((_, ref mut items)) = module.content {
            if let Err(e) = self.operation.apply(items, self.config) {
                self.errors.push(e);
            }
            visit_mut::visit_item_mod_mut(self, module);
        }
    }
}
```

**Trade-offs:**

✅ **Pros:**
- Eliminates ~50 lines of duplicated code
- Single source of truth for visitor pattern
- Type-safe with compile-time guarantees

❌ **Cons:**
- Higher cognitive complexity
- Trait with associated types may be confusing
- Harder to debug (more indirection)
- May hit limitations with mutability and lifetimes

**Difficulty:** Medium-High  
**Risk:** Medium (could break existing functionality if not careful)  
**Value:** Medium (only affects one file, but good cleanup)

### Approach 2: Multi-Format Parser Helper (Low Abstraction)

**Goal:** Extract the "try multiple parsers in order" pattern into a reusable helper.

#### Implementation Strategy

```rust
// In support/parsing.rs

/// Try multiple parsing strategies in order, returning the first success.
///
/// # Example
/// ```
/// let parsed = parse_one_of!(item_tokens => {
///     ItemEnum => handle_enum,
///     ItemStruct => handle_struct,
/// })?;
/// ```
pub fn try_parse_one_of<T>(
    tokens: TokenStream,
    parsers: Vec<Box<dyn Fn(TokenStream) -> Result<T, syn::Error>>>,
    error_msg: &str,
) -> Result<T, syn::Error> {
    for parser in parsers {
        if let Ok(result) = parser(tokens.clone()) {
            return Ok(result);
        }
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        error_msg,
    ))
}
```

Or using a macro for ergonomics:

```rust
/// Try parsing as multiple types in order.
macro_rules! try_parse {
    ($tokens:expr, $($ty:ty => $handler:expr),+ $(,)? else $error:expr) => {
        {
            $(
                if let Ok(parsed) = syn::parse2::<$ty>($tokens.clone()) {
                    return $handler(parsed);
                }
            )+
            Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                $error
            ))
        }
    };
}
```

Then in `document_fields.rs`:

```rust
pub fn document_fields_worker(attr: TokenStream, item_tokens: TokenStream) -> Result<TokenStream> {
    try_parse!(item_tokens,
        ItemEnum => |e| handle_enum(attr, e),
        ItemStruct => |s| handle_struct(attr, s),
        else "Expected struct or enum"
    )
}
```

**Trade-offs:**

✅ **Pros:**
- Simple to understand and use
- Low risk (doesn't change semantics)
- Reduces boilerplate in all three files
- Easy to add new cases

❌ **Cons:**
- Moderate code savings (~10-20 lines per file)
- Macro might obscure control flow
- Still need custom handlers for each case

**Difficulty:** Low  
**Risk:** Low  
**Value:** Low-Medium (incremental improvement)

### Approach 3: Processing Pipeline Abstraction (High Abstraction)

**Goal:** Create a unified pipeline for "find attribute → parse → validate → generate → insert".

#### Implementation Strategy

```rust
// In support/pipeline.rs

pub struct DocumentationPipeline<'a, Args, Context> {
    item: &'a mut dyn ItemWithAttrs,
    attr_name: &'static str,
    _phantom: PhantomData<(Args, Context)>,
}

impl<'a, Args, Context> DocumentationPipeline<'a, Args, Context>
where
    Args: Parse,
    Context: ExtractContext,
{
    pub fn new(item: &'a mut dyn ItemWithAttrs, attr_name: &'static str) -> Self {
        Self { item, attr_name, _phantom: PhantomData }
    }

    pub fn find_and_remove(&mut self) -> Result<Option<(usize, Args)>> {
        remove_and_parse_attribute::<Args>(self.item.attrs_mut(), self.attr_name)
    }

    pub fn extract_context(&self) -> Result<Context> {
        Context::extract(self.item)
    }

    pub fn validate(&self, args: &Args, context: &Context) -> Result<()> {
        context.validate(args)
    }

    pub fn generate_docs(&mut self, args: Args, context: Context) -> Result<()> {
        let docs = context.generate_documentation(args)?;
        self.insert_docs(docs)
    }

    fn insert_docs(&mut self, docs: Vec<(String, String)>) -> Result<()> {
        insert_doc_comments_batch(self.item.attrs_mut(), docs, 0);
        Ok(())
    }
}
```

**Trade-offs:**

✅ **Pros:**
- Very DRY - all three macros follow same pattern
- Easy to add new documentation macros
- Clear separation of concerns
- Enforces consistent behavior

❌ **Cons:**
- Very high abstraction cost
- Requires extensive trait infrastructure
- May be overengineered for 3 use cases
- Hard to customize for specific needs
- Significant refactoring effort

**Difficulty:** High  
**Risk:** High (major restructuring)  
**Value:** Medium (only valuable if adding many more doc macros)

### Approach 4: Incremental Improvements (Low Risk)

**Goal:** Make small, targeted improvements to reduce duplication without major restructuring.

#### Specific Changes

1. **Update `document_parameters.rs` to use `remove_and_parse_attribute`** (lines 32-38):
   ```rust
   // Before
   let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_PARAMETERS) else {
       return Ok(());
   };
   let attr = method.attrs.remove(attr_pos);
   
   // After
   let Some((attr_pos, args)) = 
       remove_and_parse_attribute::<ParamDocArgs>(&mut method.attrs, DOCUMENT_PARAMETERS)?
   else {
       return Ok(());
   };
   ```

2. **Extract visitor creation helpers in `document_module.rs`**:
   ```rust
   fn apply_operation_to_nested<F>(
       items: &mut [Item],
       mut operation: F,
       config: &mut Config,
   ) -> syn::Result<()>
   where
       F: FnMut(&[Item], &mut Config) -> syn::Result<()>,
   {
       let mut errors = ErrorCollector::new();
       
       for item in items {
           if let Item::Mod(module) = item {
               if let Some((_, ref items)) = module.content {
                   if let Err(e) = operation(items, config) {
                       errors.push(e);
                   }
                   // Recurse
                   apply_operation_to_nested(items, &mut operation, config)?;
               }
           }
       }
       
       errors.finish()
   }
   ```

3. **Extract common error message formatters**:
   ```rust
   // In support/validation.rs
   pub fn format_missing_doc_error(context: &str, name: &str) -> String {
       format!("Missing documentation for {context} `{name}`. All {context}s must be documented.")
   }
   ```

**Trade-offs:**

✅ **Pros:**
- Minimal risk
- Easy to review and understand
- Incremental value
- No breaking changes
- Can be done piece by piece

❌ **Cons:**
- Still leaves some duplication
- Doesn't address fundamental patterns
- Multiple small PRs instead of one comprehensive change

**Difficulty:** Low  
**Risk:** Very Low  
**Value:** Low-Medium (steady improvement)

### Approach 5: Macro-Based Code Generation (Alternative)

**Goal:** Use declarative macros to generate the boilerplate structure.

#### Implementation Strategy

```rust
// In support/macros.rs

macro_rules! define_documentation_worker {
    (
        $worker_name:ident,
        $attr_name:expr,
        parse: {
            $($variant:ident : $ty:ty => $handler:expr),+ $(,)?
        },
        error: $error_msg:expr $(,)?
    ) => {
        pub fn $worker_name(
            attr: TokenStream,
            item: TokenStream,
        ) -> crate::core::Result<TokenStream> {
            $(
                if let Ok(parsed) = syn::parse2::<$ty>(item.clone()) {
                    return $handler(attr, parsed);
                }
            )+
            
            Err(crate::core::Error::Parse(syn::Error::new(
                proc_macro2::Span::call_site(),
                $error_msg,
            )))
        }
    };
}

// Usage in document_fields.rs:
define_documentation_worker!(
    document_fields_worker,
    "document_fields",
    parse: {
        Enum: ItemEnum => handle_enum,
        Struct: ItemStruct => handle_struct,
    },
    error: "Expected struct or enum",
);
```

**Trade-offs:**

✅ **Pros:**
- Very DRY at source level
- Declarative and clear intent
- Forces consistent structure
- Easy to maintain once written

❌ **Cons:**
- Macros are harder to debug
- Poor IDE support (completion, navigation)
- Compile errors can be cryptic
- May be too rigid for complex cases
- Macro hygiene issues

**Difficulty:** Medium  
**Risk:** Medium  
**Value:** Medium (if you like macros)

## Recommendations

### Priority 1: Low-Hanging Fruit (Approach 4)

Start with incremental improvements that provide immediate value with minimal risk:

1. **Standardize on `remove_and_parse_attribute`** in `document_parameters.rs`
2. **Extract common error message formatters** to `validation.rs`
3. **Document the common patterns** in code comments

**Estimated effort:** 1-2 hours  
**Risk:** Very Low  
**Value:** Low-Medium

### Priority 2: Visitor Unification (Approach 1)

If you find yourself needing similar visitor patterns elsewhere, invest in the generic visitor:

1. **Create a generic visitor abstraction** in `support/visitors.rs`
2. **Refactor `document_module.rs`** to use the unified visitor
3. **Add comprehensive tests** to ensure behavior unchanged

**Estimated effort:** 4-6 hours  
**Risk:** Medium  
**Value:** Medium

### Priority 3: Multi-Format Parsing (Approach 2)

If adding more documentation macros or finding the pattern repeated:

1. **Create a `try_parse_one_of` helper** or macro
2. **Refactor all three workers** to use it
3. **Document the pattern** for future macros

**Estimated effort:** 2-3 hours  
**Risk:** Low  
**Value:** Low-Medium

### Not Recommended

- **Approach 3 (Pipeline):** Overengineered for current needs. Consider only if planning to add 5+ more documentation macros.
- **Approach 5 (Macro Generation):** Adds complexity without sufficient benefit. The code isn't repetitive enough to justify macro generation.

## Metrics

### Current Duplication

| Pattern | Lines Duplicated | Files Affected | Complexity |
|---------|------------------|----------------|------------|
| Visitor Pattern | ~50 | 1 | Medium |
| Multi-Format Parsing | ~15 per file | 3 | Low |
| Attribute Processing | ~10 per file | 2 | Low |
| Error Messages | ~5 per occurrence | 3 | Very Low |

### Potential Savings

| Approach | Lines Saved | Risk | Effort | Value |
|----------|-------------|------|--------|-------|
| Approach 1 (Visitor) | 40-50 | Medium | High | Medium |
| Approach 2 (Parsing) | 30-45 | Low | Low | Low-Med |
| Approach 3 (Pipeline) | 100+ | High | Very High | Medium |
| Approach 4 (Incremental) | 20-30 | Very Low | Low | Low-Med |
| Approach 5 (Macros) | 50-70 | Medium | Medium | Medium |

## Conclusion

The three documentation macros share structural patterns but have enough variation that heavy abstraction may not be worthwhile. The best strategy is:

1. **Start with Approach 4** (incremental improvements) to get quick wins
2. **Consider Approach 1** (visitor unification) if the visitor pattern appears elsewhere
3. **Consider Approach 2** (parsing helper) if adding more documentation macros
4. **Document the patterns** so future maintainers understand the design

The existing shared utilities ([`attributes.rs`](../fp-macros/src/support/attributes.rs), [`validation.rs`](../fp-macros/src/support/validation.rs), [`syntax.rs`](../fp-macros/src/support/syntax.rs)) are already doing a good job of factoring out common operations. The remaining duplication is mostly structural and may not benefit from further abstraction without significant complexity costs.
