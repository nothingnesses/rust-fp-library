# W2 Reduction Spike: Generic Wrapper Feasibility

This spike checks whether the six Run wrappers can be reduced to one
wrapper generic over closure storage, pointer brand, or Free substrate.
The goal is to avoid generating the wrapper x effect cross-product if a
smaller hand-written abstraction can express the same semantics.

## Result

Recommendation: proceed with the W2 generator.

A single generic wrapper is not the right architecture for the current
system. The differences between Box, Rc, and Arc are semantic, not just
storage choices:

- Box/default wrappers are single-shot and store `FnOnce`
  continuations.
- Rc wrappers are single-threaded multi-shot and store cloneable `Fn`
  continuations.
- Arc wrappers are thread-safe multi-shot and store
  `Fn + Send + Sync` continuations, with result and row projections that
  must also be `Send + Sync`.
- Explicit wrappers add a lifetime parameter and expose Brand-dispatched
  class coverage that the erased wrappers intentionally do not expose.
- Default `Run` has a private erased scoped-boundary representation that
  keeps selected scoped actions and outer continuations separate.

Those axes force different trait bounds, associated types, continuation
shapes, and user-facing class impls. A generic wrapper can move that
complexity into mode traits, but it cannot remove the per-mode behavior.
The resulting abstraction would still need per-mode impl blocks and
would make documentation, diagnostics, and rust-analyzer output harder to
audit than generated explicit code.

## Attempt 1: one closure-storage trait

A natural reduction is a trait that abstracts over the closure storage
used by effect cells:

```rust
trait ClosureStorage {
	type Once<'a, A, B>: 'a;
	type CloneFn<'a, A, B>: Clone + 'a;
	type SendFn<'a, A, B>: Clone + Send + Sync + 'a;
}
```

That shape does not actually unify the operation surface. The methods
that construct and call these values need different closure traits:

- `BoxBrand` can construct `Box<dyn FnOnce(A) -> B>`.
- `RcBrand` can construct `Rc<dyn Fn(A) -> B>`, but cannot construct a
  usable `Rc<dyn FnOnce(A) -> B>` because calling `FnOnce` consumes the
  trait object out of a shared pointer.
- `ArcBrand` can construct `Arc<dyn Fn(A) -> B + Send + Sync>`, but the
  `Send + Sync` auto-traits are part of the trait object type and must be
  present at construction time.

The existing independent traits (`ToDynFnOnce`, `ToDynCloneFn`, and
`ToDynSendFn`) express this accurately. Collapsing them into one trait
would require optional capabilities or mode-specific methods. At that
point the generic wrapper still branches on the same three modes the
current wrappers make explicit.

## Attempt 2: one Free-family substrate trait

Another reduction is to abstract over the Free family:

```rust
trait RunSubstrate<R, S> {
	type Program<'a, A: 'a>;
	fn pure<'a, A: 'a>(a: A) -> Self::Program<'a, A>;
}
```

This captures only the easiest operation. The real wrappers need
different method bounds:

- `Run` stores `Free<NodeBrand<R, S>, A>` behind a private
  `RunRepresentation<R, S, A>` with `TypeErasedValue` and raw scoped
  boundary frames. Its public type has no lifetime parameter and requires
  `A: 'static`.
- `RunExplicit` stores `FreeExplicit<'a, NodeBrand<R, S>, A>` and must
  keep the lifetime visible in the wrapper type.
- `RcRun` and `RcRunExplicit` need `A: Clone` on many multi-shot
  operations.
- `ArcRun` and `ArcRunExplicit` need `A: Clone + Send + Sync` and
  projection-level `Send + Sync` evidence on operations that store or
  clone continuations.

Stable Rust does not support quantifying over "all result types `A`" in
where-clauses for arbitrary associated type projections. That is the
same per-`A` HRTB-over-types limitation that already caps
`ArcRunExplicitBrand` class coverage. A substrate trait can express
individual inherent methods with per-method bounds, but it cannot produce
one clean Brand-dispatched class matrix across the six wrappers.

## Attempt 3: one wrapper with mode-specific impl blocks

A weaker reduction is possible:

```rust
struct GenericRun<Mode, R, S, A> {
	inner: Mode::Program<R, S, A>,
}
```

The mode would then have separate impls for Box, Rc, Arc, Explicit,
RcExplicit, and ArcExplicit. This keeps one wrapper name but does not
remove the cross-product. The smart constructors, row functor choices,
scoped-boundary carriers, handler bounds, and class impls still split by
mode. It also makes rustdoc and diagnostics worse because users see a
single generic type whose methods appear only under complex mode bounds,
rather than six explicit wrapper types with clear semantics.

This is a worse version of generation: the repetition remains but moves
behind harder-to-read trait machinery.

## Generator Implications

The generator should keep the six wrapper identities explicit and encode
their capability rules declaratively:

- Box/default: single-shot, `FnOnce`, erased boundary support.
- Explicit: single-shot, `FnOnce`, lifetime-carrying explicit substrate,
  Brand-dispatched class coverage where Rust permits it.
- Rc: multi-shot, `Fn`, cloneable continuations.
- RcExplicit: multi-shot, `Fn`, lifetime-carrying explicit substrate,
  Ref class coverage where Rust permits it.
- Arc: multi-shot, `Fn + Send + Sync`, thread-safe row projections.
- ArcExplicit: multi-shot, `Fn + Send + Sync`, lifetime-carrying
  explicit substrate, limited Brand-dispatched class coverage.

Generation is still the preferred approach because it preserves the
clear public surface while making the repeated implementation matrix
declared and mechanically consistent.
