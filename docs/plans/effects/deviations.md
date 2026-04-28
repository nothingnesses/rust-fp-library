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
     macro. The Run-scoping is incidental — any wrapper type
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
  in `ref` mode, and produces simpler desugarings — a
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
     wrapping closure — this works without requiring the
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
