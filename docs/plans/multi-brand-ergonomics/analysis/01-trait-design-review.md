---
title: Trait Design Review
reviewer: Agent 1
date: 2026-04-17
scope: Slot trait shape, coherence, GATs, &T blanket, Marker design
---

# Trait Design Review

## 1. Discrepancy between `brand-dispatch-traits.md` and the adopted Slot shape

The docs at `fp-library/docs/brand-dispatch-traits.md` describe `Slot_*` as
carrying a `type Out<B>` GAT for slot replacement. The plan and the adopted
POC (`slot_marker_via_slot_poc.rs`) define a different Slot that carries only
`type Marker` and no `Out<B>`. Meanwhile, the earlier `slot_production_poc.rs`
defines a Slot with `type Out<B: 'a>: 'a` and no Marker.

These are two structurally different traits serving different purposes, both
called "Slot". The plan never reconciles them. The `brand-dispatch-traits.md`
describes the `Out<B>` variant as the planned Slot, including a blanket from
`InferableBrand`, but Decision D eliminates `InferableBrand`, which would
invalidate that blanket and the entire section titled "Why `Slot_*` does not
replace `InferableBrand_*`".

**Issue:** The plan ships the Marker-only Slot and removes `InferableBrand`,
but does not specify what happens to the `Out<B>` GAT functionality or to
`brand-dispatch-traits.md`, which describes a contradictory design. The
integration surface table mentions updating `brand-dispatch-traits.md` but
says only "Update to reflect single-trait-family design", which is
insufficient given the structural conflict.

**Approaches:**

1. **Merge both associated types into one trait.** The production Slot carries
   both `type Marker` and `type Out<B: 'a>: 'a`. The `&T` blanket sets
   `Marker = Ref` and forwards `Out<B>` to the inner type's `Out<B>`. This
   preserves both capabilities in one trait. Trade-off: the trait becomes
   heavier, and every impl must specify both; but the macro generates them, so
   the cost is borne by `impl_kind!` not by users.

2. **Drop `Out<B>` entirely.** Return types already go through
   `<Brand as Kind>::Of<'a, B>`, so `Out<B>` is redundant for dispatch
   wrappers. The `slot_production_poc.rs` was an exploration that the Marker
   POC superseded. Trade-off: loses the ability to express "replace just this
   slot" without knowing the Brand, but since Brand is always resolved by the
   time dispatch runs, this is not needed.

3. **Keep them as separate traits.** `Slot_*` for Marker-based dispatch,
   a separate `SlotReplace_*` or similar for the `Out<B>` GAT. Trade-off:
   proliferates the trait family further.

**Recommendation:** Option 2. The plan's adopted POC already works without
`Out<B>`, and dispatch wrappers project the return type through
`<Brand as Kind>::Of<'a, B>`. Update `brand-dispatch-traits.md` to remove
the `Out<B>` description and the `InferableBrand` blanket section, replacing
both with the Marker-only design from the adopted POC.

## 2. Marker projection depends on Brand and A being resolvable

The plan claims (line 76-78) that "When dispatch code projects
`<FA as Slot<...>>::Marker`, the Marker commits from FA alone, before
`(Brand, A)` are resolved." This is the central design claim, but it
requires careful qualification.

