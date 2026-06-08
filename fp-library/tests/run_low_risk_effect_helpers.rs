#![cfg(feature = "effects")]

//! Integration tests for the generated Fresh, Input, and KVStore helpers.
//!
//! These tests exercise the representative non-explicit wrappers first:
//! single-shot `Run`, cloneable `RcRun`, and thread-safe `ArcRun`.
//! Each program goes through the named runner rather than a hand-written
//! handler so regressions in generated constructor or runner wiring are
//! caught at the public API boundary.

use {
	fp_library::{
		brands::*,
		types::effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			run::Run,
			run_explicit::RunExplicit,
		},
	},
	std::collections::BTreeMap,
};

type RunFreshRow = CoproductBrand<CoyonedaBrand<BoxFreshBrand<BoxBrand, usize>>, CNilBrand>;
type RcRunFreshRow = CoproductBrand<RcCoyonedaBrand<FreshBrand<RcBrand, usize>>, CNilBrand>;
type ArcRunFreshRow = CoproductBrand<ArcCoyonedaBrand<SendFreshBrand<ArcBrand, usize>>, CNilBrand>;

type RunInputRow =
	CoproductBrand<CoyonedaBrand<BoxInputBrand<BoxBrand, Option<&'static str>>>, CNilBrand>;
type RcRunInputRow =
	CoproductBrand<RcCoyonedaBrand<InputBrand<RcBrand, Option<&'static str>>>, CNilBrand>;
type ArcRunInputRow =
	CoproductBrand<ArcCoyonedaBrand<SendInputBrand<ArcBrand, Option<&'static str>>>, CNilBrand>;

type RunKVStoreRow =
	CoproductBrand<CoyonedaBrand<BoxKVStoreBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
type RcRunKVStoreRow =
	CoproductBrand<RcCoyonedaBrand<KVStoreBrand<RcBrand, &'static str, i32>>, CNilBrand>;
type ArcRunKVStoreRow =
	CoproductBrand<ArcCoyonedaBrand<SendKVStoreBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
type KVObserved = (Option<i32>, Option<i32>);
type KVMap = BTreeMap<&'static str, i32>;
type KVHandled = (KVObserved, KVMap);
type RunKVHandled = Run<CNilBrand, CNilBrand, KVHandled>;
type RcRunKVHandled = RcRun<CNilBrand, CNilBrand, KVHandled>;
type ArcRunKVHandled = ArcRun<CNilBrand, CNilBrand, KVHandled>;
type RunExplicitKVHandled = RunExplicit<'static, CNilBrand, CNilBrand, KVHandled>;
type RcRunExplicitKVHandled = RcRunExplicit<'static, CNilBrand, CNilBrand, KVHandled>;
type ArcRunExplicitKVHandled = ArcRunExplicit<'static, CNilBrand, CNilBrand, KVHandled>;

fn run_fresh_pair() -> Run<RunFreshRow, CNilBrand, (usize, usize)> {
	Run::<RunFreshRow, CNilBrand, usize>::fresh::<_>().bind(|first| {
		Run::<RunFreshRow, CNilBrand, usize>::fresh::<_>().map(move |second| (first, second))
	})
}

fn rc_run_fresh_pair() -> RcRun<RcRunFreshRow, CNilBrand, (usize, usize)> {
	RcRun::<RcRunFreshRow, CNilBrand, usize>::fresh::<_>().bind(|first| {
		RcRun::<RcRunFreshRow, CNilBrand, usize>::fresh::<_>().map(move |second| (first, second))
	})
}

fn arc_run_fresh_pair() -> ArcRun<ArcRunFreshRow, CNilBrand, (usize, usize)> {
	ArcRun::<ArcRunFreshRow, CNilBrand, usize>::fresh::<_>().bind(|first| {
		ArcRun::<ArcRunFreshRow, CNilBrand, usize>::fresh::<_>().map(move |second| (first, second))
	})
}

fn run_explicit_fresh_pair() -> RunExplicit<'static, RunFreshRow, CNilBrand, (usize, usize)> {
	RunExplicit::<'static, RunFreshRow, CNilBrand, usize>::fresh::<_>().bind(|first| {
		RunExplicit::<'static, RunFreshRow, CNilBrand, usize>::fresh::<_>()
			.map(move |second| (first, second))
	})
}

fn rc_run_explicit_fresh_pair() -> RcRunExplicit<'static, RcRunFreshRow, CNilBrand, (usize, usize)>
{
	RcRunExplicit::<'static, RcRunFreshRow, CNilBrand, usize>::fresh::<_>().bind(|first| {
		RcRunExplicit::<'static, RcRunFreshRow, CNilBrand, usize>::fresh::<_>()
			.map(move |second| (first, second))
	})
}

