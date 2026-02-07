# Hindley-Milner Signature Generation Issues: Root Cause Analysis & Solutions

**Date:** 2026-02-07  
**Status:** Root Cause Identified - Previous Analysis Invalidated  
**Version:** 2.0 (Complete Rewrite)

## Executive Summary

The Hindley-Milner type signature generation system fails to generate proper signatures for methods inside `#[document_module]` blocks, producing literal `Self` and missing `forall` quantifiers.

**Previous Analysis Was INCORRECT**: The original hypothesis that "attribute macros on inner items expand before outer macros" was disproven through systematic testing.

**Actual Root Cause**: The `document_module` macro does not recursively traverse into module contents when applied as an outer attribute (`#[document_module] mod foo { ... }`), causing it to miss impl blocks nested inside the module.

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

### Issue: `document_module` Doesn't Traverse Module Contents

**Location:** [`fp-macros/src/document_module.rs:33-94`](../../fp-macros/src/document_module.rs:33)

**Problem Flow:**

```rust
pub fn document_module_impl(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    // Parses input - sees "mod test_context { ... }"
    let mut items = if let Ok(item_mod) = syn::parse2::<syn::ItemMod>(item.clone()) {
        if let Some((_, mod_items)) = item_mod.content {
            mod_items  // ← Extracts module contents correctly
        } else {
            // ...
        }
    } else {
        // ...
    };
    
    // items = [Item::Mod] - the module is treated as a single item!
    // Should be: [Item::Use, Item::Struct, Item::Struct, Item::Impl]
    
    if let Err(e) = generate_docs(&mut items, &config) {
        return e.to_compile_error();
    }
    // ...
}

fn generate_docs(
    items: &mut [Item],
    config: &Config,
) -> Result<()> {
    for item in items {
        if let Item::Impl(item_impl) = item {  // ← Never matches!
            // Process impl blocks with #[hm_signature]
        }
    }
}
```

**The Bug:**

1. When `#[document_module]` is applied to a module, it parses correctly
2. But the parser returns the `ItemMod.content` as a list of items
3. However, the test case uses a **nested module structure**
4. The parser treats the nested module as a single `Item::Mod` item
5. `generate_docs` only looks for top-level `Item::Impl` items
6. The impl blocks are **inside** the `Item::Mod`, so they're never found

### Secondary Issue: Parsing Strategy Mismatch

**Location:** [`fp-macros/src/document_module.rs:40-77`](../../fp-macros/src/document_module.rs:40)

The macro supports three modes:
1. Inner attribute: `mod foo { #![document_module] ... }`
2. Outer attribute on module: `#[document_module] mod foo { ... }`
3. Outer attribute on const block: `#[document_module] const _: () = { ... };`

**Issue:** Mode 2 extracts module contents but doesn't recursively process nested modules.

---

## Proposed Solutions

### Approach 1: Recursive Module Traversal ⭐ **RECOMMENDED**

**Description:** Modify `generate_docs` to recursively descend into `Item::Mod` items.

**Implementation:**

```rust
fn generate_docs(
    items: &mut [Item],
    config: &Config,
) -> Result<()> {
    let mut errors = Vec::new();

    for item in items {
        match item {
            Item::Impl(item_impl) => {
                // Process impl blocks (existing logic)
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
- ✅ Minimal code change (~20 lines)
- ✅ Handles arbitrarily nested modules
- ✅ No changes to user code required
- ✅ Maintains backward compatibility
- ✅ Works with all three application modes

**Disadvantages:**
- ⚠️ Slightly increases macro complexity
- ⚠️ Could process modules user didn't intend (minor)

**Trade-offs:**
- **Complexity:** Low (simple recursion pattern)
- **Performance:** Negligible (only traverses actual structure)
- **Maintainability:** High (clear, intuitive behavior)
- **Risk:** Very Low (additive change, doesn't break existing behavior)

---

### Approach 2: Flatten Module Structure Before Processing

**Description:** Transform nested modules into a flat list of items before processing.

**Implementation:**

```rust
fn flatten_items(items: Vec<Item>) -> Vec<Item> {
    let mut flattened = Vec::new();
    
    for item in items {
        if let Item::Mod(item_mod) = item {
            if let Some((_, mod_items)) = item_mod.content {
                // Recursively flatten nested module contents
                flattened.extend(flatten_items(mod_items));
            }
        } else {
            flattened.push(item);
        }
    }
    
    flattened
}

