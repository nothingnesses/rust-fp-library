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
				RefCountedPointer,
				UnsizedCoercible,
				optics::*,
				profunctor::Closed,
			},
			kinds::*,
			types::optics::zip_with_of,
		},
		fp_macros::*,
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
	pub struct Grate<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Grating function.
		pub grate: <FnBrand<PointerBrand> as CloneableFn>::Of<
			'a,
			<FnBrand<PointerBrand> as CloneableFn>::Of<
				'a,
				<FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
					A,
				>,
				B,
			>,
			T,
		>,
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
	impl<'a, PointerBrand, S, T, A, B> Clone for Grate<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Grate` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::CloneableFn,
		/// 		types::optics::Grate,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	let get_x = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0);
		/// 	let get_y = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1);
		/// 	(f(get_x), f(get_y))
		/// });
		/// let cloned = grate.clone();
		/// assert_eq!(cloned.zip_with(|(a, b)| a + b, (1, 2), (3, 4)), (4, 6));
		/// ```
		fn clone(&self) -> Self {
			Grate {
				grate: self.grate.clone(),
			}
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
	impl<'a, PointerBrand, S, T, A, B> Grate<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a + Clone,
	{
		/// Creates a new `Grate` instance.
		#[document_signature]
		///
		#[document_parameters("The grating function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::CloneableFn,
		/// 		types::optics::Grate,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	let get_x = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0);
		/// 	let get_y = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1);
		/// 	(f(get_x), f(get_y))
		/// });
		/// assert_eq!(grate.zip_with(|(a, b)| a + b, (1, 2), (3, 4)), (4, 6));
		/// ```
		pub fn new(
			grate: impl Fn(
				<FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<FnBrand<PointerBrand> as CloneableFn>::Of<
						'a,
						<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
						A,
					>,
					B,
				>,
			) -> T
			+ 'a
		) -> Self
		where
			<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized, {
			Grate {
				grate: <FnBrand<PointerBrand> as CloneableFn>::new(grate),
			}
		}

		/// Zip two structures together using this grate and a combining function.
		///
		/// Convenience method wrapping [`zip_with_of`].
		#[document_signature]
		///
		#[document_parameters(
			"The combining function, taking a pair `(A, A)` and returning `B`.",
			"The first structure.",
			"The second structure."
		)]
		///
		#[document_returns("The combined structure.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::CloneableFn,
		/// 		types::optics::Grate,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	let get_x = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0);
		/// 	let get_y = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1);
		/// 	(f(get_x), f(get_y))
		/// });
		/// let result = grate.zip_with(|(a, b)| a + b, (1, 2), (10, 20));
		/// assert_eq!(result, (11, 22));
		/// ```
		pub fn zip_with(
			&self,
			f: impl Fn((A, A)) -> B + 'a,
			s1: S,
			s2: S,
		) -> T
		where
			<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized, {
			zip_with_of::<FnBrand<PointerBrand>, _, S, T, A, B>(self, f, s1, s2)
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
	impl<'a, Q, PointerBrand, S, T, A, B> Optic<'a, Q, S, T, A, B>
		for Grate<'a, PointerBrand, S, T, A, B>
	where
		Q: Closed<FnBrand<PointerBrand>>,
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a + Clone,
		<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized,
	{
		/// Evaluates the grate with a profunctor.
		#[document_signature]
		///
		#[document_parameters("The profunctor value to transform.")]
		///
		#[document_returns("The transformed profunctor value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::{
		/// 			CloneableFn,
		/// 			optics::*,
		/// 		},
		/// 		types::optics::Grate,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(
		/// 	|f: Rc<dyn Fn(Rc<dyn Fn(Rc<(i32, i32)>) -> i32>) -> i32>| {
		/// 		let get_x = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0);
		/// 		let get_y = <RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1);
		/// 		(f(get_x), f(get_y))
		/// 	},
		/// );
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
					let s_ptr = <PointerBrand as RefCountedPointer>::cloneable_new(s);
					<FnBrand<PointerBrand> as CloneableFn>::new(
						move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
							'a,
							<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
							A,
						>|
						      -> A { (f)(Clone::clone(&s_ptr)) },
					)
				},
				move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<FnBrand<PointerBrand> as CloneableFn>::Of<
						'a,
						<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
						A,
					>,
					B,
				>| {
					let f_brand = <FnBrand<PointerBrand> as CloneableFn>::new(move |x| f(x));
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
	impl<'a, PointerBrand, S, T, A, B> GrateOptic<'a, FnBrand<PointerBrand>, S, T, A, B>
		for Grate<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
		B: 'a + Clone,
		<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::{
		/// 			CloneableFn,
		/// 			optics::*,
		/// 			profunctor::*,
		/// 		},
		/// 		types::optics::Grate,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	(
		/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.0) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
		/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.1) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
		/// 	)
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = GrateOptic::<RcFnBrand, _, _, _, _>::evaluate::<RcFnBrand>(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate<Q: Closed<FnBrand<PointerBrand>>>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
		where
			T: 'a,
			B: 'a, {
			let grate = self.grate.clone();

			Q::dimap(
				move |s: S| {
					let s_ptr = <PointerBrand as RefCountedPointer>::cloneable_new(s);
					<FnBrand<PointerBrand> as CloneableFn>::new(
						move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
							'a,
							<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
							A,
						>|
						      -> A { (f)(Clone::clone(&s_ptr)) },
					)
				},
				move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<FnBrand<PointerBrand> as CloneableFn>::Of<
						'a,
						<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
						A,
					>,
					B,
				>| {
					let f_brand = <FnBrand<PointerBrand> as CloneableFn>::new(move |x| f(x));
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
	impl<'a, PointerBrand, S, T, A, B> SetterOptic<'a, PointerBrand, S, T, A, B>
		for Grate<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
		B: 'a + Clone,
		<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::{
		/// 			CloneableFn,
		/// 			optics::*,
		/// 			profunctor::*,
		/// 		},
		/// 		types::optics::Grate,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = Grate::<'_, RcBrand, (i32, i32), (i32, i32), i32, i32>::new(|f| {
		/// 	(
		/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.0) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
		/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.1) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
		/// 	)
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g: Rc<dyn Fn((i32, i32)) -> (i32, i32)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
		{
			GrateOptic::<FnBrand<PointerBrand>, S, T, A, B>::evaluate::<FnBrand<PointerBrand>>(
				self, pab,
			)
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
	pub struct GratePrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		pub(crate) grate_fn: <FnBrand<PointerBrand> as CloneableFn>::Of<
			'a,
			<FnBrand<PointerBrand> as CloneableFn>::Of<
				'a,
				<FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
					A,
				>,
				A,
			>,
			S,
		>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, PointerBrand, S, A> Clone for GratePrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("The cloned grate instance.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::CloneableFn,
		/// 		types::optics::GratePrime,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(
		/// 	|f: Rc<dyn Fn(Rc<dyn Fn(Rc<(i32, i32)>) -> i32>) -> i32>| {
		/// 		(
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0)),
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1)),
		/// 		)
		/// 	},
		/// );
		/// let cloned = grate.clone();
		/// assert_eq!(cloned.zip_with(|(a, b)| a + b, (1, 2), (3, 4)), (4, 6));
		/// ```
		fn clone(&self) -> Self {
			GratePrime {
				grate_fn: self.grate_fn.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	impl<'a, PointerBrand, S, A> GratePrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		/// Creates a new `GratePrime` instance.
		#[document_signature]
		///
		#[document_parameters("The grating function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::CloneableFn,
		/// 		types::optics::GratePrime,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(
		/// 	|f: Rc<dyn Fn(Rc<dyn Fn(Rc<(i32, i32)>) -> i32>) -> i32>| {
		/// 		(
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0)),
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1)),
		/// 		)
		/// 	},
		/// );
		/// assert_eq!(grate.zip_with(|(a, b)| a + b, (1, 2), (3, 4)), (4, 6));
		/// ```
		pub fn new(
			grate: impl Fn(
				<FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<FnBrand<PointerBrand> as CloneableFn>::Of<
						'a,
						<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
						A,
					>,
					A,
				>,
			) -> S
			+ 'a
		) -> Self
		where
			<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized, {
			GratePrime {
				grate_fn: <FnBrand<PointerBrand> as CloneableFn>::new(grate),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The grate instance.")]
	impl<'a, PointerBrand, S, A> GratePrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a + Clone,
	{
		/// Zip two structures together using this grate and a combining function.
		///
		/// Convenience method wrapping [`zip_with_of`].
		#[document_signature]
		///
		#[document_parameters(
			"The combining function, taking a pair `(A, A)` and returning `A`.",
			"The first structure.",
			"The second structure."
		)]
		///
		#[document_returns("The combined structure.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::CloneableFn,
		/// 		types::optics::GratePrime,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(
		/// 	|f: Rc<dyn Fn(Rc<dyn Fn(Rc<(i32, i32)>) -> i32>) -> i32>| {
		/// 		(
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0)),
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1)),
		/// 		)
		/// 	},
		/// );
		/// let result = grate.zip_with(|(a, b)| a + b, (1, 2), (10, 20));
		/// assert_eq!(result, (11, 22));
		/// ```
		pub fn zip_with(
			&self,
			f: impl Fn((A, A)) -> A + 'a,
			s1: S,
			s2: S,
		) -> S
		where
			<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized, {
			zip_with_of::<FnBrand<PointerBrand>, _, S, S, A, A>(self, f, s1, s2)
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
	impl<'a, Q, PointerBrand, S, A> Optic<'a, Q, S, S, A, A> for GratePrime<'a, PointerBrand, S, A>
	where
		Q: Closed<FnBrand<PointerBrand>>,
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a + Clone,
		<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized,
	{
		/// Evaluates the grate with a profunctor.
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::{
		/// 			CloneableFn,
		/// 			optics::*,
		/// 		},
		/// 		types::optics::GratePrime,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(
		/// 	|f: Rc<dyn Fn(Rc<dyn Fn(Rc<(i32, i32)>) -> i32>) -> i32>| {
		/// 		(
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0)),
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1)),
		/// 		)
		/// 	},
		/// );
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
					let s_ptr = <PointerBrand as RefCountedPointer>::cloneable_new(s);
					<FnBrand<PointerBrand> as CloneableFn>::new(
						move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
							'a,
							<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
							A,
						>| { (f)(Clone::clone(&s_ptr)) },
					)
				},
				move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<FnBrand<PointerBrand> as CloneableFn>::Of<
						'a,
						<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
						A,
					>,
					A,
				>| {
					let f_brand = <FnBrand<PointerBrand> as CloneableFn>::new(move |x| f(x));
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
	impl<'a, PointerBrand, S, A> GrateOptic<'a, FnBrand<PointerBrand>, S, S, A, A>
		for GratePrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a + Clone,
		<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::{
		/// 			CloneableFn,
		/// 			optics::*,
		/// 		},
		/// 		types::optics::GratePrime,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(
		/// 	|f: Rc<dyn Fn(Rc<dyn Fn(Rc<(i32, i32)>) -> i32>) -> i32>| {
		/// 		(
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.0)),
		/// 			f(<RcFnBrand as CloneableFn>::new(|s: Rc<(i32, i32)>| s.1)),
		/// 		)
		/// 	},
		/// );
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g = GrateOptic::<RcFnBrand, _, _, _, _>::evaluate::<RcFnBrand>(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate<Q: Closed<FnBrand<PointerBrand>>>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let grate = self.grate_fn.clone();

			Q::dimap(
				move |s: S| {
					let s_ptr = <PointerBrand as RefCountedPointer>::cloneable_new(s);
					<FnBrand<PointerBrand> as CloneableFn>::new(
						move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
							'a,
							<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
							A,
						>| { (f)(Clone::clone(&s_ptr)) },
					)
				},
				move |f: <FnBrand<PointerBrand> as CloneableFn>::Of<
					'a,
					<FnBrand<PointerBrand> as CloneableFn>::Of<
						'a,
						<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>,
						A,
					>,
					A,
				>| {
					let f_brand = <FnBrand<PointerBrand> as CloneableFn>::new(move |x| f(x));
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
	impl<'a, PointerBrand, S, A> SetterOptic<'a, PointerBrand, S, S, A, A>
		for GratePrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a + Clone,
		<PointerBrand as RefCountedPointer>::CloneableOf<'a, S>: Sized,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::{
		/// 			CloneableFn,
		/// 			optics::*,
		/// 			profunctor::*,
		/// 		},
		/// 		types::optics::GratePrime,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let grate = GratePrime::<'_, RcBrand, (i32, i32), i32>::new(|f| {
		/// 	(
		/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.0) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
		/// 		f(Rc::new(|s: Rc<(i32, i32)>| s.1) as Rc<dyn Fn(Rc<(i32, i32)>) -> i32>),
		/// 	)
		/// });
		/// let f = Rc::new(|x: i32| x + 1) as Rc<dyn Fn(i32) -> i32>;
		/// let g: Rc<dyn Fn((i32, i32)) -> (i32, i32)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&grate, f);
		/// assert_eq!(g((10, 20)), (11, 21));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			GrateOptic::<FnBrand<PointerBrand>, S, S, A, A>::evaluate::<FnBrand<PointerBrand>>(
				self, pab,
			)
		}
	}
}
pub use inner::*;