fn arc_run_explicit_fresh_pair()
-> ArcRunExplicit<'static, ArcRunFreshRow, CNilBrand, (usize, usize)> {
	ArcRunExplicit::<'static, ArcRunFreshRow, CNilBrand, usize>::fresh::<_>().bind(|first| {
		ArcRunExplicit::<'static, ArcRunFreshRow, CNilBrand, usize>::fresh::<_>()
			.map(move |second| (first, second))
	})
}

fn run_input_triple() -> Run<RunInputRow, CNilBrand, InputTriple> {
	Run::<RunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(|first| {
		Run::<RunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(move |second| {
			Run::<RunInputRow, CNilBrand, Option<&'static str>>::input::<_>()
				.map(move |third| (first, second, third))
		})
	})
}

fn rc_run_input_triple() -> RcRun<RcRunInputRow, CNilBrand, InputTriple> {
	RcRun::<RcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(|first| {
		RcRun::<RcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(move |second| {
			RcRun::<RcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>()
				.map(move |third| (first, second, third))
		})
	})
}

fn arc_run_input_triple() -> ArcRun<ArcRunInputRow, CNilBrand, InputTriple> {
	ArcRun::<ArcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(|first| {
		ArcRun::<ArcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(
			move |second| {
				ArcRun::<ArcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>()
					.map(move |third| (first, second, third))
			},
		)
	})
}

fn run_explicit_input_triple() -> RunExplicit<'static, RunInputRow, CNilBrand, InputTriple> {
	RunExplicit::<'static, RunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(
		|first| {
			RunExplicit::<'static, RunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(
				move |second| {
					RunExplicit::<'static, RunInputRow, CNilBrand, Option<&'static str>>::input::<_>(
					)
					.map(move |third| (first, second, third))
				},
			)
		},
	)
}

fn rc_run_explicit_input_triple() -> RcRunExplicit<'static, RcRunInputRow, CNilBrand, InputTriple> {
	RcRunExplicit::<'static, RcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(
		|first| {
			RcRunExplicit::<'static, RcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>()
				.bind(move |second| {
					RcRunExplicit::<
						'static,
						RcRunInputRow,
						CNilBrand,
						Option<&'static str>,
					>::input::<_>()
					.map(move |third| (first, second, third))
				})
		},
	)
}

fn arc_run_explicit_input_triple() -> ArcRunExplicit<'static, ArcRunInputRow, CNilBrand, InputTriple>
{
	ArcRunExplicit::<'static, ArcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>().bind(
		|first| {
			ArcRunExplicit::<'static, ArcRunInputRow, CNilBrand, Option<&'static str>>::input::<_>()
				.bind(move |second| {
					ArcRunExplicit::<
						'static,
						ArcRunInputRow,
						CNilBrand,
						Option<&'static str>,
					>::input::<_>()
					.map(move |third| (first, second, third))
				})
		},
	)
}

type InputTriple = (Option<&'static str>, Option<&'static str>, Option<&'static str>);

fn initial_store() -> KVMap {
	BTreeMap::from([("a", 7), ("b", 3)])
}

fn final_store() -> KVMap {
	BTreeMap::from([("b", 3)])
}

fn run_kv_store_program() -> Run<RunKVStoreRow, CNilBrand, (Option<i32>, Option<i32>)> {
	Run::<RunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(|before| {
		Run::<RunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>("a", Some(9)).bind(move |()| {
			Run::<RunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(
				move |after_insert| {
					Run::<RunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>("a", None)
						.map(move |()| (before, after_insert))
				},
			)
		})
	})
}

fn rc_run_kv_store_program() -> RcRun<RcRunKVStoreRow, CNilBrand, (Option<i32>, Option<i32>)> {
	RcRun::<RcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(|before| {
		RcRun::<RcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>("a", Some(9)).bind(move |()| {
			RcRun::<RcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(
				move |after_insert| {
					RcRun::<RcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>("a", None)
						.map(move |()| (before, after_insert))
				},
			)
		})
	})
}

fn arc_run_kv_store_program() -> ArcRun<ArcRunKVStoreRow, CNilBrand, (Option<i32>, Option<i32>)> {
	ArcRun::<ArcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(|before| {
		ArcRun::<ArcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>("a", Some(9)).bind(
			move |()| {
				ArcRun::<ArcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(
					move |after_insert| {
						ArcRun::<ArcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>("a", None)
							.map(move |()| (before, after_insert))
					},
				)
			},
		)
	})
}

fn run_explicit_kv_store_program()
-> RunExplicit<'static, RunKVStoreRow, CNilBrand, (Option<i32>, Option<i32>)> {
	RunExplicit::<'static, RunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(
		|before| {
			RunExplicit::<'static, RunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>("a", Some(9))
				.bind(move |()| {
					RunExplicit::<'static, RunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>(
						"a",
					)
					.bind(move |after_insert| {
						RunExplicit::<'static, RunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>(
							"a", None,
						)
						.map(move |()| (before, after_insert))
					})
				})
		},
	)
}

