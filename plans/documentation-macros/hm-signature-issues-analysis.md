# Hindley-Milner Signature Generation Issues: Root Cause Analysis & Solutions

**Date:** 2026-02-07  
**Status:** Root Cause Identified - Previous Analysis Invalidated  
**Version:** 2.1 (Updated with Existing Pattern Analysis)

## Executive Summary

The Hindley-Milner type signature generation system fails to generate proper signatures for methods inside `#[document_module]` blocks, producing literal `Self` and missing `forall` quantifiers.

**Previous Analysis Was INCORRECT**: The original hypothesis that "attribute macros on inner items expand before outer macros" was disproven through systematic testing.

**Actual Root Cause**: The `document_module` macro already uses a **two-pass architecture** and **visitor patterns**, but neither pass recursively traverses into nested module contents. Both [`extract_context`](../../fp-macros/src/document_module.rs:119) (Pass 1) and [`generate_docs`](../../fp-macros/src/document_module.rs:314) (Pass 2) only process top-level items.

**Current Output vs. Expected:**

| Context | Current | Expected |
|---------|---------|----------|
| `impl<A> CatList<A>` methods | `() -> Self` | `forall A. () -> CatList A` |
| | `&self -> bool` | `forall A. &(CatList A) -> bool` |

---

## Investigation Process

### Phase 1: Testing the Expansion Order Hypothesis

The original analysis claimed that `#[hm_signature]` expands before `#[document_module]`, preventing proper context-aware processing. This was tested by:

1. **Converting `hm_signature` to a no-op macro** that returns input unchanged
2. **Running test cases** to observe macro behavior
3. **Using `cargo expand`** to examine actual macro expansion
4. **Adding debug logging** to track execution flow

**Test Code:**
```rust
// Modified fp-macros/src/hm_signature.rs
pub fn hm_signature_impl(
    attr: proc_macro2::TokenStream,
    item_tokens: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    eprintln!("[hm_signature] NO-OP: Returning input unchanged");
    item_tokens  // Just pass through unchanged
}
```

**Test Case:**
```rust
#[document_module]
mod test_context {
    impl<A> CatList<A> {
        #[hm_signature]
        pub fn empty() -> Self { todo!() }
    }
}
```

### Phase 2: Analyzing Results

**Critical Finding from Debug Logs:**

```
[document_module] START document_module_impl
[document_module] INPUT: mod test_context
{
    use fp_macros::hm_signature; pub struct CatListBrand; pub struct
    CatList<A>(A); impl<A> CatList<A>
    {
        #[hm_signature] pub fn empty() -> Self { todo!() }
        #[hm_signature] pub fn is_empty(&self) -> bool { true }
    }
}
[document_module] Total items parsed: 1
[document_module] Item 0: "Mod"
[document_module] Starting generate_docs
```

**Key Observations:**

1. ✅ **`#[hm_signature]` attributes ARE present** in `document_module`'s input
2. ✅ **`document_module` runs FIRST** (outer macro expands before inner)
3. ❌ **Only 1 item parsed**: The module itself, not its contents
4. ❌ **Item type is `Mod`**, not `Impl` - so `generate_docs` never processes it

**Cargo Expand Output:**

```rust
mod test_context {
    use fp_macros::hm_signature;
    pub struct CatListBrand;
    pub struct CatList<A>(A);
    impl<A> CatList<A> {
        pub fn empty() -> Self {  // No doc comment generated!
            ::core::panicking::panic("not yet implemented")
        }
        pub fn is_empty(&self) -> bool {
            true
        }
    }
}
```

**Conclusion:** The `#[hm_signature]` attributes were visible to `document_module`, but it failed to process them because it doesn't descend into module contents.

---

## Root Cause Analysis

### Existing Architecture Analysis

**Key Discovery:** The codebase already implements sophisticated patterns that were overlooked in the initial analysis.

#### 1. Two-Pass Architecture (Already Exists)