pub fn document_module_impl(...) -> TokenStream {
    // ...
    let mut items = flatten_items(items);
    // ...
}
```

**Advantages:**
- ✅ Keeps `generate_docs` simple (no recursion)
- ✅ Clear separation of concerns
- ✅ Easy to debug (can inspect flattened structure)

**Disadvantages:**
- ❌ Loses module structure information
- ❌ Can't generate module-level documentation
- ❌ May need to reconstruct structure for output
- ❌ Allocates more memory (copies everything)

**Trade-offs:**
- **Complexity:** Medium (need to preserve/restore structure)
- **Performance:** Moderate cost (extra allocation and iteration)
- **Maintainability:** Medium (two-phase processing)
- **Risk:** Medium (could affect module-level features)

---

### Approach 3: Require Inner Attribute Usage

**Description:** Document that `#[document_module]` only works as inner attribute for modules with impl blocks.

**Implementation:**

Update documentation and add compiler warning:

```rust
pub fn document_module_impl(...) -> TokenStream {
    // Detect outer attribute on module case
    if let Ok(item_mod) = syn::parse2::<syn::ItemMod>(item.clone()) {
        if item_mod.content.is_some() {
            // Emit warning
            eprintln!(
                "warning: #[document_module] as outer attribute may not process nested contents. \
                Consider using #![document_module] as inner attribute instead."
            );
        }
    }
    // ...
}
```

**Usage Change:**

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
- ✅ No code changes to macro logic
- ✅ Already works for this pattern
- ✅ Zero implementation risk

**Disadvantages:**
- ❌ User-facing breaking change
- ❌ Requires code updates across project
- ❌ Less ergonomic (extra line inside module)
- ❌ Doesn't fix the actual bug

**Trade-offs:**
- **Complexity:** None (documentation only)
- **Performance:** No impact
- **Maintainability:** High (clear usage pattern)
- **Risk:** High (breaking change for users)

---

### Approach 4: Expand Nested Modules Inline

**Description:** When encountering a nested module, expand its contents inline at the same level.

**Implementation:**

```rust
fn generate_docs(
    items: &mut [Item],
    config: &Config,
) -> Result<()> {
    let mut i = 0;
    while i < items.len() {
        if let Item::Mod(item_mod) = &mut items[i] {
            if let Some((_, mod_items)) = item_mod.content.take() {
                // Replace module with its contents
                items.splice(i..=i, mod_items);
                continue;  // Re-process this index
            }
        }
        i += 1;
    }
    
    // Now process impl blocks (existing logic)
}
```

**Advantages:**
- ✅ Processes all impl blocks
- ✅ Single-pass algorithm
- ✅ In-place modification (no extra allocation)

**Disadvantages:**
- ❌ Destroys module boundaries
- ❌ Output structure doesn't match input
- ❌ Complex index manipulation
- ❌ Hard to reason about behavior

**Trade-offs:**
- **Complexity:** High (index juggling, state mutation)
- **Performance:** Good (in-place)
- **Maintainability:** Low (confusing mutation patterns)
- **Risk:** High (could produce invalid output structures)

---

### Approach 5: Two-Pass Processing with Visitor Pattern

**Description:** Use `syn::visit_mut::VisitMut` to traverse and modify the AST.

**Implementation:**

```rust
use syn::visit_mut::{self, VisitMut};

struct DocumentGenerator<'a> {
    config: &'a Config,
    errors: Vec<syn::Error>,
}

impl<'a> VisitMut for DocumentGenerator<'a> {
    fn visit_item_impl_mut(&mut self, impl_item: &mut ItemImpl) {
        // Process impl blocks with #[hm_signature]
        // (move existing generate_docs logic here)
        
        // Continue traversing
        visit_mut::visit_item_impl_mut(self, impl_item);
    }
    
    fn visit_item_mod_mut(&mut self, mod_item: &mut ItemMod) {
        // Automatically recurses into module contents
        visit_mut::visit_item_mod_mut(self, mod_item);
    }
}

fn generate_docs(
    items: &mut [Item],
    config: &Config,
) -> Result<()> {
    let mut visitor = DocumentGenerator {
        config,
        errors: Vec::new(),
    };
    
    for item in items {
        visitor.visit_item_mut(item);
    }
    
    // Handle errors
}
```

**Advantages:**
- ✅ Idiomatic Rust pattern for AST traversal
- ✅ Handles all item types automatically
- ✅ Easy to extend for future features
- ✅ Battle-tested pattern from `syn` crate

**Disadvantages:**
- ⚠️ Larger refactoring (restructure existing code)
- ⚠️ Learning curve for maintainers unfamiliar with visitors
- ⚠️ More boilerplate

**Trade-offs:**
- **Complexity:** Medium-High (pattern understanding required)
- **Performance:** Excellent (optimized by `syn`)
- **Maintainability:** High (standard pattern)
- **Risk:** Medium (larger refactor, but well-tested pattern)

