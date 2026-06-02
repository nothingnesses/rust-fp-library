//! Typed descriptors for `#[document_module]` item generators.
//!
//! These descriptors are the structured source of truth that the generator
//! builders consume. The registered specs cover generated effect cells, wrapper
//! helpers, and whether each effect uses per-pointer-brand siblings or a single
//! direct-payload cell.

use syn::Ident;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EffectName {
	Coroutine,
	Empty,
	Except,
	Fail,
	Fresh,
	Input,
	KVStore,
	Log,
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
	Throw,
	ThrowUnit,
	Rethrow,
	Note,
	FromOption,
	RunExcept,
	YieldValue,
	RunCoroutine,
	Fail,
	RunFail,
	Fresh,
	RunFreshWith,
	RunFresh,
	Input,
	RunInputSeq,
	Lookup,
	Update,
	RunKVStore,
	Log,
	RunLogVec,
	RunLogMonoid,
	Output,
	RunOutputVec,
	RunOutputMonoid,
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
pub(super) enum EffectOperationShape {
	ReaderEnvironment,
	StateCell,
	RequestValueContinuation,
	DirectPayload,
	PhantomAbort,
	FixedMessageAbort,
	TypedAbort,
	CoroutineYieldStatus,
	KeyValueStore,
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
pub(super) enum OwnedBrandCapability {
	Functor,
	Pointed,
	Semimonad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RefBrandCapability {
	Functor,
	Pointed,
	Semimonad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SendBrandCapability {
	SendPointed,
	SendRefPointed,
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
pub(super) struct WrapperBrandCapabilities {
	pub(super) owned: &'static [OwnedBrandCapability],
	pub(super) reference: &'static [RefBrandCapability],
	pub(super) send: &'static [SendBrandCapability],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WrapperCapabilityRequirements {
	pub(super) owned: &'static [OwnedBrandCapability],
	pub(super) reference: &'static [RefBrandCapability],
	pub(super) send: &'static [SendBrandCapability],
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
	pub(super) operation_shape: EffectOperationShape,
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
	pub(super) brand_capabilities: WrapperBrandCapabilities,
	pub(super) capability_rules: &'static [CapabilityRule],
}

const NO_WRAPPER_BRAND_CAPABILITIES: WrapperBrandCapabilities = WrapperBrandCapabilities {
	owned: &[],
	reference: &[],
	send: &[],
};
const RUN_EXPLICIT_BRAND_CAPABILITIES: WrapperBrandCapabilities = WrapperBrandCapabilities {
	owned: &[
		OwnedBrandCapability::Functor,
		OwnedBrandCapability::Pointed,
		OwnedBrandCapability::Semimonad,
	],
	reference: &[
		RefBrandCapability::Functor,
		RefBrandCapability::Pointed,
		RefBrandCapability::Semimonad,
	],
	send: &[],
};
const RC_RUN_EXPLICIT_BRAND_CAPABILITIES: WrapperBrandCapabilities = WrapperBrandCapabilities {
	owned: &[OwnedBrandCapability::Pointed],
	reference: &[
		RefBrandCapability::Functor,
		RefBrandCapability::Pointed,
		RefBrandCapability::Semimonad,
	],
	send: &[],
};
const ARC_RUN_EXPLICIT_BRAND_CAPABILITIES: WrapperBrandCapabilities = WrapperBrandCapabilities {
	owned: &[],
	reference: &[],
	send: &[SendBrandCapability::SendPointed, SendBrandCapability::SendRefPointed],
};
const NO_WRAPPER_CAPABILITY_REQUIREMENTS: WrapperCapabilityRequirements =
	WrapperCapabilityRequirements {
		owned: &[],
		reference: &[],
		send: &[],
	};

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

const COROUTINE_BRAND_SIBLINGS: &[BrandSibling] = &[
	BrandSibling {
		variant: EffectCellVariant::Plain,
		cell_type: "Coroutine",
		brand_type: "CoroutineBrand",
		pointer_mode: PointerMode::RcFn,
		sendability: Sendability::Local,
	},
	BrandSibling {
		variant: EffectCellVariant::Send,
		cell_type: "SendCoroutine",
		brand_type: "SendCoroutineBrand",
		pointer_mode: PointerMode::ArcSendFn,
		sendability: Sendability::SendSync,
	},
	BrandSibling {
		variant: EffectCellVariant::Boxed,
		cell_type: "BoxCoroutine",
		brand_type: "BoxCoroutineBrand",
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

const OUTPUT_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Output,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunOutputVec,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunOutputMonoid,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const COROUTINE_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::YieldValue,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunCoroutine,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const FAIL_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Fail,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunFail,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const EXCEPT_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Throw,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::ThrowUnit,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::Rethrow,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::Note,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::FromOption,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunExcept,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const LOG_METHODS: &[MethodSpec] = &[
	MethodSpec {
		method: RunWrapperMethod::Log,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunLogVec,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
	MethodSpec {
		method: RunWrapperMethod::RunLogMonoid,
		handler_name: None,
		row_bounds: LOCAL_HELPER_BOUNDS,
		capability_rules: &[],
	},
];

const KNOWN_OPERATION_SHAPES: &[EffectOperationShape] = &[
	EffectOperationShape::ReaderEnvironment,
	EffectOperationShape::StateCell,
	EffectOperationShape::RequestValueContinuation,
	EffectOperationShape::DirectPayload,
	EffectOperationShape::PhantomAbort,
	EffectOperationShape::FixedMessageAbort,
	EffectOperationShape::TypedAbort,
	EffectOperationShape::CoroutineYieldStatus,
	EffectOperationShape::KeyValueStore,
];

const EFFECT_SPECS: &[EffectSpec] = &[
	EffectSpec {
		name: EffectName::Coroutine,
		operation_shape: EffectOperationShape::CoroutineYieldStatus,
		uses_pointer_brand_siblings: true,
		brand_siblings: COROUTINE_BRAND_SIBLINGS,
		methods: COROUTINE_METHODS,
	},
	EffectSpec {
		name: EffectName::Empty,
		operation_shape: EffectOperationShape::PhantomAbort,
		uses_pointer_brand_siblings: false,
		brand_siblings: &[],
		methods: &[],
	},
	EffectSpec {
		name: EffectName::Except,
		operation_shape: EffectOperationShape::TypedAbort,
		uses_pointer_brand_siblings: false,
		brand_siblings: &[],
		methods: EXCEPT_METHODS,
	},
	EffectSpec {
		name: EffectName::Fail,
		operation_shape: EffectOperationShape::FixedMessageAbort,
		uses_pointer_brand_siblings: false,
		brand_siblings: &[],
		methods: FAIL_METHODS,
	},
	EffectSpec {
		name: EffectName::Fresh,
		operation_shape: EffectOperationShape::RequestValueContinuation,
		uses_pointer_brand_siblings: true,
		brand_siblings: FRESH_BRAND_SIBLINGS,
		methods: FRESH_METHODS,
	},
	EffectSpec {
		name: EffectName::Input,
		operation_shape: EffectOperationShape::RequestValueContinuation,
		uses_pointer_brand_siblings: true,
		brand_siblings: INPUT_BRAND_SIBLINGS,
		methods: INPUT_METHODS,
	},
	EffectSpec {
		name: EffectName::KVStore,
		operation_shape: EffectOperationShape::KeyValueStore,
		uses_pointer_brand_siblings: true,
		brand_siblings: KV_STORE_BRAND_SIBLINGS,
		methods: KV_STORE_METHODS,
	},
	EffectSpec {
		name: EffectName::Log,
		operation_shape: EffectOperationShape::DirectPayload,
		uses_pointer_brand_siblings: false,
		brand_siblings: &[],
		methods: LOG_METHODS,
	},
	EffectSpec {
		name: EffectName::Output,
		operation_shape: EffectOperationShape::DirectPayload,
		uses_pointer_brand_siblings: false,
		brand_siblings: &[],
		methods: OUTPUT_METHODS,
	},
	EffectSpec {
		name: EffectName::Reader,
		operation_shape: EffectOperationShape::ReaderEnvironment,
		uses_pointer_brand_siblings: true,
		brand_siblings: READER_BRAND_SIBLINGS,
		methods: READER_METHODS,
	},
	EffectSpec {
		name: EffectName::State,
		operation_shape: EffectOperationShape::StateCell,
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
		brand_capabilities: NO_WRAPPER_BRAND_CAPABILITIES,
		capability_rules: &[CapabilityRule::SingleShot],
	},
	WrapperSpec {
		name: WrapperName::RcRun,
		pointer_mode: PointerMode::RcFn,
		substrate: WrapperSubstrate::RcFree,
		lifetime_mode: ExplicitLifetimeMode::Static,
		sendability: Sendability::Local,
		required_row_bounds: LOCAL_WRAPPER_BOUNDS,
		brand_capabilities: NO_WRAPPER_BRAND_CAPABILITIES,
		capability_rules: &[CapabilityRule::MultiShot],
	},
	WrapperSpec {
		name: WrapperName::ArcRun,
		pointer_mode: PointerMode::ArcSendFn,
		substrate: WrapperSubstrate::ArcFree,
		lifetime_mode: ExplicitLifetimeMode::Static,
		sendability: Sendability::SendSync,
		required_row_bounds: ARC_WRAPPER_BOUNDS,
		brand_capabilities: NO_WRAPPER_BRAND_CAPABILITIES,
		capability_rules: &[CapabilityRule::MultiShot, CapabilityRule::ThreadSafe],
	},
	WrapperSpec {
		name: WrapperName::RunExplicit,
		pointer_mode: PointerMode::BoxFnOnce,
		substrate: WrapperSubstrate::FreeExplicit,
		lifetime_mode: ExplicitLifetimeMode::Explicit,
		sendability: Sendability::Local,
		required_row_bounds: LOCAL_WRAPPER_BOUNDS,
		brand_capabilities: RUN_EXPLICIT_BRAND_CAPABILITIES,
		capability_rules: &[CapabilityRule::SingleShot, CapabilityRule::ExplicitLifetime],
	},
	WrapperSpec {
		name: WrapperName::RcRunExplicit,
		pointer_mode: PointerMode::RcFn,
		substrate: WrapperSubstrate::RcFreeExplicit,
		lifetime_mode: ExplicitLifetimeMode::Explicit,
		sendability: Sendability::Local,
		required_row_bounds: LOCAL_WRAPPER_BOUNDS,
		brand_capabilities: RC_RUN_EXPLICIT_BRAND_CAPABILITIES,
		capability_rules: &[CapabilityRule::MultiShot, CapabilityRule::ExplicitLifetime],
	},
	WrapperSpec {
		name: WrapperName::ArcRunExplicit,
		pointer_mode: PointerMode::ArcSendFn,
		substrate: WrapperSubstrate::ArcFreeExplicit,
		lifetime_mode: ExplicitLifetimeMode::Explicit,
		sendability: Sendability::SendSync,
		required_row_bounds: ARC_WRAPPER_BOUNDS,
		brand_capabilities: ARC_RUN_EXPLICIT_BRAND_CAPABILITIES,
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
			Self::Coroutine => "Coroutine",
			Self::Empty => "Empty",
			Self::Except => "Except",
			Self::Fail => "Fail",
			Self::Fresh => "Fresh",
			Self::Input => "Input",
			Self::KVStore => "KVStore",
			Self::Log => "Log",
			Self::Output => "Output",
			Self::Reader => "Reader",
			Self::State => "State",
		}
	}

	pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
		if ident == "Coroutine" {
			Some(Self::Coroutine)
		} else if ident == "Empty" {
			Some(Self::Empty)
		} else if ident == "Except" {
			Some(Self::Except)
		} else if ident == "Fail" {
			Some(Self::Fail)
		} else if ident == "Fresh" {
			Some(Self::Fresh)
		} else if ident == "Input" {
			Some(Self::Input)
		} else if ident == "KVStore" {
			Some(Self::KVStore)
		} else if ident == "Log" {
			Some(Self::Log)
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
			Self::Throw => "throw",
			Self::ThrowUnit => "throw_unit",
			Self::Rethrow => "rethrow",
			Self::Note => "note",
			Self::FromOption => "from_option",
			Self::RunExcept => "run_except",
			Self::YieldValue => "yield_value",
			Self::RunCoroutine => "run_coroutine",
			Self::Fail => "fail",
			Self::RunFail => "run_fail",
			Self::Fresh => "fresh",
			Self::RunFreshWith => "run_fresh_with",
			Self::RunFresh => "run_fresh",
			Self::Input => "input",
			Self::RunInputSeq => "run_input_seq",
			Self::Lookup => "lookup",
			Self::Update => "update",
			Self::RunKVStore => "run_kv_store",
			Self::Log => "log",
			Self::RunLogVec => "run_log_vec",
			Self::RunLogMonoid => "run_log_monoid",
			Self::Output => "output",
			Self::RunOutputVec => "run_output_vec",
			Self::RunOutputMonoid => "run_output_monoid",
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
		} else if ident == "throw" {
			Some(Self::Throw)
		} else if ident == "throw_unit" {
			Some(Self::ThrowUnit)
		} else if ident == "rethrow" {
			Some(Self::Rethrow)
		} else if ident == "note" {
			Some(Self::Note)
		} else if ident == "from_option" {
			Some(Self::FromOption)
		} else if ident == "run_except" {
			Some(Self::RunExcept)
		} else if ident == "yield_value" {
			Some(Self::YieldValue)
		} else if ident == "run_coroutine" {
			Some(Self::RunCoroutine)
		} else if ident == "fail" {
			Some(Self::Fail)
		} else if ident == "run_fail" {
			Some(Self::RunFail)
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
		} else if ident == "log" {
			Some(Self::Log)
		} else if ident == "run_log_vec" {
			Some(Self::RunLogVec)
		} else if ident == "run_log_monoid" {
			Some(Self::RunLogMonoid)
		} else if ident == "output" {
			Some(Self::Output)
		} else if ident == "run_output_vec" {
			Some(Self::RunOutputVec)
		} else if ident == "run_output_monoid" {
			Some(Self::RunOutputMonoid)
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

impl EffectOperationShape {
	pub(super) const fn uses_pointer_brand_siblings(self) -> bool {
		match self {
			Self::ReaderEnvironment
			| Self::StateCell
			| Self::RequestValueContinuation
			| Self::CoroutineYieldStatus
			| Self::KeyValueStore => true,
			Self::DirectPayload
			| Self::PhantomAbort
			| Self::FixedMessageAbort
			| Self::TypedAbort => false,
		}
	}
}

impl OwnedBrandCapability {
	const fn as_str(self) -> &'static str {
		match self {
			Self::Functor => "Functor",
			Self::Pointed => "Pointed",
			Self::Semimonad => "Semimonad",
		}
	}
}

impl RefBrandCapability {
	const fn as_str(self) -> &'static str {
		match self {
			Self::Functor => "RefFunctor",
			Self::Pointed => "RefPointed",
			Self::Semimonad => "RefSemimonad",
		}
	}
}

impl SendBrandCapability {
	const fn as_str(self) -> &'static str {
		match self {
			Self::SendPointed => "SendPointed",
			Self::SendRefPointed => "SendRefPointed",
		}
	}
}

impl RowBound {
	const fn as_str(self) -> &'static str {
		match self {
			Self::WrapDrop => "WrapDrop",
			Self::Functor => "Functor",
			Self::SendFunctor => "SendFunctor",
			Self::CloneProjection => "CloneProjection",
			Self::SendSyncProjection => "SendSyncProjection",
		}
	}
}

impl CapabilityRule {
	const fn as_str(self) -> &'static str {
		match self {
			Self::SingleShot => "SingleShot",
			Self::MultiShot => "MultiShot",
			Self::ThreadSafe => "ThreadSafe",
			Self::ExplicitLifetime => "ExplicitLifetime",
		}
	}
}

impl WrapperSpec {
	pub(super) fn supports_owned_capability(
		self,
		capability: OwnedBrandCapability,
	) -> bool {
		self.brand_capabilities.owned.contains(&capability)
	}

	pub(super) fn supports_ref_capability(
		self,
		capability: RefBrandCapability,
	) -> bool {
		self.brand_capabilities.reference.contains(&capability)
	}

	pub(super) fn supports_send_capability(
		self,
		capability: SendBrandCapability,
	) -> bool {
		self.brand_capabilities.send.contains(&capability)
	}

	fn supports_capability_rule(
		self,
		rule: CapabilityRule,
	) -> bool {
		self.capability_rules.contains(&rule)
	}

	fn supports_row_bound(
		self,
		bound: RowBound,
	) -> bool {
		match bound {
			RowBound::WrapDrop => self.required_row_bounds.contains(&RowBound::WrapDrop),
			RowBound::Functor => self.sendability == Sendability::Local,
			RowBound::SendFunctor | RowBound::SendSyncProjection =>
				self.sendability == Sendability::SendSync,
			RowBound::CloneProjection =>
				self.lifetime_mode == ExplicitLifetimeMode::Explicit
					&& self.supports_capability_rule(CapabilityRule::MultiShot),
		}
	}
}

fn validate_wrapper_support(
	wrapper_spec: &WrapperSpec,
	context: &str,
	row_bounds: &[RowBound],
	capability_rules: &[CapabilityRule],
	capability_requirements: WrapperCapabilityRequirements,
) -> Result<(), String> {
	for bound in row_bounds {
		if !wrapper_spec.supports_row_bound(*bound) {
			return Err(format!(
				"`{}` does not support row bound `{}` required by {context}",
				wrapper_spec.name.as_str(),
				bound.as_str(),
			));
		}
	}

	for rule in capability_rules {
		if !wrapper_spec.supports_capability_rule(*rule) {
			return Err(format!(
				"`{}` does not support wrapper capability `{}` required by {context}",
				wrapper_spec.name.as_str(),
				rule.as_str(),
			));
		}
	}

	for capability in capability_requirements.owned {
		if !wrapper_spec.supports_owned_capability(*capability) {
			return Err(format!(
				"`{}` does not provide owned brand capability `{}` required by {context}",
				wrapper_spec.name.as_str(),
				capability.as_str(),
			));
		}
	}

	for capability in capability_requirements.reference {
		if !wrapper_spec.supports_ref_capability(*capability) {
			return Err(format!(
				"`{}` does not provide ref brand capability `{}` required by {context}",
				wrapper_spec.name.as_str(),
				capability.as_str(),
			));
		}
	}

	for capability in capability_requirements.send {
		if !wrapper_spec.supports_send_capability(*capability) {
			return Err(format!(
				"`{}` does not provide send brand capability `{}` required by {context}",
				wrapper_spec.name.as_str(),
				capability.as_str(),
			));
		}
	}

	Ok(())
}

pub(super) fn known_operation_shapes() -> &'static [EffectOperationShape] {
	KNOWN_OPERATION_SHAPES
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

pub(super) fn validate_wrapper_effect_method_support(
	wrapper: WrapperName,
	effect: EffectName,
	method: RunWrapperMethod,
) -> Result<(), String> {
	let method_spec = method_spec(effect, method).ok_or_else(|| {
		format!("`{}` is not a registered {} helper method", method.as_str(), effect.as_str(),)
	})?;
	let wrapper_spec = wrapper_spec(wrapper)
		.ok_or_else(|| format!("`{}` is not a registered wrapper", wrapper.as_str()))?;
	let row_bounds = wrapper_method_row_bounds(wrapper, effect, method).ok_or_else(|| {
		format!(
			"`{}` / `{}` / `{}` has no resolved row-bound descriptor",
			wrapper.as_str(),
			effect.as_str(),
			method.as_str(),
		)
	})?;
	let context =
		format!("`{}` `{}` helper `{}`", wrapper.as_str(), effect.as_str(), method.as_str(),);

	validate_wrapper_support(
		wrapper_spec,
		&context,
		row_bounds,
		method_spec.capability_rules,
		NO_WRAPPER_CAPABILITY_REQUIREMENTS,
	)
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

pub(super) fn validate_wrapper_core_method_support(
	wrapper: WrapperName,
	method: RunWrapperCoreMethod,
) -> Result<(), String> {
	let method_spec = wrapper_method_spec(method)
		.ok_or_else(|| format!("`{}` is not a registered wrapper-wide method", method.as_str()))?;
	let wrapper_spec = wrapper_spec(wrapper)
		.ok_or_else(|| format!("`{}` is not a registered wrapper", wrapper.as_str()))?;
	let row_bounds = wrapper_core_method_row_bounds(wrapper, method).ok_or_else(|| {
		format!(
			"`{}` / wrapper-wide method `{}` has no resolved row-bound descriptor",
			wrapper.as_str(),
			method.as_str(),
		)
	})?;
	let context = format!("`{}` wrapper-wide method `{}`", wrapper.as_str(), method.as_str());

	validate_wrapper_support(
		wrapper_spec,
		&context,
		row_bounds,
		method_spec.capability_rules,
		NO_WRAPPER_CAPABILITY_REQUIREMENTS,
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn descriptors_cover_registered_effects() {
		let effects = effect_specs();
		assert_eq!(effects.len(), 11);
		assert!(effects.iter().any(|spec| spec.name == EffectName::Coroutine));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Empty));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Except));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Fail));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Fresh));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Input));
		assert!(effects.iter().any(|spec| spec.name == EffectName::KVStore));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Log));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Output));
		assert!(effects.iter().any(|spec| spec.name == EffectName::Reader));
		assert!(effects.iter().any(|spec| spec.name == EffectName::State));
		assert_eq!(
			effect_spec(EffectName::Coroutine).map(|spec| spec.brand_siblings.len()),
			Some(3),
		);
		assert_eq!(effect_spec(EffectName::Empty).map(|spec| spec.brand_siblings.len()), Some(0),);
		assert_eq!(effect_spec(EffectName::Except).map(|spec| spec.brand_siblings.len()), Some(0),);
		assert_eq!(effect_spec(EffectName::Fail).map(|spec| spec.brand_siblings.len()), Some(0),);
		assert_eq!(effect_spec(EffectName::Fresh).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::Input).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::KVStore).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::Log).map(|spec| spec.brand_siblings.len()), Some(0),);
		assert_eq!(effect_spec(EffectName::Output).map(|spec| spec.brand_siblings.len()), Some(0),);
		assert_eq!(effect_spec(EffectName::Reader).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::State).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(
			effect_spec(EffectName::Coroutine).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::CoroutineYieldStatus),
		);
		assert_eq!(
			effect_spec(EffectName::Empty).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::PhantomAbort),
		);
		assert_eq!(
			effect_spec(EffectName::Except).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::TypedAbort),
		);
		assert_eq!(
			effect_spec(EffectName::Fail).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::FixedMessageAbort),
		);
		assert_eq!(
			effect_spec(EffectName::Fresh).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::RequestValueContinuation),
		);
		assert_eq!(
			effect_spec(EffectName::Input).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::RequestValueContinuation),
		);
		assert_eq!(
			effect_spec(EffectName::KVStore).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::KeyValueStore),
		);
		assert_eq!(
			effect_spec(EffectName::Log).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::DirectPayload),
		);
		assert_eq!(
			effect_spec(EffectName::Output).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::DirectPayload),
		);
		assert_eq!(
			effect_spec(EffectName::Reader).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::ReaderEnvironment),
		);
		assert_eq!(
			effect_spec(EffectName::State).map(|spec| spec.operation_shape),
			Some(EffectOperationShape::StateCell),
		);
		assert_eq!(
			effect_spec(EffectName::Coroutine).map(|spec| spec.uses_pointer_brand_siblings),
			Some(true),
		);
		assert_eq!(
			effect_spec(EffectName::Empty).map(|spec| spec.uses_pointer_brand_siblings),
			Some(false),
		);
		assert_eq!(
			effect_spec(EffectName::Except).map(|spec| spec.uses_pointer_brand_siblings),
			Some(false),
		);
		assert_eq!(
			effect_spec(EffectName::Fail).map(|spec| spec.uses_pointer_brand_siblings),
			Some(false),
		);
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
			effect_spec(EffectName::Log).map(|spec| spec.uses_pointer_brand_siblings),
			Some(false),
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
	fn descriptors_cover_current_and_reserved_operation_shapes() {
		let shapes = known_operation_shapes();
		assert_eq!(shapes.len(), 9);
		assert!(shapes.contains(&EffectOperationShape::ReaderEnvironment));
		assert!(shapes.contains(&EffectOperationShape::StateCell));
		assert!(shapes.contains(&EffectOperationShape::RequestValueContinuation));
		assert!(shapes.contains(&EffectOperationShape::DirectPayload));
		assert!(shapes.contains(&EffectOperationShape::PhantomAbort));
		assert!(shapes.contains(&EffectOperationShape::FixedMessageAbort));
		assert!(shapes.contains(&EffectOperationShape::TypedAbort));
		assert!(shapes.contains(&EffectOperationShape::CoroutineYieldStatus));
		assert!(shapes.contains(&EffectOperationShape::KeyValueStore));
		assert!(EffectOperationShape::RequestValueContinuation.uses_pointer_brand_siblings());
		assert!(EffectOperationShape::CoroutineYieldStatus.uses_pointer_brand_siblings());
		assert!(!EffectOperationShape::DirectPayload.uses_pointer_brand_siblings());
		assert!(!EffectOperationShape::PhantomAbort.uses_pointer_brand_siblings());
		assert!(!EffectOperationShape::FixedMessageAbort.uses_pointer_brand_siblings());
		assert!(!EffectOperationShape::TypedAbort.uses_pointer_brand_siblings());
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
	fn descriptors_encode_verified_wrapper_brand_capability_matrix() {
		let run = wrapper_spec(WrapperName::Run).expect("Run descriptor should exist");
		let run_explicit =
			wrapper_spec(WrapperName::RunExplicit).expect("RunExplicit descriptor should exist");
		let rc_run_explicit = wrapper_spec(WrapperName::RcRunExplicit)
			.expect("RcRunExplicit descriptor should exist");
		let arc_run_explicit = wrapper_spec(WrapperName::ArcRunExplicit)
			.expect("ArcRunExplicit descriptor should exist");

		assert!(run.brand_capabilities.owned.is_empty());
		assert!(run.brand_capabilities.reference.is_empty());
		assert!(run.brand_capabilities.send.is_empty());
		assert!(run_explicit.supports_owned_capability(OwnedBrandCapability::Functor));
		assert!(run_explicit.supports_owned_capability(OwnedBrandCapability::Pointed));
		assert!(run_explicit.supports_owned_capability(OwnedBrandCapability::Semimonad));
		assert!(run_explicit.supports_ref_capability(RefBrandCapability::Functor));
		assert!(run_explicit.supports_ref_capability(RefBrandCapability::Pointed));
		assert!(run_explicit.supports_ref_capability(RefBrandCapability::Semimonad));
		assert!(rc_run_explicit.supports_owned_capability(OwnedBrandCapability::Pointed));
		assert!(!rc_run_explicit.supports_owned_capability(OwnedBrandCapability::Functor));
		assert!(rc_run_explicit.supports_ref_capability(RefBrandCapability::Functor));
		assert!(rc_run_explicit.supports_ref_capability(RefBrandCapability::Pointed));
		assert!(rc_run_explicit.supports_ref_capability(RefBrandCapability::Semimonad));
		assert!(arc_run_explicit.supports_send_capability(SendBrandCapability::SendPointed));
		assert!(arc_run_explicit.supports_send_capability(SendBrandCapability::SendRefPointed));
		assert!(!arc_run_explicit.supports_ref_capability(RefBrandCapability::Functor));
	}

	#[test]
	fn descriptors_cover_registered_effect_methods() {
		assert!(method_spec(EffectName::Coroutine, RunWrapperMethod::YieldValue).is_some());
		assert!(method_spec(EffectName::Coroutine, RunWrapperMethod::RunCoroutine).is_some());
		assert!(method_spec(EffectName::Except, RunWrapperMethod::Throw).is_some());
		assert!(method_spec(EffectName::Except, RunWrapperMethod::ThrowUnit).is_some());
		assert!(method_spec(EffectName::Except, RunWrapperMethod::Rethrow).is_some());
		assert!(method_spec(EffectName::Except, RunWrapperMethod::Note).is_some());
		assert!(method_spec(EffectName::Except, RunWrapperMethod::FromOption).is_some());
		assert!(method_spec(EffectName::Except, RunWrapperMethod::RunExcept).is_some());
		assert!(method_spec(EffectName::Fail, RunWrapperMethod::Fail).is_some());
		assert!(method_spec(EffectName::Fail, RunWrapperMethod::RunFail).is_some());
		assert!(method_spec(EffectName::Fresh, RunWrapperMethod::Fresh).is_some());
		assert!(method_spec(EffectName::Fresh, RunWrapperMethod::RunFreshWith).is_some());
		assert!(method_spec(EffectName::Fresh, RunWrapperMethod::RunFresh).is_some());
		assert!(method_spec(EffectName::Input, RunWrapperMethod::Input).is_some());
		assert!(method_spec(EffectName::Input, RunWrapperMethod::RunInputSeq).is_some());
		assert!(method_spec(EffectName::KVStore, RunWrapperMethod::Lookup).is_some());
		assert!(method_spec(EffectName::KVStore, RunWrapperMethod::Update).is_some());
		assert!(method_spec(EffectName::KVStore, RunWrapperMethod::RunKVStore).is_some());
		assert!(method_spec(EffectName::Log, RunWrapperMethod::Log).is_some());
		assert!(method_spec(EffectName::Log, RunWrapperMethod::RunLogVec).is_some());
		assert!(method_spec(EffectName::Log, RunWrapperMethod::RunLogMonoid).is_some());
		assert!(method_spec(EffectName::Output, RunWrapperMethod::Output).is_some());
		assert!(method_spec(EffectName::Output, RunWrapperMethod::RunOutputVec).is_some());
		assert!(method_spec(EffectName::Output, RunWrapperMethod::RunOutputMonoid).is_some());
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
	fn wrapper_effect_method_validation_accepts_supported_descriptors() {
		assert_eq!(
			validate_wrapper_effect_method_support(
				WrapperName::Run,
				EffectName::Reader,
				RunWrapperMethod::Ask,
			),
			Ok(()),
		);
		assert_eq!(
			validate_wrapper_effect_method_support(
				WrapperName::ArcRunExplicit,
				EffectName::State,
				RunWrapperMethod::RunState,
			),
			Ok(()),
		);
	}

	#[test]
	fn wrapper_validation_rejects_unsupported_row_bounds() {
		let run = wrapper_spec(WrapperName::Run).expect("Run descriptor should exist");
		let error = validate_wrapper_support(
			run,
			"`Run` synthetic send helper",
			&[RowBound::SendFunctor],
			&[],
			NO_WRAPPER_CAPABILITY_REQUIREMENTS,
		)
		.expect_err("Run should not satisfy SendFunctor row bounds");
		assert!(
			error.contains("row bound `SendFunctor`"),
			"row-bound validation should name the unsupported bound; got: {error}",
		);

		let arc_run = wrapper_spec(WrapperName::ArcRun).expect("ArcRun descriptor should exist");
		assert_eq!(
			validate_wrapper_support(
				arc_run,
				"`ArcRun` synthetic send helper",
				&[RowBound::SendFunctor, RowBound::SendSyncProjection],
				&[],
				NO_WRAPPER_CAPABILITY_REQUIREMENTS,
			),
			Ok(()),
		);
	}

	#[test]
	fn wrapper_validation_rejects_unsupported_wrapper_rules() {
		let run = wrapper_spec(WrapperName::Run).expect("Run descriptor should exist");
		let error = validate_wrapper_support(
			run,
			"`Run` synthetic multi-shot helper",
			&[],
			&[CapabilityRule::MultiShot],
			NO_WRAPPER_CAPABILITY_REQUIREMENTS,
		)
		.expect_err("Run should not satisfy MultiShot helper requirements");
		assert!(
			error.contains("wrapper capability `MultiShot`"),
			"capability-rule validation should name the unsupported rule; got: {error}",
		);

		let rc_run = wrapper_spec(WrapperName::RcRun).expect("RcRun descriptor should exist");
		assert_eq!(
			validate_wrapper_support(
				rc_run,
				"`RcRun` synthetic multi-shot helper",
				&[],
				&[CapabilityRule::MultiShot],
				NO_WRAPPER_CAPABILITY_REQUIREMENTS,
			),
			Ok(()),
		);
	}

	#[test]
	fn wrapper_validation_rejects_unsupported_brand_capabilities() {
		let run_explicit =
			wrapper_spec(WrapperName::RunExplicit).expect("RunExplicit descriptor should exist");
		assert_eq!(
			validate_wrapper_support(
				run_explicit,
				"`RunExplicit` synthetic owned helper",
				&[],
				&[],
				WrapperCapabilityRequirements {
					owned: &[OwnedBrandCapability::Functor],
					reference: &[],
					send: &[],
				},
			),
			Ok(()),
		);

		let rc_run_explicit = wrapper_spec(WrapperName::RcRunExplicit)
			.expect("RcRunExplicit descriptor should exist");
		let owned_error = validate_wrapper_support(
			rc_run_explicit,
			"`RcRunExplicit` synthetic owned helper",
			&[],
			&[],
			WrapperCapabilityRequirements {
				owned: &[OwnedBrandCapability::Functor],
				reference: &[],
				send: &[],
			},
		)
		.expect_err("RcRunExplicit should not satisfy owned Functor requirements");
		assert!(
			owned_error.contains("owned brand capability `Functor`"),
			"owned capability validation should name the missing capability; got: {owned_error}",
		);
		assert_eq!(
			validate_wrapper_support(
				rc_run_explicit,
				"`RcRunExplicit` synthetic ref helper",
				&[],
				&[],
				WrapperCapabilityRequirements {
					owned: &[],
					reference: &[RefBrandCapability::Functor],
					send: &[],
				},
			),
			Ok(()),
		);

		let arc_run_explicit = wrapper_spec(WrapperName::ArcRunExplicit)
			.expect("ArcRunExplicit descriptor should exist");
		let ref_error = validate_wrapper_support(
			arc_run_explicit,
			"`ArcRunExplicit` synthetic ref helper",
			&[],
			&[],
			WrapperCapabilityRequirements {
				owned: &[],
				reference: &[RefBrandCapability::Functor],
				send: &[],
			},
		)
		.expect_err("ArcRunExplicit should not satisfy RefFunctor requirements");
		assert!(
			ref_error.contains("ref brand capability `RefFunctor`"),
			"ref capability validation should name the missing capability; got: {ref_error}",
		);
		assert_eq!(
			validate_wrapper_support(
				arc_run_explicit,
				"`ArcRunExplicit` synthetic send helper",
				&[],
				&[],
				WrapperCapabilityRequirements {
					owned: &[],
					reference: &[],
					send: &[SendBrandCapability::SendPointed],
				},
			),
			Ok(()),
		);
		let send_error = validate_wrapper_support(
			run_explicit,
			"`RunExplicit` synthetic send helper",
			&[],
			&[],
			WrapperCapabilityRequirements {
				owned: &[],
				reference: &[],
				send: &[SendBrandCapability::SendPointed],
			},
		)
		.expect_err("RunExplicit should not satisfy SendPointed requirements");
		assert!(
			send_error.contains("send brand capability `SendPointed`"),
			"send capability validation should name the missing capability; got: {send_error}",
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