**Location:** [`fp-macros/src/document_module.rs:103-112`](../../fp-macros/src/document_module.rs:103)

```rust
// Pass 1: Context Extraction
if let Err(e) = extract_context(&items, &mut config) {
    return e.to_compile_error();
}

// Pass 2: Documentation Generation
if let Err(e) = generate_docs(&mut items, &config) {
    return e.to_compile_error();
}
```

**What They Do:**
- **Pass 1 ([`extract_context`](../../fp-macros/src/document_module.rs:119))**: Collects type projections from `impl_kind!` macros and trait impls, builds configuration context
- **Pass 2 ([`generate_docs`](../../fp-macros/src/document_module.rs:314))**: Processes `#[hm_signature]` and `#[doc_type_params]` attributes to generate documentation

**The Bug:** Both passes only iterate over `items: &[Item]` without recursing into nested modules!

#### 2. Visitor Pattern (Already Extensively Used)

The codebase already uses `syn::visit_mut::VisitMut` in multiple places:

**Location:** [`fp-macros/src/document_module.rs`](../../fp-macros/src/document_module.rs:16)

| Visitor | Purpose | Location |
|---------|---------|----------|
| [`SelfSubstitutor`](../../fp-macros/src/document_module.rs:573) | Replaces `Self` with concrete types in signatures | Line 573 |
| [`SubstitutionVisitor`](../../fp-macros/src/document_module.rs:831) | Generic type parameter substitution | Line 831 |
| [`NormalizationVisitor`](../../fp-macros/src/document_module.rs:881) | Normalizes types for comparison | Line 881 |
| [`SelfAssocVisitor`](../../fp-macros/src/document_module.rs:781) | Detects `Self::` references (uses `Visit`, not `VisitMut`) | Line 781 |

**Example:**
```rust
impl<'a> VisitMut for SelfSubstitutor<'a> {
    fn visit_type_mut(&mut self, i: &mut Type) {
        // Transform Self types
        // ...
        visit_mut::visit_type_mut(self, i); // Continue recursion
    }
}
```

#### 3. Custom TypeVisitor (Unrelated)

**Location:** [`fp-macros/src/function_utils.rs:93`](../../fp-macros/src/function_utils.rs:93)

This is a **custom trait** for converting Rust types to Hindley-Milner types, NOT related to AST traversal:

```rust
pub trait TypeVisitor {
    type Output;
    fn visit(&mut self, ty: &Type) -> Self::Output;
    // Methods for different type variants
}
```

**Not relevant** to the module traversal issue.

### The Actual Bug: Missing Module Recursion

**Both passes fail to handle `Item::Mod`:**

```rust
fn extract_context(items: &[Item], config: &mut Config) -> Result<()> {
    for item in items {
        match item {
            Item::Macro(m) if m.mac.path.is_ident("impl_kind") => { /* ... */ }
            Item::Impl(item_impl) => { /* ... */ }
            _ => {}  // ← Item::Mod is ignored!
        }
    }
}

fn generate_docs(items: &mut [Item], config: &Config) -> Result<()> {
    for item in items {
        if let Item::Impl(item_impl) = item {  // ← Never matches nested modules
            // Process impl blocks
        }
    }
}
```

**Impact:**
- Impl blocks inside nested modules are never seen
- `impl_kind!` macros inside nested modules don't populate config
- All documentation generation is skipped for nested content

---

## Proposed Solutions

### Approach 1: Leverage Existing Visitor Infrastructure ⭐ **RECOMMENDED**

**Description:** Create module-traversing visitors that integrate with the existing two-pass architecture using the already-proven `VisitMut` pattern.

**Implementation:**

#### Step 1: Create Context Extraction Visitor

