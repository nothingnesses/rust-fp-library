use fp_macros::Apply;

struct MyBrand;
trait MyKind {}

fn main() {
    type T = Apply!(
        brand: MyBrand,
        signature: (i32),
        kind: MyKind
    );
}
