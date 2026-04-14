### Val/Ref Dispatch

The library has two parallel type class hierarchies: a **by-value** hierarchy
(`Functor`, `Semimonad`, `Foldable`, etc.) where closures receive owned values,
and a **by-reference** hierarchy (`RefFunctor`, `RefSemimonad`, `RefFoldable`,
etc.) where closures receive borrowed references. This split exists because
memoized types like `Lazy` can only lend references to their cached values,
not give up ownership (see [Limitations](./limitations-and-workarounds.md)).

Rather than exposing two separate functions per operation (`map` and `ref_map`),
the dispatch system provides a **single unified function** that routes to the
correct trait based on the closure's argument type:

```rust
use fp_library::functions::*;

// Closure takes i32 (owned) -> dispatches to Functor::map
let y = map(|x: i32| x * 2, Some(5));
assert_eq!(y, Some(10));

// Closure takes &i32 (borrowed) -> dispatches to RefFunctor::ref_map
let v = vec![1, 2, 3];
let y = map(|x: &i32| *x + 10, &v);
assert_eq!(y, vec![11, 12, 13]);
```

#### How it works

Each operation has a **dispatch trait** with two blanket impls selected by a
marker type (`Val` or `Ref`). The compiler resolves the marker from the
closure's argument type.

Using `map` as an example:

```rust,ignore
// The dispatch trait (simplified)
trait FunctorDispatch<Brand, A, B, FA, Marker> {
	fn dispatch(self, fa: FA) -> Brand::Of<B>;
}

// Val impl: closure takes owned A, container is owned
impl<Brand: Functor, A, B, F: Fn(A) -> B>
	FunctorDispatch<Brand, A, B, Brand::Of<A>, Val> for F
{
	fn dispatch(self, fa: Brand::Of<A>) -> Brand::Of<B> {
		Brand::map(self, fa)  // delegates to Functor::map
	}
}

// Ref impl: closure takes &A, container is borrowed
impl<Brand: RefFunctor, A, B, F: Fn(&A) -> B>
	FunctorDispatch<Brand, A, B, &Brand::Of<A>, Ref> for F
{
	fn dispatch(self, fa: &Brand::Of<A>) -> Brand::Of<B> {
		Brand::ref_map(self, fa)  // delegates to RefFunctor::ref_map
	}
}
```

When the caller writes `map(|x: i32| x * 2, Some(5))`:

1. The closure type `Fn(i32) -> i32` matches the Val impl (takes owned `A`).
2. The compiler infers `Marker = Val` and `FA = Option<i32>`.
3. `dispatch` delegates to `Functor::map`.

When the caller writes `map(|x: &i32| *x + 10, &v)`:

1. The closure type `Fn(&i32) -> i32` matches the Ref impl (takes `&A`).
2. The compiler infers `Marker = Ref` and `FA = &Vec<i32>`.
3. `dispatch` delegates to `RefFunctor::ref_map`.

The `FA` type parameter is key: it appears in both the dispatch trait (to
constrain the container) and in `InferableBrand` (to resolve the brand).
This is how dispatch and brand inference compose through a single type variable.
See [Brand Inference](./brand-inference.md) for how the reverse mapping from
concrete types to brands works.

#### Closureless dispatch

Functions that take no closure (`alt`, `compact`, `separate`, `join`,
`apply_first`, `apply_second`) use a variant where the **container type**
itself drives dispatch instead of a closure's argument type. Owned containers
resolve to `Val`, borrowed containers resolve to `Ref`:

```rust
use fp_library::functions::*;

// Owned containers -> Alt::alt
let y = alt(None, Some(5));
assert_eq!(y, Some(5));

// Borrowed containers -> RefAlt::ref_alt
let a = vec![1, 2];
let b = vec![3, 4];
let y = alt(&a, &b);
assert_eq!(y, vec![1, 2, 3, 4]);
```

#### Module structure

The dispatch system lives in `fp-library/src/dispatch/`, with one file per
type class operation mirroring `classes/`. Each dispatch module contains
the dispatch trait, Val/Ref impl blocks, the inference wrapper function,
and an `explicit` submodule with the brand-explicit variant:

```text
classes/functor.rs      -> Functor trait (by-value map)
classes/ref_functor.rs  -> RefFunctor trait (by-ref map)
dispatch/functor.rs     -> pub(crate) mod inner {
                              FunctorDispatch trait,
                              Val impl (Fn(A) -> B -> Functor::map),
                              Ref impl (Fn(&A) -> B -> RefFunctor::ref_map),
                              pub fn map (inference wrapper),
                              pub mod explicit { pub fn map (Brand turbofish) },
                           }
functions.rs            -> Re-exports: map (inference), explicit::map (dispatch)
```

The `functions.rs` module re-exports inference wrappers from
`crate::dispatch::*` and explicit functions from
`crate::dispatch::*/explicit::*`. There are no intermediate
`functions/*.rs` source files.

#### Relationship to thread safety and parallelism

The Val/Ref split is orthogonal to thread safety. The library has separate
`Send*` and `Par*` trait hierarchies that add `Send + Sync` bounds for
concurrent use. These axes combine independently: a type can implement
`RefFunctor` (by-ref, thread-local), `SendRefFunctor` (by-ref, thread-safe),
`ParRefFunctor` (by-ref, parallel), etc. See [parallelism.md](./parallelism.md)
for details on the thread-safe and parallel trait hierarchies.
