use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	marker::PhantomData,
	ops::Deref,
	rc::Rc,
};

fn main() {
	// Example usage of the HKT Endomorphism Monoid
	let add_one = RcFnBrand::new(|x: i32| x + 1);
	let times_two = RcFnBrand::new(|x: i32| x * 2);

	// Create two endomorphisms
	let endo_add_one = Endomorphism(add_one);
	let endo_times_two = Endomorphism(times_two);

	// Use the HKT `append` function. Note the composition order is (b)(a) -> b . a
	let composed_endo: Endomorphism<RcFnBrand, i32> =
		append::<RcFnBrand, EndomorphismBrand<RcFnBrand, i32>>(endo_add_one)(endo_times_two);

	// (5 * 2) + 1 = 11
	assert_eq!(composed_endo.0(5), 11);

	// Test the identity element from the HKT `empty` function
	let identity_endo: Endomorphism<RcFnBrand, i32> = empty::<EndomorphismBrand<RcFnBrand, i32>>();
	assert_eq!(identity_endo.0(100), 100);
}

/// Generates a [`Kind` trait][crate::hkt::kinds] of a specific arity.
///
/// This macro creates traits that represent type-level applications for different kind arities.
/// Each generated trait has an `Output` associated type that represents the concrete type
/// produced when the brand is applied to the appropriate type parameters.
///
/// # Parameters
/// * `kind_trait_name`: Trait name (e.g., `Kind0L1T`).
/// * `lifetimes`: Tuple of lifetime parameters (e.g., `('a, 'b)`).
/// * `types`: Tuple of type parameters (e.g., `(A, B)`).
/// * `kind_signature`: Kind signature (e.g., `"* -> *"`).
#[macro_export]
macro_rules! make_trait_kind {
	(
		$kind_trait_name:ident,
		$lifetimes:tt,
		$types:tt,
		$kind_signature:literal
	) => {
		make_trait_kind!(
			@impl $kind_trait_name,
			$lifetimes,
			$types,
			$kind_signature
		);
	};

	(
		@impl $kind_trait_name:ident,
		(),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output;
		}
	};

	(
		@impl $kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output<$($lifetimes),*>;
		}
	};

	(
		@impl $kind_trait_name:ident,
		(),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output<$($types),*>;
		}
	};

	(
		@impl $kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub trait $kind_trait_name {
			type Output<$($lifetimes),*, $($types),*>;
		}
	};
}

/// Generates an [`Apply` type alias][crate::hkt::apply] of a specific arity.
///
/// This macro creates type aliases that simplify the usage of kind traits by providing
/// a more convenient syntax for type applications. These aliases are used throughout
/// the library to make type signatures more readable.
///
/// # Parameters
/// * `apply_alias_name`: Type alias name (e.g., `Apply0L1T`).
/// * `kind_trait_name`: Trait name (e.g., `Kind0L1T`).
/// * `lifetimes`: Tuple of lifetime parameters (e.g., `('a, 'b)`).
/// * `types`: Tuple of type parameters (e.g., `(A, B)`).
/// * `kind_signature`: Kind signature (e.g., `"* -> *"`).
#[macro_export]
macro_rules! make_type_apply {
	(
		$apply_alias_name:ident,
		$kind_trait_name:ident,
		$lifetimes:tt,
		$types:tt,
		$kind_signature:literal
	) => {
		make_type_apply!(
			@impl $apply_alias_name,
			$kind_trait_name,
			$lifetimes,
			$types,
			$kind_signature
		);
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		(),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<Brand> = <Brand as $kind_trait_name>::Output;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		(),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<$($lifetimes),*, Brand> = <Brand as $kind_trait_name>::Output<$($lifetimes),*>;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		(),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<Brand $(, $types)*> = <Brand as $kind_trait_name>::Output<$($types),*>;
	};

	(
		@impl $apply_alias_name:ident,
		$kind_trait_name:ident,
		($($lifetimes:lifetime),+),
		($($types:ident),+),
		$kind_signature:literal
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_signature,
			"`."
		)]
		pub type $apply_alias_name<$($lifetimes),*, Brand $(, $types)*> = <Brand as $kind_trait_name>::Output<$($lifetimes),* $(, $types)*>;
	};
}

