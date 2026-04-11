//! Profunctors, which are functors contravariant in the first argument and covariant in the second.
//!
//! A profunctor represents a morphism between two categories, mapping objects and morphisms from one to the other.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Arrow is a profunctor
//! let f = |x: i32| x + 1;
//! let g = dimap::<RcFnBrand, _, _, _, _>(
//! 	|x: i32| x * 2,
//! 	|x: i32| x - 1,
//! 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
//! );
//! assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
//! ```

pub use {
	choice::*,
	closed::*,
	cochoice::*,
	costrong::*,
	strong::*,
	wander::*,
};

pub mod choice;
pub mod closed;
pub mod cochoice;
pub mod costrong;
pub mod strong;
pub mod wander;

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::*,
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for profunctors.
	///
	/// A profunctor is a type constructor that is contravariant in its first type parameter
	/// and covariant in its second type parameter. This means it can pre-compose with a
	/// function on the input and post-compose with a function on the output.
	///
	/// ### Hierarchy Unification
	///
	/// This trait is now the root of the unified profunctor and arrow hierarchies on
	/// [`Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)`](crate::kinds::Kind_266801a817966495).
	/// This unification ensures that all profunctor-based abstractions
	/// (including lenses and prisms) share a consistent higher-kinded representation with
	/// strict lifetime bounds.
	///
	/// By explicitly requiring that both type parameters outlive the application lifetime `'a`,
	/// we provide the compiler with the necessary guarantees to handle trait objects
	/// (like `dyn Fn`) commonly used in profunctor implementations. This resolves potential
	/// E0310 errors where the compiler cannot otherwise prove that captured variables in
	/// closures satisfy the required lifetime bounds.
	///
	/// ### Laws
	///
	/// `Profunctor` instances must satisfy the following laws:
	/// * Identity: `dimap(identity, identity, p) = p`.
	/// * Composition: `dimap(f2 ∘ f1, g1 ∘ g2, p) = dimap(f1, g1, dimap(f2, g2, p))`.
	#[document_examples]
	///
	/// Profunctor laws for [`RcFnBrand`](crate::brands::RcFnBrand):
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let p = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
	///
	/// // Identity: dimap(identity, identity, p) = p
	/// let id_mapped = dimap::<RcFnBrand, _, _, _, _>(identity, identity, p.clone());
	/// assert_eq!(id_mapped(5), p(5));
	/// assert_eq!(id_mapped(0), p(0));
	///
	/// // Composition: dimap(f2 ∘ f1, g1 ∘ g2, p)
	/// //            = dimap(f1, g1, dimap(f2, g2, p))
	/// let f1 = |x: i32| x + 10;
	/// let f2 = |x: i32| x * 2;
	/// let g1 = |x: i32| x - 1;
	/// let g2 = |x: i32| x * 3;
	/// let left = dimap::<RcFnBrand, _, _, _, _>(
	/// 	compose(f2, f1), // f2 ∘ f1
	/// 	compose(g1, g2), // g1 ∘ g2
	/// 	p.clone(),
	/// );
	/// let right = dimap::<RcFnBrand, _, _, _, _>(f1, g1, dimap::<RcFnBrand, _, _, _, _>(f2, g2, p));
	/// assert_eq!(left(5), right(5));
	/// assert_eq!(left(0), right(0));
	/// ```
	#[kind(type Of<'a, A: 'a, B: 'a>: 'a;)]
	pub trait Profunctor {
		/// Maps over both arguments of the profunctor.
		///
		/// This method applies a contravariant function to the first argument and a covariant
		/// function to the second argument, transforming the profunctor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The new input type (contravariant position).",
			"The original input type.",
			"The original output type.",
			"The new output type (covariant position)."
		)]
		///
		#[document_parameters(
			"The contravariant function to apply to the input.",
			"The covariant function to apply to the output.",
			"The profunctor instance."
		)]
		///
		#[document_returns("A new profunctor instance with transformed input and output types.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let f = |x: i32| x + 1;
		/// let g = dimap::<RcFnBrand, _, _, _, _>(
		/// 	|x: i32| x * 2,
		/// 	|x: i32| x - 1,
		/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
		/// );
		/// assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
		/// ```
		fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			ab: impl Fn(A) -> B + 'a,
			cd: impl Fn(C) -> D + 'a,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>);

		/// Maps contravariantly over the first argument.
		///
		/// This is a convenience method that maps only over the input (contravariant position).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The new input type.",
			"The original input type.",
			"The output type."
		)]
		///
		#[document_parameters(
			"The contravariant function to apply to the input.",
			"The profunctor instance."
		)]
		///
		#[document_returns("A new profunctor instance with transformed input type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let f = |x: i32| x + 1;
		/// let g = lmap::<RcFnBrand, _, _, _>(
		/// 	|x: i32| x * 2,
		/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
		/// );
		/// assert_eq!(g(10), 21); // (10 * 2) + 1 = 21
		/// ```
		fn lmap<'a, A: 'a, B: 'a, C: 'a>(
			ab: impl Fn(A) -> B + 'a,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, C>) {
			Self::dimap(ab, crate::functions::identity, pbc)
		}

		/// Maps covariantly over the second argument.
		///
		/// This is a convenience method that maps only over the output (covariant position).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type.",
			"The original output type.",
			"The new output type."
		)]
		///
		#[document_parameters(
			"The covariant function to apply to the output.",
			"The profunctor instance."
		)]
		///
		#[document_returns("A new profunctor instance with transformed output type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::profunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let f = |x: i32| x + 1;
		/// let g = rmap::<RcFnBrand, _, _, _>(
		/// 	|x: i32| x * 2,
		/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
		/// );
		/// assert_eq!(g(10), 22); // (10 + 1) * 2 = 22
		/// ```
		fn rmap<'a, A: 'a, B: 'a, C: 'a>(
			bc: impl Fn(B) -> C + 'a,
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, C>) {
			Self::dimap(crate::functions::identity, bc, pab)
		}
	}

	/// Maps over both arguments of the profunctor.
	///
	/// Free function version that dispatches to [the type class' associated function][`Profunctor::dimap`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the profunctor.",
		"The new input type (contravariant position).",
		"The original input type.",
		"The original output type.",
		"The new output type (covariant position)."
	)]
	///
	#[document_parameters(
		"The contravariant function to apply to the input.",
		"The covariant function to apply to the output.",
		"The profunctor instance."
	)]
	///
	#[document_returns("A new profunctor instance with transformed input and output types.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = dimap::<RcFnBrand, _, _, _, _>(
	/// 	|x: i32| x * 2,
	/// 	|x: i32| x - 1,
	/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
	/// );
	/// assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
	/// ```
	pub fn dimap<'a, Brand: Profunctor, A: 'a, B: 'a, C: 'a, D: 'a>(
		ab: impl Fn(A) -> B + 'a,
		cd: impl Fn(C) -> D + 'a,
		pbc: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>) {
		Brand::dimap(ab, cd, pbc)
	}

	/// Maps contravariantly over the first argument.
	///
	/// Free function version that dispatches to [the type class' associated function][`Profunctor::lmap`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the profunctor.",
		"The new input type.",
		"The original input type.",
		"The output type."
	)]
	///
	#[document_parameters(
		"The contravariant function to apply to the input.",
		"The profunctor instance."
	)]
	///
	#[document_returns("A new profunctor instance with transformed input type.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = lmap::<RcFnBrand, _, _, _>(
	/// 	|x: i32| x * 2,
	/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
	/// );
	/// assert_eq!(g(10), 21); // (10 * 2) + 1 = 21
	/// ```
	pub fn lmap<'a, Brand: Profunctor, A: 'a, B: 'a, C: 'a>(
		ab: impl Fn(A) -> B + 'a,
		pbc: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, C>) {
		Brand::lmap(ab, pbc)
	}

	/// Maps covariantly over the second argument.
	///
	/// Free function version that dispatches to [the type class' associated function][`Profunctor::rmap`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the profunctor.",
		"The input type.",
		"The original output type.",
		"The new output type."
	)]
	///
	#[document_parameters(
		"The covariant function to apply to the output.",
		"The profunctor instance."
	)]
	///
	#[document_returns("A new profunctor instance with transformed output type.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = rmap::<RcFnBrand, _, _, _>(
	/// 	|x: i32| x * 2,
	/// 	std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>,
	/// );
	/// assert_eq!(g(10), 22); // (10 + 1) * 2 = 22
	/// ```
	pub fn rmap<'a, Brand: Profunctor, A: 'a, B: 'a, C: 'a>(
		bc: impl Fn(B) -> C + 'a,
		pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, C>) {
		Brand::rmap(bc, pab)
	}

	/// Lifts a pure function into a profunctor context.
	///
	/// Given a type that is both a [`Category`] (providing `identity`) and a
	/// [`Profunctor`] (providing `rmap`), this function lifts a pure function
	/// `A -> B` into the profunctor as `rmap(f, identity())`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the profunctor.",
		"The input type.",
		"The output type."
	)]
	///
	#[document_parameters("The closure to lift.")]
	///
	#[document_returns("The lifted profunctor value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	pub fn arrow<'a, Brand, A, B: 'a>(
		f: impl 'a + Fn(A) -> B
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	where
		Brand: Category + Profunctor, {
		Brand::rmap(f, Brand::identity())
	}

	crate::impl_kind! {
		impl<Brand: Profunctor, A: 'static> for ProfunctorFirstAppliedBrand<Brand, A> {
			type Of<'a, B: 'a>: 'a = Apply!(<Brand as Kind!(type Of<'a, T: 'a, U: 'a>: 'a;)>::Of<'a, A, B>);
		}
	}

	/// [`Functor`] instance for [`ProfunctorFirstAppliedBrand`].
	///
	/// Maps over the second (covariant) type parameter of a profunctor via [`Profunctor::rmap`].
	#[document_type_parameters("The profunctor brand.", "The fixed first type parameter.")]
	impl<Brand: Profunctor, A: 'static> Functor for ProfunctorFirstAppliedBrand<Brand, A> {
		/// Map a function over the covariant type parameter.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The function to apply.", "The profunctor value to map over.")]
		#[document_returns("The mapped profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g =
		/// 	map_explicit::<ProfunctorFirstAppliedBrand<RcFnBrand, i32>, _, _, _, _>(|x: i32| x * 2, f);
		/// assert_eq!(g(5), 12); // (5 + 1) * 2
		/// ```
		fn map<'a, B: 'a, C: 'a>(
			f: impl Fn(B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::rmap(f, fa)
		}
	}

	impl_kind! {
		impl<Brand: Profunctor, B: 'static> for ProfunctorSecondAppliedBrand<Brand, B> {
			type Of<'a, A: 'a>: 'a = Apply!(<Brand as Kind!(type Of<'a, T: 'a, U: 'a>: 'a;)>::Of<'a, A, B>);
		}
	}

	/// [`Contravariant`] instance for [`ProfunctorSecondAppliedBrand`].
	///
	/// Contramaps over the first (contravariant) type parameter of a profunctor via [`Profunctor::lmap`].
	#[document_type_parameters("The profunctor brand.", "The fixed second type parameter.")]
	impl<Brand: Profunctor, B: 'static> Contravariant for ProfunctorSecondAppliedBrand<Brand, B> {
		/// Contramap a function over the contravariant type parameter.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters("The function to apply.", "The profunctor value to contramap over.")]
		#[document_returns("The contramapped profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g =
		/// 	contramap_explicit::<ProfunctorSecondAppliedBrand<RcFnBrand, i32>, _, _>(|x: i32| x * 2, f);
		/// assert_eq!(g(5), 11); // (5 * 2) + 1
		/// ```
		fn contramap<'a, A: 'a, C: 'a>(
			f: impl Fn(C) -> A + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::lmap(f, fa)
		}
	}
}

pub use inner::*;
