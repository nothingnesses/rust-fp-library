# Type Class Laws Checklist

This checklist tracks the implementation of property-based tests for type class laws.

## Functor Laws

- [x] **Identity**: `map(identity, fa) == fa`
- [x] **Composition**: `map(compose(f, g), fa) == map(f, map(g, fa))`

## Applicative Laws

- [x] **Identity**: `apply(pure(identity), v) == v`
- [x] **Composition**: `apply(apply(apply(pure(compose), u), v), w) == apply(u, apply(v, w))`
- [x] **Homomorphism**: `apply(pure(f), pure(x)) == pure(f(x))`
- [x] **Interchange**: `apply(u, pure(y)) == apply(pure(|f| f(y)), u)`

## Monad Laws

- [x] **Left Identity**: `bind(pure(a), f) == f(a)`
- [x] **Right Identity**: `bind(m, pure) == m`
- [x] **Associativity**: `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`

## Semigroup Laws

- [x] **Associativity**: `append(a, append(b, c)) == append(append(a, b), c)`

## Monoid Laws

- [x] **Left Identity**: `append(empty(), a) == a`
- [x] **Right Identity**: `append(a, empty()) == a`

## Semigroupoid Laws

- [x] **Associativity**: `compose(f, compose(g, h)) == compose(compose(f, g), h)`

## Category Laws

- [x] **Left Identity**: `compose(identity(), f) == f`
- [x] **Right Identity**: `compose(f, identity()) == f`

## Verified Types

- [x] Option
- [x] Vec
- [x] Identity
- [x] Result
- [x] Pair
- [x] Endomorphism
- [x] Endofunction