make_trait_kind!(Kind0L1T, (), (A), "* -> *");
make_trait_kind!(
  Kind1L0T,
  ('a),
  (),
  "' -> *"
);

make_trait_kind!(
	Kind1L2T,
	('a),
	(A, B),
	"' -> * -> * -> *"
);

make_type_apply!(Apply0L1T, Kind0L1T, (), (A), "* -> *");
make_type_apply!(
  Apply1L0T,
  Kind1L0T,
  ('a),
  (),
  "' -> *"
);

make_type_apply!(
	Apply1L2T,
	Kind1L2T,
	('a),
	(A, B),
	"' -> * -> * -> *"
);

/// Takes functions `f` and `g` and returns the function `f . g` (`f` composed with `g`).
///
/// # Type Signature
///
/// `forall a b c. (b -> c) -> (a -> b) -> a -> c`
///
/// # Parameters
///
/// * `f`: A function from values of type `B` to values of type `C`.
/// * `g`: A function from values of type `A` to values of type `B`.
///
/// # Returns
///
/// A function from values of type `A` to values of type `C`.
///
/// # Examples
///
/// ```rust
/// use fp_library::{brands::RcFnBrand, functions::compose};
/// use std::rc::Rc;
///
/// let add_one = Rc::new(|x: i32| x + 1);
/// let times_two = Rc::new(|x: i32| x * 2);
/// let times_two_add_one = compose::<RcFnBrand, _, _, _>(add_one)(times_two);
///
/// // 3 * 2 + 1 = 7
/// assert_eq!(
///     times_two_add_one(3),
///     7
/// );
/// ```
pub fn compose<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a, C: 'a>(
	f: ApplyFn<'a, ClonableFnBrand, B, C>
) -> ApplyFn<
	'a,
	ClonableFnBrand,
	ApplyFn<'a, ClonableFnBrand, A, B>,
	ApplyFn<'a, ClonableFnBrand, A, C>,
> {
	ClonableFnBrand::new(move |g: ApplyFn<'a, ClonableFnBrand, _, _>| {
		let f = f.clone();
		ClonableFnBrand::new(move |a| f(g(a)))
	})
}

/// Returns its input.
///
/// # Type Signature
///
/// `forall a. a -> a`
///
/// # Parameters
///
/// * `a`: A value.
///
/// # Returns
///
/// The same value.
///
/// # Examples
///
/// ```rust
/// use fp_library::functions::identity;
///
/// assert_eq!(
///     identity(()),
///     ()
/// );
/// ```
pub fn identity<A>(a: A) -> A {
	a
}

/// Abstraction for clonable wrappers over closures.
///
/// This trait is implemented by "Brand" types (like [`ArcFnBrand`][crate::brands::ArcFnBrand]
/// and [`RcFnBrand`][crate::brands::RcFnBrand]) to provide a way to construct
/// and type-check clonable wrappers over closures (`Arc<dyn Fn...>` or
/// `Rc<dyn Fn...>`) in a generic context, allowing library users to choose
/// between implementations at function call sites.
///
/// The lifetime `'a` ensures the function doesn't outlive referenced data,
/// while generic types `A` and `B` represent the input and output types, respectively.
pub trait ClonableFn: Category {
	type Output<'a, A: 'a, B: 'a>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> ApplyFn<'a, Self, A, B>;
}

pub type ApplyFn<'a, Brand, A, B> = <Brand as ClonableFn>::Output<'a, A, B>;

/// A type class for semigroupoids.
///
/// A `Semigroupoid` is a set of objects and composable relationships
/// (morphisms) between them.
///
/// # Laws
///
/// Semigroupoid instances must satisfy the associative law:
/// * Associativity: `compose(p)(compose(q)(r)) = compose(compose(p)(q))(r)`.
///
/// # Examples
pub trait Semigroupoid: Kind1L2T {
	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// # Type Signature
	///
	/// `forall b c d. Semigroupoid a => a c d -> a b c -> a b d`
	///
	/// # Parameters
	///
	/// * `f`: A morphism of type `a c d`.
	/// * `g`: A morphism of type `a b c`.
	///
	/// # Returns
	///
	/// The morphism `f` composed with `g` of type `a b d`.
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B, C, D>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>>;
}

/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
///
/// Free function version that dispatches to [the type class' associated function][`Semigroupoid::compose`].
///
/// # Type Signature
///
/// `forall b c d. Semigroupoid a => a c d -> a b c -> a b d`
///
/// # Parameters
///
/// * `f`: A morphism of type `a c d`.
/// * `g`: A morphism of type `a b c`.
///
/// # Returns
///
/// The morphism `f` composed with `g` of type `a b d`.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::RcFnBrand, functions::compose};
/// use std::rc::Rc;
///
/// let add_one = Rc::new(|x: i32| x + 1);
/// let times_two = Rc::new(|x: i32| x * 2);
/// let times_two_add_one = compose::<RcFnBrand, RcFnBrand, _, _, _>(add_one)(times_two);
///
/// // 3 * 2 + 1 = 7
/// assert_eq!(times_two_add_one(3), 7);
/// ```
pub fn semigroupoid_compose<'a, ClonableFnBrand: 'a + ClonableFn, Brand: Semigroupoid, B, C, D>(
	f: Apply1L2T<'a, Brand, C, D>
) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Brand, B, C>, Apply1L2T<'a, Brand, B, D>> {
	Brand::compose::<'a, ClonableFnBrand, B, C, D>(f)
}

/// A type class for categories.
///
/// `Category` extends [`Semigroupoid`] with an identity element.
///
/// # Laws
///
/// `Category` instances must satisfy the identity law:
/// * Identity: `compose(identity)(p) = compose(p)(identity)`.
pub trait Category: Semigroupoid {
	/// Returns the identity morphism.
	///
	/// # Type Signature
	///
	/// `forall t. Category a => () -> a t t`
	///
	/// # Returns
	///
	/// The identity morphism.
	fn identity<'a, T: 'a>() -> Apply1L2T<'a, Self, T, T>;
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
///
/// # Type Signature
///
/// `forall t. Category a => () -> a t t`
///
/// # Returns
///
/// The identity morphism.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::RcFnBrand, functions::identity};
///
/// assert_eq!(identity::<RcFnBrand, _>()(()), ());
/// ```
pub fn category_identity<'a, Brand: Category, T: 'a>() -> Apply1L2T<'a, Brand, T, T> {
	Brand::identity::<'a, T>()
}

/// A type class for semigroups.
///
/// A `Semigroup` is a set equipped with an associative binary operation.
///
/// In functional programming, semigroups are useful for combining values
/// in a consistent way. They form the basis for more complex structures
/// like monoids.
///
/// # Laws
///
/// Semigroup instances must satisfy the associative law:
/// * Associativity: `append(append(a)(b))(c) = append(a)(append(b)(c))`.
pub trait Semigroup<'b> {
	/// Associative operation that combines two values of the same type.
	///
	/// # Type Signature
	///
	/// `Semigroup a => a -> a -> a`
	///
	/// # Parameters
	///
	/// * `a`: First value to combine.
	/// * `b`: Second value to combine.
	///
	/// # Returns
	///
	/// The result of combining the two values using the semigroup operation.
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a;
}