```rust
struct ContextExtractorVisitor<'a> {
    config: &'a mut Config,
    errors: Vec<syn::Error>,
}

impl<'a> syn::visit_mut::VisitMut for ContextExtractorVisitor<'a> {
    fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
        if let Some((_, ref mut items)) = module.content {
            // Extract context from this module's items
            if let Err(e) = extract_context(items, self.config) {
                self.errors.push(e);
            }
            
            // Recursively process nested modules
            syn::visit_mut::visit_item_mod_mut(self, module);
        }
    }
}
```

#### Step 2: Create Documentation Generator Visitor

```rust
struct DocGeneratorVisitor<'a> {
    config: &'a Config,
    errors: Vec<syn::Error>,
}

impl<'a> syn::visit_mut::VisitMut for DocGeneratorVisitor<'a> {
    fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
        if let Some((_, ref mut items)) = module.content {
            // Generate docs for this module's items
            if let Err(e) = generate_docs(items, self.config) {
                self.errors.push(e);
            }
            
            // Recursively process nested modules
            syn::visit_mut::visit_item_mod_mut(self, module);
        }
    }
}
```

#### Step 3: Update Main Function

```rust
pub fn document_module_impl(...) -> TokenStream {
    // ... existing parsing logic ...
    
    let mut config = Config::default();
    
    // Pass 1: Context Extraction (now recursive!)
    let mut extractor = ContextExtractorVisitor {
        config: &mut config,
        errors: Vec::new(),
    };
    for item in &mut items {
        extractor.visit_item_mut(item);
    }
    if !extractor.errors.is_empty() {
        return combine_errors(extractor.errors).to_compile_error();
    }
    
    // Pass 2: Documentation Generation (now recursive!)
    let mut generator = DocGeneratorVisitor {
        config: &config,
        errors: Vec::new(),
    };
    for item in &mut items {
        generator.visit_item_mut(item);
    }
    if !generator.errors.is_empty() {
        return combine_errors(generator.errors).to_compile_error();
    }
    
    quote!(#(#items)*)
}
```

**Advantages:**
- ✅ Consistent with existing codebase patterns
- ✅ Leverages battle-tested `syn::visit_mut` infrastructure
- ✅ Fixes both passes simultaneously
- ✅ Handles arbitrarily nested modules
- ✅ Easy to extend for future features
- ✅ Minimal changes to existing `extract_context` and `generate_docs` logic
- ✅ Clear separation of concerns

**Disadvantages:**
- ⚠️ Adds ~60-80 lines of code (two visitor structs)
- ⚠️ Slightly more complex than simple recursion
- ⚠️ Requires understanding visitor pattern

**Trade-offs:**
- **Complexity:** Medium (visitor pattern, but already used extensively)
- **Performance:** Excellent (optimized by `syn`)
- **Maintainability:** High (idiomatic pattern, consistent with codebase)
- **Risk:** Low (proven pattern already used in 4+ places)
- **Extensibility:** Excellent (easy to add more visitor methods)

---

### Approach 2: Simple Recursive Calls

**Description:** Add direct recursive calls to both `extract_context` and `generate_docs` to handle `Item::Mod`.

**Implementation:**

```rust
fn extract_context(items: &[Item], config: &mut Config) -> Result<()> {
    let mut errors = Vec::new();
    
    for item in items {
        match item {
            Item::Macro(m) if m.mac.path.is_ident("impl_kind") => {
                // existing logic
            }
            Item::Impl(item_impl) => {
                // existing logic
            }
            Item::Mod(item_mod) => {
                // NEW: Recursively process module contents
                if let Some((_, ref items)) = item_mod.content {
                    if let Err(e) = extract_context(items, config) {
                        errors.push(e);
                    }
                }
            }
            _ => {}
        }
    }
    
    // Error handling
}

fn generate_docs(items: &mut [Item], config: &Config) -> Result<()> {
    let mut errors = Vec::new();
    
    for item in items {
        match item {
            Item::Impl(item_impl) => {
                // existing logic
            }
            Item::Mod(item_mod) => {
                // NEW: Recursively process module contents
                if let Some((_, ref mut mod_items)) = item_mod.content {
                    if let Err(e) = generate_docs(mod_items, config) {
                        errors.push(e);
                    }
                }
            }
            _ => {}
        }
    }
    
    // Error handling
}
```

