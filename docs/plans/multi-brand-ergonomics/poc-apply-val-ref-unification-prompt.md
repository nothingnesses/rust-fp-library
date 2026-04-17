# POC prompt: Unified Val/Ref dispatch for apply

Use this prompt to instruct an agent to research and validate
approaches for unifying `apply` (owned containers, `Fn(A) -> B`)
and `ref_apply` (borrowed containers, `Fn(&A) -> B`) into a single
dispatch function.

## Context for the agent

The fp-library crate at `/home/jessea/Documents/projects/rust-fp-lib`
has two separate type classes for applicative function application:

- `Semiapplicative::apply` (Val): takes owned containers, wrapped
  functions have type `<FnBrand as CloneFn<Val>>::Of<A, B>` which
  normalizes to e.g. `Rc<dyn Fn(A) -> B>`. Requires `A: Clone`.
- `RefSemiapplicative::ref_apply` (Ref): takes borrowed containers,
  wrapped functions have type `<FnBrand as CloneFn<Ref>>::Of<A, B>`
  which normalizes to e.g. `Rc<dyn Fn(&A) -> B>`. No `A: Clone`
  needed.

The `CloneFn` trait is already parameterized by Mode:

```rust
pub trait CloneFn<Mode: ClosureMode = Val> {
    type Of<'a, A: 'a, B: 'a>: ...;
}
```

`RcFnBrand` implements both `CloneFn<Val>` and `CloneFn<Ref>`.

Other dispatch modules (FunctorDispatch, BindDispatch, etc.) unify
Val/Ref via a Marker type parameter:

- Val impl: `Self = Brand::Of<A>`, closure is `Fn(A) -> B`
- Ref impl: `Self = &Brand::Of<A>`, closure is `Fn(&A) -> B`
- Marker is projected from `<FA as Slot<Brand, A>>::Marker`

The current `dispatch/semiapplicative.rs` handles Val only. The
challenge for unification is that apply's FnBrand bound depends on
the Marker: Val needs `FnBrand: CloneFn<Val>`, Ref needs
`FnBrand: CloneFn<Ref>`. The FnBrand bound cannot be written in
the inference wrapper without knowing the Marker, which is itself
being inferred.

A unified `ApplyDispatch` trait IS feasible since the Val and Ref
impls are keyed on different `(Self, Marker)` pairs and don't
overlap. The open question is how the inference wrapper resolves the
correct `CloneFn` mode.

Key files to read for context:

- `fp-library/src/dispatch/semiapplicative.rs` - current Val-only
  dispatch with FnBrandSlot inference
- `fp-library/src/dispatch/functor.rs` - example of Val/Ref unified
  dispatch via FunctorDispatch
- `fp-library/src/dispatch/semimonad.rs` - another example, includes
  the closureless `join` pattern
- `fp-library/src/classes/semiapplicative.rs` - Val type class
- `fp-library/src/classes/ref_semiapplicative.rs` - Ref type class
- `fp-library/src/classes/clone_fn.rs` - CloneFn trait with Mode
- `fp-library/src/dispatch.rs` - Val, Ref, ClosureMode definitions
- `fp-library/tests/slot_apply_poc.rs` - existing POC for
  Slot-based apply inference (Val + Ref inference validation)
- `fp-library/tests/poc_fn_brand_inference.rs` - FnBrandSlot POC

## Prompt

```
Research and validate approaches for unifying Val and Ref dispatch
for `apply` into a single inference wrapper function. Write POC
tests to fp-library/tests/poc_apply_unified_dispatch.rs.

The working directory is /home/jessea/Documents/projects/rust-fp-lib.
Use `just test -p fp-library --test <test_name>` to run tests. Files
use hard tabs for indentation.

The POC file must be self-contained: explain the background,
hypothesis, and findings entirely within the file. Do not reference
plan documents, decision IDs, or external file paths.

Start by reading the context files listed above to understand:
- How FunctorDispatch unifies Val/Ref (the established pattern)
- How the current semiapplicative dispatch works (Val only)
- How CloneFn<Mode> relates Val and Ref function wrapping
- How FnBrandSlot infers FnBrand from the concrete wrapper type

Then investigate these approaches (and any others you discover):

**Approach A: ApplyDispatch trait with Mode as associated type**

Define an ApplyDispatch trait where the Marker type parameter
selects the impl. Val impl bounds FnBrand on CloneFn<Val>, Ref impl
bounds on CloneFn<Ref>. The inference wrapper bounds FnBrand on
both CloneFn<Val> + CloneFn<Ref> (since RcFnBrand and ArcFnBrand
implement both), and lets the dispatch trait handle the Mode
selection internally.

Test: can the solver resolve Marker from Slot, pick the correct
impl, and have the FnBrand bound satisfied?

**Approach B: Mode type parameter on the inference wrapper**

Add a Mode type parameter to the inference wrapper that is linked
to the Marker via a trait bound. When Marker = Val, Mode = Val;
when Marker = Ref, Mode = Ref. Then bound FnBrand on
CloneFn<Mode>. This requires a way to derive Mode from Marker
(they might already be the same types).

Test: does this introduce any inference ambiguity? Can Val and Ref
still be resolved from Slot?

**Approach C: CloneFn bound on the dispatch trait, not the wrapper**

Move the CloneFn bound entirely into the dispatch trait impls.
The inference wrapper only requires FnBrand to be inferred via
FnBrandSlot (which doesn't depend on Mode). The dispatch trait's
Val impl adds its own CloneFn<Val> bound, and the Ref impl adds
CloneFn<Ref>.

Test: does the solver accept a dispatch call where FnBrand's
CloneFn bound comes from the impl, not the caller?

**Approach D: Marker IS Mode**

Check whether Val and Ref (the dispatch markers) can be directly
used as the ClosureMode parameter. If Val implements ClosureMode
(producing Fn(A) -> B) and Ref implements ClosureMode (producing
Fn(&A) -> B), then the inference wrapper can bound
FnBrand: CloneFn<Marker> where Marker comes from the Slot
projection.

Test: does this already work with the existing ClosureMode impls?

**For each approach:**
1. Write the trait/function signatures
2. Write test cases for:
   - Val + single-brand (Option)
   - Ref + single-brand (Option)
   - Val + multi-brand (Result)
   - Ref + multi-brand (Result)
3. If it compiles and tests pass, note success
4. If it fails, note the exact error and analyze why

**Also consider:**
- Whether FnBrandSlot needs a Ref variant (the current impls
  match Rc<dyn Fn(A) -> B> but not Rc<dyn Fn(&A) -> B>)
- Whether the Into bridge bounds from the current implementation
  still work in the unified case
- Whether the A: Clone bound can be conditional (only for Val)

**After all approaches, update the Finding section with:**
- Which approaches work and which fail
- The recommended approach with rationale
- Any caveats or limitations
- Whether FnBrand inference (via FnBrandSlot) works in the Ref
  case as well as Val
```
