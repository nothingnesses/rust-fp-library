//! Type classes.
//!
//! Higher-kinded type classes (those with arities > 0, e.g., [`Functor`],
//! which has arity 1) are usually implemented by
//! [`Brand` types][crate::brands], which are higher-kinded (arities > 0)
//! representation of [types][crate::types], instead of directly by concrete
//! types (which have arity 0).

pub mod applicative;
pub mod apply_first;
pub mod apply_second;
pub mod category;
pub mod clonable_fn;
pub mod defer;
pub mod foldable;
pub mod function;
pub mod functor;
pub mod monad;
pub mod monoid;
pub mod pointed;
pub mod semiapplicative;
pub mod semigroup;
pub mod semigroupoid;
pub mod semimonad;
pub mod traversable;

pub use self::applicative::Applicative;
pub use self::apply_first::ApplyFirst;
pub use self::apply_second::ApplySecond;
pub use self::category::Category;
pub use self::clonable_fn::ClonableFn;
pub use self::defer::Defer;
pub use self::foldable::Foldable;
pub use self::function::Function;
pub use self::functor::Functor;
pub use self::monad::Monad;
pub use self::monoid::Monoid;
pub use self::pointed::Pointed;
pub use self::semiapplicative::Semiapplicative;
pub use self::semigroup::Semigroup;
pub use self::semigroupoid::Semigroupoid;
pub use self::semimonad::Semimonad;
pub use self::traversable::Traversable;
