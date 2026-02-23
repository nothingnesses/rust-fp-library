//! Grate optics for operating on structures through exponentiation.
//!
//! A grate represents a way to operate on a structure by providing a way
//! to construct it from values extracted from functions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				CloneableFn,
				Closed,
				UnsizedCoercible,
			},
			kinds::*,
			types::optics::{
				GrateOptic,
				Optic,
				SetterOptic,
			},
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	/// A polymorphic grate.
	///
	/// Matches PureScript's `Grate s t a b`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub struct Grate<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Grating function.
		pub grate: <FnBrand<P> as CloneableFn>::Of<
			'a,
			<FnBrand<P> as CloneableFn>::Of<'a, <FnBrand<P> as CloneableFn>::Of<'a, S, A>, B>,
			T,
		>,
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
	impl<'a, P, S, T, A, B> Grate<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		/// Creates a new `Grate` instance.
		#[document_signature]
		///
		#[document_parameters("The grating function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::CloneableFn,
		/// 	types::optics::Grate,
		/// };
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	// f is now wrapped in RcFnBrand, so we need to call it.
		/// 	// f: Rc<dyn Fn(Rc<dyn Fn(S) -> A>) -> B>
		/// 	let get_x = <RcFnBrand as CloneableFn>::new(|(x, _)| x);
		/// 	let get_y = <RcFnBrand as CloneableFn>::new(|(_, y)| y);
		/// 	(f(get_x), f(get_y))
		/// });
		/// ```
		pub fn new(
			grate: impl Fn(
				<FnBrand<P> as CloneableFn>::Of<'a, <FnBrand<P> as CloneableFn>::Of<'a, S, A>, B>,
			) -> T
			+ 'a
		) -> Self {
			Grate {
				grate: <FnBrand<P> as CloneableFn>::new(grate),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, Q, P, S, T, A, B> Optic<'a, Q, S, T, A, B> for Grate<'a, P, S, T, A, B>
	where
		Q: Closed,
		P: UnsizedCoercible,
		S: 'a + Clone,
		T: 'a,
		A: 'a + Clone,
		B: 'a,
	{
		/// Evaluates the grate with a profunctor.
		#[document_signature]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	Kind,
		/// 	brands::*,
		/// 	classes::{
		/// 		CloneableFn,
		/// 		Closed,
		/// 	},
		/// 	types::optics::{
		/// 		Grate,
		/// 		Optic,
		/// 	},
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f: Rc<dyn Fn(Apply!(<RcFnBrand as Kind!(type Of<'b, U: 'b, V: 'b>: 'b;)>::Of<'_, (i32, i32), i32>)) -> i32>| {
		/// 	let get_x = <RcFnBrand as CloneableFn>::new(|(x, _)| x);
		/// 	let get_y = <RcFnBrand as CloneableFn>::new(|(_, y)| y);
		/// 	(f(get_x), f(get_y))
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = Optic::<RcFnBrand, _, _, _, _>::evaluate(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let grate = self.grate.clone();

			Q::dimap(
				move |s: S| {
					let s_inner = s.clone();
					Box::new(move |f: <FnBrand<P> as CloneableFn>::Of<'a, S, A>| -> A {
						// f is FnBrand<P>::Of<S, A>.
						(f)(s_inner.clone())
					}) as Box<dyn Fn(<FnBrand<P> as CloneableFn>::Of<'a, S, A>) -> A + 'a>
				},
				move |f: Box<dyn Fn(<FnBrand<P> as CloneableFn>::Of<'a, S, A>) -> B + 'a>| {
					let f_brand = <FnBrand<P> as CloneableFn>::new(move |x| f(x));
					grate(f_brand)
				},
				Q::closed(pab),
			)
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
	#[document_parameters("The grate instance.")]
	impl<'a, P, S, T, A, B> GrateOptic<'a, S, T, A, B> for Grate<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		S: 'a + Clone,
		A: 'a + Clone,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Closed,
		/// 	types::optics::{
		/// 		Grate,
		/// 		GrateOptic,
		/// 	},
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	(f(Rc::new(|(x, _)| x) as Rc<dyn Fn((i32, i32)) -> i32>), f(Rc::new(|(_, y)| y) as Rc<dyn Fn((i32, i32)) -> i32>))
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = GrateOptic::evaluate::<RcFnBrand>(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate<Q: Closed>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the grate.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, Q, P, S, T, A, B> SetterOptic<'a, Q, S, T, A, B> for Grate<'a, P, S, T, A, B>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
		S: 'a + Clone,
		A: 'a + Clone,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Closed,
		/// 	types::optics::{
		/// 		Grate,
		/// 		SetterOptic,
		/// 	},
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	(f(Rc::new(|(x, _)| x) as Rc<dyn Fn((i32, i32)) -> i32>), f(Rc::new(|(_, y)| y) as Rc<dyn Fn((i32, i32)) -> i32>))
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = SetterOptic::evaluate(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			GrateOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}

	/// A monomorphic grate.
	///
	/// Matches PureScript's `Grate' s a`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct GratePrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		pub(crate) grate_fn: <FnBrand<P> as CloneableFn>::Of<
			'a,
			<FnBrand<P> as CloneableFn>::Of<'a, <FnBrand<P> as CloneableFn>::Of<'a, S, A>, A>,
			S,
		>,
		pub(crate) _phantom: PhantomData<P>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, P, S, A> Clone for GratePrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	Kind,
		/// 	brands::*,
		/// 	classes::CloneableFn,
		/// 	types::optics::GratePrime,
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(|f: Rc<dyn Fn(Apply!(<RcFnBrand as Kind!(type Of<'b, U: 'b, V: 'b>: 'b;)>::Of<'_, (i32, i32), i32>)) -> i32>| {
		/// 	(f(<RcFnBrand as CloneableFn>::new(|(x, _)| x)), f(<RcFnBrand as CloneableFn>::new(|(_, y)| y)))
		/// });
		/// let cloned = grate.clone();
		/// ```
		fn clone(&self) -> Self {
			GratePrime {
				grate_fn: self.grate_fn.clone(),
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
	impl<'a, P, S, A> GratePrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		/// Creates a new `GratePrime` instance.
		#[document_signature]
		///
		#[document_parameters("The grating function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	Kind,
		/// 	brands::*,
		/// 	classes::CloneableFn,
		/// 	types::optics::GratePrime,
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(|f: Rc<dyn Fn(Apply!(<RcFnBrand as Kind!(type Of<'b, U: 'b, V: 'b>: 'b;)>::Of<'_, (i32, i32), i32>)) -> i32>| {
		/// 	(f(<RcFnBrand as CloneableFn>::new(|(x, _)| x)), f(<RcFnBrand as CloneableFn>::new(|(_, y)| y)))
		/// });
		/// ```
		pub fn new(
			grate: impl Fn(
				<FnBrand<P> as CloneableFn>::Of<'a, <FnBrand<P> as CloneableFn>::Of<'a, S, A>, A>,
			) -> S
			+ 'a
		) -> Self {
			GratePrime {
				grate_fn: <FnBrand<P> as CloneableFn>::new(grate),
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, Q, P, S, A> Optic<'a, Q, S, S, A, A> for GratePrime<'a, P, S, A>
	where
		Q: Closed,
		P: UnsizedCoercible,
		S: 'a + Clone,
		A: 'a + Clone,
	{
		/// Evaluates the grate with a profunctor.
		#[document_signature]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	Kind,
		/// 	brands::*,
		/// 	classes::{
		/// 		CloneableFn,
		/// 		Closed,
		/// 	},
		/// 	types::optics::{
		/// 		GratePrime,
		/// 		Optic,
		/// 	},
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(|f: Rc<dyn Fn(Apply!(<RcFnBrand as Kind!(type Of<'b, U: 'b, V: 'b>: 'b;)>::Of<'_, (i32, i32), i32>)) -> i32>| {
		/// 	(f(<RcFnBrand as CloneableFn>::new(|(x, _)| x)), f(<RcFnBrand as CloneableFn>::new(|(_, y)| y)))
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = Optic::<RcFnBrand, _, _, _, _>::evaluate(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let grate = self.grate_fn.clone();

			Q::dimap(
				move |s: S| {
					let s_inner = s.clone();
					Box::new(move |f: <FnBrand<P> as CloneableFn>::Of<'a, S, A>| {
						(f)(s_inner.clone())
					}) as Box<dyn Fn(<FnBrand<P> as CloneableFn>::Of<'a, S, A>) -> A + 'a>
				},
				move |f: Box<dyn Fn(<FnBrand<P> as CloneableFn>::Of<'a, S, A>) -> A + 'a>| {
					let f_brand = <FnBrand<P> as CloneableFn>::new(move |x| f(x));
					grate(f_brand)
				},
				Q::closed(pab),
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, P, S, A> GrateOptic<'a, S, S, A, A> for GratePrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		S: 'a + Clone,
		A: 'a + Clone,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	Kind,
		/// 	brands::*,
		/// 	classes::{
		/// 		CloneableFn,
		/// 		Closed,
		/// 	},
		/// 	types::optics::{
		/// 		GrateOptic,
		/// 		GratePrime,
		/// 	},
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(|f: Rc<dyn Fn(Apply!(<RcFnBrand as Kind!(type Of<'b, U: 'b, V: 'b>: 'b;)>::Of<'_, (i32, i32), i32>)) -> i32>| {
		/// 	(f(<RcFnBrand as CloneableFn>::new(|(x, _)| x)), f(<RcFnBrand as CloneableFn>::new(|(_, y)| y)))
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = GrateOptic::evaluate::<RcFnBrand>(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate<Q: Closed>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the grate.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, Q, P, S, A> SetterOptic<'a, Q, S, S, A, A> for GratePrime<'a, P, S, A>
	where
		P: UnsizedCoercible,
		Q: UnsizedCoercible,
		S: 'a + Clone,
		A: 'a + Clone,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Closed,
		/// 	types::optics::{
		/// 		GratePrime,
		/// 		SetterOptic,
		/// 	},
		/// };
		/// use std::rc::Rc;
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(|f| {
		/// 	(f(Rc::new(|(x, _)| x) as Rc<dyn Fn((i32, i32)) -> i32>), f(Rc::new(|(_, y)| y) as Rc<dyn Fn((i32, i32)) -> i32>))
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = SetterOptic::evaluate(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			GrateOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}
}
pub use inner::*;
