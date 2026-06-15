# POC-8b Findings: the `FnOnce`-versus-`Fn` reconciliation and `bind`/`map` over composed stored closures

Tier E, foundation sweep (follow-up to POC-8). POC-8 proved the substrate type, interpreter, and `Clone` collapse to one `Run<Store: ClosureStorage, A>` on the `Fn`-only layer, and deliberately deferred two things that the substrate-unification decision (items 5 and 6) leans on. This POC builds both.

Charter questions:

1. Can one `ClosureStorage` associated type carry the callable kind so the Box instantiation keeps a `Box<dyn FnOnce>` one-shot spine while Rc/Arc keep reusable `Fn`, all under one substrate, one interpreter, and one `Clone`? This is the load-bearing question for item 6's primary outcome (one brand per effect, no prefix axis); POC-8 left it as the documented stretch with a prefix-scheme fallback.
2. Do `bind`/`map` compose stored closures correctly across the three stores? POC-8 exercised only `pure`/`handle`.

Result: PASS on both. The `FnOnce`/`Fn` reconciliation holds on one substrate; `bind`/`map` work for all three stores. The one recorded boundary is unchanged from POC-8: construction stays per-`Store`, and this POC shows exactly why (`bind`/`map` bodies differ by move-versus-clone).

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_closure_storage_fnonce.rs`. Eleven tests pass. Run via `just filtered test --test poc_fs_closure_storage_fnonce --features effects`. As with POC-8 this is a bespoke pointer-parameterised substrate (the public `Free` is `Box`-spine only).

## What was built

- `ClosureStorage` with `Stored<'a, I, O>` set to `Box<dyn FnOnce>` for Box and `Rc`/`Arc<dyn Fn (+ Send + Sync)>` for Rc/Arc. The associated type now carries the callable kind, not only the pointer.
- `call_once<'a, I, O>(s: Self::Stored<'a, I, O>, i: I) -> O`: the bridge method takes the stored closure by value. For Box this consumes the `FnOnce` and runs it once (`Box<dyn FnOnce>: FnOnce`); for Rc/Arc it borrows the `Fn` through the owned pointer for the call (`&dyn Fn: Fn`), then the pointer drops. One signature, both kinds.
- One `enum Run<Store, A> { Pure(A), Ask(Store::Stored<'static, i32, Run<Store, A>>) }`, one generic `handle`, and one conditional `Clone` impl (`where for<'a> Store::Stored<...>: Clone`), all unchanged in shape from POC-8.
- Per-`Store` `map` and `bind` (`map_box`/`map_rc`/`map_arc`, `bind_box`/`bind_rc`/`bind_arc`): the Box bodies move the captured continuation into a `FnOnce` closure run once; the Rc/Arc bodies clone it because their composed closures are `Fn` and cannot move a capture out on each call.

Eleven tests pass, covering: one substrate holding a Box `FnOnce` program and Rc/Arc `Fn` programs through one `handle`; a Box continuation that genuinely moves a non-`Copy` capture out when run (`String::into_bytes`, so it could not be stored by an `Fn`-only GAT); an Rc continuation cloned and run three times (multi-shot reuse); the Arc instantiation statically `Send + Sync`; `map` and `bind` for all three stores; and a non-trivial bound Rc program cloned and both copies run.

## Findings

1. One `ClosureStorage` associated type carries the callable kind. The same `Stored` GAT is `FnOnce` for Box and `Fn` for Rc/Arc, and the by-value `call_once` bridges both: a `FnOnce` is consumed and run once, an `Fn` is borrowed through the owned pointer then dropped. So the Box instantiation keeps the real `Box<dyn FnOnce>` one-shot spine (the `box_continuation_is_genuinely_fnonce` test stores a closure that consumes a `String`, which an `Fn` cannot) while Rc/Arc keep reusable `Fn`, all under one `Run`, one `handle`, and one conditional `Clone`. This is the reconciliation POC-8 deferred; it works without the prefix-scheme fallback.

2. Multi-shot is the conditional `Clone`, not a second method. Calling a continuation more than once is "clone the stored closure, then `call_once` each clone." Rc/Arc are `Clone` so they re-enter; Box is not, so Box is one-shot by construction. That is the correct spine semantics (the Box spine has always been single-shot), and it falls out of the POC-8 `Clone` asymmetry rather than needing extra machinery. The by-value `call_once` is therefore not a regression for multi-shot: the multi-shot handler retains its own clone before consuming one.

3. `bind`/`map` compose stored closures, and the move-versus-clone asymmetry is exactly why construction stays per-`Store`. Composing onto a `FnOnce` (Box) moves the captured inner continuation, because the composed closure runs once; composing onto an `Fn` (Rc/Arc) must clone the captures on each call, because the composed closure can run repeatedly. The bodies therefore differ per `Store`, on top of the per-`Store` input bounds (Arc requires `Send + Sync`) and the non-injective `Store::Stored` projection already recorded in POC-8 finding 2. So `bind`/`map` join the per-`Store` construction surface (smart constructors, injection helpers) that item 7 keeps generated; they are not a new obstacle, and they do not move the substrate type/interpreter/`Clone` off their single definitions.

4. The substrate/construction split is now fully characterised. Unifies once: the substrate type, the interpreter, the `Clone` instance, and the `call_once` invocation bridge. Stays per-`Store` (generated): construction, namely the smart constructors and the `bind`/`map`/composition sites. This is the precise shape items 4 (step 3), 5, 6, and 7 describe, now demonstrated end-to-end rather than partly asserted.

## Limitations and scope

- The minimal effect is still `Ask`; `bind`/`map` are now exercised, but a multi-arm coproduct row, `Coyoneda`, and `WrapDrop` drop-safety are not part of this slice (they are item 4 step 3 integration, ordinary build risk, not a feasibility unknown). The reconciliation proved here is the closure-storage spine in isolation.
- A single generic-over-`Store` `bind`/`map` is still not expressible, for the same reasons as POC-8 finding 2 (non-injective projection; per-`Store` move-versus-clone bodies and input bounds). This POC confirms that boundary rather than removing it; the residual generation is item 7.
- The `Fn`-only POC-8 file remains the reference for the type/interpreter/`Clone` collapse; this file adds the callable-kind variation and `bind`/`map`.

## Bearing on items 5, 6, and 7

POC-8b removes the one "asserted, not built" caveat that stood over the substrate-unification decision. Item 6's primary outcome (one brand per effect, the three sibling brands and the prefix axis gone) is now demonstrated feasible: the `FnOnce`/`Fn` reconciliation that it depended on holds on one substrate, so the prefix-scheme fallback is not needed (it remains the documented fallback should the full multi-arm integration in item 4 step 3 surface a blocker, but no such blocker is known). Item 5's collapse (type, interpreter, `Clone`) was already proven by POC-8 and is unchanged. Item 7's residual-generation scope is confirmed exactly: per-`Store` construction including `bind`/`map`, with everything else unified. The G3 decision (substrate unification go) stands and is now backed end-to-end.
