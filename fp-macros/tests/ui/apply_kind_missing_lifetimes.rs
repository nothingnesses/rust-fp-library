use fp_macros::Apply;

struct MyBrand;
trait MyKind {}

fn main() {
    type T = Apply!(
        brand: MyBrand,
        kind: MyKind,
        types: (i32)
    );
}
