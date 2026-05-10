// Verifies that `bracket` rejects body and release closures with the
// wrong resource argument shape. The Box-backed `Run::bracket`
// constructor passes a boxed resource into both closures; closures that
// accept the resource by value are neither the Val-owned bracket shape
// nor any Ref-counted pointer shape.

use fp_library::{
	Apply,
	brands::{
		BoxBracketBrand,
		BoxBrand,
		CNilBrand,
		CoproductBrand,
		NodeBrand,
	},
	classes::{
		Functor,
		WrapDrop,
	},
	impl_kind,
	kinds::*,
	types::effects::run::Run,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ScopedRow;

type FirstRow = CNilBrand;
type UnderlyingRow =
	CoproductBrand<BoxBracketBrand<BoxBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>, CNilBrand>;
type Prog = Run<FirstRow, ScopedRow, i32>;

impl_kind! {
	impl for ScopedRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for ScopedRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<UnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for ScopedRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<UnderlyingRow as Functor>::map(f, fa)
	}
}

fn main() {
	let acquire: Prog = Run::pure(7);
	let _program: Prog = Run::<FirstRow, ScopedRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: i32| Run::pure((resource, resource + 35)),
		|_resource: i32| Run::pure(()),
	);
}
