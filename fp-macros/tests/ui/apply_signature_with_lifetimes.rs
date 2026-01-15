use fp_macros::Apply;

struct MyBrand;

fn main() {
    type T = Apply!(
        brand: MyBrand,
        signature: (i32),
        lifetimes: ('a)
    );
}
