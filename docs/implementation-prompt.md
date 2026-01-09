# Implementation Prompt for Zero-Cost Refactoring

You are implementing the Zero-Cost Abstractions Refactoring Plan for a Rust functional programming library. Your task is to systematically refactor the codebase from a curried, dynamic-dispatch architecture to an uncurried, monomorphized architecture while preserving the HKT (Higher-Kinded Types) infrastructure.

## Reference Documents

- **Refactoring Plan**: `docs/zero-cost-refactoring-plan.md`
- **Progress Checklist**: `docs/refactoring-checklist.md`
- **Baseline Commit**: `5355cde26271bcbb37f930a92cf3430604621749`

## Implementation Protocol

### 1. Progress Tracking

Before making changes and periodically during implementation, check your progress against the baseline:

```bash
git diff 5355cde26271bcbb37f930a92cf3430604621749
```

Use this to:
- Verify changes align with the plan
- Ensure no unintended modifications have occurred
- Track which files have been modified

After completing each step, update the checklist in `docs/refactoring-checklist.md` by marking the appropriate item as complete (`[x]`).

### 2. Per-Step Implementation Process

For **each step** in the refactoring plan, follow this process:

#### 2.1. Analysis Phase

Before implementing, analyze the current step:

1. **Read the relevant source files** to understand the current implementation
2. **Identify dependencies** - what other code depends on the code you're changing?
3. **Consider edge cases** - are there type bounds, lifetimes, or trait interactions that could cause issues?

#### 2.2. Solution Formulation

**Formulate at least two alternative solutions** when possible:

- **Solution A**: The approach described in the plan
- **Solution B**: An alternative approach you've identified

For each solution, document:
- **Advantages**: Performance, ergonomics, maintainability
- **Disadvantages**: Complexity, breaking changes, edge cases
- **Trade-offs**: What does this solution optimize for vs. sacrifice?

#### 2.3. Review Gate

**STOP and present your findings to the user before implementing.** Include:

1. **Current state summary**: What the code currently does
2. **Proposed changes**: Your recommended solution with rationale
3. **Alternative solutions**: Other approaches considered with pros/cons
4. **Potential issues**: Any risks, breaking changes, or concerns
5. **Questions**: Anything you need clarified before proceeding

Example format:

```markdown
## Step X.Y: [Step Name]

### Current State
[Description of current implementation]

### Proposed Solution (Recommended)
[Detailed description of the recommended approach]

**Advantages:**
- ...

**Disadvantages:**
- ...

### Alternative Solution
[Description of alternative approach]

**Advantages:**
- ...

**Disadvantages:**
- ...

### Potential Issues
- [Issue 1]
- [Issue 2]

### Questions for Review
- [Question 1]
- [Question 2]

**Do you approve proceeding with the recommended solution?**
```

Wait for user approval before implementing.

#### 2.4. Implementation Phase

After approval:

1. **Implement the approved solution**
2. **Write well-documented code**:
   - Include doc comments (`///`) for all public items
   - Add inline comments for non-obvious logic
   - Follow Rust idioms and the existing code style
3. **Preserve existing tests** where possible, update where necessary
4. **Add new tests** for changed functionality

#### 2.5. Verification Phase

After implementing:

1. **Run the test suite**: `cargo test`
2. **Check for compilation errors**: `cargo check`
3. **Run clippy**: `cargo clippy`
4. **Verify the diff**: `git diff 5355cde26271bcbb37f930a92cf3430604621749`
5. **Update the checklist**: Mark the step as complete

If issues are found:
- Analyze the failure
- Propose fixes
- Get approval before applying fixes

### 3. Code Standards

#### Documentation

All public items must have doc comments following this pattern:

```rust
/// Brief one-line description.
///
/// Longer description if needed, explaining the purpose and behavior.
///
/// # Type Signature
///
/// `forall a b. TypeClass f => (a -> b) -> f a -> f b`
///
/// # Parameters
///
/// * `param1`: Description of the parameter
/// * `param2`: Description of the parameter
///
/// # Returns
///
/// Description of the return value.
///
/// # Examples
///
/// ```
/// use fp_library::...;
///
/// // Example code
/// ```
pub fn example_function() { ... }
```

#### Code Style

- Follow `rustfmt` formatting (run `cargo fmt`)
- Prefer explicit type annotations in complex generic contexts
- Use meaningful parameter names (`fa` for "functor of A", `ff` for "functor of functions")
- Group related trait bounds with `where` clauses for readability

#### Error Messages

When encountering type errors, provide context:
- What types were expected vs. actual
- Which bounds are missing
- Suggestions for fixing

### 4. Handling Challenges

#### If the plan seems incorrect or incomplete

1. **Document the specific issue** clearly
2. **Explain why you believe it's problematic**
3. **Propose corrections or additions**
4. **Get approval before deviating from the plan**

#### If you discover a better approach

1. **Complete the current step as planned** (unless the plan is fundamentally flawed)
2. **Document the improvement** as a future enhancement
3. **Discuss with the user** whether to incorporate it now or later

#### If tests fail unexpectedly

1. **Analyze the failure** - is it a bug in your changes or an expected breaking change?
2. **Document the failure** with full error output
3. **Propose a fix** or explain why the test expectation should change
4. **Get approval** before modifying tests

### 5. Phase-by-Phase Guidance

#### Phase 1: Function Wrapper Traits
- Focus on understanding the current trait hierarchy
- Minimal code changes expected
- Primary goal is planning and documentation

#### Phase 2: Uncurry Type Class Traits
- This is the core refactoring work
- Changes will cascade through the codebase
- Implement traits before their implementations
- Use `cargo check` frequently to catch type errors early

#### Phase 3: Update Type Implementations
- Each type should be updated independently
- Verify type class laws still hold after changes
- Pay attention to `Clone` bounds and lifetime requirements

#### Phase 4: Update Helper Functions
- These are standalone and can be updated independently
- Ensure backward compatibility where sensible

#### Phase 5: Endofunction/Endomorphism
- These types are more complex due to their use of `dyn Fn`
- Test thoroughly with different function brands

#### Phase 6: Brand Infrastructure
- Minimal changes expected
- Focus on verification

#### Phase 7: Documentation
- Update all doc comments to reflect new API
- Ensure all examples compile and run correctly
- Update README with new usage patterns

### 6. Communication Guidelines

- **Be explicit** about what you're doing and why
- **Show your work** - include relevant code snippets
- **Ask clarifying questions** before making assumptions
- **Report progress** at each step completion
- **Flag blockers** immediately when encountered

### 7. Success Criteria

The refactoring is complete when:

1. All items in `docs/refactoring-checklist.md` are marked complete
2. `cargo test` passes with no failures
3. `cargo clippy` shows no warnings
4. All doc tests pass (`cargo test --doc`)
5. The API matches the signatures in the refactoring plan
6. Documentation is updated for all changed items

---

## Getting Started

Begin with Phase 1, Step 1.1. Read the current implementation of the `Function` trait in `fp-library/src/classes/function.rs`, then present your analysis and any proposed alternatives before proceeding.

Remember: **Always present your findings and get approval before implementing each step.**
