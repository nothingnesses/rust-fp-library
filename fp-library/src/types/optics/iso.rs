//! Isomorphism optics for bidirectional conversions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				FnBrand,
				optics::*,
			},
			classes::{
				monoid::Monoid,
				optics::*,
				profunctor::{
					Choice,
					Closed,
					Profunctor,
					Strong,
					Wander,
				},
				*,
			},
			kinds::*,
			types::optics::Tagged,
		},
		fp_macros::*,
	};

	/// A polymorphic isomorphism where types can change.
	/// This matches PureScript's `Iso s t a b`.
	///
	/// An isomorphism represents a lossless bidirectional conversion between types.
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	pub struct Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a, {
		/// Forward conversion: from structure to focus.
		pub from: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		/// Backward conversion: from focus to structure.
		pub to: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, B, T>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S, T, A, B> Clone for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Iso` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Iso,
		/// };
		///
		/// let string_chars: Iso<RcBrand, String, String, Vec<char>, Vec<char>> =
		/// 	Iso::new(|s: String| s.chars().collect(), |v: Vec<char>| v.into_iter().collect());
		/// let cloned = string_chars.clone();
		/// assert_eq!(cloned.from("hi".to_string()), vec!['h', 'i']);
		/// ```
		fn clone(&self) -> Self {
			Iso {
				from: self.from.clone(),
				to: self.to.clone(),
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
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		/// Create a new polymorphic isomorphism.
		#[document_signature]
		///
		#[document_parameters(
			"The forward conversion function.",
			"The backward conversion function."
		)]
		///
		#[document_returns("A new instance of the type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Iso,
		/// };
		///
		/// // Iso between String and Vec<char>
		/// let string_chars: Iso<RcBrand, String, String, Vec<char>, Vec<char>> =
		/// 	Iso::new(|s: String| s.chars().collect(), |v: Vec<char>| v.into_iter().collect());
		/// assert_eq!(string_chars.from("hi".to_string()), vec!['h', 'i']);
		/// ```
		pub fn new(
			from: impl 'a + Fn(S) -> A,
			to: impl 'a + Fn(B) -> T,
		) -> Self {
			Iso {
				from: <FnBrand<PointerBrand> as LiftFn>::new(from),
				to: <FnBrand<PointerBrand> as LiftFn>::new(to),
			}
		}

		/// Apply the forward conversion.
		#[document_signature]
		///
		#[document_parameters("The structure to convert.")]
		///
		#[document_returns("The focus value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Iso,
		/// };
		///
		/// let string_chars: Iso<RcBrand, String, String, Vec<char>, Vec<char>> =
		/// 	Iso::new(|s: String| s.chars().collect(), |v: Vec<char>| v.into_iter().collect());
		/// let chars = string_chars.from("hello".to_string());
		/// assert_eq!(chars, vec!['h', 'e', 'l', 'l', 'o']);
		/// ```
		pub fn from(
			&self,
			s: S,
		) -> A {
			(self.from)(s)
		}

		/// Apply the backward conversion.
		#[document_signature]
		///
		#[document_parameters("The focus value to convert.")]
		///
		#[document_returns("The structure value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::Iso,
		/// };
		///
		/// let string_chars: Iso<RcBrand, String, String, Vec<char>, Vec<char>> =
		/// 	Iso::new(|s: String| s.chars().collect(), |v: Vec<char>| v.into_iter().collect());
		/// let s = string_chars.to(vec!['h', 'i']);
		/// assert_eq!(s, "hi");
		/// ```
		pub fn to(
			&self,
			b: B,
		) -> T {
			(self.to)(b)
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
	#[document_parameters("The iso instance.")]
	impl<'a, Q, PointerBrand, S, T, A, B> Optic<'a, Q, S, T, A, B> for Iso<'a, PointerBrand, S, T, A, B>
	where
		Q: Profunctor,
		PointerBrand: UnsizedCoercible,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let from = self.from.clone();
			let to = self.to.clone();

			// The Profunctor encoding of an Iso is:
			// iso from to = dimap from to
			Q::dimap(move |s| from(s), move |b| to(b), pab)
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
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> IsoOptic<'a, S, T, A, B>
		for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> = IsoOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Profunctor + 'static>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
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
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> AffineTraversalOptic<'a, S, T, A, B>
		for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	AffineTraversalOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Strong + Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand used by the profunctor's `Closed` instance.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The iso instance.")]
	impl<'a, FunctionBrand: CloneableFn, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a>
		GrateOptic<'a, FunctionBrand, S, T, A, B> for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	GrateOptic::<RcFnBrand, _, _, _, _>::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Closed<FunctionBrand>>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
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
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> LensOptic<'a, S, T, A, B>
		for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> = LensOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Strong>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
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
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> PrismOptic<'a, S, T, A, B>
		for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	PrismOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
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
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> TraversalOptic<'a, S, T, A, B>
		for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	TraversalOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> GetterOptic<'a, S, A> for Iso<'a, PointerBrand, S, S, A, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = GetterOptic::evaluate(&iso, f);
		/// assert_eq!(folded.run((42,)), 42);
		/// ```
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IsoOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The focus type."
	)]
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> FoldOptic<'a, S, A> for Iso<'a, PointerBrand, S, S, A, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = FoldOptic::evaluate(&iso, f);
		/// assert_eq!(folded.run((42,)), "42".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IsoOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the iso.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The iso instance.")]
	impl<'a, Q, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> SetterOptic<'a, Q, S, T, A, B>
		for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			IsoOptic::evaluate::<FnBrand<Q>>(self, pab)
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
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S: 'a, T: 'a, A: 'a, B: 'a> ReviewOptic<'a, S, T, A, B>
		for Iso<'a, PointerBrand, S, T, A, B>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
		/// let f = Tagged::new(42);
		/// let reviewed = ReviewOptic::evaluate(&iso, f);
		/// assert_eq!(reviewed.0, (42,));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			let to = self.to.clone();
			Tagged::new(to(pab.0))
		}
	}

	/// A concrete isomorphism type where types do not change.
	/// This matches PureScript's `Iso' s a`.
	///
	/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	pub struct IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a, {
		pub(crate) from_fn: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
		pub(crate) to_fn: Apply!(<FnBrand<PointerBrand> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, S>),
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The iso instance.")]
	impl<'a, PointerBrand, S, A> Clone for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `IsoPrime` instance that is a copy of the original.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IsoPrime,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let cloned = iso.clone();
		/// assert_eq!(cloned.from((42,)), 42);
		/// ```
		fn clone(&self) -> Self {
			IsoPrime {
				from_fn: self.from_fn.clone(),
				to_fn: self.to_fn.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		/// Create a new monomorphic isomorphism from bidirectional conversion functions.
		#[document_signature]
		///
		#[document_parameters(
			"The forward conversion function.",
			"The backward conversion function."
		)]
		///
		#[document_returns("A new instance of the type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IsoPrime,
		/// };
		///
		/// // Iso between a newtype and its inner value
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// assert_eq!(iso.from((10,)), 10);
		/// ```
		pub fn new(
			from: impl 'a + Fn(S) -> A,
			to: impl 'a + Fn(A) -> S,
		) -> Self {
			IsoPrime {
				from_fn: <FnBrand<PointerBrand> as LiftFn>::new(from),
				to_fn: <FnBrand<PointerBrand> as LiftFn>::new(to),
			}
		}

		/// Apply the forward conversion.
		#[document_signature]
		///
		#[document_parameters("The structure to convert.")]
		///
		#[document_returns("The focus value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IsoPrime,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// assert_eq!(iso.from((42,)), 42);
		/// ```
		pub fn from(
			&self,
			s: S,
		) -> A {
			(self.from_fn)(s)
		}

		/// Apply the backward conversion.
		#[document_signature]
		///
		#[document_parameters("The focus value to convert.")]
		///
		#[document_returns("The structure value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IsoPrime,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// assert_eq!(iso.to(42), (42,));
		/// ```
		pub fn to(
			&self,
			a: A,
		) -> S {
			(self.to_fn)(a)
		}

		/// Reverse the isomorphism.
		#[document_signature]
		///
		#[document_returns(
			"A new `IsoPrime` instance with the forward and backward conversions swapped."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	types::optics::IsoPrime,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let reversed = iso.reversed();
		/// assert_eq!(reversed.from(42), (42,));
		/// assert_eq!(reversed.to((42,)), 42);
		/// ```
		pub fn reversed(&self) -> IsoPrime<'a, PointerBrand, A, S> {
			IsoPrime {
				from_fn: self.to_fn.clone(),
				to_fn: self.from_fn.clone(),
			}
		}
	}

	// Optic implementation for IsoPrime<PointerBrand, S, A>
	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, Q, PointerBrand, S, A> Optic<'a, Q, S, S, A, A> for IsoPrime<'a, PointerBrand, S, A>
	where
		Q: Profunctor,
		PointerBrand: UnsizedCoercible,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let from_fn = self.from_fn.clone();
			let to_fn = self.to_fn.clone();

			// The Profunctor encoding of an Iso is:
			// iso from to = dimap from to
			Q::dimap(move |s| from_fn(s), move |a| to_fn(a), pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> AffineTraversalOptic<'a, S, S, A, A>
		for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	AffineTraversalOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Strong + Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand used by the profunctor's `Closed` instance.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, FunctionBrand: CloneableFn, PointerBrand, S: 'a, A: 'a>
		GrateOptic<'a, FunctionBrand, S, S, A, A> for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	GrateOptic::<RcFnBrand, _, _, _, _>::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Closed<FunctionBrand>>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> LensOptic<'a, S, S, A, A> for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> = LensOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Strong>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> PrismOptic<'a, S, S, A, A> for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	PrismOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Choice>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> TraversalOptic<'a, S, S, A, A>
		for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	TraversalOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> GetterOptic<'a, S, A> for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters(
			"The return type of the forget profunctor.",
			"The reference-counted pointer type."
		)]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = Forget::<RcBrand, i32, i32, i32>::new(|x| x);
		/// let folded = GetterOptic::evaluate(&iso, f);
		/// assert_eq!(folded.run((42,)), 42);
		/// ```
		fn evaluate<R: 'a + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IsoOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> FoldOptic<'a, S, A> for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The monoid type.", "The reference-counted pointer type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x| x.to_string());
		/// let folded = FoldOptic::evaluate(&iso, f);
		/// assert_eq!(folded.run((42,)), "42".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			IsoOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the iso.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, Q, PointerBrand, S: 'a, A: 'a> SetterOptic<'a, Q, S, S, A, A>
		for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
		Q: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			IsoOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> ReviewOptic<'a, S, S, A, A>
		for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = Tagged::new(42);
		/// let reviewed = ReviewOptic::evaluate(&iso, f);
		/// assert_eq!(reviewed.0, (42,));
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<TaggedBrand as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			let to_fn = self.to_fn.clone();
			Tagged::new(to_fn(pab.0))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus."
	)]
	#[document_parameters("The monomorphic iso instance.")]
	impl<'a, PointerBrand, S: 'a, A: 'a> IsoOptic<'a, S, S, A, A> for IsoPrime<'a, PointerBrand, S, A>
	where
		PointerBrand: UnsizedCoercible,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
		#[document_returns("The transformed profunctor value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::optics::*,
		/// 	functions::*,
		/// 	types::optics::*,
		/// };
		///
		/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let modifier: std::rc::Rc<dyn Fn((i32,)) -> (i32,)> = IsoOptic::evaluate::<RcFnBrand>(&iso, f);
		/// assert_eq!(modifier((41,)), (42,));
		/// ```
		fn evaluate<Q: Profunctor + 'static>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
			Optic::<Q, S, S, A, A>::evaluate(self, pab)
		}
	}
}
pub use inner::*;
