# effekt

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/effects/effekt/`

## Purpose

Stage 1 research document: classify `effekt` against the five effect-row
encodings catalogued in [../port-plan.md](../port-plan.md) section 4.1.
Identify whether this codebase (which is a language, not a library)
represents a genuinely novel encoding worth deeper investigation in Stage 2. The relevant question is whether effekt's capability-based approach
could inform a Rust encoding rather than whether effekt's implementation
is itself portable.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

Effekt uses a _capability-passing_ substrate with lexically scoped, implicit second-class effect parameters. Effects are declared as `interface` types (e.g., `interface Exception[E]`; exceptions/library/common/exception.effekt:8), and handlers bind them via `try`...`with` syntax (exceptions/library/common/exception.effekt:57-58). At the type-system level, a function's effect requirements are tracked as a set of `InterfaceType` objects collected in an `Effects` container (effekt/shared/src/main/scala/effekt/symbols/types.scala:86-114). Each effect in scope is represented as a `BlockParam` with an `isImplicit` flag set to `True` (effekt/shared/src/main/scala/effekt/symbols/symbols.scala:121), meaning the compiler automatically threads capabilities through function calls based on lexical scope, never as first-class values. The runtime representation uses a `CapabilityScope` hierarchy (effekt/shared/src/main/scala/effekt/typer/CapabilityScope.scala) that maps each effect type to its handler, with `BindSome` for explicit handler bindings and `BindAll` for effect-polymorphic handlers that auto-instantiate fresh capabilities (CapabilityScope.scala:53-99). At the intermediate-code level, capabilities manifest as regular parameters in the machine representation (effekt/shared/src/main/scala/effekt/machine/Tree.scala:70), but the compiler ensures they are only visible through the implicit-parameter mechanism, not as first-class terms.

### Distinctive contribution relative to baseline

Effekt deviates from Free-monad-based approaches (PureScript `Run`, Haskell mtl, frunk-style coproducts) by refusing to make effect handlers first-class values. Instead, effects are _implicit second-class parameters_ inferred from lexical scope; a handler bound by `try`...`with` introduces a capability binding that is automatically passed to all callees that mention the effect in their signature. This eliminates the need for a heterogeneous list or row-type data structure entirely: the effect row exists only in the type signature (as `Effects`), never at runtime as a reified object. This is closer to Scala's implicit parameters or Haskell's `ReaderT`-with-scoped-binding than to Free monads, but with the crucial difference that Effekt enforces lexical scoping syntactically: you cannot pass a capability outside its handler's scope, and the compiler checks this statically.

### Classification against port-plan section 4.1

Effekt does _not_ cleanly fit any of the five options listed in port-plan section 4.1. It is a _genuinely novel_ encoding that combines ideas from Options 5 (trait-bound set / implicit dispatch) and a form of effect-polymorphic constraint solving unique to Effekt. The closest analogy is Option 5 (mtl-style, one-trait-per-effect), but with a critical departure: Effekt programs are _not_ first-class polymorphic functions parametrized over an implicit-parameter-like monadic constraint. Instead, programs are generic terms with effect annotations, and the compiler specializes them by discovering which capabilities are in scope. This is a form of _capability inference_ rather than constraint-based parametrization. If forced to choose: Effekt is Option 5-adjacent (effect-polymorphic, implicit dispatch) but fundamentally incompatible with the Free-family commitment because effects are never reified as first-class terms.

### Scoped-operations handling (`local`, `catch`, and similar)

Effekt handles scoped operations via the `try`...`with` construct, where `try` brackets a region and `with` introduces a handler and its associated capability binding (library/common/exception.effekt:57-58; examples/pos/file.effekt:3-6). The handler's methods receive an implicit `resume` parameter (a continuation) that allows them to return control to the caller (examples/pos/file.effekt:5; probabilistic.effekt:39-42). Higher-order scoped operations like `loop` { f } are expressed by having the handler (or a helper function) recursively re-enter the scope, passing the capability to the action function as a block parameter (library/common/control.effekt:10-16). The lexical scoping is enforced by the compiler: a capability binding from one handler is invisible outside its `try` block unless it is explicitly passed as a block parameter to an enclosing scope.

### Openness approach

Effekt achieves extensibility by allowing users to define new `interface` types as effect specifications (library/common/exception.effekt:8; library/common/io.effekt:8-24). New handlers are defined using `try`...`with` blocks that match an interface type, with no requirement for registration or row-type modification. Effect polymorphism is inferred: a function that mentions an effect in its signature (e.g., `def lookup[A](l: List[A], idx: Int): A / indexOutOfBounds` in examples/pos/probabilistic.effekt:7) automatically becomes effect-polymorphic; callers must be in a handler's scope to call it. The `CapabilityScope` machinery ensures that new effects can be introduced at any program point without modifying existing code (CapabilityScope.scala:53-99). This is fully open in the sense that new effects require no compiler support beyond the standard handler mechanism, but it is not open in the sense of row-type extension: you cannot express "all effects except these three"; the effect set is always inferred from what is lexically in scope.

### Rust portability assessment

Simulating Effekt's capability-passing in Rust would require encoding implicit second-class parameters as explicit function arguments (since Rust has no implicit parameters), effectively converting it to a variant of Option 4 (hybrid coproduct + macro sugar). A feasible approach: (1) use a macro to inject a parameter of type `&dyn Handler` into every function that uses effects, similar to shtsoft's effect-trait design or `karpal-effect`; (2) represent handlers as trait objects `dyn Effect` that encapsulate the set of active capabilities; (3) rely on Rust's lexical scope and lifetime checker to enforce that capabilities do not escape their handler's scope. The main blockers are (a) Rust's lack of implicit parameters requires explicit macro expansion, (b) trait objects introduce indirection and lose type information that Effekt's compiler uses for optimization, (c) the borrow checker may conflict with the lifetime of capability references in nested handler scopes. The capability-passing idea itself is portable, but the implicit-parameter mechanism is not; the result would be less elegant than Effekt's source syntax but broadly feasible if the macro overhead is acceptable.

### Relevance to port-plan

This finding does _not_ change the port-plan's direction materially. Effekt is incompatible with the Free-family commitment (Section 2.1 of port-plan.md): because Effekt programs are not first-class values, a faithful port would require abandoning the design principle that `Run` programs can be composed, inspected, and transformed as data structures. However, the capability-passing approach offers a complementary _alternative design_ if the Free-family commitment were relaxed: Effekt demonstrates that a purely implicit, lexically-scoped effect dispatch can be simpler and more efficient than Free-monad-based row-type encodings. For the immediate port-plan goal (choosing between Options 1-5 for a Free-compatible encoding), Effekt is not applicable. For future work on an alternative effect system (Stage 3 or beyond), Effekt's approach merits a deep-dive into implicit-parameter simulation in Rust.

### References

- `library/common/exception.effekt:8` : `interface Exception[E]` effect declaration.
- `library/common/exception.effekt:57-58` : handler syntax with `try`...`with`.
- `library/common/control.effekt:10-16` : higher-order scoped operations (`loop`).
- `library/common/io.effekt` : async effect interface (example of effect decl.).
- `examples/pos/file.effekt` : capability scoping and lexical binding in action.
- `examples/pos/probabilistic.effekt:7` : effect-polymorphic function signature.
- `examples/pos/probabilistic.effekt:39-42` : handler using `resume` continuation.
- `effekt/shared/src/main/scala/effekt/symbols/types.scala:86-114` : `Effects` container and effect-set representation.
- `effekt/shared/src/main/scala/effekt/symbols/symbols.scala:121` : `BlockParam` with `isImplicit` flag.
- `effekt/shared/src/main/scala/effekt/typer/CapabilityScope.scala` : capability scoping and lexical resolution (lines 11-99).
- `README.md:1-20` : language overview and research-level disclaimer.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1500 (excluding this template boilerplate)
