#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBracketExplicitBrand,
				BoxBrand,
				BracketExplicitBrand,
				RcBrand,
				SendBracketExplicitBrand,
			},
			classes::{
				Extract,
				Functor,
				Pointer,
				RefCountedPointer,
				RefFunctor,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynFnOnce,
				ToDynSendFn,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcFreeExplicit,
				FreeExplicit,
				RcFreeExplicit,
			},
		},
		fp_macros::*,
	};

	// ===== BoxBracketExplicit (BoxBrand + FnOnce, single-shot, Box<FreeExplicit> substrate) =====

	/// Scoped resource-management effect for `RunExplicit` substrates.
	/// Mirrors [`BoxBracket`] structurally with one substrate swap:
	/// stored closures return [`Box<FreeExplicit<'a, Sub, _>>`](FreeExplicit)
	/// programs instead of [`Free<Sub, _>`]. The outer `Box` matches
	/// [`FreeExplicit::wrap`]'s expected layer-program type
	/// (`<F>::Of<'a, Box<FreeExplicit<'a, F, A>>>`); without it the
	/// substrate's `Node::Scoped` constructor rejects the layer with a
	/// type-mismatch on the GAT projection.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and the FreeExplicit programs.",
		"The pointer brand storing the closures (BoxBrand only by structural bound).",
		"The substrate brand over which acquire / body / release return FreeExplicit programs.",
		"The resource type produced by acquire and consumed by body / release.",
		"The body's result type."
	)]
	pub enum BoxBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynFnOnce,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a, {
		/// Acquire a resource, run `body` with it to produce a paired
		/// body result, then run `release` to clean up. Mirrors
		/// [`BoxBracket::Bracket`](crate::types::effects::bracket::BoxBracket::Bracket)
		/// over the boxed FreeExplicit substrate.
		Bracket {
			/// The acquire program, stored as a unit-arg B-thunk
			/// returning `Box<FreeExplicit<'a, Sub, A>>`.
			acquire: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> Box<FreeExplicit<'a, Sub, A>>>,
			/// The body closure returning `Box<FreeExplicit<'a, Sub, (A, B)>>`.
			#[expect(
				clippy::type_complexity,
				reason = "BoxBracketExplicit cells store closures returning Box<FreeExplicit<...>> programs derived from the substrate brand Sub; the nested GAT and FreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce dispatch over."
			)]
			body: <P as Pointer>::Of<
				'a,
				dyn 'a + FnOnce(<P as Pointer>::Of<'a, A>) -> Box<FreeExplicit<'a, Sub, (A, B)>>,
			>,
			/// The release closure returning `Box<FreeExplicit<'a, Sub, ()>>`.
			#[expect(
				clippy::type_complexity,
				reason = "BoxBracketExplicit cells store closures returning Box<FreeExplicit<...>> programs derived from the substrate brand Sub; the nested GAT and FreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce dispatch over."
			)]
			release: <P as Pointer>::Of<
				'a,
				dyn 'a + FnOnce(<P as Pointer>::Of<'a, A>) -> Box<FreeExplicit<'a, Sub, ()>>,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, Sub: WrapDrop + 'static, A: 'static, B: 'static> for BoxBracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = BoxBracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Functor for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`; mirrors [`BoxBracketBrand`'s `Functor`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The bracket effect.")]
		///
		#[document_returns("The bracket effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(7))
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure((7, 42)))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(()))
		/// 		}),
		/// 	};
		/// let mapped = <BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as Functor>::map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// match mapped {
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(Box::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(Box::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== BracketExplicit (RcBrand + Fn, multi-shot, RcFreeExplicit substrate) =====

	/// Scoped resource-management effect for `RcRunExplicit` substrates.
	/// Mirrors [`Bracket`] structurally with one substrate swap:
	/// stored closures return [`RcFreeExplicit<'a, Sub, _>`] programs
	/// instead of [`RcFree<Sub, _>`].
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and the RcFreeExplicit programs.",
		"The pointer brand storing the closures (RcBrand only).",
		"The substrate brand over which acquire / body / release return RcFreeExplicit programs.",
		"The resource type.",
		"The body's result type."
	)]
	pub enum BracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a, {
		/// Mirrors
		/// [`Bracket::Bracket`](crate::types::effects::bracket::Bracket::Bracket)
		/// over the RcFreeExplicit substrate.
		Bracket {
			/// The acquire program returning `RcFreeExplicit<'a, Sub, A>`.
			acquire:
				<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> RcFreeExplicit<'a, Sub, A>>,
			/// The body closure returning `RcFreeExplicit<'a, Sub, (A, B)>`.
			#[expect(
				clippy::type_complexity,
				reason = "BracketExplicit cells store closures returning RcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and RcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFreeExplicit<'a, Sub, (A, B)>,
			>,
			/// The release closure returning `RcFreeExplicit<'a, Sub, ()>`.
			#[expect(
				clippy::type_complexity,
				reason = "BracketExplicit cells store closures returning RcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and RcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFreeExplicit<'a, Sub, ()>,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, Sub: WrapDrop + 'static, A: 'static, B: 'static> for BracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = BracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime.",
		"The pointer brand.",
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	#[document_parameters("The bracket-explicit effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for BracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a,
	{
		/// Clones by refcount-bumping the stored pointers.
		#[document_signature]
		///
		#[document_returns(
			"A new bracket-explicit effect sharing acquire / body / release by refcount."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let original: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let cloned = original.clone();
		/// match cloned {
		/// 	BracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				BracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => BracketExplicit::Bracket {
					acquire: <P as RefCountedPointer>::Of::clone(acquire),
					body: <P as RefCountedPointer>::Of::clone(body),
					release: <P as RefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Functor for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`; mirrors [`BracketBrand`'s `Functor`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect.")]
		///
		#[document_returns("The bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as Functor>::map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// match mapped {
		/// 	BracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== SendBracketExplicit (ArcBrand + Fn + Send + Sync, ArcFreeExplicit substrate) =====

	/// Thread-safe scoped resource-management effect for
	/// `ArcRunExplicit` substrates. Mirrors [`SendBracket`] structurally
	/// with one substrate swap: stored closures return
	/// [`ArcFreeExplicit<'a, Sub, _>`] programs instead of
	/// [`ArcFree<Sub, _>`].
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and the ArcFreeExplicit programs.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The substrate brand over which acquire / body / release return ArcFreeExplicit programs.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	pub enum SendBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop + 'a,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a, {
		/// Mirrors
		/// [`SendBracket::Bracket`](crate::types::effects::bracket::SendBracket::Bracket)
		/// over the ArcFreeExplicit substrate.
		Bracket {
			/// The acquire program returning `ArcFreeExplicit<'a, Sub, A>`.
			acquire: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(()) -> ArcFreeExplicit<'a, Sub, A> + Send + Sync,
			>,
			/// The body closure returning `ArcFreeExplicit<'a, Sub, (A, B)>`.
			#[expect(
				clippy::type_complexity,
				reason = "SendBracketExplicit cells store closures returning ArcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and ArcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFreeExplicit<'a, Sub, (A, B)>
					+ Send
					+ Sync,
			>,
			/// The release closure returning `ArcFreeExplicit<'a, Sub, ()>`.
			#[expect(
				clippy::type_complexity,
				reason = "SendBracketExplicit cells store closures returning ArcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and ArcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFreeExplicit<'a, Sub, ()>
					+ Send
					+ Sync,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, Sub: WrapDrop + 'static, A: Send + Sync + 'static, B: Send + Sync + 'static> for SendBracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = SendBracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime.",
		"The pointer brand.",
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	#[document_parameters("The send-bracket-explicit effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for SendBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop + 'a,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a,
	{
		/// Clones by refcount-bumping the stored pointers.
		#[document_signature]
		///
		#[document_returns(
			"A new send-bracket-explicit effect sharing acquire / body / release by refcount."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let original: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::sync::Arc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::sync::Arc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => SendBracketExplicit::Bracket {
					acquire: <P as SendRefCountedPointer>::Of::clone(acquire),
					body: <P as SendRefCountedPointer>::Of::clone(body),
					release: <P as SendRefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	// ===== SendFunctor for SendBracketExplicitBrand =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> SendFunctor for SendBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Identity `send_map`; mirrors [`SendBracketBrand`'s `SendFunctor`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The send-bracket-explicit effect.")]
		///
		#[document_returns("The send-bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<SendBracketExplicitBrand<ArcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 		|x: i32| x + 1,
		/// 		bracket,
		/// 	);
		/// match mapped {
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::sync::Arc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::sync::Arc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// SendFunctor stubs for the non-Send Explicit-family brands. Mirror
	// the Erased-family stub precedent: required by the substrate's
	// `NodeBrand<R, S>: SendFunctor` cascade when S contains these
	// brands; never reached at runtime because the corresponding
	// wrappers (RunExplicit / RcRunExplicit) do not exercise the
	// SendFunctor path.

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync stub bound).",
		"The body's result type (Send + Sync stub bound)."
	)]
	impl<Sub, A, B> SendFunctor for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub; mirrors [`BoxBracketBrand`'s `SendFunctor`] stub.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect.")]
		///
		#[document_returns("The bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Reachable only through brand-dispatch via NodeBrand<R, S>: SendFunctor;
		/// // never invoked at runtime on RunExplicit substrates.
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(7))
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure((7, 42)))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(()))
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 		|x: i32| x + 1,
		/// 		bracket,
		/// 	);
		/// match mapped {
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(Box::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(Box::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync stub bound).",
		"The body's result type (Send + Sync stub bound)."
	)]
	impl<Sub, A, B> SendFunctor for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub; mirrors [`BracketBrand`'s `SendFunctor`] stub.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect.")]
		///
		#[document_returns("The bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// match mapped {
		/// 	BracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== WrapDrop impls for Explicit-family brands =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> WrapDrop for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(7))
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure((7, 42)))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(()))
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(
		/// 		bracket
		/// 	),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> WrapDrop for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(bracket),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> WrapDrop for SendBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-bracket-explicit effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<SendBracketExplicitBrand<ArcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(
		/// 		bracket
		/// 	),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	// ===== Extract impls for Explicit-family brands =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Extract for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynFnOnce,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(7))
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure((7, 42)))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(()))
		/// 		}),
		/// 	};
		/// match bracket {
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(Box::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(Box::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BracketExplicit cells require dispatcher-driven evaluation; substrate-required Extract is unreachable on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"BoxBracketExplicitBrand::extract invoked; BracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Extract for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// match bracket {
		/// 	BracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BracketExplicit cells require dispatcher-driven evaluation; substrate-required Extract is unreachable on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"BracketExplicitBrand::extract invoked; BracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> Extract for SendBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// match bracket {
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::sync::Arc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::sync::Arc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "SendBracketExplicit cells require dispatcher-driven evaluation; substrate-required Extract is unreachable on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"SendBracketExplicitBrand::extract invoked; SendBracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	// ===== RefFunctor impls for Explicit-family brands =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Stub-everywhere `ref_map`; mirrors [`BoxBracketBrand`'s
		/// `RefFunctor`]. `BoxBracketExplicit` is non-`Clone`; the
		/// returned cell is constructed with `unreachable!()` stub
		/// closures.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters(
			"The function (ignored).",
			"The bracket-explicit effect projection (ignored)."
		)]
		///
		#[document_returns(
			"A new bracket-explicit effect with all three closures as panicking stubs."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(7))
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure((7, 42)))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			Box::new(FreeExplicit::<IdentityBrand, _>::pure(()))
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as RefFunctor>::ref_map(
		/// 		|x: &i32| *x + 1,
		/// 		&bracket,
		/// 	);
		/// match mapped {
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert!(
		/// 			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| acquire(()))).is_err()
		/// 		);
		/// 		assert!(
		/// 			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(Box::new(7))))
		/// 				.is_err()
		/// 		);
		/// 		assert!(
		/// 			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| release(Box::new(7))))
		/// 				.is_err()
		/// 		);
		/// 	}
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxBracketExplicitBrand::ref_map cannot replicate the cell from a reference because BoxBracketExplicit is non-Clone (Box<dyn FnOnce> is uncloneable). The path is reachable only through synthetic non-Coyoneda first-order rows on RunExplicit, which real programs do not exercise."
		)]
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			_fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			BoxBracketExplicit::Bracket {
				acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> Box<FreeExplicit<'a, Sub, A>> {
					unreachable!(
						"BoxBracketExplicitBrand::ref_map's stub acquire invoked; BoxBracketExplicit is non-Clone and the impl is reachable only through synthetic substrate paths"
					)
				}),
				body: <BoxBrand as ToDynFnOnce>::new(
					|_a: <BoxBrand as Pointer>::Of<'a, A>| -> Box<FreeExplicit<'a, Sub, (A, B)>> {
						unreachable!(
							"BoxBracketExplicitBrand::ref_map's stub body invoked; BoxBracketExplicit is non-Clone and the impl is reachable only through synthetic substrate paths"
						)
					},
				),
				release: <BoxBrand as ToDynFnOnce>::new(
					|_a: <BoxBrand as Pointer>::Of<'a, A>| -> Box<FreeExplicit<'a, Sub, ()>> {
						unreachable!(
							"BoxBracketExplicitBrand::ref_map's stub release invoked; BoxBracketExplicit is non-Clone and the impl is reachable only through synthetic substrate paths"
						)
					},
				),
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// `Clone::clone(fa)`; mirrors [`BracketBrand`'s `RefFunctor`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect projection.")]
		///
		#[document_returns(
			"A clone of the bracket-explicit effect (acquire / body / release Rc-bumped)."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as RefFunctor>::ref_map(
		/// 	|x: &i32| *x + 1,
		/// 	&bracket,
		/// );
		/// match mapped {
		/// 	BracketExplicit::Bracket {
		/// 		acquire,
		/// 		body,
		/// 		release,
		/// 	} => {
		/// 		assert_eq!(acquire(()).evaluate(), 7);
		/// 		assert_eq!(body(std::rc::Rc::new(7)).evaluate(), (7, 42));
		/// 		assert_eq!(release(std::rc::Rc::new(7)).evaluate(), ());
		/// 	}
		/// }
		/// ```
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa.clone()
		}
	}
}

pub use inner::*;
