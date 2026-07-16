//! The multi-shot `SubShift` effect driven end to end: capture, per-branch
//! re-entry, abort by drop, the state ordering zoo under both stacking
//! orders, the NonDet derivation (first-order nondeterminism falls out of
//! calling one captured continuation once per branch, the `Alt` fan-out
//! reproduced), and the fork primitive derived over the delimiter's exit
//! (the program receives its own re-callable continuation as a value).

#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::{
			CNilBrand,
			RcBrand,
		},
		define_row,
		types::{
			Free,
			effects::{
				handle::multi_shot::{
					extract,
					handle_accum,
				},
				state::{
					StateBrand,
					StateF,
					get,
					put,
				},
				sub_shift::{
					SubShiftBrand,
					SubShiftExit,
					run_sub_shift,
					sub_shift,
				},
			},
		},
	},
	std::rc::Rc,
};

define_row! {
	/// A one-cell prompt delimiting straight to the empty row.
	pub row PromptRow {
		SubShiftBrand<CNilBrand, i32, i32>,
	}
}

/// The `State` step shared by the zoo tests.
fn state_step<P>(
	s: i32,
	op: StateF<'static, i32, P>,
) -> (i32, P) {
	match op {
		StateF::Get(k) => (s, k(s)),
		StateF::Put(next, k) => (next, k(())),
	}
}

// The one-shot composition still holds on the multi-shot delimiter: the
// body resumes once with 5, the tail adds 1, the body doubles the
// completed answer.
#[test]
fn capture_resumes_and_post_processes() {
	let program: Free<PromptRow, i32, RcBrand> = sub_shift::<_, _, i32, _, _>(|exit| {
		exit(5).bind_multi_shot(|ans: i32| Free::pure(ans * 2))
	})
	.bind_multi_shot(|v: i32| Free::pure(v + 1));
	let narrowed: Free<CNilBrand, i32, RcBrand> = run_sub_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 12);
}

// The multi-shot moment: one captured continuation, two invocations, each
// re-running the tail (x10) to a fresh completed answer.
#[test]
fn one_continuation_invoked_once_per_branch() {
	let program: Free<PromptRow, i32, RcBrand> = sub_shift::<_, _, i32, _, _>(|exit| {
		let again = exit.clone();
		exit(1).bind_multi_shot(move |first: i32| {
			again(2).bind_multi_shot(move |second: i32| Free::pure(first + second))
		})
	})
	.bind_multi_shot(|v: i32| Free::pure(v * 10));
	let narrowed: Free<CNilBrand, i32, RcBrand> = run_sub_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 30);
}

// Dropping the continuation aborts the unresumed tail structurally: the
// body never invokes it, supplies the answer directly, and the tail's
// probe never fires.
#[test]
fn dropping_the_continuation_aborts_the_tail() {
	let tail_ran = Rc::new(std::cell::Cell::new(false));
	let probe = tail_ran.clone();
	let program: Free<PromptRow, i32, RcBrand> =
		sub_shift::<_, _, i32, _, _>(|_exit| Free::pure(-1)).bind_multi_shot(move |v: i32| {
			probe.set(true);
			Free::pure(v)
		});
	let narrowed: Free<CNilBrand, i32, RcBrand> = run_sub_shift(program, Free::pure);
	assert_eq!(extract(narrowed), -1);
	assert!(!tail_ran.get());
}

define_row! {
	/// A forking prompt collecting every branch leaf.
	pub row AltPromptRow {
		SubShiftBrand<CNilBrand, Vec<(bool, bool)>, bool>,
	}
}

/// The NonDet body: invokes the continuation once per branch (`true`
/// first) and concatenates the completed answers.
fn alt_body(
	exit: SubShiftExit<CNilBrand, Vec<(bool, bool)>, bool>
) -> Free<CNilBrand, Vec<(bool, bool)>, RcBrand> {
	let other = exit.clone();
	exit(true).bind_multi_shot(move |kept: Vec<(bool, bool)>| {
		other(false).bind_multi_shot(move |rest: Vec<(bool, bool)>| {
			let mut all = kept.clone();
			all.extend(rest);
			Free::pure(all)
		})
	})
}

