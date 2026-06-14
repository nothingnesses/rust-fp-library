# POC-1 Findings: type-level partition of a unified row

Tier A, foundation sweep. Charter question: can a type-level `Partition` split a unified row into first-order and higher-order sub-rows, and can a value round-trip through a split and reinjection while preserving membership? This is the FS-3 (facade) crux: one user-facing row that splits internally into today's dual `Node<FO, HO>`.

Result: PASS.

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_partition.rs`. Built on stable Rust with `--features effects`, over the real `CoproductBrand`/`CNilBrand` brand row and the real `frunk_core` coproduct value type (re-exported through `fp_library::types::effects::coproduct`), per setup step S2. Five tests pass.

## What was built

Type-level partition. A `Partition` trait over the brand row with `type First` and `type Higher`, defined recursively:

- `CNilBrand`: both sides empty.
- `CoproductBrand<H, T>` where `H: OrderedEffect, T: Partition`: the head's placement is dispatched through a helper `PartitionPlace<H, RestFirst, RestHigher>` keyed on the head's order marker (`H::Order`). The `FirstOrderMark` impl prepends `H` to the first-order side; the `HigherOrderMark` impl prepends it to the higher-order side.

Keying the conditional on the order marker (rather than trying to write two overlapping impls of `Partition` for `CoproductBrand`) is what makes this work on stable Rust without specialization: there is exactly one `Partition` impl for `CoproductBrand`, and the branch is chosen by the two non-overlapping `PartitionPlace` impls. The partition is total over any row of `OrderedEffect` brands and preserves the relative order of effects within each sub-row (verified: a `Get, Catch, Ask, Local` row partitions to `Get, Ask` and `Catch, Local`).

Value-level round-trip. The facade must, at runtime, route a unified-row value to the correct sub-row and reinject it. This is exactly frunk's `subset` (split into a subset, returning the remainder otherwise) and `embed` (reinject into the superset), the same membership machinery the library already uses for `expand`. The tests confirm an active variant in the first-order subset is extracted and reinjected unchanged, and a variant outside it is reported in the remainder with its value intact.

## Findings

1. The type-level partition is total and stable, with no overlapping-impl or specialization need. The marker-keyed helper trait is the load-bearing technique and is the same shape POC-0 used for classification, so the two compose.
2. `subset` and `embed` resolved as methods on `Coproduct` without importing `CoproductSubsetter`/`CoproductEmbedder` explicitly (the compiler flagged those imports as unused). So the value-level split and reinjection ride frunk's existing, already-depended-on machinery with minimal ceremony, and "membership preserved across split and reinjection" is frunk's `subset`/`embed` contract rather than anything this design must re-establish.
3. FS-3 (facade) is feasible. Both halves it needs, the type-level split into `Node<First, Higher>` and the value-level routing/reinjection, are demonstrated independently and on the real encoding.

## Limitations and scope

- The type-level partition operates on the brand row; the value-level round-trip is shown on a frunk `Coproduct` of stand-in payload types. The two are not yet wired through the `Kind` projection (`CoproductBrand<H, T>::Of<X> = Coproduct<H::Of<X>, T::Of<X>>`), because projecting the partitioned sub-rows to value coproducts needs `Functor`-bearing effect rows. That full wiring (unified value -> `Node::First`/`Node::Scoped` via the projected partition) belongs to POC-3 and the integration tier; POC-1's purpose was to prove each half is expressible against the real primitives, which it does.
- This settles feasibility, not desirability. Whether FS-3 is worth shipping as an interim (it deletes no internals; see the open question and gate G1) is unchanged by this result; POC-1 only removes the "is the split even expressible" risk.

## Bearing on the gates

For gate G1: the facade's enabling mechanism is feasible, so FS-3 remains a viable cheap increment if the team wants the single-row surface before FS-1/FS-2 land. The partition and classification techniques (POC-0, POC-1) share the marker-keyed-helper shape, which is reusable for the order-directed dispatch in POC-3.
