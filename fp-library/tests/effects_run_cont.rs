//! The whole-row CPS driver, driven end to end: on suspension, `run_cont`
//! hands the operation callback the row layer with every continuation
//! already folded into a target-producing application; on completion the
//! pure callback fires. The callback owns each step, so it can force the
//! continuation synchronously (a direct drive) or store it and return (a
//! scheduling driver that trampolines, spending no native stack on program
//! depth).
//!
//! Pinned here: (1) a synchronous callback drives a multi-effect row to
//! completion by brand-keyed selection; (2) a scheduling callback defers
//! each force into an external loop, driving a 100k-deep program with
//! constant native stack.
#![cfg(feature = "effects")]

use {
	fp_library::{
		define_effect,
		define_row,
		types::{
			Coyoneda,
			Free,
			effects::handle::run_cont,
		},
	},
	std::{
		cell::{
			Cell,
			RefCell,
		},
		rc::Rc,
	},
};

define_effect! {
	/// Doubles a number.
	#[handler_state(none)]
	pub effect Double {
		/// Resume with twice `value`.
		fn double(value: i32) -> i32;
	}
}

define_effect! {
	/// A unit step.
	#[handler_state(none)]
	pub effect Tick {
		/// Advance one step, resuming with unit.
		fn tick() -> ();
	}
}

// -- The synchronous drive --

define_row! {
	/// Doubling alongside ticking.
	pub row MathRow {
		DoubleBrand,
		TickBrand,
	}
}

#[test]
fn a_synchronous_callback_drives_a_multi_effect_row_to_completion() {
	// The callback selects each suspended operation by brand and forces its
	// continuation immediately, so the whole program collapses to its value.
	let ticks = Rc::new(Cell::new(0));
	let observed = ticks.clone();
	let program: Free<MathRow, i32> = double(2).bind(|four| tick().bind(move |()| double(four)));
	let result = run_cont(
		program,
		move |layer| {
			let layer = match layer.uninject() {
				Ok(cell) => {
					let cell: Coyoneda<'static, DoubleBrand, i32> = cell;
					return match cell.lower() {
						DoubleF::Double(value, resume) => resume(value * 2),
					};
				}
				Err(rest) => rest,
			};
			match layer.uninject() {
				Ok(cell) => {
					let cell: Coyoneda<'static, TickBrand, i32> = cell;
					match cell.lower() {
						TickF::Tick(resume) => {
							observed.set(observed.get() + 1);
							resume(())
						}
					}
				}
				Err(terminal) => match terminal {},
			}
		},
		|value| value,
	);
	assert_eq!(result, 8);
	assert_eq!(ticks.get(), 1);
}

// -- The scheduling drive --

define_row! {
	/// The one-cell ticking row.
	pub row TickRow {
		TickBrand,
	}
}

/// One deferred continuation force, stored by the callback and run by the
/// external loop.
type Thunk = Box<dyn FnOnce()>;

#[test]
fn a_scheduling_callback_defers_each_force_and_drives_deep_programs_iteratively() {
	// The callback never forces a continuation: it stores the force as a
	// thunk and returns, so each program layer costs one queue exchange in
	// the external loop rather than a native stack frame. 100k layers drive
	// to completion.
	const DEPTH: usize = 100_000;
	let pending: Rc<RefCell<Option<Thunk>>> = Rc::new(RefCell::new(None));
	let finished: Rc<Cell<Option<i32>>> = Rc::new(Cell::new(None));

	let mut program: Free<TickRow, i32> = Free::pure(7);
	for _ in 0 .. DEPTH {
		program = tick().bind(move |()| program);
	}

	let queue = pending.clone();
	let done = finished.clone();
	run_cont(
		program,
		move |layer| {
			let cell: Coyoneda<'static, TickBrand, ()> = match layer.uninject() {
				Ok(cell) => cell,
				Err(terminal) => match terminal {},
			};
			match cell.lower() {
				TickF::Tick(resume) => {
					queue.borrow_mut().replace(Box::new(move || resume(())));
				}
			}
		},
		move |value| {
			done.set(Some(value));
		},
	);

	let mut steps = 0_usize;
	loop {
		// Take the thunk in its own statement so the borrow ends before the
		// thunk re-enters the callback and stores its successor.
		let next = pending.borrow_mut().take();
		let Some(thunk) = next else { break };
		thunk();
		steps += 1;
	}
	assert_eq!(steps, DEPTH);
	assert_eq!(finished.get(), Some(7));
}