// The NonDet derivation: two captures, each body invoking its continuation
// once per branch, reproduce the Alt effect's four-leaf fan-out in the
// same true-first depth-first order. Multi-shot shift is the primitive
// first-order nondeterminism falls out of.
#[test]
fn nondeterminism_derives_from_per_branch_re_entry() {
	let program: Free<AltPromptRow, (bool, bool), RcBrand> =
		sub_shift::<_, _, bool, _, _>(alt_body).bind_multi_shot(|first: bool| {
			sub_shift::<_, _, bool, _, _>(alt_body)
				.bind_multi_shot(move |second: bool| Free::pure((first, second)))
		});
	let narrowed: Free<CNilBrand, Vec<(bool, bool)>, RcBrand> =
		run_sub_shift(program, |leaf| Free::pure(vec![leaf]));
	assert_eq!(extract(narrowed), vec![(true, true), (true, false), (false, true), (false, false)]);
}

define_row! {
	/// A forking prompt beside integer state, delimiting to the state-only
	/// residual: the zoo's global order (the delimiter runs first).
	pub row GlobalZooRow {
		SubShiftBrand<GlobalStateRow, Vec<i32>, bool>,
		StateBrand<i32>,
	}
}

define_row! {
	/// The residual row after the global-order prompt is delimited.
	pub row GlobalStateRow {
		StateBrand<i32>,
	}
}

/// The forking body over the state-only residual.
fn global_zoo_body(
	exit: SubShiftExit<GlobalStateRow, Vec<i32>, bool>
) -> Free<GlobalStateRow, Vec<i32>, RcBrand> {
	let other = exit.clone();
	exit(true).bind_multi_shot(move |left: Vec<i32>| {
		other(false).bind_multi_shot(move |right: Vec<i32>| {
			let mut all = left.clone();
			all.extend(right);
			Free::pure(all)
		})
	})
}

// Global order (the zoo's state-outside case): the delimiter runs first,
// so both branches' state operations sequence into one residual and the
// right branch reads the left branch's write; the oracle is the alt and
// scoped zoos' (6, [3, 6]).
#[test]
fn state_outside_the_delimiter_threads_one_state_through_branches() {
	let program: Free<GlobalZooRow, i32, RcBrand> = put::<i32, GlobalZooRow, _, RcBrand>(1)
		.bind_multi_shot(|()| {
			sub_shift::<_, _, bool, _, _>(global_zoo_body).bind_multi_shot(|left: bool| {
				let advance = if left { 2 } else { 3 };
				get::<i32, GlobalZooRow, _, RcBrand>().bind_multi_shot(move |seen: i32| {
					put::<i32, GlobalZooRow, _, RcBrand>(seen + advance)
						.bind_multi_shot(|()| get::<i32, GlobalZooRow, _, RcBrand>())
				})
			})
		});
	let narrowed: Free<GlobalStateRow, Vec<i32>, RcBrand> =
		run_sub_shift(program, |leaf| Free::pure(vec![leaf]));
	let stated: Free<CNilBrand, (i32, Vec<i32>), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(0, narrowed, state_step);
	assert_eq!(extract(stated), (6, vec![3, 6]));
}

define_row! {
	/// A forking prompt beside integer state, delimiting to the empty row:
	/// the zoo's branch-local order (the state fold runs first and rides
	/// the re-emitted capture cell).
	pub row LocalZooRow {
		SubShiftBrand<CNilBrand, Vec<(i32, i32)>, bool>,
		StateBrand<i32>,
	}
}

define_row! {
	/// The prompt-only row after the state fold narrows the zoo row.
	pub row LocalPromptRow {
		SubShiftBrand<CNilBrand, Vec<(i32, i32)>, bool>,
	}
}

