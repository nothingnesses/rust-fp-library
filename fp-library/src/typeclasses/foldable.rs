use crate::{
	functions::{compose, flip, identity},
	hkt::{Apply, Kind},
};

pub trait Foldable {
	// Evaluation steps should be:
	// ```
	// // Starting with z=0
	// let result = ((Self::fold_right(fold_step_composer))(identity))(t)(z);

	// // For t = [1,2,3], this becomes:
	// result = ((Self::fold_right(fold_step_composer, identity))(t))(z)

	// // Where fold_right builds:
	// // fold_step_composer(1, fold_step_composer(2, fold_step_composer(3, identity)))

	// // fold_step_composer(a, g) = compose(compose_flipped, f_flipped(a)) = f_flipped(a) . g
	// // So:
	// fold_step_composer(3, identity) = f_flipped(3)(identity(b)) = identity(b)*2 +3 = b*2 +3
	// fold_step_composer(2, g) = g(b*2 +2)
	// fold_step_composer(1, g) = g(b*2 +1)

	// // Final application:
	// result = ((b*2 +1)*2 +2)*2 +3
	// result(0) = ((0*2 +1)*2 +2)*2 +3 = (1*2 +2)*2 +3 = (4)*2 +3 = 11
	// ```
	fn fold_left<A, B>(
		f: impl Fn(B) -> Box<dyn Fn(A) -> B> + Clone
	) -> impl Fn(B) -> Box<dyn Fn(Apply<Self, (A,)>) -> B>
	where
		Self: Kind<(A,)>,
		A: Clone
	{
		// (a, b) -> c
		// flip f = f: ((a -> b) -> c) -> (b -> a -> c)
		// compose f g = f: (b -> c) -> g: (a -> b) -> (a -> c)
		// flip compose = (a -> b) -> (b -> c) -> (a -> c)
		let compose_flipped = flip(compose);
		// f = b -> a -> b
		// flip f = a -> b -> b
		let f_flipped = flip(f);
		// compose compose_flipped f_flipped = ?
		/*
		a (this was f) = (b, c) -> b
		d (this was flip) = ((e, f) -> g) -> ((f, e) -> g)
		h (this was compose) = (i: (k -> l), j: (l -> m)) -> ji: (m -> k)
		dh (this was compose_flipped) = (j, i) -> ji
		da (this was f_flipped) = (c, b) -> b
		hdhda (this was compose compose_flipped f_flipped) = (dh, da) -> dadh
		bji: (b, (m -> k)) -> z
		 */
		let fold_step_composer = compose(compose_flipped)(Box::new(f_flipped));
		move |z| {
			Box::new(move |t| {
				(((Self::fold_right(fold_step_composer))(identity))(t))(z)
			})
		}
	}

	fn fold_right<A, B>(
		f: impl Fn(A) -> Box<dyn Fn(B) -> B>
	) -> impl Fn(B) -> Box<dyn Fn(Apply<Self, (A,)>) -> B>
	where
		Self: Kind<(A,)>;
}
