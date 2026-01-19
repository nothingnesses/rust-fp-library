use fp_library::brands::ArcFnBrand;
use fp_library::classes::send_clonable_fn::SendClonableFn;
use std::rc::Rc;

fn main() {
    let rc = Rc::new(42);
    // Should fail because rc is not Send
    let _ = <ArcFnBrand as SendClonableFn>::send_clonable_fn_new(move |_: ()| {
        println!("{}", rc);
    });
}
