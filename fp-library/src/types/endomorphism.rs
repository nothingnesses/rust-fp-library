//! Wrapper for endomorphisms (morphisms `c a a` in a category) with [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances based on categorical composition.
//!
//! A more general form of `Endofunction` that works with any [`Category`](crate::classes::Category), not just functions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				Category,
				Monoid,
				Semigroup,
			},
			kinds::*,
		},
		fp_macros::{
			document_examples,
			document_fields,
			document_parameters,
			document_returns,
			document_type_parameters,
		},
		std::{
			fmt::{
				self,
				Debug,
				Formatter,
			},
			hash::Hash,
		},
	};

	/// A wrapper for endomorphisms (morphisms from an object to the same object) that enables monoidal operations.
	///
	/// `Endomorphism c a` represents a morphism `c a a` where `c` is a `Category`.
	/// For the category of functions, this represents functions of type `a -> a`.
	///
	/// It exists to provide a monoid instance where:
	///
	/// * The binary operation [append][Semigroup::append] is [morphism composition][crate::classes::semigroupoid::Semigroupoid::compose].
	/// * The identity element [empty][Monoid::empty] is the [identity morphism][Category::identity].
	///
	/// The wrapped morphism can be accessed directly via the [`.0` field][Endomorphism#structfield.0].
	///
	/// ### Hierarchy Unification
	///
	/// `Endomorphism` now requires that its object type `A` outlive the lifetime `'a` of the
	/// endomorphism itself (`A: 'a`). This is necessary to satisfy the requirements of the
	/// unified [`Kind_266801a817966495`] used by the [`Category`] hierarchy.
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	///
	#[document_fields("The wrapped morphism.")]
	///
	pub struct Endomorphism<'a, C: Category, A: 'a>(
		pub Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>),
	);

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	impl<'a, C: Category, A: 'a> Endomorphism<'a, C, A> {
		/// Creates a new `Endomorphism`.
		///
		/// This function wraps a morphism `c a a` in an `Endomorphism` struct.
		#[document_signature]
		///
		#[document_parameters("The morphism to wrap.")]
		///
		#[document_returns("A new `Endomorphism`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(f.0(5), 10);
		/// ```
		pub fn new(
			f: Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>)
		) -> Self {
			Self(f)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	#[document_parameters("The morphism to clone.")]
	impl<'a, C: Category, A: 'a> Clone for Endomorphism<'a, C, A>
	where
		Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>): Clone,
	{
		#[document_signature]
		#[document_returns("A new `Endomorphism` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let cloned = f.clone();
		/// assert_eq!(cloned.0(5), 10);
		/// ```
		fn clone(&self) -> Self {
			Self::new(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	#[document_parameters("The morphism to format.")]
	impl<'a, C: Category, A: 'a> Debug for Endomorphism<'a, C, A>
	where
		Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>): Debug,
	{
		#[document_signature]
		#[document_parameters("The formatter to use.")]
		#[document_returns("The result of the formatting operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // Debug formatting is available when the inner function type implements Debug.
		/// // Verify the endomorphism applies correctly:
		/// assert_eq!(f.0(5), 10);
		/// ```
		fn fmt(
			&self,
			fmt: &mut Formatter<'_>,
		) -> fmt::Result {
			fmt.debug_tuple("Endomorphism").field(&self.0).finish()
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	impl<'a, C: Category, A: 'a> Eq for Endomorphism<'a, C, A> where
		Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>): Eq
	{
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	#[document_parameters("The morphism to hash.")]
	impl<'a, C: Category, A: 'a> Hash for Endomorphism<'a, C, A>
	where
		Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>): Hash,
	{
		#[document_signature]
		#[document_type_parameters("The type of the hasher.")]
		#[document_parameters("The hasher state to update.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // Hash is available when the inner function type implements Hash.
		/// // Verify the endomorphism applies correctly:
		/// assert_eq!(f.0(5), 10);
		/// ```
		fn hash<H: std::hash::Hasher>(
			&self,
			state: &mut H,
		) {
			self.0.hash(state);
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	#[document_parameters("The morphism to compare.")]
	impl<'a, C: Category, A: 'a> Ord for Endomorphism<'a, C, A>
	where
		Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>): Ord,
	{
		#[document_signature]
		#[document_parameters("The other morphism to compare to.")]
		#[document_returns("The ordering of the values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // Ord is available when the inner function type implements Ord.
		/// // Both produce the same output for the same input:
		/// assert_eq!(f.0(5), g.0(5));
		/// ```
		fn cmp(
			&self,
			other: &Self,
		) -> std::cmp::Ordering {
			self.0.cmp(&other.0)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	#[document_parameters("The morphism to compare.")]
	impl<'a, C: Category, A: 'a> PartialEq for Endomorphism<'a, C, A>
	where
		Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>): PartialEq,
	{
		#[document_signature]
		#[document_parameters("The other morphism to compare to.")]
		#[document_returns("True if the values are equal, false otherwise.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // PartialEq is available when the inner function type implements PartialEq.
		/// // Both produce the same output for the same input:
		/// assert_eq!(f.0(5), g.0(5));
		/// ```
		fn eq(
			&self,
			other: &Self,
		) -> bool {
			self.0 == other.0
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	#[document_parameters("The morphism to compare.")]
	impl<'a, C: Category, A: 'a> PartialOrd for Endomorphism<'a, C, A>
	where
		Apply!(<C as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>): PartialOrd,
	{
		#[document_signature]
		#[document_parameters("The other morphism to compare to.")]
		#[document_returns("An ordering if the values can be compared, none otherwise.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// // PartialOrd is available when the inner function type implements PartialOrd.
		/// // Both produce the same output for the same input:
		/// assert_eq!(f.0(5), g.0(5));
		/// ```
		fn partial_cmp(
			&self,
			other: &Self,
		) -> Option<std::cmp::Ordering> {
			self.0.partial_cmp(&other.0)
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	impl<'a, C: Category, A: 'a> Semigroup for Endomorphism<'a, C, A> {
		/// The result of combining the two values using the semigroup operation.
		///
		/// This method composes two endomorphisms into a single endomorphism using the underlying category's composition.
		/// Note that `Endomorphism` composition is reversed relative to standard function composition:
		/// `append(f, g)` results in `f . g` (read as "f after g"), meaning `g` is applied first, then `f`.
		#[document_signature]
		///
		#[document_parameters(
			"The second morphism to apply (the outer function).",
			"The first morphism to apply (the inner function)."
		)]
		///
		#[document_returns("The composed morphism `a . b`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let g = Endomorphism::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
		///
		/// // f(g(x)) = (x + 1) * 2
		/// let h = append::<_>(f, g);
		/// assert_eq!(h.0(5), 12);
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Self::new(C::compose(a.0, b.0))
		}
	}

	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The category of the morphism.",
		"The object of the morphism."
	)]
	impl<'a, C: Category, A: 'a> Monoid for Endomorphism<'a, C, A> {
		/// The identity element.
		///
		/// This method returns the identity endomorphism, which wraps the identity morphism of the underlying category.
		#[document_signature]
		///
		#[document_returns("The identity endomorphism.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let id = empty::<Endomorphism<RcFnBrand, i32>>();
		/// assert_eq!(id.0(5), 5);
		/// ```
		fn empty() -> Self {
			Self::new(C::identity())
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::RcFnBrand,
			classes::{
				cloneable_fn::CloneableFn,
				monoid::empty,
				semigroup::append,
			},
		},
		quickcheck_macros::quickcheck,
	};

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(val: i32) -> bool {
		let f = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let g = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_mul(2)
		}));
		let h = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_sub(3)
		}));

		let lhs = append(f.clone(), append(g.clone(), h.clone()));
		let rhs = append(append(f, g), h);

		lhs.0(val) == rhs.0(val)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(val: i32) -> bool {
		let f = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endomorphism<RcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(val) == f.0(val)
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endomorphism<RcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(val) == f.0(val)
	}
}
