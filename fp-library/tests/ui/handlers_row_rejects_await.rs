//! A `#[handlers]` row cannot hold the `Await` base-lift cell: the one-pass
//! handler surface requires every member to carry the emitted
//! `HandlerPieces` impl, and `Await` has none, because interpreting it is a
//! suspension an async driver awaits rather than an arm a synchronous loop
//! applies. Await-bearing rows interpret through the narrowing runners and
//! the `run_async` terminal driver instead.
#![cfg(feature = "effects")]

use fp_library::{
	brands::AwaitBrand,
	define_row,
	types::effects::state::StateBrand,
};

define_row! {
	/// Integer state alongside the future base-lift cell.
	#[handlers]
	pub row AppRow {
		StateBrand<i32>,
		AwaitBrand,
	}
}

fn main() {}
