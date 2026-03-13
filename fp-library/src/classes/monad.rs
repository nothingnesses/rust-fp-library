//! Monads, allowing for sequencing computations where the structure depends on previous results.
//!
//! A monad combines [`Pointed`][crate::classes::Pointed] (for lifting values with
//! [`pure`][crate::functions::pure]) and [`Semimonad`][crate::classes::Semimonad]
//! (for chaining computations with [`bind`][crate::functions::bind]).
//! The [`m_do!`][fp_macros::m_do] macro provides do-notation for writing monadic code
//! in a flat, readable style.
//!
//! ### Examples
//!
//! Chaining fallible computations with [`Option`]:
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//! use fp_macros::m_do;
//!
//! fn safe_div(a: i32, b: i32) -> Option<i32> {
//! 	if b == 0 { None } else { Some(a / b) }
//! }
//!
//! // Each `<-` extracts the value; None short-circuits the whole block
//! let result = m_do!(OptionBrand {
//! 	x <- safe_div(100, 2);
//! 	y <- safe_div(x, 5);
//! 	pure(y + 1)
//! });
//! assert_eq!(result, Some(11));
//!
//! // Short-circuits on failure
//! let result = m_do!(OptionBrand {
//! 	x <- safe_div(100, 0);
//! 	y <- safe_div(x, 5);
//! 	pure(y + 1)
//! });
//! assert_eq!(result, None);
//! ```
//!
//! List comprehensions with [`Vec`]:
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//! use fp_macros::m_do;
//!
//! // Generate Pythagorean triples up to 10
//! let triples = m_do!(VecBrand {
//! 	x <- (1..=10i32).collect::<Vec<_>>();
//! 	y <- (x..=10).collect::<Vec<_>>();
//! 	z <- (y..=10).collect::<Vec<_>>();
//! 	_ <- if x * x + y * y == z * z { vec![()] } else { vec![] };
//! 	pure((x, y, z))
//! });
//! assert_eq!(triples, vec![(3, 4, 5), (6, 8, 10)]);
//! ```
//!
//! Error handling with [`Result`]:
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//! use fp_macros::m_do;
//!
//! fn parse_field(input: &str) -> Result<i32, String> {
//! 	input.parse().map_err(|_| format!("invalid: {}", input))
//! }
//!
//! let result: Result<i32, String> = m_do!(ResultErrAppliedBrand<String> {
//! 	x <- parse_field("10");
//! 	y <- parse_field("20");
//! 	let sum = x + y;
//! 	pure(sum)
//! });
//! assert_eq!(result, Ok(30));
//!
//! let result: Result<i32, String> = m_do!(ResultErrAppliedBrand<String> {
//! 	x <- parse_field("10");
//! 	y <- parse_field("abc");
//! 	pure(x + y)
//! });
//! assert_eq!(result, Err("invalid: abc".to_string()));
//! ```
//!
//! The `m_do!` macro supports typed bindings, let bindings, sequencing, and
//! automatic `pure` rewriting:
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//! use fp_macros::m_do;
//!
//! // Typed bindings
//! let r = m_do!(OptionBrand { x: i32 <- Some(5); pure(x * 2) });
//! assert_eq!(r, Some(10));
//!
//! // Let bindings for pure local computations
//! let r = m_do!(OptionBrand {
//! 	x <- Some(5);
//! 	let y = x * 2;
//! 	pure(y)
//! });
//! assert_eq!(r, Some(10));
//!
//! // Sequencing: execute for effects, discard result
//! let r = m_do!(OptionBrand { Some(()); pure(42) });
//! assert_eq!(r, Some(42));
//!
//! // `pure(...)` is auto-rewritten with the correct brand
//! let r = m_do!(OptionBrand {
//! 	x <- Some(5);
//! 	y <- pure(x + 1);
//! 	pure(x + y)
//! });
//! assert_eq!(r, Some(11));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for monads, allowing for sequencing computations where the
	/// structure of the computation depends on the result of the previous
	/// computation.
	///
	/// `class (Applicative m, Semimonad m) => Monad m`
	///
	/// A lawful `Monad` must satisfy three laws:
	///
	/// 1. **Left identity**: `bind(pure(a), f) ≡ f(a)`: lifting a value and
	///    immediately binding it is the same as applying the function directly.
	/// 2. **Right identity**: `bind(m, pure) ≡ m`: binding a computation to
	///    `pure` leaves it unchanged.
	/// 3. **Associativity**: `bind(bind(m, f), g) ≡ bind(m, |x| bind(f(x), g))`:
	///    the order of nesting doesn't matter, only the order of operations.
	#[document_examples]
	///
	/// Monad laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	/// use fp_macros::m_do;
	///
	/// let f = |x: i32| Some(x + 1);
	/// let g = |x: i32| Some(x * 2);
	///
	/// // Left identity: bind(pure(a), f) ≡ f(a)
	/// assert_eq!(
	/// 	bind::<OptionBrand, _, _>(pure::<OptionBrand, _>(5), f),
	/// 	f(5),
	/// );
	/// // With m_do!: wrapping in pure then binding is the same as calling f
	/// assert_eq!(
	/// 	m_do!(OptionBrand { x <- pure(5); pure(x + 1) }),
	/// 	Some(6),
	/// );
	///
	/// // Right identity: bind(m, pure) ≡ m
	/// assert_eq!(
	/// 	bind::<OptionBrand, _, _>(Some(42), pure::<OptionBrand, _>),
	/// 	Some(42),
	/// );
	/// // With m_do!: extracting and re-wrapping is a no-op
	/// assert_eq!(
	/// 	m_do!(OptionBrand { x <- Some(42); pure(x) }),
	/// 	Some(42),
	/// );
	///
	/// // Associativity: bind(bind(m, f), g) ≡ bind(m, |x| bind(f(x), g))
	/// assert_eq!(
	/// 	bind::<OptionBrand, _, _>(
	/// 		bind::<OptionBrand, _, _>(Some(5), f),
	/// 		g,
	/// 	),
	/// 	bind::<OptionBrand, _, _>(Some(5), |x| bind::<OptionBrand, _, _>(f(x), g)),
	/// );
	/// // With m_do!: sequential binds compose naturally
	/// assert_eq!(
	/// 	m_do!(OptionBrand { x <- Some(5); y <- pure(x + 1); pure(y * 2) }),
	/// 	Some(12),
	/// );
	/// ```
	///
	/// Monad laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| vec![x, x + 1];
	/// let g = |x: i32| vec![x * 10];
	///
	/// // Left identity: bind(pure(a), f) ≡ f(a)
	/// assert_eq!(bind::<VecBrand, _, _>(pure::<VecBrand, _>(3), f), f(3),);
	///
	/// // Right identity: bind(m, pure) ≡ m
	/// assert_eq!(bind::<VecBrand, _, _>(vec![1, 2, 3], pure::<VecBrand, _>), vec![1, 2, 3],);
	///
	/// // Associativity: bind(bind(m, f), g) ≡ bind(m, |x| bind(f(x), g))
	/// let m = vec![1, 2];
	/// assert_eq!(
	/// 	bind::<VecBrand, _, _>(bind::<VecBrand, _, _>(m.clone(), f), g,),
	/// 	bind::<VecBrand, _, _>(m, |x| bind::<VecBrand, _, _>(f(x), g)),
	/// );
	/// ```
	pub trait Monad: Applicative + Semimonad {}

	/// Blanket implementation of [`Monad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}

	/// Executes a monadic action conditionally.
	///
	/// Evaluates the monadic boolean condition, then returns one of the two branches
	/// depending on the result. Both branches are provided as monadic values.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the monad.",
		"The type of the value produced by each branch."
	)]
	///
	#[document_parameters(
		"A monadic computation that produces a boolean.",
		"The computation to execute if the condition is `true`.",
		"The computation to execute if the condition is `false`."
	)]
	///
	#[document_returns("The result of the selected branch.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = if_m::<OptionBrand, _>(Some(true), Some(1), Some(0));
	/// assert_eq!(result, Some(1));
	///
	/// let result = if_m::<OptionBrand, _>(Some(false), Some(1), Some(0));
	/// assert_eq!(result, Some(0));
	///
	/// let result = if_m::<OptionBrand, i32>(None, Some(1), Some(0));
	/// assert_eq!(result, None);
	/// ```
	pub fn if_m<'a, Brand: Monad, A: 'a>(
		cond: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>),
		then_branch: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		else_branch: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
		Brand::bind(cond, move |c| if c { then_branch.clone() } else { else_branch.clone() })
	}

	/// Performs a monadic action when a monadic condition is true.
	///
	/// Evaluates the monadic boolean condition, then executes the action if the
	/// result is `true`, otherwise returns `pure(())`.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the computations.", "The brand of the monad.")]
	///
	#[document_parameters(
		"A monadic computation that produces a boolean.",
		"The action to perform if the condition is true."
	)]
	///
	#[document_returns("The action if the condition is true, otherwise `pure(())`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// assert_eq!(when_m::<OptionBrand>(Some(true), Some(())), Some(()));
	/// assert_eq!(when_m::<OptionBrand>(Some(false), Some(())), Some(()));
	/// assert_eq!(when_m::<OptionBrand>(None, Some(())), None);
	/// ```
	pub fn when_m<'a, Brand: Monad>(
		cond: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>),
		action: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Clone, {
		Brand::bind(cond, move |c| if c { action.clone() } else { Brand::pure(()) })
	}

	/// Performs a monadic action unless a monadic condition is true.
	///
	/// Evaluates the monadic boolean condition, then executes the action if the
	/// result is `false`, otherwise returns `pure(())`.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the computations.", "The brand of the monad.")]
	///
	#[document_parameters(
		"A monadic computation that produces a boolean.",
		"The action to perform if the condition is false."
	)]
	///
	#[document_returns("The action if the condition is false, otherwise `pure(())`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// assert_eq!(unless_m::<OptionBrand>(Some(false), Some(())), Some(()));
	/// assert_eq!(unless_m::<OptionBrand>(Some(true), Some(())), Some(()));
	/// assert_eq!(unless_m::<OptionBrand>(None, Some(())), None);
	/// ```
	pub fn unless_m<'a, Brand: Monad>(
		cond: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>),
		action: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Clone, {
		Brand::bind(cond, move |c| if !c { action.clone() } else { Brand::pure(()) })
	}
}

pub use inner::*;
