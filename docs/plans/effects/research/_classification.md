# Stage 1 classification: aggregated findings

**Status:** complete
**Last updated:** 2026-04-24

## 1. What Stage 1 set out to answer

Stage 1 surveyed 13 effect-system codebases, one research document per
codebase, classifying each against the five open row encodings catalogued
in [../port-plan.md](../port-plan.md) section 4.1:

1. Type-level heterogeneous list / nested coproduct with Peano indices.
2. Typenum-indexed sum list (binary naturals, O(log n) depth).
3. Trait-object dispatch with `TypeId` tags.
4. Hybrid: coproduct + macro sugar.
5. Trait-bound set (mtl-style). Ruled out by the Free-family commitment
   in port-plan section 4.4 but retained as a benchmark.

Each Stage 1 file answered a narrow question: is this codebase a variant
of one of those options, or is its approach genuinely novel and worth a
Stage 2 deep dive? This synthesis aggregates those verdicts, groups the
codebases by encoding family, identifies cross-cutting findings the
per-codebase files could not see in isolation, and nominates the Stage 2
deep dives justified by the combined evidence.

## 2. Classification table

Columns: core substrate; classification against section 4.1; one-line
novelty verdict.

| Codebase          | Core substrate                                            | Section 4.1 mapping                                | Novelty                                 |
| ----------------- | --------------------------------------------------------- | -------------------------------------------------- | --------------------------------------- |
| freer-simple      | Free over open Union; FTCQueue tail                       | Option 1 (Peano)                                   | Variant.                                |
| polysemy          | Free (`Sem`) over Union; `ElemOf` int-backed Peano        | Option 1 (Peano)                                   | Variant; Tactical for higher-order.     |
| fused-effects     | mtl-style `Algebra sig m` over `:+:` coproduct; no Free   | Option 5 with Option 1 substrate; no first-class   | Variant (Option 5 family).              |
| EvEff             | Typed `Context` GADT of handlers; `Ctl` multi-prompt      | None of 1-5; evidence passing                      | Genuinely novel.                        |
| MpEff             | Evidence passing + GHC RTS multi-prompt (`prompt#`)       | None of 1-5; hybrid row-track + dynamic dispatch   | Genuinely novel; non-portable.          |
| heftia            | Free over Freer FTCQueue; data-effects coproduct          | Option 1 with dual-row scoped-effect layer         | Variant; scoped ops are first-class.    |
| in-other-words    | First-class `Union` coproduct; Derivs/Prims reformulate   | Option 1 (Peano `ElemOf`)                          | Variant.                                |
| Effekt (language) | Capability passing; implicit second-class parameters      | Option 5-adjacent; no first-class programs         | Genuinely novel.                        |
| Koka (language)   | Row-poly surface; compile-time linearised evidence vector | None of 1-5; new axis (compile-time indices)       | Genuinely novel.                        |
| corophage         | frunk Peano `Coproduct` + `Effects![...]` macro; fauxgen  | Option 4 (confirmed)                               | Variant.                                |
| effing-mad        | frunk Peano `Coproduct`; nightly `Coroutine` trait        | Option 1 (confirmed)                               | Variant.                                |
| reffect           | Custom Peano-like `UInt<U>` tags over right-nested tuple  | Option 1 in substance, despite plan's Option 2 tag | Variant; plan misclassifies it.         |
| fx-rs             | Struct-based `Has<T>` / `Put<T, U>` evidence traits       | None of 1-5; Option 5-adjacent                     | Novel; forecloses first-class programs. |

