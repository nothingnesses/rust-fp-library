use std::sync::Arc;

fn main() {}

pub trait Kind0 {
	type Output;
}
pub trait Kind1 {
	type Output<A>;
}
pub trait Kind2 {
	type Output<A, B>;
}
// pub trait Kind {
// 	type Output<Parameters>;
// }

pub type Apply0<Brand> = <Brand as Kind0>::Output;
pub type Apply1<Brand, A> = <Brand as Kind1>::Output<A>;
pub type Apply2<Brand, A, B> = <Brand as Kind2>::Output<A, B>;
// pub type Apply<Brand, Parameters> = <Brand as Kind>::Output<Parameters>;

// impl<Brand> Kind for Brand
// where
// 	Brand: Kind0,
// {
// 	type Output = Apply0<Brand>;
// }

// impl<Brand> Kind for Brand
// where
// 	Brand: Kind1,
// {
// 	type Output<A> = Apply1<Brand, A>;
// }

// pub trait Brand<Concrete, Parameters>: Kind<Parameters> {
// 	fn inject(a: Concrete) -> Self::Output;
// 	fn project(a: Self::Output) -> Concrete;
// }

pub type ArcFn<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;

pub trait Functor: Kind1 {
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solo<A>(pub A);

pub struct SoloBrand;

impl Kind1 for SoloBrand {
	type Output<A> = Solo<A>;
}

impl SoloBrand {
	fn inject<A>(a: Solo<A>) -> Apply1<Self, A> {
		a
	}

	fn project<A>(a: Apply1<Self, A>) -> Solo<A> {
		a
	}
}

impl Functor for SoloBrand {
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa| Self::inject(Solo(f(Self::project(fa).0))))
	}
}
