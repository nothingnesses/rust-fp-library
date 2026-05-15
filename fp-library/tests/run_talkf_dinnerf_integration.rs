//! Integration test for the canonical "lovely evening" Run example:
//! a Talk effect has `speak` and `listen`, a Dinner effect has `eat`
//! and `check_please`, and the program handles those effects in stages.
//!
//! The original PureScript source is
//! [`purescript-run/test/Examples.purs`](https://github.com/natefaubion/purescript-run/blob/abec7c343e92154d44b9dafd52b91ee82d32a870/test/Examples.purs#L13-L106).
//!
//! The Talk handler lowers `speak` into a standard State update that
//! appends to the transcript and lowers `listen` into a standard Reader
//! ask. The Dinner handler lowers food stock and billing into the same
//! State effect. The final Reader and State handlers close the program.
//! This keeps every observable result inside the effect program while
//! exercising custom first-order effects, built-in first-order effects,
//! row narrowing via `handle_with`, and all-handlers handling
//! via `handle`.

use {
	fp_library::{
		Apply,
		brands::{
			BoxBrand,
			BoxReaderBrand,
			BoxStateBrand,
			CNilBrand,
		},
		classes::{
			Functor,
			WrapDrop,
		},
		effects,
		handlers,
		impl_kind,
		kinds::*,
		types::effects::{
			reader::BoxReader,
			run::Run,
			state::BoxState,
		},
	},
	std::{
		cell::RefCell,
		rc::Rc,
	},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Food {
	Pizza,
	Chizburger,
}

type IsThereMore = bool;
type Bill = i32;

#[derive(Clone, Debug, PartialEq, Eq)]
struct EveningState {
	stock: i32,
	bill: Bill,
	transcript: Vec<String>,
}

struct TalkBrand;

enum TalkF<'a, A> {
	Speak(String, A),
	Listen(Box<dyn FnOnce(String) -> A + 'a>),
}

impl_kind! {
	impl for TalkBrand {
		type Of<'a, A: 'a>: 'a = TalkF<'a, A>;
	}
}

impl Functor for TalkBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			TalkF::Speak(line, next) => TalkF::Speak(line, f(next)),
			TalkF::Listen(reply) => TalkF::Listen(Box::new(move |input| f(reply(input)))),
		}
	}
}

impl WrapDrop for TalkBrand {
	fn drop<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> Option<A> {
		match fa {
			TalkF::Speak(_, next) => Some(next),
			TalkF::Listen(_) => None,
		}
	}
}

struct DinnerBrand;

enum DinnerF<'a, A> {
	Eat(Food, Box<dyn FnOnce(IsThereMore) -> A + 'a>),
	CheckPlease(Box<dyn FnOnce(Bill) -> A + 'a>),
}

impl_kind! {
	impl for DinnerBrand {
		type Of<'a, A: 'a>: 'a = DinnerF<'a, A>;
	}
}

impl Functor for DinnerBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			DinnerF::Eat(food, reply) => DinnerF::Eat(food, Box::new(move |more| f(reply(more)))),
			DinnerF::CheckPlease(reply) =>
				DinnerF::CheckPlease(Box::new(move |bill| f(reply(bill)))),
		}
	}
}

impl WrapDrop for DinnerBrand {
	fn drop<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> Option<A> {
		match fa {
			DinnerF::Eat(..) | DinnerF::CheckPlease(_) => None,
		}
	}
}

type FirstRow = effects![
	BoxReaderBrand<BoxBrand, &'static str>,
	BoxStateBrand<BoxBrand, EveningState>,
	DinnerBrand,
	TalkBrand
];
type AfterTalkRow = effects![
	BoxReaderBrand<BoxBrand, &'static str>,
	BoxStateBrand<BoxBrand, EveningState>,
	DinnerBrand
];
type AfterDinnerRow =
	effects![BoxReaderBrand<BoxBrand, &'static str>, BoxStateBrand<BoxBrand, EveningState>];
type ScopedRow = CNilBrand;

type Program<A> = Run<FirstRow, ScopedRow, A>;
type AfterTalkProgram<A> = Run<AfterTalkRow, ScopedRow, A>;
type AfterDinnerProgram<A> = Run<AfterDinnerRow, ScopedRow, A>;

fn speak(line: impl Into<String>) -> Program<()> {
	Run::lift::<TalkBrand, _>(TalkF::Speak(line.into(), ()))
}

fn listen() -> Program<String> {
	Run::lift::<TalkBrand, _>(TalkF::Listen(Box::new(|input| input)))
}

fn eat(food: Food) -> Program<IsThereMore> {
	Run::lift::<DinnerBrand, _>(DinnerF::Eat(food, Box::new(|more| more)))
}

fn check_please() -> Program<Bill> {
	Run::lift::<DinnerBrand, _>(DinnerF::CheckPlease(Box::new(|bill| bill)))
}

fn dinner_time() -> Program<()> {
	speak("I'm famished!").bind(|()| {
		eat(Food::Pizza).bind(|is_there_more| {
			if is_there_more {
				dinner_time()
			} else {
				check_please().bind(|_bill| speak("Outrageous!"))
			}
		})
	})
}

fn lovely_evening() -> Program<()> {
	listen().bind(|guest| speak(format!("Nice to meet you, {guest}!")).bind(|()| dinner_time()))
}

fn run_talk<A: 'static>(program: Program<A>) -> AfterTalkProgram<A> {
	program.handle_with::<TalkBrand, _, AfterTalkRow>(move |op: TalkF<'_, AfterTalkProgram<A>>| {
		match op {
			TalkF::Speak(line, next) =>
				Run::<AfterTalkRow, ScopedRow, EveningState>::get().bind(move |mut state| {
					state.transcript.push(line);
					Run::<AfterTalkRow, ScopedRow, ()>::put::<EveningState, _>(state)
						.bind(move |()| next)
				}),
			TalkF::Listen(reply) => Run::<AfterTalkRow, ScopedRow, &'static str>::ask()
				.bind(move |input| reply(input.to_string())),
		}
	})
}

