pub trait Kind<A> {
	type Output;
}

pub type Apply<Brand, A> = <Brand as Kind<A>>::Output;

pub trait Functor {
	/// forall f a b. Functor f => (a -> b) -> f a -> f b
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
		Self: Kind<A> + Kind<B>;
}

/// forall f a b. Functor f => (a -> b) -> f a -> f b
pub fn map<Brand, F, A, B>(f: F) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
where
	F: Fn(A) -> B + Copy,
	Brand: Kind<A> + Kind<B> + Functor,
{
	move |fa| Brand::map::<F, A, B>(f)(fa)
}

pub trait Sequence {
	/// forall f a b. Sequence f => f (a -> b) -> f a -> f b
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
		Self: Kind<F> + Kind<A> + Kind<B>;
}

/// forall f a b. Sequence f => f (a -> b) -> f a -> f b
pub fn sequence<Brand, F, A, B>(ff: Apply<Brand, F>) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
where
	Brand: Kind<F> + Kind<A> + Kind<B> + Sequence,
	F: Fn(A) -> B + Copy,
{
	let f = Brand::sequence::<F, A, B>(ff);
	move |fa| f(fa)
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Maybe<A> {
	Just(A),
	#[default]
	Nothing,
}

pub struct MaybeBrand;

impl<A> Kind<A> for MaybeBrand {
	type Output = Maybe<A>;
}

impl Sequence for MaybeBrand {
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
	{
		move |fa| match (ff, fa) {
			(Maybe::Just(f), Maybe::Just(a)) => Maybe::Just(f(a)),
			_ => Maybe::Nothing,
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Either<A, B> {
	Left(A),
	Right(B),
}

pub struct EitherAppliedLeftBrand<E>(E);

impl<E, A> Kind<A> for EitherAppliedLeftBrand<E> {
	type Output = Either<E, A>;
}

impl<E> Functor for EitherAppliedLeftBrand<E> {
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
	{
		move |fa| match fa {
			Either::Left(e) => Either::Left(e),
			Either::Right(a) => Either::Right(f(a)),
		}
	}
}

impl<E> Sequence for EitherAppliedLeftBrand<E>
where
	E: Clone,
{
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
	{
		move |fa| match (&ff, &fa) {
			(Either::Left(e), _) => Either::Left(e.clone()),
			(Either::Right(f), _) => map::<EitherAppliedLeftBrand<_>, _, _, _>(f)(fa),
		}
	}
}

fn main() {
	println!("{:?}", sequence::<MaybeBrand, _, _, _>(Maybe::Just(|x| x + 1))(Maybe::Just(0)));
	println!(
		"{:?}",
		sequence::<EitherAppliedLeftBrand<_>, _, _, _>(Either::Right::<(), _>(|x| x + 1))(
			Either::Right::<(), _>(0)
		)
	);
}
