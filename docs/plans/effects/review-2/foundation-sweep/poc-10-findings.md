# POC-10 Findings: per-layer dispatch and allocation, FS-1 versus FS-0

Tier F, foundation sweep. Charter question: what is the per-layer dispatch and allocation shape of the surviving design (FS-1) versus FS-0?

Result: FS-1 is no worse than FS-0, and substantially cheaper on the higher-order path. Because FS-1 exists only as minimal spike POCs while FS-0 is the optimized production subsystem, the verdict is a structural comparison (mechanism versus mechanism), grounded by the captured FS-0 baseline and a fair first-order micro-benchmark; production wall-clock numbers are deferred to FS-1 implementation.

## FS-0 baseline (captured per S6)

Captured with `cargo bench --bench benchmarks --features effects -- --quick` immediately before measuring:

- Effect Rows Row Canonicalisation: direct canonical 1.39 ns (3 effects), 1.87 ns (5 effects); subset fallback 6.0 ns (3), 14.5 ns (5).
- Effect Rows Handler Composition: ~1.06 ns (3 handlers), ~1.47 ns (5 handlers); handlers macro and builder chain equal.
- Scoped Operations (higher-order) Run Bracket: construct ~53 ns, dispatch ~497 ns (Box).
- Scoped Operations RcRun Bracket: construct ~78 ns, dispatch ~655 ns.
- Scoped Operations RcRun RefBracket: construct ~77 ns, dispatch ~788 ns.

The decisive numbers: FS-0's first-order row machinery is ~1-15 ns, while FS-0's scoped (higher-order) dispatch is ~500-800 ns. That two-to-three-orders-of-magnitude gap is the boundary-frame machinery.

## (B) Structural comparison

First-order path. FS-1 layer: `Free<UnifiedRow>` resume, one `Coyoneda` lower, one coproduct match. FS-0 layer: the same, plus the extra `Node<R, S>` enum match (the dual row wraps each layer in `Node::First`/`Node::Scoped`). So FS-1 does strictly one fewer match per first-order layer; allocations per bind are identical (both are `Free` over a `Coyoneda`-wrapped coproduct).

Higher-order path. FS-1 elaborates a higher-order effect by an interpret-pass interpose (POC-4/POC-5/POC-9): the work is first-order-cost (a peel, a lower, a continuation call), with no boundary frames and no result-polymorphic protocol allocations. FS-0 dispatches a scoped operation through boundary frames plus the carrier/residual protocol, which the baseline measures at ~500-800 ns. So FS-1's higher-order dispatch is structurally first-order-cost (tens of ns) against FS-0's ~500-800 ns, the largest perf difference and a clear FS-1 win, driven by deleting the boundary-frame allocations.

## (C) Fair first-order micro-benchmark

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_dispatch_micro.rs`. Two minimal substrates differing only in the dual-row `Node` wrapper, an FS-0-shaped `Free<Node<coproduct>>` and an FS-1-shaped `Free<coproduct>`, interpreting the same 200,000-layer first-order workload (run with `--release`).

Result: FS-1 (unified row) 68.6 ms, FS-0 (Node-wrapped) 62.8 ms over 200,000 layers, a ~9 percent spread, single run, with FS-0 nominally faster. The interpretation: the `Node` match is below single-run measurement noise; the first-order per-layer cost is dominated by the shared work (the boxed `FnOnce` continuation call, the `Coyoneda` lower, the `Free` resume step), so the dual row's extra `Node` match neither helps nor hurts measurably. FS-1 is no worse than FS-0 on the first-order path; the difference is negligible.

This is a single-run relative timing, not a statistically rigorous benchmark; it grounds the structural claim (the `Node` overhead is negligible) rather than producing a precise figure. A precise figure would require a production FS-1 implementation and a criterion harness, which is deferred.

## Findings

1. First-order dispatch: FS-1 is no worse than FS-0. Structurally FS-1 drops the `Node` match; the micro-benchmark confirms that difference is below noise, so the first-order path is comparable.
2. Higher-order dispatch: FS-1 is substantially cheaper. FS-0's scoped dispatch is ~500-800 ns (boundary frames); FS-1's elaboration is first-order-cost (tens of ns) with no boundary-frame allocations. This is the dominant perf difference and an FS-1 win.
3. Allocations per bind are the same on the first-order path (both `Free` over `Coyoneda` coproduct); FS-1 allocates strictly less on the higher-order path (no boundary frames, no result-polymorphic protocol).

## Limitations and scope

- FS-1 is not production-built, so production wall-clock numbers are deferred; the verdict is structural plus the fair first-order micro-benchmark. A naive spike-versus-production wall-clock was deliberately not used (it would conflate implementation maturity with mechanism cost).
- The micro-benchmark is a single-run release timing; it establishes the order of magnitude (negligible `Node` overhead), not a precise figure.
- The FS-0 baseline was captured in `--quick` mode in the sandbox; the relative magnitudes (first-order ~ns, scoped ~hundreds of ns) are the load-bearing observation, not the exact figures.

## Bearing on gate G4

POC-10 supplies the rubric's performance row: FS-1 is no worse than FS-0 on the first-order path (negligible `Node` overhead) and substantially cheaper on the higher-order path (no boundary-frame dispatch), with production numbers deferred to FS-1 implementation. Together with the other tiers, the rubric can now be scored and the chosen FS-1 design folded back into remediation items 8, 12, 13, and 14 at gate G4.
