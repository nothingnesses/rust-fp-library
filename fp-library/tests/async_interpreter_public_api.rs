//! Public async-interpreter API, driven on a real runtime.
//!
//! Motivation. The async interpreter exposes two public pieces: the
//! `Run::await_future` smart constructor, which embeds a `Future` into a
//! program's first-order row as an await effect, and the `Run::run_async`
//! method, which drives such a program to completion as a runtime-agnostic
//! future. This test exercises both from an external crate (the same vantage
//! point a library user has), on the Tokio runtime, to confirm the public
//! surface composes and genuinely awaits.
//!
//! Method. The program interleaves the two public abilities: it awaits a real
//! Tokio timer, feeds the result through an ordinary `Identity` handler, then
//! awaits another timer computed from that result. The await effect sits at
//! the head of the first-order row (the position `run_async` interprets); the
//! `Identity` effect is the row tail, dispatched to the handler list. Because
//! the embedded futures are real timers, completion requires the Tokio
//! scheduler to drive them, not a single poll. The program threads
//! 20 -> 21 -> 42.

#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::{
			AwaitBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
		},
		handlers,
		types::{
			Identity,
			effects::run::Run,
		},
	},
	std::time::Duration,
};

type Rest = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type Row = CoproductBrand<CoyonedaBrand<AwaitBrand>, Rest>;
type Prog<A> = Run<Row, CNilBrand, A>;

#[tokio::test(flavor = "current_thread")]
async fn public_api_runs_await_program_on_tokio() {
	let program: Prog<u64> = Run::await_future(async {
		tokio::time::sleep(Duration::from_millis(1)).await;
		20
	})
	.bind(|first| Run::lift::<IdentityBrand, _>(Identity(first + 1)))
	.bind(|second| {
		Run::await_future(async move {
			tokio::time::sleep(Duration::from_millis(1)).await;
			second * 2
		})
	});

	let result = program
		.run_async(handlers! {
			IdentityBrand: |operation: Identity<Prog<u64>>| operation.0,
		})
		.await;

	assert_eq!(result, 42);
}
