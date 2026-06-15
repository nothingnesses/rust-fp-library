//! POC-1 (foundation sweep, Tier A): type-level partition of a unified row.
//!
//! Charter question: can a type-level `Partition` split a unified row into
//! first-order and higher-order sub-rows, and can a value round-trip through a
//! split/reinject preserving membership? This is the FS-3 (facade) crux: a
//! single user-facing row that splits internally into today's dual `Node<FO,
//! HO>`.
//!
//! Harness (charter setup step S2): standalone from the effects subsystem,
//! built on the library's real `CoproductBrand`/`CNilBrand` row primitives and
//! the real `frunk_core` coproduct value type (re-exported, behind the
//! `effects` feature) with its `CoproductSubsetter`/`CoproductEmbedder`
//! membership machinery, so the technique is proven against the actual
//! encoding. Throwaway spike code; it need only compile and pass its
//! assertions.

#![cfg(feature = "effects")]

use fp_library::{
	brands::{
		CNilBrand,
		CoproductBrand,
	},
	types::effects::coproduct::{
		CNil,
		Coproduct,
	},
};

// Order classification (as in POC-0; redefined here because POC files are
// standalone).
pub struct FirstOrderMark;
pub struct HigherOrderMark;
pub trait OrderedEffect {
	type Order;
}

// Type-level partition of a brand row into a first-order sub-row and a
// higher-order sub-row, preserving the relative order within each. The
// conditional on the head's order is dispatched through `PartitionPlace`, which
// is keyed on the order marker, so no overlapping impls are needed.
pub trait Partition {
	type First;
	type Higher;
}

impl Partition for CNilBrand {
	type First = CNilBrand;
	type Higher = CNilBrand;
}

impl<H, T> Partition for CoproductBrand<H, T>
where
	H: OrderedEffect,
	T: Partition,
	H::Order: PartitionPlace<H, <T as Partition>::First, <T as Partition>::Higher>,
{
	type First =
		<H::Order as PartitionPlace<H, <T as Partition>::First, <T as Partition>::Higher>>::First;
	type Higher =
		<H::Order as PartitionPlace<H, <T as Partition>::First, <T as Partition>::Higher>>::Higher;
}

// Places the head `H` onto the first-order or higher-order sub-row according to
// the order marker `Self`, given the already-partitioned tail (`RF`, `RH`).
pub trait PartitionPlace<H, RF, RH> {
	type First;
	type Higher;
}

impl<H, RF, RH> PartitionPlace<H, RF, RH> for FirstOrderMark {
	type First = CoproductBrand<H, RF>;
	type Higher = RH;
}

impl<H, RF, RH> PartitionPlace<H, RF, RH> for HigherOrderMark {
	type First = RF;
	type Higher = CoproductBrand<H, RH>;
}

// Reflexive type-equality witness, for asserting the partition's outputs.
trait SameType<T> {}
impl<T> SameType<T> for T {}
fn assert_same_type<A: SameType<B>, B>() {}

// Sample effects: two first-order, two higher-order, interleaved.
struct GetBrand;
impl OrderedEffect for GetBrand {
	type Order = FirstOrderMark;
}
struct CatchBrand;
impl OrderedEffect for CatchBrand {
	type Order = HigherOrderMark;
}
struct AskBrand;
impl OrderedEffect for AskBrand {
	type Order = FirstOrderMark;
}
struct LocalBrand;
impl OrderedEffect for LocalBrand {
	type Order = HigherOrderMark;
}

type MixedRow = CoproductBrand<
	GetBrand,
	CoproductBrand<CatchBrand, CoproductBrand<AskBrand, CoproductBrand<LocalBrand, CNilBrand>>>,
>;
type ExpectedFirst = CoproductBrand<GetBrand, CoproductBrand<AskBrand, CNilBrand>>;
type ExpectedHigher = CoproductBrand<CatchBrand, CoproductBrand<LocalBrand, CNilBrand>>;

#[test]
fn partition_splits_mixed_row_preserving_relative_order() {
	assert_same_type::<<MixedRow as Partition>::First, ExpectedFirst>();
	assert_same_type::<<MixedRow as Partition>::Higher, ExpectedHigher>();
}

#[test]
fn partition_of_empty_row_is_empty_on_both_sides() {
	assert_same_type::<<CNilBrand as Partition>::First, CNilBrand>();
	assert_same_type::<<CNilBrand as Partition>::Higher, CNilBrand>();
}

#[test]
fn partition_of_all_first_order_leaves_higher_empty() {
	type AllFo = CoproductBrand<GetBrand, CoproductBrand<AskBrand, CNilBrand>>;
	assert_same_type::<<AllFo as Partition>::First, AllFo>();
	assert_same_type::<<AllFo as Partition>::Higher, CNilBrand>();
}

// Value-level round-trip: the facade must, at runtime, route a unified-row
// value to the correct sub-row and reinject it. This is exactly frunk's
// `subset` (split into a subset, returning the remainder otherwise) plus
// `embed` (reinject into the superset), the same membership machinery the
// library already uses for `expand`. Element types stand in for projected
// effect operations; what is proven is that the split and reinjection preserve
// the value and its position.
type FullValueRow = Coproduct<i32, Coproduct<&'static str, Coproduct<bool, CNil>>>;
type FirstSubsetValues = Coproduct<i32, Coproduct<bool, CNil>>;

#[test]
fn value_routes_to_first_subset_and_reinjects() {
	let original: FullValueRow = Coproduct::inject(7_i32);
	// Split: the active variant is in the first-order subset.
	let routed: Result<FirstSubsetValues, _> = original.subset();
	// The active variant is in the first-order subset, so `subset` returns `Ok`.
	assert!(
		matches!(routed, Ok(Coproduct::Inl(7))),
		"expected the i32 variant to land in the first-order subset"
	);
	if let Ok(first) = routed {
		// Reinject the subset value back into the full row.
		let reinjected: FullValueRow = first.embed();
		assert!(matches!(reinjected, Coproduct::Inl(7)));
	}
}

#[test]
fn value_in_higher_remainder_is_reported_as_remainder() {
	let original: FullValueRow = Coproduct::inject("scoped");
	// The `&str` variant is not in the first-order subset, so `subset` reports
	// it in the remainder (the higher-order side), preserving the value.
	let routed: Result<FirstSubsetValues, _> = original.subset();
	// The `&str` variant is not in the first-order subset, so `subset` returns
	// `Err` with the value preserved in the remainder.
	assert!(
		matches!(routed, Err(Coproduct::Inl("scoped"))),
		"the &str variant should not be in the first-order subset"
	);
}
