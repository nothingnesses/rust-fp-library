# The `define_effect!` input and output specification

Status: validated (item 11 step 1). The specification is validated in two halves per the adopted OQ-11A decision, and both have passed: the expressibility half (written spec instances for the Phase D shapes, no emission) is recorded in this document, and the emission half (the proof-of-concept macro generating real code under test for `State`, `Throw`, and `Catch`, commit `e4585dd6` on `feat/effects-fs1`) is recorded at the end with its outcome. The macro's full implementation and the built-in ports are item 11 steps 2 and 3.

This document specifies the input grammar and output inventory of the public `define_effect!` macro (remediation plan item 11), built fresh in `fp-macros/src/effects/` with the `fs1` slice as its expansion-equivalence baseline. It settles the generator inputs the item 21 slice review recorded, and it carries the two Phase D spec instances (`Choose`, `Coroutine`) as non-emitting artefacts until Phase D implements them.

## Scope and baseline

One `define_effect!` invocation defines one effect: the brand, the operations enum, the kind projection, the `Functor` instance, the order marker, and the row-generic smart constructors. It does not emit interpreters, handler state bundles, or rows; those are program-side surfaces built from what the macro emits. There is no registry: each invocation is self-contained, the built-in effects are ordinary invocations of the same macro (the registry convergence), and nothing is keyed by name, so the old name-collision class is gone by construction.

The expansion-equivalence baseline is the `fs1` slice's hand-written per-effect modules. The public shape generalises the slice in exactly one place: the slice's higher-order cells name the crate-internal concrete `Row` directly, while emitted higher-order cells take the row as a type parameter (see the row-recursion section). The two shapes converge through the slice's existing parameterise-and-pin convention: a row pins a parameterised brand with a `type <Effect>Pinned = <Effect>Brand<TheRow, ...>` alias, exactly as the slice's `LocalPinned`/`ListenPinned`/`BracketPinned`/`CensorPinned` aliases already do for their non-row parameters.

## Input grammar

The spec is a block of constructor signatures grouped under the effect, taking reffect's trait-like grouping as the shape reference: each operation is written as the `fn` signature its smart constructor will have, and everything else is derived from those signatures. This is the operation-enum-like spec of the adopted decision: the `fn` list is in one-to-one correspondence with the emitted operations-enum variants.

```rust,ignore
define_effect! {
	/// Doc comment for the effect (emitted onto the brand).
	#[handler_state(shared_by_reference)]
	pub effect State<S: 'static> {
		/// Doc comment for the operation (emitted onto the variant and constructor).
		fn get() -> S;
		fn put(value: S) -> ();
	}
}
```

Grammar elements:

