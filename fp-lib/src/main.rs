use fp_lib::{
	Functions,
	typeclasses::functor::Functor2,
	types::{
		either::{Either, EitherBrand},
		maybe::{Maybe, MaybeBrand},
	},
};

fn main() {
	println!("{:?}", Functions::map::<MaybeBrand, _, _, _>(|x| x + 1)(Maybe::Just(0)));
	println!(
		"{:?}",
		<Either<(), usize> as Functor2<EitherBrand, (), usize>>::map(|x| x + 1)(Either::Right(0))
	);
}
