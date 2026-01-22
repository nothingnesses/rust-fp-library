use fp_library::brands::ArcFnBrand;
use fp_library::classes::send_cloneable_fn::SendCloneableFn;
use std::cell::RefCell;

fn main() {
    let cell = RefCell::new(42);
    // Should fail because cell is not Sync, so the closure is not Sync
    let _ = <ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(move |_: ()| {
        println!("{:?}", cell);
    });
}
