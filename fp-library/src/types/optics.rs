//! Optics for composable data accessors using profunctor encoding.
//!
//! This module provides a trait-based profunctor optic implementation that is a high-fidelity
//! port of PureScript's `purescript-profunctor-lenses`. It allows composing lenses, prisms,
//! and other optics while maintaining type safety and zero-cost abstractions through monomorphization.
//!
//! ### Comparison with PureScript
//!
//! The implementation mirrors the PureScript `Optic` definition closely:
//!
//! | Feature | PureScript | Rust (`fp-library`) |
//! | :--- | :--- | :--- |
//! | **Optic Definition** | `p a b -> p s t` | `trait Optic<S, T, A, B>` |
//! | **Lens** | `Strong p => Optic p s t a b` | `struct Lens<P, S, T, A, B>` |
//! | **Lens'** | `Lens s s a a` | `struct LensPrime<P, S, A>` |
//! | **Composition** | `Semigroupoid` / `<<<` | `struct Composed` / `optics_compose` |
//!
//! While PureScript uses the `Semigroupoid` instance of functions for composition,
//! this library uses a specialized `Composed` struct. This allows Rust to perform
//! zero-cost composition through monomorphization while preserving the `Optic` trait
//! boundaries without needing the rank-2 polymorphism that PureScript relies on.
//! Lenses in this library use [`FnBrand`](crate::brands::FnBrand) to support
//! capturing closures and reference-counted storage.
//!
//! ### Lifetime Support
//!
//! The optics hierarchy has been updated to include a lifetime parameter `'a`. This allows
//! optics to work with non-static types (e.g., types containing references like `&str`) by
//! ensuring that the captured functions and the types they operate on are valid for the
//! same lifetime.
//!
//! This change was necessary because of the unification of profunctor and arrow hierarchies on
//! [`Kind_266801a817966495`], which requires that type arguments to a Kind outlive the
//! lifetime argument (`type Of<'a, T: 'a, U: 'a>: 'a;`).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::optics::*,
//! };
//!
//! // Define a simple struct
//! #[derive(Clone, Debug, PartialEq)]
//! struct Person {
//! 	name: String,
//! 	age: i32,
//! }
//!
//! // Create a lens for the age field
//! let age_lens: LensPrime<RcBrand, Person, i32> = LensPrime::new(
//! 	|p: Person| p.age,
//! 	|(p, age)| Person {
//! 		age,
//! 		..p
//! 	},
//! );
//!
//! let person = Person {
//! 	name: "Alice".to_string(),
//! 	age: 30,
//! };
//! let age = age_lens.view(person.clone());
//! assert_eq!(age, 30);
//!
//! let updated = age_lens.set(person.clone(), 31);
//! assert_eq!(updated.age, 31);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{Choice, CloneableFn, Strong, UnsizedCoercible},
			kinds::*,
		},
		fp_macros::{document_parameters, document_signature, document_type_parameters},
		std::marker::PhantomData,
	};

	/// A trait for optics that can be evaluated with any profunctor constraint.
	///
	/// This trait allows optics to be first-class values that can be composed
	/// and stored while preserving their polymorphism over profunctor types.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub trait Optic<'a, S: 'a, T: 'a, A: 'a, B: 'a> {
		/// Evaluate the optic with a profunctor.
		///
		/// This method applies the optic transformation to a profunctor value.
		#[document_signature]
		///
		#[document_type_parameters("The profunctor type.")]
		///
		#[document_parameters("The profunctor value to transform.")]
		fn evaluate<P: Strong + Choice>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
	}

	/// Composition of two optics.
	///
	/// This struct represents the composition of two optics, allowing them to be
	/// combined into a single optic that applies both transformations.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct Composed<'a, S, T, M, N, A, B, O1, O2> {
		/// The outer optic (applied second).
		pub first: O1,
		/// The inner optic (applied first).
		pub second: O2,
		pub(crate) _phantom: PhantomData<&'a (S, T, M, N, A, B)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	impl<'a, S, T, M, N, A, B, O1, O2> Composed<'a, S, T, M, N, A, B, O1, O2> {
		/// Create a new composed optic.
		#[document_signature]
		///
		#[document_parameters(
			"The outer optic (applied second).",
			"The inner optic (applied first)."
		)]
		pub fn new(
			first: O1,
			second: O2,
		) -> Self {
			Composed { first, second, _phantom: PhantomData }
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	#[document_parameters("The composed optic instance.")]
	impl<'a, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2> Optic<'a, S, T, A, B> for Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: Optic<'a, S, T, M, N>,
		O2: Optic<'a, M, N, A, B>,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		fn evaluate<P: Strong + Choice>(
			&self,
			pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let pmn = self.second.evaluate::<P>(pab);
			self.first.evaluate::<P>(pmn)
		}
	}

	/// Compose two optics into a single optic.
	///
	/// While PureScript uses the `Semigroupoid` operator (`<<<`) for composition because
	/// its optics are functions, this library uses a specialized `Composed` struct.
	/// This is necessary because Rust represents the polymorphic profunctor constraint
	/// as a trait method (`Optic::evaluate<P>`), and the `Composed` struct enables
	/// static dispatch and zero-cost composition through monomorphization.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The source type of the outer structure.",
		"The target type of the outer structure.",
		"The source type of the intermediate structure.",
		"The target type of the intermediate structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The first optic.",
		"The second optic."
	)]
	///
	#[document_parameters("The outer optic (applied second).", "The inner optic (applied first).")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::optics::*,
	/// };
	///
	/// #[derive(Clone, Debug, PartialEq)]
	/// struct Address {
	/// 	street: String,
	/// }
	/// #[derive(Clone, Debug, PartialEq)]
	/// struct User {
	/// 	address: Address,
	/// }
	///
	/// let address_lens: LensPrime<RcBrand, User, Address> = LensPrime::new(
	/// 	|u: User| u.address.clone(),
	/// 	|(_, a)| User {
	/// 		address: a,
	/// 	},
	/// );
	/// let street_lens: LensPrime<RcBrand, Address, String> = LensPrime::new(
	/// 	|a: Address| a.street.clone(),
	/// 	|(_, s)| Address {
	/// 		street: s,
	/// 	},
	/// );
	///
	/// let user_street = optics_compose(address_lens, street_lens);
	/// let user = User {
	/// 	address: Address {
	/// 		street: "High St".to_string(),
	/// 	},
	/// };
	///
	/// // Composed optics are evaluated through a profunctor instance (e.g., RcFnBrand).
	/// // This lifts a function on the focus (A -> B) to a function on the structure (S -> T).
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|s: String| s.to_uppercase());
	/// let modifier = user_street.evaluate::<RcFnBrand>(f);
	/// let updated = modifier(user);
	///
	/// assert_eq!(updated.address.street, "HIGH ST");
	/// ```
	pub fn optics_compose<'a, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2>(
		first: O1,
		second: O2,
	) -> Composed<'a, S, T, M, N, A, B, O1, O2>
	where
		O1: Optic<'a, S, T, M, N>,
		O2: Optic<'a, M, N, A, B>,
	{
		Composed::new(first, second)
	}

	/// A polymorphic lens for accessing and updating a field where types can change.
	/// This matches PureScript's `Lens s t a b`.
	///
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub struct Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		/// Getter function.
		pub view: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		/// Setter function.
		pub set: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, B), T>),
		pub(crate) _phantom: PhantomData<P>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
	{
		/// Create a new polymorphic lens.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, String, i32, String> = Lens::new(|x| x, |(_, s)| s);
		/// ```
		pub fn new(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, B)) -> T,
		) -> Self {
			Lens {
				view: <FnBrand<P> as CloneableFn>::new(view),
				set: <FnBrand<P> as CloneableFn>::new(set),
				_phantom: PhantomData,
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, i32, i32, i32> = Lens::new(|x| x, |(_, y)| y);
		/// assert_eq!(l.view(10), 10);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.view)(s)
		}

		/// Set the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new value for the focus.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Lens,
		/// };
		///
		/// let l: Lens<RcBrand, i32, i32, i32, i32> = Lens::new(|x| x, |(_, y)| y);
		/// assert_eq!(l.set(10, 20), 20);
		/// ```
		pub fn set(
			&self,
			s: S,
			b: B,
		) -> T {
			(self.set)((s, b))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, P, S, T, A, B> Optic<'a, S, T, A, B> for Lens<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a + Clone,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		fn evaluate<Q: Strong + Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
		{
			let view = self.view.clone();
			let set = self.set.clone();

			Q::dimap(
				move |s: S| (view(s.clone()), s),
				move |(b, s): (B, S)| set((s, b)),
				Q::first(pab),
			)
		}
	}

	/// A concrete lens type for accessing and updating a field in a structure where types do not change.
	/// This matches PureScript's `Lens' s a`.
	///
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		pub(crate) view_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		pub(crate) set_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, (S, A), S>),
		pub(crate) _phantom: PhantomData<P>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The lens instance.")]
	impl<'a, P, S, A> Clone for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		fn clone(&self) -> Self {
			LensPrime {
				view_fn: self.view_fn.clone(),
				set_fn: self.set_fn.clone(),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, P, S: 'a, A: 'a> LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
	{
		/// Create a new monomorphic lens from a getter and setter.
		#[document_signature]
		///
		#[document_parameters("The getter function.", "The setter function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// ```
		pub fn new(
			view: impl 'a + Fn(S) -> A,
			set: impl 'a + Fn((S, A)) -> S,
		) -> Self {
			LensPrime {
				view_fn: <FnBrand<P> as CloneableFn>::new(view),
				set_fn: <FnBrand<P> as CloneableFn>::new(set),
				_phantom: PhantomData,
			}
		}

		/// View the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to view.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.view(42), 42);
		/// ```
		pub fn view(
			&self,
			s: S,
		) -> A {
			(self.view_fn)(s)
		}

		/// Set the focus of the lens in a structure.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The new value for the focus.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.set(10, 20), 20);
		/// ```
		pub fn set(
			&self,
			s: S,
			a: A,
		) -> S {
			(self.set_fn)((s, a))
		}

		/// Update the focus of the lens in a structure using a function.
		#[document_signature]
		///
		#[document_parameters("The structure to update.", "The function to apply to the focus.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::LensPrime,
		/// };
		///
		/// let l: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, y)| y);
		/// assert_eq!(l.over(10, |x| x + 1), 11);
		/// ```
		pub fn over(
			&self,
			s: S,
			f: impl Fn(A) -> A,
		) -> S
		where
			S: Clone,
		{
			let a = self.view(s.clone());
			self.set(s, f(a))
		}
	}

	// Optic implementation for LensPrime<P, S, A>
	// Note: This implements monomorphic update (S -> S, A -> A)
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic lens instance.")]
	impl<'a, P, S, A> Optic<'a, S, S, A, A> for LensPrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a + Clone,
		A: 'a,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		fn evaluate<Q: Strong + Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			let view_fn = self.view_fn.clone();
			let set_fn = self.set_fn.clone();

			// The Profunctor encoding of a Lens is:
			// lens get set = dimap (\s -> (get s, s)) (\(b, s) -> set s b) . first
			Q::dimap(
				move |s: S| (view_fn(s.clone()), s),
				move |(a, s): (A, S)| set_fn((s, a)),
				Q::first(pab),
			)
		}
	}
}

pub use inner::*;
