//! Focused substrate tests for scoped Writer operation cells.
//!
//! These tests exercise the neutral `censor` and `listen` cells before
//! wrapper smart constructors and standard Writer handlers are layered
//! on top. They verify the cell-level shape only: `censor` keeps a
//! same-result action and a log transformation, while `listen` records
//! the selected action value type separately from the action program
//! stored in the row slot.

use {
	core::marker::PhantomData,
	fp_library::{
		brands::{
			ArcBrand,
			BoxBrand,
			BoxWriterCensorBrand,
			BoxWriterListenBrand,
			RcBrand,
			SendWriterCensorBrand,
			SendWriterListenBrand,
			WriterCensorBrand,
			WriterListenBrand,
		},
		classes::{
			Functor,
			SendFunctor,
			ToDynCloneFn,
			ToDynFn,
			ToDynFnOnce,
			ToDynSendFn,
		},
		types::effects::writer::{
			BoxWriterCensor,
			BoxWriterListen,
			SendWriterCensor,
			SendWriterListen,
			WriterCensor,
			WriterListen,
		},
	},
	std::{
		cell::Cell,
		rc::Rc,
	},
};

#[test]
fn censor_cells_map_action_without_consuming_log_transform() {
	let box_cell: BoxWriterCensor<'static, BoxBrand, String, i32> = BoxWriterCensor::Censor {
		censor: <BoxBrand as ToDynFn>::new(|log: String| format!("{log}!")),
		action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
	};
	let box_mapped =
		<BoxWriterCensorBrand<BoxBrand, String> as Functor>::map(|value| value + 1, box_cell);
	match box_mapped {
		BoxWriterCensor::Censor {
			censor,
			action,
		} => {
			assert_eq!(censor("hello".to_owned()), "hello!");
			assert_eq!(action(()), 42);
		}
	}

	let rc_cell: WriterCensor<'static, RcBrand, String, i32> = WriterCensor::Censor {
		censor: <RcBrand as ToDynCloneFn>::new(|log: String| format!("{log}?")),
		action: <RcBrand as ToDynCloneFn>::new(|_: ()| 20),
	};
	let rc_mapped =
		<WriterCensorBrand<RcBrand, String> as Functor>::map(|value| value * 2, rc_cell);
	match rc_mapped {
		WriterCensor::Censor {
			censor,
			action,
		} => {
			assert_eq!(censor("ready".to_owned()), "ready?");
			assert_eq!(action(()), 40);
			assert_eq!(action(()), 40);
		}
	}

	let arc_cell: SendWriterCensor<'static, ArcBrand, String, i32> = SendWriterCensor::Censor {
		censor: <ArcBrand as ToDynSendFn>::new(|log: String| format!("{log}.")),
		action: <ArcBrand as ToDynSendFn>::new(|_: ()| 14),
	};
	let arc_mapped = <SendWriterCensorBrand<ArcBrand, String> as SendFunctor>::send_map(
		|value| value * 3,
		arc_cell,
	);
	match arc_mapped {
		SendWriterCensor::Censor {
			censor,
			action,
		} => {
			assert_eq!(censor("done".to_owned()), "done.");
			assert_eq!(action(()), 42);
			assert_eq!(action(()), 42);
		}
	}
}

#[test]
fn box_censor_transform_is_reusable_while_action_stays_single_shot() {
	let call_count = Rc::new(Cell::new(0));
	let calls_from_censor = Rc::clone(&call_count);
	let action_capture = String::from("hello");

	let cell: BoxWriterCensor<'static, BoxBrand, String, Vec<String>> = BoxWriterCensor::Censor {
		censor: <BoxBrand as ToDynFn>::new(move |log: String| {
			calls_from_censor.set(calls_from_censor.get() + 1);
			format!("{log}!")
		}),
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| {
			vec![action_capture, String::from("world")]
		}),
	};

	match cell {
		BoxWriterCensor::Censor {
			censor,
			action,
		} => {
			let emitted_logs = action(());
			let transformed_logs: Vec<_> = emitted_logs.into_iter().map(censor).collect();

			assert_eq!(transformed_logs, vec![String::from("hello!"), String::from("world!")]);
			assert_eq!(call_count.get(), 2);
		}
	}
}

#[test]
fn listen_cells_map_action_program_while_preserving_action_marker() {
	let box_cell: BoxWriterListen<'static, BoxBrand, String, i32, &'static str> =
		BoxWriterListen::Listen {
			action: <BoxBrand as ToDynFnOnce>::new(|_: ()| "selected"),
			result: PhantomData,
		};
	let box_mapped =
		<BoxWriterListenBrand<BoxBrand, String, i32> as Functor>::map(str::len, box_cell);
	match box_mapped {
		BoxWriterListen::Listen {
			action,
			result: _,
		} => {
			let _: PhantomData<fn(i32) -> String> = PhantomData;
			assert_eq!(action(()), "selected".len());
		}
	}

	let rc_cell: WriterListen<'static, RcBrand, String, i32, &'static str> = WriterListen::Listen {
		action: <RcBrand as ToDynCloneFn>::new(|_: ()| "again"),
		result: PhantomData,
	};
	let rc_mapped = <WriterListenBrand<RcBrand, String, i32> as Functor>::map(str::len, rc_cell);
	match rc_mapped {
		WriterListen::Listen {
			action,
			result: _,
		} => {
			let _: PhantomData<fn(i32) -> String> = PhantomData;
			assert_eq!(action(()), "again".len());
			assert_eq!(action(()), "again".len());
		}
	}

	let arc_cell: SendWriterListen<'static, ArcBrand, String, i32, &'static str> =
		SendWriterListen::Listen {
			action: <ArcBrand as ToDynSendFn>::new(|_: ()| "shared"),
			result: PhantomData,
		};
	let arc_mapped =
		<SendWriterListenBrand<ArcBrand, String, i32> as SendFunctor>::send_map(str::len, arc_cell);
	match arc_mapped {
		SendWriterListen::Listen {
			action,
			result: _,
		} => {
			let _: PhantomData<fn(i32) -> String> = PhantomData;
			assert_eq!(action(()), "shared".len());
			assert_eq!(action(()), "shared".len());
		}
	}
}