/// The forking body over the empty residual, at the state-paired leaf.
fn local_zoo_body(
	exit: SubShiftExit<CNilBrand, Vec<(i32, i32)>, bool>
) -> Free<CNilBrand, Vec<(i32, i32)>, RcBrand> {
	let other = exit.clone();
	exit(true).bind_multi_shot(move |left: Vec<(i32, i32)>| {
		other(false).bind_multi_shot(move |right: Vec<(i32, i32)>| {
			let mut all = left.clone();
			all.extend(right);
			Free::pure(all)
		})
	})
}

// Branch-local order (the zoo's state-inside case): the state fold stacked
// under the delimiter rides the re-emitted capture cell and is cloned into
// each re-entry, so both branches read the prefix state and their writes
// die with the branch; the oracle is the alt zoo's [(3, 3), (4, 4)].
#[test]
fn state_under_the_delimiter_forks_per_re_entry() {
	let program: Free<LocalZooRow, i32, RcBrand> = put::<i32, LocalZooRow, _, RcBrand>(1)
		.bind_multi_shot(|()| {
			sub_shift::<_, _, bool, _, _>(local_zoo_body).bind_multi_shot(|left: bool| {
				let advance = if left { 2 } else { 3 };
				get::<i32, LocalZooRow, _, RcBrand>().bind_multi_shot(move |seen: i32| {
					put::<i32, LocalZooRow, _, RcBrand>(seen + advance)
						.bind_multi_shot(|()| get::<i32, LocalZooRow, _, RcBrand>())
				})
			})
		});
	let folded: Free<LocalPromptRow, (i32, i32), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(0, program, state_step);
	let narrowed: Free<CNilBrand, Vec<(i32, i32)>, RcBrand> =
		run_sub_shift(folded, |leaf| Free::pure(vec![leaf]));
	assert_eq!(extract(narrowed), vec![(3, 3), (4, 4)]);
}

/// The fork primitive's resume value: the program either receives its own
/// re-callable continuation (the fork parent) or a plain value (a resumed
/// branch), heftia's `SubShiftFork` shape derived over the delimiter's
/// exit.
#[derive(Clone)]
pub enum ForkResume {
	/// The fork parent: holds the continuation that re-enters this same
	/// capture with a resumed value.
	Fork(Rc<dyn Fn(i32) -> Free<CNilBrand, Vec<i32>, RcBrand>>),
	/// A resumed branch, carrying the value the continuation was invoked
	/// with.
	Resumed(i32),
}

define_row! {
	/// The fork-primitive prompt.
	pub row ForkPromptRow {
		SubShiftBrand<CNilBrand, Vec<i32>, ForkResume>,
	}
}

// The fork primitive derives over the exit in one body: resume first with
// the continuation itself (wrapped to feed re-entries through `Resumed`),
// so the program-side answer clause can invoke its own continuation once
// per branch. Two invocations from the answer clause collect two leaves.
#[test]
fn fork_primitive_derives_over_the_exit() {
	let program: Free<ForkPromptRow, ForkResume, RcBrand> =
		sub_shift::<_, _, ForkResume, _, _>(|exit| {
			let re_enter = {
				let through = exit.clone();
				Rc::new(move |v: i32| through(ForkResume::Resumed(v)))
			};
			exit(ForkResume::Fork(re_enter))
		});
	let narrowed: Free<CNilBrand, Vec<i32>, RcBrand> =
		run_sub_shift(program, |resume| match resume {
			ForkResume::Fork(cont) => {
				let again = cont.clone();
				cont(10).bind_multi_shot(move |first: Vec<i32>| {
					again(20).bind_multi_shot(move |second: Vec<i32>| {
						let mut all = first.clone();
						all.extend(second);
						Free::pure(all)
					})
				})
			}
			ForkResume::Resumed(v) => Free::pure(vec![v]),
		});
	assert_eq!(extract(narrowed), vec![10, 20]);
}
