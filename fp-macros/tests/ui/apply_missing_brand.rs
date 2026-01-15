use fp_macros::Apply;

struct MyBrand;

fn main() {
    type T = Apply!(
        signature: (i32)
    );
}