**Advantages:**
- ✅ Minimal code change (~20-30 lines total)
- ✅ Simple to understand
- ✅ Direct fix to the problem
- ✅ No new abstractions needed

**Disadvantages:**
- ❌ Inconsistent with existing visitor-based patterns
- ❌ Adds pattern matching complexity to existing functions
- ❌ Less extensible (need to modify both functions for any new pass)
- ❌ Doesn't leverage existing infrastructure

**Trade-offs:**
- **Complexity:** Low (simple recursion)
- **Performance:** Excellent (minimal overhead)
- **Maintainability:** Medium (deviates from established patterns)
- **Risk:** Very Low (straightforward change)
- **Extensibility:** Low (requires modifying multiple functions)

---

### Approach 3: Flatten Module Structure Before Processing

**Description:** Transform nested modules into a flat list of items before the two-pass processing.

**Implementation:**

```rust
fn flatten_items(items: Vec<Item>) -> Vec<Item> {
    let mut flattened = Vec::new();
    
    for item in items {
        if let Item::Mod(mut item_mod) = item {
            if let Some((_, mod_items)) = item_mod.content.take() {
                // Recursively flatten nested module contents
                flattened.extend(flatten_items(mod_items));
            }
            // Add the now-empty module (or skip it)
        } else {
            flattened.push(item);
        }
    }
    
    flattened
}

pub fn document_module_impl(...) -> TokenStream {
    // Parse items
    let items = /* ... */;
    
    // NEW: Flatten before processing
    let mut items = flatten_items(items);
    
    // Existing two-pass logic works unchanged
    extract_context(&items, &mut config);
    generate_docs(&mut items, &config);
    
    // ...
}
```

**Advantages:**
- ✅ No changes needed to existing `extract_context` or `generate_docs`
- ✅ Clear preprocessing step
- ✅ Easy to debug (can inspect flattened structure)