fn run_dinner<A: 'static>(program: AfterTalkProgram<A>) -> AfterDinnerProgram<(Bill, A)> {
	let handled = program.handle_with::<DinnerBrand, _, AfterDinnerRow>(
		|op: DinnerF<'_, AfterDinnerProgram<A>>| match op {
			DinnerF::Eat(_food, reply) => Run::<AfterDinnerRow, ScopedRow, EveningState>::get()
				.bind(move |mut state| {
					if state.stock > 0 {
						state.stock -= 1;
						state.bill += 1;
						Run::<AfterDinnerRow, ScopedRow, ()>::put::<EveningState, _>(state)
							.bind(move |()| reply(true))
					} else {
						reply(false)
					}
				}),
			DinnerF::CheckPlease(reply) => Run::<AfterDinnerRow, ScopedRow, EveningState>::get()
				.bind(move |state| reply(state.bill)),
		},
	);

	handled.bind(|result| {
		Run::<AfterDinnerRow, ScopedRow, EveningState>::get().bind(move |state| {
			Run::<AfterDinnerRow, ScopedRow, (Bill, A)>::pure((state.bill, result))
		})
	})
}

fn close_reader_and_state(
	program: AfterDinnerProgram<(Bill, ())>,
	reader_value: &'static str,
	initial_state: EveningState,
) -> ((Bill, ()), EveningState) {
	let state_cell = Rc::new(RefCell::new(initial_state));
	let state_for_handler = Rc::clone(&state_cell);

	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, &'static str>: move |op: BoxReader<'_, BoxBrand, &'static str, AfterDinnerProgram<(Bill, ())>>| {
				match op {
					BoxReader::Ask(k) => k(reader_value),
				}
			},
			BoxStateBrand<BoxBrand, EveningState>: move |op: BoxState<'_, BoxBrand, EveningState, AfterDinnerProgram<(Bill, ())>>| {
				match op {
					BoxState::Get(k) => k(state_for_handler.borrow().clone()),
					BoxState::Put(next_state, k) => {
						*state_for_handler.borrow_mut() = next_state;
						k(())
					}
				}
			},
		},
		fp_library::types::effects::scoped_nt(),
	);

	let final_state = state_cell.borrow().clone();
	(result, final_state)
}

#[test]
fn talk_and_dinner_are_handled_in_stages() {
	let after_talk = run_talk(lovely_evening());
	let after_dinner = run_dinner(after_talk);
	let (result, final_state) = close_reader_and_state(
		after_dinner,
		"I am Groot",
		EveningState {
			stock: 10,
			bill: 0,
			transcript: Vec::new(),
		},
	);

	assert_eq!(result, (10, ()));
	assert_eq!(final_state.stock, 0);
	assert_eq!(final_state.bill, 10);
	assert_eq!(
		final_state.transcript.first().map(String::as_str),
		Some("Nice to meet you, I am Groot!")
	);
	assert_eq!(
		final_state.transcript.iter().filter(|line| line.as_str() == "I'm famished!").count(),
		11
	);
	assert_eq!(final_state.transcript.last().map(String::as_str), Some("Outrageous!"));
	assert_eq!(final_state.transcript.len(), 13);
}

#[test]
fn dinner_handler_reports_no_more_food_when_stock_is_empty() {
	let after_talk = run_talk(dinner_time());
	let after_dinner = run_dinner(after_talk);
	let (result, final_state) = close_reader_and_state(
		after_dinner,
		"I am Groot",
		EveningState {
			stock: 0,
			bill: 7,
			transcript: Vec::new(),
		},
	);

	assert_eq!(result, (7, ()));
	assert_eq!(final_state.stock, 0);
	assert_eq!(final_state.bill, 7);
	assert_eq!(
		final_state.transcript,
		vec!["I'm famished!".to_string(), "Outrageous!".to_string()]
	);
	assert_ne!(Food::Pizza, Food::Chizburger);
}
