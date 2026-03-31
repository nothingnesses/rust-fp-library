# Feasibility: HKT Brand for CoyonedaExplicit

## 1. A brand for the existing boxed-function CoyonedaExplicit is feasible

The encoding is `CoyonedaExplicitBrand<F, B>` with `Of<'a, A> = CoyonedaExplicit<'a, F, B, A>`.
Once `F` and `B` are fixed by the brand, the type has the right shape `('a, A) -> Type`
for `Kind_cdc7cd43dac7585f`. Both `F` and `B` would need `'static` bounds, which follows
the same pattern as `TryLazyBrand<E, Config>` and `CoyonedaBrand<F>`.

## 2. Functor impl preserves fusion

The Functor impl would delegate to `CoyonedaExplicit::map`, which composes the new
function with the stored `Box<dyn Fn(B) -> A + 'a>`. The underlying `F::Of<'a, B>` is
never touched until `lower`. So `map::<CoyonedaExplicitBrand<VecBrand, i32>, _, _>(f, coyo)`
composes functions; only one `F::map` call happens at `lower` time. Fusion survives.

## 3. The generic-function variant cannot have a brand

If `CoyonedaExplicit` were changed to `CoyonedaExplicit<'a, F, B, A, Func>` with
`Func: Fn(B) -> A + 'a`, no brand is possible. `Kind::Of<'a, A>` must be a single
concrete type for each `A`, but `Func` changes with every `map` call (each composition
produces a new closure type). There is no way to fill in `Func` in the associated type
definition.

## 4. B is fixed in the brand, which is correct

`B` being invariant across maps is exactly what makes fusion work. The trade-off is that
generic code must know `B`, but this is not a practical problem since the user always
knows `B` at construction time.

## 5. Comparison with CoyonedaBrand

`CoyonedaExplicitBrand<F, B>` would be strictly better than `CoyonedaBrand<F>` for
fusion (1 call to `F::map` vs k calls), with 1 box per map instead of 2. The cost is
carrying `B` in the brand.
