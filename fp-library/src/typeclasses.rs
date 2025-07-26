//! Typeclasses.

pub mod apply;
pub mod apply_first;
pub mod apply_second;
pub mod bind;
pub mod functor;
pub mod pure;

pub use self::apply::*;
pub use self::apply_first::*;
pub use self::apply_second::*;
pub use self::bind::*;
pub use self::functor::*;
pub use self::pure::*;
