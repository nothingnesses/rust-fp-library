//! Optics for composable data accessors using profunctor encoding.
//!
//! This module provides a trait-based profunctor optic implementation that allows
//! composing lenses, prisms, and other optics while maintaining type safety and
//! zero-cost abstractions through monomorphization.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{types::optics::*, brands::*, functions::*};
//!
//! // Define a simple struct
//! #[derive(Clone, Debug, PartialEq)]
//! struct Person {
//!     name: String,
//!     age: i32,
//! }
//!
//! // Create a lens for the age field
//! let age_lens = Lens::new(
//!     |p: &Person| p.age,
//!     |p: Person, age: i32| Person { age, ..p }
//! );
//!
//! let person = Person { name: "Alice".to_string(), age: 30 };
//! let age = age_lens.view(&person);
//! assert_eq!(age, 30);
//!
//! let updated = age_lens.set(person.clone(), 31);
//! assert_eq!(updated.age, 31);
//! ```

use crate::{Apply, classes::{Strong, Choice}, kinds::*};

/// A trait for optics that can be run with any profunctor constraint.
///
/// This trait allows optics to be first-class values that can be composed
/// and stored while preserving their polymorphism over profunctor types.
pub trait Optic<S, T, A, B> {
	/// Run the optic with a profunctor.
	///
	/// This method applies the optic transformation to a profunctor value.
	fn run<'a, P: Strong>(
		&self,
		pab: Apply!(<P as Kind!( type Of<'a, U, V>; )>::Of<'a, A, B>),
	) -> Apply!(<P as Kind!( type Of<'a, U, V>; )>::Of<'a, S, T>)
	where
		A: 'a,
		B: 'a,
		S: 'a,
		T: 'a;
}

/// Composition of two optics.
///
/// This struct represents the composition of two optics, allowing them to be
/// combined into a single optic that applies both transformations.
pub struct Composed<S, T, M, N, A, B, O1, O2>
where
	O1: Optic<S, T, M, N>,
	O2: Optic<M, N, A, B>,
{
	/// The outer optic (applied second).
	pub first: O1,
	/// The inner optic (applied first).
	pub second: O2,
	_phantom: std::marker::PhantomData<(S, T, M, N, A, B)>,
}

impl<S, T, M, N, A, B, O1, O2> Composed<S, T, M, N, A, B, O1, O2>
where
	O1: Optic<S, T, M, N>,
	O2: Optic<M, N, A, B>,
{
	/// Create a new composed optic.
	pub fn new(first: O1, second: O2) -> Self {
		Composed {
			first,
			second,
			_phantom: std::marker::PhantomData,
		}
	}
}

impl<S, T, M, N, A, B, O1, O2> Optic<S, T, A, B> for Composed<S, T, M, N, A, B, O1, O2>
where
	O1: Optic<S, T, M, N>,
	O2: Optic<M, N, A, B>,
{
	fn run<'a, P: Strong>(
		&self,
		pab: Apply!(<P as Kind!( type Of<'a, U, V>; )>::Of<'a, A, B>),
	) -> Apply!(<P as Kind!( type Of<'a, U, V>; )>::Of<'a, S, T>)
	where
		A: 'a,
		B: 'a,
		S: 'a,
		T: 'a,
		M: 'a,
		N: 'a,
	{
		self.first.run(self.second.run(pab))
	}
}

/// A concrete lens type for accessing and updating a field in a structure.
///
/// A lens focuses on a single value within a data structure, allowing you to
/// view (get) and update (set) that value while preserving the rest of the structure.
#[derive(Clone)]
pub struct Lens<S, A> {
	view_fn: fn(&S) -> A,
	set_fn: fn(S, A) -> S,
}

impl<S, A> Lens<S, A> {
	/// Create a new lens from view and set functions.
	///
	/// ### Parameters
	///
	/// - `view`: A function that extracts the focused value from the structure.
	/// - `set`: A function that updates the structure with a new focused value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::*;
	///
	/// #[derive(Clone)]
	/// struct Point { x: i32, y: i32 }
	///
	/// let x_lens = Lens::new(
	///     |p: &Point| p.x,
	///     |p: Point, x: i32| Point { x, ..p }
	/// );
	///
	/// let point = Point { x: 10, y: 20 };
	/// assert_eq!(x_lens.view(&point), 10);
	///
	/// let updated = x_lens.set(point, 15);
	/// assert_eq!(updated.x, 15);
	/// assert_eq!(updated.y, 20);
	/// ```
	pub fn new(view: fn(&S) -> A, set: fn(S, A) -> S) -> Self {
		Lens {
			view_fn: view,
			set_fn: set,
		}
	}

	/// View the focused value.
	///
	/// Extracts the value that this lens focuses on from the structure.
	pub fn view(&self, s: &S) -> A {
		(self.view_fn)(s)
	}

	/// Set the focused value.
	///
	/// Updates the structure with a new value for the focused field.
	pub fn set(&self, s: S, a: A) -> S {
		(self.set_fn)(s, a)
	}

	/// Modify the focused value with a function.
	///
	/// Applies a function to the focused value and updates the structure.
	pub fn over(&self, s: S, f: impl Fn(A) -> A) -> S
	where
		A: Clone,
	{
		let a = self.view(&s);
		self.set(s, f(a))
	}
}

/// A concrete prism type for accessing and constructing a variant in a sum type.
///
/// A prism focuses on one variant of a sum type (like `Result` or `Option`),
/// allowing you to preview (try to extract) that variant and review (construct) it.
#[derive(Clone)]
pub struct Prism<S, A> {
	preview_fn: fn(S) -> Option<A>,
	review_fn: fn(A) -> S,
}

impl<S, A> Prism<S, A> {
	/// Create a new prism from preview and review functions.
	///
	/// ### Parameters
	///
	/// - `preview`: A function that attempts to extract the focused value from the structure.
	/// - `review`: A function that constructs the structure from the focused value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::*;
	///
	/// let ok_prism: Prism<Result<i32, String>, i32> = Prism::new(
	///     |r: Result<i32, String>| r.ok(),
	///     |x: i32| Ok(x)
	/// );
	///
	/// assert_eq!(ok_prism.preview(Ok(42)), Some(42));
	/// assert_eq!(ok_prism.preview(Err("error".to_string())), None);
	/// assert_eq!(ok_prism.review(42), Ok(42));
	/// ```
	pub fn new(preview: fn(S) -> Option<A>, review: fn(A) -> S) -> Self {
		Prism {
			preview_fn: preview,
			review_fn: review,
		}
	}

	/// Preview the focused value.
	///
	/// Attempts to extract the value if this variant is present.
	pub fn preview(&self, s: S) -> Option<A> {
		(self.preview_fn)(s)
	}

	/// Review (construct) from the focused value.
	///
	/// Constructs the structure from the focused value.
	pub fn review(&self, a: A) -> S {
		(self.review_fn)(a)
	}
}

/// Compose two optics into a single optic.
///
/// This function creates a `Composed` optic that applies the second optic
/// first, then applies the first optic to the result.
///
/// ### Examples
///
/// ```
/// use fp_library::types::optics::*;
///
/// #[derive(Clone)]
/// struct Address { street: String, number: i32 }
///
/// #[derive(Clone)]
/// struct Person { name: String, address: Address }
///
/// let address_lens = Lens::new(
///     |p: &Person| p.address.clone(),
///     |p: Person, a: Address| Person { address: a, ..p }
/// );
///
/// let number_lens = Lens::new(
///     |a: &Address| a.number,
///     |a: Address, n: i32| Address { number: n, ..a }
/// );
///
/// let person = Person {
///     name: "Alice".to_string(),
///     address: Address { street: "Main St".to_string(), number: 42 }
/// };
///
/// // Compose lenses to access nested field
/// let composed = compose_optics(address_lens, number_lens);
/// ```
pub fn compose_optics<S, T, M, N, A, B, O1, O2>(
	first: O1,
	second: O2,
) -> Composed<S, T, M, N, A, B, O1, O2>
where
	O1: Optic<S, T, M, N>,
	O2: Optic<M, N, A, B>,
{
	Composed::new(first, second)
}
