//! Typed descriptors for `#[document_module]` item generators.
//!
//! These descriptors are the structured source of truth that the generator
//! builders will consume. The current registered specs cover the already-proven
//! Reader and State surfaces, while the descriptor model also records whether an
//! effect uses per-pointer-brand siblings or a single direct-payload cell.

use syn::Ident;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EffectName {
	Fresh,
	Input,
	KVStore,
	Output,
	Reader,
	State,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WrapperName {
	Run,
	RcRun,
	ArcRun,
	RunExplicit,
	RcRunExplicit,
	ArcRunExplicit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RunWrapperMethod {
	Ask,
	Asks,
	RunReader,
	Fresh,
	RunFreshWith,
	RunFresh,
	Input,
	RunInputSeq,
	Lookup,
	Update,
	RunKVStore,
	Get,
	Put,
	Modify,
	RunState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RunWrapperCoreMethod {
	Expand,
	Weaken,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EffectCellVariant {
	Plain,
	Send,
	Boxed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PointerMode {
	BoxFnOnce,
	RcFn,
	ArcSendFn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WrapperSubstrate {
	Free,
	RcFree,
	ArcFree,
	FreeExplicit,
	RcFreeExplicit,
	ArcFreeExplicit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExplicitLifetimeMode {
	Static,
	Explicit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Sendability {
	Local,
	SendSync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowBound {
	WrapDrop,
	Functor,
	SendFunctor,
	CloneProjection,
	SendSyncProjection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HandlerName {
	RunReader,
	RunState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CapabilityRule {
	SingleShot,
	MultiShot,
	ThreadSafe,
	ExplicitLifetime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RowEmbedEvidence {
	FirstOrderAndScoped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BrandSibling {
	pub(super) variant: EffectCellVariant,
	pub(super) cell_type: &'static str,
	pub(super) brand_type: &'static str,
	pub(super) pointer_mode: PointerMode,
	pub(super) sendability: Sendability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MethodSpec {
	pub(super) method: RunWrapperMethod,
	pub(super) handler_name: Option<HandlerName>,
	pub(super) row_bounds: &'static [RowBound],
	pub(super) capability_rules: &'static [CapabilityRule],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EffectSpec {
	pub(super) name: EffectName,
	pub(super) uses_pointer_brand_siblings: bool,
	pub(super) brand_siblings: &'static [BrandSibling],
	pub(super) methods: &'static [MethodSpec],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WrapperMethodSpec {
	pub(super) method: RunWrapperCoreMethod,
	pub(super) row_embed_evidence: RowEmbedEvidence,
	pub(super) row_bounds: &'static [RowBound],
	pub(super) capability_rules: &'static [CapabilityRule],
	pub(super) docs: WrapperMethodDocs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WrapperMethodDocs {
	pub(super) summary: &'static str,
	pub(super) example_subject: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WrapperMethodDescriptor {
	pub(super) wrapper: &'static WrapperSpec,
	pub(super) method: &'static WrapperMethodSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WrapperSpec {
	pub(super) name: WrapperName,
	pub(super) pointer_mode: PointerMode,
	pub(super) substrate: WrapperSubstrate,
	pub(super) lifetime_mode: ExplicitLifetimeMode,
	pub(super) sendability: Sendability,
	pub(super) required_row_bounds: &'static [RowBound],
	pub(super) capability_rules: &'static [CapabilityRule],
}

const LOCAL_WRAPPER_BOUNDS: &[RowBound] = &[RowBound::WrapDrop, RowBound::Functor];
const ARC_WRAPPER_BOUNDS: &[RowBound] = &[RowBound::WrapDrop, RowBound::SendFunctor];
const LOCAL_HELPER_BOUNDS: &[RowBound] = &[RowBound::Functor];
const ARC_HELPER_BOUNDS: &[RowBound] = &[RowBound::SendFunctor, RowBound::SendSyncProjection];
const RC_EXPLICIT_HELPER_BOUNDS: &[RowBound] = &[RowBound::Functor, RowBound::CloneProjection];
const ARC_EXPLICIT_HELPER_BOUNDS: &[RowBound] =
	&[RowBound::SendFunctor, RowBound::CloneProjection, RowBound::SendSyncProjection];
const ROW_EMBED_LOCAL_BOUNDS: &[RowBound] = &[RowBound::WrapDrop, RowBound::Functor];
const ROW_EMBED_ARC_BOUNDS: &[RowBound] =
	&[RowBound::WrapDrop, RowBound::SendFunctor, RowBound::SendSyncProjection];
const ROW_EMBED_RC_EXPLICIT_BOUNDS: &[RowBound] =
	&[RowBound::WrapDrop, RowBound::Functor, RowBound::CloneProjection];
const ROW_EMBED_ARC_EXPLICIT_BOUNDS: &[RowBound] = &[
	RowBound::WrapDrop,
	RowBound::SendFunctor,
	RowBound::CloneProjection,
	RowBound::SendSyncProjection,
];

const READER_BRAND_SIBLINGS: &[BrandSibling] = &[
	BrandSibling {
		variant: EffectCellVariant::Plain,
		cell_type: "Reader",
		brand_type: "ReaderBrand",
		pointer_mode: PointerMode::RcFn,
		sendability: Sendability::Local,
	},
	BrandSibling {
		variant: EffectCellVariant::Send,
		cell_type: "SendReader",
		brand_type: "SendReaderBrand",
		pointer_mode: PointerMode::ArcSendFn,
		sendability: Sendability::SendSync,
	},
	BrandSibling {
		variant: EffectCellVariant::Boxed,
		cell_type: "BoxReader",
		brand_type: "BoxReaderBrand",
		pointer_mode: PointerMode::BoxFnOnce,
		sendability: Sendability::Local,
	},
];

const STATE_BRAND_SIBLINGS: &[BrandSibling] = &[
	BrandSibling {
		variant: EffectCellVariant::Plain,
		cell_type: "State",
		brand_type: "StateBrand",
		pointer_mode: PointerMode::RcFn,
		sendability: Sendability::Local,
	},
	BrandSibling {
		variant: EffectCellVariant::Send,
		cell_type: "SendState",
		brand_type: "SendStateBrand",
		pointer_mode: PointerMode::ArcSendFn,
		sendability: Sendability::SendSync,
	},
	BrandSibling {
		variant: EffectCellVariant::Boxed,
		cell_type: "BoxState",
		brand_type: "BoxStateBrand",
		pointer_mode: PointerMode::BoxFnOnce,
		sendability: Sendability::Local,
	},
];

const FRESH_BRAND_SIBLINGS: &[BrandSibling] = &[
	BrandSibling {
		variant: EffectCellVariant::Plain,
		cell_type: "Fresh",
		brand_type: "FreshBrand",
		pointer_mode: PointerMode::RcFn,
		sendability: Sendability::Local,
	},
	BrandSibling {
		variant: EffectCellVariant::Send,
		cell_type: "SendFresh",
		brand_type: "SendFreshBrand",
		pointer_mode: PointerMode::ArcSendFn,
		sendability: Sendability::SendSync,
	},
	BrandSibling {
		variant: EffectCellVariant::Boxed,
		cell_type: "BoxFresh",
		brand_type: "BoxFreshBrand",
		pointer_mode: PointerMode::BoxFnOnce,
		sendability: Sendability::Local,
	},
];

const INPUT_BRAND_SIBLINGS: &[BrandSibling] = &[
	BrandSibling {
		variant: EffectCellVariant::Plain,
		cell_type: "Input",
		brand_type: "InputBrand",
		pointer_mode: PointerMode::RcFn,
		sendability: Sendability::Local,
	},
	BrandSibling {
		variant: EffectCellVariant::Send,
		cell_type: "SendInput",
		brand_type: "SendInputBrand",
		pointer_mode: PointerMode::ArcSendFn,
		sendability: Sendability::SendSync,
	},
	BrandSibling {
		variant: EffectCellVariant::Boxed,
		cell_type: "BoxInput",
		brand_type: "BoxInputBrand",
		pointer_mode: PointerMode::BoxFnOnce,
		sendability: Sendability::Local,
	},
];

const KV_STORE_BRAND_SIBLINGS: &[BrandSibling] = &[
	BrandSibling {
		variant: EffectCellVariant::Plain,
		cell_type: "KVStore",
		brand_type: "KVStoreBrand",
		pointer_mode: PointerMode::RcFn,
		sendability: Sendability::Local,
	},
	BrandSibling {
		variant: EffectCellVariant::Send,
		cell_type: "SendKVStore",
		brand_type: "SendKVStoreBrand",
		pointer_mode: PointerMode::ArcSendFn,
		sendability: Sendability::SendSync,
	},
	BrandSibling {
		variant: EffectCellVariant::Boxed,
		cell_type: "BoxKVStore",
		brand_type: "BoxKVStoreBrand",
		pointer_mode: PointerMode::BoxFnOnce,
		sendability: Sendability::Local,
	},
];

const READER_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Ask,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::Asks,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunReader,
		handler_name: Some(HandlerName::RunReader),
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const STATE_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Get,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::Put,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::Modify,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunState,
		handler_name: Some(HandlerName::RunState),
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const FRESH_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Fresh,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunFreshWith,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunFresh,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const INPUT_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Input,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunInputSeq,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const KV_STORE_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Lookup,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::Update,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunKVStore,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const EFFECT_SPECS: &[EffectSpec] = &[
	EffectSpec {
		name: EffectName::Fresh,
		uses_pointer_brand_siblings: true,
		brand_siblings: FRESH_BRAND_SIBLINGS,
		methods: FRESH_METHODS,
	},
	EffectSpec {
		name: EffectName::Input,
		uses_pointer_brand_siblings: true,
		brand_siblings: INPUT_BRAND_SIBLINGS,
		methods: INPUT_METHODS,
	},
	EffectSpec {
		name: EffectName::KVStore,
		uses_pointer_brand_siblings: true,
		brand_siblings: KV_STORE_BRAND_SIBLINGS,
		methods: KV_STORE_METHODS,
	},
	EffectSpec {
		name: EffectName::Output,
		uses_pointer_brand_siblings: false,
		brand_siblings: &[],
		methods: &[],
	},
	EffectSpec {
		name: EffectName::Reader,
		uses_pointer_brand_siblings: true,
		brand_siblings: READER_BRAND_SIBLINGS,
		methods: READER_METHODS,
	},
	EffectSpec {
		name: EffectName::State,
		uses_pointer_brand_siblings: true,
		brand_siblings: STATE_BRAND_SIBLINGS,
		methods: STATE_METHODS,
	},
];

const WRAPPER_SPECS: &[WrapperSpec] = &[
	WrapperSpec {
		name: WrapperName::Run,
		pointer_mode: PointerMode::BoxFnOnce,
		substrate: WrapperSubstrate::Free,
		lifetime_mode: ExplicitLifetimeMode::Static,
		sendability: Sendability::Local,
		required_row_bounds: LOCAL_WRAPPER_BOUNDS,
		capability_rules: &[CapabilityRule::SingleShot],
	},
	WrapperSpec {
		name: WrapperName::RcRun,
		pointer_mode: PointerMode::RcFn,
		substrate: WrapperSubstrate::RcFree,
		lifetime_mode: ExplicitLifetimeMode::Static,
		sendability: Sendability::Local,
		required_row_bounds: LOCAL_WRAPPER_BOUNDS,
		capability_rules: &[CapabilityRule::MultiShot],
	},
	WrapperSpec {
		name: WrapperName::ArcRun,
		pointer_mode: PointerMode::ArcSendFn,
		substrate: WrapperSubstrate::ArcFree,
		lifetime_mode: ExplicitLifetimeMode::Static,
		sendability: Sendability::SendSync,
		required_row_bounds: ARC_WRAPPER_BOUNDS,
		capability_rules: &[CapabilityRule::MultiShot, CapabilityRule::ThreadSafe],
	},
	WrapperSpec {
		name: WrapperName::RunExplicit,
		pointer_mode: PointerMode::BoxFnOnce,
		substrate: WrapperSubstrate::FreeExplicit,
		lifetime_mode: ExplicitLifetimeMode::Explicit,
		sendability: Sendability::Local,
		required_row_bounds: LOCAL_WRAPPER_BOUNDS,
		capability_rules: &[CapabilityRule::SingleShot, CapabilityRule::ExplicitLifetime],
	},
	WrapperSpec {
		name: WrapperName::RcRunExplicit,
		pointer_mode: PointerMode::RcFn,
		substrate: WrapperSubstrate::RcFreeExplicit,
		lifetime_mode: ExplicitLifetimeMode::Explicit,
		sendability: Sendability::Local,
		required_row_bounds: LOCAL_WRAPPER_BOUNDS,
		capability_rules: &[CapabilityRule::MultiShot, CapabilityRule::ExplicitLifetime],
	},
	WrapperSpec {
		name: WrapperName::ArcRunExplicit,
		pointer_mode: PointerMode::ArcSendFn,
		substrate: WrapperSubstrate::ArcFreeExplicit,
		lifetime_mode: ExplicitLifetimeMode::Explicit,
		sendability: Sendability::SendSync,
		required_row_bounds: ARC_WRAPPER_BOUNDS,
		capability_rules: &[
			CapabilityRule::MultiShot,
			CapabilityRule::ThreadSafe,
			CapabilityRule::ExplicitLifetime,
		],
	},
];

const WRAPPER_METHOD_SPECS: &[WrapperMethodSpec] = &[
	WrapperMethodSpec {
		method: RunWrapperCoreMethod::Expand,
		row_embed_evidence: RowEmbedEvidence::FirstOrderAndScoped,
		row_bounds: ROW_EMBED_LOCAL_BOUNDS,
		capability_rules: &[],
		docs: WrapperMethodDocs {
			summary: "Widen both effect rows of a program to compatible supersets.",
			example_subject: "Compose independently-rowed programs after widening them.",
		},
	},
	WrapperMethodSpec {
		method: RunWrapperCoreMethod::Weaken,
		row_embed_evidence: RowEmbedEvidence::FirstOrderAndScoped,
		row_bounds: ROW_EMBED_LOCAL_BOUNDS,
		capability_rules: &[],
		docs: WrapperMethodDocs {
			summary: "Convenience wrapper for widening a program by one effect.",
			example_subject: "Lift a single-effect program into a wider row.",
		},
	},
];

impl EffectName {
	pub(super) const fn as_str(self) -> &'static str {
		match self {
			Self::Fresh => "Fresh",
			Self::Input => "Input",
			Self::KVStore => "KVStore",
			Self::Output => "Output",
			Self::Reader => "Reader",
			Self::State => "State",
		}
	}

	pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
		if ident == "Fresh" {
			Some(Self::Fresh)
		} else if ident == "Input" {
			Some(Self::Input)
		} else if ident == "KVStore" {
			Some(Self::KVStore)
		} else if ident == "Output" {
			Some(Self::Output)
		} else if ident == "Reader" {
			Some(Self::Reader)
		} else if ident == "State" {
			Some(Self::State)
		} else {
			None
		}
	}
}

impl WrapperName {
	pub(super) const fn as_str(self) -> &'static str {
		match self {
			Self::Run => "Run",
			Self::RcRun => "RcRun",
			Self::ArcRun => "ArcRun",
			Self::RunExplicit => "RunExplicit",
			Self::RcRunExplicit => "RcRunExplicit",
			Self::ArcRunExplicit => "ArcRunExplicit",
		}
	}

	pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
		if ident == "Run" {
			Some(Self::Run)
		} else if ident == "RcRun" {
			Some(Self::RcRun)
		} else if ident == "ArcRun" {
			Some(Self::ArcRun)
		} else if ident == "RunExplicit" {
			Some(Self::RunExplicit)
		} else if ident == "RcRunExplicit" {
			Some(Self::RcRunExplicit)
		} else if ident == "ArcRunExplicit" {
			Some(Self::ArcRunExplicit)
		} else {
			None
		}
	}
}

impl RunWrapperMethod {
	pub(super) const fn as_str(self) -> &'static str {
		match self {
			Self::Ask => "ask",
			Self::Asks => "asks",
			Self::RunReader => "run_reader",
			Self::Fresh => "fresh",
			Self::RunFreshWith => "run_fresh_with",
			Self::RunFresh => "run_fresh",
			Self::Input => "input",
			Self::RunInputSeq => "run_input_seq",
			Self::Lookup => "lookup",
			Self::Update => "update",
			Self::RunKVStore => "run_kv_store",
			Self::Get => "get",
			Self::Put => "put",
			Self::Modify => "modify",
			Self::RunState => "run_state",
		}
	}

	pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
		if ident == "ask" {
			Some(Self::Ask)
		} else if ident == "asks" {
			Some(Self::Asks)
		} else if ident == "run_reader" {
			Some(Self::RunReader)
		} else if ident == "fresh" {
			Some(Self::Fresh)
		} else if ident == "run_fresh_with" {
			Some(Self::RunFreshWith)
		} else if ident == "run_fresh" {
			Some(Self::RunFresh)
		} else if ident == "input" {
			Some(Self::Input)
		} else if ident == "run_input_seq" {
			Some(Self::RunInputSeq)
		} else if ident == "lookup" {
			Some(Self::Lookup)
		} else if ident == "update" {
			Some(Self::Update)
		} else if ident == "run_kv_store" {
			Some(Self::RunKVStore)
		} else if ident == "get" {
			Some(Self::Get)
		} else if ident == "put" {
			Some(Self::Put)
		} else if ident == "modify" {
			Some(Self::Modify)
		} else if ident == "run_state" {
			Some(Self::RunState)
		} else {
			None
		}
	}
}

impl RunWrapperCoreMethod {
	pub(super) const fn as_str(self) -> &'static str {
		match self {
			Self::Expand => "expand",
			Self::Weaken => "weaken",
		}
	}

	pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
		if ident == "expand" {
			Some(Self::Expand)
		} else if ident == "weaken" {
			Some(Self::Weaken)
		} else {
			None
		}
	}
}

pub(super) fn effect_specs() -> &'static [EffectSpec] {
	EFFECT_SPECS
}

pub(super) fn wrapper_specs() -> &'static [WrapperSpec] {
	WRAPPER_SPECS
}

pub(super) fn wrapper_method_specs() -> &'static [WrapperMethodSpec] {
	WRAPPER_METHOD_SPECS
}

pub(super) fn effect_spec(effect: EffectName) -> Option<&'static EffectSpec> {
	effect_specs().iter().find(|spec| spec.name == effect)
}

pub(super) fn wrapper_spec(wrapper: WrapperName) -> Option<&'static WrapperSpec> {
	wrapper_specs().iter().find(|spec| spec.name == wrapper)
}

pub(super) fn wrapper_method_spec(
	method: RunWrapperCoreMethod
) -> Option<&'static WrapperMethodSpec> {
	wrapper_method_specs().iter().find(|spec| spec.method == method)
}

pub(super) fn wrapper_method_descriptor(
	wrapper: WrapperName,
	method: RunWrapperCoreMethod,
) -> Option<WrapperMethodDescriptor> {
	Some(WrapperMethodDescriptor {
		wrapper: wrapper_spec(wrapper)?,
		method: wrapper_method_spec(method)?,
	})
}

pub(super) fn method_spec(
	effect: EffectName,
	method: RunWrapperMethod,
) -> Option<&'static MethodSpec> {
	effect_spec(effect)?.methods.iter().find(|spec| spec.method == method)
}

pub(super) fn wrapper_method_row_bounds(
	wrapper: WrapperName,
	effect: EffectName,
	method: RunWrapperMethod,
) -> Option<&'static [RowBound]> {
	let method_spec = method_spec(effect, method)?;
	let wrapper_spec = wrapper_spec(wrapper)?;

	match (wrapper_spec.sendability, wrapper_spec.lifetime_mode) {
		(Sendability::SendSync, ExplicitLifetimeMode::Explicit) => Some(ARC_EXPLICIT_HELPER_BOUNDS),
		(Sendability::SendSync, ExplicitLifetimeMode::Static) => Some(ARC_HELPER_BOUNDS),
		(Sendability::Local, ExplicitLifetimeMode::Explicit)
			if wrapper == WrapperName::RcRunExplicit =>
			Some(RC_EXPLICIT_HELPER_BOUNDS),
		_ => Some(method_spec.row_bounds),
	}
}