/// A higher-kinded Semigroup, abstracting over the lifetime parameter.
pub trait Semigroup1L0T: Kind1L0T
where
	for<'a> Apply1L0T<'a, Self>: Semigroup<'a>,
{
}

/// Associative operation that combines two values of the same type.
///
/// Free function version that dispatches to [the type class' associated function][`Semigroup::append`].
///
/// # Type Signature
///
/// `Semigroup a => a -> a -> a`
///
/// # Parameters
///
/// * `a`: First value to combine.
/// * `b`: Second value to combine.
///
/// # Returns
///
/// The result of combining the two values using the semigroup operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::RcFnBrand, functions::append};
///
/// assert_eq!(
///     append::<RcFnBrand, String>("Hello, ".to_string())("World!".to_string()),
///     "Hello, World!"
/// );
/// ```
pub fn append<'a, ClonableFnBrand: 'a + ClonableFn, HktBrand: Semigroup1L0T>(
	a: Apply1L0T<'a, HktBrand>
) -> ApplyFn<'a, ClonableFnBrand, Apply1L0T<'a, HktBrand>, Apply1L0T<'a, HktBrand>>
where
	for<'b> Apply1L0T<'b, HktBrand>: Semigroup<'b>,
{
	<Apply1L0T<'a, HktBrand> as Semigroup<'a>>::append::<ClonableFnBrand>(a)
}

/// A type class for monoids.
///
/// `Monoid` extends [`Semigroup`] with an identity element. A monoid is a set
/// equipped with an associative binary operation and an identity element.
///
/// In functional programming, monoids are useful for combining values in
/// a consistent way, especially when accumulating results or folding
/// collections.
///
/// # Laws
///
/// `Monoid` instances must satisfy the following laws:
/// * Left identity: `append(empty(), x) = x`.
/// * Right identity: `append(x, empty()) = x`.
/// * Associativity: `append(append(x, y), z) = append(x, append(y, z))`.
pub trait Monoid<'a>: Semigroup<'a> {
	/// Returns the identity element for the monoid.
	///
	/// # Type Signature
	///
	/// `Monoid a => () -> a`
	///
	/// # Returns
	///
	/// The identity element which, when combined with any other element
	/// using the semigroup operation, leaves the other element unchanged.
	fn empty() -> Self;
}

