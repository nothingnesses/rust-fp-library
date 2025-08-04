//! Typeclasses.

pub mod applicative;
pub mod apply;
pub mod apply_first;
pub mod apply_second;
pub mod bind;
pub mod foldable;
pub mod functor;
pub mod monad;
pub mod monoid;
pub mod pure;
pub mod semigroup;

pub use self::applicative::Applicative;
pub use self::apply::Apply;
pub use self::apply_first::ApplyFirst;
pub use self::apply_second::ApplySecond;
pub use self::bind::Bind;
pub use self::foldable::Foldable;
pub use self::functor::Functor;
pub use self::monad::Monad;
pub use self::monoid::Monoid;
pub use self::pure::Pure;
pub use self::semigroup::Semigroup;