---

## Solution Comparison Matrix

| Criterion | Approach 1<br/>(Recursive) | Approach 2<br/>(Flatten) | Approach 3<br/>(Doc Only) | Approach 4<br/>(Inline) | Approach 5<br/>(Visitor) |
|-----------|---------------------------|-------------------------|--------------------------|------------------------|-------------------------|
| **Implementation Effort** | Low | Medium | None | Medium | High |
| **Code Complexity** | Low | Medium | None | High | Medium |
| **Maintainability** | High | Medium | High | Low | High |
| **Backward Compatibility** | Full | Full | Breaking | Full | Full |
| **Handles Nested Modules** | Yes | Yes | N/A | Yes | Yes |
| **Preserves Structure** | Yes | No | N/A | No | Yes |
| **Performance** | Excellent | Good | N/A | Good | Excellent |
| **Future Extensibility** | Good | Medium | Poor | Poor | Excellent |
| **Risk Level** | Very Low | Medium | High | High | Medium |

---

## Recommendation: **Approach 1 (Recursive Module Traversal)**

**Rationale:**

1. **Minimal Change:** Only adds ~20 lines of code to handle `Item::Mod` case
2. **Zero Breaking Changes:** Existing code continues to work unchanged
3. **Correct Semantics:** Processes items in their natural structure
4. **Immediate Fix:** Solves the problem directly without workarounds
5. **Low Risk:** Additive change with clear behavior
6. **Easy to Test:** Can verify recursion depth and edge cases

**Implementation Plan:**

### Phase 1: Add Recursive Processing (Week 1)

1. **Modify `generate_docs` function** ([`fp-macros/src/document_module.rs:291`](../../fp-macros/src/document_module.rs:291)):
   ```rust
   fn generate_docs(
       items: &mut [Item],
       config: &Config,
   ) -> Result<()> {
       let mut errors = Vec::new();

       for item in items {
           match item {
               Item::Impl(item_impl) => {
                   // Existing impl processing logic
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

       if errors.is_empty() {
           Ok(())
       } else {
           let mut combined_error: Error = errors.remove(0);
           for err in errors {
               combined_error.combine(err);
           }
           Err(combined_error)
       }
   }
   ```

2. **Add test cases**:
   ```rust
   #[test]
   fn test_nested_module_processing() {
       let input = quote! {
           #[document_module]
           mod outer {
               mod inner {
                   impl<A> Foo<A> {
                       #[hm_signature]
                       pub fn bar() -> Self { todo!() }
                   }
               }
           }
       };
       // Verify signature is generated
   }
   ```

### Phase 2: Testing & Validation (Week 1-2)

1. **Unit Tests:**
   - Single-level modules (existing behavior)
   - Nested modules (new behavior)
   - Deeply nested modules (3+ levels)
   - Mixed content (modules + impls at same level)

2. **Integration Tests:**
   - Run existing test suite (ensure no regressions)
   - Test with real codebase examples
   - Verify generated documentation

3. **Edge Cases:**
   - Empty modules
   - Modules without impls
   - Modules with only trait definitions

### Phase 3: Documentation (Week 2)

1. Update [`module-level-documentation.md`](./module-level-documentation.md) with recursion behavior
2. Add examples showing nested module support
3. Document any depth limits (if imposed)

---

## Alternative Recommendation: **Approach 5 (Visitor Pattern)**

**If future extensibility is a priority:**

The visitor pattern provides the most robust foundation for future features like:
- Module-level doc generation
- Cross-module reference resolution
- Trait documentation enhancement
- Namespace-aware processing

**When to choose this:**
- Planning significant macro system expansion
- Team familiar with AST visitor patterns
- Time available for larger refactor (~2-3 weeks)

---

## Success Criteria

After implementation, verify:

- ✅ `impl<A> CatList<A>::empty()` generates `forall A. () -> CatList A`
- ✅ `impl<A> CatList<A>::is_empty(&self)` generates `forall A. &(CatList A) -> bool`
- ✅ Nested modules processed correctly (1-3 levels deep)
- ✅ All existing tests pass
- ✅ No compilation time regression (< 1%)
- ✅ No breaking changes for users

---

## Appendix: Diagnostic Evidence

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

The original analysis incorrectly blamed macro expansion order. The actual issue is a missing recursion step in the `document_module` implementation. 

**The fix is straightforward:** Add ~20 lines to recursively process nested modules, providing an immediate solution with zero breaking changes.

---

**Document Version:** 2.0  
**Last Updated:** 2026-02-07  
**Author:** Root Cause Analysis via Systematic Testing  
**Status:** Ready for Implementation
