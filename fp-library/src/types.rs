//! Data types and their corresponding implementations.

pub mod arc_fn;
pub mod endofunction;
pub mod endomorphism;
pub mod identity;
pub mod lazy;
pub mod once_cell;
pub mod once_lock;
pub mod option;
pub mod pair;
pub mod rc_fn;
pub mod result;
pub mod string;
pub mod vec;

pub use self::endofunction::Endofunction;
pub use self::endomorphism::Endomorphism;
pub use self::identity::Identity;
pub use self::lazy::Lazy;
pub use self::pair::Pair;
