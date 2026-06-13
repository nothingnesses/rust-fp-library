# Ideas From External Effect Systems

Assessment of the three newly cloned Rust repositories, then the wider reference directory. Each idea is judged for this library specifically (stable Rust, Brand-encoded HKTs, freer core, six-wrapper matrix).

## 1. rust-effects-denful

What it is: a dependency-free freer monad over string-keyed effects (`Effect { name: String, param: Arc<dyn Any> }`) with an FTCQueue continuation queue, a trampolined terminal interpreter, and a `rotate` scoped interpreter that forwards unknown effects outward with a resume-the-scope closure.

Worth adopting:

- The `HandlerResult::Resume`/`Abort` split. Handlers return an explicit verdict: resume the continuation with a value, or abort and discard it. The local system encodes abort implicitly (a handler simply does not call the continuation and returns some other program), which works but is invisible in types and easy to get wrong in user handlers. For the planned public `define_effect!`/handler macros, generating handler signatures that make the abortive case explicit (an enum return, or distinct `resume`/`abort` constructors on a context argument) would make user handlers self-documenting. This pairs well with the single-shot guard discussion (architecture 3.4): an explicit verdict type can also statically prevent double-resume on the Box family.
- The Common Lisp condition/restart system (`conditions.rs`) as a first-order effect: signal a condition, handlers choose a named restart, computation continues without unwinding. It is a genuinely useful error-recovery model distinct from Except/Catch, expressible today as a custom effect; a good candidate for an examples gallery or an optional effect module once `define_effect!` lands.
- `has_handler` (reflectively ask whether an effect is handled) translates to a compile-time idea here: optional effects via row membership, for example a combinator that runs a program fragment only if `EBrand` is in the row. Type-level `Member`-or-not dispatch is awkward in Rust (overlap), so the practical version is two explicit entry points; note it and move on.

Avoid: string-keyed effects and `Arc<dyn Any>` payloads (the local typed rows are categorically better); the single shared handler-state type with lens-based `adapt` composition (the local per-handler closures with captured cells already compose better).

## 2. effect-rs

What it is: a ZIO/Effect-TS-style monolithic async monad, `Effect<A, E, R> = Arc<dyn Fn(Arc<R>) -> BoxFuture<Exit<A, E>>>`, on tokio; typed errors via a `Cause<E>` tree; fibers, interruption, scopes with finalizers. Not an extensible-effects system; no rows, no handlers.

Worth adopting (mostly when the async/parallel story expands):

- `Cause<E>` as an error model: `Fail(E) | Die(defect) | Interrupt | Sequential(..) | Parallel(..)`. Once this library has parallel combinators or finalizers (Bracket release failing after a primary failure), a plain `E` in Except cannot represent "both branches failed" or "the cleanup also failed". A `Cause`-shaped error payload (as a library type usable as the Except `E`) is the proven answer; adopting the shape early costs little and avoids a breaking error-model change later.
- `sandbox`/`unsandbox` (temporarily lift the full cause into the typed error channel) is a clean combinator pattern for handler-level error surgery; cheap to provide once a Cause type exists.
- Interruption masking via lexically scoped flags with innermost-wins semantics (`uninterruptible`/`interruptible`) is the right primitive shape if the async interpreter ever grows cancellation; record it in the async design notes now so the future design does not reinvent it.
- `Effect::block` (a do-notation bridge where `?` works over `Exit`) is a reminder that `im_do!` could support `?`-style early return for Except rows; worth a look when revisiting do-notation ergonomics.

Avoid: the `Arc<dyn Fn>`-per-combinator encoding (every `map` allocates a closure; the local queue-based bind is strictly better); monomorphic `R`/`E` parameters (rows already solve this); taking a tokio dependency in core.

## 3. effect-lite

What it is: despite the name, dependency injection rather than algebraic effects: `trait Effect<D> { type Output; fn resolve(self, dep: D) -> Output }` with zero-cost adapter structs (`MapDependency`, `Then`, `Merge`, `Provide`, `Either`), a no-std executor, and an incomplete futures adapter (two `todo!()`s).

Worth adopting:

- Little, directly. The one transferable idea is `MapDependency` (contravariant adaptation of required capability), which is conceptually the local row-embedding; the zero-cost-adapter-struct style is already how the local optics work.
- `Either<L, R>` for unifying two differently-typed program branches without boxing is a pattern worth remembering for combinator-returning APIs, but the local wrappers are concrete types, so the problem rarely arises.

Avoid: treating it as prior art for effect handlers (no continuations, no handlers, no rows); the incomplete futures layer.

## 4. Survey shortlist (remaining reference projects)

Ranked by usefulness to this library; the rest of the directory (effekt, koka as compilers, eff as GHC-primop research, fused-effects/in-other-words carrier styles, fx-rs product-state style, rust-effects which is unrelated typeclass code) informed the comparisons but yields no direct imports beyond what is noted below.

1. EvEff / koka, the operation taxonomy. Operations classified as tail-resumptive (`function`: resume exactly once, immediately) execute in place with no continuation capture; only true `operation`s pay for capture. This is the theory behind refactoring item R11: most of this library's first-order ops are tail-resumptive by construction, and the benchmark question is whether the generic Coyoneda-lower-plus-positional-dispatch path leaves measurable money on the table versus a fused fast path.
2. MpEff, multi-prompt semantics for multi-shot state. Two specifics: `Ctl`'s continuation composition (the local CatList plays the same role), and `mpromptIORef` (snapshot-and-restore mutable cells across resumptions) as the disciplined way to combine cell-backed handler state with multi-shot continuations. If R5's threaded runners prove hard for some handler, snapshot-restore cells are the documented fallback semantics. Mandatory reading before designing CC/Shift (coverage candidate 8).
3. freer-simple, FTCQueue discipline. The local `CatList`-of-continuations serves the same purpose; the check worth doing is that the amortized left-view rotation is actually O(1) under the boundary-frame detach/reattach pattern (frames carry queues around; repeated `into_free` lowering of a boundary could re-rotate). A targeted benchmark (deep program inside a Catch) settles it.
4. polysemy, `Weaving` and `Scoped`. `Weaving` is the negative example the local design rightly avoids (heftia's elaboration argument applies); read it to confirm the boundary-frame protocol does not accidentally reintroduce functorial-state weaving through the accumulate traversals. `Scoped` is the positive example: the install-a-sub-interpreter-for-a-region pattern behind coverage candidate 9 (Provider).
5. switch-resume, delimited continuations over async. Captures "the rest of the task" as `Box<dyn FnOnce(Arg) -> BoxFuture<T>>` with no unsafe, single prompt. As a substrate experiment it suggests a path to one-shot `Shift` on the Box family via the async driver (the continuation is the rest of the `run_async` future); worth a note in the CC/Shift design round, not before.
6. effing-mad / reffect / corophage, coroutine-based systems (nightly except corophage). Not adoptable as substrate (this library is deliberately freer-based on stable), but two ergonomic ideas transfer: reffect's `#[group]`/`#[group_handler]` macros (define an effect group as a trait, derive the plumbing) are a good shape study for the public `define_effect!`; corophage's GAT `Resume<'r>` (handlers resuming with borrowed data) maps to the Explicit family's lifetime support and is worth considering in the `define_effect!` design so custom Explicit-family effects can have borrowing continuations. corophage's benchmark result (positional coproduct dispatch monomorphizes to a flat switch, position-independent at around 45ns) is also directly relevant calibration for R11: it suggests dispatch position is not the local bottleneck either, and the interesting costs are allocation and queue traffic.
