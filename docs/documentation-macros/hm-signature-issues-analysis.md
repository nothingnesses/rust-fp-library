# Hindley-Milner Signature Generation Issues: Analysis & Solutions

**Date:** 2026-02-07
**Status:** Analysis Verified - Legitimacy Confirmed via Reproduction

## Executive Summary

The Hindley-Milner type signature generation system has two fundamental design flaws:

1. **Macro Expansion Order Issue**: Standalone `#[hm_signature]` attributes expand before `#[document_module]`, preventing proper context-aware processing
2. **Missing Type Context**: Generic parameters from impl blocks are invisible to signature generation, causing `Self` types and missing `forall` quantifiers

**Current Output vs. Expected:**

| Context | Current | Expected |
|---------|---------|----------|
| `impl<A> CatList<A>` methods | `() -> Self` | `forall A. () -> CatList A` |
| | `&self -> bool` | `forall A. &(CatList A) -> bool` |
| Trait impl methods | `(A -> B, Self A) -> Self B` | `(A -> B, CatList A) -> CatList B` |

---

## Root Cause Analysis

### Issue 1: Macro Expansion Order

**Problem:** Rust expands attribute macros on inner items before outer macros on containing modules.

**Evidence:**
```rust
#[fp_macros::document_module]
mod inner {
    impl<A> CatList<A> {
        #[hm_signature]  // ← Expands FIRST
        pub fn empty() -> Self { ... }
    }
}
```

**Execution Order:**
1. `#[hm_signature]` on `empty()` expands → generates `() -> Self` (no impl context)
2. `#[document_module]` expands → sees already-processed code (no `#[hm_signature]` attribute remains)

**Diagnostic Evidence:**
Analysis of the expansion order using debug logs in [`fp-macros/src/document_module.rs`](fp-macros/src/document_module.rs) confirmed that when `document_module` is used as an outer attribute, it receives a token stream where `hm_signature` has already been expanded and removed.

```
[document_module] START document_module_impl
[document_module] INPUT: mod test_context { ... }
[generate_signature] Generic names: {}        # ← No 'A' from impl
[generate_signature] self_type_name: None     # ← No CatList context
[generate_signature] concrete_types: {}       # ← Empty set
[generate_signature] return: Variable("Self") # ← Self not replaced
```

### Issue 2: Missing Impl Context in Standalone Macro

**Problem:** The standalone `hm_signature` macro (in `fp-macros/src/hm_signature.rs`) only sees the method signature, not the surrounding impl block.

**Code Location:**
```rust
// fp-macros/src/hm_signature.rs:12-51
pub fn hm_signature_impl(
    attr: proc_macro2::TokenStream,
    item_tokens: proc_macro2::TokenStream,  // ← Only the method
) -> proc_macro2::TokenStream {
    let mut item = match GenericItem::parse(item_tokens) { ... }
    // No access to impl<A> CatList<A> context
}
```

**Missing Information:**
- Impl block's generic parameters (`<A>`)
- Concrete type name (`CatList`)
- Relationship between `Self` and `CatList<A>`

### Issue 3: Brand Name Extraction Failure

**Problem:** `extract_concrete_type_name()` in `document_module.rs` doesn't properly extract "CatList" from "CatListBrand".

**Code Location:**
```rust
// fp-macros/src/document_module.rs:226-243
fn extract_concrete_type_name(
    ty: &syn::Type,
    config: &Config,
) -> Option<String> {
    // Only gets first segment: "CatListBrand"
    // Should strip "Brand" suffix → "CatList"
}
```

**Impact:** Even for trait impls processed by `document_module`, `Self` remains unreplaced because the concrete type name is never added to `config.concrete_types`.

---

## Proposed Solutions

### Approach 1: Impl-Context-Aware Standalone Macro ⭐ **RECOMMENDED**

**Description:** Enhance the standalone `#[hm_signature]` macro to detect and extract impl block context.

**Implementation:**

