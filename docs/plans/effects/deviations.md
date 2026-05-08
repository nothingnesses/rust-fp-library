# Implementation deviations: effects port

This file is the post-write log of per-step implementation
choices that diverged from the original plan text. Each entry
describes a step's implementation choice, why it diverged, and
(where useful) what the plan text said vs what shipped.

Entries are append-only, grouped by phase and step. They are
load-bearing for code review (so reviewers know why a step's
output isn't a literal transcription of the plan text) and for
future maintenance (so the next implementer reading the code
understands subtle choices).

For resolved blockers (load-bearing questions that paused
implementation until investigated), see [resolutions.md](resolutions.md).
For active blockers, current progress, and the implementation
phasing, see [plan.md](plan.md).

## Phase 4: Scoped effects (heftia-inspired dual row)

### Step 3.3.3: marker-struct doctests for 4 of 6 smart constructors; `ignore` for `ArcRun::bracket`; `BoxBracketExplicit` substrate refined to `Box<FreeExplicit<...>>`

Step 3.3.3 lands six per-wrapper `bracket` smart constructors at [run.rs](../../../fp-library/src/types/effects/run.rs), [rc_run.rs](../../../fp-library/src/types/effects/rc_run.rs), [arc_run.rs](../../../fp-library/src/types/effects/arc_run.rs), [run_explicit.rs](../../../fp-library/src/types/effects/run_explicit.rs), [rc_run_explicit.rs](../../../fp-library/src/types/effects/rc_run_explicit.rs), and [arc_run_explicit.rs](../../../fp-library/src/types/effects/arc_run_explicit.rs). Each pairs the wrapper with its substrate-correct cell from B19 closure: `Run` -> `BoxBracket` (Free), `RcRun` -> `Bracket` (RcFree), `ArcRun` -> `SendBracket` (ArcFree), `RunExplicit` -> `BoxBracketExplicit` (`Box<FreeExplicit>`), `RcRunExplicit` -> `BracketExplicit` (RcFreeExplicit), `ArcRunExplicit` -> `SendBracketExplicit` (ArcFreeExplicit). Three deviations from a naive port of the `local` smart constructor template:

- **Marker-struct doctests for 4 of 6 wrappers.** `Run::bracket`, `RcRun::bracket`, `RunExplicit::bracket`, `RcRunExplicit::bracket` doctests use the marker-struct workaround validated by the [B18 POC](../../../fp-library/tests/poc_bracket_marker_row.rs): a zero-sized `struct ScopedRow` with manual `Kind` (via `impl_kind!`), `WrapDrop`, and `Functor` impls delegating to an `UnderlyingRow` type alias that contains the wrapper's bracket brand. The marker breaks the type-alias cyclicity Rust would otherwise reject for `type ScopedRow = CoproductBrand<*BracketBrand<P, NodeBrand<R, ScopedRow>, A, B>, CNilBrand>`. `ArcRunExplicit::bracket`'s doctest additionally needs `SendFunctor` on the marker (delegating to `UnderlyingRow`) and adds a `#![recursion_limit = "512"]` attribute at the doctest top to satisfy rustc's type-check recursion budget through the `ArcFreeExplicit`-substrate cascade.

- **`ArcRun::bracket` doctest uses `ignore` block + simpler runnable assertion.** The marker-struct workaround that succeeds for the other five wrappers fails for `ArcRun::bracket` because `SendBracketBrand`'s Kind impl bound (`Sub: Kind_cdc...<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>`) interacts with the marker's `Sub = NodeBrand<CNilBrand, ScopedRow>` projection to create a `Send + Sync` evaluation cycle that exceeds rustc's overflow limit (raising `recursion_limit` to 2048 did not help; the cycle is structural, not just deep). The smart constructor itself compiles cleanly when used in non-doctest contexts (verified via `just check -p fp-library`); only the doctest fixture's marker-struct setup triggers the overflow. The doctest is split into a runnable simple-program assertion (constructs `ArcRun::pure` on a non-recursive scoped row, satisfies the `#[document_examples]` macro's runnable-assertion requirement) plus an `ignore` block sketching the bracket call site for documentation purposes. End-to-end exercise of `ArcRun::bracket` lives in step 3.3.4's `tests/run_bracket.rs` integration tests, where the marker struct's `Send + Sync` is checked once at the test-crate level rather than recursively in a doctest fixture.

- **`BoxBracketExplicit` substrate refined from `FreeExplicit<'a, Sub, _>` to `Box<FreeExplicit<'a, Sub, _>>`.** Discovered during `RunExplicit::bracket` implementation: [`FreeExplicit::wrap`](../../../fp-library/src/types/free_explicit.rs)'s expected layer-program type is `<F>::Of<'a, Box<FreeExplicit<'a, F, A>>>` (boxed; the outer `Box` is part of the substrate's type contract because `FreeExplicit` is by-value, unlike `Free`/`RcFree`/`ArcFree`/`RcFreeExplicit`/`ArcFreeExplicit` which embed an `Rc`/`Arc`/`Option<...>` inner). Without the outer `Box`, `Node::Scoped(layer)` rejects the layer with a type-mismatch on the GAT projection. Mirrors the `BoxLocal`-on-`RunExplicit::local` precedent which also threads `Box<FreeExplicit<...>>` as the cell's program type. The B19 closure's initial `BoxBracketExplicit` cell design used `FreeExplicit<'a, Sub, _>` directly; the discovery during step 3.3.3 forced the refinement, applied retroactively at the B19 closure commit's bracket.rs and propagated to the new smart constructor (which calls `Box::new(acquire.into_free_explicit())`, `Box::new(body(a).into_free_explicit())`, etc.). Rc/Arc Explicit-family cells (`BracketExplicit`, `SendBracketExplicit`) keep the unboxed `RcFreeExplicit<'a, Sub, _>` / `ArcFreeExplicit<'a, Sub, _>` substrate (matching their respective `wrap` signatures).

- **`ArcRunExplicit::bracket` where-clause carries 6 `Send + Sync` projections.** The cell stores three differently-typed program returns over the same substrate brand: `ArcFreeExplicit<'a, Sub, A>` (acquire), `ArcFreeExplicit<'a, Sub, (A, B)>` (body), `ArcFreeExplicit<'a, Sub, ()>` (release). Each requires a separate `Send + Sync` bound on the `R` and `ScopedRow` projections at that specific GAT-filled type, plus the `Clone + Send + Sync` bound on the `NodeBrand<R, ScopedRow>` projection at the body's `(A, B)` type. Total: 6 row-projection bounds. Comparable to `ArcRunExplicit::local`'s where clause (which has 3 such bounds because Local has only one program-result type), the doubling reflects Bracket's three-step cell shape. Could be flattened with a unified result enum (e.g., `BracketStep<A, B>`), but that would move the acquire-vs-body-vs-release invariant from compile time to runtime; per the user's compile-time-invariant preference, the explicit triple is retained.

### Step 3.3.1 B19 closure: substrate split per Free family (3 existing cells fixed + 3 new Explicit-family cells)

The Bracket Val foundational scaffold (step 3.3.1, commit `1be2af3e`) hardcoded `Free<Sub, _>` in all three sibling cells' field types (`BoxBracket` / `Bracket` / `SendBracket`). This was substrate-correct for `Run::bracket` only; the other 5 wrappers each have a distinct substrate, so a body closure for any non-Run wrapper would return its wrapper's program type (`RcFree` / `ArcFree` / `FreeExplicit` / `RcFreeExplicit` / `ArcFreeExplicit`), which doesn't match the cell's hardcoded `Free<Sub, _>`. Surfaced as B19; closed via Option C (split into 6 cells per Free family) per the [B19 closure entry](resolutions.md#resolved-2026-05-08-phase-4-step-3.3.1-foundational-scaffold-cells-hardcode-freesub-_-b19-closed-via-option-c-split-into-6-cells-per-free-family).

The closure rework lands as a single `feat(effects)` commit on top of `1be2af3e` and `46754fc0` (no per-step subnumbering). Concrete contents:

- **Existing cells fixed.** [`Bracket<'a, P, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) (RcBrand sibling) field types switch `Free<Sub, _>` to `RcFree<Sub, _>`; [`SendBracket<'a, P, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) (ArcBrand sibling) switches to `ArcFree<Sub, _>`. [`BoxBracket<'a, P, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) keeps `Free<Sub, _>` (already correct for `Run`). `SendBracket`'s where clause acquires the GAT-projection-Send-Sync bound on `Sub` (`Sub: WrapDrop + Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync> + 'static`) because `ArcFree<Sub, _>` requires `Sub` to satisfy that bound for the GAT projection to be `Send + Sync` (mirrors the `ArcRun<R, S, A>` precedent at [arc_run.rs](../../../fp-library/src/types/effects/arc_run.rs)).

- **Three Explicit-family cell siblings added.** [`BoxBracketExplicit<'a, P, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) (stores `FreeExplicit<'a, Sub, _>`); [`BracketExplicit<'a, P, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) (stores `RcFreeExplicit<'a, Sub, _>`); [`SendBracketExplicit<'a, P, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) (stores `ArcFreeExplicit<'a, Sub, _>`). Each parallels its Erased sibling line-for-line with a manual `Clone` for the Rc/Arc-pointer cells (no `Clone` for the Box-pointer cell). Bound difference vs the Erased family: the Explicit-family cells use `Sub: WrapDrop + 'a` (the lifetime-bearing variant); the Explicit-family `ArcFreeExplicit<'a, F, A>` does not require the GAT-projection-Send-Sync bound on `F` at the type level (only `F: WrapDrop + 'a`), so `SendBracketExplicit`'s where clause is simpler than `SendBracket`'s.

- **Three Explicit-family brand declarations added** at [`fp-library/src/brands/effects.rs`](../../../fp-library/src/brands/effects.rs): `BoxBracketExplicitBrand<P, Sub, A, B>` / `BracketExplicitBrand<P, Sub, A, B>` / `SendBracketExplicitBrand<P, Sub, A, B>`, each a 4-param `PhantomData` brand mirroring its Erased sibling.

- **Per-brand trait impls.** Each new Explicit-family brand gets the same five substrate-required impls as its Erased sibling: `Functor::map` is identity (returns `fa` unchanged because `Of<'a, X>` is independent of `X` under Option A); `SendFunctor::send_map` is identity for `SendBracketExplicitBrand<ArcBrand, _, _, _>` and a stub for `BoxBracketExplicitBrand<BoxBrand, _, _, _>` and `BracketExplicitBrand<RcBrand, _, _, _>` (mirrors the Erased-family stub precedent because the BoxBrand and RcBrand projections are not `Send + Sync`); `WrapDrop::drop` returns `None`; `Extract::extract` is a panicking `unreachable!` stub; `RefFunctor::ref_map` is a panicking-stub mirror for `BoxBracketExplicitBrand` (cell is non-`Clone`) and `Clone::clone(fa)` for `BracketExplicitBrand` (`SendBracketExplicitBrand` deliberately omits `RefFunctor`, mirroring the [`SendBracketBrand`-no-`RefFunctor` deviation](#step-332-sendbracketbrand-skips-reffunctor-mirrors-sendcatchbrand--sendlocalbrand--sendreflocalbrand-precedents)).

- **Doctest substrate.** The 4 SendBracket doctests and the SendFunctor/WrapDrop/Extract doctests for SendBracketExplicit use [`IdentityBrand`](../../../fp-library/src/types/identity.rs) as the substrate brand instead of [`ThunkBrand`](../../../fp-library/src/types/thunk.rs); `Thunk<'a, A>` contains `Box<dyn FnOnce>` which is not `Sync`, so `ThunkBrand` cannot satisfy `ArcFree`'s GAT-projection-Send-Sync bound. `IdentityBrand` is the standard substrate for `ArcFree` doctests at [arc_free.rs](../../../fp-library/src/types/arc_free.rs); the existing 6 Bracket (RcBrand) doctests retain `ThunkBrand` since `RcFree<F, A>` only requires `F: WrapDrop + 'static`.

- **POC unchanged.** [`fp-library/tests/poc_bracket_marker_row.rs`](../../../fp-library/tests/poc_bracket_marker_row.rs) was tested against `BoxBracket` + `Free<NodeBrand<CNilBrand, MarkerRow>, _>`; both are unchanged by B19 closure, so the POC remains valid as-is.

The substrate split doubles the cell-and-brand surface from 3 to 6 (3 Erased + 3 Explicit) per scoped effect; this is bounded mechanical work, not architectural debt. Step 3.3.5 (RefBracket Ref foundational scaffold) inherits the same per-Free-family split (4 RefBracket cells = 2 pointer brands x 2 Free families per [B15 sibling-count asymmetry](resolutions.md#resolved-2026-05-07-phase-4-step-3.3-sub-step-splitting--bracket-acquire-field-layout-cycle-reuse--refbracket-sibling-count-asymmetry-b14--b15--b16-closed)).

### Step 3.3.2: `SendBracketBrand` skips `RefFunctor`; mirrors `SendCatchBrand` / `SendLocalBrand` / `SendRefLocalBrand` precedents

Step 3.3.2 lands `RefFunctor` impls for [`BoxBracketBrand<BoxBrand, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) (stub) and [`BracketBrand<RcBrand, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs) (clone-based) but deliberately omits `RefFunctor` for [`SendBracketBrand<ArcBrand, Sub, A, B>`](../../../fp-library/src/types/effects/bracket.rs). The omission mirrors the [`SendCatchBrand`-no-`RefFunctor` deviation](#step-312-sendcatchbrand-skips-reffunctor-the-cascade-through-arcrunexplicitbrand-does-not-require-it), the [`SendLocalBrand`-no-`RefFunctor` deviation](#step-322-sendlocalbrand-skips-reffunctor-the-cascade-through-arcrunexplicitbrand-does-not-require-it), and the [`SendRefLocalBrand`-no-`RefFunctor` deviation](#step-326-sendreflocalbrand-skips-reffunctor-the-cascade-through-arcrunexplicitbrand-does-not-require-it) for the same structural reasons.

The cascade chain that would require `RefFunctor` on a scoped-row brand is `*RunExplicitBrand: RefFunctor` -> `*FreeExplicitBrand: RefFunctor` -> `NodeBrand<R, S>: RefFunctor` -> `S: RefFunctor`. The brand-level docstring on [`ArcRunExplicitBrand`](../../../fp-library/src/brands/effects.rs) records that the `Ref`/`SendRef`-family hierarchy is not reachable through brand-level delegation because `ArcFreeExplicitBrand` does not implement it. The cascade therefore terminates at `ArcFreeExplicitBrand` without ever requiring `RefFunctor` on the scoped row's brands; `SendBracketBrand: !RefFunctor` is benign for the Arc-family substrate.

A hypothetical `SendBracketBrand: RefFunctor` impl would also fail structurally for the same reason `SendBracketBrand: !Functor` is unproblematic under Option A: `RefFunctor::ref_map`'s closure parameter `func: impl Fn(&A) -> B + 'a` lacks the `Send + Sync` bounds that `<ArcBrand as ToDynSendFn>::new` would require for storing the new closures, and (more importantly) no useful work is possible since the cell is structurally fixed by `Sub` / `A` / `B` and the natural impl is identity-clone.

The [`BracketBrand<RcBrand, Sub, A, B>::ref_map`](../../../fp-library/src/types/effects/bracket.rs) impl is the simplest in the codebase: `fa.clone()` (one line). This is a direct consequence of Option A's identity `Functor::map` (step 3.3.1): under Option A the cell's GAT projection `Of<'a, X>` is independent of the trait's universal type parameter X, so any X-to-Y mapping reduces to a clone. The [`BoxBracketBrand<BoxBrand, Sub, A, B>::ref_map`](../../../fp-library/src/types/effects/bracket.rs) impl is a panicking-stub mirror (BoxBracket is non-Clone): three closures using `unreachable!()` returns coerce to the cell's three different concrete return types (`Free<Sub, A>` / `Free<Sub, (A, B)>` / `Free<Sub, ()>`) without needing to actually construct any of them. The path is reachable only through synthetic non-Coyoneda first-order rows on `RunExplicit`'s `RefFunctor` cascade, which real programs do not exercise.

No brand-projection helpers were introduced. Catch / Local / RefLocal needed `*_modify_ref` / `*_action_thunk_ref` helpers because their cells contained the trait's universal `A` type parameter (Catch's action and handler return `A`; Local's action thunk returns `A`), and the GAT projection inside the trait impl's HRTB-bearing scope failed to normalize against the concrete enum (the [`unwrap_first` precedent](../../../fp-library/src/types/effects/arc_run.rs)). Bracket's cell type doesn't reference the trait's universal `A` at all (under Option A the cell's body return is `Free<Sub, (A_brand, B_brand)>` where A_brand and B_brand come from the brand, not from the trait's universal scope); the GAT projection normalizes cleanly without escape helpers.

### Step 3.3.1: Option C (FreeShape HKT-trait decomposition) failed Rust's well-formedness check; fell back to Option A (5-param struct + 4-param brand)

Step 3.3.1 ships the [Bracket Val foundational scaffold](../../../fp-library/src/types/effects/bracket.rs) using Option A (5-param struct `Bracket<'a, P, Sub, A, B>` and 4-param brand `BracketBrand<P, Sub, A, B>`) rather than the user-approved primary Option C (3-param brand `BracketBrand<P, A, B>` with `<X as FreeShape>::F` projection). Per the [B17 plan-text amendment](resolutions.md#resolved-2026-05-08-phase-4-step-3.3.1-bracket-cells-three-differently-typed-program-returns-vs-substrates-single-gat-parameter-pattern-b17-closed), Option A was the explicit fallback "if FreeShape's HRTB-bearing trait surfaces structural issues during implementation."

The Option C probe wrote a `FreeShape` trait (a 2-line trait with associated types `F` and `Inner`) plus a blanket impl `impl<F, A> FreeShape for Free<F, A>`, and a `BoxBracket<'a, P, A, B, X>` cell with where-clause `X: 'a + FreeShape` so the cell could spell `acquire: <P>::Of<'a, dyn FnOnce(()) -> Free<<X as FreeShape>::F, A>>`. The probe failed at the `Functor::map` impl on `BoxBracketBrand<BoxBrand, A, B>`, which forms `<Self as Kind>::Of<'a, X>` for unbounded `X: 'a` (the `Functor` trait's method signature is fixed). Rust's well-formedness check on the GAT body required `X: FreeShape` (transitively from the cell's where-clause), but the bound is not in scope; rustc reported `error[E0277]: the trait bound X: FreeShape is not satisfied` with the help "consider further restricting type parameter X with trait FreeShape." The trait bound cannot be added because `Functor::map`'s signature is fixed by the trait. The probe was reverted; `FreeShape` is not in the codebase.

Option A carries the substrate brand `Sub` explicitly through the cell's struct and the brand's GAT projection ignores the GAT-filled `X` (the brand's `Of<'a, X>` resolves to `Bracket<'a, P, Sub, A, B>` regardless of `X`). The recursive type cycle through `Sub` (which references `NodeBrand<R, ScopedRow>` whose `ScopedRow` contains `BracketBrand<P, Sub, A, B>` recursively) is broken at the value level by `PhantomData<(P, Sub, A, B)>` on the brand: the brand is zero-sized regardless of `Sub`'s layout, so Rust accepts the recursive type names without infinite-layout errors.

The semantic cost of Option A: `Functor::map` is the identity (returns `fa` unchanged) because `Of<'a, X>` is independent of `X`; `SendFunctor::send_map` similarly. `WrapDrop::drop` returns `None` (the cell's body program result cannot be materialised without first running acquire to produce a resource). `Extract::extract` is a panicking `unreachable!` stub (substrate-required cascade reaches `Bracket` only on synthetic paths; production interpret routes scoped layers to the bracket dispatcher (step 7's `BracketDispatcher`), not `Extract`). All four impls are documented in-file with explicit reasons; the `#[expect(clippy::unreachable, reason = "...")]` attribute suppresses the lint at the impl site.

Phase 4 step 3.3 plan text inherits the Option A switch for both Bracket Val (3.3.1-3.3.4) and RefBracket Ref (3.3.5-3.3.8); the user-facing `bracket(acquire, body, release)` smart-constructor signatures (step 3.3.3) thread `Sub = NodeBrand<R, S>` per wrapper, paralleling the existing Catch/Local smart-constructor patterns.

### Step 3.2.6: `SendRefLocalBrand` skips `RefFunctor`; the cascade through `ArcRunExplicitBrand` does not require it

Step 3.2.6 lands `RefFunctor` impls for [`BoxRefLocalBrand<BoxBrand, E>`](../../../fp-library/src/types/effects/ref_local.rs) and [`RefLocalBrand<RcBrand, E>`](../../../fp-library/src/types/effects/ref_local.rs) but deliberately omits `RefFunctor` for [`SendRefLocalBrand<ArcBrand, E>`](../../../fp-library/src/types/effects/ref_local.rs). The omission mirrors the [`SendLocalBrand`-no-`RefFunctor` deviation](#step-322-sendlocalbrand-skips-reffunctor-the-cascade-through-arcrunexplicitbrand-does-not-require-it) and the [`SendCatchBrand`-no-`RefFunctor` deviation](#step-312-sendcatchbrand-skips-reffunctor-the-cascade-through-arcrunexplicitbrand-does-not-require-it) for the same structural reasons.

The cascade chain that would require `RefFunctor` on a scoped-row brand is `*RunExplicitBrand: RefFunctor` -> `*FreeExplicitBrand: RefFunctor` -> `NodeBrand<R, S>: RefFunctor` -> `S: RefFunctor`. The brand-level docstring on [`ArcRunExplicitBrand`](../../../fp-library/src/brands/effects.rs) records that the `Ref`/`SendRef`-family hierarchy is not reachable through brand-level delegation because `ArcFreeExplicitBrand` does not implement it. The cascade therefore terminates at `ArcFreeExplicitBrand` without ever requiring `RefFunctor` on the scoped row's brands; `SendRefLocalBrand: !RefFunctor` is benign for the Arc-family substrate.

A hypothetical `SendRefLocalBrand: RefFunctor` impl would also fail structurally for the same reason `SendRefLocalBrand: !Functor` does: `RefFunctor::ref_map`'s closure parameter `func: impl Fn(&A) -> B + 'a` lacks the `Send + Sync` bounds required by [`<ArcBrand as ToDynSendFn>::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) for storing the new closures in a `SendRefLocal::Local`'s `Arc<dyn Fn + Send + Sync>` cells.

Per the B-thunk action representation closed in [B9](resolutions.md#resolved-2026-05-07-phase-4-step-3.2-sub-step-splitting--local-action-layout-cycle-reuse--file-organization-b8--b9--b10-closed), the [`BoxRefLocalBrand`'s `RefFunctor` impl](../../../fp-library/src/types/effects/ref_local.rs) is a stub on both fields (`Box<dyn FnOnce(&E) -> E>` cannot be cloned through a reference and `Box<dyn FnOnce(()) -> A>` cannot be invoked through a reference); the path is structurally unreachable in real programs since `RunExplicitBrand: RefFunctor` is reachable only through synthetic non-Coyoneda first-order rows. The [`RefLocalBrand`'s `RefFunctor` impl](../../../fp-library/src/types/effects/ref_local.rs) is faithful: clones the `Rc<dyn Fn>` modify pointer (preserved unchanged because modify's `&E -> E` signature does not depend on the result type), clones the action thunk's Rc, and post-composes `func` over the action's output via `<RcBrand as ToDynCloneFn>::new`. Two `#[doc(hidden)]` brand-projection helpers ([`ref_local_modify_ref`](../../../fp-library/src/types/effects/ref_local.rs) and [`ref_local_action_thunk_ref`](../../../fp-library/src/types/effects/ref_local.rs)) escape the trait impl's HRTB-bearing scope so the GAT projection normalizes against the concrete `RefLocal` enum, mirroring the [`local_modify_ref` / `local_action_thunk_ref` precedent](../../../fp-library/src/types/effects/local.rs) from step 3.2.2.

### Step 3.2.5: `SendRefLocalBrand` ships only with `SendFunctor` (not `Functor`); mirrors `SendLocalBrand` and `SendCatchBrand` precedents

Step 3.2.5 lands the [`RefLocal`](../../../fp-library/src/types/effects/ref_local.rs) Ref foundational scaffold (three sibling effect types `BoxRefLocal` / `RefLocal` / `SendRefLocal`; three brands `BoxRefLocalBrand` / `RefLocalBrand` / `SendRefLocalBrand`) but deliberately omits `Functor` for [`SendRefLocalBrand<ArcBrand, E>`](../../../fp-library/src/types/effects/ref_local.rs). [scoped.rs's "Per-scoped-effect-brand substrate-required traits" subsection](../../../fp-library/src/types/effects/scoped.rs) claims each scoped-effect brand placed in a `ScopedCoproduct` must implement five traits including `Functor`; the omission is structural.

`Functor::map`'s closure parameter `f: impl Fn(A) -> B + 'a` lacks the `Send + Sync` bounds that [`<ArcBrand as ToDynSendFn>::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) requires for closure storage in the `SendRefLocal::Local`'s action thunk cell. Post-composing `f` into a new `Arc<dyn Fn(()) -> B + Send + Sync>` therefore fails to type-check. Mirrors the Phase 3 [`SendStateBrand` precedent](../../../fp-library/src/types/effects/state.rs), the [`SendCatchBrand` precedent](#step-311-sendcatchbrand-ships-only-with-sendfunctor-not-functor-scopedrss-all-five-required-claim-is-over-broad), and the [`SendLocalBrand` precedent](#step-321-sendlocalbrand-ships-only-with-sendfunctor-not-functor-mirrors-sendcatchbrand-precedent) below.

The same `RefFunctor`-omission rationale will apply when step 3.2.6 lands the `RefFunctor` impls for the Box and Rc flavours of `RefLocal`; a separate deviation entry will be added at that step.

### Step 3.2.5: `ToDynFnOnce` extended with `ref_new`; closes the closure-trait matrix asymmetry surfaced by `BoxRefLocal::modify`'s `FnOnce(&E) -> E` storage

Step 3.2.5 extends [`ToDynFnOnce`](../../../fp-library/src/classes/to_dyn_fn_once.rs) with `fn ref_new<'a, A: 'a, B: 'a>(f: impl 'a + FnOnce(&A) -> B) -> Self::Of<'a, dyn 'a + FnOnce(&A) -> B>` plus a free-function shim `to_ref_dyn_fn_once`, and adds the parallel `ref_new` impl on `BoxBrand` at [`box_ptr.rs`](../../../fp-library/src/types/box_ptr.rs). The trait extension is bundled into the foundational-scaffold commit because `BoxRefLocal::modify` cannot be constructed without it.

The other three closure traits in the abstraction layer ([`ToDynFn`](../../../fp-library/src/classes/to_dyn_fn.rs), [`ToDynCloneFn`](../../../fp-library/src/classes/to_dyn_clone_fn.rs), [`ToDynSendFn`](../../../fp-library/src/classes/to_dyn_send_fn.rs)) all shipped both `new` and `ref_new` from their original landing in Phase 3.5; `ToDynFnOnce` shipped only `new` because Phase 3.5's effects (State, Reader, Choose) had no by-reference `FnOnce` storage requirement. The matrix asymmetry was invisible until step 3.2.5 introduced `BoxRefLocal::modify: Box<dyn FnOnce(&E) -> E>`. The extension is mechanical translation of the existing `ref_new` pattern; ~30 lines including doctests.

The bundling-into-3.2.5 decision (rather than a separate prep commit) follows the Phase 4 step 3.1.3 precedent of shipping small substrate fixes alongside the step they unblock; full options analysis closed via [B11](resolutions.md#resolved-2026-05-07-phase-4-step-3.2.5-todynfnonceref_new-matrix-gap--variant-naming-b11--b12-closed).

### Step 3.2.5: `BoxRefLocal` / `RefLocal` / `SendRefLocal` variants uniformly named `Local` (mirroring Val flavour); not `RefLocal`

Step 3.2.5 names the single variant on each of the three sibling Ref-flavoured enums simply `Local` rather than `RefLocal`. Pattern-match: `BoxRefLocal::Local { .. }` / `RefLocal::Local { .. }` / `SendRefLocal::Local { .. }`. The type tag (`BoxRefLocal` vs `BoxLocal`) carries the Val/Ref flavour info already; smart-constructor dispatch at the call site (`local(modify, action)` per [decisions.md line 549](decisions.md)) names the operation, not the variant. Closes [B12](resolutions.md#resolved-2026-05-07-phase-4-step-3.2.5-todynfnonceref_new-matrix-gap--variant-naming-b11--b12-closed).

The alternative (`RefLocal::RefLocal { .. }`) was rejected on repetitive-pattern grounds; the same uniformly-`Local` choice will apply to `Bracket` / `RefBracket` (step 3.3) by precedent.

### Step 3.2.2: `SendLocalBrand` skips `RefFunctor`; the cascade through `ArcRunExplicitBrand` does not require it

Step 3.2.2 lands `RefFunctor` impls for [`BoxLocalBrand<BoxBrand, E>`](../../../fp-library/src/types/effects/local.rs) and [`LocalBrand<RcBrand, E>`](../../../fp-library/src/types/effects/local.rs) but deliberately omits `RefFunctor` for [`SendLocalBrand<ArcBrand, E>`](../../../fp-library/src/types/effects/local.rs). The omission mirrors the [`SendCatchBrand`-no-`RefFunctor` deviation](#step-312-sendcatchbrand-skips-reffunctor-the-cascade-through-arcrunexplicitbrand-does-not-require-it) below for the same structural reasons.

The cascade chain that would require `RefFunctor` on a scoped-row brand is `*RunExplicitBrand: RefFunctor` -> `*FreeExplicitBrand: RefFunctor` -> `NodeBrand<R, S>: RefFunctor` -> `S: RefFunctor`. The brand-level docstring on [`ArcRunExplicitBrand`](../../../fp-library/src/brands/effects.rs) records that the `Ref`/`SendRef`-family hierarchy is not reachable through brand-level delegation because `ArcFreeExplicitBrand` does not implement it. The cascade therefore terminates at `ArcFreeExplicitBrand` without ever requiring `RefFunctor` on the scoped row's brands; `SendLocalBrand: !RefFunctor` is benign for the Arc-family substrate.

A hypothetical `SendLocalBrand: RefFunctor` impl would also fail structurally for the same reason `SendLocalBrand: !Functor` does (logged at [step 3.2.1 below](#step-321-sendlocalbrand-ships-only-with-sendfunctor-not-functor-mirrors-sendcatchbrand-precedent)): `RefFunctor::ref_map`'s closure parameter `func: impl Fn(&A) -> B + 'a` lacks the `Send + Sync` bounds required by [`<ArcBrand as ToDynSendFn>::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) for storing the new closures in a `SendLocal::Local`'s `Arc<dyn Fn + Send + Sync>` cells.

Per the B-thunk action representation closed in [B9](resolutions.md#resolved-2026-05-07-phase-4-step-3.2-sub-step-splitting--local-action-layout-cycle-reuse--file-organization-b8--b9--b10-closed), the [`BoxLocalBrand`'s `RefFunctor` impl](../../../fp-library/src/types/effects/local.rs) is a stub on both fields (`Box<dyn FnOnce(E) -> E>` cannot be cloned through a reference and `Box<dyn FnOnce(()) -> A>` cannot be invoked through a reference); the path is structurally unreachable in real programs since `RunExplicitBrand: RefFunctor` is reachable only through synthetic non-Coyoneda first-order rows. The [`LocalBrand`'s `RefFunctor` impl](../../../fp-library/src/types/effects/local.rs) is faithful: clones the `Rc<dyn Fn>` modify pointer (preserved unchanged because modify's `E -> E` signature does not depend on the result type), clones the action thunk's Rc, and post-composes `func` over the action's output via `<RcBrand as ToDynCloneFn>::new`. Two `#[doc(hidden)]` brand-projection helpers ([`local_modify_ref`](../../../fp-library/src/types/effects/local.rs) and [`local_action_thunk_ref`](../../../fp-library/src/types/effects/local.rs)) escape the trait impl's HRTB-bearing scope so the GAT projection normalizes against the concrete `Local` enum, mirroring the [`arc_run::unwrap_first` precedent](../../../fp-library/src/types/effects/arc_run.rs).

### Step 3.2.1: `SendLocalBrand` ships only with `SendFunctor` (not `Functor`); mirrors `SendCatchBrand` precedent

Step 3.2.1 lands the [`Local`](../../../fp-library/src/types/effects/local.rs) Val foundational scaffold (three sibling effect types `BoxLocal` / `Local` / `SendLocal`; three brands `BoxLocalBrand` / `LocalBrand` / `SendLocalBrand`) but deliberately omits `Functor` for [`SendLocalBrand<ArcBrand, E>`](../../../fp-library/src/types/effects/local.rs). [scoped.rs's "Per-scoped-effect-brand substrate-required traits" subsection](../../../fp-library/src/types/effects/scoped.rs) claims each scoped-effect brand placed in a `ScopedCoproduct` must implement five traits including `Functor`; the omission is structural, not an oversight.

`Functor::map`'s closure parameter `f: impl Fn(A) -> B + 'a` lacks the `Send + Sync` bounds that [`<ArcBrand as ToDynSendFn>::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) requires for closure storage in the `SendLocal::Local`'s action thunk cell. Post-composing `f` into a new `Arc<dyn Fn(()) -> B + Send + Sync>` therefore fails to type-check.

Mirrors the Phase 3 [`SendStateBrand` precedent](../../../fp-library/src/types/effects/state.rs) and the [`SendCatchBrand` precedent](#step-311-sendcatchbrand-ships-only-with-sendfunctor-not-functor-scopedrss-all-five-required-claim-is-over-broad) below. The Arc-family substrate's program-traversal machinery [`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs) routes through `<S as SendFunctor>::send_map` (whose closure parameter carries the required `Send + Sync` bounds), not through `<S as Functor>::map`, so the missing `Functor` impl is unreachable for Arc-family programs.

The same `RefFunctor`-omission rationale will apply when step 3.2.2 lands the `RefFunctor` impls for the Box and Rc flavours of `Local` (mirroring the [`SendCatchBrand`-no-`RefFunctor` deviation](#step-312-sendcatchbrand-skips-reffunctor-the-cascade-through-arcrunexplicitbrand-does-not-require-it)); a separate deviation entry will be added at that step.

### Step 3.1.2: `SendCatchBrand` skips `RefFunctor`; the cascade through `ArcRunExplicitBrand` does not require it

Step 3.1.2 lands `RefFunctor` impls for [`BoxCatchBrand<BoxBrand, E>`](../../../fp-library/src/types/effects/catch.rs) and [`CatchBrand<RcBrand, E>`](../../../fp-library/src/types/effects/catch.rs) but deliberately omits `RefFunctor` for [`SendCatchBrand<ArcBrand, E>`](../../../fp-library/src/types/effects/catch.rs). [scoped.rs's "Per-scoped-effect-brand substrate-required traits" subsection](../../../fp-library/src/types/effects/scoped.rs) claims each scoped-effect brand must implement five traits including `RefFunctor`; the omission for `SendCatchBrand` is structurally justified.

The cascade chain that would require `RefFunctor` on a scoped-row brand is `*RunExplicitBrand: RefFunctor` -> `*FreeExplicitBrand: RefFunctor` -> `NodeBrand<R, S>: RefFunctor` -> `S: RefFunctor`. The brand-level docstring on [`ArcRunExplicitBrand`](../../../fp-library/src/brands/effects.rs) records that the `Ref`/`SendRef`-family hierarchy is not reachable through brand-level delegation because `ArcFreeExplicitBrand` does not implement it (auto-derive of `Send + Sync` on `ArcFreeExplicit` requires a per-`A` HRTB on the [`Kind`](../../../fp-library/src/kinds.rs) projection that stable Rust's trait method signatures cannot carry). The cascade therefore terminates at `ArcFreeExplicitBrand` without ever requiring `RefFunctor` on the scoped row's brands; `SendCatchBrand: !RefFunctor` is benign for the Arc-family substrate.

A hypothetical `SendCatchBrand: RefFunctor` impl would also fail structurally for the same reason `SendCatchBrand: !Functor` does (logged at step 3.1.1 above): `RefFunctor::ref_map`'s closure parameter `func: impl Fn(&A) -> B + 'a` lacks the `Send + Sync` bounds required by [`<ArcBrand as ToDynSendFn>::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) for storing the new handler in a `SendCatch::Catch`'s `Arc<dyn Fn + Send + Sync>` cell.

The scoped.rs comment will be refined in step 3.1.4 (or a 3.1 follow-up doc commit) to clarify that Send-flavoured brands skip both `Functor` and `RefFunctor` per the `SendCatchBrand` precedent, and that the cascade through `*RunExplicitBrand: RefFunctor` reaches scoped-row brands only when `S` is non-Arc-flavoured.

### Step 3.1.1: `SendCatchBrand` ships only with `SendFunctor` (not `Functor`); scoped.rs's "all five required" claim is over-broad

[scoped.rs's "Per-scoped-effect-brand substrate-required traits" subsection](../../../fp-library/src/types/effects/scoped.rs) claims each scoped-effect brand placed in a `ScopedCoproduct` must implement five traits: `Functor`, `SendFunctor`, `WrapDrop`, `RefFunctor`, `Extract`. The shipped step 3.1.1 foundational scaffold at [`catch.rs`](../../../fp-library/src/types/effects/catch.rs) implements four of the five for [`SendCatchBrand<ArcBrand, E>`](../../../fp-library/src/brands/effects.rs) (`SendFunctor`, `WrapDrop`, `Extract`; plus the deferred `RefFunctor` per [B5](plan.md#b5-reffunctor-gat-normalization-for-scoped-effect-closure-cell-brands)) but deliberately omits `Functor`. The omission is structural, not an oversight: `Functor::map`'s closure parameter `f: impl Fn(A) -> B + 'a` lacks the `Send + Sync` bounds that [`<ArcBrand as ToDynSendFn>::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) requires for closure storage in the `SendCatch` handler cell. Post-composing `f` into a new `Arc<dyn Fn(E) -> B + Send + Sync>` therefore fails to type-check.

This mirrors Phase 3's [`SendStateBrand` precedent](../../../fp-library/src/types/effects/state.rs): `SendStateBrand<ArcBrand, S>` ships only with `SendFunctor`, not `Functor`, for the same structural reason. The Arc-family substrate's program-traversal machinery [`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs) routes through `<S as SendFunctor>::send_map` (whose closure parameter carries the required `Send + Sync` bounds), not through `<S as Functor>::map`, so the missing `Functor` impl is unreachable for Arc-family programs.

The scoped.rs comment will be refined in step 3.1.4 (or a 3.1 follow-up doc commit) to clarify that Send-flavoured brands skip `Functor` per the `SendStateBrand` precedent and that `NodeBrand<R, S>: Functor` is reachable only when `S` excludes Send-flavoured brands.

### Step 2.1: `RcRun::interpose` carries an extra `EmbedIndices` type parameter beyond plan.md's sketched `<EBrand, Idx>` signature

[plan.md Phase 4 step 2](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) sketches the substrate primitive as `interpose<EBrand, Idx>(...)`. The shipped signature at [`RcRun::interpose`](../../../fp-library/src/types/effects/rc_run.rs) carries four type parameters: `EBrand, Idx, RMinusE, EmbedIndices`. The two extra parameters are structural necessities of the Rust type system, not design changes:

- **`RMinusE`** matches the same role it plays on the existing [`RcRun::interpret_with`](../../../fp-library/src/types/effects/rc_run.rs#L913) primitive: the row brand for "the original row with `EBrand` removed at position `Idx`". `Member<EBrand, Idx>::project` on the row's layer returns `Result<EBrand_projection, Self::Remainder>`; the `Remainder`'s row-brand identity has to be named at the type level so the unmatched-arm Functor map (`<RMinusE as Functor>::map`) and the embed-back step (`CoproductEmbedder<R, EmbedIndices>`) can refer to it. Plan.md's sketch elided this parameter for brevity; the `interpret_with` precedent shows this is standard.
- **`EmbedIndices`** is new to interpose. `interpret_with` narrows the row to `RMinusE` and rebuilds the program in the narrowed row, so no embed step is needed. `interpose` keeps the row at `R`, so the unmatched-arm rebuilt layer (which is `RMinusE`-typed after `Functor::map`) must be embedded back into the `R`-typed shape via [`CoproductEmbedder<<R>::Of<...>, EmbedIndices>`](../../../fp-library/src/types/effects/coproduct.rs). The `EmbedIndices` is an HList of [`CoprodInjector`](../../../fp-library/src/types/effects/coproduct.rs) position witnesses (one per non-`EBrand` variant in `R`); frunk_core resolves it through type inference at the call site, but the parameter must be present on the function so the compiler has a name to bind the inference result to. There is no obvious way to derive `EmbedIndices` from `Idx` alone because `Idx` only locates `EBrand` in `R`; the embedding witness specifies how each of the OTHER variants in `R` maps back, which is independent type-level information.

The user-facing turbofish convention `prog.interpose::<EBrand, _, RMinusE, _>(...)` lets `Idx` and `EmbedIndices` both stay as `_` for type inference; users only spell `EBrand` and `RMinusE` explicitly. This matches `interpret_with`'s ergonomics for `Idx`.

If a future deviation finds a way to derive `EmbedIndices` automatically from `Idx` (e.g., via a custom trait that exposes it as an associated type on `Member<E, Idx>`), the parameter could be hidden. For now, the four-parameter form is the most pragmatic shape that types cleanly with frunk_core's existing trait family.

The shipped signature applies only to `RcRun` so far (sub-step 2.1); the remaining five wrapper sub-steps (2.2-2.6) are expected to mirror the same shape.

## Phase 1: Free family

### Step 1: `FreeExplicit` promotion

- **Removed `OptionBrand`-using POC tests.** Adding the
  `F: Extract + Functor + 'a` bound to `FreeExplicit` (required
  by the iterative `Drop` impl per
  [decisions.md](decisions.md) section 4.4) means
  `OptionBrand` can no longer back a `FreeExplicit`, since `None`
  has no value to surrender and `OptionBrand` therefore cannot
  lawfully implement `Extract`. The POC's `q5_two_effect_run`
  short-circuit test and the `evaluate_option` helper were dropped;
  the same Run-shaped semantics are reachable via handler
  interpretation in Phase 3+. This is exactly the caveat the
  decision predicts ("this forces every effect functor used with
  `FreeExplicit` to implement `Extract`"), but the plan step text
  said to "replace the local definition with an import" without
  explicitly listing test removals, so it is recorded here.
- **Introduced `FreeExplicitView` enum.** The POC's `FreeExplicit`
  was a two-variant enum directly. The production type wraps the
  variants in `view: Option<FreeExplicitView>` so the custom
  `Drop` impl can move the view out via `Option::take` without
  producing a sentinel `A` value. `FreeExplicitView` is `pub` and
  re-exported alongside `FreeExplicit` to keep the variants
  visible for users who want to pattern-match. No external test
  or bench needed to change shape; the POC tests only used
  `pure`, `wrap`, `bind`, and `evaluate` (no direct match on the
  variants).

### Step 2: `RcFree`

- **`RcFree` uses `Rc<dyn Any>` (not `Box<dyn Any>`) for the
  type-erased value cell.** Decision 4.4's table summarises
  `RcFree`'s erasure as "`Box<dyn Any>` + CatList" while also
  committing to "Cloneable: Yes, O(1)". `Box<dyn Any>` is not
  `Clone`, so the literal table reading conflicts with the Clone
  commitment. The minimal resolution is to swap the Box-erased
  cell for an `Rc<dyn Any>`, which keeps the `dyn Any` erasure
  shape but lets the inner state participate in Clone. Recovering
  an owned `A` from the cell uses `Rc::try_unwrap` and falls back
  to `(*shared).clone()` when the cell is shared, which constrains
  the public methods that perform the final downcast (`to_view`,
  `resume`, `evaluate`, `lower_ref`, `peel_ref`, `hoist_free`) to
  require `A: Clone`. This matches the multi-shot semantics: a
  handler that wants to evaluate the same program more than once
  needs the result type to be reproducible.
- **`RcFree<F, A>` is `Rc<RcFreeInner<F, A>>` (outer `Rc`
  wrapping).** Step 2's text says "follow the `Free` template"
  without specifying outer-Rc-wrapping, but the unconditional
  O(1) Clone commitment plus the `Suspend` arm holding
  `F::Of<RcFree<F, RcTypeErasedValue>>` produce a recursive Clone
  bound that only resolves cleanly when `RcFree: Clone` is
  unconditional. Outer-Rc-wrapping (the
  [`RcCoyoneda`](../../../fp-library/src/types/rc_coyoneda.rs)
  pattern) makes Clone trivially `Rc::clone(&self.inner)`. State-
  extending operations (`bind`, `map`, `wrap`, `lift_f`,
  `cast_phantom`) use `Rc::try_unwrap` to move out when uniquely
  owned and clone the inner state otherwise.
- **`RcContinuation` is a newtype, not the bare
  `<RcFnBrand as CloneFn>::Of` projection.** Step 2's text says
  "expressed via `FnBrand<RcBrand>`". Using the macro-mediated GAT
  projection directly as a type alias does not parse (the type
  parameter `F` does not surface through the `Apply!` expansion).
  The production type uses a thin newtype
  `RcContinuation<F>(Rc<dyn Fn(...)>)` with the same in-memory
  shape as `<RcFnBrand as CloneFn>::Of`, and constructs values via
  `<RcFnBrand as LiftFn>::new(...)` so the library's unified
  function-pointer abstraction is still on the construction path.
  The newtype's `Clone` impl bumps the underlying `Rc`'s refcount.

### Step 3: `ArcFree`

- **`ArcFree` carries the same trio of deviations as `RcFree`**
  (the type-erased value uses `Arc<dyn Any + Send + Sync>` for
  `Clone`/`Send`/`Sync` participation, the substrate is wrapped
  in outer `Arc<Inner>`, and `ArcContinuation<F>` is a newtype
  wrapping `Arc<dyn Fn(...) + Send + Sync>` constructed via
  `<ArcFnBrand as SendLiftFn>::new`). All three deviations carry
  forward unchanged from step 2's analysis with `Rc` substituted
  for `Arc`.
- **Associated-type-bound trick is propagated to every struct
  and impl.** Decision 4.4 names the trick
  (`Kind<Of<'a, A>: Send + Sync>`) but does not prescribe scope.
  In production, `Send + Sync` auto-derivation on `ArcFreeInner`
  via the `F::Of<...>` field requires the bound at the struct
  definition. To keep all uses of the inner data type-checkable,
  the same
  `Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>`
  bound is added to `ArcContinuation<F>`, `ArcFreeView<F>`,
  `ArcFreeStep<F, A>`, `ArcFreeInner<F, A>`, `ArcFree<F, A>`, and
  every `impl` block that mentions any of them. This is verbose
  but mechanical; `ArcCoyoneda`'s template uses the same trick at
  fewer sites because its trait-object internal representation
  hides the `F::Of` from auto-derivation.

### Step 4: `RcFreeExplicit`

- **`RcFreeExplicitBrand<F>` struct and `impl_kind!` registration
  land in step 4, not step 7.** Step 4's text says
  "Brand-compatible: this is the multi-shot variant that carries
  Brand dispatch in Phase 1 step 7", which on a strict reading
  could mean step 7 introduces both the brand struct and the
  trait impls. Step 1 set the precedent of pairing the brand
  struct + `impl_kind!` with the type definition
  (`FreeExplicitBrand<F>` was added in step 1 even though its
  `Functor`/`Pointed`/`Semimonad`/`Monad` impls are scheduled for
  step 7). Step 4 follows the same precedent: the brand and
  `Kind` registration ship now, the trait hierarchies ship in
  step 7. This keeps step 7's scope to "trait impls" only.
- **`Wrap` variant holds `RcFreeExplicit` directly, not
  `Box<RcFreeExplicit>`.** `FreeExplicit`'s `Wrap` variant uses
  `F::Of<'a, Box<FreeExplicit<'a, F, A>>>` because the outer struct
  is unboxed and a recursive type needs indirection to be sized.
  `RcFreeExplicit`'s outer wrapper is `Rc<RcFreeExplicitInner>`,
  which already provides the indirection, so the `Wrap` arm holds
  `F::Of<'a, RcFreeExplicit<'a, F, A>>` directly. Skipping the
  `Box` layer avoids one extra heap hop per node and keeps the
  `F::extract` call site free of a `*extracted` deref.
- **`to_view(self)` is exposed as a public consuming method.**
  Step 4's text only names `lower_ref(&self)` and
  `peel_ref(&self)`. `peel_ref` is naturally implemented as
  `self.clone().to_view()`, which requires a consuming `to_view`
  on the underlying type (the `view` field is private). Exposing
  `to_view` publicly keeps the implementation symmetric with
  `RcFree::to_view` and avoids burying the consuming version as
  a private helper. `FreeExplicit` does not have `to_view`
  because it does not have `peel_ref` either.
- **Inherent-method API is intentionally narrower than
  `RcFree`'s.** `RcFree` exposes `pure`, `wrap`, `lift_f`,
  `bind`, `map`, `to_view`, `resume`, `evaluate`, `hoist_free`,
  plus `lower_ref` / `peel_ref`. `RcFreeExplicit` exposes only
  `pure`, `wrap`, `bind`, `evaluate`, `to_view`, `lower_ref`,
  `peel_ref`. The omitted methods (`lift_f`, `map`, `resume`,
  `hoist_free`) belong on the Brand-dispatched API surface that
  step 7 builds via `Functor` / `Pointed` / `Semimonad` /
  `Monad`, so adding them as inherent methods here would
  duplicate that surface. `RcFree` has them inherently because
  the Erased family has no Brand dispatch at all (decisions
  section 4.4); the Explicit family routes the same operations
  through the trait hierarchy.

### Step 5: `ArcFreeExplicit`

- **`Kind<Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync>`
  associated-type-bound trick is dropped from the struct.**
  Step 5's text says "Same `Kind<Of<'a, A>: Send + Sync>`
  associated-type-bound trick as `ArcFree`". `ArcFree` works
  because `A` is fixed (`ArcTypeErasedValue`) and the bound's
  GAT instantiation is concrete
  (`Of<'static, ArcFree<F, ArcTypeErasedValue>>`). `ArcFreeExplicit`
  has generic `A`, so the analogous bound is
  `Kind<Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync>`
  parameterised by both `'a` and `A`. The `impl_kind!`
  registration for `ArcFreeExplicitBrand<F>` requires the bound
  for any `'a` and `A`, which is
  `for<'a, A> Kind<Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync>` --
  an HRTB-over-types that stable Rust does not support
  ([fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)
  section "No Rank-N Types"). With the bound on the struct, the
  `impl_kind!` cannot prove `ArcFreeExplicit<'a, F, A>` is
  well-formed for arbitrary `'a` / `A` and fails to compile. With
  the bound off, `impl_kind!` compiles and `Send + Sync`
  auto-derive still works for concrete `F` (e.g.,
  `IdentityBrand`) via type-walk resolution. The
  `is_send_and_sync` test passes for
  `ArcFreeExplicit<'_, IdentityBrand, i32>`. Step 7's brand
  impls will need to add concrete `Send + Sync` bounds at impl
  sites where they require thread-safety guarantees over generic
  `F`, mirroring how the `ArcCoyoneda` precedent threads bounds
  through individual impls rather than the struct.
- **`bind` requires `A: Clone + Send + Sync`** -- `Send + Sync`
  from the closure-storage shape, `Clone` from the
  shared-inner-state recovery fallback. `RcFreeExplicit::bind`
  required `A: Clone` for the same shared-inner-state reason;
  `ArcFreeExplicit::bind` adds `Send + Sync` because the
  `Arc<dyn Fn + Send + Sync>` continuation cell forces all
  values flowing through it to be thread-safe. This matches
  `ArcFree::bind`'s bound profile.

### Step 6: `SendFunctor` trait family

- **Nine new trait files, not the four named in the step
  text.** The plan listed only four files
  (`send_functor.rs`, `send_pointed.rs`, `send_semimonad.rs`,
  `send_monad.rs`), but a faithful by-value parallel of the
  existing `send_ref_*` family needs the full applicative-family
  scaffolding so `SendMonad: SendApplicative + SendSemimonad`
  can mirror
  [`Monad`](../../../fp-library/src/classes/monad.rs)'s shape.
  Step 6 ships nine files: the four named, plus
  `send_lift.rs` (`SendLift::send_lift2`),
  `send_semiapplicative.rs` (`SendSemiapplicative::send_apply`
  using `SendCloneFn`), `send_apply_first.rs` and
  `send_apply_second.rs` (blanket-implemented over `SendLift`),
  and `send_applicative.rs` (`SendApplicative` blanket over the
  `SendPointed + SendSemiapplicative + SendApplyFirst + SendApplySecond`
  combination). With these, `SendMonad`'s supertrait chain is
  identical to the by-value `Monad`'s, just with `Send + Sync`
  bounds layered on. Step text underspecified the file count;
  the trait family wouldn't have been usable for typeclass-generic
  thread-safe code without the full chain.
- **`OptionBrand` implements the full applicative family of
  `Send*` traits, not just the three originally scoped.** The
  step text named `ArcCoyonedaBrand` as the bonus integration.
  `ArcCoyonedaBrand` cannot implement `SendPointed`,
  `SendSemimonad`, `SendLift`, or `SendSemiapplicative` because
  all four go through
  [`ArcCoyoneda::lift`](../../../fp-library/src/types/arc_coyoneda.rs)
  which requires `F::Of<'a, A>: Clone + Send + Sync`, a per-`A`
  bound (same blocker as the by-value `Pointed` / `Semimonad`
  cases the
  [limitations doc](../../../fp-library/docs/limitations-and-workarounds.md)
  classifies for `RcCoyoneda` / `ArcCoyoneda`). To give every
  new trait method a working executable doctest (which
  `#[document_examples]` requires), `OptionBrand` was given
  trivial impls of `SendPointed` / `SendFunctor` / `SendLift` /
  `SendSemiapplicative` / `SendSemimonad`. `Option<A>` is
  `Send + Sync` whenever `A` is, so all five impls are
  mechanical and align with `OptionBrand`'s existing `Pointed` /
  `Functor` / `Lift` / `Semiapplicative` / `Semimonad` impls.
  `SendApplicative` / `SendApplyFirst` / `SendApplySecond` /
  `SendMonad` follow via the blanket impls.
- **`ArcCoyonedaBrand` implements `SendFunctor` only, not the
  full `SendFunctor` / `SendPointed` / `SendSemimonad` /
  `SendMonad` quartet the plan named.** The step text said
  "the full hierarchy lands" for `ArcCoyonedaBrand` because
  "ArcCoyoneda's by-value path has no Clone bound". This was
  over-optimistic: `ArcCoyoneda::map` has no Clone bound, but
  `ArcCoyoneda::pure` and `ArcCoyoneda::bind` both go through
  `lift` which carries a per-`A`
  `F::Of<'a, A>: Clone + Send + Sync` bound. The trait
  signatures cannot express that bound (no HRTB-over-types).
  The module-level docs and the brand-impl block comment in
  [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)
  are updated to record this. `ArcCoyonedaBrand` joins
  `RcCoyonedaBrand`'s precedent of partial brand-level coverage
  with the rest of the operations available as inherent methods
  on the concrete `ArcCoyoneda` type.
- **`#[document_examples]` macro requires real Rust code
  blocks.** Removing the macro from a method silences its
  function but emits a deprecation warning that `-D warnings`
  (in `just clippy`) escalates to error. The macro also rejects
  ` ```ignore` and other non-Rust-marker fences. The chosen
  resolution is to give every newly-defined trait method a
  working executable doctest, which is what motivated adding
  the `OptionBrand` impls (above). For traits whose only
  motivating use case is `ArcFreeExplicitBrand` in step 7, the
  `OptionBrand` examples serve as canonical shape demonstrations
  until step 7's impls land and provide the substantive use
  case.

### Step 8: per-variant Criterion benches

- **`Free` bench uses `ThunkBrand`, not `IdentityBrand`.** The
  other five Free variants use `IdentityBrand` (matching the
  existing
  [free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)
  POC bench). `Free` cannot: `Free<F, A>`'s `Wrap` arm holds
  `F::Of<Free<F, TypeErasedValue>>` where
  `TypeErasedValue = Box<dyn Any>`, and `Identity<T>` is `T`
  with no indirection, so the layout recursion has no
  termination; `Free<IdentityBrand, A>` fails to compile with
  `error[E0391]: cycle detected when computing layout`. The
  Rc/Arc Erased family escapes via the outer `Rc<Inner>` /
  `Arc<Inner>` wrapper; the Explicit family escapes via either
  `Box<...>` (in `FreeExplicit`'s `Wrap` arm) or the outer
  `Rc<Inner>` / `Arc<Inner>` wrapper. `ThunkBrand` is the brand
  the existing `Free` unit tests already use; `Thunk<A>` holds a
  boxed closure, which provides the indirection. Step 8's text
  said "replicates the full set across all six variants"
  without specifying brand choice, so the deviation is recorded
  here.

## Phase 1 follow-up: `WrapDrop` migration

### Commit 1: `WrapDrop` trait and per-variant struct/Drop swap

- **Did not split impl blocks for `Rc` / `Arc` variants; used
  per-method `where F: Extract` instead.** `free.rs` already
  organised its inherent methods into three impl blocks
  (construction / functor-dependent / `evaluate`-only); after
  migration the third block uses
  `F: Extract + WrapDrop + Functor + 'static`, and the other
  two use `F: WrapDrop + Functor + 'static`. The other five
  variants (`RcFree`, `ArcFree`, `FreeExplicit`,
  `RcFreeExplicit`, `ArcFreeExplicit`) bundle their methods
  into a single inherent impl block. To avoid a structural
  rewrite, `evaluate` and `lower_ref` (the only methods that
  call `F::extract`) get `where F: Extract` as a per-method
  bound, and the impl block stays at
  `F: WrapDrop + Functor + 'a` (or `'static`). Per-method
  bounds are valid Rust, and the alternative (splitting the
  impl block) would touch ~100 unrelated lines of impl block
  delimiters and document attributes per variant. The plan's
  "Methods that call `F::extract` semantically (`evaluate`,
  `resume`, etc.) keep `where F: Extract` on their impl
  blocks." is satisfied modulo the choice of where the `where`
  clause lives (impl block vs method).
- **Added `WrapDrop` to the `Free<F, A>` `evaluate` impl block's
  bound list, not just `Extract`.** The plan says the
  `evaluate` impl block keeps `where F: Extract`. After
  migration, however, `evaluate`'s body calls `current.to_view()`,
  which now lives in a `WrapDrop`-bounded impl block. For the
  `to_view` method to be in scope inside `evaluate`, the
  `evaluate` impl block must also satisfy
  `F: WrapDrop + Functor + 'static`. The bound is therefore
  `F: Extract + WrapDrop + Functor + 'static` for that block.
  `Extract` and `WrapDrop` remain separate traits with no
  supertrait relationship between them, mirroring the
  resolution's design intent.
- **Did not introduce a `wrap_drop` free function.** The
  [`Extract`](../../../fp-library/src/classes/extract.rs)
  module pairs the trait with a free function `extract<F, A>`
  for ergonomic call sites. `WrapDrop` is internal-facing
  (the only consumer is the `Free` family's `Drop`), and call
  sites already use the fully-qualified
  `<F as WrapDrop>::drop::<X>(fa)` syntax (with explicit `X`
  because Rust cannot infer it from `Self::Of<'a, X>`). A
  free function would just shadow the standard
  `std::ops::Drop::drop` name without adding ergonomics, so
  this commit ships the trait alone.
- **Drop-body `if-let` chains use `&&` (Rust 2024
  let-chains).** Clippy's `collapsible_if` lint (denied via
  `-D warnings`) rejected the nested
  `if let Some(extracted) = ... { if let Ok(mut owned) = ... { ... } }`
  pattern in `RcFree`'s and `ArcFree`'s `Drop` bodies. The
  collapsed form
  `if let Some(extracted) = ... && let Ok(mut owned) = ... { ... }`
  is the lint-suggested rewrite and uses stable let-chains
  (Rust 1.94+). Inner `if let`s on `owned.view.take()` are
  not collapsed because the second branch (the
  `inner_conts.uncons()` drain) must run regardless of
  whether the inner view exists.

### Commit 2: `Functor` -> `Kind` relaxation on the struct

- **Asymmetric per-variant relaxation: Erased family relaxes the
  inherent impl block bound; Explicit family does not.** The
  plan text says to add `where F: Functor` to "impl blocks that
  call `F::map`" with the example list `wrap`, `lift_f`,
  `to_view`, and methods that go through them transitively
  (`evaluate`, `resume`, `fold_free`). It explicitly notes that
  `pure`, `bind`, `map` (the inherent method) do not need
  `Functor`. This is true for the Erased family (`Free`,
  `RcFree`, `ArcFree`), where `bind` just snocs the new
  continuation onto a `CatList` without inspecting the suspended
  layer.
  For the Explicit family (`FreeExplicit`,
  `RcFreeExplicit`, `ArcFreeExplicit`), the inherent `bind`
  walks the spine through `bind_boxed`, which calls `F::map`
  recursively at each `Wrap` node (the recursive structural
  shape requires this; there is no `CatList` to defer the
  walk into). So `bind` (and via it `map`, `wrap`, `lift_f`)
  all transitively need `F: Functor`. The Explicit family
  therefore keeps its inherent impl block at
  `F: WrapDrop + Functor + 'a`; only the data-type / `Drop`
  declarations relax. This does not affect Run usability:
  `NodeBrand<R, S>` is expected to impl `Functor` for the same
  reason `CoproductBrand<H, T>` does (Phase 2 step 2), so the
  Explicit Run variants will still get full method coverage.
- **Erased family uses per-method `where F: Functor`, not
  separate impl blocks.** The plan suggests "impl blocks that
  call `F::map`". `free.rs` already organised methods into
  three impl blocks (construction / functor-dependent /
  evaluate), so the construction block stays at
  `F: WrapDrop + 'static` and the functor-dependent block is at
  `F: WrapDrop + Functor + 'static`. For `rc_free.rs` and
  `arc_free.rs`, all methods bundle into a single inherent
  impl block; rather than splitting it into two, the impl
  block bound stays at `F: WrapDrop + 'static` and methods
  that need `Functor` (`wrap`, `lift_f`, `to_view`, `resume`,
  `peel_ref`, `evaluate`, `lower_ref`, `hoist_free`) get
  `where F: Functor` per-method. Same rationale as commit 1's
  per-method `where F: Extract`: the alternative (splitting
  blocks) would touch ~100 unrelated lines per variant.
- **Brand-level impls keep `F: Functor` even where they could
  be relaxed.** On `FreeExplicitBrand<F>` and the
  Rc/Arc-Explicit brands, the `Pointed` impl could in
  principle drop `F: Functor` (its only call is to
  `FreeExplicit::pure(a)`, which doesn't need `Functor` after
  the relaxation). But the `Functor` and `Semimonad` impls
  call `fa.bind(...)` which needs `F: Functor` (per the
  asymmetric Erased/Explicit point above), and the `RefFunctor`
  / `RefSemimonad` impls call helpers that use `F::ref_map`
  and `FreeExplicit::wrap`. Relaxing only `Pointed` and
  `RefPointed` while leaving the other four at
  `F: Functor + ...` was not done; consistency was preferred,
  and the resulting bound parity matches the plan's
  "impl blocks that call F::map" guidance applied
  conservatively at the brand-impl level.

## Phase 2: Run substrate

### Step 1: `frunk_core` dependency + Coproduct adapter

- **Actual frunk_core 0.4 trait names are `CoprodInjector` /
  `CoprodUninjector` / `CoproductSubsetter` /
  `CoproductEmbedder`, not `Plucker` / `Sculptor` /
  `Embedder`.** Step 1's text and Implementation note 1 refer
  to the HList-style names ("Plucker / Sculptor / Embedder")
  which match frunk_core's HList module. The Coproduct module
  uses the Coproduct-style names; `CoprodUninjector` is the
  Plucker analog
  (`uninject(self) -> Result<T, Self::Remainder>`),
  `CoproductSubsetter` is the Sculptor analog
  (`subset(self) -> Result<Targets, Self::Remainder>`), and
  `CoproductEmbedder` is the Embedder analog
  (`embed(self) -> Out`). The adapter at
  [`fp-library/src/types/effects/coproduct.rs`](../../../fp-library/src/types/effects/coproduct.rs)
  uses the Coproduct-style names directly. Future plan
  references to Plucker / Sculptor / Embedder for the Coproduct
  adapter should be read as the Coproduct-style trait family
  above.
- **No newtype wrappers ship; the adapter is re-exports only.**
  The plan text says "newtype wrappers around
  `frunk_core::coproduct::{Coproduct, CNil}` plus impl blocks
  bridging Plucker / Sculptor / Embedder", on the premise that
  Brand-style impls require a local newtype to satisfy the
  orphan rules. A probe at
  `fp-library/tests/coproduct_brand_probe.rs` (committed during
  step 1 and removed in step 2 once the brands landed in
  production) disproved that premise: because `Kind_*` is
  fp-library's own trait, a generic `CoproductBrand<H, T>` (a
  local Brand struct) can carry the `impl_kind!` registration
  with `Of<'a, A> = Coproduct<H::Of<'a, A>, T::Of<'a, A>>`
  directly on the foreign value type. No wrapper is needed at
  the Brand boundary. The initial draft of step 1 shipped
  `BrandedCoproduct<H, T>(pub Coproduct<H, T>)` and `BrandedCNil`
  with `From` conversions and `CoprodInjector` /
  `CoprodUninjector` bridge impls; once the probe proved them
  dead-weight, they were removed in the same step. The adapter
  now re-exports frunk_core's types and trait family verbatim
  and points downstream consumers at the upcoming
  `crate::brands::CoproductBrand` (Phase 2 step 2). Two unit
  tests exercise raw Coproduct inject / uninject as the trait
  family's smoke test.
- **`frunk_core` chosen over `frunk`.** Step 1's text and
  Implementation note 1 name `frunk_core` directly. The
  umbrella `frunk` crate re-exports `frunk_core` plus
  `frunk_proc_macros`, `frunk_derives`, `Validated`, and
  `frunk_laws`. The effects port uses only `Coproduct`, `CNil`,
  the index types, and the four bridged traits, all of which
  live in `frunk_core`. Choosing `frunk_core` keeps the
  proc-macro / `syn` chain out of the dependency graph
  (fp-library already has `fp-macros` for proc-macros) and lets
  a future swap to `frunk` be a one-line Cargo.toml change since
  `frunk` is API-compatible with `frunk_core`. License is MIT
  for both, already on the [`deny.toml`](../../../deny.toml)
  allow-list.

### Step 3: `Member<E, Idx>` trait

- **`Member` layers on `CoprodInjector` + `CoprodUninjector`,
  not `CoproductSubsetter`.** Step 3's text says
  "Member<E, Indices> trait for first-order injection /
  projection, layered on top of `frunk_core::CoproductSubsetter`
  via the adapter from step 1". `CoproductSubsetter` is the
  row-narrowing (sculpt) trait, which takes a row of Targets
  and returns either the narrowed row or the remainder.
  `Member` is single-effect inject / project, which composes
  from `CoprodInjector::inject` (lift one effect into the row)
  and `CoprodUninjector::uninject` (try to extract one effect,
  returning the remainder). The blanket impl on Member delegates
  to those two traits directly. `CoproductSubsetter` remains
  the right primitive for row narrowing in handler code but is
  not what Member needs.
- **`Member` is a new trait with delegated methods, not a
  marker supertrait alias.** Three trait shapes were
  considered: (a) a marker supertrait
  `Member<E, Idx>: CoprodInjector<E, Idx> + CoprodUninjector<E, Idx>`
  with no own methods, (b) a new trait with `inject` / `project`
  methods that delegate to the frunk impls (the chosen shape),
  and (c) no Member trait at all (use frunk traits directly).
  Approach (b) was chosen so where-clauses, error messages, and
  rustdoc on smart-constructor signatures use fp-library's
  `Member` vocabulary rather than frunk's
  `CoprodInjector + CoprodUninjector` pair. The indirection is
  zero-cost at runtime; the small maintenance surface is worth
  the public-API insulation from frunk_core's internal naming
  choices and the closer match to PureScript Run's `Member r`
  precedent.
- **`Member` is single-effect only; row narrowing stays through
  `CoproductSubsetter` directly.** Step 3's text mentions only
  "first-order injection / projection". A separate
  `Members<Targets, Indices>` plural trait that bundles
  `CoproductSubsetter` for the same single-bound convenience
  may be added later when Phase 3 handler code wants it; until
  then, handler call sites use `CoproductSubsetter` directly.
  Adding `Members` is purely additive and does not affect
  `Member`.
- **`Member` is agnostic to Coyoneda wrapping.** Row variants
  emitted by the `effects!` macro (Phase 2 step 8) are
  `Coyoneda<E, A>`-wrapped, so `Member<Coyoneda<E, A>, Idx>` is
  what smart constructors prove against the row. `Member`
  itself does not bake in any Coyoneda assumption; the wrapping
  policy belongs to the smart constructors that the macro emits
  (Phase 2 step 9). If step 9's call sites want a sugar trait
  `EffectMember<E, Idx>` that finds the position whose Coyoneda
  wraps `E`, that lands then on top of `Member`, not as a
  redefinition of it.

### Step 4: split into 4a (foundation) and 4b (Explicit family)

- **The plan's "step 4" is split into two commits.** Steps 1, 2,
  and 3 each landed as a single commit of 60-300 lines. Step 4
  as written bundles a structurally larger amount of work into
  one commit: three `WrapDrop` impls for existing row brands,
  the `Node` / `NodeBrand` machinery (~250 lines), three Erased
  Run wrapper types (~600 lines combined), three Explicit Run
  wrapper types plus three new brands plus their `WrapDrop`
  impls, and a full brand-level type-class hierarchy
  (`Pointed` / `Functor` / `Semimonad` / `Monad` plus
  `RefFunctor` / `RefPointed` / `RefSemimonad` / `RefMonad` and
  `SendPointed` / `SendRef*`) on the three Explicit brands
  delegating to `FreeExplicitBrand`'s impls. Estimated total
  ~1500-3000 new lines across 7+ new files.

  Splitting:
  - **4a (this commit):** foundation. Row-brand `WrapDrop`
    impls (`CNilBrand`, `CoproductBrand`, `CoyonedaBrand`),
    `Node` / `NodeBrand` machinery, the three Erased Run wrapper
    types (`Run`, `RcRun`, `ArcRun`) with `Drop` / `Clone`
    inheritance and `from_*_free` / `into_*_free` zero-cost
    construction sugar. Verify-clean.
  - **4b (next commit):** the Explicit family. Three Explicit
    Run wrappers (`RunExplicit`, `RcRunExplicit`,
    `ArcRunExplicit`), three new brands
    (`RunExplicitBrand`, `RcRunExplicitBrand`,
    `ArcRunExplicitBrand`), their `WrapDrop` impls, and the
    full brand-level type-class hierarchy delegating to
    `FreeExplicitBrand`.

  This deviates from the protocol's "one step per commit" rule
  but keeps each commit independently reviewable and finishable
  in a single agent session. The user-facing operations
  (`pure` / `peel` / `send` / `bind` / `map` / `lift_f` /
  `evaluate` / `handle`) on all six Run variants remain in
  Phase 2 step 5 as written.

- **`fp-library/src/types/run/` is renamed to
  `fp-library/src/types/effects/`** (and the parent module file
  from `types/run.rs` to `types/effects.rs`). Step 4's plan text
  literally says "Six `Run` types at `fp-library/src/types/run.rs`
  (and sibling files)". The current file is the parent module
  declaring submodules for the broader effects subsystem
  (`coproduct`, `member`, `variant_f`, plus the new
  `node` / `run` / `rc_run` / `arc_run`); none of those are
  Run-specific. Naming the parent module `effects` matches what
  the file's own doc comment already calls itself ("Effects
  subsystem") and what the plan / decisions docs use throughout
  ("effects subsystem", "effects port", "effects plan").

  Renaming also avoids the `module_inception` clippy lint that
  would fire on `types::run::run::Run` (the lint is denied via
  `-D warnings`); without the rename, the lint would have
  required `#[expect(clippy::module_inception)]` on the inner
  `pub mod run;` declaration. The rename eliminates the lint at
  the source rather than papering over it.

  Affected import sites: tests and inner modules of
  `node.rs` / `run.rs` / `rc_run.rs` / `arc_run.rs` reference
  `crate::types::effects::*`; brand doc-comments in `brands.rs`
  for `NodeBrand` / `CoproductBrand` / `CNilBrand` /
  `CoyonedaBrand` updated; markdown link references in
  [`plan.md`](plan.md), [`resolutions.md`](resolutions.md), and
  this file updated.

- **Three Erased Run wrappers ship as tuple-struct newtypes
  with `from_*_free` / `into_*_free` zero-cost conversion
  rather than as type aliases.** Step 4's plan text says
  "Each is a thin wrapper over its Free variant"; an alternative
  reading would have used `pub type Run<R, S, A> = Free<NodeBrand<R, S>, A>;`.
  Tuple-struct newtypes were chosen so the user-facing public
  API (`Run::pure`, `Run::peel`, `Run::send`, etc., landing in
  step 5) lives on `Run` rather than as inherent methods on
  `Free`, which would conflate the two abstractions. Zero-cost
  conversion via `from_free` / `into_free` keeps the
  newtype-vs-alias distinction free at the call site.

- **`ArcRun`'s where-clause uses an associated-type bound on
  the `NodeBrand<R, S>` projection rather than recursive
  `Send + Sync` constraints on `R`, `S` separately.** The
  bound shape is
  `NodeBrand<R, S>: WrapDrop + Kind_*<Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync> + 'static`.
  This mirrors `ArcFree`'s own struct-level bound from Phase 1
  step 3 (which uses the same trick on `F` directly) and is the
  pattern that lets the compiler auto-derive `Send + Sync` on
  the inner `ArcFreeInner`. Recursive `R: Send + Sync` /
  `S: Send + Sync` constraints would not be sufficient because
  the substrate's continuations live in `Arc<dyn Fn + Send + Sync>`
  storage that needs the `Of<...>: Send + Sync` projection
  resolved at the brand level, not at the row component level.

- **No `WrapDrop` impl for `RcCoyonedaBrand` or `ArcCoyonedaBrand`
  in 4a.** The plan's step-4 inventory of `WrapDrop` impls lists
  `CoyonedaBrand` only. The Run typical pattern uses
  `CoyonedaBrand` on the first-order row, so the existing impl
  is sufficient for the `RcRun` / `ArcRun` substrates' bound
  resolution as long as the user picks `CoyonedaBrand` (or
  `IdentityBrand` directly) for the row's head brands. Adding
  `WrapDrop` for the refcounted Coyoneda brands is a follow-up
  if step-4b's tests or a Phase 3 handler stack genuinely need
  them.

- **Brand-level coverage on the Explicit Run brands ships only
  the achievable subset.** The plan text named a full
  `Functor / Pointed / Semimonad / Monad` hierarchy plus the
  `Ref*` and `SendRef*` equivalents on the three Explicit Run
  brands. Step 4b ships:
  - `RunExplicitBrand`: `Functor`, `Pointed`, `Semimonad`,
    `RefFunctor`, `RefPointed`, `RefSemimonad`.
  - `RcRunExplicitBrand`: `Pointed` plus `RefFunctor`,
    `RefPointed`, `RefSemimonad`.
  - `ArcRunExplicitBrand`: `SendPointed` only.

  `Monad` / `RefMonad` / `SendMonad` are unreachable because the
  blanket impls require `Applicative` / `RefApplicative` /
  `SendApplicative`, which the underlying
  `*FreeExplicitBrand`s deliberately do not implement. The
  `SendRef*` hierarchy is unreachable on `ArcRunExplicitBrand`
  because `ArcFreeExplicitBrand` does not implement it (the
  `for<'a, A>` HRTB needed to express
  `Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync` at the
  impl-block level is not in stable Rust). Inherent `bind` and
  `map` methods on `RcRunExplicit` and `ArcRunExplicit` cover
  the by-value monadic surface for concrete-type call sites.
  See [resolutions.md](resolutions.md#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)
  for full rationale.

- **Ref hierarchy on `RunExplicitBrand` and `RcRunExplicitBrand`
  is bounded by `R: RefFunctor, S: RefFunctor`.** The brand
  impls compile against the cascade
  `R: RefFunctor + 'static, S: RefFunctor + 'static`. Step 4b
  adds `RefFunctor` impls on the row brands to satisfy this
  cascade, but
  [`CoyonedaBrand`](../../../fp-library/src/brands.rs) does not
  implement `RefFunctor`, so canonical Run effect rows
  (`CoproductBrand<CoyonedaBrand<E_i>, ...>`) do not satisfy
  the cascade in practice. Brand-level `Ref*` dispatch is
  reachable only for synthetic rows whose head brands implement
  `RefFunctor` directly (e.g.,
  `CoproductBrand<IdentityBrand, CNilBrand>`). Adding
  `RefFunctor` to `CoyonedaBrand` is scope-creep beyond step
  4b; tracked separately.

- **Row-brand `RefFunctor` and `Extract` cascade impls land in
  step 4b.** Step 4a's row-brand inventory listed only
  `Functor` and `WrapDrop`. Step 4b extends to include
  `RefFunctor` impls on `CNilBrand`, `CoproductBrand<H, T>`, and
  `NodeBrand<R, S>` (required by the Ref-hierarchy delegation
  per the bullet above) and `Extract` impls on the same three
  brands. The `Extract` cascade is needed because
  [`FreeExplicit::evaluate`](../../../fp-library/src/types/free_explicit.rs)
  requires `F: Extract`, and tests / doctests over canonical
  Run-shaped programs assert evaluation results. Without the
  cascade, brand-level construction works but `evaluate()` does
  not; with it, programs whose row brands themselves have
  `Extract` (e.g., `IdentityBrand`-based test rows) can be
  evaluated.

- **`Node<'a, R, S, A>` gets a manual `Clone` impl.**
  [`RcFreeExplicit::evaluate`](../../../fp-library/src/types/rc_free_explicit.rs)
  and
  [`ArcFreeExplicit::evaluate`](../../../fp-library/src/types/arc_free_explicit.rs)
  carry the per-`A` bound
  `Apply!(<F as Kind!(...)>::Of<'a, *FreeExplicit<'a, F, A>>): Clone`
  (used in the shared-state recovery fallback when the outer
  refcount is not unique). For `F = NodeBrand<R, S>`, this
  expands to
  `Node<'a, R, S, *FreeExplicit<'a, NodeBrand<R, S>, A>>: Clone`.
  The manual `Clone` impl on `Node` is bounded by
  `Apply!(<R as Kind!(...)>::Of<'a, A>): Clone` and the `S`
  projection's `Clone`; it clones the active variant's payload.

- **`SendRefFunctor` cascade on row brands is _not_ added.** The
  plan's step 4b active-blocker entry anticipated a Send-side
  cascade alongside the by-reference cascade. Since
  `ArcRunExplicitBrand` cannot have a `SendRef` hierarchy (per
  the brand-level coverage bullet above), there is no consumer
  for `SendRefFunctor` on the row brands at this stage.
  Deferred until a future need surfaces.

- **Re-export pattern follows the
  [`optics`](../../../fp-library/src/types/optics.rs) precedent:
  selective top-level + comprehensive subsystem-scoped.** The
  six Run wrapper headline types
  (`Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
  `ArcRunExplicit`) ship at the top level
  ([`crate::types::*`](../../../fp-library/src/types.rs)); the
  same six plus `Node` and `VariantF` ship at
  [`crate::types::effects::*`](../../../fp-library/src/types/effects.rs)
  for the namespaced form. Brand types stay in
  [`crate::brands::*`](../../../fp-library/src/brands.rs) per
  the existing precedent for all brand types. See
  [resolutions.md](resolutions.md#resolved-2026-04-27-re-export-pattern-for-the-effects-subsystem-types-follows-the-optics-ab-hybrid)
  for the full options analysis.

### Step 5: `pure` / `peel` / `send` core operations on six Run variants

- **`*Run::send` takes the `Node`-projection value, not the
  row-variant layer.** The plan-text expectation was that `send`
  takes the first-order row variant `R::Of<'_, A>` and constructs
  `Node::First(layer)` internally before delegating to
  `*Free::lift_f`. While implementing `ArcRun::send` this shape
  failed to compile: `ArcFree`'s struct-level HRTB
  (`Of<'static, ArcFree<...>>: Send + Sync`) poisons GAT
  normalization in any scope mentioning it, so constructing a
  `Node::First(layer)` literal there cannot be unified with the
  `<NodeBrand<R, S> as Kind>::Of<'_, A>` projection that
  `lift_f` expects. The workaround that succeeds: pass an
  already-projection-typed value as the parameter, never
  construct a Node literal inside the HRTB scope.

  Eleven experiments at
  [`fp-library/tests/arc_run_normalization_probe.rs`](../../../fp-library/tests/arc_run_normalization_probe.rs)
  isolated the trigger and validated the workaround. The probe
  file ships in `tests/` (trimmed to four passing patterns) as
  a regression test documenting the limit. See
  [resolutions.md](resolutions.md#resolved-2026-04-27-runsend-takes-a-node-projection-value-to-sidestep-gat-normalization-poisoning-under-arcfrees-hrtb)
  for the full investigation.

  The signature change applies symmetrically to all six wrappers
  (`Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
  `ArcRunExplicit`) so the API is uniform; only `ArcRun` strictly
  requires it (the others would compile with the natural shape
  too, but uniformity matters for step 7 macros and step 9
  smart constructors that emit `send` calls).

- **`FreeExplicit::to_view` precursor.** `FreeExplicit`'s `view`
  field is private. `RunExplicit::peel` needs to walk the view
  to expose the underlying `FreeExplicitView::Pure(a)` /
  `FreeExplicitView::Wrap(layer)` shape. A small precursor
  [`pub fn to_view(self) -> FreeExplicitView<'a, F, A>`](../../../fp-library/src/types/free_explicit.rs)
  was added on `FreeExplicit`, mirroring the existing
  `RcFreeExplicit::to_view` / `ArcFreeExplicit::to_view`
  methods. Not in the plan-text inventory; recorded here.

- **Doctests use `peel`-based assertions (not `evaluate`-based)
  for the Erased family `Run` and `RcRun`.**
  `Free::evaluate`, `RcFree::evaluate`, and `ArcFree::evaluate`
  require `F: Extract` on the substrate functor, which
  `CoyonedaBrand` does not implement. To assert non-trivial
  behavior in doctests without pulling in `Extract`-having row
  brands, the `pure` and `peel` doctests on `Run` and `RcRun`
  use `peel`-based assertions
  (`assert!(matches!(run.peel(), Ok(value)))`); doctests on
  `RcRun::pure` and `RcRun::peel` switch to an `Identity`-headed
  row so `peel`'s per-projection `Clone` bound is satisfied
  (`Identity<RcFree>: Clone` is unconditional). `ArcRun`'s
  `peel` similarly works only with `Identity`-headed rows. The
  `Run::send` doctest uses a `Coyoneda`-headed row (`Run` lacks
  the `Clone` bound on `peel` since `Free::resume` doesn't carry
  one) and asserts `is_err()` on the resulting program (the
  full pattern-match into `Coyoneda<...>` value would require
  evaluating the Coyoneda function which is beyond the scope of
  step 5).

### Step 6: Erased -> Explicit conversion via `From` impls

- **API direction interpretation: Erased -> Explicit only.**
  The plan text reads "Conversion methods between paired Erased
  and Explicit Run variants: `Run::into_explicit() -> RunExplicit`,
  ..., and the reverse `RunExplicit::from_erased(...)`, etc.".
  The phrase "the reverse" is ambiguous: it could mean "the
  reverse direction (Explicit -> Erased)" or "the
  reverse-construction-style API (constructor on the target
  type rather than method on the input)". Step 6 implements the
  latter reading: a single Erased -> Explicit conversion per
  pair, exposed via standard
  [`From`](https://doc.rust-lang.org/std/convert/trait.From.html)
  impls so users get both call styles
  (`Explicit::from(erased)` and `erased.into()`) from one impl.
  Three considerations drive this reading: (1) the plan's own
  "Preserves multi-shot / Send + Sync properties" clarification
  only describes the Erased -> Explicit direction
  (`RcRun -> RcRunExplicit keeps multi-shot`, `ArcRun ->
ArcRunExplicit keeps Send + Sync`); (2) the literal name
  `from_erased` ("from an Erased value") takes an Erased input,
  not produces one; (3) the existing precedent from step 4b's
  wrappers ships `from_*_free` and `into_*_free` for the
  Free <-> Run pair, also as two API styles for the same
  direction. If the Explicit -> Erased direction is needed in
  a later phase, it can be added as a new step rather than
  inferred from this ambiguous wording.
- **Trait-based conversion via
  [`From`](https://doc.rust-lang.org/std/convert/trait.From.html),
  not custom inherent methods.** The plan text spells the
  conversions as `Run::into_explicit()` (method on Erased) and
  `RunExplicit::from_erased(...)` (constructor on Explicit).
  The wider codebase uses
  [`From`](https://doc.rust-lang.org/std/convert/trait.From.html)
  for sibling-type conversions extensively
  ([rc_coyoneda.rs:852](../../../fp-library/src/types/rc_coyoneda.rs),
  [arc_coyoneda.rs:879](../../../fp-library/src/types/arc_coyoneda.rs),
  [lazy.rs](../../../fp-library/src/types/lazy.rs) and
  [trampoline.rs](../../../fp-library/src/types/trampoline.rs)
  for the Lazy <-> Trampoline pair, the
  [TryLazy / TryThunk / TrySendThunk family](../../../fp-library/src/types/try_lazy.rs);
  approximately 35
  [`From`](https://doc.rust-lang.org/std/convert/trait.From.html)
  impls across the type modules). Step 6 follows that
  precedent: a single
  [`From<*Run<R, S, A>> for *RunExplicit<'static, R, S, A>`](https://doc.rust-lang.org/std/convert/trait.From.html)
  impl per pair, with the conversion logic in the `from` body.
  The blanket
  [`Into`](https://doc.rust-lang.org/std/convert/trait.Into.html)
  impl gives `run.into()` for free. This consolidates two
  inherent methods per pair (`into_explicit` + `from_erased`)
  into one trait impl, matches Rust idiom, and removes the
  delegation indirection while preserving both call styles at
  use sites.
  [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html)
  was considered but does not apply: the conversion is total
  once type-level bounds are satisfied.
- **`From` impl lives in the destination file.** The codebase
  precedent splits between source-file
  ([rc_coyoneda.rs:852](../../../fp-library/src/types/rc_coyoneda.rs)
  has `From<RcCoyoneda> for Coyoneda`) and destination-file
  ([thunk.rs:320](../../../fp-library/src/types/thunk.rs) has
  `From<Lazy> for Thunk`,
  [try_lazy.rs:467](../../../fp-library/src/types/try_lazy.rs)
  has `From<TryThunk> for TryLazy`, etc.). Destination-file is
  the dominant precedent, and it matches the original
  plan-text's `RunExplicit::from_erased(...)` placement
  intuition (the constructor lives on the destination). Step 6
  places the three impls in
  [run_explicit.rs](../../../fp-library/src/types/effects/run_explicit.rs),
  [rc_run_explicit.rs](../../../fp-library/src/types/effects/rc_run_explicit.rs),
  and
  [arc_run_explicit.rs](../../../fp-library/src/types/effects/arc_run_explicit.rs).
- **Bounds: per-method bounds on Rc/Arc variants migrate to
  impl-block `where` clauses.** Inherent methods can carry
  per-method `where` clauses;
  [`From::from`](https://doc.rust-lang.org/std/convert/trait.From.html)
  cannot (the trait signature is fixed). The Rc variant's
  `A: Clone` and projection `Clone` bound, and the Arc
  variant's `A: Clone + Send + Sync`, projection `Clone` bound,
  and `NodeBrand<R, S>: Functor` bound (the latter not implied
  by the existing impl-block bound, which only carries
  `WrapDrop` and the `Send + Sync` projection HRTB) all move to
  the
  `impl<...> From<*Run<R, S, A>> for *RunExplicit<'static, R, S, A> where ...`
  block-level `where` clause. The Run variant has no extra
  bounds beyond its impl block.
- **GAT-poisoning workaround: passes through cleanly without
  surfacing.** The Arc impl operates inside the HRTB-bearing
  impl-block scope on
  [`ArcRun`](../../../fp-library/src/types/effects/arc_run.rs)
  (the `Of<'static, ArcFree<...>>: Send + Sync` projection
  HRTB). Step 5 established that constructing
  `Node::First(layer)` literals inside such a scope fails GAT
  normalization. Step 6 composes its conversion entirely from
  values whose projection types come from `peel`'s return and
  from `Functor::map`'s output, never from inline `Node::*`
  literal construction; the `ArcFreeExplicit::wrap` call
  receives the mapped projection value directly. Compilation
  passed without any of the four workaround patterns from
  [`fp-library/tests/arc_run_normalization_probe.rs`](../../../fp-library/tests/arc_run_normalization_probe.rs)
  needing to be invoked.
- **Tests exercise both call styles.** The 12 new tests split
  six exercising `*RunExplicit::from(erased)` (the
  constructor-style, in the destination file's tests) and six
  exercising `erased.into()` (the method-style via the blanket
  [`Into`](https://doc.rust-lang.org/std/convert/trait.Into.html)
  impl, in the source file's tests). This documents both API
  surfaces as part of the regression suite.

### Step 7: `im_do!` macro and supporting inherent methods (design pre-locked, implementation pending)

This entry captures the design decisions for step 7 made
ahead of implementation, so the implementer (whether the user
or a future agent session) lands a consistent, well-documented
result. Three things differ from the plan-text's literal step
7 description: the macro's name, its scope (extended from
"Erased family only" to "all six wrappers"), and the
forward-reservation of an applicative companion name.

- **Macro name: `im_do!` ("Inherent Monadic do") instead of
  `run_do!`.** The plan-text named the macro `run_do!`,
  scoping it nominally to the Run subsystem. Step 7's design
  review surfaced two reasons to prefer a dispatch-path name
  over a subsystem-scoped name:
  1. The dispatch pattern (inherent-method calls instead of
     trait dispatch) is the load-bearing fact about the
     macro. The Run-scoping is incidental , any wrapper type
     with inherent `bind` could use the same macro.
  2. The applicative companion (see below) needs a parallel
     name. `run_a_do!` reads awkwardly; `run_ado!` is
     asymmetric to the existing `m_do!` / `a_do!` pair.
     `im_do!` / `ia_do!` mirrors `m_do!` / `a_do!` cleanly
     while adding the dispatch-path prefix `i` for "inherent".

  Renaming a macro after users start writing call sites is
  costly (deprecation cycle, documentation churn). Locking
  the name in before step 7 ships is much cheaper.

- **Forward-reserved applicative companion: `ia_do!`
  ("Inherent Applicative do").** Step 7 itself ships only
  the monadic form (`im_do!`); applicative inherent
  do-notation is deferred until a concrete need surfaces
  (likely Phase 3+ when handler composition introduces
  independent-bind patterns over `ArcRun`). However, the
  name is reserved in plan.md and in step 7's commit message
  so that whenever the applicative form lands, the
  convention is already in place.

  **Same-length naming is deliberate.** `im_do!` and
  `ia_do!` are 5 characters each; `m_do!` and `a_do!` are
  4 characters each. Within each pair, the monadic and
  applicative forms have identical lengths so neither is
  typographically disfavored. This is intentional design
  guidance: applicative composition is generally preferable
  to monadic composition when binds are independent (it
  allows parallelization, avoids closure-nesting issues
  in `ref` mode, and produces simpler desugarings , a
  single `liftN` / `map` call instead of nested `bind`s).
  Users should reach for `ia_do!` over `im_do!` (and
  `a_do!` over `m_do!`) whenever the binds are independent;
  giving the applicative form a longer name would subtly
  push users toward the wrong default. This mirrors the
  PureScript `do` / `ado` convention's same-length
  pairing.

- **Scope expansion: `im_do!` covers all six Run wrappers,
  not just the Erased family.** The plan-text restricted
  the macro to `Run` / `RcRun` / `ArcRun` and assumed
  `m_do!(ref RcRunExplicitBrand)` would handle by-reference
  do-notation on the Explicit family. Step 7's design
  review surfaced two reasons to expand scope:
  1. **Canonical Coyoneda-headed rows can't reach the
     brand-level `ref` form.** `CoyonedaBrand: RefFunctor`
     is unimplementable on stable Rust (per
     [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)),
     so `m_do!(ref RcRunExplicitBrand<R, S> { ... })` over
     a row containing `CoyonedaBrand` (which the `effects!`
     macro emits in the canonical case) fails type checking
     at the row-brand cascade. Users with canonical effect
     rows need an alternative path. Inherent `ref_bind`
     (added in step 7b) sidesteps the cascade by cloning
     the program and calling by-value `bind` with a
     wrapping closure , this works without requiring the
     row brand to be `RefFunctor`. `im_do!(ref RcRunExplicit { ... })`
     desugars to inherent `ref_bind` calls, providing the
     by-reference path that `m_do!(ref ...)` cannot reach.
  2. **`m_do!(ref ArcRunExplicitBrand)` is permanently
     unreachable.** `ArcFreeExplicitBrand: SendRefFunctor`
     is unimplementable on stable Rust (per the limitations
     doc; the closure passed to `send_ref_map` returns a
     value whose `Send + Sync` auto-derive needs the
     per-`A` `Kind<Of<'a, A>: Send + Sync>` HRTB).
     `ArcRunExplicitBrand` would only delegate, so it
     inherits the gap. `im_do!(ref ArcRunExplicit { ... })`
     is the only by-reference path available for
     `ArcRunExplicit`.

  Covering all six wrappers with one macro produces a
  coherent user-facing story: `im_do!` works wherever
  inherent `bind` (and `ref_bind`, for the four
  `Clone`-able wrappers) is available, regardless of the
  underlying brand-dispatch story.

- **Sub-task split: 7a, 7b, 7c.** The plan-text's step 7
  is now structurally three sub-steps. They could land as
  one commit each or bundle into 7a+7b together with 7c
  separate; the implementer decides at commit time per the
  oversized-step-splitting protocol in
  [`.claude/CLAUDE.md`](../../../CLAUDE.md). Recommended
  decomposition:
  - 7a: inherent `bind` and `map` on `Run`, `RcRun`, `ArcRun`,
    `RunExplicit` (the four wrappers that don't already
    have them; `RcRunExplicit` and `ArcRunExplicit` shipped
    them in step 4b).
  - 7b: inherent `ref_bind` and `ref_map` on `RcRun`, `ArcRun`,
    `RcRunExplicit`, `ArcRunExplicit` (the four `Clone`-able
    wrappers).
  - 7c: `im_do!` macro itself, with by-value and `ref` forms,
    plus the `compile_fail` UI test for `im_do!(ref ...)`
    on non-`Clone` wrappers (`Run`, `RunExplicit`).

- **Documentation strategy: name and rationale captured in
  three places.** When step 7c lands the macro, the macro's
  doc-comment (the `//!` module doc and the `///` proc-macro
  function doc) should explain:
  1. What the name means: "im" is short for "inherent
     monadic" (parallel to "m" for "monadic" in `m_do!`).
     The matching applicative form is `ia_do!` ("inherent
     applicative", parallel to `a_do!`).
  2. Why "inherent": the macro desugars to inherent method
     calls (`expr.bind(|x| ...)`) rather than trait
     dispatch (`<Brand as Semimonad>::bind(expr, |x| ...)`).
     This is the dispatch path that works for types whose
     brand can't satisfy `Semimonad` due to per-`A`
     bounds that stable Rust can't carry in trait method
     signatures.
  3. When to use `im_do!` vs `m_do!`: prefer `m_do!` when
     the type's brand has full `Semimonad` coverage
     (typeclass-generic, no clones); use `im_do!` when
     `m_do!` doesn't reach (e.g., Erased Run family,
     or canonical-row `ref` mode).
  4. Why the same length as `ia_do!`: encourages users to
     prefer the applicative form when binds are
     independent, mirroring the `m_do!` / `a_do!` pairing.

  Cross-references: this deviations.md entry, the macro's
  source-level doc comment, and the eventual user-facing
  guide in `fp-library/docs/run.md` (Phase 5 step 4) all
  carry the same rationale. A precursor note in
  `effects.rs`'s module docstring during step 7c is
  acceptable until Phase 5 lands the full guide.

- **Shared input parser.** When step 7c writes the macro,
  extract the existing `m_do!` / `a_do!` input parser (in
  `fp-macros/src/m_do/input.rs` and
  `fp-macros/src/a_do/input.rs`) into a shared
  `fp-macros/src/support/do_input.rs` (or similar) so
  surface-syntax features stay consistent across all four
  macros. This prevents drift when (e.g.) typed-bind
  syntax is extended in one and forgotten in others.

### Step 7c.2b: `im_do!` proc-macro implementation

The macro lands at
[`fp-macros/src/effects/im_do/codegen.rs`](../../../fp-macros/src/effects/im_do/codegen.rs)
under a new
[`fp-macros/src/effects/`](../../../fp-macros/src/effects/)
subsystem directory, mirroring the existing `m_do.rs` /
`m_do/codegen.rs` shape. The proc-macro export is registered in
[`fp-macros/src/lib.rs`](../../../fp-macros/src/lib.rs).
Two implementation choices diverge from the design pre-lock or
warrant explicit capture:

- **Bare path syntax for `pure` rewriting (`Wrapper::pure(...)`,
  not `<Wrapper>::pure(...)`).** The first iteration emitted
  `<#wrapper>::pure(#args)` and `<#wrapper>::ref_pure(&(#args))`,
  expecting `<Wrapper>::method` to behave identically to
  `Wrapper::method` for inherent associated functions. It does
  not: `<Type>::method(...)` is the fully-qualified form where
  `Type` must be a complete type, so `<Run>::pure(...)` fails
  with `error[E0107]: missing generics for struct Run` because
  `Run<R, S, A>` requires three type parameters and `Run` alone
  is incomplete in fully-qualified position. The bare path form
  `Run::pure(...)` lets rustc infer the generics from the
  expression context (return type, argument types). Switched
  to `#wrapper::pure(#args)` and `#wrapper::ref_pure(&(#args))`;
  16 of 16 integration tests then compiled. Users who pass a
  qualified path like `crate::types::effects::run::Run` get
  `crate::types::effects::run::Run::pure(...)`, also valid.
  The `Type` ASTs that would break this (incomplete generics,
  e.g. `Run<R, S>` written by the user) are nonsensical anyway:
  the third parameter `A` must be inferred from the call site,
  so users would never write them.

- **No custom diagnostic for `im_do!(ref ...)` on non-`Clone`
  wrappers.** The pre-lock asked for "a clear `cannot use
`ref` form on non-`Clone` wrapper` diagnostic". The macro
  cannot introspect at expansion time whether a wrapper has
  inherent `ref_bind` / `ref_pure`, so emitting a custom
  `compile_error!` would require either: (a) a hardcoded
  allowlist of wrapper names (brittle, fails on aliases and
  module-qualified paths), or (b) generating a trait-witness
  bound that fails for non-`Clone` types (more complex than
  the macro warrants). Instead, the macro emits straight
  inherent method calls, and rustc's natural error
  ("no method named `ref_bind` found for struct `Run<R, S, A>`"
  followed by "no function or associated item named `ref_pure`
  found") names the wrapper directly. The compile_fail UI test
  at
  [`fp-library/tests/ui/im_do_ref_on_non_clone_wrapper.rs`](../../../fp-library/tests/ui/im_do_ref_on_non_clone_wrapper.rs)
  captures this error; the test file's source comments
  document the property. Only `Run` is exercised (a single
  failure demonstrates the property; the same error pattern
  applies to `RunExplicit`).

- **Method-call syntax handles ref-mode borrowing
  automatically.** The plan's design pre-lock reused
  `wrap_container_ref` from `m_do/codegen.rs` to wrap the
  container expression in `&(...)` for ref dispatch. That
  helper exists because the brand-level free function
  `bind::<Brand, _, _, _, _>(container, ...)` takes the
  container as a value parameter; `ref_bind` requires `&container`.
  For `im_do!`'s method-call dispatch, Rust's auto-ref handles
  this: `(expr).ref_bind(...)` automatically borrows `expr` as
  `&expr` for the `&self` receiver. The `wrap_container_ref`
  import was removed from the codegen; only
  `format_bind_param` and `format_discard_param` are reused
  from
  [`fp-macros/src/m_do/codegen.rs`](../../../fp-macros/src/m_do/codegen.rs).

### Step 8: `effects!` macro migration plus `raw_effects!` companion

The macros land at
[`fp-macros/src/effects/effects_macro.rs`](../../../fp-macros/src/effects/effects_macro.rs)
with the shared lexical-sort helper at
[`fp-macros/src/effects/row_sort.rs`](../../../fp-macros/src/effects/row_sort.rs).
fp-library exposes `raw_effects!` via a new
[`__internal`](../../../fp-library/src/lib.rs) module marked
`#[doc(hidden)]`. Three implementation choices warrant explicit
capture:

- **File renamed from `effects/effects.rs` to
  `effects/effects_macro.rs` to dodge clippy's `module_inception`
  lint.** The plan-text named the file
  `fp-macros/src/effects/effects.rs`, which would create the
  module path `crate::effects::effects`. Clippy's
  `module_inception` lint (an `-D warnings` rule under
  `just clippy`) flags any module nested inside a same-named
  parent: `error: module has the same name as its containing
module`. Three options were considered: (a) add
  `#[allow(clippy::module_inception)]` to the inner module;
  (b) flatten the worker code into the parent
  `fp-macros/src/effects.rs`; (c) rename the inner file. Option
  (c) chosen because (a) leaves a static-analysis exception in
  the codebase that future maintainers must understand, and (b)
  scales poorly as the effects subsystem accumulates other
  per-macro modules (`scoped_effects/`, `handlers/`, etc.) which
  would each need similar special-casing in `effects.rs`.
  Renaming the file is a one-line deviation from plan-text with
  no semantic consequence: the proc-macro is still named
  `effects!` (registered in `lib.rs`), and the worker function
  path is `crate::effects::effects_macro::effects_worker` rather
  than `crate::effects::effects::effects_worker`. The file's
  module-doc cross-references this entry.

- **`raw_effects!` is `#[doc(hidden)]` on the proc-macro export,
  re-exported through `fp_library::__internal`.** Decisions
  section 4.6 specifies `crate::__internal::raw_effects!` as the
  public-facing path for fp-library-internal use. Two
  ergonomic concerns shaped the implementation:
  1. fp-library does `pub use fp_macros::*;` which star-exports
     every proc-macro at the crate root, including
     `raw_effects!`. Switching to explicit re-exports (listing
     each macro by name) would scale poorly across many
     macros and fight the existing pattern, so the star
     re-export stays.
  2. Without further intervention, `fp_library::raw_effects!`
     would be reachable and indexed by rustdoc as a top-level
     public macro, contradicting decisions section 4.6's
     "not part of the public surface" intent.

  Resolution: mark the proc-macro export `#[doc(hidden)]` in
  fp-macros so rustdoc skips it, and add a new
  `pub mod __internal { pub use fp_macros::raw_effects; }` to
  fp-library's `lib.rs` (also `#[doc(hidden)]`). The internal
  intent is then visible at every call site as
  `fp_library::__internal::raw_effects!`; the top-level path
  remains technically reachable but undocumented. fp-library's
  own internal usage routes through `__internal` so the
  internal-only convention is enforced by call-site discipline.

- **`assert_type_eq` pattern for canonical-ordering tests.** The
  plan's success criterion for `effects!` is that input order
  doesn't affect the resulting type:
  `effects![A, B] == effects![B, A]` at the type level. Rust
  has no built-in compile-time type-equality assertion, so the
  test pattern is:

  ```rust
  fn assert_type_eq<T>(_: PhantomData<T>, _: PhantomData<T>) {}
  let p1: PhantomData<R1> = PhantomData;
  let p2: PhantomData<R2> = PhantomData;
  assert_type_eq(p1, p2); // compiles iff R1 == R2
  ```

  The function takes two `PhantomData<T>` parameters (note: same
  `T`); passing `PhantomData<R1>` and `PhantomData<R2>` of
  different types fails compilation. The integration test file
  uses this pattern across empty / single / two / three-brand
  inputs. `static_assertions::assert_type_eq_all!` would be a
  drop-in alternative but would add a dev-dependency for a
  one-off test pattern; the inline helper is preferred.

- **Coyoneda-wrapped rows don't satisfy `RcRun::peel`'s `Clone`
  bound.** `RcRun::peel` requires
  `NodeBrand<R, S>::Of<'static, RcFree<...>>: Clone`, which for
  `IdentityBrand`-headed rows is satisfied directly. For
  `CoyonedaBrand`-wrapped rows (the canonical `effects!`
  output), the projection contains a
  `Coyoneda<F, RcFree<...>>` whose stored continuation is a
  trait object that is not `Clone`. The integration test
  `effects_row_drives_run_wrapper` therefore tests construction
  only (drops the program); the `peel`-exercising
  `raw_effects_row_drives_run_wrapper_with_peel` uses
  `raw_effects!` (un-wrapped form) which satisfies the bound.
  This is a documented limitation of the canonical Coyoneda
  form on the Erased Rc family, not a step 8 regression; the
  Explicit family's `peel` doesn't carry the Clone bound and
  works with both forms.

### Step 9b: bundled with 9e to keep `arc_run.rs` compiling

The 2026-04-28 expanded resolution decomposed step 9 into nine
independent sub-steps (9a-9i). Sub-step 9b ("replace
`F: Functor` with `F: SendFunctor` on `ArcFree`") and sub-step
9e ("switch `ArcRun` to `SendFunctor`-routed dispatch") were
listed separately. In practice they cannot land independently:
[`ArcRun::peel`](../../../fp-library/src/types/effects/arc_run.rs)
calls
[`ArcFree::resume`](../../../fp-library/src/types/arc_free.rs)
and
[`ArcRun::send`](../../../fp-library/src/types/effects/arc_run.rs)
calls
[`ArcFree::lift_f`](../../../fp-library/src/types/arc_free.rs).
After 9b's bound replacement, both `ArcFree` methods require
`F: SendFunctor`; `ArcRun`'s methods can no longer satisfy the
new bound with their existing `NodeBrand<R, S>: Functor`
constraint. Compilation breaks at every `ArcRun` method that
delegates to `ArcFree`.

Bundled 9b and 9e into a single commit. The combined commit:

- `ArcFree`: eight per-method `F: Functor` -> `F: SendFunctor`,
  three `F::map` -> `F::send_map`, plus `A: Send + Sync` added
  to `wrap` (because `send_map`'s closure parameter type
  `ArcFree<F, A>` requires `Send + Sync`, which transitively
  requires `A: Send + Sync`).
- `ArcRun`: two per-method `NodeBrand<R, S>: Functor` ->
  `NodeBrand<R, S>: SendFunctor`, one
  `<NodeBrand<R, S> as Functor>::map` ->
  `<NodeBrand<R, S> as SendFunctor>::send_map` in `peel`'s
  body. `Functor` import removed (unused); `SendFunctor`
  added.
- `arc_run_normalization_probe.rs`: pattern-A's `send` method
  (which mirrors production `*Run::send` for HRTB regression
  coverage) updated to `SendFunctor` to track the production
  shape.
- `arc_run_explicit.rs`'s `From<ArcRun>` impl: gains
  `SendFunctor` bounds on `R, S, NodeBrand<R, S>` to satisfy
  `ArcRun::peel`'s new requirement (the impl body's own
  `<NodeBrand<R, S> as Functor>::map` call stays unchanged
  because it routes through the not-yet-migrated
  `ArcFreeExplicit::wrap`; 9c+9f will resolve that side).

The same coupling is expected for 9c+9f (ArcFreeExplicit and
ArcRunExplicit). Future sub-steps 9d, 9g, 9h, 9i remain
independent.

The "verifies independently" criterion stays satisfied: the
9a, 9b+9e, 9c+9f, 9d, 9g, 9h, 9i sequence of commits each
verifies clean. The bundling reduces sub-step count from nine
to seven without weakening the per-commit review property.

### Steps 9d and 9g bundled: brand-level Send-aware surface unchanged on both Arc Explicit brands; inherent `ArcFreeExplicit::map` lands as concrete-type workaround

[Plan.md step 9d](plan.md) said "at minimum `SendFunctor`
should now be implementable" on
[`ArcFreeExplicitBrand`](../../../fp-library/src/brands.rs)
following the 9c substrate migration. The post-9c re-evaluation
found this prediction did not hold. A scratch
[`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
impl delegating through
[`ArcFreeExplicit::bind`](../../../fp-library/src/types/arc_free_explicit.rs)
(`fa.bind(move |a| ArcFreeExplicit::pure(func(a)))`) was
attempted; rustc rejected it with four blocking bounds:

1. `A: Clone` (from
   [`bind`](../../../fp-library/src/types/arc_free_explicit.rs)'s
   where-clause; not in
   [`SendFunctor::send_map`](../../../fp-library/src/classes/send_functor.rs)'s
   signature).
2. `<F as Kind>::Of<'_, ArcFreeExplicit<'_, F, A>>: Clone` (the
   per-`A` HRTB on the suspended layer; unexpressible in trait
   method signatures).
3. `<F as Kind>::Of<'_, ArcFreeExplicit<'_, F, A>>: Send` (same
   shape).
4. `<F as Kind>::Of<'_, ArcFreeExplicit<'_, F, A>>: Sync` (same
   shape).

These are the exact blockers that
[step 4b's resolution](resolutions.md#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)
documented for the parallel by-value `Functor`/`Semimonad` chain
on
[`RcFreeExplicitBrand`](../../../fp-library/src/brands.rs). The
9c substrate migration only changed which `F` trait
[`ArcFreeExplicit::bind_boxed`](../../../fp-library/src/types/arc_free_explicit.rs)
routes through internally (`F::map` to `F::send_map`); it did
not eliminate the `Clone` cascade on
[`into_inner_owned`](../../../fp-library/src/types/arc_free_explicit.rs)'s
shared-`Arc` recovery path, which is intrinsic to the
`Arc<Inner>` data shape.

Same blockers apply to
[`SendSemimonad::send_bind`](../../../fp-library/src/classes/send_semimonad.rs)
(calls `bind` directly) and to
[`SendLift::send_lift2`](../../../fp-library/src/classes/send_lift.rs)
(`bind`-based body needs the `F::Of: Clone + Send + Sync`
per-`A` bound on the suspended layer even though `A` and `B`
have `Clone` in the trait signature). The
[`SendSemiapplicative`](../../../fp-library/src/classes/send_semiapplicative.rs)
/
[`SendApplicative`](../../../fp-library/src/classes/send_applicative.rs)
/
[`SendMonad`](../../../fp-library/src/classes/send_monad.rs)
cascade is then blocked transitively via supertraits.

Action taken:

- Refreshed the inline comment block at
  [`fp-library/src/types/arc_free_explicit.rs`](../../../fp-library/src/types/arc_free_explicit.rs)
  so the post-9c re-evaluation is explicit (the existing comment
  correctly stated the blockers but pre-dated the
  SendFunctor-routed substrate; the refresh saves future
  implementors from redoing the probe).
- Added a Send-aware brand-level coverage table to
  [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)
  parallel to the existing by-value and by-reference tables for
  the Free Explicit family. The new table enumerates `SendFunctor`
  / `SendPointed` / `SendSemimonad` / `SendLift` coverage on
  [`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)
  and explains the binding constraint.
- Landed inherent
  [`ArcFreeExplicit::map`](../../../fp-library/src/types/arc_free_explicit.rs)
  as the concrete-type workaround for the unreachable
  brand-level `SendFunctor::send_map`. The per-`A`
  `Clone + Send + Sync` bounds that cannot live in the trait
  method signature fit cleanly in the inherent method's
  where-clause. Body delegates to existing
  [`bind`](../../../fp-library/src/types/arc_free_explicit.rs)
  via the standard `bind(|a| pure(f(a)))` pattern. Mirrors the
  [`ArcFree::map`](../../../fp-library/src/types/arc_free.rs)
  precedent for brand-blocked operations on the Erased family.
  Naming: the bare `map` (not `send_map`) follows the
  established Arc-substrate inherent-method convention used by
  [`ArcFree::map`](../../../fp-library/src/types/arc_free.rs)
  and
  [`ArcRunExplicit::map`](../../../fp-library/src/types/effects/arc_run_explicit.rs),
  where `Send + Sync` bounds live in the where-clause and the
  bare name is unambiguous because the non-Send variant is
  not implementable on the same type.
  Three new unit tests cover the basic transformation,
  composition with `bind`, and cross-thread `send`-via-`spawn`.

Scope discussion: this commit fuses two related concerns
(documenting the brand-level coverage gap + adding the inherent
workaround). The user explicitly chose this fold ("option 1")
over a follow-up split. The two pieces address the same
underlying gap, so reviewing them together makes the
"unreachable at brand level, reachable at concrete-type level"
contrast legible. `SendLift::send_lift2` and the rest of the
applicative cascade are NOT lifted to inherent methods at this
time; no Free family member has an inherent `lift2`, so adding
one would set new precedent for the whole family. Callers
needing lifted binary application can compose `bind` with `pure`
directly. This keeps the API addition minimal while still closing
the most-needed Send-aware monadic-companion (`map`).

Bundling rationale (9d and 9g into one commit): 9g's plan text
predicts "`SendPointed` plus whatever cascades from
`ArcFreeExplicitBrand`'s expanded surface" on
[`ArcRunExplicitBrand`](../../../fp-library/src/brands.rs). With
9d landing zero new brand-level impls on `ArcFreeExplicitBrand`,
nothing cascades; `ArcRunExplicitBrand` already has
[`SendPointed`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
from step 4b; and the wrapper's inherent surface
(`bind` / `map` / `ref_map` / `ref_pure`) is already complete from
steps 4b, 7a, 7b, and 7c.1. The Send-aware `map` on
[`ArcRunExplicit`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
already exists with the appropriate
`A: Clone + Send + Sync` and `Of: Clone + Send + Sync` bounds in
its where-clause (it's named `map`, not `send_map`, matching the
Arc-substrate naming convention that this commit also adopts via
the rename described above). So 9g has no code to land at all;
it ships as the documentation-only logical consequence of 9d's
re-evaluation outcome.

The bundling here is different in flavour from the 9b+9e and
9c+9f bundles documented above: those bundled because their
substrate/wrapper migrations were technically coupled (couldn't
land independently without breaking compilation). 9d and 9g are
not technically coupled in that sense; rather, they are
_logically_ coupled: 9g's outcome is a strict consequence of 9d's,
and splitting them across two commits would mean a separate "9g:
no actions taken; here's why" commit immediately after 9d. The
single combined commit tells one coherent story about post-9c
re-evaluation across both Arc Explicit brands and avoids a
content-free follow-up commit.

The "verifies independently" criterion the original 2026-04-28
expanded resolution required for the 9-sub-step decomposition
remains satisfied: this commit verifies clean under `just verify`,
and the next commit (9h: universal `*Run::lift`) is independent
of both 9d and 9g.

Implication for sub-step 9i: 9i lands `SendRefFunctor` (and
related Send-aware Ref-family traits) on
`ArcRunExplicitBrand` via inherent-method delegation. That
strategy uses the wrapper's existing inherent
`ref_map` / `ref_bind` / `ref_pure` (already in place from steps
7b and 7c.1) and is independent of 9d's and 9g's brand-level
re-evaluation. No change to 9i's scope.

### Step 9h: per-wrapper Coyoneda-variant pairing corrected to substrate-pointer-matched

The plan's
[step 9h per-wrapper delta table](plan.md) listed bare `Coyoneda`
for `RcRun::lift` and `RcRunExplicit::lift`. Rust rejects this:
`*Run::send` (and `*Run::peel` for the shared-pointer wrappers)
carries a per-method `Of<'_, *Free<..., *TypeErasedValue>>: Clone`
bound that is intrinsic to the `Rc`/`Arc`-shared substrate state.
With a `CoyonedaBrand`-headed row, that bound resolves to
`Coyoneda<...>: Clone`, which is unsatisfiable because
[`Coyoneda`](../../../fp-library/src/types/coyoneda.rs)'s
`Box<dyn FnOnce>` continuation is single-shot and not `Clone`. So
the methods compile in isolation but cannot be called with the
Run-canonical Coyoneda-headed row.

The
[2026-04-28 WIP branch `step-9-wip-with-arc-blocker`](https://github.com/nothingnesses/rust-fp-library/tree/step-9-wip-with-arc-blocker)
captured the WIP author's intended bound shape but did not validate
the integration tests. The same issue would have surfaced as a test
failure on `rc_run_lift_constructs` and
`rc_run_explicit_lift_round_trip` against the WIP code.

Corrected per-wrapper delta:

| Wrapper          | `'a`         | Coyoneda variant             | Driver                                                            |
| :--------------- | :----------- | :--------------------------- | :---------------------------------------------------------------- |
| `Run`            | `'static`    | `Coyoneda<'static, _, _>`    | `Free` is single-shot; no `Clone` cascade.                        |
| `RcRun`          | `'static`    | `RcCoyoneda<'static, _, _>`  | `RcFree`'s shared `Rc` state needs `Clone` on the row projection. |
| `ArcRun`         | `'static`    | `ArcCoyoneda<'static, _, _>` | `ArcFree`'s shared `Arc` state needs `Clone + Send + Sync`.       |
| `RunExplicit`    | `'a` (param) | `Coyoneda<'a, _, _>`         | `FreeExplicit` has no shared state.                               |
| `RcRunExplicit`  | `'a` (param) | `RcCoyoneda<'a, _, _>`       | `RcFreeExplicit`'s shared `Rc` state, same as `RcRun`.            |
| `ArcRunExplicit` | `'a` (param) | `ArcCoyoneda<'a, _, _>`      | `ArcFreeExplicit`'s shared `Arc` state, same as `ArcRun`.         |

The pattern: each wrapper's `lift` uses the Coyoneda variant whose
pointer kind matches the wrapper's substrate's pointer kind. This is
a uniform pairing rule rather than a per-wrapper exception.

Side artefact: step 9a added
[`ArcCoyonedaBrand: WrapDrop`](../../../fp-library/src/types/arc_coyoneda.rs)
and noted the impl mirrored
[`RcCoyonedaBrand`](../../../fp-library/src/types/rc_coyoneda.rs)'s
pattern, but `RcCoyonedaBrand` did not actually carry that impl.
This bundle adds it (also returns `None`, mirroring
[`CoyonedaBrand: WrapDrop`](../../../fp-library/src/types/coyoneda.rs)),
unblocking `RcCoyonedaBrand`-headed rows for use as `NodeBrand` row
brands on `RcRun`/`RcRunExplicit`. Without it, the
`NodeBrand<R, S>: WrapDrop` requirement on the `RcRun` /
`RcRunExplicit` struct definitions fails to recurse through
`CoproductBrand<RcCoyonedaBrand<...>, ...>: WrapDrop`.

The plan's `Run::lift` signature (already in production via commit
`34b6a97`) is unchanged; only the five new wrapper methods land in
this bundle. The reference signature in plan.md step 9h remains
correct for `Run`; the per-wrapper delta table above is the
correction.

`ArcRun::lift` uses the
[`lift_node` HRTB-poisoning fallback](../../../fp-library/src/types/effects/arc_run.rs)
the resolution anticipated. Inline construction of the
`Node::First` literal inside `ArcRun`'s impl-block scope failed
with a GAT-normalization error
(`Node<'_, {unknown}, ...> != <NodeBrand<R, S> as Kind>::Of<'static, A>`),
exactly the 2026-04-27 limit. Factoring the literal-build step into
the free `lift_node` function outside the HRTB-bearing scope
sidesteps the poisoning. The other five wrappers build the literal
inline successfully.

The integration test file
[`fp-library/tests/run_lift.rs`](../../../fp-library/tests/run_lift.rs)
ships 11 tests: round-trip on each of the six wrappers (all real
round-trips with the matched Coyoneda variant), second-branch
`Member` resolution on `Run` and `RunExplicit`, inferred-`Idx`
verification, and `lift().bind(...)` composition on `Run` and
`RunExplicit`. The Run-canonical row uses bare `CoyonedaBrand`; the
Rc/Arc-family rows use `RcCoyonedaBrand`/`ArcCoyonedaBrand`
respectively, matching the corrected per-wrapper delta.

### Step 9i: SendRef cascade reduced to `SendRefPointed` only; `SendRefFunctor` / `SendRefSemimonad` remain blocked

The plan's
[step 9i reference shape](plan.md) predicted that
`SendRefFunctor`, `SendRefSemimonad`, and the cascade above
would all land on
[`ArcRunExplicitBrand`](../../../fp-library/src/brands.rs) via
inherent-method delegation through the wrapper's
[`ref_map`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
/ `ref_bind` / `ref_pure` methods. Two probes against rustc
confirmed only `SendRefPointed` admits this delegation:

- `SendRefPointed` works (matching bounds; no closure parameter).
  Trait carries `A: Clone + Send + Sync`, matching
  [`ArcRunExplicit::ref_pure`](../../../fp-library/src/types/effects/arc_run_explicit.rs)'s
  bounds exactly.
- `SendRefFunctor` is blocked by four constraints, three of them
  the same per-`A` HRTB blockers documented for
  `ArcFreeExplicitBrand: SendFunctor` in 9d (`A: Clone`; per-`A`
  `<R as Kind>::Of<...>: Clone + Send + Sync`; same on `S` and
  `NodeBrand<R, S>`); plus a closure-bound mismatch:
  [`SendRefFunctor::send_ref_map`](../../../fp-library/src/classes/send_ref_functor.rs)'s
  closure is `Fn(&A) -> B + Send + 'a` (only `Send`), but
  `ArcRunExplicit::ref_map` requires `Send + Sync` (the substrate
  stores closures in `Arc<dyn Fn + Send + Sync>`).
- `SendRefSemimonad` is blocked by the same pattern.
- `SendRefSemiapplicative` -> `SendRefApplicative` ->
  `SendRefMonad` blanket-derive from these and so are blocked
  transitively.

Trait-tightening alternatives considered:

- **Add `Sync` to the closure**: would resolve the closure-bound
  mismatch on `SendRefFunctor`/`SendRefSemimonad`, but would break
  callers passing `Send`-only closures (e.g.,
  `LazyBrand<ArcLazyConfig>::send_ref_map` users whose closures
  capture non-`Sync` thread-safe values). The asymmetry vs.
  [`SendFunctor::send_map`](../../../fp-library/src/classes/send_functor.rs)
  (which already requires `Send + Sync`) suggests this is a
  legitimate consistency improvement worth pursuing as a separate
  refactor, but is out of 9i's scope and only resolves one of
  four blockers.
- **Add `A: Clone`**: would conceptually violate the ref-family
  contract (`send_ref_map` operates on `&A`, never moving or
  cloning `A`) and force unnecessary bounds on impls that don't
  need them
  ([`ArcLazy::ref_map`](../../../fp-library/src/types/lazy.rs)
  produces `B` from `&A` via `evaluate()` returning `&A`; no `A`
  is moved). Even with this, the per-`A`
  `F::Of<...>: Clone + Send + Sync` HRTBs would remain unresolved.

Neither alternative reaches all four blockers; the per-`A` HRTB
on the suspended layer is the same fundamental gap that no
combination of stable-Rust trait-method bounds can express. This
is the same wall the by-value `SendFunctor` cascade hit on
`ArcFreeExplicitBrand` in 9d.

Action taken:

- Implemented
  [`SendRefPointed for ArcRunExplicitBrand`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
  via `ArcRunExplicit::ref_pure` delegation. Body is a one-liner;
  inline comment block above the impl explains why the broader
  SendRef cascade does not delegate.
- Added a Send-aware Ref-family brand-level coverage table to
  [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)
  enumerating the per-trait blockers (`SendRefPointed` reachable;
  `SendRefFunctor`/`SendRefSemimonad`/cascade blocked) and noting
  the trait-tightening tradeoff that doesn't fully resolve.

User-facing impact: the inherent
[`ArcRunExplicit::ref_map`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
/ `ref_bind` methods carry the per-`A` bounds explicitly in their
where-clauses and remain the by-reference Send-aware surface for
callers operating on the concrete type. The
[`im_do!(ref ArcRunExplicit { ... })`](../../../fp-macros/src/effects/im_do/codegen.rs)
macro form (Phase 2 step 7c) already desugars to these inherent
methods, so user code paths are unaffected by the brand-level
gap.

`ArcRun` (Erased family) has no brand, so its SendRef coverage
stays inherent-method-only via `im_do!(ref ArcRun { ... })` per
the plan; no brand-level work needed there.

Step 9 is now complete with all nine sub-steps landed (or
documented-as-blocked-with-workaround). Step 10 (POC test
migration plus deletion of the `poc-effect-row/` workspace) is
the last remaining Phase 2 step.

### Step 10a: POC migration brings forward 20 of 25 tests; 10b (workspace deletion) deferred for explicit user confirmation

Step 10's plan text says "Migrate the 25 row-canonicalisation
tests from `poc-effect-row/tests/` into
`fp-library/tests/run_row_canonicalisation.rs` as the regression
baseline. Verify all pass under the production types (exercise
both Erased and Explicit Run families). Delete the POC repository
once the migration lands."

Split into two sub-commits because deletion is destructive and
warrants explicit user confirmation independent of the migration:

- **10a (this commit)**: migrate the tests.
- **10b (held)**: delete `poc-effect-row/` workspace once the
  user confirms the migration is acceptable.

The migration brings forward 20 of the POC's 25 tests:

| POC tests                 | Migration outcome                                                                                                                                                                                                                                                                                                                                                                                                             |
| :------------------------ | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `feasibility::t01`-`t07`  | Migrated and adapted to use fp-library's existing brands (`IdentityBrand` / `OptionBrand` / `ResultBrand` / `BoxBrand` / `CatListBrand` / `ThunkBrand` / `SendThunkBrand` / `TryThunkBrand`). 7 tests covering 2-/3-brand canonicalisation across all 6 permutations, lexical-canonical-form check, empty / single-brand edge cases, and same-root-different-params sort.                                                     |
| `feasibility::t08`        | Skipped (does not translate): lifetime-parameter-bearing raw effect type. Production effect brands are zero-sized `'static` markers (no lifetime params at the brand level); the test exercises a property production brands cannot have.                                                                                                                                                                                     |
| `feasibility::t09`-`t09a` | Migrated as runtime-Coproduct subsetter tests with distinct value types so `Member`-style position-by-type inference resolves unambiguously. 2 tests.                                                                                                                                                                                                                                                                         |
| `feasibility::t10`        | Skipped (analog covered): "handler accepts macro output as runtime value". In POC, effect types served as both row brands AND runtime value types, so the macro's emitted row was directly constructable. In production, brand types are zero-sized markers; the analog is the all-six-Run-wrappers integration tests (a row brand drives the wrapper's type parameters).                                                     |
| `feasibility::t11`-`t13`  | Migrated as 5- and 7-brand scaling tests plus 5-element runtime subsetter test. 3 tests.                                                                                                                                                                                                                                                                                                                                      |
| `feasibility::t14`-`t16`  | Skipped (does not test fp-library): `tstr_crates` compile-time string-ordering demos. Adding `tstr` as a dev-dep would not strengthen the regression baseline.                                                                                                                                                                                                                                                                |
| `coyoneda::c01`-`c02`     | Migrated as Coyoneda-wrapping property tests on the production `effects!` macro. 2 tests covered by `effects_two_brands_canonicalise_with_coyoneda_wrap` (matches the wrapping shape and 2-ordering canonicalisation) and `raw_effects_skips_coyoneda_wrap_vs_effects` (effects! vs raw_effects! contrast).                                                                                                                   |
| `coyoneda::c03`-`c05`     | Skipped (does not translate): POC-local `Coyoneda` lift+decoder mechanics. Production `Coyoneda` has no decoder closure (uses brand-Kind machinery directly); production `Coyoneda` has its own unit tests for its lift/map/lower behaviour.                                                                                                                                                                                  |
| `coyoneda::c06`           | Migrated as `subsetter_over_runtime_coyoneda_wrapped_values`: constructs a production `Coyoneda::lift(Identity(7))` runtime value, builds a non-canonical `Coproduct<Coyoneda<OptionBrand, _>, Coproduct<Coyoneda<IdentityBrand, _>, CNil>>`, and runs `.subset()` to recover the canonical permutation. Verifies the IdentityBrand-Coyoneda value lands in position `Inl` post-subset and lowers back to the lifted value 7. |
| `coyoneda::c07`           | Migrated as `effects_generic_brands_canonicalise_with_coyoneda_wrap`. 1 test.                                                                                                                                                                                                                                                                                                                                                 |
| `coyoneda::c08`           | Skipped (analog covered): Coproduct-of-Coyoneda end-to-end fmap dispatch is exercised by the existing [`tests/run_lift.rs`](../../../fp-library/tests/run_lift.rs) round-trip tests on all six Run wrappers. `*Run::lift` desugars to `Free::wrap(F::map(\|a\| Free::pure(a), node))`, which round-trips correctly only if Coproduct's recursive `Functor` impl dispatches to the active variant's `Coyoneda::map`.           |

Plus net-new coverage that wasn't in the POC:

- All 6 permutations of 3 brands (POC tested 3 of 6).
- `effects!` vs `raw_effects!` Coyoneda-wrapping contrast in
  one focused test.
- All six Run wrappers driven by row brands
  (Erased: `Run`/`RcRun`/`ArcRun`; Explicit:
  `RunExplicit`/`RcRunExplicit`/`ArcRunExplicit`). The POC didn't
  have Run wrappers; this addition exercises the
  "row brand drives wrapper" requirement the plan explicitly
  calls out.

Subsetter tests (POC `t09` / `t09a` / `t10` / `t13`) reframed:
the POC's tests conflated brand-level row types with runtime
value types because POC types served both roles. In production
the distinction is real (brand types are zero-sized markers; runtime
values are constructed from concrete types like `Coproduct<i32,
Coproduct<bool, Coproduct<&'static str, CNil>>>`). Three migrated
subsetter tests use distinct value types so `Member`-style
position-by-type inference resolves unambiguously, exercising
`CoproductSubsetter::subset()` on 3- and 5-element runtime
permutations.

Arc-family Run wrapper tests use `ArcCoyonedaBrand`-headed rows
(not bare `CoyonedaBrand`) per the substrate's struct-level
`Of<'static, ArcFree<..., ArcTypeErasedValue>>: Send + Sync`
requirement. This is the same constraint that drove the
per-wrapper Coyoneda-variant pairing rule documented in
[step 9h](#step-9h-per-wrapper-coyoneda-variant-pairing-corrected-to-substrate-pointer-matched).

Deferred for 10b: the destructive `git rm -r poc-effect-row/`.
The POC workspace declares its own `[workspace]` block (detached
from the outer cargo workspace), so deletion is safe but
irreversible. Holding for explicit user confirmation.

### Step 10b: `poc-effect-row/` workspace deleted; Phase 2 complete

`git rm -r poc-effect-row/` removed 8 tracked files; the
~97MB total includes the `target/` cache that was not tracked.
The workspace's `[workspace]` block detached it from the outer
cargo workspace, so the removal had no effect on `just verify`
(no Cargo dependency edges to clean up).

Subsumption ledger (final, post-amendment to step 10a):

- 21 of 25 POC tests directly migrated or covered in
  [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs).
- 4 POC tests skipped with documented rationale: `feasibility::t08`
  (lifetime-parameter-bearing raw effect type, does not translate
  because production brands are zero-sized `'static` markers);
  `feasibility::t14`-`t16` (3 `tstr_crates` compile-time
  string-ordering demos that do not test fp-library).
- 1 POC test (`coyoneda::c08`) implicitly covered by the
  end-to-end round-trip tests in
  [`tests/run_lift.rs`](../../../fp-library/tests/run_lift.rs)
  on all six Run wrappers (lift -> peel -> lower recovers the
  value, which only round-trips correctly if Coproduct's
  recursive Functor impl dispatches to the active variant's
  Coyoneda).
- 1 POC test (`feasibility::t10`) replaced by an analog: the
  all-six-Run-wrappers integration tests, which exercise the
  production analog of "the macro's emitted row drives the
  wrapper's type parameters" (POC's brand-AND-runtime-value
  conflation does not translate; the Run-wrapper integration is
  the production framing).

Documentation maintained:

- The standalone planning doc
  [`docs/plans/effects/poc-effect-row-canonicalisation.md`](poc-effect-row-canonicalisation.md)
  is preserved as research history; deletion of the workspace
  does not invalidate the findings it documents.
- Plan.md `Other artefacts` section updated to past-tense
  (records the deletion); `Reference map` POC validation bullet
  updated; the historical-strategy note about POC-to-production
  migration in the planning section gains a past-tense annotation
  pointing to where the migration actually landed (steps 8 and
  10a) and where the workspace went (step 10b).
- Historical references in step narratives, the research/survey
  sections, and the Phase 2 step text remain as past-tense
  documentation of where the migrated tests came from. They are
  not re-edited.

Phase 2 is now complete (all 10 steps landed). Phase 3
(first-order effect handlers, interpreters, natural
transformations) is the next phase.

## Phase 3: First-order effect handlers, interpreters, natural transformations

### Step 1: `handlers!{...}` macro plus `nt()` builder fallback

Plan text says only:

> `handlers!{...}` macro in `fp-macros/src/effects/handlers.rs`
> producing tuple-of-closures keyed on the row's type-level
> structure. Builder fallback (`nt().on::<E>(handler)...`) as the
> non-macro path ([decisions.md](decisions.md) section 4.6).

The runtime carrier shape for the "tuple-of-closures keyed on the
row's type-level structure" was unspecified. Implementation
choices made (recorded here so step 2's interpreter consumes a
known shape):

- **Runtime carrier is a dedicated cons-list,
  `HandlersNil` / `HandlersCons<H, T>`, in
  [`fp-library/src/types/effects/handlers.rs`](../../../fp-library/src/types/effects/handlers.rs)**,
  with each handler wrapped in a `Handler<E, F>` newtype that pins
  the brand identity at the type level via
  `PhantomData<fn() -> E>`. The cell shape mirrors the row brand
  chain `CoproductBrand<H, T>` / `CNilBrand` cell-for-cell so the
  Phase 3 step 2 interpreter can recurse through both lists in
  lock-step (handler `head` matches row brand head; handler `tail`
  recurses into row brand tail). Closure type `F` stays fully
  generic; step 2 will pin the concrete shape via an interpreter
  trait bound.
- **Distinct types from `frunk_core`'s `HCons`/`HNil`.** The
  coproduct adapter
  ([`fp-library/src/types/effects/coproduct.rs`](../../../fp-library/src/types/effects/coproduct.rs))
  already re-exports `frunk_core::hlist::{HCons, HNil}` for the
  row-encoding indexing machinery (`Here` / `There`,
  `CoprodInjector`, etc.). Reusing the same types for the handler
  carrier would conflate two distinct roles (type-level position
  proofs vs runtime closure carriers) and prevent inherent-method
  dispatch on the handler-list types (the `.on()` builder method
  needs to live on the list types directly, which can't be done on
  foreign types without an extension trait dance). Rolling our own
  `HandlersNil`/`HandlersCons` keeps the intent visible at call
  sites and lets `.on()` be inherent.
- **Builder uses prepend semantics; macro sorts.** `nt()` returns
  `HandlersNil`; `.on::<EBrand, F>(self, handler)` on either
  `HandlersNil` or `HandlersCons<H, T>` returns a new
  `HandlersCons<Handler<E, F>, Self>` (i.e., the new handler is at
  the head). Chained `.on()` calls therefore produce a list whose
  head is the most-recently-added handler. Users wanting builder
  output to match the macro's lexical-canonical order call
  `.on()` in reverse-lexical order. Documented under the
  module-level "Builder ordering" section in
  [`handlers.rs`](../../../fp-library/src/types/effects/handlers.rs)
  and in the `handlers!` macro doc-comment in
  [`fp-macros/src/lib.rs`](../../../fp-macros/src/lib.rs). The
  macro takes the user-provided list, sorts entries lexically by
  `quote!(brand).to_string()` (matching `effects!`'s sort key
  exactly via the same `quote::quote` stringification), and emits
  the cons chain in canonical order so the macro and builder paths
  produce structurally-identical values when fed equivalent
  inputs.
- **Macro-side worker lives at
  [`fp-macros/src/effects/handlers.rs`](../../../fp-macros/src/effects/handlers.rs)
  next to
  [`effects_macro.rs`](../../../fp-macros/src/effects/effects_macro.rs)**
  and follows the same shape: a `*_worker` function returning
  `syn::Result<TokenStream>`, plus a thin `#[proc_macro] pub fn
handlers` entry-point in
  [`fp-macros/src/lib.rs`](../../../fp-macros/src/lib.rs). The
  shared lexical-sort helper in
  [`row_sort.rs`](../../../fp-macros/src/effects/row_sort.rs) is
  not reused: `row_sort.rs`'s `parse_and_sort_types` parses
  `Punctuated<Type, Token![,]>`, but `handlers!` parses
  `Punctuated<HandlerEntry, Token![,]>` where each `HandlerEntry`
  is `Type: Expr`. Inlining the small parse-then-sort-by-stringified-brand
  loop is cheaper than refactoring `row_sort` into a generic
  helper that takes both a parser and a key-extractor. If a future
  macro (e.g., `scoped_effects!` or its handler-side analog) ends
  up needing the same key-extraction shape, the helper can be
  generalised then; one duplicate sort loop is below the threshold
  that justifies the abstraction now.
- **`Handler<E, F>` uses `PhantomData<fn() -> E>` rather than
  `PhantomData<E>`.** The `fn() -> E` form keeps `Handler<E, F>`
  free of variance and `Send`/`Sync` concerns inherited from `E`
  itself, which matters because `E` is a row brand (typically a
  zero-sized marker type) used purely as a type-level tag. The
  variance-free form is the standard "phantom for tagging" idiom
  in Rust ecosystem code (e.g., `std::marker::PhantomPinned` ships
  this exact shape).
- **Re-export pattern.** Per the optics A+B hybrid (decisions.md
  section 4.4 resolution), the handler types are re-exported at
  the subsystem-scope `crate::types::effects::*`
  (`Handler` / `HandlersCons` / `HandlersNil` / `nt`) but not
  promoted to the top-level `crate::types::*`. The Run wrappers
  hold the headline-types tier; the handler-list machinery is a
  supporting detail of the effects subsystem.
- **No re-export of `handlers!` through
  `fp_library::__internal`.** Unlike `raw_effects!` (which is
  internal-only and re-exported through `__internal` so the call
  site signals "fp-library-internal use"), `handlers!` is
  user-facing and re-exported through the standard
  `pub use fp_macros::*` path in
  [`fp-library/src/lib.rs`](../../../fp-library/src/lib.rs). The
  builder fallback is also user-facing (no `__internal`
  marker).

What landed in this commit:

- New file: [`fp-library/src/types/effects/handlers.rs`](../../../fp-library/src/types/effects/handlers.rs)
  with `Handler<E, F>` newtype, `HandlersNil`, `HandlersCons<H,
T>`, the `.on::<E, F>(...)` inherent builder methods on both
  list types, the `nt()` entry-point function, and 6 inline unit
  tests covering builder semantics and struct-literal
  construction.
- New file: [`fp-macros/src/effects/handlers.rs`](../../../fp-macros/src/effects/handlers.rs)
  with the `HandlerEntry` parser, the `handlers_worker` function,
  and 6 token-string assertion tests covering empty input, single
  entry, two-entry canonical-ordering equivalence, lexical-sort
  head ordering, generic brand parameters, and trailing-comma
  acceptance.
- New file: [`fp-library/tests/handlers_macro.rs`](../../../fp-library/tests/handlers_macro.rs)
  with 10 integration tests exercising the macro and builder
  end-to-end (canonical-shape equivalence between macro and
  builder for aligned input, handler-closure invocation through
  the head/tail chain, three-entry sort, brand pinning,
  trailing-comma acceptance, builder prepend semantics).
- Wiring: `pub mod handlers;` added to
  [`fp-macros/src/effects.rs`](../../../fp-macros/src/effects.rs);
  `handlers::handlers_worker` import and `#[proc_macro] pub fn
handlers(...)` entry-point added to
  [`fp-macros/src/lib.rs`](../../../fp-macros/src/lib.rs);
  `pub mod handlers;` plus
  `pub use handlers::{Handler, HandlersCons, HandlersNil, nt}`
  added to
  [`fp-library/src/types/effects.rs`](../../../fp-library/src/types/effects.rs).

Verification: `just verify` clean (fmt, check, clippy with
`-D warnings`, deny, doc, test). 2437 unit tests pass; 10
integration tests added; 6 worker tests added.

Open follow-ups for step 2:

- The interpreter trait will pin the closure shape `F` per
  effect (e.g., a closure of shape
  `FnMut(Coyoneda<E, X>) -> ...interpreter target...`). Step 1
  cannot pin `F` because the interpreter target type isn't
  decided yet (step 2's `interpret`/`run`/`runAccum` family will
  determine it).
- Negative-case `compile_fail` UI tests (handler missing for an
  effect, wrong type ascription, etc.) ship in step 6, not step
  1, per the plan's step partition.

### Step 2: `interpret` / `run` / `run_accum` recursive-target interpreter family

Plan text reads:

> `interpret` / `run` / `runAccum` recursive-target interpreter
> family in `fp-library/src/types/effects/interpreter.rs`.

Implementation choices made (recorded so step 3's
`MonadRec`-target family can mirror the same shape):

- **`DispatchHandlers<'a, Layer, NextProgram>` trait at
  [`fp-library/src/types/effects/interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs).**
  Walks a [`HandlersCons`](../../../fp-library/src/types/effects/handlers.rs) /
  [`HandlersNil`](../../../fp-library/src/types/effects/handlers.rs)
  in lock-step with the row's value-level
  [`Coproduct`](../../../fp-library/src/types/effects/coproduct.rs)
  chain. Three `HandlersCons<Handler<EBrand, F>, T>` impls cover
  one Coyoneda variant each
  ([`Coyoneda`](../../../fp-library/src/types/coyoneda.rs),
  [`RcCoyoneda`](../../../fp-library/src/types/rc_coyoneda.rs),
  [`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs))
  because the per-wrapper Coyoneda variant pairing rule (from
  Phase 2 step 9h) means each Run wrapper's row has a different
  Coyoneda type at the value level. The duplication is mechanical:
  identical body, different `lower*` method (bare `Coyoneda::lower`
  takes `self`; the Rc/Arc variants ship `lower_ref(&self)` only).
  Arc variant adds `Send + Sync` bounds and `Functor + SendFunctor`.
- **Mono-in-`A` step-function shape, matching PureScript Run's
  runtime model.** Per [`purescript-run/src/Run.purs:184-217`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs#L178-L217),
  `interpret = run` and the implemented form's handler is
  `(VariantF r (Run r a) -> m (Run r a))` -- mono in `a`. The
  Rust port adopts this directly: handler closures are mono in
  `A`, with `A` flowing in from the program's result type.
  Handler-list values are specialized to one program's `A`. A
  rank-2-polymorphic alternative via the existing
  [`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)
  trait is documented in plan.md's Phase 6+ deferred-items
  section as a future companion entry-point.
- **Per-wrapper inherent methods.** Each of the six Run wrappers
  ships inherent `interpret` / `run` / `run_accum` methods.
  `run` is a thin alias for `interpret` per PureScript Run's
  `interpret = run`. `run_accum` accepts an `init` state value
  and threads state via closure captures (`Rc<RefCell<S>>` for
  single-threaded substrates, `Arc<Mutex<S>>` for ArcRun /
  ArcRunExplicit); state is ephemeral to the loop, mirroring
  PureScript Run's `runAccum` shape that returns `m a` (not
  `m (s, a)`). Threading via captures rather than a separate
  stateful trait avoids doubling the trait machinery.
- **Loop body and `Node::First` dispatch.** Each method's loop
  is a `match prog.peel() { Ok(a) => return a, Err(node) => ...
}`. For Run / RcRun / RunExplicit / RcRunExplicit /
  ArcRunExplicit, the `Err` arm pattern-matches `Node::First` /
  `Node::Scoped` directly. For `ArcRun`, the same pattern fails
  GAT normalization under the struct-level HRTB (the same wall
  documented for `ArcRun::send` in Phase 2 step 5's
  resolutions.md entry). The workaround mirrors the `lift_node`
  precedent: a free function `unwrap_first<R, S, A>` defined
  outside the impl-block scope receives the `Node`-projection
  value, pattern-matches inside its non-HRTB scope, and returns
  the first-order layer payload. `ArcRun::interpret` calls
  `unwrap_first` and then dispatches. `ArcRunExplicit` does NOT
  hit this wall (its struct-level bounds don't include the
  `Of: Send + Sync` HRTB; the `Send + Sync` cascade comes
  through per-method bounds), so it pattern-matches inline.
- **`Scoped` arm panics with `unreachable!`.** Phase 3 first-order
  interpretation does not route scoped layers; the `Node::Scoped`
  arm panics with a descriptive message. This is gated by
  `#[expect(clippy::unreachable, reason = "...")]` per method
  (Run / RcRun / RunExplicit / RcRunExplicit / ArcRunExplicit /
  ArcRun's `unwrap_first` helper). Phase 4 will route scoped
  layers, replacing the panic with real dispatch.
- **Inner brand as the handler-list key.** The `handlers!` macro
  uses the inner effect brand (`IdentityBrand`, `StateBrand`,
  etc.) as the handler key for ALL wrappers, matching `effects!`'s
  sort key. The DispatchHandlers impls bind the inner brand to
  `EBrand` and pattern-match on the relevant Coyoneda value
  variant; users don't need to know which Coyoneda wrapper is
  in use at the row level.
- **`#[document_examples]` doctests use the per-wrapper Coyoneda-variant
  brand for the row.** `Run` / `RunExplicit` doctests use
  `CoyonedaBrand<IdentityBrand>` for the row; `RcRun` /
  `RcRunExplicit` use `RcCoyonedaBrand<IdentityBrand>`;
  `ArcRun` / `ArcRunExplicit` use `ArcCoyonedaBrand<IdentityBrand>`.
  Per the per-wrapper Coyoneda variant pairing rule.

What landed in this commit:

- New file: [`fp-library/src/types/effects/interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs)
  with the `DispatchHandlers` trait and four impls
  (`HandlersNil`/`CNil` base case; one cons-cell impl per Coyoneda
  variant).
- Wiring: `pub mod interpreter` and `pub use interpreter::DispatchHandlers`
  in
  [`fp-library/src/types/effects.rs`](../../../fp-library/src/types/effects.rs).
- Inherent methods on six Run wrappers for `interpret`, `run`,
  `run_accum`. ArcRun gains the `unwrap_first` HRTB-poisoning
  workaround helper.
- New file: [`fp-library/tests/run_interpret.rs`](../../../fp-library/tests/run_interpret.rs)
  with 12 integration tests across all six wrappers.
- Plan.md Phase 6+ deferred-items gains an `interpret_nt` entry
  for a future
  [`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)-based
  companion entry-point.

Verification: `just verify` clean. 2456 unit tests; 12 integration
tests added; doctests across the methods compile and pass.

Open follow-ups for step 3 (`MonadRec`-target family):

- Step 3's `interpret_rec` / `run_rec` / `run_accum_rec` mirror
  this step's per-wrapper inherent-method layout but route
  through `MonadRec`'s `tail_rec_m` instead of host-stack
  recursion. The `DispatchHandlers` trait reuses unchanged;
  only the loop body changes.
- The Phase 6+ `interpret_nt` companion entry-point remains
  deferred unless the closure-mono-in-A constraint blocks a
  real user need.

### Step 3: pipeline row-narrowing `interpret_with::<EBrand>` plus empty-row terminal `extract`

Pipeline-narrowing per-wrapper inherent
`interpret_with::<EBrand, Idx, RMinusE>(handler) -> Wrapper<RMinusE, S, A>`
plus empty-dual-row terminal `extract(self) -> A` shipped across
all six Run wrappers.

What the plan called for, and what diverged:

- **No `DispatchOneHandler` trait shipped.** The plan text under
  Phase 3 step 3 (and the resolution doc's
  "Heftia row architecture clarification" / "What ships under the
  resolution" subsections) anticipated a parallel
  `DispatchOneHandler<'a, Layer, NextProgram>` trait beside the
  step 2 `DispatchHandlers` trait. Three design shapes were
  considered:
  1. A trait keyed on the Coyoneda variant carrying just the
     `lower` step (parallel to step 2's three impls). Body would
     be one line per impl (`handler(self.lower())` /
     `handler(self.lower_ref())`).
  2. A trait keyed on the Coproduct chain shape with three
     blanket impls walking the chain via `Member::project`.
     Coherence requires marker disambiguation (`DispatchOne`
     phantom token) and the body is essentially
     `match Member::project(layer) { Ok(coyo) => ..., Err(rest) => ... }`.
  3. Inline dispatch in each wrapper's `interpret_with` body
     using `Member::project` plus the per-Coyoneda-variant
     `lower` / `lower_ref` choice.
     Approaches 1 and 2 were rejected as over-engineered: each Run
     wrapper already statically pairs with a single Coyoneda variant
     per the per-wrapper pairing rule from Phase 2 step 9h, so the
     abstraction enables no shared code paths. Approach 3 (inline)
     wins on readability without losing capability. The
     `DispatchOneHandler` name is preserved in the resolution doc as
     the design vocabulary; the runtime artifact is the inherent
     method body.
- **Panic-free `extract` via empty-dual-row tightening.** The
  plan text described `extract(self) -> A` for "empty-row Run".
  Initial implementation tightened only the first-order row
  (`R = CNilBrand`) and panicked on the `Node::Scoped` arm via
  `unreachable!`, mirroring step 2's `interpret` panic. User
  review pushed for a no-panic-by-construction shape: tighten
  the where-bound to BOTH rows (`R = CNilBrand AND S = CNilBrand`).
  Both arms then carry uninhabited
  [`CNil`](../../../fp-library/src/types/effects/coproduct.rs)
  payloads; the body's exhaustive `match cnil {}` on each side
  diverges to `!`, statically proving no runtime panic. The
  trade-off: `extract` is no longer callable on programs whose
  first-order row is empty but whose scoped row carries layers.
  Phase 4 will introduce a separate elimination operation for
  non-empty scoped rows; the design intent is that `extract`'s
  remit stays "fully-pure programs only".
- **Three new `ArcRun` HRTB-free helpers.** ArcRun's struct-level
  HRTB (`<NodeBrand<R, S> as Kind>::Of<'static, ArcFree<...>>: Send + Sync`)
  poisons GAT normalization in scope. Implementing
  `interpret_with` and `extract` cleanly required three sibling
  helpers parallel to Phase 2 step 5's `lift_node` and
  `unwrap_first` precedents:
  - [`make_node_first<R, S, A>`](../../../fp-library/src/types/effects/arc_run.rs):
    HRTB-free helper that constructs a `Node::First` projection.
    Used by `interpret_with`'s unmatched arm to build the layer
    outside the caller's HRTB scope.
  - [`wrap_first_arc<RMinusE, S, A>`](../../../fp-library/src/types/effects/arc_run.rs):
    HRTB-poisoning workaround for the
    [`ArcFree::wrap`](../../../fp-library/src/types/arc_free.rs)
    call. Receives an already-built `Node` projection (constructed
    by `make_node_first`) and forwards it to `ArcFree::wrap`. The
    function body therefore performs no GAT projection
    construction inside its own HRTB-bearing scope, so the
    projection equality declared by
    [`impl_kind!`](../../../fp-macros/src/lib.rs) normalizes
    cleanly.
  - [`unwrap_pure_node<Inner, Ret>`](../../../fp-library/src/types/effects/arc_run.rs):
    HRTB-free helper that statically eliminates a `Node` over an
    empty dual row. Both `Node` arms carry uninhabited `CNil`
    payloads, so the match diverges to `!`, which coerces to the
    caller's `Ret` without panic. Two type parameters: `Inner`
    is the inner-program type stored in the `Node`'s continuation
    slot; `Ret` is the caller's expected return type. Used by
    `ArcRun::extract`.
    Only `ArcRun` (not `ArcRunExplicit`) needs these helpers
    because the Explicit variants carry the `Send + Sync` bounds
    per-method rather than at the struct level, so their bodies
    pattern-match `Node` literals inline successfully. The other
    five Run wrappers (Run, RunExplicit, RcRun, RcRunExplicit,
    ArcRunExplicit) all pattern-match inline.
- **Handler bound: `impl Fn + Clone + 'static` (plus `Send + Sync` on Arc).**
  The recursive narrowing in both arms requires cloning the
  handler before invoking it inside the
  [`Functor::map`](../../../fp-library/src/classes/functor.rs)
  closure that walks each inner sub-program. Each `Functor::map`
  invocation may call its closure multiple times (once per inner
  slot in the layer's content); the closure clones the handler
  per call. For typical Identity-shaped effects this is one
  clone per peeled chain link; for closure-shaped effects
  (e.g., `State<S>`) the recursive interpret_with is deferred
  inside the new state function. Users who want cheap-to-clone
  handlers can wrap captured state in `Rc` / `Arc`. The same
  approach is used internally by step 2's
  `run_accum`-via-closure-capture pattern.
- **Recursion is host-stack-frame per peeled layer.** Unlike
  step 2's `interpret` which uses a `loop`, step 3's
  `interpret_with` recurses structurally via `Functor::map`
  inside each layer's body. Per the
  [WrapDrop probe](../../../fp-library/tests/run_wrap_depth_probe.rs),
  Run-typical patterns have structural depth at most 1, but the
  CHAIN depth (number of peel-able layers) is not bounded. Programs
  with deep chains of eager-recursing effects (Identity-shaped)
  can blow the host stack. Phase 3 step 4 will provide the
  stack-safe alternative via `tail_rec_m` for external `MBrand`
  targets. Per the resolutions doc this trade-off is intentional:
  step 3's pipeline shape uniquely enables partial interpretation
  / handler ordering / compositional handler libraries; step 4
  is the stack-safe extraction sibling.
- **Per-wrapper Coyoneda variant pairing.** The Member bound
  uses the wrapper's matching Coyoneda variant: `Coyoneda` for
  Run / RunExplicit; `RcCoyoneda` for RcRun / RcRunExplicit;
  `ArcCoyoneda` for ArcRun / ArcRunExplicit. Matched-arm dispatch
  uses the corresponding `lower` (consume) or `lower_ref` (clone)
  per the per-wrapper `peel`/`lift` substrate constraints
  established in Phase 2 step 9h.
- **Lint expectation cleanup on ArcRun.** `ArcRun::interpret_with`
  initially carried `#[expect(clippy::unreachable, ...)]` from
  the symmetric copy with the other wrappers. ArcRun routes its
  layer extraction through `unwrap_first` (which has its own
  panic on `Node::Scoped`); the outer `interpret_with`'s body
  has no inline `unreachable!`. The expectation was unfulfilled
  and removed.

What landed in this commit:

- New per-wrapper inherent methods on
  [Run](../../../fp-library/src/types/effects/run.rs),
  [RunExplicit](../../../fp-library/src/types/effects/run_explicit.rs),
  [RcRun](../../../fp-library/src/types/effects/rc_run.rs),
  [RcRunExplicit](../../../fp-library/src/types/effects/rc_run_explicit.rs),
  [ArcRun](../../../fp-library/src/types/effects/arc_run.rs),
  [ArcRunExplicit](../../../fp-library/src/types/effects/arc_run_explicit.rs):
  `interpret_with::<EBrand, Idx, RMinusE>(handler)` for pipeline
  row-narrowing.
- New per-wrapper inherent methods (in separate impl blocks
  scoped to `Wrapper<CNilBrand, CNilBrand, A>`): `extract(self) -> A`
  for empty-dual-row terminal extraction. Panic-free by
  construction via exhaustive match on the uninhabited `CNil`
  payloads.
- New helpers in
  [`arc_run.rs`](../../../fp-library/src/types/effects/arc_run.rs):
  `make_node_first`, `wrap_first_arc`, `unwrap_pure_node`. All
  `#[doc(hidden)]`. Sibling to the existing `lift_node` and
  `unwrap_first`.
- New file: [`fp-library/tests/run_interpret_with.rs`](../../../fp-library/tests/run_interpret_with.rs)
  with 16 integration tests across all six wrappers
  (single-effect narrow-and-extract, bind-chain
  narrow-and-extract, pure-program extract).

Verification: `just verify` clean. 2471 unit tests; 16 new
integration tests added; doctests across the `interpret_with` and
`extract` methods on all six wrappers compile and pass.

Open follow-ups for step 4 (`MonadRec`-target family):

- Step 4's `interpret_rec` / `run_rec` / `run_accum_rec` mirror
  step 2's per-wrapper inherent-method layout (M-extraction;
  uses the existing `DispatchHandlers` trait) but route through
  `MonadRec`'s `tail_rec_m` instead of a host-stack `while` loop.
  The trait reuses unchanged.
- Step 5's first-order effect smart constructors (`State`,
  `Reader`, `Except`, `Writer`, `Choose`) plug into both step 2's
  all-handlers-at-once `interpret` and step 3's pipeline
  `interpret_with`; the chain-and-extract idiom
  (`prog.interpret_with::<E1>(...).interpret_with::<E2>(...).extract()`)
  is the typical user-facing surface for narrowing programs to
  pure values.

#### M3C: parameterise `interpret_with` over `P: RefCountedPointer` (drops `Clone` bound)

Per the
[2026-05-03 reversal resolution](resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer)
M3C, the user-facing handler bound on `interpret_with` drops
from `Fn + Clone + 'static` (plus `Send + Sync` on Arc) to
`Fn + 'static` (plus `Send + Sync` on Arc). Each wrapper's
public outer method wraps the user handler in
`<P as RefCountedPointer>::Of<'_, F>` once at entry and
delegates to a private inner `interpret_with_shared` that
takes the wrapped pointer by value; recursive narrowing clones
the pointer (refcount bump) instead of the underlying closure.
This permits handlers that capture move-only resources (e.g., a
`BufWriter`) without users having to wrap their captures in
`Rc<RefCell<_>>` themselves.

What diverged from the plan text:

- **Public-outer + private-inner method split.** The original
  step 3 plan text described one `interpret_with` method per
  wrapper with the recursion inline. M3C introduces a private
  `interpret_with_shared::<EBrand, Idx, RMinusE, F>` companion
  on each wrapper. The outer method has the same generic
  parameters as before (`<EBrand, Idx, RMinusE>`, with `F`
  hidden behind `impl Fn`); the inner method names `F`
  explicitly so the recursion can pass `<P as RefCountedPointer>::Of<'_, F>`
  through each call. Users only ever call the public outer; the
  private inner is implementation detail.
- **Pointer brand fixed per wrapper, not user-visible.** The
  resolution proposed parameterising over `P: RefCountedPointer`
  in the abstract; the implementation fixes `P` per wrapper.
  `Run`, `RunExplicit`, `RcRun`, `RcRunExplicit` thread
  [`RcBrand`](../../../fp-library/src/brands.rs);
  `ArcRun`, `ArcRunExplicit` thread
  [`ArcBrand`](../../../fp-library/src/brands.rs). User call
  sites see no extra turbofish parameter.
- **`(*handler)(mapped)` deref-call invocation pattern.** The
  inner method invokes the wrapped handler via the
  `Deref<Target = F>` projection from
  [`RefCountedPointer::Of<'_, T>`](../../../fp-library/src/classes/ref_counted_pointer.rs).
  `Rc<F>: Fn` does not hold for arbitrary `F: Fn`; the call
  goes through `*handler` (a place of type `F`) which then
  dispatches via `Fn::call(&*handler, args)`.
- **Doc-attribute requirement on the private inner method.**
  The `#[document_module]` macro validates all impl-block
  methods regardless of visibility; private methods are not
  exempt. Each `interpret_with_shared` carries full
  `#[document_signature]` / `#[document_type_parameters]` /
  `#[document_parameters]` / `#[document_returns]` /
  `#[document_examples]` attributes. The example exercises
  the method indirectly through the public `interpret_with`
  (which delegates to `interpret_with_shared`), since the
  inner method is private and not user-callable.
  `#[doc(hidden)]` was considered as an opt-out but the macro
  still validates; `#[allow(deprecated)]` on the method does
  not suppress the macro's emitted warnings (the warning span
  comes from impl-block-level macro-emitted code, not the
  method itself).
- **Arc family bounds keep `Send + Sync` on `F`.** The Arc
  inner method's `F` bound retains `Send + Sync + 'static`
  (or `+ 'a` for `ArcRunExplicit`) so that the wrapped
  `Arc<F>` is itself `Send + Sync` (`Arc<T>: Send + Sync`
  requires `T: Send + Sync`); the move-closure that captures
  the wrapped pointer for `SendFunctor::send_map` then
  satisfies that combinator's `Send + Sync` requirement on
  its closure argument.
- **Three-commit split.** The reversal resolution's land-order
  prescribed F1D + F3A + M3C as a single combined commit.
  Implementation split into three focused commits per finding
  (`05be270` F1D, `f8031c5` F3A, this commit M3C). Each
  reversal stands alone in review and `just verify` is clean
  at every commit boundary; the total diff is the same.

### Step 4: MonadRec-target interpreter family (`interpret_rec` / `run_rec` / `run_accum_rec`)

Per-wrapper inherent
`interpret_rec::<MBrand>(handlers) -> M::Of<A>` (plus the
`run_rec` alias and the stateful `run_accum_rec::<MBrand, St>`)
shipped across all six Run wrappers, driven by
[`tail_rec_m`](../../../fp-library/src/classes/monad_rec.rs) for
stack-safety on external `MBrand: MonadRec` targets. Locked-in
design from the
[2026-05-02 resolution](resolutions.md#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading)
((Q1 = A) mirror PureScript handler shape; (Q2 = A1) relax
`DispatchHandlers::dispatch` to `&self` + `Fn`; (Q3 = A) closure-
capture state threading).

What landed across two commits:

1. **Commit 1: trait relaxation refactor (`bd540d5`).**
   - [`DispatchHandlers::dispatch`](../../../fp-library/src/types/effects/interpreter.rs)
     `&mut self` -> `&self` in trait def and all four impls
     (`HandlersNil` base case + three Coyoneda-variant cons-cell
     impls).
   - `Handler::F: FnMut` -> `F: Fn` in all three Coyoneda-variant
     cons-cell impls.
   - Dropped the now-unused `mut` binding from the existing
     `interpret`/`run`/`run_accum` method signatures on all six
     wrappers (mechanical sed substitution).
   - Module-doc paragraph in
     [`interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs)
     explains why `&self` (callable from `Fn` closures like
     `tail_rec_m`'s step closure; mutation pushed to interior
     mutability at the user level via `Rc<RefCell<_>>` /
     `Arc<Mutex<_>>` captures).
2. **Commit 2: rec interpreter family (this commit).** Per-wrapper
   inherent `interpret_rec` / `run_rec` / `run_accum_rec` methods
   on all six wrappers using the relaxed `DispatchHandlers` trait
   with `NextProgram = M::Of<Run<R, S, A>>`.

What the plan called for, and what diverged:

- **M-lifetime pinning.** The plan text and the 2026-05-02
  resolution did not anticipate that the dispatch trait's
  `for<'h>` HRTB over `M::Of<'h, Run<R, S, A>>` would fail when
  M's GAT carries `'h` in non-reference position (e.g.,
  `Thunk<'h, T> = Box<dyn FnOnce() -> T + 'h>`). Stable Rust
  closures cannot be HRTB-polymorphic over types with a lifetime
  in non-reference position, so user handlers like
  `|op: Identity<Thunk<'h, ...>>| ...` cannot satisfy
  `for<'h>`. The fix: pin M's lifetime parameter at the
  per-wrapper natural choice (`'static` for the Erased family
  whose struct is `'static`-only; `'a` for the Explicit family
  whose struct already carries `'a`). The `for<'h>` quantifier
  remains on the row brand's projection (`<R as Kind>::Of<'h, ...>`)
  only. This is a concrete instance of the stable-Rust HRTB-over-
  types limit that has appeared elsewhere in fp-library
  (per-`A` Clone bounds, brand-level `Send + Sync` cascade); the
  workaround is principled rather than a one-off.
- **Arc family: M-`Of<...>: Send + Sync` bound.** `ArcRun` and
  `ArcRunExplicit` use [`SendFunctor::send_map`](../../../fp-library/src/classes/send_functor.rs)
  to lift inner programs to M-wrapped, which requires the
  M-wrapped continuation type to be `Send + Sync`. So both Arc
  wrappers add a `<MBrand as Kind>::Of<'static, ArcRun<R, S, A>>: Send + Sync`
  (or `'a` for ArcRunExplicit) where-clause bound on top of their
  existing per-`peel` cascade.
- **Arc family: ThunkBrand doctest limitation.** `Thunk<'a, T>`
  is `Box<dyn FnOnce() -> T + 'a>`; the trait object does not
  carry `Send + Sync` by default. So the Arc wrappers'
  `Of<'_, ArcRun<...>>: Send + Sync` bound rules out
  `MBrand = ThunkBrand`. The Arc family's rec doctests use
  `OptionBrand` instead. This is a fundamental property of
  `Thunk`'s representation, not a limitation of the rec
  interpreter design; users who want stack-safe + thread-safe
  interpretation should use a Send + Sync M brand or wait for a
  hypothetical `SendThunk` brand.
- **`ArcRun` HRTB-poisoning workaround reused.** `ArcRun`'s
  `interpret_rec` body extracts `Node::First` payloads via
  [`unwrap_first`](../../../fp-library/src/types/effects/arc_run.rs)
  (the existing helper from Phase 2 step 5 / Phase 3 step 2),
  not via inline pattern matching, because the struct's
  `Send + Sync` HRTB poisons GAT normalization on `Node` literals
  in scope. No new helpers required (unlike step 3 which added
  three; step 4's loop returns `M::Of<A>` directly so it does
  not construct narrowed Run programs in scope).
- **Two-commit split.** The trait relaxation landed as a
  separate `refactor(effects):` commit ahead of the new methods
  per the resolutions doc's per-commit breakdown. The mechanical
  refactor verifies cleanly on its own (1361+ existing tests
  unaffected); landing them together would have made the diff
  hard to review.
- **`DispatchHandlers` trait reuse.** Per (5.B) the existing
  trait is reused unchanged with `NextProgram` instantiated as
  `M::Of<Run<R, S, A>>` (in place of `Run<R, S, A>` for step 2).
  No parallel `DispatchHandlersRec` trait was needed.
- **State threading via closure captures.** `run_accum_rec`'s
  `init: St` parameter is moved into the user's chosen state
  cell at the call site (`Rc<RefCell<St>>` for non-Arc;
  `Arc<Mutex<St>>` for Arc). Parity with step 2's `run_accum`;
  state-via-`StateT` deferred to Phase 6+.

What landed in this commit:

- Eight new public methods per wrapper x 6 wrappers = 18 new
  inherent methods total (3 per wrapper:
  `interpret_rec` / `run_rec` / `run_accum_rec`).
- New file: [`fp-library/tests/run_interpret_rec.rs`](../../../fp-library/tests/run_interpret_rec.rs)
  with 18 integration tests across all six wrappers (Erased
  non-Arc + ThunkBrand for stack-safety; all six wrappers +
  OptionBrand for short-circuit; per-wrapper `run_accum_rec`
  state threading).
- New imports across the six wrapper files: `MonadRec`,
  `Pointed` (or `SendPointed` for Arc Explicit), `tail_rec_m`,
  `core::ops::ControlFlow`.
- Doctests on each rec method exercise the canonical-row-and-
  handler combination. Arc wrappers' doctests use `OptionBrand`
  (Thunk is not Send + Sync); non-Arc wrappers' doctests use
  `ThunkBrand` to exercise the stack-safety story.

Verification: `just verify` clean. 2489 unit tests; 18 new
integration tests; doctests across the rec methods on all six
wrappers compile and pass.

Open follow-ups:

- Phase 3 step 5 (standard first-order effect smart constructors:
  `State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`, `Choose`)
  is the next work; these plug into all three interpreter
  primitives (steps 2's all-handlers, 3's pipeline, 4's
  MonadRec-target).
- The Phase 6+ `interpret_nt` companion entry-point remains
  deferred unless the closure-mono-in-A constraint blocks a
  real user need.
- Future migration path for `ArcRun`: if Phase 4 / 5 push the
  HRTB-free helper count past ~7-8, consider migrating
  `ArcRun` to `ArcRunExplicit`'s per-method-bound shape (the
  `Send + Sync` bounds move from the struct to each method).
  Cost: bound restatement; benefit: helper proliferation
  eliminated. Currently provisional; revisit if helper count
  grows.

### Step 5a.1: State effect type machinery

`StateBrand<P, S>` registration + `State<'a, P, S, A>` enum

- `Functor` impl shipped per the
  [2026-05-03 resolution](resolutions.md#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization)
  ((1.b) + (2.a) + (3.a-1)).

What landed:

- [`StateBrand<P, S>`](../../../fp-library/src/brands.rs)
  registration in `brands.rs`, parameterised by
  `P: ToDynCloneFn` (the pointer brand for the stored
  continuations) and `S: 'static`.
- [`fp-library/src/types/effects/state.rs`](../../../fp-library/src/types/effects/state.rs)
  with `State<'a, P, S, A>`:
  - `Get(<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A>)`
  - `Put(S, <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>)`
- `Functor` impl on `StateBrand<P, S>` that composes the user-
  supplied `f: A -> B` with each variant's stored continuation
  via [`<P as ToDynCloneFn>::new(closure)`](../../../fp-library/src/classes/to_dyn_clone_fn.rs).
- `impl_kind!` registration connecting `StateBrand<P, S>` to
  the `Of<'a, A>: 'a = State<'a, P, S, A>` projection.
- Per-method doctest exercising the Functor instance with
  `RcBrand`.

What the plan called for, and what diverged:

- **`SendFunctor` impl deferred.** Per the locked-in design
  resolution, `StateBrand<ArcBrand, S>` should support the
  Arc family. Implementing this requires the bound
  `<ArcBrand as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
  per-`A`, which hits stable Rust's HRTB-over-types limit (the
  same wall that blocked brand-level `SendFunctor` on
  `ArcFreeExplicitBrand` in Phase 2 step 9d / 9g / 9i). The
  impl is deferred with a code comment in `state.rs`; an
  active blocker in plan.md tracks the open design decision
  for how to ship Arc family smart constructors. See the
  [2026-05-03 active blocker](plan.md#active-blockers).
- **All wrappers use FnBrand-wrapped continuations**, not just
  the multi-shot ones. Single-shot wrappers (Run, RunExplicit)
  accept the small Rc-allocation cost of going through
  `<RcBrand as ToDynCloneFn>::new` rather than `Box::new`. This
  unifies the type surface (one `State<'a, P, S, A>` for all
  wrappers); the alternative of two State types (one for
  single-shot with `Box<dyn FnOnce>`, one for multi-shot with
  `Rc<dyn Fn>`) was rejected as over-fragmenting the API.
- **`S: 'static` simplification.** The state type is bounded
  `'static` at the brand level; users with non-`'static` state
  (e.g., state holding borrowed data) cannot use `StateBrand`
  directly. Lifetime-parameterised state is deferred to Phase
  6+ if user demand surfaces.

Verification: `just verify` clean. 2490 unit tests + 1 new
doctest on the Functor impl compile and pass.

### Step 5a.2: Run::get and Run::put smart constructors

Run-only smart constructors for State, the first per-wrapper
variant in the rollout per the resolution's (1.b) decision.

What landed:

- `Run::get<Idx>() -> Run<R, ScopedRow, A>` in the existing
  `impl<R, ScopedRow, A> Run<R, ScopedRow, A>` block. The state
  type and program result type coincide for `get`, so it sits
  on the existing impl block without specializing `A`.
- `Run::put<StateType, Idx>(s: StateType) -> Run<R, ScopedRow, ()>`
  in a separate `impl<R, ScopedRow> Run<R, ScopedRow, ()>` block.
  The state type is generic and typically requires turbofish
  because `put`'s result is `()` (which doesn't constrain the
  state type from the call site).
- Both thread `RcBrand` as the pointer kind. Continuations
  constructed via
  `<RcBrand as ToDynCloneFn>::new(closure)`; direct
  `Rc::new(closure)` returns `Rc<{closure_type}>` rather than
  `Rc<dyn Fn>` and would not match `State::Get`/`Put`'s slot.
- Per-method doctests on each constructor exercise the
  canonical-row instantiation.

What the plan called for, and what diverged:

- **`#[document_parameters("...")]` cannot annotate impl blocks
  with no `&self` / `self` methods.** Both new impl blocks
  contain only associated functions (no receiver), so they use
  `#[document_type_parameters(...)]` only. Documented as
  general macro-attribute knowledge in subsequent commits.
- **Trybuild `.stderr` regenerated for the
  `im_do_ref_on_non_clone_wrapper` UI test.** Adding `Run::get`
  / `Run::put` changes rustc's "consider using one of the
  following associated functions" suggestion list, so the
  captured `.stderr` shifts. Regenerated via
  `TRYBUILD=overwrite cargo test --test compile_fail`.

Verification: `just verify` clean. 2492 unit tests + 2 new
doctests (Run::get, Run::put) compile and pass.

Open follow-ups:

- 5a.5: `RunExplicit::get/put`, `RcRunExplicit::get/put`
  (Explicit non-Arc family, blocker-independent).
- 5a.4: `ArcRun::get/put` blocked on the
  [2026-05-03 SendFunctor active blocker](plan.md#active-blockers).
- 5a.6: `ArcRunExplicit::get/put` same blocker.
- Integration tests in `fp-library/tests/run_state.rs` once
  all six wrappers' smart constructors land.

### Step 5a.3: RcRun::get and RcRun::put smart constructors

Mirrors 5a.2's `Run::get` / `Run::put` pattern across the
multi-shot single-thread Erased substrate. Threads
[`RcBrand`](../../../fp-library/src/brands.rs) as the pointer
kind, just like 5a.2 (`Run` and `RcRun` both run on the
single-thread substrate; the difference is multi-shot vs
single-shot, which is orthogonal to the handler-pointer
choice).

What landed:

- `RcRun::get<Idx>() -> Self` on a new
  `impl<R, ScopedRow, A> RcRun<R, ScopedRow, A>` block (state
  type and result type coincide for `get`).
- `RcRun::put<StateType: Clone + 'static, Idx>(s) -> Self` on
  a separate `impl<R, ScopedRow> RcRun<R, ScopedRow, ()>`
  block (state-type generic; turbofish typically required
  since `put`'s result is `()`).
- A manual
  [`Clone`](../../../fp-library/src/types/effects/state.rs)
  impl for `State<'a, P, S, A>` gated on `S: Clone + 'a`.
- Per-method doctests + a doctest on the `State::Clone` impl.

What the plan called for, and what diverged:

- **Manual `State::Clone` impl shape.** The plan text simply
  said "mirrors 5a.2's pattern," but `RcRun::lift`'s where-
  clause carries an `Apply!(<EBrand as Kind!(...)>::Of<'static, A>): Clone`
  bound that 5a.2's `Run::lift` does not. For
  `EBrand = StateBrand<RcBrand, A>`, this expands to
  `State<'static, RcBrand, A, A>: Clone`, which `State`
  did not satisfy out of the box. A manual `Clone` impl was
  required. The bound shape is `S: Clone + 'a` (because
  `Put`'s `S` field is cloned structurally); the
  continuation-pointer field
  `<P as RefCountedPointer>::Of<'a, dyn ...>` is
  unconditionally `Clone` per the trait's associated-type
  bound, so `k.clone()` always works without a `P`-side
  Clone bound.
- **Why not `#[derive(Clone)]`.** A derive would add
  `P: Clone` and `A: Clone` bounds (rustc's derive emits
  trait bounds for every type parameter that appears in
  fields, even at the type-projection level). Neither holds
  nor is needed: `P` is a brand (zero-sized), and `A`
  appears only inside the `dyn Fn(...) -> A`'s return type,
  which is reached via the `Rc<dyn ...>` indirection that's
  unconditionally `Clone`. Manual impl with the minimum
  bound is the right shape.
- **`A: Clone + 'static` requirement on `RcRun::get`'s impl
  block.** 5a.2's `Run::get` has `A: 'static` only. `RcRun::get`
  needs `A: Clone` because the doctest's `peel().is_err()`
  call exercises `RcRun::peel`'s substrate-Clone bound. This
  is the standard cascade for the shared-substrate family.
- **`StateType: Clone + 'static` on `RcRun::put`.** Same
  cascade reason: the substrate's `peel` requires the inner
  effect to be Clone, and `State::Clone` requires
  `S: Clone`.

The bound cascade applies to all four shared-substrate
wrappers' smart constructors:

- `Run::get/put` (5a.2): no `A: Clone` (single-shot,
  `Box<dyn FnOnce>` substrate; `peel` does not require Clone).
- `RcRun::get/put` (5a.3, this step): adds `A: Clone +
'static` and `StateType: Clone + 'static`.
- `RunExplicit::get/put` (5a.5, pending): `Run`-shaped
  bounds, no `Clone`.
- `RcRunExplicit::get/put` (5a.5, pending): `RcRun`-shaped
  bounds, with `Clone`.
- `ArcRun::get/put` (5a.4, blocked): adds `Send + Sync`
  cascade plus `Clone`.
- `ArcRunExplicit::get/put` (5a.6, blocked): adds
  `Send + Sync` cascade plus `Clone`.

Verification: `just verify` clean. 2500+ unit tests + 3 new
doctests (`RcRun::get`, `RcRun::put`, `State::Clone`) compile
and pass. The
`fp-library/tests/ui/im_do_ref_on_non_clone_wrapper.stderr`
file is unchanged (the existing UI test targets `Run`'s
diagnostic, not `RcRun`'s).

Open follow-ups: same as 5a.2's open follow-ups minus 5a.3
itself.

### Step 5a.5: RunExplicit and RcRunExplicit get/put smart constructors

Mirrors 5a.2 (`Run::get/put`) and 5a.3 (`RcRun::get/put`) across
the Explicit-substrate non-Arc family. One commit covering both
wrappers because the Explicit-vs-Erased axis is orthogonal to
the single-thread-vs-thread-safe axis: both `RunExplicit` and
`RcRunExplicit` thread
[`RcBrand`](../../../fp-library/src/brands.rs) as the pointer
kind, with the same `<RcBrand as ToDynCloneFn>::new(closure)`
continuation-construction call across all four methods.

What landed:

- `RunExplicit::get<Idx>() -> Self` on a new
  `impl<'a, R, ScopedRow, A: 'a> RunExplicit<'a, R, ScopedRow, A>`
  block (state and result type coincide for `get`).
- `RunExplicit::put<StateType: 'static, Idx>(s) -> Self` on a
  separate `impl<'a, R, ScopedRow> RunExplicit<'a, R, ScopedRow, ()>`
  block (state-type generic).
- `RcRunExplicit::get<Idx>() -> Self` and
  `RcRunExplicit::put<StateType: Clone + 'static, Idx>(s) -> Self`
  on parallel impl blocks.

What the plan called for, and what diverged:

- **`A: 'static` on the `get` method even on Explicit
  wrappers.** `RunExplicit` and `RcRunExplicit` carry an
  `'a` lifetime parameter, so it would be natural to expect
  the smart constructor's state type to follow `'a`. Instead
  the bound is `A: 'static`. The driver is
  [`StateBrand<P, S>`](../../../fp-library/src/brands.rs)'s
  [`impl_kind!`](../../../fp-macros/src/lib.rs) registration:
  `S: 'static` is fixed at the brand level, pinning the
  state type regardless of the enclosing wrapper's lifetime.
  This is a deliberate constraint on the State effect's
  scope; lifting it would require parameterising `StateBrand`
  by an additional lifetime, which the locked-in design
  rejected.
- **No substrate-`Clone` cascade on `RcRunExplicit::lift`.**
  `RcRun::lift` (Erased shared substrate) requires
  `Apply!(<NodeBrand<R, S>>::Of<..., RcFree<...>>): Clone`
  for the recursive walk's projection cloning; 5a.3 cascades
  this to `RcRun::get/put`. `RcRunExplicit::lift` (Explicit
  shared substrate) does not require this bound because
  `RcFreeExplicit` uses `Box`-in-`Wrap` with the `Rc<Inner>`
  outer wrap; the walk doesn't need additional projection-
  `Clone` bounds. Net: `RcRunExplicit::get/put`'s where-
  clauses are simpler than `RcRun::get/put`'s.
- **One commit covering both wrappers.** 5a.2 (Run) and 5a.3
  (RcRun) each landed as a separate commit, one wrapper per
  commit. 5a.5 bundles both Explicit wrappers because the
  pattern is well-trodden after 5a.3 (the `RunExplicit` half
  reuses 5a.2's bound shape; the `RcRunExplicit` half reuses
  5a.3's bound shape). Splitting would have meant ~4 method
  bodies per commit, well below the 1500-line / 7-file
  threshold for an oversized step.

Per-wrapper `Clone` cascade summary (now four wrappers
covered):

- `Run::get/put` (5a.2): `A: 'static` (or `StateType: 'static`
  on `put`); no `Clone`.
- `RunExplicit::get/put` (5a.5): same as `Run`'s; no `Clone`.
- `RcRun::get/put` (5a.3): adds `A: Clone + 'static` and
  `StateType: Clone + 'static`; cascades substrate-`Clone`
  bound from `RcRun::lift`.
- `RcRunExplicit::get/put` (5a.5): adds `A: Clone + 'static`
  and `StateType: Clone + 'static`; no substrate-`Clone`
  bound (Explicit substrate doesn't need it).
- `ArcRun::get/put` (5a.4, blocked): pending.
- `ArcRunExplicit::get/put` (5a.6, blocked): pending.

Verification: `just verify` clean. 2500+ unit tests + 4 new
doctests (`RunExplicit::get`, `RunExplicit::put`,
`RcRunExplicit::get`, `RcRunExplicit::put`) compile and pass.
The `fp-library/tests/ui/im_do_ref_on_non_clone_wrapper.stderr`
file is unchanged (the UI test targets `Run`'s diagnostic).

Open follow-ups: 5a.4 (`ArcRun::get/put`) and 5a.6
(`ArcRunExplicit::get/put`) blocked on the
[2026-05-03 SendFunctor active blocker](plan.md#active-blockers);
integration tests in `fp-library/tests/run_state.rs` once all
six wrappers' smart constructors land.

### Step 5a.4 + 5a.6: ArcRun and ArcRunExplicit get/put smart constructors plus SendStateBrand / SendState

Closes step 6a (all six wrappers covered) under the
[2026-05-03 SendFunctor option-(c) resolution](resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified).
Adds a parallel `SendStateBrand<P, S>` /
`SendState<'a, P, S, A>` type for the Arc family, alongside
the existing `StateBrand<P, S>` / `State<'a, P, S, A>` for
the non-Arc family.

What landed:

- [`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
  brand registration parallel to `StateBrand<P, S>`.
- [`SendState<'a, P, S, A>`](../../../fp-library/src/types/effects/state.rs)
  enum whose variants store
  `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A + Send + Sync>`
  (the `+ Send + Sync` is baked into the trait object's
  bounds, so the projection is structurally `Send + Sync`).
- `impl_kind!` for `SendStateBrand`.
- Manual `Clone` impl for `SendState` gated on
  `S: Clone + 'a` (mirrors 5a.3's `State::Clone`).
- `SendFunctor` impl for `SendStateBrand` (the whole point
  of (c); implementable because the projection is
  structurally `Send + Sync`).
- `ArcRun::get<Idx>() -> Self` and
  `ArcRun::put<StateType: Clone + Send + Sync + 'static, Idx>(s) -> Self`
  using `SendStateBrand<ArcBrand, A>` /
  `SendStateBrand<ArcBrand, StateType>` in the row.
- `ArcRunExplicit::get<Idx>() -> Self` and
  `ArcRunExplicit::put<StateType: Clone + Send + Sync + 'static, Idx>(s) -> Self`
  same pattern, with the Explicit substrate's per-method
  `Send + Sync` cascade through
  `ArcFreeExplicit<...>: Send + Sync`.

What the plan called for, and what diverged:

- **Option (b) was ratified first, then discovered
  unimplementable.** The
  [original 2026-05-03 ratification](resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds)
  locked in per-method `Send + Sync` bounds at smart-
  constructor sites. Implementation of `ArcRun::get` /
  `ArcRun::put` under that lock-in failed at compile time:
  `Arc<dyn Fn(...)>` is structurally `!Send + !Sync` because
  the trait object's bounds don't include `Send + Sync`, and
  `Arc<T>: Send + Sync` requires `T: Send + Sync`
  structurally. Use-site bounds can't refine a structural
  type-level fact. The blocker was reopened and re-ratified
  with option (c). See the
  [option (c) resolution](resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)
  for the full investigation, alternatives (including (d.1)
  polymorphic-projection State and (d.2) unconditional Send-
  aware representation), and rationale.
- **Two State types instead of one.** Users with mixed
  Rc/Arc programs face two distinct row brands they must
  use depending on the substrate. The Phase 3 step 7
  `define_effect!` macro can hide this distinction by
  selecting the right brand per wrapper.
- **No `Functor` impl on `SendStateBrand`.** `Functor::map`'s
  signature only requires `f: Fn` (no `Send + Sync`), so a
  `Functor` impl could not construct the Send-aware trait
  object that `SendState`'s variants require.
  [`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
  is independent of `Functor` in fp-library (not a
  supertrait), so the gap is sound; users reach for
  `SendFunctor::send_map` directly.
- **`A: Clone + Send + Sync + 'static`** on the Arc smart
  constructors. The `'static` comes from `SendStateBrand<P, S>`'s
  `impl_kind!` registration (`S: 'static`); `Clone` cascades
  from `SendState::Clone`'s `S: Clone` bound; `Send + Sync`
  cascades from `ArcRun::lift` / `ArcRunExplicit::lift`'s
  per-method bounds.

Per-wrapper smart-constructor brand summary (now all six
covered):

- `Run::get/put` (5a.2): `StateBrand<RcBrand, S>`.
- `RcRun::get/put` (5a.3): `StateBrand<RcBrand, S>`.
- `RunExplicit::get/put` (5a.5): `StateBrand<RcBrand, S>`.
- `RcRunExplicit::get/put` (5a.5): `StateBrand<RcBrand, S>`.
- `ArcRun::get/put` (5a.4, this step):
  `SendStateBrand<ArcBrand, S>`.
- `ArcRunExplicit::get/put` (5a.6, this step):
  `SendStateBrand<ArcBrand, S>`.

Verification: `just verify` clean. 2500+ unit tests + 6 new
doctests (`SendState::Clone`,
`SendStateBrand::SendFunctor::send_map`, `ArcRun::get`,
`ArcRun::put`, `ArcRunExplicit::get`, `ArcRunExplicit::put`)
compile and pass.

Open follow-ups: integration tests in
`fp-library/tests/run_state.rs` covering bind-chain
composition, run with handlers dispatching State, and
`interpret`-with-closure-capture state threading through
Get/Put for each of the six wrappers. The Arc family subset
of those tests was unblocked by the
[step 5a.4 + 5a.6 follow-up below](#step-5a4--5a6-follow-up-2026-05-04-arccoyoneda-algebra-migrated-to-sendfunctor).
Step 6 (`Reader`, `Except`, `Writer`, `Choose` smart
constructors) follows the 6a per-wrapper rollout pattern;
`Reader` will likely need a parallel `SendReaderBrand` for
the same reason `State` needed `SendStateBrand`.

### Step 5a.4 + 5a.6 follow-up (2026-05-04): `ArcCoyoneda` algebra migrated to `SendFunctor`

Closes the downstream gap surfaced by the 5a.4 + 5a.6
`SendStateBrand` rollout. Full investigation in
[resolutions.md](resolutions.md#resolved-2026-05-04-phase-3-step-6a-downstream-blocker-arccoyonedas-algebra-migrated-to-sendfunctor-option-a).

What landed:

- [`fp-library/src/types/vec.rs`](../../../fp-library/src/types/vec.rs):
  `SendFunctor` impl for `VecBrand` (byte-identical body to
  `Functor::map`'s, with tighter `Send + Sync` bounds).
- [`fp-library/src/types/arc_coyoneda.rs`](../../../fp-library/src/types/arc_coyoneda.rs):
  inner `ArcCoyonedaLowerRef` trait method bound migrated
  from `F: Functor` to `F: SendFunctor`. Three layer impls
  (Base, MapLayer, NewLayer) updated; bodies use
  `F::send_map`. `MapLayer` and `NewLayer` impl-blocks gain
  `B: Send + Sync + 'a`. Public methods (`lower_ref`,
  `collapse`, `hoist`, `fold_map`, `bind`, `apply`, `lift2`)
  migrated. Main impl block gains `A: Send + Sync + 'a`.
  `ArcCoyoneda::map<B>` and `ArcCoyoneda::new<B>` gain
  `B: Send + Sync + 'a`. `From<ArcCoyoneda> for Coyoneda`
  bounds tightened to `F: SendFunctor` and `A: Send + Sync`.
- [`fp-library/src/types/effects/interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs):
  dropped `+ Functor` from the ArcCoyoneda dispatch impl's
  `EBrand` bound (now just
  `EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static`).
- [`fp-library/src/types/effects/arc_run.rs`](../../../fp-library/src/types/effects/arc_run.rs):
  `A: Send + Sync` added to the `lift_node` helper's
  where-clause.
- [`fp-library/tests/ui/arc_coyoneda_requires_send.stderr`](../../../fp-library/tests/ui/arc_coyoneda_requires_send.stderr):
  re-blessed; the `Rc<i32>` rejection now points at the
  impl-block-level `A: Send + Sync + 'a` bound rather than
  the inner `Apply!` clone-bound on `lift`.

What diverged:

- **Bound placement matches ArcFree's precedent.**
  `A: Send + Sync + 'a` lives at the main impl block;
  `F: SendFunctor` lives at method-level where-clauses; the
  struct definition stays minimal. Considered struct-level
  bounds but rejected because they would propagate to every
  type-position mention of `ArcCoyoneda` and tighten the API
  surface beyond what's needed.
- **Brand-level `Foldable` on `ArcCoyonedaBrand` dropped.**
  `Foldable::fold_map`'s trait method declares `A: Clone`
  only, but the post-migration body needs `A: Send + Sync`,
  and Rust forbids tightening trait method bounds in impls.
  Two test functions (`fold_map_on_mapped` and
  `foldable_consistency_vec`) updated to use the inherent
  `ArcCoyoneda::fold_map` method instead. Restoring the
  brand-level surface requires a `SendFoldable` trait
  parallel to `SendFunctor`; deferred to a separate
  follow-up commit before `run_state.rs` ships.

Verification: `just verify` clean. Existing 2500+ unit tests
plus all doctests pass; the `arc_coyoneda_requires_send` UI
compile-fail test still rejects `Rc<i32>` (with the new
error pointing at the impl-block bound).

Open follow-ups:

- Integration tests in
  [`fp-library/tests/run_state.rs`](../../../fp-library/tests/run_state.rs)
  landed in a separate `test(effects):` commit covering all
  six wrappers (3 tests per wrapper, 18 total).

### Step 5a.4 + 5a.6 second follow-up (2026-05-04): `SendFoldable` trait + brand-level fold restored on `ArcCoyonedaBrand`

Closes the `SendFoldable` follow-up flagged by the
[2026-05-04 ArcCoyoneda migration resolution](resolutions.md#resolved-2026-05-04-phase-3-step-6a-downstream-blocker-arccoyonedas-algebra-migrated-to-sendfunctor-option-a).
The previous follow-up dropped the brand-level
[`Foldable`](../../../fp-library/src/classes/foldable.rs) impl
on `ArcCoyonedaBrand` because `Foldable::fold_map` declares
only `A: Clone` on the trait method but the post-migration
body needs `A: Send + Sync`, which Rust forbids tightening
in impls. The Send-aware parallel
[`SendFoldable`](../../../fp-library/src/classes/send_foldable.rs)
bakes `A: Send + Sync` into the trait method signature, so
the impl can call `ArcCoyoneda::lower_ref` cleanly.

What landed:

- [`fp-library/src/classes/send_foldable.rs`](../../../fp-library/src/classes/send_foldable.rs):
  new trait with three methods (`send_fold_map`,
  `send_fold_right`, `send_fold_left`), each with default
  implementations in terms of the others (matching
  `SendRefFoldable`'s pattern). `FnBrand: SendLiftFn` plus
  `Send + Sync` bounds on `A`, the closure, and the
  monoid/accumulator. Free function `send_fold_map<Brand: SendFoldable, ...>`
  for explicit dispatch.
- [`fp-library/src/classes.rs`](../../../fp-library/src/classes.rs):
  module registration alphabetically between `send_deferrable`
  and `send_functor`.
- [`fp-library/src/types/vec.rs`](../../../fp-library/src/types/vec.rs):
  `SendFoldable` impl for `VecBrand` (overrides
  `send_fold_map` directly with a one-line iterator body;
  `send_fold_right` / `send_fold_left` use the trait
  defaults).
- [`fp-library/src/types/arc_coyoneda.rs`](../../../fp-library/src/types/arc_coyoneda.rs):
  `SendFoldable` impl for `ArcCoyonedaBrand<F>` requiring
  `F: SendFunctor + SendFoldable + 'static`; body delegates
  to `F::send_fold_map` after lowering. Module-level doc
  and brand-level summary comment updated to mention the new
  surface. Two property tests (`fold_map_on_mapped`,
  `foldable_consistency_vec`) cleaned up; the latter now
  exercises brand-level `send_fold_map` dispatch.

What diverged:

- **Three-method default-implementations cycle.** Mirrors
  `SendRefFoldable`'s pattern (each method has a default
  implementation in terms of one other), so implementors
  only need to provide one. `Foldable` uses the same pattern
  modulo the Send-aware bounds.
- **No impls on `OptionBrand` or `ResultBrand`.** The 2026-05-04
  migration resolution flagged these as cascade candidates,
  but the only consumer is `ArcCoyonedaBrand<F>` where `F`
  is the substrate brand inside the Coyoneda. The current
  test surface uses `F = VecBrand`; `OptionBrand` and
  `ResultBrand` impls can land when the first user-side need
  surfaces.

Verification: `just verify` clean. 2520 unit tests + new
doctests (`send_fold_map` free function, `VecBrand::send_fold_map`,
`VecBrand::send_fold_right`, `VecBrand::send_fold_left`,
`ArcCoyonedaBrand::send_fold_map`) compile and pass.

### Step 5a integration tests (2026-05-04): `run_state.rs`

End-to-end integration tests for the State effect smart
constructors on all six Run wrappers, landed at
[`fp-library/tests/run_state.rs`](../../../fp-library/tests/run_state.rs).
Three tests per wrapper:

- `*_get_returns_current_state`: a single Get effect
  dispatched through a handler that reads from a captured
  cell.
- `*_put_writes_state`: a single Put effect dispatched
  through a handler that writes to a captured cell.
- `*_get_put_get_bind_chain`: a bind-chained program
  (`get >>= |s| put(s + 1) >>= |_| get`) verifying state
  threads through the bind continuation.

State threading is via user-side closure captures
(`Rc<RefCell<S>>` for the four non-Arc wrappers,
`Arc<Mutex<S>>` for the two Arc wrappers). The non-Arc
family threads `RcBrand` and uses `StateBrand<RcBrand, S>`
in the row; the Arc family threads `ArcBrand` and uses
`SendStateBrand<ArcBrand, S>` whose closure projection
bakes in `Send + Sync` at the type level.

Implementation notes:

- **Closure parameter types use `'_` (not `'static`) for
  the inner State projection lifetime** so the closure is
  HRTB-polymorphic over State's projection lifetime (e.g.,
  `op: State<'_, RcBrand, i32, Run<...>>` rather than
  `op: State<'static, ...>`). Pinning the projection
  lifetime to `'static` produces a
  "FnOnce-not-general-enough" error because `interpret`'s
  where-clause requires `for<'a> Fn(State<'a, ...>) -> ...`.
  The wrapper's outer lifetime stays `'static` (e.g.,
  `RunExplicit<'static, ...>`) because that's pinned by the
  program type.

Verification: 18 tests pass; full `just verify` clean.

### Brands reorg (2026-05-04): extract effect-specific brands to `brands/effects.rs`

Refactor commit landed before step 5b to mirror the existing
[`crate::types::effects`](../../../fp-library/src/types/effects)
sub-module organisation. Brands whose corresponding types live
in `types/effects/` now cluster in
[`crate::brands::effects`](../../../fp-library/src/brands/effects.rs),
re-exported flat at `crate::brands` so user-facing paths
(`crate::brands::StateBrand`, etc.) are unchanged.

Brands moved:

- `ArcRunExplicitBrand`, `RcRunExplicitBrand`,
  `RunExplicitBrand` (Run-wrapper brands for the Explicit
  family).
- `CoproductBrand`, `CNilBrand`, `NodeBrand` (effect-row
  machinery).
- `StateBrand`, `SendStateBrand` (the State first-order
  effect).

Brands kept in `brands.rs`:

- Coyoneda variants (`CoyonedaBrand`, `RcCoyonedaBrand`,
  `ArcCoyonedaBrand`, `CoyonedaExplicitBrand`) and Free-family
  brands (`FreeExplicitBrand`, `RcFreeExplicitBrand`,
  `ArcFreeExplicitBrand`) stay because their corresponding
  types live in `crate::types` (not `crate::types::effects`);
  they are general functor / free-monad abstractions that
  the effects subsystem consumes but does not own.
- All non-effect brands (substrate brands, container brands,
  Lazy brands, etc.) keep their existing positions.

The new module wraps in
`#[fp_macros::document_module]` + `mod inner` +
`pub use inner::*` per the project pattern. Doc strings
rewritten to be self-contained (no external plan-doc or
third-party-repo hyperlinks).

Considered moving effects-related traits to a
`classes/effects/` sub-module but decided against it: the
effects-specific traits in the codebase
([`DispatchHandlers`](../../../fp-library/src/types/effects/interpreter.rs),
[`Member`](../../../fp-library/src/types/effects/member.rs))
already live in `types/effects/`, co-located with the types
they support. The remaining traits in `classes/`
([`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs),
[`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs))
are general categorical / structural interfaces that
effects happen to be a primary consumer of, not effects-
specific in nature. `classes/` stays flat.

### Step 5b: `ask` smart constructors on all six Run wrappers (`Reader` effect)

Adds the
[`Reader<'a, P, E, A>`](../../../fp-library/src/types/effects/reader.rs)
first-order effect type with the single `Ask` operation,
parallel to the State / `Get` shape. Per-wrapper `ask` smart
constructors lift the identity-on-environment continuation
through each wrapper's substrate-appropriate pointer kind.

What landed:

- [`ReaderBrand<P, E>`](../../../fp-library/src/brands/effects.rs)
  brand registration (parameterised by pointer brand `P` and
  environment type `E`, parallel to `StateBrand<P, S>`).
- [`Reader<'a, P, E, A>`](../../../fp-library/src/types/effects/reader.rs)
  enum with single `Ask` variant holding
  `<P as RefCountedPointer>::Of<'a, dyn Fn(E) -> A>`. Manual
  `Clone` impl (refcount-bumps the continuation pointer);
  `Functor` impl composes `f: A -> B` with the stored
  continuation via `ToDynCloneFn::new`.
- `impl_kind!` for `ReaderBrand`.
- [`SendReaderBrand<P, E>`](../../../fp-library/src/brands/effects.rs)
  brand registration for the Arc family (analogous to
  `SendStateBrand`); `Arc<dyn Fn(E) -> A>` (without
  `+ Send + Sync` in the trait object's bounds) is
  structurally `!Send + !Sync`, so a parallel brand whose
  projection bakes the marker traits in at the type level is
  required for end-to-end dispatch through `*Run::interpret`
  on Arc-substrate programs.
- [`SendReader<'a, P, E, A>`](../../../fp-library/src/types/effects/reader.rs)
  enum with single `Ask` variant holding
  `<P as SendRefCountedPointer>::Of<'a, dyn Fn(E) -> A + Send + Sync>`.
  Manual `Clone`; `SendFunctor` impl. No `Functor` impl
  (would require constructing a Send-aware trait object from
  a non-Send-bounded `f: Fn`, which it can't).
- `Run::ask`, `RcRun::ask`, `RunExplicit::ask`,
  `RcRunExplicit::ask` smart constructors using
  `ReaderBrand<RcBrand, A>` in the row.
- `ArcRun::ask`, `ArcRunExplicit::ask` smart constructors
  using `SendReaderBrand<ArcBrand, A>` in the row, with the
  per-wrapper `Send + Sync` cascade matching `ArcRun::get` /
  `ArcRunExplicit::get`'s pattern.
- Integration tests in
  [`fp-library/tests/run_reader.rs`](../../../fp-library/tests/run_reader.rs):
  12 tests (2 per wrapper) covering single-Ask dispatch and a
  bind-chained `ask >>= |e1| ask >>= |e2| pure(e1 + e2)`
  program verifying the same environment is delivered on each
  successive Ask within one program.

What diverged:

- Reader's `Functor::map` body has only one variant
  (vs. State's two), so the impl is shorter; otherwise the
  shape matches.
- The `ask` smart constructor mirrors `get` exactly because
  both produce "read this thing back as the result"
  semantics; the call-site looks identical except for the
  brand name (`StateBrand` vs `ReaderBrand`).

Verification: 12 Reader tests pass; full `just verify` clean.

### Step 5c: `throw` smart constructors on all six Run wrappers (`Except` effect)

Adds the
[`Except<'a, E, A>`](../../../fp-library/src/types/effects/except.rs)
first-order effect type with the single `Throw` operation.
Per-wrapper `throw` smart constructors lift an error of type
`E` through each wrapper's substrate.

What landed:

- [`ExceptBrand<E>`](../../../fp-library/src/brands/effects.rs)
  brand registration. Parameterised only by the error type
  `E`, with no pointer-brand `P` parameter (`Except` has no
  continuation, so it does not need substrate-pointer
  selection). The same brand serves all six Run wrappers.
- [`Except<'a, E, A: 'a>`](../../../fp-library/src/types/effects/except.rs)
  enum with a single `Throw(E, PhantomData<&'a A>)` variant.
  The `A` parameter is phantom (`Throw` never returns to the
  caller); `PhantomData<&'a A>` keeps the type within the
  [`Kind`](../../../fp-library/src/kinds.rs) trait's
  `Of<'a, A: 'a>: 'a` contract without imposing variance
  constraints from references the type does not own.
- Manual `Clone` impl gated on `E: Clone`.
- `Functor` and `SendFunctor` impls. Both bodies discard the
  mapping function `f` (since `Throw` carries no `A`-typed
  payload) and rebuild the variant with the new phantom type
  parameter.
- `Run::throw`, `RcRun::throw`, `RunExplicit::throw`,
  `RcRunExplicit::throw` smart constructors using
  `ExceptBrand<ErrorType>` in the row.
- `ArcRun::throw`, `ArcRunExplicit::throw` smart constructors
  using the same `ExceptBrand<ErrorType>` (no parallel
  `SendExceptBrand` needed). Per-wrapper `Send + Sync`
  cascades on `ErrorType` instead.
- Integration tests in
  [`fp-library/tests/run_except.rs`](../../../fp-library/tests/run_except.rs):
  12 tests (2 per wrapper) covering single-Throw dispatch
  and a `pure(x).bind(|_| throw(e))` chain verifying that
  Throw can appear after a successful bind step.

What diverged:

- **No parallel `SendExceptBrand` for the Arc family.**
  State and Reader needed `Send*Brand` siblings because
  their `dyn Fn(...)` continuations are structurally
  `!Send + !Sync` without the marker traits baked into the
  trait object's bounds. `Except` has no `dyn Fn`
  continuation, so this concern doesn't apply; the per-
  wrapper `Send + Sync` bound on `ErrorType` is sufficient.
- **`PhantomData<&'a A>` rather than
  `PhantomData<(&'a (), fn() -> A)>`.** Initial draft
  attempted to encode covariance in `A` via `fn() -> A`, but
  clippy flagged the resulting tuple as `type_complexity`.
  Reverted to the simpler `PhantomData<&'a A>` (which makes
  `A` invariant via the reference); since `A` is purely a
  type-level marker for the `Functor` interface and never
  participates in field data, invariance is unproblematic.

Verification: 12 Except tests pass; full `just verify` clean.

### Step 5d: `tell` smart constructors on all six Run wrappers (`Writer` effect)

Adds the
[`Writer<'a, W, A>`](../../../fp-library/src/types/effects/writer.rs)
first-order effect type with the single `Tell` operation.
Per-wrapper `tell` smart constructors lift a log value of
type `W` through each wrapper's substrate.

What landed:

- [`WriterBrand<W>`](../../../fp-library/src/brands/effects.rs)
  brand registration. Parameterised only by the log type `W`,
  with no pointer-brand parameter (same shape as
  `ExceptBrand`; `Writer` has no continuation so it does not
  need substrate-pointer selection). The same brand serves
  all six Run wrappers.
- [`Writer<'a, W, A: 'a>`](../../../fp-library/src/types/effects/writer.rs)
  enum with a single `Tell(W, A, PhantomData<&'a ()>)`
  variant. Unlike `Except`, `A` is owned (the next-program
  value), not phantom; the `PhantomData<&'a ()>` exists only
  to satisfy the [`Kind`](../../../fp-library/src/kinds.rs)
  trait's `Of<'a, A: 'a>: 'a` contract since `'a` is unused
  in the variants.
- Manual `Clone` impl gated on `W: Clone, A: Clone`.
- `Functor` and `SendFunctor` impls. The mapping function `f`
  is applied to the stored `A` to produce a new `Writer<'a,
W, B>`; the log value is carried unchanged.
- `Run::tell`, `RcRun::tell`, `RunExplicit::tell`,
  `RcRunExplicit::tell` smart constructors using
  `WriterBrand<LogType>` in the row.
- `ArcRun::tell`, `ArcRunExplicit::tell` smart constructors
  using the same `WriterBrand<LogType>` (no parallel
  `SendWriterBrand` needed). Per-wrapper `Send + Sync`
  cascades on `LogType` instead.
- Integration tests in
  [`fp-library/tests/run_writer.rs`](../../../fp-library/tests/run_writer.rs):
  12 tests (2 per wrapper) covering single-Tell dispatch and
  a `tell(a) >>= |_| tell(b)` chain verifying that both logs
  are captured in order.

What diverged:

- **No parallel `SendWriterBrand` for the Arc family.** Same
  reasoning as Except: no `dyn Fn` continuation means the
  `Send + Sync` cascade reduces to a per-wrapper bound on
  `LogType` alone.
- **`tell` lives on the `Run<R, S, ()>` impl block** (mirrors
  `put`) because the result type is `()`. The smart
  constructor takes a `LogType` argument (the log value) and
  produces a `Run<R, S, ()>`.

Verification: 12 Writer tests pass; full `just verify`
clean.

### Step 5e: `choose` smart constructors on the four multi-shot Run wrappers (`Choose` effect) plus Erased Free family multi-shot substrate fix

Adds the nondeterministic-branching
[`Choose<'a, P, A>`](../../../fp-library/src/types/effects/choose.rs)
first-order effect type with the single `Alt(P::Of<'a, dyn 'a +
Fn(bool) -> A>)` variant. Per-wrapper `choose` smart constructors
lift the effect into a row whose handler can run the continuation
twice (once per branch) to capture both outcomes.

Step 5e shipped together with a substrate fix on the Erased Free
family because the original implementation panicked at runtime on
the Erased pair. See the 2026-05-04 resolution
[`Phase 3 step 5e Erased Free family multi-shot dispatch via RcCatList/ArcCatList`](resolutions.md#resolved-2026-05-04-phase-3-step-5e-erased-free-family-multi-shot-dispatch-via-rccatlist--arccatlist-option-1c-ii-parallel-reference-counted-catlist-variants)
for the design discussion.

What landed:

- [`ChooseBrand<P>`](../../../fp-library/src/brands/effects.rs)
  brand parameterised by `P: ToDynCloneFn` (typically `RcBrand`)
  for the Rc-substrate variant; serves the Rc-family wrappers.
- [`SendChooseBrand<P>`](../../../fp-library/src/brands/effects.rs)
  parallel for the Arc family; the projection bakes `Send + Sync`
  into the trait-object bounds so Arc-substrate programs satisfy
  thread-safety end-to-end.
- [`Choose<'a, P, A>`](../../../fp-library/src/types/effects/choose.rs)
  with `Functor` and `SendFunctor` impls.
- `RcRun::choose`, `RcRunExplicit::choose`, `ArcRun::choose`,
  `ArcRunExplicit::choose` smart constructors on each wrapper's
  `Self<R, S, bool>` impl block.
- Substrate fix: new
  [`RcCatList<A>`](../../../fp-library/src/types/rc_cat_list.rs)
  and [`ArcCatList<A>`](../../../fp-library/src/types/arc_cat_list.rs)
  reference-counted catenable list variants. `RcFree` and `ArcFree`
  switch their continuation queues to these new types and replace
  the `Cell<Option<...>>::take` / `Mutex<Option<...>>::take`
  workaround in `to_view` with capture-and-clone-per-call.
- Integration tests in
  [`fp-library/tests/run_choose.rs`](../../../fp-library/tests/run_choose.rs):
  4 tests (one per multi-shot wrapper) exercising lift -> bind ->
  peel and confirming the program suspends at the lifted `Alt`
  effect with continuations attached.

What diverged from the original Phase 3 step 5e plan:

- **Substrate fix shipped alongside the smart constructors.** The
  original step 5e plan assumed the Erased Free family supported
  multi-shot dispatch directly. It didn't. Rather than demote the
  2026-05-03 Q4=ii resolution ("`Choose` ships on all four
  multi-shot wrappers"), the substrate was extended with two
  parallel O(1)-cloneable catenable list types. The smart
  constructors and integration tests pass on all four wrappers as
  originally specified.
- **Choose ships only on `Self<R, S, bool>` impl blocks.** The
  result type is structurally `bool` (the branch), so a separate
  impl block parallel to the existing `get`/`put` bodies wasn't
  reusable; a new `impl<R, S> *Run<R, S, bool>` block was added
  per wrapper.
- **No parallel test for the substrate fix in isolation.** The
  Choose integration tests double as substrate validation; if the
  substrate fix regresses, those tests panic. This avoids a
  mock-only test that re-implements the failure mode.

Verification: 4 Choose tests pass; full pre-existing test suite
passes unchanged (no regression on single-inner Free workloads).
Per-Suspend cost on those workloads adds one `Rc::clone` (or
`Arc::clone`) for the captured continuation queue, which is a
refcount bump, not a structural copy.

### Step 7: `compile_fail` UI tests for Phase 3 negative cases

Adds three trybuild UI tests under
[`fp-library/tests/ui/`](../../../fp-library/tests/ui/) wired
into the existing
[`fp-library/tests/compile_fail.rs`](../../../fp-library/tests/compile_fail.rs)
harness (one-line glob `t.compile_fail("tests/ui/*.rs")` picks
them up automatically). Each `.rs` file is a small program that
should fail to compile, paired with a `.stderr` snapshot
generated via
`TRYBUILD=overwrite cargo test -p fp-library --test compile_fail`.

What landed:

- [`run_choose_not_found.rs`](../../../fp-library/tests/ui/run_choose_not_found.rs):
  verifies single-shot wrappers (`Run`, `RunExplicit`) reject
  the `Choose` smart constructor. Surfaces as
  [`E0599 no function or associated item named 'choose' found for struct 'Run<R, S, A>'`](https://doc.rust-lang.org/error_codes/E0599.html).
  Only `Run` is exercised; the same property applies to
  `RunExplicit` and a single failure suffices to demonstrate
  it.
- [`run_smart_constructor_type_mismatch.rs`](../../../fp-library/tests/ui/run_smart_constructor_type_mismatch.rs):
  verifies a smart constructor's result type is bound to the
  row's effect parameterization, not free to vary. Row carries
  `ReaderBrand<RcBrand, String>`; binding ascribes
  `Run<FirstRow, Scoped, i32>`; type inference unifies the
  row's `A` with the ascription's `A` and surfaces
  [`E0277 the trait bound 'CNil: CoprodUninjector<Coyoneda<'static, ReaderBrand<RcBrand, i32>, i32>, _>' is not satisfied`](https://doc.rust-lang.org/error_codes/E0277.html)
  via the
  [`Member`](../../../fp-library/src/types/effects/member.rs)
  trait's recursion.
- [`interpret_missing_handler.rs`](../../../fp-library/tests/ui/interpret_missing_handler.rs):
  verifies `interpret` rejects a handler list that doesn't
  cover every effect in the row. The
  [`DispatchHandlers`](../../../fp-library/src/types/effects/interpreter.rs)
  trait walks the handler list and the row in lock-step:
  `HandlersNil` only matches `CNil`. With a 2-effect row
  (`IdentityBrand` + `OptionBrand`) and a handler list
  covering only `IdentityBrand`, the recursion bottoms out at
  `HandlersNil` against `Coproduct<Coyoneda<OptionBrand, ...>, CNil>`,
  surfacing
  [`E0277 'HandlersNil: DispatchHandlers<...>' is not satisfied`](https://doc.rust-lang.org/error_codes/E0277.html).

What diverged from the original step 7 plan:

- **Three tests instead of four.** The plan listed four
  negative cases (handler missing, wrong type ascription,
  multi-shot via single-shot `Run`, `Choose` on single-shot
  wrappers). Cases 3 and 4 are the same property exercised
  from two angles: `Choose`'s smart constructor only exists on
  multi-shot wrappers, and using it on a single-shot wrapper
  produces the method-not-found error. One test
  (`run_choose_not_found.rs`) covers both; a second test
  proving the same property on `RunExplicit` would be
  redundant.
- **No test for cross-brand mismatch (`ChooseBrand<ArcBrand>`
  on `RcRun`).** Out of scope for step 7; the existing three
  tests cover the four originally-listed cases. Cross-brand
  mismatch detection is a property of the smart constructors'
  where-clauses, which would surface as a different error
  shape; worth adding if a future regression motivates it.
- **No `multi_brand_diagonal`-style test for the row-vs-handler
  positional alignment** (e.g., handler list in the wrong
  order). The trybuild test
  [`multi_brand_diagonal.rs`](../../../fp-library/tests/ui/multi_brand_diagonal.rs)
  already exercises a similar property at the row construction
  level; adding a Phase-3-specific positional-alignment test
  would duplicate it. The
  [`handlers!`](../../../fp-macros/src/effects/handlers.rs)
  macro's lexical sort matches the row brand's lexical sort,
  so user-side ordering errors are mechanically prevented at
  macro expansion time.

Verification: 23 UI tests pass (20 pre-existing + 3 new).
`just verify` clean across all sub-recipes.

### Step 8: review-remediation documentation pass

Bundles the doc-shaped items from
[`remediation_proposals.md`](review/0_first_order_effects_implementation/remediation_proposals.md)
that the substantive Phase 3 code work didn't already absorb.
Closes Phase 3.

What landed:

- **F2A + F5A:** new entries in
  [`plan.md`'s Out of scope](plan.md#out-of-scope) section.
  One entry explains why the freer-monad encoding has no
  callable continuation primitive (Plotkin-Pretnar `k`):
  handlers fold sub-programs via the
  [`DispatchHandlers`](../../../fp-library/src/types/effects/interpreter.rs)
  trait but do not receive a uniform resumable continuation;
  multi-shot semantics are achievable via the per-effect
  closure but the shape is per-effect, not per-handler. A
  second entry explains why the headline `interpret` API is
  not a rank-2 natural transformation: stable Rust closures
  cannot be `A`-polymorphic. Both cross-link to
  [`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)
  and [`Free::fold_free`](../../../fp-library/src/types/free.rs)
  as the rank-2 escape hatches.
- **F4A:** the [Success criteria](plan.md#success-criteria)
  "single-shot vs. multi-shot" claim weakened to apply to the
  Free wrapper's spine consumption only; per-effect closures
  (`State`'s `dyn Fn`, `Choose`'s `dyn Fn(bool) -> A`) carry
  their multi-shot property at the effect-instance level on
  every wrapper that hosts the effect, independent of the
  wrapper's spine semantics.
- **M4 audit + Coyoneda fusion docs:** added a
  "Coyoneda fusion at the call site" doc block to
  [`StateBrand`'s Functor impl](../../../fp-library/src/types/effects/state.rs)
  documenting that production rows wrap `StateBrand` in
  [`CoyonedaBrand`](../../../fp-library/src/brands.rs), so
  `StateBrand::map`'s per-call `Rc`/`Arc` allocation is
  amortised to one per **layer dispatch** (not per user-side
  `.map()`). Direct call-sites are exercised only by
  doctests; the rows in
  [`Run`](../../../fp-library/src/types/effects/run.rs) /
  [`RcRun`](../../../fp-library/src/types/effects/rc_run.rs) /
  [`ArcRun`](../../../fp-library/src/types/effects/arc_run.rs)
  / their `Explicit` siblings always interpose Coyoneda.
- **M6A async / IO workaround:** added a paragraph to the
  [interpreter module docs](../../../fp-library/src/types/effects/interpreter.rs)
  describing
  [`tokio::task::spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
  as the supported escape hatch for interleaving async work
  with synchronous effect interpretation, plus
  [`tokio::runtime::Handle::block_on`](https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.block_on)
  for handler closures that need to call out to async APIs.
  Explains why no `async fn` interpreter ships (no
  [`MonadRec`](../../../fp-library/src/classes/monad_rec.rs)
  impl for `Future`-shaped target monads).
- **M7A `Fn` vs `FnOnce` asymmetry note:** one-line note added
  to [`Run::bind`](../../../fp-library/src/types/effects/run.rs)
  and [`DispatchHandlers::dispatch`](../../../fp-library/src/types/effects/interpreter.rs)
  documenting that `bind` takes `f: FnOnce(A) -> ...`
  (single-shot, matching the Free continuation queue) while
  handler closures stored in
  [`Handler<E, F>`](../../../fp-library/src/types/effects/handlers.rs)
  are bound `F: Fn` (multi-shot, callable inside
  `tail_rec_m`'s step closure). Each cross-references the
  other so readers navigating the API surface see the
  asymmetry from both call sites.

What diverged from the original step 8 plan:

- **Minor m1-m9 findings excluded.** The original step 8
  description listed "all minor m1-m9 findings" as part of
  the bundle, but the
  [`remediation_proposals.md` sequencing plan](review/0_first_order_effects_implementation/remediation_proposals.md)
  explicitly slates them for a separate "polish" commit
  before the next public release. Step 8 ships the focused
  doc remediations for F2/F4/F5/M4/M6/M7 only.
- **Mid-step feedback on self-contained source docs.** The
  user flagged during the M6A edit that all source-code
  documentation must be self-contained (no plan / decisions
  / resolutions / `Phase N step M` / GitHub-URL references).
  The new spawn_blocking text was rewritten to remove
  "Phase 6+" and "Phase 3 step 1" references; the rule was
  saved as a memory note. Pre-existing source-doc external
  references across the effects module (~78 occurrences as
  of 2026-05-04) are addressed in the immediately-following
  commit, not in step 8 proper.

Verification: `just verify` clean across all sub-recipes.

### Step 9 (post-step-8 follow-up): scrub external references from source-code docs

Not a numbered phasing step; a doc-cleanup commit triggered by
mid-step-8 user feedback that all source-code documentation must
be self-contained. Scrubs `Phase N step M` identifiers,
`https://github.com/...` URLs into the project, and bare
references to plan / decisions / resolutions / deviations
documents from `.rs` source files. Replaces them with
self-contained explanations or intra-doc links to in-crate
symbols. No semantic changes; doc text only.

Two cross-cutting commits landed in the same set as 5a.1 / 5a.2;
not tied to a specific phase step but worth recording for
context:

#### `4f0e977`: `docs(effects):` document_module wrappers

Wrapped [`handlers.rs`](../../../fp-library/src/types/effects/handlers.rs),
[`interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs),
and [`member.rs`](../../../fp-library/src/types/effects/member.rs)
in `#[fp_macros::document_module]` + `mod inner { ... }` +
`pub use inner::*;` per user request. Each item gained the
required attribute markers (`document_signature`,
`document_type_parameters`, `document_parameters`,
`document_returns`, `document_examples`).

What diverged:

- **`#[document_parameters]` cannot annotate impl blocks with
  no methods that take a receiver.** The
  `Handler::<E, F>::new` block has only an associated function
  (no `&self`/`self`), so `document_parameters` is omitted.
- **Wrapping in `mod inner { ... }` brings inner items out of
  scope for module-level (`//!`) doc-link references.** Module
  docs that previously referenced `[`HandlersNil`]` etc. are
  rewritten to use full crate paths
  (`[`HandlersNil`](crate::types::effects::handlers::HandlersNil)`).

#### `3a5a0a8`: `fix(macros):` reject trivial assertions

Tightened
[`#[document_examples]`](../../../fp-macros/src/documentation/document_examples.rs)
validation to reject six trivially-true assertion patterns
(`assert!(true)`, `debug_assert!(true)`,
`assert_eq!(true, true)`, `assert_eq!((), ())`,
`assert_ne!(true, false)`, `assert_ne!(false, true)`). Even if
a code block contains a meaningful assertion alongside a
trivial one, the trivial form is treated as noise and rejected.

Refactored 19 existing trivial-assertion doctests across
fp-library + 5 in fp-macros to meaningful assertions:

- 6 Free-family Drop tests construct a post-drop value and
  assert via `resume()` / `evaluate()`.
- 5 interpreter.rs `dispatch` examples exercise via the user-
  facing `*Run::interpret` path with `assert_eq!(result, N)`.
- 6 Run-family `from_*_free` / `Clone` doctests bind the
  constructed value (no underscore prefix) and assert via
  `peel()` / `resume()`.
- 2 Arc helper doctests use `assert!(matches!(...))` rather
  than match-arm `assert!(true)`.
- 2 fixtures use `core::mem::size_of` (uninhabited types are
  size-0).
- 5 fp-macros test fixtures use `assert_eq!(1 + 1, 2)`.

Side effect: `rc_run.rs`'s Clone doctest's `FirstRow` switched
from `CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>`
to `CoproductBrand<IdentityBrand, CNilBrand>` so `peel()` works
(bare `Coyoneda` is `!Clone`, blocking `RcRun::peel`'s
substrate-`Clone` bound).

## Phase 3.5: Pointer-brand-pattern retrofit

### Sub-step 2: three-sibling-types interpretation of effect retrofit

[plan.md sub-step 2](plan.md#phase-35-pointer-brand-pattern-retrofit)
specifies that `Run::get` / `Run::put` / `Run::ask` (and the
`RunExplicit` parallels) "switch from `StateBrand<RcBrand, S>` to
`StateBrand<BoxBrand, S>`" (and similar for Reader / Choose) so the
default `Run` family uses `Box<dyn FnOnce>` continuations via the new
[`ToDynFnOnce`](../../../fp-library/src/classes/to_dyn_fn_once.rs)
trait. Taken literally, this would have the existing
[`StateBrand<P, S>`](../../../fp-library/src/brands/effects.rs)
accept `BoxBrand` as `P`. The implementation diverges: the actual
landed shape introduces parallel sibling brands
[`BoxStateBrand<P, S>`](../../../fp-library/src/brands/effects.rs) /
[`BoxReaderBrand<P, E>`](../../../fp-library/src/brands/effects.rs) /
[`BoxChooseBrand<P>`](../../../fp-library/src/brands/effects.rs) plus
parallel sibling types
[`BoxState<'a, P, S, A>`](../../../fp-library/src/types/effects/state.rs) /
[`BoxReader<'a, P, E, A>`](../../../fp-library/src/types/effects/reader.rs) /
[`BoxChoose<'a, P, A>`](../../../fp-library/src/types/effects/choose.rs)
each bounded by `where P: ToDynFnOnce` (which only `BoxBrand`
satisfies). `Run::get` / `Run::put` / `Run::ask` now thread
`BoxStateBrand<BoxBrand, A>` / `BoxReaderBrand<BoxBrand, A>` (no smart
constructor for `BoxChooseBrand` because `Choose` ships only on
multi-shot wrappers per the
[2026-05-03 wrapper-parameterization resolution](resolutions.md);
`BoxChoose` exists for substrate uniformity).

**Rationale for the three-sibling interpretation rather than the
literal "single brand parametrised over `P`" reading:**

1. `StateBrand<P, S>`'s existing variant types are
   `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A>` and the
   companion `Fn(()) -> A`. `BoxBrand` does NOT implement
   [`RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
   because `Box<dyn Fn>` is not `Clone`. Generalising the `where`
   bound on `StateBrand` would not reach `BoxBrand`; it would only
   accept the same `Rc` / `Arc` brands the original bound already
   accepts.

2. The structural mismatch is deeper than the bound: the closure
   trait _shape_ differs per pointer brand. `BoxBrand` needs
   `dyn FnOnce(S) -> A` (single-shot); `RcBrand` needs
   `dyn Fn(S) -> A` (multi-shot, cloneable); `ArcBrand` needs
   `dyn Fn(S) -> A + Send + Sync`. A unified `State<P, S, A>` would
   require either GAT-over-closure-traits (not in stable Rust) or a
   new abstraction trait `ClosureCarrier` whose `make_closure<F: ???>`
   bound differs per impl (not expressible in stable Rust either).

3. The three-sibling pattern has direct precedent: Phase 3 step 5a.4
   introduced
   [`SendStateBrand`](../../../fp-library/src/brands/effects.rs) and
   [`SendState`](../../../fp-library/src/types/effects/state.rs) as
   siblings to the original `StateBrand` / `State` for exactly the
   same structural reason (Arc's `Send + Sync` trait object differs
   structurally from Rc's; see
   [the 2026-05-03 option-(c) resolution](resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)).
   The Arc-substrate Run wrappers thread `SendStateBrand<ArcBrand, S>`
   instead of `StateBrand<ArcBrand, S>`. The Phase 3.5 retrofit
   extends this to a third sibling: the `Box`-substrate Run wrappers
   thread `BoxStateBrand<BoxBrand, S>`.

4. plan.md's `StateBrand<BoxBrand, S>` notation is loose shorthand
   for "the State-family brand parameterised by `BoxBrand`". The
   actual artifact name (`BoxStateBrand`) keeps the family naming
   convention `<Property>StateBrand<P, S>`: empty prefix for
   Rc-substrate, `Send` for Arc-substrate (`Send + Sync` property),
   `Box` for default-substrate (`FnOnce` property).

**Per-sibling design choices:**

- `Functor` impls on `BoxStateBrand` / `BoxReaderBrand` /
  `BoxChooseBrand` are specialised to `BoxBrand` (rather than
  generic over `P: ToDynFnOnce`) because the body invokes the stored
  continuation directly via `k(arg)`, which requires the projection
  `<P as Pointer>::Of<'_, dyn FnOnce(...) -> A>` to itself implement
  [`FnOnce`]. The standard library provides this implementation only
  for [`Box`] (the
  [`impl<F: ?Sized + FnOnce<Args>> FnOnce<Args> for Box<F>`](https://doc.rust-lang.org/stable/core/ops/trait.FnOnce.html#impl-FnOnce%3CArgs%3E-for-Box%3CF,+A%3E)
  blanket impl on `Box`). Generalising would require extending
  [`ToDynFnOnce`](../../../fp-library/src/classes/to_dyn_fn_once.rs)
  with a `call_once` trait method that consumes the projection, which
  goes beyond Phase 3.5 sub-step 1's surface and is not needed since
  `BoxBrand` is the only valid `P`.

- Three new types ship without `Clone` impls (the `Send` / non-`Send`
  siblings keep their existing `Clone` impls): `Box<dyn FnOnce>` is
  not `Clone`, and the default Run wrapper's program-tree is also
  not `Clone`, so cloning the effect would have no compatible
  consumer. This is the structural reason `Choose` does not ship a
  smart constructor for `BoxChoose` on `Run` / `RunExplicit`: a
  `Choose` handler invokes the continuation twice.

- No `SendFunctor` impls on the three new brands: `BoxBrand`'s
  projection lacks `Send + Sync` bounds in the trait object, and the
  default Run wrappers are single-thread by design.

**Test impact and migration:**

[`fp-library/tests/run_state.rs`](../../../fp-library/tests/run_state.rs)
and
[`fp-library/tests/run_reader.rs`](../../../fp-library/tests/run_reader.rs)
already cover all six wrappers. The retrofit updates the
`RunStateRow` and `RunReaderRow` type aliases (used by both `Run` and
`RunExplicit` tests) from `StateBrand<RcBrand, i32>` /
`ReaderBrand<RcBrand, i32>` to `BoxStateBrand<BoxBrand, i32>` /
`BoxReaderBrand<BoxBrand, i32>`. The handler closures in
`Run::interpret` / `RunExplicit::interpret` calls switch from `State`
/ `Reader` to `BoxState` / `BoxReader` operands, with the closure
body invocation simplified from `(*k)(arg)` (Rc-deref-then-call) to
`k(arg)` (Box's blanket `FnOnce` impl, consuming the box on the
single call). `RcRun` / `RcRunExplicit` / `ArcRun` /
`ArcRunExplicit` test handlers stay unchanged.

The
[`run_smart_constructor_type_mismatch.rs`](../../../fp-library/tests/ui/run_smart_constructor_type_mismatch.rs)
compile_fail UI test (Phase 3 step 7) is updated to use
`BoxReaderBrand<BoxBrand, String>` in the row, mirroring the new
shape of `Run::ask`'s smart constructor signature; the test still
demonstrates the `String != i32` mismatch via row/ascription
unification. Sibling `.stderr` regenerated via `TRYBUILD=overwrite
cargo test --test compile_fail`.

`Choose<BoxBrand, A>` is structurally not exposed via any smart
constructor on `Run` / `RunExplicit` per plan.md sub-step 2's note
("the retrofit still defines it for substrate uniformity but no
smart constructor exposes it"); the type is reachable only through
direct user code constructing a `BoxChoose` value.