fn rc_run_explicit_kv_store_program()
-> RcRunExplicit<'static, RcRunKVStoreRow, CNilBrand, (Option<i32>, Option<i32>)> {
	RcRunExplicit::<'static, RcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(
		|before| {
			RcRunExplicit::<'static, RcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>(
				"a",
				Some(9),
			)
			.bind(move |()| {
				RcRunExplicit::<'static, RcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>(
					"a",
				)
				.bind(move |after_insert| {
					RcRunExplicit::<'static, RcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>(
						"a", None,
					)
					.map(move |()| (before, after_insert))
				})
			})
		},
	)
}

fn arc_run_explicit_kv_store_program()
-> ArcRunExplicit<'static, ArcRunKVStoreRow, CNilBrand, (Option<i32>, Option<i32>)> {
	ArcRunExplicit::<'static, ArcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>("a").bind(
		|before| {
			ArcRunExplicit::<'static, ArcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>(
				"a",
				Some(9),
			)
			.bind(move |()| {
				ArcRunExplicit::<'static, ArcRunKVStoreRow, CNilBrand, Option<i32>>::lookup::<_, _>(
					"a",
				)
				.bind(move |after_insert| {
					ArcRunExplicit::<'static, ArcRunKVStoreRow, CNilBrand, ()>::update::<_, i32, _>(
						"a", None,
					)
					.map(move |()| (before, after_insert))
				})
			})
		},
	)
}

#[test]
fn run_fresh_helpers_thread_counter() {
	let custom: Run<CNilBrand, CNilBrand, ((usize, usize), usize)> =
		run_fresh_pair().run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
	assert_eq!(custom.extract(), ((10, 12), 14));

	let standard: Run<CNilBrand, CNilBrand, ((usize, usize), usize)> =
		run_fresh_pair().run_fresh::<_, CNilBrand>();
	assert_eq!(standard.extract(), ((0, 1), 2));
}

#[test]
fn rc_run_fresh_helpers_thread_counter() {
	let custom: RcRun<CNilBrand, CNilBrand, ((usize, usize), usize)> =
		rc_run_fresh_pair().run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
	assert_eq!(custom.extract(), ((10, 12), 14));

	let standard: RcRun<CNilBrand, CNilBrand, ((usize, usize), usize)> =
		rc_run_fresh_pair().run_fresh::<_, CNilBrand>();
	assert_eq!(standard.extract(), ((0, 1), 2));
}

#[test]
fn arc_run_fresh_helpers_thread_counter() {
	let custom: ArcRun<CNilBrand, CNilBrand, ((usize, usize), usize)> =
		arc_run_fresh_pair().run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
	assert_eq!(custom.extract(), ((10, 12), 14));

	let standard: ArcRun<CNilBrand, CNilBrand, ((usize, usize), usize)> =
		arc_run_fresh_pair().run_fresh::<_, CNilBrand>();
	assert_eq!(standard.extract(), ((0, 1), 2));
}

#[test]
fn run_explicit_fresh_helpers_thread_counter() {
	let custom: RunExplicit<'static, CNilBrand, CNilBrand, ((usize, usize), usize)> =
		run_explicit_fresh_pair().run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
	assert_eq!(custom.extract(), ((10, 12), 14));

	let standard: RunExplicit<'static, CNilBrand, CNilBrand, ((usize, usize), usize)> =
		run_explicit_fresh_pair().run_fresh::<_, CNilBrand>();
	assert_eq!(standard.extract(), ((0, 1), 2));
}

#[test]
fn rc_run_explicit_fresh_helpers_thread_counter() {
	let custom: RcRunExplicit<'static, CNilBrand, CNilBrand, ((usize, usize), usize)> =
		rc_run_explicit_fresh_pair()
			.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
	assert_eq!(custom.extract(), ((10, 12), 14));

	let standard: RcRunExplicit<'static, CNilBrand, CNilBrand, ((usize, usize), usize)> =
		rc_run_explicit_fresh_pair().run_fresh::<_, CNilBrand>();
	assert_eq!(standard.extract(), ((0, 1), 2));
}