/// A higher-kinded Monoid, abstracting over the lifetime parameter.
pub trait Monoid1L0T: Semigroup1L0T
where
	for<'a> Apply1L0T<'a, Self>: Monoid<'a>,
{
}

/// Returns the identity element for the monoid.
///
/// Free function version that dispatches to [the type class' associated function][`Monoid::empty`].
///
/// # Type Signature
///
/// `Monoid a => () -> a`
///
/// # Returns
///
/// The identity element which, when combined with any other element
/// using the semigroup operation, leaves the other element unchanged.
///
/// # Examples
///
/// ```
/// use fp_library::functions::empty;
///
/// assert_eq!(empty::<String>(), "".to_string());
///
pub fn empty<'a, HktBrand>() -> Apply1L0T<'a, HktBrand>
where
	HktBrand: Monoid1L0T,
	for<'b> Apply1L0T<'b, HktBrand>: Monoid<'b>,
{
	<Apply1L0T<'a, HktBrand> as Monoid<'a>>::empty()
}

/// A brand type for reference-counted closures (Rc<dyn Fn(A) -> B>).
///
/// This struct implements [ClonableFn] to provide a way to construct and
/// type-check [Rc]-wrapped closures in a generic context. The lifetime 'a
/// ensures the closure doesn't outlive referenced data, while A and B
/// represent input and output types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;

impl Kind1L2T for RcFnBrand {
	type Output<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
}

impl ClonableFn for RcFnBrand {
	type Output<'a, A: 'a, B: 'a> = Apply1L2T<'a, Self, A, B>;

	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> ApplyFn<'a, Self, A, B> {
		Rc::new(f)
	}
}

impl Semigroupoid for RcFnBrand {
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B, C, D>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>> {
		ClonableFnBrand::new::<'a, _, _>(move |g: Apply1L2T<'a, Self, B, C>| {
			compose::<'a, Self, _, _, _>(f.clone())(g)
		})
	}
}

impl Category for RcFnBrand {
	fn identity<'a, T: 'a>() -> Apply1L2T<'a, Self, T, T> {
		Self::new::<'a, _, _>(identity)
	}
}

/// A wrapper for endomorphisms (morphisms from an object to itself) that enables monoidal operations.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [morphism composition][Semigroupoid::compose].
/// * The identity element [empty][Monoid::empty] is the [identity morphism][Category::identity].
pub struct Endomorphism<'a, CategoryBrand: Category, A: 'a>(pub Apply1L2T<'a, CategoryBrand, A, A>);

impl<'a, CategoryBrand, A> Clone for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
	fn clone(&self) -> Self {
		Endomorphism(self.0.clone())
	}
}

impl<'a, CategoryBrand, A> Debug for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Debug,
{
	fn fmt(
		&self,
		f: &mut Formatter<'_>,
	) -> fmt::Result {
		f.debug_tuple("Endomorphism").field(&self.0).finish()
	}
}

impl<'b, CategoryBrand, A> Semigroup<'b> for Endomorphism<'b, CategoryBrand, A>
where
	CategoryBrand: Category + 'b,
	A: 'b,
	Apply1L2T<'b, CategoryBrand, A, A>: Clone,
{
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		ClonableFnBrand::new(move |b: Self| {
			Endomorphism(semigroupoid_compose::<'b, ClonableFnBrand, CategoryBrand, _, _, _>(
				a.0.clone(),
			)(b.0))
		})
	}
}

impl<'a, CategoryBrand, A> Monoid<'a> for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
	fn empty() -> Self {
		Endomorphism(CategoryBrand::identity::<'a, _>())
	}
}

pub struct EndomorphismBrand<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);

impl<CategoryBrand, A> Kind1L0T for EndomorphismBrand<CategoryBrand, A>
where
	A: 'static,
	CategoryBrand: Category,
{
	type Output<'a> = Endomorphism<'a, CategoryBrand, A>;
}

impl<CategoryBrand, A> Semigroup1L0T for EndomorphismBrand<CategoryBrand, A>
where
	CategoryBrand: Category + 'static,
	A: 'static,
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
}

