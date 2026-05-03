# Agent Prompt: Adversarial Review of `effects.rs`

Copy the section below into the agent. It assumes the agent has read
access to both repositories and can use Read, Grep, Glob, and the LSP
tool.

---

## Task

You are performing an **adversarial technical review** of a
work-in-progress Rust algebraic effects implementation. The user wants
you to be harsh and specific. They are not looking for a balanced
report; they are looking for a list of every meaningful problem you
can find, with the most important problems being **fundamental flaws**
that cannot be fixed without redesigning the system.

Do not soften your critique. Do not pad with positives unless a
positive directly bears on whether a negative is a real problem (e.g.,
"this looks bad but the design doc explicitly addresses it on line N
by..."). Do not invent fixes unless asked; the goal is diagnosis, not
treatment.

## Files

- **Implementation under review:**
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects.rs`
  (the entire `effects` module tree under that path; read the
  surrounding files as needed to understand the implementation,
  including any submodules, handlers, interpreters, and supporting
  types).
- **Design plan that the implementation is following:**
  `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md`
  Read this to understand stated intent, declared scope, and which
  features are explicitly out-of-scope versus accidentally missing.
  A "fundamental flaw" is one the plan itself does not foresee or
  whose stated mitigation does not actually work.
- **Rubric / theoretical reference:**
  `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/algebraic_effects.md`
  This document defines what algebraic effect systems are, what
  capabilities a complete system should support, and how various
  implementations differ. Use it as the standard the implementation
  should be measured against. Cite specific section numbers when
  invoking it (e.g., "fails Section 6.1 #4: handler clauses do not
  receive a callable continuation").

## Surrounding context

- The host project is a Rust functional-programming library that
  encodes higher-kinded types via a Brand pattern and uses heavy
  type-level machinery (see `CLAUDE.md` and `AGENTS.md` at the repo
  root for conventions). The effects module is one of the project's
  more ambitious subsystems.
- The implementation is **explicitly WIP**. Things may be unimplemented
  or stubbed. Distinguish between "not yet implemented but the design
  accommodates it" (note as a future risk) and "the design itself
  precludes implementing it" (call out as fundamental).
- Rust's lack of native delimited continuations, its borrow checker,
  and its restricted higher-rank polymorphism make algebraic effects
  hard to encode well. Pay particular attention to whether the design
  has run into these constraints and how it has responded.

## Dimensions to evaluate

Evaluate the implementation along at least the following axes. For
each, decide whether the implementation succeeds, partially succeeds,
or fails, and explain in concrete terms with file and line citations.

1. **Operational semantics.** Does the implementation actually deliver
   algebraic-effect semantics, or is it a lookalike? Specifically:
   - Are operations decoupled from their interpretations?
   - Do handlers receive (something equivalent to) the continuation of
     the operation?
   - Is the continuation single-shot, multi-shot, or escape-only? Is
     this consistent with the design doc's claims?
   - Can the same program be reinterpreted by swapping handlers
     without code changes?
2. **Effect row encoding.** How are effect sets represented at the
   type level? Is the encoding open (extensible)? Does subsumption
   work? Are there restrictions on row size, ordering, or repetition
   that the design doc does not acknowledge?
3. **Handler composition.** Can handlers be composed in arbitrary
   orders? Is there an n^2 instance burden hidden in the
   implementation? Are there combinations of handlers the type system
   accepts but that produce wrong semantics?
4. **Higher-order / scoped operations.** How does the implementation
   treat operations like `local`, `catch`, `mask`, `bracket` that take
   computations as arguments? See Section 6.2 #8 of the rubric. Is
   this addressed at all? If not addressed, is the omission a stated
   non-goal or an oversight?
5. **Resource safety.** What happens to acquired resources when a
   handler discards or duplicates a continuation? What happens on
   panic? Are there leaks, double-frees, or use-after-free risks
   reachable from safe code?
6. **Soundness.** Are there `unsafe` blocks? Are their invariants
   documented and actually upheld by the surrounding safe API? Can a
   user trigger UB without writing `unsafe` themselves? Pay special
   attention to anything involving raw pointers, `mem::transmute`,
   `Pin`, or `Send`/`Sync` claims.
7. **Type inference and ergonomics.** What does a typical caller's
   signature look like? How verbose are effect rows in practice? Are
   there inference cliffs (cases where the user must annotate
   extensively to get the program to compile)? How does this compare
   to what an MTL-style or tagless-final encoding in Rust would
   require?
8. **Performance.** What is the per-operation cost in this encoding?
   Is each operation an allocation? Are there hidden quadratic costs
   in the bind/sequence operation (cf. left-bind blowup in naive free
   monads)? Is there any fusion or specialization?
9. **Continuation flexibility.** Can the implementation express the
   canonical examples from Section 8.2 of the rubric (multi-shot
   nondeterminism)? If not, why not? Is this fixable in this design or
   does it require a different encoding?
10. **Interaction with `IO` / async.** Can effectful programs perform
    real IO inside handlers? How does the system interoperate with
    Rust async? Are futures, runtimes, and cancellation accounted for?
11. **Debuggability.** What does a panic inside a handled effect look
    like? Are stack traces intelligible?
12. **Internal consistency.** Where does the implementation diverge
    from the plan? Where does the plan describe behaviour the
    implementation does not yet provide? Where does the implementation
    rely on properties the plan does not justify?
13. **Comparison to known designs.** Which existing system, of those
    summarized in Section 7 of the rubric, does this implementation
    most resemble? What problems of that family does it inherit? Does
    the design avoid any of them?

## What counts as a "fundamental flaw"

Mark a problem as **fundamental** only if at least one of the
following is true:

- Fixing it would require changing the core encoding (e.g., switching
  from a free-monad style to evidence-passing style, or vice versa).
- It violates a property the design doc explicitly claims to
  guarantee.
- It precludes a capability listed in Section 6 of the rubric (Core
  capabilities, Higher-order capabilities, or Pragmatic capabilities)
  that the plan claims to support.
- It is a soundness hole reachable from safe code.

Mark a problem as **major** if it is a serious bug or design problem
that can be fixed by local refactoring without changing the core
encoding.

Mark a problem as **minor** if it is a real defect but routine to fix
(naming, documentation, missing trait impls, dead code, small API
inconsistencies).

If you are uncertain whether something is fundamental or major, err
toward fundamental and explain the uncertainty.

## Output format

Write the report to
`/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/review_effects_rs.md`.
If a file already exists at that path, overwrite it; prior reviews
are tracked through git history.

Use this Markdown structure:

```
# Adversarial Review of effects.rs

## Summary
<3-5 sentences. Your overall verdict, the count of fundamental /
major / minor findings, and the single most important issue.>

## Fundamental Flaws
For each:
### F1. <Short title>
- **Where:** <file:line citations, multiple if needed>
- **What:** <concrete description of the problem>
- **Why fundamental:** <why local fixes do not suffice; cite which
  rubric section or plan claim it violates>
- **Worked example:** <a minimal program or scenario that exhibits
  the problem>

## Major Issues
Same structure, less depth.

## Minor Issues
Bullet list with file:line and a one-sentence description each.

## Plan vs. Implementation Drift
Section listing places where the plan and the implementation
disagree, with the plan citation and the implementation citation.

## Comparison to Existing Designs
Identify the closest analogue (Polysemy, freer-simple, fused-effects,
EvEff, MpEff, Effekt, heftia, etc.), name the inherited weaknesses,
and identify any novel problems specific to this design.
```

## Rules of engagement

- Cite file paths as relative links: `[effects.rs:42](path#L42)`.
- Quote short snippets (~5 lines) when they make a point sharper.
- Use ASCII only: no em-dashes, en-dashes, unicode arrows or math, no
  emoji or unicode symbols. Use `->`, `<-`, `>=`, `<=`, `!=`. Use
  commas or semicolons in place of dashes.
- Do not propose redesigns unless flagging that the issue is
  "redesign-only" (which is what makes it fundamental in the first
  place).
- Do not run tests, build, or modify files. This is a read-only
  review.
- Use the LSP tool aggressively for type questions; the user's
  CLAUDE.md notes it is configured and useful for this codebase's
  HKT/Brand machinery.
- If the implementation references types or traits defined elsewhere
  in `fp-library`, follow them. The effects module does not stand
  alone.
- If the plan is silent on a topic where the implementation makes a
  consequential choice, treat that as itself a finding (the design
  has unstated commitments).
- Length budget: aim for a thorough report. Do not artificially
  shorten. Do not pad either.

## Starting steps

1. Read `algebraic_effects.md` end to end if you have not already; it
   defines the standards you will measure against.
2. Read `plan.md` end to end, noting claimed scope, claimed
   guarantees, and explicit non-goals.
3. List all files in and below
   `fp-library/src/types/effects/` (and confirm `effects.rs` is the
   right entry point; the path may be a module file with submodules).
4. Read the implementation in dependency order: core effect types
   first, then handlers, then interpreters, then user-facing surface.
5. Form a preliminary thesis about the encoding family (free-monad,
   evidence-passing, tagless, capability, etc.) and verify it against
   the code.
6. Then, with the rubric and plan in mind, work through the dimensions
   above.
7. Write the report.
