use fp_lib::{
	brands::{OptionBrand, ResultWithErrBrand},
	functions::sequence,
};

fn main() {
	let a = |x: &usize| x.to_owned() + 1;
	println!("{:?}", sequence::<OptionBrand, _, _, _>(&Some(a))(&Some(0)));
	println!("{:?}", sequence::<ResultWithErrBrand<_>, _, _, _>(&Ok::<_, ()>(a))(&Ok::<_, ()>(0)));
}