**Disadvantages:**
- ❌ Destroys module structure (output doesn't match input)
- ❌ Loses module boundaries in generated code
- ❌ Can't generate module-level documentation
- ❌ Allocates new vectors (memory overhead)
- ❌ Would need to reconstruct structure for output

**Trade-offs:**
- **Complexity:** Medium (need to handle structure preservation)
- **Performance:** Moderate cost (extra allocation)
- **Maintainability:** Medium (preprocessing step)
- **Risk:** Medium-High (output structure mismatch)
- **Extensibility:** Poor (fundamentally incompatible with module-scoped features)

---

### Approach 4: Require Inner Attribute Usage

**Description:** Document that `#[document_module]` only works correctly as an inner attribute for modules containing impl blocks.

**Implementation:**

Add compiler warning:

```rust
pub fn document_module_impl(...) -> TokenStream {
    if let Ok(item_mod) = syn::parse2::<syn::ItemMod>(item.clone()) {
        if item_mod.content.is_some() {
            eprintln!(
                "warning: #[document_module] as outer attribute may not process nested contents. \
                Use #![document_module] as inner attribute instead."
            );
        }
    }
    // ...
}
```

Update documentation:
```rust
// Instead of:
#[document_module]
mod foo {
    impl<A> Bar<A> { ... }
}

// Use:
mod foo {
    #![document_module]
    impl<A> Bar<A> { ... }
}
```

**Advantages:**
- ✅ Zero code changes to macro logic
- ✅ Already works for inner attribute pattern
- ✅ No implementation risk

**Disadvantages:**
- ❌ User-facing breaking change
- ❌ Requires updating all usage sites
- ❌ Less ergonomic
- ❌ Doesn't actually fix the bug
- ❌ Still fails for nested modules even with inner attribute

**Trade-offs:**
- **Complexity:** None (documentation only)
- **Performance:** No impact
- **Maintainability:** High (clearer usage pattern)
- **Risk:** High (breaking change, doesn't fully solve problem)
- **Extensibility:** None

---

## Solution Comparison Matrix

| Criterion | Approach 1<br/>(Visitor) | Approach 2<br/>(Simple Recursion) | Approach 3<br/>(Flatten) | Approach 4<br/>(Doc Only) |
|-----------|-------------------------|----------------------------------|------------------------|--------------------------|
| **Consistency with Codebase** | Excellent | Poor | Fair | N/A |
| **Implementation Effort** | Medium | Low | Medium | None |
| **Code Complexity** | Medium | Low | Medium | None |
| **Maintainability** | High | Medium | Medium | High |
| **Fixes Both Passes** | Yes | Yes | Yes | No |
| **Preserves Structure** | Yes | Yes | No | N/A |
| **Performance** | Excellent | Excellent | Good | N/A |
| **Extensibility** | Excellent | Low | Poor | None |
| **Risk Level** | Low | Very Low | Medium | High |
| **Pattern Consistency** | ✅ Uses existing VisitMut | ❌ Adds ad-hoc recursion | ⚠️ New pattern | N/A |

---

## Recommendation: **Approach 1 (Leverage Existing Visitor Infrastructure)**

**Rationale:**

1. **Architectural Consistency:** Aligns with existing visitor-based patterns (`SelfSubstitutor`, `SubstitutionVisitor`, `NormalizationVisitor`, `SelfAssocVisitor`)
2. **Proven Infrastructure:** `syn::visit_mut` is battle-tested and already used extensively
3. **Both Passes Fixed:** Handles context extraction AND documentation generation
4. **Future-Proof:** Easy to extend for module-level features or additional passes
5. **Maintainable:** Clear, idiomatic pattern that other Rust macro developers will recognize
6. **Low Risk:** Leverages existing, proven infrastructure

**Why Not Approach 2 (Simple Recursion)?**

While Approach 2 requires less code (~30 lines vs ~70 lines), it:
- Breaks the established pattern of using visitors for AST traversal
- Makes the codebase inconsistent (some traversal uses visitors, some uses manual recursion)
- Is harder to extend (requires modifying multiple functions for new features)
- Doesn't leverage the robust error handling and traversal logic built into `syn::visit_mut`

**Implementation Plan:**

### Phase 1: Implement Visitors (Week 1)

1. **Create `ContextExtractorVisitor`**:
   - Implement `VisitMut` trait
   - Call `extract_context` for each module
   - Collect errors

2. **Create `DocGeneratorVisitor`**:
   - Implement `VisitMut` trait
   - Call `generate_docs` for each module
   - Collect errors

3. **Update `document_module_impl`**:
   - Replace direct calls with visitor pattern
   - Handle collected errors

### Phase 2: Testing (Week 1-2)

1. **Unit Tests:**
   - Single-level modules (verify no regression)
   - Nested modules (new behavior)
   - Deeply nested modules (3+ levels)
   - Mixed content (modules + impls at same level)
   - Empty modules
   - Modules without impls

2. **Integration Tests:**
   - Run existing test suite
   - Test with real codebase examples
   - Verify `impl_kind!` context extraction in nested modules
   - Verify `#[hm_signature]` generation in nested modules

3. **Edge Cases:**
   - Module re-exports
   - Conditional compilation (`#[cfg]`)
   - Macro-generated modules

### Phase 3: Documentation (Week 2)

1. Update [`module-level-documentation.md`](./module-level-documentation.md)
2. Add examples showing nested module support
3. Document visitor pattern usage
4. Add inline documentation for new visitors

---

## Success Criteria

After implementation, verify:

- ✅ `impl<A> CatList<A>::empty()` generates `forall A. () -> CatList A`
- ✅ `impl<A> CatList<A>::is_empty(&self)` generates `forall A. &(CatList A) -> bool`
- ✅ Nested modules processed correctly (1-5 levels deep)
- ✅ `impl_kind!` macros in nested modules populate context
- ✅ All existing tests pass
- ✅ No compilation time regression (< 2%)
- ✅ No breaking changes for users
- ✅ Visitor pattern consistent with existing code

---

## Appendix A: Existing Visitor Pattern Examples

### Example 1: SelfSubstitutor

**Location:** [`fp-macros/src/document_module.rs:573`](../../fp-macros/src/document_module.rs:573)

```rust
impl<'a> VisitMut for SelfSubstitutor<'a> {
    fn visit_type_mut(&mut self, i: &mut Type) {
        match i {
            Type::Path(type_path) => {
                // Replace Self with concrete type
                if let Some(first) = type_path.path.segments.first()
                    && first.ident == "Self"
                {
                    *i = self.self_ty.clone();
                    return;
                }
            }
            _ => {}
        }
        visit_mut::visit_type_mut(self, i);
    }
}
```

**Usage Pattern:**
```rust
let mut substitutor = SelfSubstitutor { /* ... */ };
substitutor.visit_signature_mut(&mut synthetic_sig);
```

### Example 2: SubstitutionVisitor

**Location:** [`fp-macros/src/document_module.rs:831`](../../fp-macros/src/document_module.rs:831)

```rust
impl VisitMut for SubstitutionVisitor<'_> {
    fn visit_type_mut(&mut self, i: &mut Type) {
        if let Type::Path(type_path) = i {
            // Perform generic parameter substitution
            // ...
        }
        visit_mut::visit_type_mut(self, i);
    }
}
```

These examples demonstrate the established pattern that Approach 1 follows.

---

## Appendix B: Diagnostic Evidence

### Test Setup

```rust
// fp-macros/tests/repro_issues.rs
#[document_module]
mod test_context {
    pub struct CatListBrand;
    pub struct CatList<A>(A);

    impl<A> CatList<A> {
        #[hm_signature]
        pub fn empty() -> Self { todo!() }

        #[hm_signature]
        pub fn is_empty(&self) -> bool { true }
    }
}
```

### Debug Output (With No-Op `hm_signature`)

```
[hm_signature] NO-OP: Returning input unchanged
[document_module] START document_module_impl
[document_module] INPUT: mod test_context
{
    use fp_macros::hm_signature; pub struct CatListBrand; pub struct
    CatList<A>(A); impl<A> CatList<A>
    {
        #[hm_signature] pub fn empty() -> Self { todo!() }
        #[hm_signature] pub fn is_empty(&self) -> bool { true }
    }
}
[document_module] Total items parsed: 1
[document_module] Item 0: "Mod"
[document_module] Starting generate_docs
```

**Key Finding:** The `#[hm_signature]` attributes ARE present in the input to `document_module`, disproving the expansion order hypothesis.

### Cargo Expand Output

```rust
mod test_context {
    use fp_macros::hm_signature;
    pub struct CatListBrand;
    pub struct CatList<A>(A);
    impl<A> CatList<A> {
        pub fn empty() -> Self {  // ← NO DOC COMMENT
            ::core::panicking::panic("not yet implemented")
        }
        pub fn is_empty(&self) -> bool {  // ← NO DOC COMMENT
            true
        }
    }
}
```

**Confirmation:** No signatures were generated despite attributes being present.

---

## Conclusion

The original analysis incorrectly blamed macro expansion order. The actual issue is that the already-sophisticated two-pass architecture with visitor patterns doesn't handle nested modules.

**The fix leverages existing patterns:** Use the proven `VisitMut` infrastructure (already used in 4+ places) to add module traversal to both passes, maintaining architectural consistency and providing a robust, extensible solution.

---

**Document Version:** 2.1  
**Last Updated:** 2026-02-07  
**Author:** Root Cause Analysis via Systematic Testing + Codebase Pattern Analysis  
**Status:** Ready for Implementation