- Visibility: any Rust visibility before `effect`; it is applied to every emitted item.
- Effect generics: ordinary generic parameters with bounds (`pub effect Except<E: 'static>`). Every effect type parameter must be `'static` in the v1 grammar (the erased `Box`-store substrate the slice runs on); the `Explicit`-family lifetime axis (corophage's borrowed-resume consideration) is a recorded deferral below.
- Operations: `fn name(payload...) -> Resume;`. The return type is the continuation position: it is the value the handler resumes the continuation with. An omitted return type means `-> ()`.
- No-resume operations: `fn throw() -> !;`. A never return means the operation aborts and carries no continuation at all; the emitted variant stores `PhantomData` of the hole instead of a callable, exactly as the slice's `ThrowF`/`EmptyF` do. Illegal states are unrepresentable: there is no stored continuation that must never be called.
- Payload kinds, by parameter type:
  - A plain type (`value: S`, `key: &'static str`) is a by-value payload field.
  - `Program<T>` is a sub-program over the ambient row with result `T`; it is stored as `Free<R, T>` where `R` is the row parameter the macro adds to the effect (see the row-recursion section).
  - `impl FnOnce(Args...) -> Ret` is a callable payload, stored as `Box<dyn FnOnce(Args...) -> Ret + 'a>`. `Ret` may itself be `Program<T>` (a program-returning callable, like `Catch`'s `recover` and `Bracket`'s `body`/`release`). The constructor parameter is literally the written `impl FnOnce` type, so the spec line and the constructor signature read identically.
  - `impl Fn(Args...) -> Ret` is reserved for re-callable payloads and is rejected in v1 (see the multi-shot axis).
- Order derivation: the effect is higher-order if and only if any operation carries a `Program<T>` payload directly or in a callable return position; otherwise it is first-order. The `OrderOf` marker is computed from this, never declared, so the marker and the cell shape derive from one input and cannot diverge.
- `#[multi_shot]` on an operation declares that a handler may resume the continuation more than once (nondeterminism). The v1 emission rejects it with a "reserved for the multi-shot stores" error rather than emitting single-shot storage that would be wrong; the grammar carries it now so the Phase D shapes typecheck as specs (parse, don't validate: the axis is expressible, and an unimplementable spec is rejected at the boundary rather than emitting code with the wrong drop/resume semantics).
- `#[handler_state(...)]` on the effect is required and declares the handler-state taxonomy: `none` (the handler holds no state; `Throw`, `Catch`, `Coroutine`), `scoped_by_value` (a by-value field scoped by derivation at recursive interpretation, like `Reader`'s environment under `Local`), `shared_by_reference` (a shared cell surviving recursive interpretation, like `State`/`Writer`/`Fresh`/`Input`/`KVStore`), or `threaded_by_value` (an accumulator threaded through the interpretation loop and forked per branch; reserved until item 14's threaded runners exist). The attribute is declarative: it is emitted into the generated documentation so every effect states its state discipline explicitly, and it forces the definer to decide the taxonomy at definition time.
- `#[crate_path(...)]` on the effect overrides the emitted item paths' crate root (default `::fp_library`), so fp-library itself and its integration tests can expand the macro against `crate` paths.
- Doc comments on the effect and on each operation are mandatory and pass through to the emitted items; the macro appends the derived facts (order, handler-state class, and for higher-order effects the elaboration depth contract) to the generated docs.

## Naming and collision rules

Names derive only from the spec text, with no implicit renaming:

- The effect name `Name` derives `NameBrand` (the brand) and `NameF` (the operations enum).
- Each operation `fn foo_bar` derives the variant `FooBar` by the deterministic rule: split the snake_case name on underscores, capitalise each segment, concatenate. The smart constructor keeps the spec name `foo_bar` verbatim.
- Collisions are expansion errors, never silently suffixed: two operations whose derived variant names coincide; an operation whose constructor name coincides with a derived type name; an effect whose derived names collide with each other. The slice's ad hoc `_e`/`_op` suffixes (`throw_e`, `identity_op`) existed only because the slice re-exports every constructor into one flat `pub(crate)` block; the public surface is module-per-effect with user-controlled imports (`use except::throw as throw_e` at the consumer if both are wanted flat), so cross-effect constructor name reuse is not a collision and the macro does not rename around it.
- Rust keywords are not renamed around either: an operation that would need a keyword name must be spelled differently by the definer (the precedent is `yield_value` for `Coroutine`, carried over from the pre-FS-1 surface).

## Output inventory

One invocation emits, for an effect `Name` with operations `op_i`:

1. The brand: `pub struct NameBrand<R?, Generics...>(PhantomData<...>)`, with the row parameter `R` present exactly when the effect is higher-order, and `PhantomData` present exactly when the brand has type parameters (a bare `pub struct NameBrand;` otherwise, as `StateBrand` in the slice).
2. The operations enum: `pub enum NameF<'a, R?, Generics..., A>` with one variant per operation. A variant stores its by-value payload fields, its `Program` payloads as `Free<R, T>`, its callable payloads as `Box<dyn FnOnce(...) -> ... + 'a>` (with `Program` returns as `Free<R, T>`), and its continuation as `k: Box<dyn FnOnce(Resume) -> A + 'a>`, or `PhantomData<A>` for a `-> !` operation. First-order variants are tuple variants (the payload fields in spec order, then the continuation or the `PhantomData`, matching the slice's `StateF`/`ExceptF` shapes); higher-order variants use named fields including `k` (they are cells the interpreter destructures by name, matching the slice's cell structs). Higher-order effects with one operation are still enums (a single-variant enum, as the slice's normalised `ThrowF`/`EmptyF` already are): the interpreter's `match` is total over the variants, so adding an operation to any effect forces every dispatch arm to be revisited (explicit failure, total matches). This is a deliberate, documented delta from the slice's remaining struct-shaped higher-order cells (`CatchCell`, `BracketCell`, `LocalCell`, `ListenCell`, `CensorCell`); step 2 normalises those to the emitted single-variant-enum shape as part of the expansion comparison.
3. The kind projection: an `impl_kind!` mapping `NameBrand<...>` to `NameF<'a, ..., A>` under the `type Of<'a, T: 'a>: 'a` kind.
4. The `Functor` instance: a total match over the variants, composing the mapped function into each continuation (`k: Box::new(move |x| f(k(x)))`), rebuilding `PhantomData` for no-resume variants, and moving every other field through unchanged.
5. The order marker: `impl OrderOf for NameBrand<...> { type Order = FirstOrder | HigherOrder; }`, computed by the order-derivation rule.
6. The smart constructors: one per operation, row-generic. For a first-order operation the shape is `pub fn get<R, I>() -> Free<R, S>` where the bounds require `Apply!(R::Of<'static, S>)` to implement the coproduct injection of `Coyoneda<'static, NameBrand<...>, S>` at an inferred index `I` (the `frunk_core` `CoprodInjector` machinery the slice's concrete `Coproduct::inject` calls resolve through); the body builds the variant with the identity continuation, lifts it into `Coyoneda`, injects it into the row cell, and wraps it with `Free::lift_f`, exactly as the slice constructors do with the row generalised. Higher-order constructors additionally take their `Program` payloads as `Free<R, T>` parameters and their callable payloads as the spec's literal `impl FnOnce` parameters. `Coyoneda`-wrapping is the row-cell convention: constructors assume the row's cells are `CoyonedaBrand`-wrapped brands, which is what row assembly emits.
7. Documentation: the spec doc comments plus the derived facts (order; handler-state class; for higher-order effects, the elaboration depth contract sentence). The emitted items must carry the documentation attributes `document_module` validation expects, so the built-ins regenerated through the macro can graduate into `#[fp_macros::document_module]` modules and clear the `fs1` exception; the exact attribute set is fixed at step 2 against `document_module`'s validators.

Not emitted, by design:

- No `WrapDrop` impl per effect: the row's cells are `CoyonedaBrand`-wrapped, and `CoyonedaBrand<F, Store>` already carries the blanket `WrapDrop` for any kind-conforming `F` (the cell's continuation is closure-captured, so a suspended chain is shallow per layer), with `CoproductBrand` delegating across the row. This is the drop policy: first-order effects need nothing; higher-order cells own materialised sub-programs by value, so dropping nested higher-order cells recurses with the same depth budget as elaborating them, which is the documented depth contract (native stack grows with higher-order nesting depth, not program length).
- No interpreter arms and no `Handlers` bundle: dispatch arms are program-side code. The non-divergence guarantee (generator input 1) is structural rather than emitted: the arm's shape IS the operations enum, the interpreter's `match coyo.lower()` is checked total by the compiler against that enum, and the order marker is computed from the same operation list, so there is no second source of truth that could drift.
- No row types: rows are assembled program-side (the row-assembly companion below).

## Row integration and the recursion knot

Higher-order cells contain `Free<R, T>` sub-programs over the very row `R` they sit in. The slice cuts this knot with module-level concreteness: `CatchCell` names the crate-internal `Row` alias, and the cycle (`Row` names `CatchBrand`, whose projection names `Row`) is legal because it passes through the lazily-normalised `Of` associated-type projection. A public macro cannot name the user's row, so the emitted brand carries the row as its first type parameter (`CatchBrand<R, RAction>`) and the cell stores `Free<R, RAction>`.

That parameter makes a self-referencing type-alias row illegal: `type MyRow = CoproductBrand<..., CoyonedaBrand<CatchBrand<MyRow, ()>>, ...>` is an alias cycle (rejected at alias expansion, before any projection could break it). The knot is cut nominally: a row containing higher-order effects is declared as a nominal brand (`struct MyRow;`) whose kind projection maps to the coproduct chain, so the self-reference sits inside the projection body and is lazy, exactly like the slice's arrangement one level up. The nominal row then delegates `Functor` (and anything else interpretation needs) to the coproduct chain it projects to; the delegation is mechanical, and a row-assembly companion macro (the `effects!` successor) is the ergonomic wrapper that emits the nominal brand plus its delegations. The companion macro is item 11 step 2 scope; the emission proof of concept hand-writes one nominal row to validate the knot itself.

Recursive constructor bounds (`R` appearing inside its own injection bound, as in `catch<R, ...>(...)` requiring the row to contain `CatchBrand<R, ...>`'s cell) are ordinary Rust (a type parameter may appear in its own bounds); the proof of concept exercises this too.

Pinned rows: a row commits to a parameterised brand's concrete types through the slice's parameterise-and-pin convention (`type CatchPinned = CatchBrand<MyRow, ()>`), stating the commitment once for the row and its dispatch arms.

## Generator-input resolutions

The eleven observations the item 21 review recorded as generator inputs, each resolved by this specification:

1. Marker/arm non-divergence: both derive from the operation list; the marker is computed (order-derivation rule), the arm shape is the emitted enum, and interpreter matches are compiler-checked total against it. No declared order exists to drift.
2. Template conventions as emitted shapes: callables are `Box<dyn FnOnce + 'a>`; cells are parameterised and pinned at the row (`*Pinned` aliases); the payload shape is the one operations enum per effect (single-variant for single-op effects, including higher-order ones as a documented delta from the slice's struct cells).
3. Deterministic naming: the naming and collision rules section; names are spec-verbatim with a deterministic variant derivation and expansion errors on collision, no implicit suffixes.
4. `Handlers` field taxonomy: the required `#[handler_state(...)]` attribute with the four classes (`none`, `scoped_by_value`, `shared_by_reference`, `threaded_by_value` reserved for item 14), emitted into docs. The expressibility half surfaced the need for `none` and `threaded_by_value` beyond the review's two.
5. Parse-don't-validate construction seam: handler state is program-side, so the macro emits nothing for it; the convention stands that built-in handler-state types validate at construction (seeded builders in the `Fixture` style) and interpreters trust them. Recorded here so steps 2 and 3 build the built-ins' state types to it.
6. General interpose: the emitted `Functor` plus constructors are sufficient for single-layer walkers (the slice's shape); the general transformer requires re-callable replacement closures (`Fn`, not `FnOnce`, called once per occurrence) and recursion into continuations, and is interpreter-side vocabulary scoped to steps 2 and 3, not effect-definition scope.
7. Elaboration recursion contract: stated in emitted higher-order docs and mirrored by the drop story (both scale with nesting depth, not program length).
8. Append-only log invariant: a handler-contract statement in the built-in `Writer`'s docs (its `#[handler_state(shared_by_reference)]` accumulator is append-only, which is what makes `Listen`'s delta slicing sound). Handler contracts are definer-authored prose the macro passes through.
9. Error-accepting recover: callable payloads take arbitrary inputs, so a typed catch is `fn catch(action: Program<RAction>, recover: impl FnOnce(E) -> Program<RAction>) -> RAction;` with `E` an effect generic. The slice's unit-recover catch is the degenerate case (its recoverable abort, the bare `Throw`, carries no payload).
10. Non-`Copy` bracket resources: a resource that flows into more than one callable payload must be duplicable by the elaborator, so the effect's generics must bound it `Clone` (with `Copy` the trivial case) and its docs must state which callable receives the original. The macro enforces nothing structurally (consumption is an interpreter-side fact); the built-in `Bracket` adopts `R: Clone` with `release` receiving the clone.
11. Stable kind-bound naming: no hash-named generated kind trait (`Kind_cdc7cd43dac7585f`) is ever hand-written into the emission. The macro lives inside `fp-macros`, so it names the kind trait through the same generator that produces it (the name is computed from the parsed `type Of<'a, T: 'a>: 'a` signature by the code `trait_kind!` itself uses), one source of truth that cannot drift; this is how the proof of concept emits its projections. The human-facing stable re-export alias landed at step 2 as `kinds::LifetimeUnaryKind`, with the slice's order-directed peel machinery as its first hand-written consumer.

## Recorded deferrals

- Multi-shot continuation storage (`#[multi_shot]`, `impl Fn` payloads): the axis is expressible in the grammar and rejected at emission until Phase D lands the multi-shot interpreter stepping at the `Rc`/`Arc` stores; single-shot `Box<dyn FnOnce>` storage cannot host a twice-resumed continuation, and emitting it anyway would be the illegal-state route. Trade-off: `Choose` cannot be emitted until then; accepted because nothing before item 14 can run it either.
- Thread-safety bounds (`Send + Sync` payload/callable bounds for `Arc`-store rows): not in the v1 grammar; the slice validates `Arc`-store rows type-compose only. A `#[thread_safe]`-style axis rides item 18's async/runtime work, where its consumer exists.
- `Explicit`-family lifetimes (corophage's borrowed-resume `Resume<'r>` consideration): v1 targets the erased `'static` substrate the slice runs on. The `fn`-signature grammar extends naturally with lifetimes when the `Explicit` family gains its effect surface; deferring keeps step 1's validation against code that exists.
- `threaded_by_value` handler state: reserved until item 14's `handle_accum` loop defines what is emitted or documented for it.

## Expressibility validation: the Phase D shapes

Per the adopted OQ-11A decision, the two deleted Phase D effects are written as spec instances with no emission, checked against the backup-branch semantics (`backup/effects-dual-row-pre-fs1`) and items 14, 16, and 17. They are carried here as the non-emitting artefacts until Phase D re-ports them.

### `Choose` (items 14 and 17)

Backup-branch semantics: `Choose a = Alt (Boolean -> a)` (the purescript-run shape); the handler runs the continuation once per branch (`true` then `false`), so the continuation is resumed more than once and only the multi-shot wrappers shipped its constructor.

```rust,ignore
define_effect! {
	/// Nondeterministic branching.
	#[handler_state(threaded_by_value)]
	pub effect Choose {
		/// Fork interpretation: the handler resumes this continuation once per
		/// branch, `true` for the left branch and `false` for the right.
		#[multi_shot]
		fn alt() -> bool;
	}
}
```

Checks:

- The operation is expressible as one no-payload, `bool`-resume operation with the `#[multi_shot]` marker carrying the resumed-twice contract; the marker is what switches the emitted continuation storage to a re-callable form when Phase D implements it.
- Item 14 (threaded accumulators): the handler-state class is `threaded_by_value` (the accumulator forks per branch in the `handle_accum` loop), which is what forced the taxonomy's fourth class; nothing else in item 14 touches the effect definition, since the threaded runners are interpreter-side.
- Item 17 (scoped choice): `ChooseH` is an ordinary higher-order effect in this grammar, `fn choose_scoped(left: Program<RAction>, right: Program<RAction>) -> RAction;`, elaborated program-side into first-order `alt` operations; no grammar extension is needed.

### `Coroutine` (item 16)

Backup-branch semantics: `Coroutine<Out, In>` yields an `Out` to the runner and resumes with an `In` (`Yield(out, resume)`); the runner (`run_coroutine`) removes the coroutine cell and returns either a completed result or a yielded output paired with a resume continuation.

```rust,ignore
define_effect! {
	/// Cooperative yielding: emit an `Out` to the runner, resume with an `In`.
	#[handler_state(none)]
	pub effect Coroutine<Out: 'static, In: 'static> {
		/// Yield `output` to the runner; the continuation resumes with the
		/// runner's answer.
		fn yield_value(output: Out) -> In;
	}
}
```

Checks:

- The operation is expressible as one by-value payload (`Out`) with a distinct resume type (`In`), both effect generics; this instance is the exerciser for effect-level generics on a first-order effect (the slice's `Except<E>` covers the single-parameter case).
- The yielded-or-done runner shape is an interpretation surface (`run_coroutine`'s return type), not an effect-definition concern, so it needs nothing from the grammar; item 16's streaming vocabulary (producer/consumer/transformer aliases, `connect`, for-substitution) is row vocabulary over this one operation.
- The `yield` keyword collision is resolved by the definer-side naming rule (`yield_value`), matching the pre-FS-1 surface's precedent.

Verdict: both shapes are expressible in the grammar with no extensions beyond what the instances themselves forced and this document records (the `#[multi_shot]` marker, the `none` and `threaded_by_value` handler-state classes); the grammar needed no changes for items 16 and 17, and item 14's impact is confined to the reserved taxonomy class. The expressibility half passes.

## Emission validation: the proof of concept

The emission half generates real code under test for three effects chosen to cover every v1 emission path:

- `State` (two first-order operations, distinct resume types, `shared_by_reference`): the plain-payload and continuation paths against the slice's exemplar.
- `Throw` (one no-resume operation, `none`): the `-> !`/`PhantomData` path.
- `Catch` (one higher-order operation with a `Program` payload, a program-returning callable, and a value-threading continuation): the row-parameterised brand, the nominal-row knot, and the recursive constructor bound. `Catch` is chosen over `Bracket` because it is the minimal cell exhibiting every higher-order field kind (`Bracket` adds more callables of the same kinds but nothing type-theoretically new), and its value-threading continuation is the general elaboration shape the slice pinned.

The proof of concept lives as a fresh `define_effect` module in `fp-macros/src/effects/` plus a `#[cfg(test)]` consumer module in `fp-library` (using the `#[crate_path(crate)]` override), which assembles a hand-written nominal row over the three emitted effects, hand-writes the mini interpreter against the emitted enums, and re-proves the bucket-A State-with-Catch ordering oracle (a write before a caught throw survives) on macro-emitted definitions. Pass criteria: the emitted code compiles, the oracle test is green, the nominal-row knot and recursive bounds typecheck, and the emitted shapes match the slice template modulo the two documented deltas (row parameter; single-variant enums for higher-order cells). Fallback per the evidence-grounding principle: if the row-generic constructor bounds prove unwritable in stable Rust, the fallback is emitting constructors concrete-per-row behind the row-assembly companion macro (the row macro instantiates each member effect's constructors against the row it emits), which preserves the public surface at the cost of monomorphic constructor homes; if that also fails, surface the impasse.

Outcome: passed, with no fallback needed (commit `e4585dd6` on `feat/effects-fs1`: `fp-macros/src/effects/define_effect.rs` and the consumer `fp-library/src/types/effects/define_effect_poc.rs`). The emitted code compiles under the full `just verify` gate (format, clippy at deny-warnings, the full test suites, docs) and the effects-off build; the four proof tests are green (the ordering oracle, uncaught-throw abort, recovery-skipped-on-success, and the computed order markers checked by type-level probes); the nominal-row knot behaved exactly as specified (the row's self-reference is legal through the brand's kind projection and a type alias in the same position is a definition cycle); and the row-generic constructors resolved by inference at every call site, including `catch`'s recursive `R`-in-its-own-bound spelling. Two v1 boundaries surfaced and are recorded: the emission named the order-marker vocabulary at its then crate-internal home, so the macro was usable only in-crate (cleared at step 2, which graduated `OrderOf`/`FirstOrder`/`HigherOrder` to the public `types::effects::order` module, making the default `::fp_library` emission paths resolve externally); and the proof interpreter reuses the slice's parameterise-and-pin convention for the row's `Catch` commitment, confirming that convention carries over to emitted brands unchanged.

Step 2 carried the validation onto the slice itself (commits `75601a7d` and `7279d52d` on `feat/effects-fs1`): `define_row!` was born per the OQ-11B decision and validated by two consumers (the proof-of-concept row converted to it, then the slice's own `Row`, whose alias spelling the nominal emission replaced); `State` and `Catch` were ported onto `define_effect!` in place with all slice tests green; and the cargo-expand comparison against the pre-port hand-written modules matched exactly modulo the two documented deltas (effect generics pinned at the `Row` instead of inside the effect; row-generic constructors bounded on coproduct injection in place of concrete-`Row` ones), with `Catch`'s cell normalised to the single-variant named-field enum by regeneration. One emission fix landed with it: both macros now wrap their kind impls in an anonymous-const scope carrying the `kinds` glob, so the emission is self-contained at any invocation site instead of inheriting `impl_kind!`'s requirement that the caller glob-import `kinds`.
