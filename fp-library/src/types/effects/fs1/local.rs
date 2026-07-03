//! FS-1 slice: the `Local` higher-order effect.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructors, and its bucket A parity test. `Local`
//! owns a sub-program and an environment transform; the parent interpreter
//! elaborates it by running the action under a `Handlers` whose `env` is the
//! transform applied to the inherited one, so the `Reader` `ask`s inside the
//! action see the modified environment (no boundary frame). The cell fields are
//! `pub(super)` because the parent interpreter destructures them.

use {
	super::{
		HigherOrder,
		Node,
		OrderOf,
		Row,
	},
	crate::{
		Apply,
		classes::Functor,
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::coproduct::Coproduct,
		},
	},
	std::marker::PhantomData,
};

/// Local is a higher-order effect: it owns an action sub-program and an
/// environment transform `modify`. The interpreter elaborates it by running the
/// action under a `Handlers` whose `env` is `modify(env)`, so every `Reader`
/// `ask` inside the action reads the transformed environment for the duration of
/// the action; the action's result then flows to the continuation (no boundary
/// frame). The cell is generic in the environment type `Env` and the action
/// result `RAction`; this slice's `Row` pins both to `i32`.
pub(crate) struct LocalBrand<Env, RAction>(PhantomData<(Env, RAction)>);
pub(crate) struct LocalCell<'a, Env: 'static, RAction: 'static, Next> {
	pub(super) modify: Box<dyn FnOnce(Env) -> Env + 'a>,
	pub(super) action: Free<Row, RAction>,
	pub(super) k: Box<dyn FnOnce(RAction) -> Next + 'a>,
}
impl_kind! {
	impl<Env: 'static, RAction: 'static> for LocalBrand<Env, RAction> {
		type Of<'a, Next: 'a>: 'a = LocalCell<'a, Env, RAction, Next>;
	}
}
impl<Env: 'static, RAction: 'static> Functor for LocalBrand<Env, RAction> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		let LocalCell {
			modify,
			action,
			k,
		} = fa;
		LocalCell {
			modify,
			action,
			k: Box::new(move |v| f(k(v))),
		}
	}
}
impl<Env: 'static, RAction: 'static> OrderOf for LocalBrand<Env, RAction> {
	type Order = HigherOrder;
}

pub(crate) fn local(
	modify: impl FnOnce(i32) -> i32 + 'static,
	action: Free<Row, i32>,
) -> Free<Row, i32> {
	let cell: LocalCell<'static, i32, i32, i32> = LocalCell {
		modify: Box::new(modify),
		action,
		k: Box::new(|v| v),
	};
	let coyo: Coyoneda<'static, LocalBrand<i32, i32>, i32> = Coyoneda::lift(cell);
	let node: Node<i32> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

/// The by-reference sibling: `modify` borrows the inherited environment rather
/// than taking it by value. Because the environment is `i32` (`Copy`), it folds
/// into the same [`LocalCell`] by adapting the borrow at the call site.
pub(crate) fn ref_local(
	modify: impl FnOnce(&i32) -> i32 + 'static,
	action: Free<Row, i32>,
) -> Free<Row, i32> {
	local(move |env| modify(&env), action)
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Fixture,
			Row,
			ask,
			local,
			ref_local,
			run,
		},
	};

	// Behaviour-parity oracle bucket A: Local environment modification scope.
	// `local(|e| e + 1, ask().bind(|env| pure(env * 2)))` over env 10 yields
	// `(10 + 1) * 2 = 22`: the `ask` inside the action reads the transformed
	// environment for the duration of the action.
	#[test]
	fn local_runs_action_under_the_modified_environment() {
		let program: Free<Row, i32> = local(|e| e + 1, ask().bind(|env| Free::pure(env * 2)));

		let fx = Fixture::with_env(10);
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(22));
	}

	// Behaviour-parity oracle bucket A: an outer `map` runs after the action's
	// result flows out. `22` mapped by `|v| v + 1` is `23`.
	#[test]
	fn local_outer_map_runs_after_the_action() {
		let program: Free<Row, i32> =
			local(|e| e + 1, ask().bind(|env| Free::pure(env * 2))).map(|v| v + 1);

		let fx = Fixture::with_env(10);
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(23));
	}

	// Behaviour-parity oracle bucket A: an outer `bind` runs after the action's
	// result flows out. `22` bound by `|v| pure(v + 20)` is `42`.
	#[test]
	fn local_outer_bind_runs_after_the_action() {
		let program: Free<Row, i32> =
			local(|e| e + 1, ask().bind(|env| Free::pure(env * 2))).bind(|v| Free::pure(v + 20));

		let fx = Fixture::with_env(10);
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(42));
	}

	// Behaviour-parity oracle bucket A: the by-reference `ref_local`. Its
	// `modify` borrows the environment (`|e| *e + 5`); over env 10 the action
	// reads `15`, so `15 * 2 = 30`.
	#[test]
	fn ref_local_runs_action_under_the_modified_environment() {
		let program: Free<Row, i32> = ref_local(|e| *e + 5, ask().bind(|env| Free::pure(env * 2)));

		let fx = Fixture::with_env(10);
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(30));
	}
}
