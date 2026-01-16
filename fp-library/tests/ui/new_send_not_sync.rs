use fp_library::brands::ArcFnBrand;
use fp_library::classes::send_clonable_fn::SendClonableFn;
use std::cell::RefCell;

fn main() {
    let cell = RefCell::new(42);
    // Should fail because cell is not Sync, so the closure is not Sync
    let _ = <ArcFnBrand as SendClonableFn>::new_send(move |_: ()| {
        println!("{:?}", cell);
    });
}
