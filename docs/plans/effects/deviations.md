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
