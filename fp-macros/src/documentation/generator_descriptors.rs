//! Typed descriptors for `#[document_module]` item generators.
//!
//! These descriptors are the structured source of truth that the generator
//! builders will consume. The first version intentionally covers only the
//! already-proven Reader and State surfaces so the descriptor refactor can be
//! checked against existing macro-expansion baselines.

use syn::Ident;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EffectName {
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
	Get,
	Put,
	Modify,
	RunState,
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
	pub(super) brand_siblings: &'static [BrandSibling],
	pub(super) methods: &'static [MethodSpec],
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

const EFFECT_SPECS: &[EffectSpec] = &[
	EffectSpec {
		name: EffectName::Reader,
		brand_siblings: READER_BRAND_SIBLINGS,
		methods: READER_METHODS,
	},
	EffectSpec {
		name: EffectName::State,
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

impl EffectName {
	pub(super) const fn as_str(self) -> &'static str {
		match self {
			Self::Reader => "Reader",
			Self::State => "State",
		}
	}

	pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
		if ident == "Reader" {
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

pub(super) fn effect_specs() -> &'static [EffectSpec] {
	EFFECT_SPECS
}

pub(super) fn wrapper_specs() -> &'static [WrapperSpec] {
	WRAPPER_SPECS
}

pub(super) fn effect_spec(effect: EffectName) -> Option<&'static EffectSpec> {
	effect_specs().iter().find(|spec| spec.name == effect)
}

pub(super) fn wrapper_spec(wrapper: WrapperName) -> Option<&'static WrapperSpec> {
	wrapper_specs().iter().find(|spec| spec.name == wrapper)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn descriptors_cover_reader_and_state() {
		let effects = effect_specs();
		assert_eq!(effects.len(), 2);
		assert!(effects.iter().any(|spec| spec.name == EffectName::Reader));
		assert!(effects.iter().any(|spec| spec.name == EffectName::State));
		assert_eq!(effect_spec(EffectName::Reader).map(|spec| spec.brand_siblings.len()), Some(3),);
		assert_eq!(effect_spec(EffectName::State).map(|spec| spec.brand_siblings.len()), Some(3),);
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
}