1. **Parse surrounding context** in `hm_signature_impl`:
```rust
pub fn hm_signature_impl(
    attr: proc_macro2::TokenStream,
    item_tokens: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    // Parse the item
    let item = match GenericItem::parse(item_tokens) { ... };
    
    // NEW: Check if we're inside an impl block by examining the token stream
    // Look for patterns like `impl<T> TypeName<T> { #[hm_signature] fn ... }`
    let impl_context = extract_impl_context_from_tokens(&item_tokens);
    
    let mut config = load_config();
    if let Some(ctx) = impl_context {
        config.self_type_name = ctx.type_name;
        config.concrete_types.insert(ctx.type_name.clone());
        // Merge impl generics into signature
    }
    
    let signature = generate_signature(sig, None, &config);
    // ...
}
```

2. **Add impl context extraction**:
```rust
struct ImplContext {
    type_name: String,        // "CatList"
    generics: syn::Generics,  // <A>
}

fn extract_impl_context_from_tokens(
    tokens: &proc_macro2::TokenStream
) -> Option<ImplContext> {
    // Search backwards/upwards in token stream for impl pattern
    // This is possible but requires careful parsing
}
```

**Advantages:**
- ✅ Works with existing code structure
- ✅ No changes needed to source files
- ✅ Fixes both standalone and `document_module` cases
- ✅ Single source of truth for signature generation

**Disadvantages:**
- ⚠️ Complex token stream parsing
- ⚠️ May be fragile to macro expansion order changes
- ⚠️ Requires careful handling of nested impls

**Trade-offs:**
- **Complexity:** Medium-High (complex parsing logic)
- **Maintainability:** Medium (relies on token stream structure)
- **Compatibility:** High (backward compatible)
- **Robustness:** Medium (depends on consistent macro expansion)

---

### Approach 2: Document Module Takes Over

**Description:** Remove standalone `#[hm_signature]` usage inside `#[document_module]` blocks. Let `document_module` handle all signature generation.

**Implementation:**

1. **Update all source files**:
```rust
// BEFORE
#[fp_macros::document_module]
mod inner {
    impl<A> CatList<A> {
        #[hm_signature]  // ← Remove this
        pub fn empty() -> Self { ... }
    }
}

// AFTER  
#[fp_macros::document_module]
mod inner {
    impl<A> CatList<A> {
        pub fn empty() -> Self { ... }  // ← No attribute
    }
}
```

2. **Auto-detect methods needing signatures** in `document_module`:
```rust
// In generate_docs()
for impl_item in &mut item_impl.items {
    if let ImplItem::Fn(method) = impl_item {
        // NEW: Auto-generate for ALL public methods
        if method.vis == syn::Visibility::Public {
            // Generate HM signature automatically
        }
    }
}
```

**Advantages:**
- ✅ Clean separation: `document_module` owns all documentation
- ✅ Proper impl context always available
- ✅ No macro expansion order issues
- ✅ Simpler mental model

**Disadvantages:**
- ❌ Requires updating ~100+ `#[hm_signature]` annotations
- ❌ Breaking change for standalone usage
- ❌ Loses fine-grained control over which methods get signatures

**Trade-offs:**
- **Complexity:** Low (simpler overall design)
- **Maintainability:** High (single code path)
- **Compatibility:** Low (requires migration)
- **Robustness:** High (no expansion order issues)

---

### Approach 3: Delayed Expansion with Marker Macro

**Description:** Introduce a marker macro that delays signature generation until `document_module` runs.

**Implementation:**

1. **Create marker macro**:
```rust
#[proc_macro_attribute]
pub fn doc_hm_signature(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    // Don't generate signature here, just add marker attribute
    let item: proc_macro2::TokenStream = item.into();
    quote! {
        #[__doc_hm_signature_marker]
        #item
    }.into()
}
```

2. **Update usage**:
```rust
#[fp_macros::document_module]
mod inner {
    impl<A> CatList<A> {
        #[doc_hm_signature]  // ← New marker macro
        pub fn empty() -> Self { ... }
    }
}
```

3. **Process in document_module**:
```rust
if let Some(attr_pos) = find_attribute(&method.attrs, "__doc_hm_signature_marker") {
    method.attrs.remove(attr_pos);
    // Now generate signature with full context
}
```

**Advantages:**
- ✅ Explicit opt-in per method
- ✅ Works with `document_module` context
- ✅ Minimal source changes (rename macro)
- ✅ Maintains fine-grained control

**Disadvantages:**
- ⚠️ Two different macros (`hm_signature` vs `doc_hm_signature`)
- ⚠️ Requires migrating all `#[hm_signature]` inside `document_module`
- ⚠️ Still allows incorrect usage

