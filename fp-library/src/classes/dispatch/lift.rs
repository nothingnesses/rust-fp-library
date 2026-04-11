//! Dispatch for [`Lift::lift2`](crate::classes::Lift::lift2) through
//! [`lift5`], and their by-reference counterparts
//! [`RefLift::ref_lift2`](crate::classes::RefLift::ref_lift2) etc.
//!
//! Provides `Lift2Dispatch` through `Lift5Dispatch` traits and unified
//! `lift2` through `lift5` free functions.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let z = lift2_explicit::<OptionBrand, _, _, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
//! assert_eq!(z, Some(3));
//!
//! let x = RcLazy::pure(3);
//! let y = RcLazy::pure(4);
//! let z = lift2_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _>(
//! 	|a: &i32, b: &i32| *a + *b,
//! 	&x,
//! 	&y,
//! );
//! assert_eq!(*z.evaluate(), 7);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Lift,
				RefLift,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- Lift2Dispatch --

	/// Trait that routes a lift2 operation to the appropriate type class method.
	///
	/// `Fn(A, B) -> C` resolves to [`Val`], `Fn(&A, &B) -> C` resolves to [`Ref`].
	/// The `FA` and `FB` type parameters are inferred from the container arguments:
	/// owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift2Dispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, FA, FB, Marker> {
		/// Perform the dispatched lift2 operation.
		#[document_signature]
		#[document_parameters("The first context.", "The second context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let z = lift2_explicit::<OptionBrand, _, _, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
		/// assert_eq!(z, Some(3));
		/// ```
		fn dispatch(
			self,
			fa: FA,
			fb: FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>);
	}

	/// Routes `Fn(A, B) -> C` closures to [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The first type.",
		"The second type.",
		"The result type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, F>
		Lift2Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			Val,
		> for F
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
		F: Fn(A, B) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters("The first context.", "The second context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let z = lift2_explicit::<OptionBrand, _, _, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
		/// assert_eq!(z, Some(3));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::lift2(self, fa, fb)
		}
	}

	/// Routes `Fn(&A, &B) -> C` closures to [`RefLift::ref_lift2`].
	///
	/// The containers must be passed by reference (`&fa`, `&fb`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The brand.",
		"The first type.",
		"The second type.",
		"The result type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, C, F>
		Lift2Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			Ref,
		> for F
	where
		Brand: RefLift,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(&A, &B) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters(
			"A reference to the first context.",
			"A reference to the second context."
		)]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let x = RcLazy::pure(3);
		/// let y = RcLazy::pure(4);
		/// let z = lift2_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32| *a + *b,
		/// 	&x,
		/// 	&y,
		/// );
		/// assert_eq!(*z.evaluate(), 7);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Brand::ref_lift2(self, fa, fb)
		}
	}

	/// Lifts a binary function into a functor context.
	///
	/// Dispatches to either [`Lift::lift2`] or [`RefLift::ref_lift2`]
	/// based on the closure's argument types.
	///
	/// The `Marker`, `FA`, and `FB` type parameters are inferred automatically
	/// by the compiler from the closure's argument types and the container
	/// arguments.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to lift.",
		"The first context (owned for Val, borrowed for Ref).",
		"The second context (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let z = lift2_explicit::<OptionBrand, _, _, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
	/// assert_eq!(z, Some(3));
	/// ```
	pub fn lift2<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, C: 'a, FA, FB, Marker>(
		f: impl Lift2Dispatch<'a, Brand, A, B, C, FA, FB, Marker>,
		fa: FA,
		fb: FB,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		f.dispatch(fa, fb)
	}

	// -- Lift3Dispatch --

	/// Trait that routes a lift3 operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First type.",
		"Second type.",
		"Third type.",
		"Result type.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"The third container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift3Dispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		FA,
		FB,
		FC,
		Marker,
	> {
		/// Perform the dispatched lift3 operation.
		#[document_signature]
		#[document_parameters("First context.", "Second context.", "Third context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift3_explicit::<OptionBrand, _, _, _, _, _, _, _, _>(
		/// 	|a, b, c| a + b + c,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// );
		/// assert_eq!(r, Some(6));
		/// ```
		fn dispatch(
			self,
			fa: FA,
			fb: FB,
			fc: FC,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>);
	}

	/// Routes `Fn(A, B, C) -> D` closures through [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, D, F>
		Lift3Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			Val,
		> for F
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: 'a,
		F: Fn(A, B, C) -> D + 'a,
	{
		#[document_signature]
		#[document_parameters("First context.", "Second context.", "Third context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift3_explicit::<OptionBrand, _, _, _, _, _, _, _, _>(
		/// 	|a, b, c| a + b + c,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// );
		/// assert_eq!(r, Some(6));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) {
			Brand::lift2(move |(a, b), c| self(a, b, c), Brand::lift2(|a, b| (a, b), fa, fb), fc)
		}
	}

	/// Routes `Fn(&A, &B, &C) -> D` closures through [`RefLift::ref_lift2`].
	///
	/// The containers must be passed by reference (`&fa`, `&fb`, `&fc`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, C, D, F>
		Lift3Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			Ref,
		> for F
	where
		Brand: RefLift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
		D: 'a,
		F: Fn(&A, &B, &C) -> D + 'a,
	{
		#[document_signature]
		#[document_parameters("First context.", "Second context.", "Third context.")]
		#[document_returns("A new context containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let a = RcLazy::pure(1);
		/// let b = RcLazy::pure(2);
		/// let c = RcLazy::pure(3);
		/// let r = lift3_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32, c: &i32| *a + *b + *c,
		/// 	&a,
		/// 	&b,
		/// 	&c,
		/// );
		/// assert_eq!(*r.evaluate(), 6);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) {
			Brand::ref_lift2(
				move |(a, b): &(A, B), c: &C| self(a, b, c),
				&Brand::ref_lift2(|a: &A, b: &B| (a.clone(), b.clone()), fa, fb),
				fc,
			)
		}
	}

	/// Lifts a ternary function into a functor context.
	///
	/// Dispatches to [`Lift::lift2`] or [`RefLift::ref_lift2`] based on the closure's argument types.
	///
	/// When dispatched through the Ref path (`Fn(&A, &B, &C) -> D`), the intermediate
	/// types `A` and `B` must implement [`Clone`] because the implementation builds
	/// the ternary lift from nested binary `ref_lift2` calls, which requires
	/// constructing intermediate tuples.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First type.",
		"Second type.",
		"Third type.",
		"Result type.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"The third container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker."
	)]
	#[document_parameters(
		"The function to lift.",
		"First context (owned for Val, borrowed for Ref).",
		"Second context (owned for Val, borrowed for Ref).",
		"Third context (owned for Val, borrowed for Ref)."
	)]
	#[document_returns("A new context containing the result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let r = lift3_explicit::<OptionBrand, _, _, _, _, _, _, _, _>(
	/// 	|a, b, c| a + b + c,
	/// 	Some(1),
	/// 	Some(2),
	/// 	Some(3),
	/// );
	/// assert_eq!(r, Some(6));
	/// ```
	pub fn lift3<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		FA,
		FB,
		FC,
		Marker,
	>(
		f: impl Lift3Dispatch<'a, Brand, A, B, C, D, FA, FB, FC, Marker>,
		fa: FA,
		fb: FB,
		fc: FC,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) {
		f.dispatch(fa, fb, fc)
	}

	// -- Lift4Dispatch --

	/// Trait that routes a lift4 operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"The third container type (owned or borrowed), inferred from the argument.",
		"The fourth container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift4Dispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		E: 'a,
		FA,
		FB,
		FC,
		FD,
		Marker,
	> {
		/// Perform the dispatched lift4 operation.
		#[document_signature]
		#[document_parameters("First.", "Second.", "Third.", "Fourth.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift4_explicit::<OptionBrand, _, _, _, _, _, _, _, _, _, _>(
		/// 	|a, b, c, d| a + b + c + d,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// );
		/// assert_eq!(r, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
			fb: FB,
			fc: FC,
			fd: FD,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>);
	}

	/// Routes `Fn(A, B, C, D) -> E` closures through [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, D, E, Func>
		Lift4Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			E,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			Val,
		> for Func
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: 'a,
		Func: Fn(A, B, C, D) -> E + 'a,
	{
		#[document_signature]
		#[document_parameters("First.", "Second.", "Third.", "Fourth.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift4_explicit::<OptionBrand, _, _, _, _, _, _, _, _, _, _>(
		/// 	|a, b, c, d| a + b + c + d,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// );
		/// assert_eq!(r, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
			Brand::lift2(
				move |((a, b), c), d| self(a, b, c, d),
				Brand::lift2(move |(a, b), c| ((a, b), c), Brand::lift2(|a, b| (a, b), fa, fb), fc),
				fd,
			)
		}
	}

	/// Routes `Fn(&A, &B, &C, &D) -> E` closures through [`RefLift::ref_lift2`].
	///
	/// The containers must be passed by reference.
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, C, D, E, Func>
		Lift4Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			E,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			Ref,
		> for Func
	where
		Brand: RefLift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: 'a,
		E: 'a,
		Func: Fn(&A, &B, &C, &D) -> E + 'a,
	{
		#[document_signature]
		#[document_parameters("First.", "Second.", "Third.", "Fourth.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let a = RcLazy::pure(1);
		/// let b = RcLazy::pure(2);
		/// let c = RcLazy::pure(3);
		/// let d = RcLazy::pure(4);
		/// let r = lift4_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32, c: &i32, d: &i32| *a + *b + *c + *d,
		/// 	&a,
		/// 	&b,
		/// 	&c,
		/// 	&d,
		/// );
		/// assert_eq!(*r.evaluate(), 10);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
			Brand::ref_lift2(
				move |((a, b), c): &((A, B), C), d: &D| self(a, b, c, d),
				&Brand::ref_lift2(
					move |(a, b): &(A, B), c: &C| ((a.clone(), b.clone()), c.clone()),
					&Brand::ref_lift2(|a: &A, b: &B| (a.clone(), b.clone()), fa, fb),
					fc,
				),
				fd,
			)
		}
	}

	/// Lifts a quaternary function into a functor context.
	///
	/// When dispatched through the Ref path (`Fn(&A, &B, &C, &D) -> E`), the
	/// intermediate types `A`, `B`, and `C` must implement [`Clone`] because
	/// the implementation builds the quaternary lift from nested binary
	/// `ref_lift2` calls, which requires constructing intermediate tuples.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"First.",
		"Second.",
		"Third.",
		"Fourth.",
		"Result.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"The third container type (owned or borrowed), inferred from the argument.",
		"The fourth container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker."
	)]
	#[document_parameters(
		"The function to lift.",
		"First (owned for Val, borrowed for Ref).",
		"Second (owned for Val, borrowed for Ref).",
		"Third (owned for Val, borrowed for Ref).",
		"Fourth (owned for Val, borrowed for Ref)."
	)]
	#[document_returns("Result context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let r = lift4_explicit::<OptionBrand, _, _, _, _, _, _, _, _, _, _>(
	/// 	|a, b, c, d| a + b + c + d,
	/// 	Some(1),
	/// 	Some(2),
	/// 	Some(3),
	/// 	Some(4),
	/// );
	/// assert_eq!(r, Some(10));
	/// ```
	pub fn lift4<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		E: 'a,
		FA,
		FB,
		FC,
		FD,
		Marker,
	>(
		f: impl Lift4Dispatch<'a, Brand, A, B, C, D, E, FA, FB, FC, FD, Marker>,
		fa: FA,
		fb: FB,
		fc: FC,
		fd: FD,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
		f.dispatch(fa, fb, fc, fd)
	}

	// -- Lift5Dispatch --

	/// Trait that routes a lift5 operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"The third container type (owned or borrowed), inferred from the argument.",
		"The fourth container type (owned or borrowed), inferred from the argument.",
		"The fifth container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait Lift5Dispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		E: 'a,
		G: 'a,
		FA,
		FB,
		FC,
		FD,
		FE,
		Marker,
	> {
		/// Perform the dispatched lift5 operation.
		#[document_signature]
		#[document_parameters("1st.", "2nd.", "3rd.", "4th.", "5th.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift5_explicit::<OptionBrand, _, _, _, _, _, _, _, _, _, _, _, _>(
		/// 	|a, b, c, d, e| a + b + c + d + e,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// 	Some(5),
		/// );
		/// assert_eq!(r, Some(15));
		/// ```
		fn dispatch(
			self,
			fa: FA,
			fb: FB,
			fc: FC,
			fd: FD,
			fe: FE,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>);
	}

	/// Routes `Fn(A, B, C, D, E) -> G` closures through [`Lift::lift2`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, D, E, G, Func>
		Lift5Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			E,
			G,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Val,
		> for Func
	where
		Brand: Lift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: Clone + 'a,
		G: 'a,
		Func: Fn(A, B, C, D, E) -> G + 'a,
	{
		#[document_signature]
		#[document_parameters("1st.", "2nd.", "3rd.", "4th.", "5th.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let r = lift5_explicit::<OptionBrand, _, _, _, _, _, _, _, _, _, _, _, _>(
		/// 	|a, b, c, d, e| a + b + c + d + e,
		/// 	Some(1),
		/// 	Some(2),
		/// 	Some(3),
		/// 	Some(4),
		/// 	Some(5),
		/// );
		/// assert_eq!(r, Some(15));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			fe: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>) {
			Brand::lift2(
				move |(((a, b), c), d), e| self(a, b, c, d, e),
				Brand::lift2(
					move |((a, b), c), d| (((a, b), c), d),
					Brand::lift2(
						move |(a, b), c| ((a, b), c),
						Brand::lift2(|a, b| (a, b), fa, fb),
						fc,
					),
					fd,
				),
				fe,
			)
		}
	}

	/// Routes `Fn(&A, &B, &C, &D, &E) -> G` closures through [`RefLift::ref_lift2`].
	///
	/// The containers must be passed by reference.
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"Closure."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, C, D, E, G, Func>
		Lift5Dispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			E,
			G,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Ref,
		> for Func
	where
		Brand: RefLift,
		A: Clone + 'a,
		B: Clone + 'a,
		C: Clone + 'a,
		D: Clone + 'a,
		E: 'a,
		G: 'a,
		Func: Fn(&A, &B, &C, &D, &E) -> G + 'a,
	{
		#[document_signature]
		#[document_parameters("1st.", "2nd.", "3rd.", "4th.", "5th.")]
		#[document_returns("Result context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let a = RcLazy::pure(1);
		/// let b = RcLazy::pure(2);
		/// let c = RcLazy::pure(3);
		/// let d = RcLazy::pure(4);
		/// let e = RcLazy::pure(5);
		/// let r = lift5_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32, c: &i32, d: &i32, e: &i32| *a + *b + *c + *d + *e,
		/// 	&a,
		/// 	&b,
		/// 	&c,
		/// 	&d,
		/// 	&e,
		/// );
		/// assert_eq!(*r.evaluate(), 15);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			fc: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>),
			fd: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>),
			fe: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>) {
			Brand::ref_lift2(
				move |(((a, b), c), d): &(((A, B), C), D), e: &E| self(a, b, c, d, e),
				&Brand::ref_lift2(
					move |((a, b), c): &((A, B), C), d: &D| {
						(((a.clone(), b.clone()), c.clone()), d.clone())
					},
					&Brand::ref_lift2(
						move |(a, b): &(A, B), c: &C| ((a.clone(), b.clone()), c.clone()),
						&Brand::ref_lift2(|a: &A, b: &B| (a.clone(), b.clone()), fa, fb),
						fc,
					),
					fd,
				),
				fe,
			)
		}
	}

	/// Lifts a quinary function into a functor context.
	///
	/// When dispatched through the Ref path (`Fn(&A, &B, &C, &D, &E) -> G`),
	/// the intermediate types `A`, `B`, `C`, and `D` must implement [`Clone`]
	/// because the implementation builds the quinary lift from nested binary
	/// `ref_lift2` calls, which requires constructing intermediate tuples.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"1st.",
		"2nd.",
		"3rd.",
		"4th.",
		"5th.",
		"Result.",
		"The first container type (owned or borrowed), inferred from the argument.",
		"The second container type (owned or borrowed), inferred from the argument.",
		"The third container type (owned or borrowed), inferred from the argument.",
		"The fourth container type (owned or borrowed), inferred from the argument.",
		"The fifth container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker."
	)]
	#[document_parameters(
		"The function to lift.",
		"1st (owned for Val, borrowed for Ref).",
		"2nd (owned for Val, borrowed for Ref).",
		"3rd (owned for Val, borrowed for Ref).",
		"4th (owned for Val, borrowed for Ref).",
		"5th (owned for Val, borrowed for Ref)."
	)]
	#[document_returns("Result context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let r = lift5_explicit::<OptionBrand, _, _, _, _, _, _, _, _, _, _, _, _>(
	/// 	|a, b, c, d, e| a + b + c + d + e,
	/// 	Some(1),
	/// 	Some(2),
	/// 	Some(3),
	/// 	Some(4),
	/// 	Some(5),
	/// );
	/// assert_eq!(r, Some(15));
	/// ```
	pub fn lift5<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		E: 'a,
		G: 'a,
		FA,
		FB,
		FC,
		FD,
		FE,
		Marker,
	>(
		f: impl Lift5Dispatch<'a, Brand, A, B, C, D, E, G, FA, FB, FC, FD, FE, Marker>,
		fa: FA,
		fb: FB,
		fc: FC,
		fd: FD,
		fe: FE,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>) {
		f.dispatch(fa, fb, fc, fd, fe)
	}
}

pub use inner::*;