Rust's associated type projection `<FA as Slot<'a, Brand, A>>::Marker`
syntactically names Brand and A. The projection can normalise only if the
compiler can identify which impl applies. For this to happen from "FA alone",
the compiler must be able to determine that Marker has the same value across
all Slot impls that FA could satisfy. This works because:

- If FA is `&T` for some T, the blanket impl is the only match for `&T` as
  Self, and it unconditionally sets Marker = Ref regardless of Brand and A.
- If FA is an owned concrete type, all its Slot impls set Marker = Val.

This reasoning is correct for the current impl landscape but relies on an
invariant: **every owned Slot impl for a given Self type must agree on the
same Marker value.** The plan never states this invariant explicitly, and the
macro generation path has no enforcement mechanism.

**Issue:** If a future type were to have some Slot impls with Marker = Val and
others with Marker = Ref (pathological, but not prevented by the trait
definition), the Marker projection would become ambiguous and break inference.

**Approaches:**

1. **Document the invariant and enforce it in `impl_kind!`.** The macro
   already controls all Slot generation. Add a compile-time assertion or
   simply never generate mixed Marker values for the same Self type. This is
   the cheapest option. Trade-off: purely convention-based if someone writes
   manual Slot impls.

2. **Split Slot into two traits: one for Marker resolution (no Brand param)
   and one for brand keying.** This structurally prevents mixed Marker values
   per Self. Trade-off: adds a trait, complicates the design, and may
   reintroduce the coherence issues that led to the current shape.

3. **Use a sealed trait for Marker.** Make `Marker` a sealed associated type
   that can only be `Val` or `Ref`, and add a separate `IsOwned` / `IsRef`
   trait that Slot impls must be consistent with. Trade-off: more machinery
   for a low-probability failure mode.

**Recommendation:** Option 1. The invariant holds naturally for all types the
library controls, and `impl_kind!` is the sole generator. Document the
invariant in the Slot trait's rustdoc and add a comment in `impl_kind!`
explaining that all Slot impls for a given Self must agree on Marker.

## 3. `&T` blanket and nested references / smart pointers

The `&T` blanket is:

```rust
impl<'a, T: ?Sized, Brand, A: 'a> Slot<'a, Brand, A> for &T
where T: Slot<'a, Brand, A>, Brand: Kind { type Marker = Ref; }
```

This covers single-level `&Option<A>`, `&Vec<A>`, etc. However:

**3a. Nested references (`&&Option<A>`):**
The blanket applies recursively: `&&Option<A>` matches with `T = &Option<A>`,
which in turn matches with `T = Option<A>`. Marker = Ref at every level.
The FunctorDispatch Ref impl takes `FA = &'b Apply!(Brand::Of<'a, A>)`,
meaning it expects exactly one level of reference. If someone passes
`&&Some(5)` to `map`, FA would be `&&Option<i32>`, which does not match the
Ref impl's FA pattern of `&Option<i32>`. So `map(|x: &i32| ..., &&opt)`
would fail to compile. This is probably correct behaviour, but the plan does
not discuss it.

**3b. Smart pointers (Box, Rc, Arc):**
`Box<Option<A>>`, `Rc<Option<A>>`, `Arc<Option<A>>` do not implement Slot
and there is no blanket for them. Users cannot write
`map(|x: i32| x+1, Box::new(Some(5)))`. This is fine because the library
does not treat smart-pointer-wrapped containers as functors. However, the
library does use `Rc` and `Arc` internally for function wrapping (FnBrand).
The Slot impl for `apply`'s `ff` parameter keys on the inner payload type,
not on the smart pointer wrapper, so this should not cause issues.

\*\*3c. `&dyn Trait` and `&(dyn Slot<...> + 'a)`:
The `?Sized` bound on T means `&dyn SomeTrait` could in theory satisfy the
blanket if the underlying trait object implements Slot. In practice, branded
container types are always `Sized`, so this is not a concern.

**Issue:** Only 3a is potentially surprising to users. The rest are
non-issues.

**Approaches:**

1. **Document that only one level of `&` is supported by dispatch.** This is
   already implicit in FunctorDispatch's Ref impl shape, but worth stating
   explicitly in the Slot trait docs.

2. **Add a negative test.** A compile-fail test for `map(f, &&some_val)` with
   a clear error message would confirm the behaviour is intentional.

3. **Do nothing.** The type system already prevents misuse; the Ref
   FunctorDispatch impl simply will not match `&&T`.

**Recommendation:** Option 2. Add a compile-fail UI test during phase 1 to
lock in the expected behaviour and ensure the error message is reasonable.

## 4. `'static` bounds on multi-brand Slot impls

The Result Slot impls use `'static` bounds on the "other" type parameter:

```rust
impl<'a, A: 'a, E: 'static> Slot<'a, ResultErrAppliedBrand<E>, A> for Result<A, E> { ... }
impl<'a, T: 'static, A: 'a> Slot<'a, ResultOkAppliedBrand<T>, A> for Result<T, A> { ... }
```

These `'static` bounds are necessary for coherence: without them, the two
impls would overlap when `A = E` (or `A = T`) because both would apply to
`Result<X, X>` for arbitrary X. The `'static` bound on the non-active
parameter breaks the symmetry enough for the coherence checker.

**Issue:** The `'static` constraint means `Result<A, E>` where E contains
references cannot use the inference path via `ResultErrAppliedBrand<E>`. For
example, `Result<i32, &str>` where the user wants to `map` over the Ok side
would need `&str: 'static`, which holds, so this specific case is fine. But
`Result<i32, &'a str>` with a non-static lifetime `'a` would not satisfy
`E: 'static` and would fall back to `explicit::`.

This mirrors the existing `Kind` impls for `ResultErrAppliedBrand<E>` which
likely already carry `E: 'static`. So this is not a new restriction introduced
by Slot.

**Approaches:**

1. **Accept the `'static` constraint as inherent.** It follows from coherence
   requirements and matches the existing Kind impls. Document it.

2. **Explore lifetime-parameterised brand types** like
   `ResultErrAppliedBrand<'a, E>` that could weaken the bound to `E: 'a`.
   Trade-off: changes the Brand type signature, which ripples through the
   entire Kind/Slot/dispatch machinery. Substantial rework for a marginal
   gain.

3. **Use negative impls (unstable) to break overlap without `'static`.**
   Not viable on stable Rust.

**Recommendation:** Option 1. The `'static` bound is already present in the
existing Kind impls and is a known trade-off. Document it in the Slot trait
docs and in the multi-brand closure annotation docs so users understand why
`Result<i32, &'a str>` requires `explicit::`.

## 5. Marker = Val / Marker = Ref does not cover Pin<&mut T>

The current dispatch model has exactly two modes: owned (`Val`) and
shared-reference (`Ref`). There is no `&mut T` or `Pin<&T>` / `Pin<&mut T>`
mode. For `&mut T`, the `&T` blanket does not apply (it matches only `&T`,
not `&mut T`). There is no blanket for `&mut T`.

**Issue:** If the library ever wants to support `map` over `&mut Container`,
the current Val/Ref binary is insufficient. This is not a blocking issue for
the plan as written, since mutable-reference dispatch is not in scope. But
the Marker design bakes in a two-valued enum, and extending it later would
require adding a third Marker variant and a third FunctorDispatch impl.

**Approaches:**

1. **Acknowledge as a future concern.** The plan's scope explicitly excludes
   `&mut` dispatch, and adding it later is mechanically possible by adding a
   `MutRef` marker, a `&mut T` blanket, and a third dispatch impl. No design
   decision needed now.

2. **Make Marker an open associated type now** rather than restricting it to
   Val/Ref. This would allow future extension without changing Slot's trait
   shape. Trade-off: loses the property that Marker is a closed two-valued
   type, which the solver may rely on for exhaustive dispatch.

3. **Seal the Marker type.** Add `Val`, `Ref`, and `MutRef` now even if
   MutRef is unused. Trade-off: dead code and a third FunctorDispatch impl
   that does nothing.

**Recommendation:** Option 1. This is a non-issue for the plan as written.
If `&mut` dispatch is needed in the future, the extension path is
straightforward. No action needed now.

## 6. GAT limitations: associated type projection through the `&T` blanket

The `slot_production_poc.rs` variant (with `Out<B>` GAT) delegates Out
through the reference blanket:

```rust
impl<...> Slot<'a, Brand, A> for &T where T: Slot<'a, Brand, A> {
    type Out<B: 'a> = <T as Slot<'a, Brand, A>>::Out<B>;
}
```

This is a GAT that forwards to another GAT through a where-clause. Known
rustc issues with GAT normalisation (e.g., rust-lang/rust#100013 on
incomplete normalisation of GATs under certain trait-bound configurations)
could affect this pattern.

However, the adopted design (Marker-only Slot) does not use a GAT at all.
`type Marker` is a plain associated type, not generic. The GAT concerns
apply only if Option 1 from Issue 1 is chosen (merging Out<B> into Slot).

**Issue:** If the final Slot trait includes `Out<B>`, the `&T` blanket's GAT
forwarding is a potential site for compiler bugs. If Slot carries only
Marker (a plain associated type), this issue does not apply.

**Approaches:**

1. **Stay with Marker-only Slot (no GAT).** The return type is computed via
   `<Brand as Kind>::Of<'a, B>`, which is a GAT on Kind, not on Slot. Kind's
   GAT is already in production and exercised by the entire library. No new
   GAT surface.

2. **If Out<B> is needed, test thoroughly.** The `slot_production_poc.rs`
   already validates the forwarding pattern compiles and normalises on the
   current toolchain (rustc 1.94.1). Pin the MSRV and add regression tests.

3. **Use a type alias instead of a GAT.** Not possible in trait associated
   types without GATs.

**Recommendation:** Option 1. Keep the Marker-only Slot design. This avoids
introducing any new GAT surface area in Slot. The existing Kind GATs are
battle-tested within the library.

## 7. Coherence with the `&T` blanket across crates

The `&T` blanket is:

```rust
impl<'a, T: ?Sized, Brand, A: 'a> Slot<'a, Brand, A> for &T
where T: Slot<'a, Brand, A>, Brand: Kind { type Marker = Ref; }
```

This is a blanket impl parameterised over T, Brand, and A. Downstream crates
cannot add Slot impls for `&TheirType` because the blanket already covers all
`&T`. This is the intended behaviour, and coherence is satisfied because the
blanket is defined in the same crate as the trait.

However, downstream crates also cannot add Slot impls for their own owned
types, because Slot is defined in `fp-library` and downstream types are
foreign. The `impl_kind!` macro must be invoked from within `fp-library` or
from a crate that has a blanket-generating macro. This is the same constraint
as the existing Kind/InferableBrand system, so it is not a new issue.

**Issue:** No new coherence concern. The `&T` blanket is structurally sound
under Rust's orphan rules. Downstream extensibility is limited but this
matches the existing design.

**Recommendation:** No action needed. This is a non-issue.

## 8. Trait family generalisation across Kind arities

The plan states that one Slot trait is generated per Kind arity, matching the
existing Kind/InferableBrand pattern. The arity-2 POC
(`slot_arity2_poc.rs`) validates this for `Slot2` with `bimap`.

The generalisation is mechanical: `Slot_k` takes Brand plus k type parameters
(matching the k in `Kind_k::Of<A1, ..., Ak>`). Each arity gets its own
`&T` blanket with Marker = Ref.

**Issue:** The Marker-based dispatch must also generalise per arity. For
arity 2, `BimapDispatch` has its own Val/Ref impls, and the Marker from
`Slot2` projects correctly (validated by POC 6). For arity 2, the Slot
bound in the bimap signature uses two of the type parameters (A and C in
the POC, corresponding to the "pre-transform" types), not all four. The
choice of which parameters appear in the Slot bound versus which appear only
in the dispatch trait is a design decision that must be made consistently
per operation.

The POC demonstrates this for bimap:
`FA: Slot2<'a, Brand, A, C>` where A and C are the inputs, and B and D are
the outputs (appearing only in BimapDispatch). This pattern is sound and
mirrors how the Kind signature works (Kind2::Of takes two params, bimap
transforms both).

**Approaches:** No approaches needed; the generalisation works. The only note
is that each new arity requires a new `&T` blanket impl, which `trait_kind!`
should generate automatically.

**Recommendation:** Verify during implementation that `trait_kind!` generates
the `&T` blanket for each arity it emits Slot for. This is a mechanical check,
not a design concern.

## 9. Decision D and the loss of unique-brand assertion

Decision D removes `InferableBrand` entirely. The `brand-dispatch-traits.md`
documents three capabilities that only `InferableBrand` provides:

1. Unique-brand assertion (`FA: InferableBrand` proves FA has one brand).
2. `FA::Brand` projection for type expressions.
3. Return-type inference for non-closure operations (`pure`, etc.).

The plan acknowledges (in the "Operations that cannot use Slot inference"
section) that `pure`, `alt`, `empty`, and `sequence` remain `explicit::` for
multi-brand types. But for single-brand types, these operations currently work
without turbofish because `InferableBrand` provides the Brand.

**Issue:** After removing `InferableBrand`, how do `pure(5)` (returning
`Option<i32>`) or `empty()` (returning `Vec<i32>`) infer their Brand? The
plan does not address this. If these operations currently rely on
`InferableBrand::Brand` as an associated-type projection in their signature,
removing the trait breaks them for ALL types, not just multi-brand ones.

**Approaches:**

1. **Keep `InferableBrand` for non-closure operations.** Slot handles
   closure-directed dispatch; InferableBrand handles return-type-directed
   dispatch. This contradicts Decision D but preserves the existing inference
   for `pure`, `empty`, etc. Trade-off: two trait families remain, which
   the plan wanted to eliminate.

2. **Move the unique-brand projection into Slot with a default.** Add a
   `type Brand` associated type to Slot that defaults to the Brand trait
   parameter. For single-brand types with exactly one Slot impl, the
   projection could work. Trade-off: Rust does not support "there is exactly
   one impl" reasoning, so this does not actually work.

3. **Accept that `pure`, `empty`, etc. require `explicit::` for ALL types
   after InferableBrand removal, then introduce a separate `UniqueBrand`
   trait (essentially InferableBrand renamed) that is kept specifically for
   these operations.** Trade-off: this IS InferableBrand under a new name,
   which contradicts Decision D's "eliminate entirely" framing.

4. **Retain InferableBrand but rename it and reframe it as a companion to
   Slot, not a competitor.** Decision D's motivation is eliminating the
   parallel trait family, but InferableBrand and Slot serve non-overlapping
   purposes. Keeping both with clear documentation of their roles is simpler
   than removing one and losing functionality.

**Recommendation:** Option 4, or a clarification that Decision D means
"InferableBrand is no longer used by closure-taking operations" rather than
"InferableBrand is deleted". The plan should explicitly state what happens to
`pure(5)`, `empty()`, `sequence(xs)` for single-brand types. If these
currently work without turbofish, removing InferableBrand would be a
regression. This is the most significant gap in the plan.

## 10. Marker projection timing claim

The plan states (line 76-78): "the Marker commits from FA alone, before
`(Brand, A)` are resolved."

This claim is about the Rust trait solver's evaluation order, which is not
formally specified. The claim holds empirically (validated by POCs on rustc
1.94.1) but relies on the solver's current behaviour: when it sees
`<FA as Slot<'a, Brand, A>>::Marker` and FA is concrete (e.g., `&T` or
`Option<A>`), it can determine Marker without fully resolving Brand and A
because all matching impls agree on Marker's value.

**Issue:** This is implementation-defined behaviour of rustc's trait solver.
The new trait solver (currently under development, tracked at
rust-lang/rust#107374) may change evaluation order. If the new solver
requires Brand and A to be resolved before projecting Marker, the entire
design breaks.

**Approaches:**

1. **Accept the risk and track the new solver.** The design works today and
   on all stable rustc versions. If the new solver changes behaviour, the
   issue will surface during the solver's stabilisation period (which is
   years away). Trade-off: potential future rework.

2. **Add a CI job testing against nightly with the new solver enabled**
   (`-Znext-solver`). This gives early warning. Trade-off: nightly CI is
   noisy and may produce false positives.

3. **Restructure to avoid depending on projection timing.** Instead of
   projecting Marker through Slot, use a separate trait `IsRef<FA>` that
   resolves Marker independently of Brand/A. Trade-off: adds a trait, and
   the `&T` blanket must be duplicated.

**Recommendation:** Option 1, with option 2 as a low-cost supplement. The
POCs validate the design on current stable rustc. Adding a periodic
nightly-new-solver CI check is cheap insurance. The design is sound under the
current solver, and restructuring pre-emptively for an unspecified future
solver change would be premature.

## Summary of issues by severity

| #   | Issue                                                  | Severity      | Recommendation                                                               |
| --- | ------------------------------------------------------ | ------------- | ---------------------------------------------------------------------------- |
| 9   | Loss of InferableBrand breaks `pure`/`empty` inference | High          | Clarify Decision D scope; likely need to retain InferableBrand or equivalent |
| 1   | Slot shape discrepancy between docs and adopted POC    | Medium        | Drop Out<B> from Slot; update brand-dispatch-traits.md                       |
| 2   | Undocumented Marker-agreement invariant                | Medium        | Document and enforce in impl_kind!                                           |
| 10  | Marker projection relies on solver evaluation order    | Medium        | Accept risk; add nightly-new-solver CI                                       |
| 4   | `'static` bounds on multi-brand Slot impls             | Low           | Document as known limitation                                                 |
| 3   | Nested `&&T` behaviour undocumented                    | Low           | Add compile-fail UI test                                                     |
| 6   | GAT concerns if Out<B> is added to Slot                | Low           | Keep Marker-only design to avoid                                             |
| 5   | No `&mut T` Marker variant                             | Informational | Future concern; no action now                                                |
| 7   | Coherence and downstream extensibility                 | Informational | Non-issue; matches existing design                                           |
| 8   | Arity generalisation                                   | Informational | Works; verify trait_kind! generates blankets                                 |