**Trade-offs:**
- **Complexity:** Medium (two code paths)
- **Maintainability:** Medium (must maintain both)
- **Compatibility:** Medium (migration needed but clearer)
- **Robustness:** High (explicit control)

---

### Approach 4: Proc Macro Helper Attribute

**Description:** Use Rust's `#[proc_macro_attribute]` on the impl block itself to capture context.

**Implementation:**

1. **Create impl-level attribute**:
```rust
#[proc_macro_attribute]
pub fn documented_impl(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);
    
    // Extract context once
    let context = ImplContext::from_impl(&impl_block);
    
    // Process all methods with #[hm_signature]
    let processed = process_impl_methods(impl_block, context);
    
    quote!(#processed).into()
}
```

2. **Usage**:
```rust
#[fp_macros::document_module]
mod inner {
    #[documented_impl]  // ← New attribute
    impl<A> CatList<A> {
        #[hm_signature]
        pub fn empty() -> Self { ... }
    }
}
```

**Advantages:**
- ✅ Captures impl context at the right level
- ✅ Works with expansion order
- ✅ Can process multiple methods efficiently
- ✅ Clear scope of impact

**Disadvantages:**
- ⚠️ Adds another layer of macros
- ⚠️ Requires changes to every impl block
- ⚠️ Interaction with `document_module` unclear

**Trade-offs:**
- **Complexity:** Medium-High (new macro type)
- **Maintainability:** Medium (another integration point)
- **Compatibility:** Low (significant source changes)
- **Robustness:** High (explicit context capture)

---

## Solution Comparison Matrix

| Criterion | Approach 1<br/>(Impl-Aware) | Approach 2<br/>(Doc Module) | Approach 3<br/>(Marker) | Approach 4<br/>(Helper Attr) |
|-----------|---------------------------|---------------------------|----------------------|----------------------------|
| **Implementation Complexity** | High | Low | Medium | Medium-High |
| **Migration Effort** | None | High | Medium | High |
| **Backward Compatibility** | Full | Breaking | Semi | Breaking |
| **Robustness** | Medium | High | High | High |
| **Maintainability** | Medium | High | Medium | Medium |
| **Fine-grained Control** | Yes | No | Yes | Yes |
| **Works Standalone** | Yes | No | Partial | Yes |
| **Clear Semantics** | Medium | High | High | High |

---

## Recommendation: Hybrid Approach (1 + 3)

**Primary Strategy:** Approach 1 (Impl-Context-Aware Standalone Macro)  
**Fallback/Enhancement:** Approach 3 (Marker Macro for Explicit Context)

### Rationale

1. **Backward Compatibility**: Approach 1 requires zero source changes
2. **Immediate Fix**: Solves the problem for existing code
3. **Future Flexibility**: Approach 3 provides explicit opt-in when needed
4. **Progressive Enhancement**: Can implement Approach 1 first, add Approach 3 later

### Implementation Plan

#### Phase 1: Fix Standalone Macro (Weeks 1-2)

1. **Enhance token stream parsing**:
   - Add `extract_impl_context()` function
   - Detect `impl<...> Type<...>` patterns
   - Extract generics and type name

2. **Update `hm_signature_impl()`**:
   - Call `extract_impl_context()` 
   - Populate `Config` with impl information
   - Merge impl generics into method signature

3. **Fix brand name extraction**:
   - Update `extract_concrete_type_name()` to strip "Brand" suffix
   - Add result to `config.concrete_types`

4. **Add comprehensive tests**:
   - Test impl context extraction
   - Test generic parameter merging
   - Test Self → ConcreteName replacement

#### Phase 2: Add Marker Macro (Week 3)

1. **Create `doc_hm_signature` marker**:
   - Lightweight pass-through with marker attribute
   - Document when to use vs standalone

2. **Update `document_module`**:
   - Detect marker attribute
   - Process with full context

3. **Documentation**:
   - When to use `#[hm_signature]` (standalone, simple cases)
   - When to use `#[doc_hm_signature]` (complex contexts, nested impls)

#### Phase 3: Testing & Validation (Week 4)

1. **Verify all signatures**:
   - Run `cargo doc` and inspect output
   - Compare against expected signatures
   - Add regression tests

2. **Performance testing**:
   - Measure compilation time impact
   - Optimize if needed

### Success Criteria

