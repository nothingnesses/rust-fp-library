//! Stack-safe Free monad over a functor with O(1) [`bind`](crate::functions::bind) operations.
//!
//! Enables building computation chains without stack overflow by using a catenable list of continuations. Note: requires `'static` types and cannot implement the library's HKT traits due to type erasure.
//!
//! ## Comparison with PureScript
//!
//! This implementation is based on the PureScript [`Control.Monad.Free`](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs) module
//! and the ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) technique. It shares the same core algorithmic properties (O(1) bind, stack safety)
//! but differs significantly in its intended use case and API surface.
//!
//! ### Key Differences
//!
//! 1. **Interpretation Strategy**:
//!    * **PureScript**: Designed as a generic Abstract Syntax Tree (AST) that can be interpreted into *any* target
//!      monad using `runFree` or `foldFree` by providing a natural transformation at runtime.
//!    * **Rust**: Designed primarily for **stack-safe execution** of computations. The interpretation logic is
//!      baked into the [`Evaluable`](crate::classes::Evaluable) trait implemented by the functor `F`.
//!      The [`Free::wrap`] method wraps a functor layer containing a Free computation.
//!
//! 2. **API Surface**:
//!    * **PureScript**: Rich API including `liftF`, `hoistFree`, `resume`, `foldFree`.
//!    * **Rust**: Focused API with construction (`pure`, `wrap`, `lift_f`, `bind`), execution (`evaluate`),
//!      introspection (`resume`), and interpretation (`fold_free`).
//!      * `hoistFree` is missing.
//!
//! 3. **Terminology**:
//!    * Rust's `Free::wrap` corresponds to PureScript's `wrap`.
//!
//! ### Capabilities and Limitations
//!
//! **What it CAN do:**
//! * Provide stack-safe recursion for monadic computations (trampolining).
//! * Prevent stack overflows when chaining many `bind` operations.
//! * Execute self-describing effects (like [`Thunk`](crate::types::Thunk)).
//!
//! **What it CANNOT do (easily):**
//! * Act as a generic DSL where the interpretation is decoupled from the operation type.
//!   * *Example*: You cannot easily define a `DatabaseOp` enum and interpret it differently for
//!     production (SQL) and testing (InMemory) using this `Free` implementation, because
//!     `DatabaseOp` must implement a single `Runnable` trait.
//!   * Note: `fold_free` with `NaturalTransformation` does support this pattern for simple cases.
//!
//! ### Lifetimes and Memory Management
//!
//! * **PureScript**: Relies on a garbage collector and `unsafeCoerce`. This allows it to ignore
//!   lifetimes and ownership, enabling a simpler implementation that supports all types.
//! * **Rust**: Relies on ownership and `Box<dyn Any>` for type erasure. `Any` requires `'static`
//!   to ensure memory safety (preventing use-after-free of references). This forces `Free` to
//!   only work with `'static` types, preventing it from implementing the library's HKT traits
//!   which require lifetime polymorphism.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	types::*,
//! };
//!
//! // ✅ CAN DO: Stack-safe recursion
//! let free = Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::ThunkBrand,
			classes::{
				Deferrable,
				Evaluable,
				Functor,
				Monad,
				Pointed,
				Semimonad,
			},
			kinds::*,
			types::{
				CatList,
				Thunk,
			},
		},
		fp_macros::*,
		std::{
			any::Any,
			marker::PhantomData,
		},
	};

	/// A natural transformation from functor `F` to functor `G`.
	///
	/// This trait represents a polymorphic function that transforms `F<B>` into `G<B>`
	/// for any type `B`. It is the Rust workaround for rank-2 polymorphism, which cannot
	/// be expressed directly with closures.
	///
	/// Natural transformations are used by [`Free::fold_free`] to interpret a free monad
	/// over functor `F` into an arbitrary monad `G`.
	#[document_type_parameters("The source functor brand.", "The target functor brand.")]
	pub trait NaturalTransformation<F: Functor + 'static, G: Functor + 'static> {
		/// Applies the natural transformation to a value of type `F<B>`,
		/// producing a value of type `G<B>`.
		#[document_signature]
		///
		#[document_type_parameters("The inner type being transformed over.")]
		///
		#[document_parameters("The functor value to transform.")]
		///
		#[document_returns("The transformed value in the target functor.")]
		fn transform<B: 'static>(
			&self,
			fb: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, B>),
		) -> Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, B>);
	}

	/// A type-erased value for internal use.
	///
	/// This type alias represents a value whose type has been erased to [`Box<dyn Any>`].
	/// It is used within the internal implementation of [`Free`] to allow for
	/// heterogeneous chains of computations in the [`CatList`].
	pub type TypeErasedValue = Box<dyn Any>;

	/// A type-erased continuation.
	///
	/// This type alias represents a function that takes a [`TypeErasedValue`]
	/// and returns a new [`Free`] computation (also type-erased).
	#[document_type_parameters("The base functor.")]
	pub type Continuation<F> = Box<dyn FnOnce(TypeErasedValue) -> Free<F, TypeErasedValue>>;

	/// The internal representation of the [`Free`] monad.
	///
	/// This enum encodes the structure of the free monad, supporting
	/// pure values, suspended computations, and efficient concatenation of binds.
	#[document_type_parameters(
		"The base functor (must implement [`Functor`]).",
		"The result type."
	)]
	pub enum FreeInner<F, A>
	where
		F: Functor + 'static,
		A: 'static, {
		/// A pure value.
		///
		/// This variant represents a computation that has finished and produced a value.
		Pure(A),

		/// A suspended computation.
		///
		/// This variant represents a computation that is suspended in the functor `F`.
		/// The functor contains the next step of the computation.
		Wrap(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)),

		/// A mapped computation.
		///
		/// This variant stores a value and a mapping function, avoiding the
		/// type-erasure roundtrip that would occur if `map` were implemented
		/// via `bind`. This provides a more direct path for functor mapping.
		Map {
			/// The inner computation whose result will be mapped.
			value: Box<Free<F, TypeErasedValue>>,
			/// The mapping function to apply to the result.
			f: Box<dyn FnOnce(TypeErasedValue) -> A>,
		},

		/// A bind operation.
		///
		/// This variant represents a computation followed by a sequence of continuations.
		/// It uses a [`CatList`] to store continuations, ensuring O(1) append complexity
		/// for left-associated binds.
		Bind {
			/// The initial computation.
			head: Box<Free<F, TypeErasedValue>>,
			/// The list of continuations to apply to the result of `head`.
			continuations: CatList<Continuation<F>>,
			/// Phantom data for type parameter `A`.
			_marker: PhantomData<A>,
		},
	}

	/// The Free monad with O(1) bind via [`CatList`].
	///
	/// This implementation follows ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) to ensure
	/// that left-associated binds do not degrade performance.
	///
	/// # HKT and Lifetime Limitations
	///
	/// `Free` does not implement HKT traits (like `Functor`, `Monad`) from this library.
	///
	/// ## The Conflict
	/// * **The Traits**: The `Kind` trait implemented by the `Functor` hierarchy requires the type
	///   constructor to accept *any* lifetime `'a` (e.g., `type Of<'a, A> = Free<F, A>`).
	/// * **The Implementation**: This implementation uses [`Box<dyn Any>`] to type-erase continuations
	///   for the "Reflection without Remorse" optimization. `dyn Any` strictly requires `A: 'static`.
	///
	/// This creates an unresolvable conflict: `Free` cannot support non-static references (like `&'a str`),
	/// so it cannot satisfy the `Kind` signature.
	///
	/// ## Why not use the "Naive" Recursive Definition?
	///
	/// A naive definition (`enum Free { Pure(A), Wrap(F<Box<Free<F, A>>>) }`) would support lifetimes
	/// and HKT traits. However, it was rejected because:
	/// 1.  **Stack Safety**: `run` would not be stack-safe for deep computations.
	/// 2.  **Performance**: `bind` would be O(N), leading to quadratic complexity for sequences of binds.
	///
	/// This implementation prioritizes **stack safety** and **O(1) bind** over HKT trait compatibility.
	#[document_type_parameters(
		"The base functor (must implement [`Functor`]).",
		"The result type."
	)]
	///
	pub struct Free<F, A>(pub(crate) Option<FreeInner<F, A>>)
	where
		F: Functor + 'static,
		A: 'static;

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The Free monad instance to operate on.")]
	impl<F, A> Free<F, A>
	where
		F: Functor + 'static,
		A: 'static,
	{
		/// Creates a pure `Free` value.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `Free` computation that produces `a`.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			Free(Some(FreeInner::Pure(a)))
		}

		/// Creates a suspended computation from a functor value.
		#[document_signature]
		///
		#[document_parameters("The functor value containing the next step.")]
		///
		#[document_returns("A `Free` computation that performs the effect `fa`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let eval = Thunk::new(|| Free::pure(42));
		/// let free = Free::<ThunkBrand, _>::wrap(eval);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn wrap(
			fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)
		) -> Self {
			Free(Some(FreeInner::Wrap(fa)))
		}

		/// Lifts a functor value into the Free monad.
		///
		/// This is the primary way to inject effects into Free monad computations.
		/// Equivalent to PureScript's `liftF` and Haskell's `liftF`.
		#[document_signature]
		///
		/// ### Implementation
		///
		/// ```text
		/// liftF fa = wrap (map pure fa)
		/// ```
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns("A `Free` computation that performs the effect and returns the result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // Lift a simple computation
		/// let thunk = Thunk::new(|| 42);
		/// let free = Free::<ThunkBrand, _>::lift_f(thunk);
		/// assert_eq!(free.evaluate(), 42);
		///
		/// // Build a computation from raw effects
		/// let computation = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
		/// 	.bind(|x| Free::lift_f(Thunk::new(move || x * 2)))
		/// 	.bind(|x| Free::lift_f(Thunk::new(move || x + 5)));
		/// assert_eq!(computation.evaluate(), 25);
		/// ```
		pub fn lift_f(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)) -> Self {
			// Map the value to a pure Free, then wrap it
			Free::wrap(F::map(Free::pure, fa))
		}

		/// Monadic bind with O(1) complexity.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `Free` computation that chains `f` after this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1));
		/// assert_eq!(free.evaluate(), 43);
		/// ```
		pub fn bind<B: 'static>(
			mut self,
			f: impl FnOnce(A) -> Free<F, B> + 'static,
		) -> Free<F, B> {
			// Type-erase the continuation
			let erased_f: Continuation<F> = Box::new(move |val: TypeErasedValue| {
				// SAFETY: type is maintained by internal invariant - mismatch indicates a bug
				#[allow(clippy::expect_used)]
				let a: A = *val.downcast().expect("Type mismatch in Free::bind");
				let free_b: Free<F, B> = f(a);
				free_b.erase_type()
			});

			// Extract inner safely
			// SAFETY: Free values are used exactly once - double consumption indicates a bug
			#[allow(clippy::expect_used)]
			let inner = self.0.take().expect("Free value already consumed");

			match inner {
				// Pure: create a Bind with this continuation
				FreeInner::Pure(a) => {
					let head: Free<F, TypeErasedValue> = Free::pure(a).erase_type();
					Free(Some(FreeInner::Bind {
						head: Box::new(head),
						continuations: CatList::singleton(erased_f),
						_marker: PhantomData,
					}))
				}

				// Wrap: wrap in a Bind
				FreeInner::Wrap(fa) => {
					let head = Free::wrap(fa).boxed_erase_type();
					Free(Some(FreeInner::Bind {
						head,
						continuations: CatList::singleton(erased_f),
						_marker: PhantomData,
					}))
				}

				// Map: wrap in a Bind
				FreeInner::Map {
					value,
					f: map_fn,
				} => {
					let head = Free(Some(FreeInner::Map {
						value,
						f: map_fn,
					}));
					Free(Some(FreeInner::Bind {
						head: head.boxed_erase_type(),
						continuations: CatList::singleton(erased_f),
						_marker: PhantomData,
					}))
				}

				// Bind: snoc the new continuation onto the CatList (O(1)!)
				FreeInner::Bind {
					head,
					continuations: conts,
					..
				} => Free(Some(FreeInner::Bind {
					head,
					continuations: conts.snoc(erased_f),
					_marker: PhantomData,
				})),
			}
		}

		/// Functor map: transforms the result without changing structure.
		///
		/// Uses the [`Map`](FreeInner::Map) variant directly to avoid the
		/// type-erasure overhead of going through [`bind`](Free::bind).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the mapping function.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `Free` computation with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(10).map(|x| x * 2);
		/// assert_eq!(free.evaluate(), 20);
		/// ```
		pub fn map<B: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> Free<F, B> {
			let erased_self = self.erase_type();
			let erased_f: Box<dyn FnOnce(TypeErasedValue) -> B> =
				Box::new(move |val: TypeErasedValue| {
					// SAFETY: type is maintained by internal invariant - mismatch indicates a bug
					#[allow(clippy::expect_used)]
					let a: A = *val.downcast().expect("Type mismatch in Free::map");
					f(a)
				});
			Free(Some(FreeInner::Map {
				value: Box::new(erased_self),
				f: erased_f,
			}))
		}

		/// Decomposes this `Free` computation into one step.
		///
		/// Returns `Ok(a)` if the computation is a pure value, or
		/// `Err(f_free)` if the computation is suspended in the functor `F`,
		/// where `f_free` contains the next `Free` computation wrapped in `F`.
		///
		/// For `Bind` and `Map` variants, the continuation chain is collapsed first by
		/// iteratively applying continuations until a `Pure` or `Wrap` is reached.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` if the computation is a pure value, `Err(fa)` if it is a suspended computation."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // Pure returns Ok
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert!(matches!(free.resume(), Ok(42)));
		///
		/// // Wrap returns Err containing the functor layer
		/// let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		/// assert!(free.resume().is_err());
		/// ```
		pub fn resume(
			self
		) -> Result<A, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)> {
			// We process the bind chain iteratively, similar to evaluate,
			// but stop when we reach a Pure or Wrap at the top level.
			let mut current: Free<F, TypeErasedValue> = self.erase_type();
			let mut continuations: CatList<Continuation<F>> = CatList::empty();

			loop {
				// SAFETY: Free values are used exactly once - double consumption indicates a bug
				#[allow(clippy::expect_used)]
				let inner = current.0.take().expect("Free value already consumed");

				match inner {
					FreeInner::Pure(val) => {
						match continuations.uncons() {
							Some((continuation, rest)) => {
								current = continuation(val);
								continuations = rest;
							}
							None => {
								// No more continuations - we have a final pure value
								// SAFETY: type is maintained by internal invariant
								#[allow(clippy::expect_used)]
								return Ok(*val
									.downcast::<A>()
									.expect("Type mismatch in Free::resume final downcast"));
							}
						}
					}

					FreeInner::Wrap(fa) => {
						// We have a suspended computation. We need to rebuild the
						// Free<F, A> from the type-erased Free<F, TypeErasedValue>
						// by re-attaching the remaining continuations.
						if continuations.is_empty() {
							// No continuations to reattach; just reconstruct the typed version.
							// fa is F<Free<F, TypeErasedValue>>. We need F<Free<F, A>>.
							let typed_fa = F::map(
								|inner_free: Free<F, TypeErasedValue>| {
									// Wrap the erased free in a Bind that downcasts back to A
									let cont: Continuation<F> =
										Box::new(move |val: TypeErasedValue| {
											// SAFETY: type is maintained by internal invariant
											#[allow(clippy::expect_used)]
											let a: A = *val
												.downcast()
												.expect("Type mismatch in Free::resume downcast");
											Free::<F, A>::pure(a).erase_type()
										});
									Free(Some(FreeInner::Bind {
										head: Box::new(inner_free),
										continuations: CatList::singleton(cont),
										_marker: PhantomData,
									}))
								},
								fa,
							);
							return Err(typed_fa);
						} else {
							// There are continuations remaining. We need to attach them
							// to the inner Free computations via map.
							// Use Cell to move the CatList into the Fn closure (called exactly once).
							let remaining = std::cell::Cell::new(Some(continuations));
							let typed_fa = F::map(
								move |inner_free: Free<F, TypeErasedValue>| {
									// SAFETY: functors call map exactly once per element
									#[allow(clippy::expect_used)]
									let conts = remaining
										.take()
										.expect("Free::resume map called more than once");
									// Create a Bind node that first runs inner_free,
									// then applies the remaining continuations, then
									// downcasts back to A.
									let downcast_cont: Continuation<F> =
										Box::new(move |val: TypeErasedValue| {
											// SAFETY: type is maintained by internal invariant
											#[allow(clippy::expect_used)]
											let a: A = *val
												.downcast()
												.expect("Type mismatch in Free::resume downcast");
											Free::<F, A>::pure(a).erase_type()
										});
									let all_conts = conts.snoc(downcast_cont);
									Free(Some(FreeInner::Bind {
										head: Box::new(inner_free),
										continuations: all_conts,
										_marker: PhantomData,
									}))
								},
								fa,
							);
							return Err(typed_fa);
						}
					}

					FreeInner::Map {
						value,
						f: map_fn,
					} => {
						// Convert the Map into a continuation and continue the loop
						let map_cont: Continuation<F> = Box::new(move |val: TypeErasedValue| {
							let mapped = map_fn(val);
							Free::pure(mapped)
						});
						current = *value;
						continuations = CatList::singleton(map_cont).append(continuations);
					}

					FreeInner::Bind {
						head,
						continuations: inner_continuations,
						..
					} => {
						current = *head;
						continuations = inner_continuations.append(continuations);
					}
				}
			}
		}

		/// Interprets this `Free` monad into a target monad `G` using a natural transformation.
		///
		/// This is the standard `foldFree` operation from Haskell/PureScript. It recursively
		/// processes the free structure, applying the natural transformation at each suspended
		/// layer to convert from functor `F` into monad `G`.
		///
		/// For `Pure(a)`, returns `G::pure(a)`.
		/// For `Wrap(fa)`, applies the natural transformation to get `G<Free<F, A>>`,
		/// then binds recursively to interpret the rest.
		#[document_signature]
		///
		#[document_type_parameters("The target monad brand.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("The result of interpreting this free computation in monad `G`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::Functor,
		/// 	kinds::*,
		/// 	types::*,
		/// };
		///
		/// // Define a natural transformation from Thunk to Option
		/// #[derive(Clone)]
		/// struct ThunkToOption;
		/// impl NaturalTransformation<ThunkBrand, OptionBrand> for ThunkToOption {
		/// 	fn transform<B: 'static>(
		/// 		&self,
		/// 		fb: Thunk<'static, B>,
		/// 	) -> Option<B> {
		/// 		Some(fb.evaluate())
		/// 	}
		/// }
		///
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		/// assert_eq!(result, Some(42));
		/// ```
		pub fn fold_free<G>(
			self,
			nt: impl NaturalTransformation<F, G> + Clone + 'static,
		) -> Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			G: Monad + 'static, {
			match self.resume() {
				Ok(a) => G::pure(a),
				Err(fa) => {
					// fa: F<Free<F, A>>
					// Transform F<Free<F, A>> into G<Free<F, A>> using the natural transformation
					let nt2 = nt.clone();
					let ga: Apply!(
						<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>
					) = nt.transform(fa);
					// Bind over G to recursively fold the inner Free<F, A>
					G::bind(ga, move |inner_free: Free<F, A>| {
						inner_free.fold_free::<G>(nt2.clone())
					})
				}
			}
		}

		/// Converts to type-erased form.
		#[document_signature]
		#[document_returns(
			"A `Free` computation where the result type has been erased to `Box<dyn Any>`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// let erased = free.erase_type();
		/// assert!(erased.evaluate().is::<i32>());
		/// ```
		pub fn erase_type(mut self) -> Free<F, TypeErasedValue> {
			// SAFETY: Free values are used exactly once - double consumption indicates a bug
			#[allow(clippy::expect_used)]
			let inner = self.0.take().expect("Free value already consumed");

			match inner {
				FreeInner::Pure(a) => Free(Some(FreeInner::Pure(Box::new(a) as TypeErasedValue))),
				FreeInner::Wrap(fa) => {
					// Map over the functor to erase the inner type
					let erased = F::map(|inner: Free<F, A>| inner.erase_type(), fa);
					Free(Some(FreeInner::Wrap(erased)))
				}
				FreeInner::Map {
					value,
					f: map_fn,
				} => {
					// Compose the map function with boxing to produce TypeErasedValue
					let erased_f: Box<dyn FnOnce(TypeErasedValue) -> TypeErasedValue> =
						Box::new(move |val: TypeErasedValue| {
							Box::new(map_fn(val)) as TypeErasedValue
						});
					Free(Some(FreeInner::Map {
						value,
						f: erased_f,
					}))
				}
				FreeInner::Bind {
					head,
					continuations,
					..
				} => Free(Some(FreeInner::Bind {
					head,
					continuations,
					_marker: PhantomData,
				})),
			}
		}

		/// Converts to boxed type-erased form.
		#[document_signature]
		#[document_returns("A boxed `Free` computation where the result type has been erased.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// let boxed = free.boxed_erase_type();
		/// assert!(boxed.evaluate().is::<i32>());
		/// ```
		pub fn boxed_erase_type(self) -> Box<Free<F, TypeErasedValue>> {
			Box::new(self.erase_type())
		}

		/// Executes the Free computation, returning the final result.
		///
		/// This is the "trampoline" that iteratively processes the
		/// [`CatList`] of continuations without growing the stack.
		#[document_signature]
		///
		#[document_returns("The final result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn evaluate(self) -> A
		where
			F: Evaluable, {
			// Start with a type-erased version
			let mut current: Free<F, TypeErasedValue> = self.erase_type();
			let mut continuations: CatList<Continuation<F>> = CatList::empty();

			loop {
				// SAFETY: Free values are used exactly once - double consumption indicates a bug
				#[allow(clippy::expect_used)]
				let inner = current.0.take().expect("Free value already consumed");

				match inner {
					FreeInner::Pure(val) => {
						// Try to apply the next continuation
						match continuations.uncons() {
							Some((continuation, rest)) => {
								current = continuation(val);
								continuations = rest;
							}
							None => {
								// No more continuations - we're done!
								// SAFETY: type is maintained by internal invariant - mismatch indicates a bug
								#[allow(clippy::expect_used)]
								return *val
									.downcast::<A>()
									.expect("Type mismatch in Free::evaluate final downcast");
							}
						}
					}

					FreeInner::Wrap(fa) => {
						// Run the effect to get the inner Free
						current = <F as Evaluable>::evaluate(fa);
					}

					FreeInner::Map {
						value,
						f: map_fn,
					} => {
						// Convert the Map into a continuation and continue the loop
						let map_cont: Continuation<F> = Box::new(move |val: TypeErasedValue| {
							let mapped = map_fn(val);
							Free::pure(mapped)
						});
						current = *value;
						continuations = CatList::singleton(map_cont).append(continuations);
					}

					FreeInner::Bind {
						head,
						continuations: inner_continuations,
						..
					} => {
						// Merge the inner continuations with outer ones
						// This is where CatList's O(1) append shines!
						current = *head;
						continuations = inner_continuations.append(continuations);
					}
				}
			}
		}
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The free monad instance to drop.")]
	impl<F, A> Drop for Free<F, A>
	where
		F: Functor + 'static,
		A: 'static,
	{
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = Free::<ThunkBrand, _>::pure(42);
		/// } // drop called here
		/// assert!(true);
		/// ```
		fn drop(&mut self) {
			// We take the inner value out.
			let inner = self.0.take();

			// Handle Map variant: take out the inner value to continue the chain.
			if let Some(FreeInner::Map {
				mut value, ..
			}) = inner
			{
				let mut current = value.0.take();

				// Walk through any nested Bind or Map chains.
				loop {
					match current {
						Some(FreeInner::Bind {
							mut head, ..
						}) => {
							current = head.0.take();
						}
						Some(FreeInner::Map {
							mut value, ..
						}) => {
							current = value.0.take();
						}
						_ => break,
					}
				}
				return;
			}

			// If the top level is a Bind, we need to start the iterative drop chain.
			if let Some(FreeInner::Bind {
				mut head, ..
			}) = inner
			{
				// head is Box<Free<F, TypeEraseValue>>.
				// We take its inner value to continue the chain.
				// From now on, everything is typed as FreeInner<F, TypeEraseValue>.
				let mut current = head.0.take();

				loop {
					match current {
						Some(FreeInner::Bind {
							mut head, ..
						}) => {
							current = head.0.take();
						}
						Some(FreeInner::Map {
							mut value, ..
						}) => {
							current = value.0.take();
						}
						_ => break,
					}
				}
			}
		}
	}

	#[document_type_parameters("The result type.")]
	impl<A: 'static> Deferrable<'static> for Free<ThunkBrand, A> {
		/// Creates a `Free` computation from a thunk.
		///
		/// This delegates to `Free::wrap` and `Thunk::new`.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the free computation.")]
		///
		#[document_returns("The deferred free computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Deferrable,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let task: Free<ThunkBrand, i32> = Deferrable::defer(|| Free::pure(42));
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'static) -> Self
		where
			Self: Sized, {
			Self::wrap(Thunk::new(f))
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::{
				OptionBrand,
				ThunkBrand,
			},
			classes::Functor,
			types::thunk::Thunk,
		},
	};

	/// Tests `Free::pure`.
	///
	/// **What it tests:** Verifies that `pure` creates a computation that simply returns the provided value.
	/// **How it tests:** Constructs a `Free::pure(42)` and runs it, asserting the result is 42.
	#[test]
	fn test_free_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests `Free::roll`.
	///
	/// **What it tests:** Verifies that `roll` creates a computation from a suspended effect.
	/// **How it tests:** Wraps a `Free::pure(42)` inside a `Thunk`, rolls it into a `Free`, and runs it to ensure it unwraps correctly.
	#[test]
	fn test_free_roll() {
		let eval = Thunk::new(|| Free::pure(42));
		let free = Free::<ThunkBrand, _>::wrap(eval);
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests `Free::bind`.
	///
	/// **What it tests:** Verifies that `bind` correctly chains computations and passes values between them.
	/// **How it tests:** Chains `pure(42) -> bind(+1) -> bind(*2)` and asserts the result is (42+1)*2 = 86.
	#[test]
	fn test_free_bind() {
		let free =
			Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1)).bind(|x| Free::pure(x * 2));
		assert_eq!(free.evaluate(), 86);
	}

	/// Tests stack safety of `Free::evaluate`.
	///
	/// **What it tests:** Verifies that `run` can handle deep recursion without stack overflow (trampolining).
	/// **How it tests:** Creates a recursive `count_down` function that builds a chain of 100,000 `bind` calls.
	/// If the implementation were not stack-safe, this would crash with a stack overflow.
	#[test]
	fn test_free_stack_safety() {
		fn count_down(n: i32) -> Free<ThunkBrand, i32> {
			if n == 0 { Free::pure(0) } else { Free::pure(n).bind(|n| count_down(n - 1)) }
		}

		// 100,000 iterations should overflow stack if not safe
		let free = count_down(100_000);
		assert_eq!(free.evaluate(), 0);
	}

	/// Tests stack safety of `Free::drop`.
	///
	/// **What it tests:** Verifies that dropping a deep `Free` computation does not cause a stack overflow.
	/// **How it tests:** Constructs a deep `Free` chain (similar to `test_free_stack_safety`) and lets it go out of scope.
	#[test]
	fn test_free_drop_safety() {
		fn count_down(n: i32) -> Free<ThunkBrand, i32> {
			if n == 0 { Free::pure(0) } else { Free::pure(n).bind(|n| count_down(n - 1)) }
		}

		// Construct a deep chain but DO NOT run it.
		// When `free` goes out of scope, `Drop` should handle it iteratively.
		let _free = count_down(100_000);
	}

	/// Tests `Free::bind` on a `Wrap` variant.
	///
	/// **What it tests:** Verifies that `bind` works correctly when applied to a suspended computation (`Wrap`).
	/// **How it tests:** Creates a `Wrap` (via `wrap`) and `bind`s it.
	#[test]
	fn test_free_bind_on_wrap() {
		let eval = Thunk::new(|| Free::pure(42));
		let free = Free::<ThunkBrand, _>::wrap(eval).bind(|x| Free::pure(x + 1));
		assert_eq!(free.evaluate(), 43);
	}

	/// Tests `Free::lift_f`.
	///
	/// **What it tests:** Verifies that `lift_f` correctly lifts a functor value into the Free monad.
	/// **How it tests:** Lifts a simple thunk and verifies the result.
	#[test]
	fn test_free_lift_f() {
		let thunk = Thunk::new(|| 42);
		let free = Free::<ThunkBrand, _>::lift_f(thunk);
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests `Free::lift_f` with bind.
	///
	/// **What it tests:** Verifies that `lift_f` can be used to build computations with `bind`.
	/// **How it tests:** Chains multiple `lift_f` calls with `bind`.
	#[test]
	fn test_free_lift_f_with_bind() {
		let computation = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x * 2)))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x + 5)));
		assert_eq!(computation.evaluate(), 25);
	}

	/// Tests `Free::resume` on a `Pure` variant.
	///
	/// **What it tests:** Verifies that `resume` on a pure value returns `Ok(value)`.
	/// **How it tests:** Creates a `Free::pure(42)` and asserts `resume()` returns `Ok(42)`.
	#[test]
	fn test_free_resume_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		assert!(matches!(free.resume(), Ok(42)));
	}

	/// Tests `Free::resume` on a `Wrap` variant.
	///
	/// **What it tests:** Verifies that `resume` on a suspended computation returns `Err(functor_layer)`.
	/// **How it tests:** Creates a `Free::wrap(...)` and asserts `resume()` returns `Err(_)`,
	/// then evaluates the inner `Free` to verify the value is preserved.
	#[test]
	fn test_free_resume_wrap() {
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		let result = free.resume();
		assert!(result.is_err());
		// Evaluate the inner thunk to verify the value
		let thunk = result.unwrap_err();
		let inner_free: Free<ThunkBrand, i32> = thunk.evaluate();
		assert_eq!(inner_free.evaluate(), 99);
	}

	/// Tests `Free::resume` on a `Bind` variant.
	///
	/// **What it tests:** Verifies that `resume` correctly collapses bind chains.
	/// **How it tests:** Creates a `pure(10).bind(|x| pure(x + 5))` and checks
	/// that resume returns `Ok(15)` after collapsing the bind.
	#[test]
	fn test_free_resume_bind() {
		let free = Free::<ThunkBrand, _>::pure(10).bind(|x: i32| Free::pure(x + 5));
		assert!(matches!(free.resume(), Ok(15)));
	}

	/// Tests `Free::resume` on a `Bind` with `Wrap`.
	///
	/// **What it tests:** Verifies that `resume` correctly handles bind chains that end in a `Wrap`.
	/// **How it tests:** Creates a `wrap(...).bind(...)` and checks that resume returns `Err(_)`.
	#[test]
	fn test_free_resume_bind_on_wrap() {
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(10)))
			.bind(|x: i32| Free::pure(x + 5));
		let result = free.resume();
		assert!(result.is_err());
		// The inner computation, when evaluated, should produce 15
		let thunk = result.unwrap_err();
		let inner_free: Free<ThunkBrand, i32> = thunk.evaluate();
		assert_eq!(inner_free.evaluate(), 15);
	}

	/// A natural transformation from `ThunkBrand` to `OptionBrand`.
	///
	/// Evaluates the thunk and wraps the result in `Some`.
	#[derive(Clone)]
	struct ThunkToOption;
	impl NaturalTransformation<ThunkBrand, OptionBrand> for ThunkToOption {
		fn transform<B: 'static>(
			&self,
			fb: Thunk<'static, B>,
		) -> Option<B> {
			Some(fb.evaluate())
		}
	}

	/// Tests `Free::fold_free` on a `Pure` variant.
	///
	/// **What it tests:** Verifies that `fold_free` on a pure value wraps it in the target monad.
	/// **How it tests:** Folds `Free::pure(42)` with `ThunkToOption` and asserts it returns `Some(42)`.
	#[test]
	fn test_free_fold_free_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		assert_eq!(result, Some(42));
	}

	/// Tests `Free::fold_free` on a suspended computation.
	///
	/// **What it tests:** Verifies that `fold_free` correctly interprets a computation with effects.
	/// **How it tests:** Lifts a thunk into Free, then folds with `ThunkToOption`.
	#[test]
	fn test_free_fold_free_wrap() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 42));
		let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		assert_eq!(result, Some(42));
	}

	/// Tests `Free::fold_free` on a chained computation.
	///
	/// **What it tests:** Verifies that `fold_free` correctly interprets a multi-step computation.
	/// **How it tests:** Chains several binds and folds with `ThunkToOption`.
	#[test]
	fn test_free_fold_free_chain() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x * 2)))
			.bind(|x| Free::pure(x + 5));
		let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		assert_eq!(result, Some(25));
	}

	/// Tests `Free::map` on a pure value.
	///
	/// **What it tests:** Verifies that the `Map` variant correctly transforms a pure value.
	/// **How it tests:** Creates `Free::pure(10).map(|x| x * 2)` and evaluates.
	#[test]
	fn test_free_map_pure() {
		let free = Free::<ThunkBrand, _>::pure(10).map(|x| x * 2);
		assert_eq!(free.evaluate(), 20);
	}

	/// Tests chained `Free::map` operations.
	///
	/// **What it tests:** Verifies that multiple map operations compose correctly.
	/// **How it tests:** Chains three maps and verifies the final result.
	#[test]
	fn test_free_map_chain() {
		let free = Free::<ThunkBrand, _>::pure(5).map(|x| x + 1).map(|x| x * 3).map(|x| x - 2);
		assert_eq!(free.evaluate(), 16);
	}

	/// Tests `Free::map` on a wrapped computation.
	///
	/// **What it tests:** Verifies that map works on suspended computations.
	/// **How it tests:** Wraps a thunk in Free, maps over it, and evaluates.
	#[test]
	fn test_free_map_wrap() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 7)).map(|x| x * 3);
		assert_eq!(free.evaluate(), 21);
	}

	/// Tests `Free::map` followed by `Free::bind`.
	///
	/// **What it tests:** Verifies that map and bind interoperate correctly.
	/// **How it tests:** Maps, then binds, and checks the result.
	#[test]
	fn test_free_map_then_bind() {
		let free = Free::<ThunkBrand, _>::pure(10).map(|x| x + 5).bind(|x| Free::pure(x * 2));
		assert_eq!(free.evaluate(), 30);
	}

	/// Tests `Free::bind` followed by `Free::map`.
	///
	/// **What it tests:** Verifies that bind and map interoperate correctly.
	/// **How it tests:** Binds, then maps, and checks the result.
	#[test]
	fn test_free_bind_then_map() {
		let free = Free::<ThunkBrand, _>::pure(10).bind(|x| Free::pure(x + 5)).map(|x| x * 2);
		assert_eq!(free.evaluate(), 30);
	}

	/// Tests stack safety of deep `Free::map` chains.
	///
	/// **What it tests:** Verifies that deeply nested map chains do not overflow the stack.
	/// **How it tests:** Creates a chain of 10,000 map operations.
	#[test]
	fn test_free_map_deep_chain() {
		let mut free = Free::<ThunkBrand, _>::pure(0i64);
		for _ in 0 .. 10_000 {
			free = free.map(|x| x + 1);
		}
		assert_eq!(free.evaluate(), 10_000);
	}

	/// Tests drop safety of deep `Free::map` chains.
	///
	/// **What it tests:** Verifies that dropping deeply nested map chains does not overflow the stack.
	/// **How it tests:** Creates a deep map chain and drops it without evaluating.
	#[test]
	fn test_free_map_drop_safety() {
		let mut free = Free::<ThunkBrand, _>::pure(0i64);
		for _ in 0 .. 10_000 {
			free = free.map(|x| x + 1);
		}
		drop(free);
	}
}