pub(super) fn wrapper_core_method_row_bounds(
	wrapper: WrapperName,
	method: RunWrapperCoreMethod,
) -> Option<&'static [RowBound]> {
	let _method_spec = wrapper_method_spec(method)?;
	let wrapper_spec = wrapper_spec(wrapper)?;

	match (wrapper_spec.sendability, wrapper_spec.lifetime_mode) {
		(Sendability::SendSync, ExplicitLifetimeMode::Explicit) =>
			Some(ROW_EMBED_ARC_EXPLICIT_BOUNDS),
		(Sendability::SendSync, ExplicitLifetimeMode::Static) => Some(ROW_EMBED_ARC_BOUNDS),
		(Sendability::Local, ExplicitLifetimeMode::Explicit)
			if wrapper == WrapperName::RcRunExplicit =>
			Some(ROW_EMBED_RC_EXPLICIT_BOUNDS),
		_ => Some(ROW_EMBED_LOCAL_BOUNDS),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn descriptors_cover_reader_and_state() {
		let effects = effect_specs();
		assert_eq!(effects.len(), 6);
		assert!(effects.iter().any(|spec| spec.name == EffectName::Fresh));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Input));
		assert!(effects.iter().any(|spec| spec.name == EffectName::KVStore));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Output));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Reader));
		assert!(effects.iter().any(|spec| spec.name == EffectName::State));
		assert_eq!(effect_spec(EffectName::Fresh).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::Input).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::KVStore).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::Output).map(|spec| spec.brand_siblings.len()), Some(0),);
		assert_eq!(effect_spec(EffectName::Reader).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::State).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(
			effect_spec(EffectName::Fresh).map(|spec| spec.uses_pointer_brand_siblings),
			Some(true),
		);
		assert_eq!(
			effect_spec(EffectName::Input).map(|spec| spec.uses_pointer_brand_siblings),
			Some(true),
		);
		assert_eq!(
			effect_spec(EffectName::KVStore).map(|spec| spec.uses_pointer_brand_siblings),
			Some(true),
		);
		assert_eq!(
			effect_spec(EffectName::Output).map(|spec| spec.uses_pointer_brand_siblings),
			Some(false),
		);
		assert_eq!(
			effect_spec(EffectName::Reader).map(|spec| spec.uses_pointer_brand_siblings),
			Some(true),
		);
		assert_eq!(
			effect_spec(EffectName::State).map(|spec| spec.uses_pointer_brand_siblings),
			Some(true),
		);
	}

	#[test]
	fn descriptors_cover_six_wrappers() {
		let wrappers = wrapper_specs();
		assert_eq!(wrappers.len(), 6);
		assert!(wrappers.iter().any(|spec| spec.name == WrapperName::Run));
		assert!(wrappers.iter().any(|spec| spec.name == WrapperName::RcRun));
		assert!(wrappers.iter().any(|spec| spec.name == WrapperName::ArcRun));
		assert!(wrappers.iter().any(|spec| spec.name == WrapperName::RunExplicit));
		assert!(wrappers.iter().any(|spec| spec.name == WrapperName::RcRunExplicit));
		assert!(wrappers.iter().any(|spec| spec.name == WrapperName::ArcRunExplicit));
	}

	#[test]
	fn descriptors_cover_reader_and_state_methods() {
		assert!(method_spec(EffectName::Fresh, RunWrapperMethod::Fresh).is_some());
		assert!(method_spec(EffectName::Fresh, RunWrapperMethod::RunFreshWith).is_some());
		assert!(method_spec(EffectName::Fresh, RunWrapperMethod::RunFresh).is_some());
		assert!(method_spec(EffectName::Input, RunWrapperMethod::Input).is_some());
		assert!(method_spec(EffectName::Input, RunWrapperMethod::RunInputSeq).is_some());
		assert!(method_spec(EffectName::KVStore, RunWrapperMethod::Lookup).is_some());
		assert!(method_spec(EffectName::KVStore, RunWrapperMethod::Update).is_some());
		assert!(method_spec(EffectName::KVStore, RunWrapperMethod::RunKVStore).is_some());
		assert!(method_spec(EffectName::Reader, RunWrapperMethod::Ask).is_some());
		assert!(method_spec(EffectName::Reader, RunWrapperMethod::Asks).is_some());
		assert!(method_spec(EffectName::Reader, RunWrapperMethod::RunReader).is_some());
		assert!(method_spec(EffectName::State, RunWrapperMethod::Get).is_some());
		assert!(method_spec(EffectName::State, RunWrapperMethod::Put).is_some());
		assert!(method_spec(EffectName::State, RunWrapperMethod::Modify).is_some());
		assert!(method_spec(EffectName::State, RunWrapperMethod::RunState).is_some());
		assert!(method_spec(EffectName::Reader, RunWrapperMethod::Get).is_none());
		assert!(method_spec(EffectName::State, RunWrapperMethod::Ask).is_none());
	}

	#[test]
	fn descriptors_cover_wrapper_core_methods() {
		let methods = wrapper_method_specs();
		assert_eq!(methods.len(), 2);
		assert!(methods.iter().any(|spec| spec.method == RunWrapperCoreMethod::Expand));
		assert!(methods.iter().any(|spec| spec.method == RunWrapperCoreMethod::Weaken));
		assert_eq!(
			wrapper_method_spec(RunWrapperCoreMethod::Expand).map(|spec| spec.row_embed_evidence),
			Some(RowEmbedEvidence::FirstOrderAndScoped),
		);
		assert_eq!(
			wrapper_method_spec(RunWrapperCoreMethod::Weaken).map(|spec| spec.row_embed_evidence),
			Some(RowEmbedEvidence::FirstOrderAndScoped),
		);
		assert_eq!(
			wrapper_method_spec(RunWrapperCoreMethod::Expand).map(|spec| spec.docs.summary),
			Some("Widen both effect rows of a program to compatible supersets."),
		);
	}

	#[test]
	fn wrapper_core_method_descriptor_combines_wrapper_and_method_specs() -> syn::Result<()> {
		let descriptor =
			wrapper_method_descriptor(WrapperName::ArcRunExplicit, RunWrapperCoreMethod::Expand)
				.ok_or_else(|| {
					syn::Error::new(
						proc_macro2::Span::call_site(),
						"ArcRunExplicit expand descriptor should be registered",
					)
				})?;

		assert_eq!(descriptor.wrapper.name, WrapperName::ArcRunExplicit);
		assert_eq!(descriptor.wrapper.pointer_mode, PointerMode::ArcSendFn);
		assert_eq!(descriptor.wrapper.substrate, WrapperSubstrate::ArcFreeExplicit);
		assert_eq!(descriptor.wrapper.lifetime_mode, ExplicitLifetimeMode::Explicit);
		assert_eq!(descriptor.wrapper.sendability, Sendability::SendSync);
		assert_eq!(descriptor.method.method, RunWrapperCoreMethod::Expand);
		Ok(())
	}

	#[test]
	fn wrapper_specific_bounds_reflect_arc_and_explicit_modes() {
		assert_eq!(
			wrapper_method_row_bounds(WrapperName::Run, EffectName::State, RunWrapperMethod::Get),
			Some(LOCAL_HELPER_BOUNDS),
		);
		assert_eq!(
			wrapper_method_row_bounds(
				WrapperName::ArcRun,
				EffectName::State,
				RunWrapperMethod::Get
			),
			Some(ARC_HELPER_BOUNDS),
		);
		assert_eq!(
			wrapper_method_row_bounds(
				WrapperName::RcRunExplicit,
				EffectName::State,
				RunWrapperMethod::Get
			),
			Some(RC_EXPLICIT_HELPER_BOUNDS),
		);
		assert_eq!(
			wrapper_method_row_bounds(
				WrapperName::ArcRunExplicit,
				EffectName::State,
				RunWrapperMethod::Get
			),
			Some(ARC_EXPLICIT_HELPER_BOUNDS),
		);
	}

	#[test]
	fn wrapper_core_method_bounds_reflect_arc_and_explicit_modes() {
		assert_eq!(
			wrapper_core_method_row_bounds(WrapperName::Run, RunWrapperCoreMethod::Expand),
			Some(ROW_EMBED_LOCAL_BOUNDS),
		);
		assert_eq!(
			wrapper_core_method_row_bounds(WrapperName::ArcRun, RunWrapperCoreMethod::Expand),
			Some(ROW_EMBED_ARC_BOUNDS),
		);
		assert_eq!(
			wrapper_core_method_row_bounds(
				WrapperName::RcRunExplicit,
				RunWrapperCoreMethod::Expand
			),
			Some(ROW_EMBED_RC_EXPLICIT_BOUNDS),
		);
		assert_eq!(
			wrapper_core_method_row_bounds(
				WrapperName::ArcRunExplicit,
				RunWrapperCoreMethod::Expand
			),
			Some(ROW_EMBED_ARC_EXPLICIT_BOUNDS),
		);
	}
}