Two findings emerge only from the table. First, the Option 1 cluster has
six members (seven if you count fused-effects' coproduct substrate),
while Option 2 and Option 3 have zero real reference implementations.
Second, the novelty column divides along a different axis than row
encoding: the novel codebases address _dispatch_ and _scoping_, not
_row representation_.

## 3. Novel encodings

Four codebases fall outside options 1-5 in substantive ways. A fifth
(fx-rs) is novel but already ruled out by the Free-family commitment
and is covered briefly in section 4 as an Option 5 neighbour.

### 3.1 Evidence passing (EvEff, MpEff)

EvEff and MpEff both implement the Xie / Leijen evidence-passing
semantics. Effect rows are type-level lists (superficially like Option
1), but at runtime the row becomes a typed linked list of handler
_values_ rather than a discriminated union of effect operations. When a
program performs an effect, dispatch is a linear scan over that context
driven by a type-equality constraint (`In h e` with `HEqual`;
[eveff.md](eveff.md) and [mpeff.md](mpeff.md)), not a coproduct
projection. This gives flat, cache-friendly dispatch at the cost of
linear lookup per operation. It also sidesteps the row-ordering problem
that options 1, 2, and 4 share: membership is by type equality, not
position, so permutations of the same row are dispatched identically.

MpEff additionally unifies evidence passing with multi-prompt delimited
continuations via GHC's `prompt#` and `control0#` RTS primitives. This
is what makes `local`, `catch`, and `mask` fall out as native
control-flow features, without the Tactical or HFunctor machinery that
polysemy and fused-effects need. The same primitives are what make
MpEff non-portable to Rust: the port-plan has ruled out delimited
continuations in section 1.2.

EvEff's lookup mechanism, however, is separable from the continuation
substrate. A Rust port could simulate typed handler lookup via a
runtime linked list of boxed handlers with a compile-time type-equality
proof, without needing prompts. The `Ctl` monad stays Haskell-only; the
dispatch idea does not. This is the justification for a Stage 2 deep
dive.

### 3.2 Compile-time evidence indices (Koka)

Koka ([koka.md](koka.md)) has row-polymorphic effect types at the
source level, but the compiler's `OpenResolve` pass
(src/Core/OpenResolve.hs:110-204) erases the row structure by computing
a flat integer vector of effect indices at compile time. The runtime
carries no row, only the vector. Scoped handler nesting is supported
via mask levels on the indices. This is a different axis from options
1-5: rather than encoding the row in a type constructor (coproduct,
trait-bound, or TypeId map) that persists to runtime, Koka erases the
row entirely.

The indexing half of Koka's strategy is Rust-portable and could be
folded into Option 3 (TypeId dispatch) or Option 4 (macro sugar) to
replace runtime type inspection with stable compile-time indices. The
scoped-handler half requires multi-prompt continuations and is not
portable for the same reason MpEff is not. This motivates the same
Stage 2 deep dive as 3.1, specifically scoped to the index-generation
pass (separable from continuations).

### 3.3 Capability passing (Effekt)

Effekt ([effekt.md](effekt.md)) makes effects into implicit
second-class parameters threaded by the compiler based on lexical
scope. A `try`...`with` block introduces a capability binding; callees
that mention the effect in their signature receive it automatically.
The row exists only in type signatures, never at runtime.

This is Option 5-adjacent (effect set lives in a constraint, not a
data structure) but goes beyond mtl because the compiler, not the
user, threads the capability. Rust cannot replicate implicit
parameters; a simulation would require either a macro layer to inject
`&dyn Handler` arguments or a novel effect-row syntax. Programs are
not first-class values, which conflicts with the port-plan's section
4.4 commitment. Effekt is therefore a reference point, not a candidate.
No Stage 2 deep dive is recommended unless the Free-family commitment
is ever relaxed.

### 3.4 Heftia's dual row (architectural, not encoding)

Heftia ([heftia.md](heftia.md)) uses a standard Option 1 coproduct for
its row encoding; the novelty is architectural, not encoding-level. It
splits effects into two parallel rows (first-order algebraic effects
and higher-order scoped effects) and reifies scoped operations like
`Catch` and `Span` as first-class AST constructors, elaborated by a
separate `interposeWith` pass. This eliminates the Tactical boilerplate
polysemy needs for scoped effects and the HFunctor scaffolding
fused-effects needs.

Heftia is mentioned here rather than in section 4 because the
scoped-effect question is cross-cutting: all Free-family designs need
an answer to it, and heftia's is the cleanest reviewed. It motivates a
Stage 2 deep dive on scoped-operation ergonomics for the Rust port,
alongside polysemy's Tactical and in-other-words' Effly-wrapper pattern
as comparison points.

## 4. Variants of known options

**Option 1 family (six members).** freer-simple, polysemy, heftia,
in-other-words, effing-mad, and reffect all use a type-level list with
Peano-style membership indices. Each layers a distinctive contribution
on the same substrate: freer-simple adds an FTCQueue tail for stack
safety; polysemy adds Weaving, Tactical, and Scoped for higher-order
effects; heftia adds the dual-row separation of scoped effects;
in-other-words adds Derivs/Prims reformulation to dodge the quadratic
handler-instance problem; effing-mad trades the Free AST for the
unstable Rust `Coroutine` trait; reffect uses its own phantom-typed
`UInt<U>` wrapper over a right-nested tuple but is structurally
equivalent. None of these is a row-encoding novelty; Option 1 is the
dominant choice in the Free-family world.

**Option 2 family (zero members).** No codebase surveyed implements
Option 2 as the port-plan defines it. reffect is the plan's named
reference, but its tag type `UInt<U>` is a single-parameter phantom
wrapper, not a two-parameter binary natural like `typenum::UInt<U,
B0|B1>`. Type depth is O(n), identical to frunk's `There<T>`
([reffect.md](reffect.md)). This is the most consequential factual
finding of Stage 1: the Rust ecosystem has no known reference
implementation of Option 2 as the plan describes it. The "proportional"
compile-time and error-message improvements the plan attributes to
Option 2 are theoretical, not demonstrated.

**Option 3 family (zero members).** No codebase surveyed uses
`Box<dyn Any>` + `TypeId` dispatch as its primary substrate.
PureScript `Run` itself does (at runtime, `Run` is a
`{ type: String, value, map }` object), but none of the Rust, Haskell,
or research-language codebases in the survey use this approach for a
typed row. The honourable-mention crates (`anymap`, `typemap`,
`axum::Extension`) in the port-plan are dictionary-style, not
effect-row libraries. Option 3 is a feasible fallback but has no
production validation for the port's specific use case.

**Option 4 family (one member).** corophage
([corophage.md](corophage.md)) is the sole direct reference, confirmed.
Its `Effects![...]` macro preserves user order (no lexical sorting, no
deduplication), expands to a right-nested frunk coproduct with Peano
indices, and supports `...Tail` spread syntax. Single-shot `FnOnce`
handlers rule out multi-shot effects like `Choose`. Its lifetime
parameter `'a` on effects is a concrete validation of the port-plan's
`FreeExplicit` variant for non-`'static` payloads; a detail the plan's
summary did not call out.

**Option 5 family (two members).** fused-effects
([fused-effects.md](fused-effects.md)) is Option 5 with an Option 1
coproduct substrate underneath, but dispatches via the `Algebra sig m`
typeclass rather than wrapping the coproduct in Free. Programs are
generic functions, not first-class values. fx-rs ([fx-rs.md](fx-rs.md))
uses struct-based evidence traits `Has<T>` / `Put<T, U>` rather than
trait bounds, but also forecloses on first-class programs. Both are
ruled out by the port-plan section 4.4 commitment; they serve as
fallback designs if the Free-family approach becomes unmanageable.

## 5. Recommendations for Stage 2

Three deep dives are justified by Stage 1 evidence. Each is scoped to
a single question the Stage 1 skim could not resolve.

### 5.1 Evidence-passing dispatch in Rust (priority 1)

**Question:** Can Rust host a typed handler-vector dispatch (EvEff /
Koka indexing) that is ergonomic and competitive with nested coproduct
traversal, without requiring delimited continuations?

**Sources:** EvEff's `Context` GADT with `In h e` / `HEqual`
constraint; Koka's compile-time `OpenResolve` index-generation pass.
The two share a core idea (handlers as first-class values addressed by
type-level identity, not list position) and differ in when the
identity is resolved (runtime linear scan for EvEff, compile-time
indexing for Koka).

**Deliverable:** a minimal Rust prototype of a typed `Context` trait
and `In<H, E>` constraint, plus a rough benchmark against an Option 1
coproduct baseline. This either surfaces a sixth candidate for section
4.1 or explicitly rules it out.

### 5.2 Coroutine substrate versus Free AST (priority 2)

**Question:** Does the port actually need a Free monad, or can
coroutines alone preserve the first-class-program property section 4.4
requires?

**Sources:** corophage, effing-mad, and reffect all use coroutines
without a Free wrapper. corophage wraps the coroutine in a `Program`
type that carries handler-composition metadata; reffect uses
`Handle<Coro, H, Markers>` for similar effect. All three provide
inspectable, suspendable, resumable program values. fused-effects
represents the opposite pole: Free is explicitly rejected and
first-class programs are lost as a consequence.

**Deliverable:** a mapping of which Free-family properties
(multi-shot interpretation, `peel`/`send` composition, pure-data
interpreters like `runPure`, hoisting) survive under a coroutine-only
substrate. If all survive, Free becomes an implementation option, not
a requirement; the port plan's section 4.4 currently treats them as
coupled.

### 5.3 Scoped-operation ergonomics across encodings (priority 3)

**Question:** Which scoped-effect pattern minimises boilerplate for
the Rust port while staying compatible with the chosen row encoding?

**Sources:** heftia's dual-row first-class constructors;
in-other-words' `Effly` continuation wrapper with Derivs/Prims
reformulation; polysemy's Tactical; freer-simple's flat interposition.
Each pays a different cost to support `local` and `catch`. MpEff's
native multi-prompt control is the non-portable aspirational target.

**Deliverable:** a Rust sketch of how each pattern would express
`Reader.local` and `Error.catch`, with a qualitative assessment of
type-error quality, handler-composition complexity, and whether the
pattern needs a second effect row.

Not recommended: a deep dive on capability passing (Effekt) or
evidence-struct patterns (fx-rs), because both conflict with the
Free-family commitment and would be scoped out by design. Not
recommended: a row-normalisation deep dive, because that is a design
decision to make during Option 4 implementation, not a research
question.

## 6. Relevance to port-plan

Five specific changes to `../port-plan.md` are warranted based on
Stage 1 evidence. This synthesis does not make the edits; they are
human-review items.

**Section 4.1 Option 2.** The claim that reffect is a reference
implementation with O(log n) binary-natural indices is incorrect.
reffect uses a Peano-equivalent wrapper. The option can remain in the
plan as a theoretical design point, but the "real-world reference"
line should be removed or replaced with a note that no Rust crate
currently implements Option 2 as described.

**Section 4.1 Option 1.** effing-mad's reliance on Rust nightly's
unstable `Coroutine` trait should be flagged as a practical blocker
for any Option 1 recommendation. The trait has no stabilisation
timeline.

**Section 4.1 Option 4.** corophage's per-effect lifetime parameter
`'a` is a non-obvious implementation detail that directly supports the
port's `FreeExplicit` variant. The plan's summary should note this
pattern so a later implementation does not accidentally tie effects
to `'static`.

**Section 4.1, new encoding axis.** Evidence passing (typed
handler-context dispatch) is not captured by options 1-5. Whether it
becomes a sixth option or a documented-and-declined alternative should
be decided after the Stage 2 deep dive in 5.1. Koka's compile-time
index generation is a related candidate hybrid enhancement on Option
3 or Option 4, also pending that deep dive.

**Section 4.4.** No change. The Free-family commitment is reconfirmed
by Stage 1: the codebases that reject it (fused-effects, Effekt,
fx-rs) do so by giving up properties the port explicitly wants
(first-class programs, multi-shot interpretation, handler composition
via `peel`). The Stage 2 coroutine-versus-Free dive in 5.2 may decouple
"first-class programs" from "Free monad" inside the commitment, but
will not relax the commitment itself.

No other sections of the port plan are affected by Stage 1 findings.

## Closing checklist

- [x] All sections above populated
- [x] Status updated to `complete`
- [x] `_status.md` updated to tick `_classification.md` and list any
      Stage 2 deep dives under "Stage 2: deep dives"
- [x] Word count under ~2500