#[test]
fn arc_run_explicit_fresh_helpers_thread_counter() {
	let custom: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((usize, usize), usize)> =
		arc_run_explicit_fresh_pair()
			.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
	assert_eq!(custom.extract(), ((10, 12), 14));

	let standard: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((usize, usize), usize)> =
		arc_run_explicit_fresh_pair().run_fresh::<_, CNilBrand>();
	assert_eq!(standard.extract(), ((0, 1), 2));
}

#[test]
fn run_input_seq_returns_none_after_exhaustion() {
	let handled: Run<CNilBrand, CNilBrand, InputTriple> =
		run_input_triple().run_input_seq::<&'static str, _, CNilBrand>(["red", "blue"]);
	assert_eq!(handled.extract(), (Some("red"), Some("blue"), None));
}

#[test]
fn rc_run_input_seq_returns_none_after_exhaustion() {
	let handled: RcRun<CNilBrand, CNilBrand, InputTriple> =
		rc_run_input_triple().run_input_seq::<&'static str, _, CNilBrand>(["red", "blue"]);
	assert_eq!(handled.extract(), (Some("red"), Some("blue"), None));
}

#[test]
fn arc_run_input_seq_returns_none_after_exhaustion() {
	let handled: ArcRun<CNilBrand, CNilBrand, InputTriple> =
		arc_run_input_triple().run_input_seq::<&'static str, _, CNilBrand>(["red", "blue"]);
	assert_eq!(handled.extract(), (Some("red"), Some("blue"), None));
}

#[test]
fn run_explicit_input_seq_returns_none_after_exhaustion() {
	let handled: RunExplicit<'static, CNilBrand, CNilBrand, InputTriple> =
		run_explicit_input_triple().run_input_seq::<&'static str, _, CNilBrand>(["red", "blue"]);
	assert_eq!(handled.extract(), (Some("red"), Some("blue"), None));
}

#[test]
fn rc_run_explicit_input_seq_returns_none_after_exhaustion() {
	let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, InputTriple> =
		rc_run_explicit_input_triple().run_input_seq::<&'static str, _, CNilBrand>(["red", "blue"]);
	assert_eq!(handled.extract(), (Some("red"), Some("blue"), None));
}

#[test]
fn arc_run_explicit_input_seq_returns_none_after_exhaustion() {
	let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, InputTriple> =
		arc_run_explicit_input_triple()
			.run_input_seq::<&'static str, _, CNilBrand>(["red", "blue"]);
	assert_eq!(handled.extract(), (Some("red"), Some("blue"), None));
}

#[test]
fn run_kv_store_applies_lookup_insert_and_delete() {
	let handled: RunKVHandled =
		run_kv_store_program().run_kv_store::<_, i32, _, CNilBrand>(initial_store());
	assert_eq!(handled.extract(), ((Some(7), Some(9)), final_store()));
}

#[test]
fn rc_run_kv_store_applies_lookup_insert_and_delete() {
	let handled: RcRunKVHandled =
		rc_run_kv_store_program().run_kv_store::<_, i32, _, CNilBrand>(initial_store());
	assert_eq!(handled.extract(), ((Some(7), Some(9)), final_store()));
}

#[test]
fn arc_run_kv_store_applies_lookup_insert_and_delete() {
	let handled: ArcRunKVHandled =
		arc_run_kv_store_program().run_kv_store::<_, i32, _, CNilBrand>(initial_store());
	assert_eq!(handled.extract(), ((Some(7), Some(9)), final_store()));
}

#[test]
fn run_explicit_kv_store_applies_lookup_insert_and_delete() {
	let handled: RunExplicitKVHandled =
		run_explicit_kv_store_program().run_kv_store::<_, i32, _, CNilBrand>(initial_store());
	assert_eq!(handled.extract(), ((Some(7), Some(9)), final_store()));
}

#[test]
fn rc_run_explicit_kv_store_applies_lookup_insert_and_delete() {
	let handled: RcRunExplicitKVHandled =
		rc_run_explicit_kv_store_program().run_kv_store::<_, i32, _, CNilBrand>(initial_store());
	assert_eq!(handled.extract(), ((Some(7), Some(9)), final_store()));
}

#[test]
fn arc_run_explicit_kv_store_applies_lookup_insert_and_delete() {
	let handled: ArcRunExplicitKVHandled =
		arc_run_explicit_kv_store_program().run_kv_store::<_, i32, _, CNilBrand>(initial_store());
	assert_eq!(handled.extract(), ((Some(7), Some(9)), final_store()));
}