- ✅ `impl<A> CatList<A>::empty()` generates `forall A. () -> CatList A`
- ✅ `impl<A> CatList<A>::is_empty(&self)` generates `forall A. &(CatList A) -> bool`  
- ✅ Trait impl methods generate `CatList` instead of `Self`
- ✅ No source file changes required
- ✅ All existing tests pass
- ✅ Compilation time impact < 5%

---

## Alternative Recommendations by Use Case

### If Minimizing Complexity is Priority
→ **Approach 2** (Document Module Takes Over)
- Simplest overall design
- Requires migration but cleaner long-term
- Best for greenfield projects

### If Backward Compatibility is Critical
→ **Approach 1** (Impl-Context-Aware)
- Zero breaking changes
- Works with existing code
- Best for established projects

### If Explicit Control is Needed
→ **Approach 3** (Marker Macro)
- Clear intent
- Works well with document_module
- Best for complex scenarios

### If Reusability is Priority
→ **Approach 4** (Helper Attribute)
- Composable design
- Can be used elsewhere
- Best for framework development

---

## Implementation Notes

### Key Files to Modify

1. **`fp-macros/src/hm_signature.rs`**:
   - Add impl context extraction
   - Update `hm_signature_impl()` function

2. **`fp-macros/src/document_module.rs`**:
   - Fix `extract_concrete_type_name()`
   - Improve brand → type mapping

3. **`fp-macros/src/function_utils.rs`**:
   - Update `type_to_hm()` to handle Self properly
   - Add concrete type tracking

### Testing Strategy

1. **Unit Tests**:
   - Token stream parsing
   - Context extraction
   - Type name mapping

2. **Integration Tests**:
   - Full signature generation
   - Document module integration
   - Brand trait impls

3. **Regression Tests**:
   - Existing signatures remain correct
   - No performance degradation

### Risks & Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| Token parsing breaks | High | Medium | Comprehensive test suite, fallback logic |
| Performance regression | Medium | Low | Benchmark before/after, optimize hot paths |
| Edge cases missed | Medium | Medium | Extensive testing, gradual rollout |
| Macro expansion changes | High | Low | Document assumptions, add guards |

---

## Conclusion

The recommended **Hybrid Approach (1 + 3)** provides:
- ✅ Immediate fix with zero migration
- ✅ Future flexibility for complex cases
- ✅ Robust long-term solution
- ✅ Clear upgrade path

This approach balances:
- **Short-term needs**: Fix existing code without changes
- **Long-term goals**: Clean, maintainable architecture
- **Risk management**: Progressive enhancement with fallbacks

### Verification Results (Added 2026-02-07)

The legitimacy of these issues was confirmed using a reproduction test case in [`fp-macros/tests/repro_issues.rs`](fp-macros/tests/repro_issues.rs). The following key observations were made:

1.  **Macro Execution Race**: Debug logs in [`fp-macros/src/document_module.rs`](fp-macros/src/document_module.rs) confirmed that `hm_signature` attributes on methods are expanded and removed before the `document_module` macro on the module even receives the token stream. This confirms **Issue 1**.
2.  **Context Loss**: The Early-expanding `hm_signature` reported `Generic names: {}` and `self_type_name: None` even when nested inside an `impl<A> CatList<A>` block. This confirms **Issue 2**.
3.  **Signature Output**: The generated signature for `pub fn empty() -> Self` was literally `() -> Self` instead of the expected `forall A. () -> CatList A`.

**Next Steps:**
1. Review and approve this analysis
2. Create implementation tickets for Phase 1
3. Set up test infrastructure
4. Begin implementation with comprehensive testing

---

## Appendix: Diagnostic Output

### Current Behavior (Broken)

```rust
impl<A> CatList<A> {
    #[hm_signature]
    pub const fn empty() -> Self
}
```

**Generates:** `() -> Self`

**Debug Log:**
```
[generate_signature] Generic names: {}
[generate_signature] self_type_name: None
[generate_signature] forall: []
[generate_signature] return: Variable("Self")
```

### Expected Behavior (Fixed)

**Should Generate:** `forall A. () -> CatList A`

**Expected Debug Log:**
```
[generate_signature] Generic names: {"A"}
[generate_signature] self_type_name: Some("CatList")
[generate_signature] forall: ["A"]
[generate_signature] return: Constructor("CatList", [Variable("A")])
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-07  
**Author:** AI Assistant (Diagnostic Analysis)  
**Status:** Ready for Review
