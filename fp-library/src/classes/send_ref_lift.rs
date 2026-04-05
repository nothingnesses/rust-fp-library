//! Thread-safe lifting of binary functions via references with [`send_ref_lift2`].
//!
//! Like [`RefLift::ref_lift2`](crate::classes::RefLift::ref_lift2), but the function
//! must be `Send` and element types must be `Send + Sync`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let x = ArcLazy::new(|| 3);
//! let y = ArcLazy::new(|| 4);
//! let z = send_ref_lift2::<LazyBrand<ArcLazyConfig>, _, _, _>(|a: &i32, b: &i32| *a + *b, x, y);
//! assert_eq!(*z.evaluate(), 7);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for lifting a binary function into a context using references,
	/// with `Send + Sync` bounds.
	///
	/// This is the thread-safe counterpart of [`RefLift`](crate::classes::RefLift).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefLift {
		/// Lifts a thread-safe binary function over two contexts using references.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters("The function to lift.", "The first context.", "The second context.")]
		///
		#[document_returns("A new context containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let x = ArcLazy::new(|| 3);
		/// let y = ArcLazy::new(|| 4);
		/// let z = LazyBrand::<ArcLazyConfig>::send_ref_lift2(|a: &i32, b: &i32| *a + *b, x, y);
		/// assert_eq!(*z.evaluate(), 7);
		/// ```
		fn send_ref_lift2<'a, A: Send + Sync + 'a, B: Send + Sync + 'a, C: Send + Sync + 'a>(
			func: impl Fn(&A, &B) -> C + Send + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>);
	}

	/// Lifts a thread-safe binary function over two contexts using references.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefLift::send_ref_lift2`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result."
	)]
	///
	#[document_parameters("The function to lift.", "The first context.", "The second context.")]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let x = ArcLazy::new(|| 3);
	/// let y = ArcLazy::new(|| 4);
	/// let z = send_ref_lift2::<LazyBrand<ArcLazyConfig>, _, _, _>(|a: &i32, b: &i32| *a + *b, x, y);
	/// assert_eq!(*z.evaluate(), 7);
	/// ```
	pub fn send_ref_lift2<
		'a,
		Brand: SendRefLift,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a,
		C: Send + Sync + 'a,
	>(
		func: impl Fn(&A, &B) -> C + Send + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		Brand::send_ref_lift2(func, fa, fb)
	}
}

pub use inner::*;