impl<CategoryBrand, A> Monoid1L0T for EndomorphismBrand<CategoryBrand, A>
where
	CategoryBrand: Category + 'static,
	A: 'static,
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
}

pub struct Endofunction<'a, ClonableFnBrand: ClonableFn, A: 'a>(ApplyFn<'a, ClonableFnBrand, A, A>);

impl<'a, ClonableFnBrand, A> Clone for Endofunction<'a, ClonableFnBrand, A>
where
	ClonableFnBrand: ClonableFn,
	A: 'a,
{
	fn clone(&self) -> Self {
		Endofunction(self.0.clone())
	}
}

impl<'b, ClonableFnBrand, A> Semigroup<'b> for Endofunction<'b, ClonableFnBrand, A>
where
	ClonableFnBrand: ClonableFn + 'b,
	A: 'b,
{
	fn append<'a, CFB: 'a + 'b + ClonableFn>(a: Self) -> ApplyFn<'a, CFB, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		CFB::new(move |b: Self| {
			Endofunction(compose::<'b, ClonableFnBrand, _, _, _>(a.0.clone())(b.0))
		})
	}
}

impl<'a, ClonableFnBrand, A> Monoid<'a> for Endofunction<'a, ClonableFnBrand, A>
where
	ClonableFnBrand: ClonableFn + 'a,
	A: 'a,
{
	fn empty() -> Self {
		Endofunction(ClonableFnBrand::new(identity))
	}
}

/// A type class for structures that can be folded to a single value.
///
/// A `Foldable` represents a structure that can be folded over to combine its elements
/// into a single result. This is useful for operations like summing values, collecting into a collection,
/// or applying monoidal operations.
///
/// A minimum implementation of `Foldable` requires the manual implementation of at least [`Foldable::fold_right`] or [`Foldable::fold_map`].
pub trait Foldable: Kind0L1T {
	/// Maps values to a monoid and combines them.
	///
	/// The default implementation of `fold_map` is implemented in terms of [`fold_right`], [`compose`], [`append`][crate::functions::append] and [`empty`][crate::functions::empty] where:
	///
	/// `fold_map f = (fold_right ((compose append) f)) empty`
	///
	/// # Type Signature
	///
	/// `forall a. Foldable f, Monoid m => (a -> m) -> f a -> m`
	///
	/// # Parameters
	///
	/// * `f`: A function that converts from values into monoidal elements.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final monoid obtained from the folding operation.
	fn fold_map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, M: Monoid<'a> + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, M>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, M> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, A>| {
			let append_ =
				ClonableFnBrand::new::<'a, M, _>(Semigroup::<'a>::append::<ClonableFnBrand>);
			let compose_append = compose::<'a, ClonableFnBrand, A, M, _>(append_);
			let compose_append_f: ApplyFn<
				'a,
				ClonableFnBrand,
				A,
				ApplyFn<'a, ClonableFnBrand, M, M>,
			> = compose_append(f.clone());
			let fold_right_compose_append_f =
				Self::fold_right::<'a, ClonableFnBrand, A, M>(compose_append_f);
			let fold_right_compose_append_f_empty: ApplyFn<
				'a,
				ClonableFnBrand,
				Apply0L1T<Self, A>,
				M,
			> = fold_right_compose_append_f(M::empty());
			fold_right_compose_append_f_empty(fa)
		})
	}

	/// Folds the structure by applying a function from right to left.
	///
	/// The default implementation of `fold_right` is implemented in terms of [`fold_map`] using the [`Endomorphism` monoid][`crate::types::Endomorphism`] where:
	///
	/// `((fold_right f) b) fa = ((fold_map f) fa) b`
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable f => (a -> b -> b) -> b -> f a -> b`
	///
	/// # Parameters
	///
	/// * `f`: A curried binary function that takes in the next item in the structure, the current value of the accumulator and returns the next value of accumulator.
	/// * `b`: Initial value of type `B`.
	/// * `fa`: A foldable structure containing values of type `A`.
	///
	/// # Returns
	///
	/// Final value of type `B` obtained from the folding operation.
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			let f = f.clone();
			ClonableFnBrand::new(move |fa: Apply0L1T<Self, A>| {
				(Self::fold_map::<'a, ClonableFnBrand, _, _>(ClonableFnBrand::new({
					let f = f.clone();
					move |a: A| Endofunction::<'a, ClonableFnBrand, _>(f(a))
				}))(fa)
				.0)(b.clone())
			})
		})
	}
}
