failures:

---- fp-library/src/types/optics/composed.rs - types::optics::composed::inner::Composed<'a,S,S,M,M,A,A,O1,O2>::evaluate (line 47) stdout ----
error[E0277]: the trait bound `(usize, i32): Monoid` is not satisfied
   --> fp-library/src/types/optics/composed.rs:71:64
    |
 27 | > as IndexedFoldOptic<usize, (i32, String), i32>>::evaluate::< (usize, i32), RcBrand >(&composed, Indexed::new(f));
    |                                                                ^^^^^^^^^^^^ the trait `Monoid` is not implemented for `(usize, i32)`
    |
    = help: the following other types implement trait `Monoid`:
              CatList<A>
              Endofunction<'a, FnBrand, A>
              Endomorphism<'a, C, A>
              SendEndofunction<'a, FnBrand, A>
              String
              Thunk<'a, A>
              TryThunk<'a, A, E>
              Vec<A>
note: required by a bound in `fp_library::classes::IndexedFoldOptic::evaluate`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:484:22
    |
484 |     fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
    |                         ^^^^^^ required by this bound in `IndexedFoldOptic::evaluate`

error[E0271]: type mismatch resolving `<ForgetBrand<..., ...> as Kind_266801a817966495>::Of<'_, ..., i32> == Forget<'_, ..., ..., ..., ...>`
  --> fp-library/src/types/optics/composed.rs:71:99
   |
27 | > as IndexedFoldOptic<usize, (i32, String), i32>>::evaluate::< (usize, i32), RcBrand >(&composed, Indexed::new(f));
   |                                                                                                   ^^^^^^^^^^^^^^^ expected `Forget<'_, RcBrand, (usize, i32), ..., ...>`, found `Forget<'_, RcBrand, (usize, i32), ..., i32>`
   |
   = note: expected struct `fp_library::types::optics::Forget<'_, _, _, _, (usize, i32)>`
              found struct `fp_library::types::optics::Forget<'_, _, _, _, i32>`
   = note: the full name for the type has been written to '/tmp/rustdoctesthcK3Gd/rust_out.long-type-11778204997242526513.txt'
   = note: consider using `--verbose` to print the full type name to the console

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0271, E0277.
For more information about an error, try `rustc --explain E0271`.
Couldn't compile the test.
---- fp-library/src/types/optics/composed.rs - types::optics::composed::inner::Composed<'a,S,S,M,M,A,A,O1,O2>::evaluate (line 47) stdout ----
error[E0271]: type mismatch resolving `<ForgetBrand<..., ...> as Kind_266801a817966495>::Of<'_, ..., i32> == Forget<'_, ..., ..., ..., ...>`
  --> fp-library/src/types/optics/composed.rs:71:101
   |
27 | > as IndexedGetterOptic<usize, (i32, String), i32>>::evaluate::< (usize, i32), RcBrand >(&composed, Indexed::new(f));
   |                                                                                                     ^^^^^^^^^^^^^^^ expected `Forget<'_, RcBrand, (usize, i32), ..., ...>`, found `Forget<'_, RcBrand, (usize, i32), ..., i32>`
   |
   = note: expected struct `fp_library::types::optics::Forget<'_, _, _, _, (usize, i32)>`
              found struct `fp_library::types::optics::Forget<'_, _, _, _, i32>`
   = note: the full name for the type has been written to '/tmp/rustdoctestoPNObe/rust_out.long-type-17259431172369162762.txt'
   = note: consider using `--verbose` to print the full type name to the console

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0271`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_as_index (line 815) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/functions.rs:823:16
    |
 11 | let as_index = optics_as_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                |
    |                expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/functions.rs:824:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&as_index, (42, "hi".to_string())), 10);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_as_index::AsIndex<'a,P,O,I,S,T,A,B>::evaluate (line 840) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/functions.rs:848:16
    |
 11 | let as_index = optics_as_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                |
    |                expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/functions.rs:849:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&as_index, (42, "hi".to_string())), 10);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::PositionsTraversalFunc<F>::apply (line 14) stdout ----
error[E0433]: failed to resolve: use of undeclared type `IndexedTraversalFunc`
  --> fp-library/src/types/optics/functions.rs:27:32
   |
16 | let result: Option<Vec<i32>> = IndexedTraversalFunc::apply::<OptionBrand, _>(&p, f, s);
   |                                ^^^^^^^^^^^^^^^^^^^^ use of undeclared type `IndexedTraversalFunc`
   |
help: a struct with a similar name exists
   |
16 - let result: Option<Vec<i32>> = IndexedTraversalFunc::apply::<OptionBrand, _>(&p, f, s);
16 + let result: Option<Vec<i32>> = IndexedTraversal::apply::<OptionBrand, _>(&p, f, s);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::optics::IndexedTraversalFunc;
   |

error[E0599]: no function or associated item named `traversed` found for struct `fp_library::types::optics::Traversal<'a, P, S, T, A, B, F>` in the current scope
   --> fp-library/src/types/optics/functions.rs:21:64
    |
 10 | let t = Traversal::<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::traversed::<VecBrand>();
    |                                                                ^^^^^^^^^ function or associated item not found in `fp_library::types::optics::Traversal<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>`
    |
note: if you're trying to build a new `fp_library::types::optics::Traversal<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>`, consider using `fp_library::types::optics::Traversal::<'a, P, S, T, A, B, F>::new` which returns `fp_library::types::optics::Traversal<'_, _, _, _, _, _, _>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/traversal.rs:119:3
    |
119 |         pub fn new(traversal: F) -> Self {
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_reindexed (line 899) stdout ----
error[E0107]: function takes 9 generic arguments but 10 generic arguments were supplied
   --> fp-library/src/types/optics/functions.rs:907:17
    |
 11 | let reindexed = optics_reindexed::<RcBrand, _, _, _, String, _, _, _, _, _>(|i: usize| format!("{}", i), &l);
    |                 ^^^^^^^^^^^^^^^^ expected 9 generic arguments          --- help: remove the unnecessary generic argument
    |
note: function defined here, with 9 generic parameters: `P`, `O`, `I`, `J`, `S`, `T`, `A`, `B`, `F`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:889:9
    |
889 |     pub fn optics_reindexed<'a, P, O, I, J, S, T, A, B, F>(
    |            ^^^^^^^^^^^^^^^^     -  -  -  -  -  -  -  -  -

error[E0277]: the trait bound `impl IndexedOpticAdapter<'_, _, String, (i32, String), (i32, String), i32, i32> + '_: IndexedGetterOptic<'_, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/functions.rs:908:43
    |
 12 | assert_eq!(optics_indexed_view::<RcBrand, _, _, _, _>(&reindexed, (42, "hi".to_string())), ("0".to_string(), 42));
    |                                           ^ the trait `IndexedGetterOptic<'_, _, _, _>` is not implemented for `impl IndexedOpticAdapter<'_, _, String, (i32, String), (i32, String), i32, i32> + '_`
    |
help: the following other types implement trait `IndexedGetterOptic<'a, I, S, A>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_getter.rs:138:2
    |
138 | /     impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A> for IndexedGetter<'a, P, I, S, A>
139 | |     where
140 | |         P: UnsizedCoercible,
    | |____________________________^ `IndexedGetter<'a, P, I, S, A>`
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_lens.rs:466:2
    |
466 | /     impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A>
467 | |         for IndexedLens<'a, P, I, S, S, A, A>
468 | |     where
469 | |         P: UnsizedCoercible,
    | |____________________________^ `IndexedLens<'a, P, I, S, S, A, A>`
...
849 | /     impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A>
850 | |         for IndexedLensPrime<'a, P, I, S, A>
851 | |     where
852 | |         P: UnsizedCoercible,
    | |____________________________^ `fp_library::types::optics::IndexedLensPrime<'a, P, I, S, A>`
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/composed.rs:976:2
    |
976 | /     impl<'a, I: 'a, S: 'a, A: 'a, M: 'a, O1, O2> IndexedGetterOptic<'a, I, S, A>
977 | |         for Composed<'a, S, S, M, M, A, A, O1, O2>
978 | |     where
979 | |         O1: GetterOptic<'a, S, M>,
980 | |         O2: IndexedGetterOptic<'a, I, M, A>,
    | |____________________________________________^ `Composed<'a, S, S, M, M, A, A, O1, O2>`
note: required by a bound in `fp_library::functions::optics_indexed_view`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:518:6
    |
512 |     pub fn optics_indexed_view<'a, P, O, I, S, A>(
    |            ------------------- required by a bound in this function
...
518 |         O: IndexedGetterOptic<'a, I, S, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_indexed_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_reindexed::Reindexed<'a,P,O,I,J,S,T,A,B,F>::evaluate_indexed (line 928) stdout ----
error[E0107]: function takes 9 generic arguments but 10 generic arguments were supplied
   --> fp-library/src/types/optics/functions.rs:936:17
    |
 11 | let reindexed = optics_reindexed::<RcBrand, _, _, String, String, _, _, _, _, _>(|i: usize| format!("{}", i), &l);
    |                 ^^^^^^^^^^^^^^^^ expected 9 generic arguments               --- help: remove the unnecessary generic argument
    |
note: function defined here, with 9 generic parameters: `P`, `O`, `I`, `J`, `S`, `T`, `A`, `B`, `F`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:889:9
    |
889 |     pub fn optics_reindexed<'a, P, O, I, J, S, T, A, B, F>(
    |            ^^^^^^^^^^^^^^^^     -  -  -  -  -  -  -  -  -

error[E0277]: the trait bound `impl IndexedOpticAdapter<'_, _, String, (i32, String), (i32, String), i32, i32> + '_: IndexedGetterOptic<'_, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/functions.rs:937:43
    |
 12 | assert_eq!(optics_indexed_view::<RcBrand, _, _, _, _>(&reindexed, (42, "hi".to_string())), ("0".to_string(), 42));
    |                                           ^ the trait `IndexedGetterOptic<'_, _, _, _>` is not implemented for `impl IndexedOpticAdapter<'_, _, String, (i32, String), (i32, String), i32, i32> + '_`
    |
help: the following other types implement trait `IndexedGetterOptic<'a, I, S, A>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_getter.rs:138:2
    |
138 | /     impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A> for IndexedGetter<'a, P, I, S, A>
139 | |     where
140 | |         P: UnsizedCoercible,
    | |____________________________^ `IndexedGetter<'a, P, I, S, A>`
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_lens.rs:466:2
    |
466 | /     impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A>
467 | |         for IndexedLens<'a, P, I, S, S, A, A>
468 | |     where
469 | |         P: UnsizedCoercible,
    | |____________________________^ `IndexedLens<'a, P, I, S, S, A, A>`
...
849 | /     impl<'a, P, I: 'a, S: 'a, A: 'a> IndexedGetterOptic<'a, I, S, A>
850 | |         for IndexedLensPrime<'a, P, I, S, A>
851 | |     where
852 | |         P: UnsizedCoercible,
    | |____________________________^ `fp_library::types::optics::IndexedLensPrime<'a, P, I, S, A>`
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/composed.rs:976:2
    |
976 | /     impl<'a, I: 'a, S: 'a, A: 'a, M: 'a, O1, O2> IndexedGetterOptic<'a, I, S, A>
977 | |         for Composed<'a, S, S, M, M, A, A, O1, O2>
978 | |     where
979 | |         O1: GetterOptic<'a, S, M>,
980 | |         O2: IndexedGetterOptic<'a, I, M, A>,
    | |____________________________________________^ `Composed<'a, S, S, M, M, A, A, O1, O2>`
note: required by a bound in `fp_library::functions::optics_indexed_view`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:518:6
    |
512 |     pub fn optics_indexed_view<'a, P, O, I, S, A>(
    |            ------------------- required by a bound in this function
...
518 |         O: IndexedGetterOptic<'a, I, S, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_indexed_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_un_index (line 740) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/functions.rs:748:17
    |
 11 | let unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                 |
    |                 expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/functions.rs:749:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&unindexed, (42, "hi".to_string())), 42);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_un_index::UnIndex<'a,P,O,I,S,T,A,B>::evaluate (line 765) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/functions.rs:773:17
    |
 11 | let unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                 |
    |                 expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/functions.rs:774:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&unindexed, (42, "hi".to_string())), 42);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/functions.rs - types::optics::functions::inner::positions (line 1036) stdout ----
error[E0599]: no function or associated item named `traversed` found for struct `fp_library::types::optics::Traversal<'a, P, S, T, A, B, F>` in the current scope
   --> fp-library/src/types/optics/functions.rs:1043:64
    |
 10 | let t = Traversal::<RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::traversed::<VecBrand>();
    |                                                                ^^^^^^^^^ function or associated item not found in `fp_library::types::optics::Traversal<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>`
    |
note: if you're trying to build a new `fp_library::types::optics::Traversal<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>`, consider using `fp_library::types::optics::Traversal::<'a, P, S, T, A, B, F>::new` which returns `fp_library::types::optics::Traversal<'_, _, _, _, _, _, _>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/traversal.rs:119:3
    |
119 |         pub fn new(traversal: F) -> Self {
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0599`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,A,A,Folded<Brand>>::folded (line 234) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_fold.rs:242:15
    |
 11 |     IndexedFold::folded::<VecBrand>();
    |                  ^^^^^^------------ help: remove the unnecessary generics
    |                  |
    |                  expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_fold.rs:241:10
    |
241 |         pub fn folded() -> Self {
    |                ^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed.rs - types::optics::indexed::inner::IndexedBrand<P,I>::dimap (line 22) stdout ----
Test executable failed (exit status: 101).

stderr:

thread 'main' (488521) panicked at fp-library/src/types/optics/indexed.rs:17:1:
assertion `left == right` failed
  left: 41
 right: 25
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


---- fp-library/src/types/optics/indexed.rs - types::optics::indexed::inner::IndexedBrand<P,I>::wander (line 20) stdout ----
error: cannot find macro `Apply` in this scope
  --> fp-library/src/types/optics/indexed.rs:30:134
   |
13 |     fn apply<'b, M: Applicative>(&self, f: Box<dyn Fn(A) -> Apply!(<M as Kind!(type Of<'c, T: 'c>: 'c;)>::Of<'a, B>) + 'a>, s: A) -> Apply!(<M as Kind!(type Of<'c, T: 'c>:...
   |                                                                                                                                      ^^^^^
   |
help: consider importing one of these macros
   |
 2 + use fp_library::Apply;
   |
 2 + use fp_macros::Apply;
   |

error: cannot find macro `Apply` in this scope
  --> fp-library/src/types/optics/indexed.rs:30:61
   |
13 |     fn apply<'b, M: Applicative>(&self, f: Box<dyn Fn(A) -> Apply!(<M as Kind!(type Of<'c, T: 'c>: 'c;)>::Of<'a, B>) + 'a>, s: A) -> Apply!(<M as Kind!(type Of<'c, T: 'c>:...
   |                                                             ^^^^^
   |
help: consider importing one of these macros
   |
 2 + use fp_library::Apply;
   |
 2 + use fp_macros::Apply;
   |

error[E0405]: cannot find trait `Applicative` in this scope
  --> fp-library/src/types/optics/indexed.rs:30:21
   |
13 |     fn apply<'b, M: Applicative>(&self, f: Box<dyn Fn(A) -> Apply!(<M as Kind!(type Of<'c, T: 'c>: 'c;)>::Of<'a, B>) + 'a>, s: A) -> Apply!(<M as Kind!(type Of<'c, T: 'c>:...
   |                     ^^^^^^^^^^^ not found in this scope
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::Applicative;
   |

error: aborting due to 3 previous errors

For more information about this error, try `rustc --explain E0405`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::evaluate_indexed (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_fold.rs:18:73
    |
 10 | let l = IndexedFold::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _>::folded::<VecBrand>();
    |                                                                         ^^^^^^------------ help: remove the unnecessary generics
    |                                                                         |
    |                                                                         expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_fold.rs:241:10
    |
241 |         pub fn folded() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_fold.rs:19:18
    |
 11 | let _unindexed = optics_un_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _, _>(&l);
    |                  ^^^^^^^^^^^^^^^ expected 7 generic arguments                    --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_fold.rs:18:73
    |
 10 | let l = IndexedFold::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _>::folded::<VecBrand>();
    |                                                                         ^^^^^^------------ help: remove the unnecessary generics
    |                                                                         |
    |                                                                         expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_fold.rs:241:10
    |
241 |         pub fn folded() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_fold.rs:19:18
    |
 11 | let _unindexed = optics_as_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _, _>(&l);
    |                  ^^^^^^^^^^^^^^^ expected 7 generic arguments                    --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::Folded<Brand>::apply (line 14) stdout ----
error[E0432]: unresolved import `fp_library::classes::optics::indexed_fold`
 --> fp-library/src/types/optics/indexed_fold.rs:19:19
  |
8 |     classes::optics::indexed_fold::IndexedFoldFunc,
  |                      ^^^^^^^^^^^^ could not find `indexed_fold` in `optics`

error[E0603]: module `indexed_fold` is private
  --> fp-library/src/types/optics/indexed_fold.rs:18:17
   |
 7 |     types::optics::indexed_fold::Folded,
   |                    ^^^^^^^^^^^^ private module
   |
note: the module `indexed_fold` is defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics.rs:93:1
   |
93 | mod indexed_fold;
   | ^^^^^^^^^^^^^^^^

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0432, E0603.
For more information about an error, try `rustc --explain E0432`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,A,
Folded<Brand>>::folded (line 276) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_fold.rs:284:20
    |
 11 |     IndexedFoldPrime::folded::<VecBrand>();
    |                       ^^^^^^------------ help: remove the unnecessary generics
    |                       |
    |                       expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_fold.rs:283:10
    |
283 |         pub fn folded() -> Self {
    |                ^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::evaluate_indexed (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_fold.rs:18:63
    |
 10 | let l = IndexedFoldPrime::<RcBrand, usize, Vec<i32>, i32, _>::folded::<VecBrand>();
    |                                                               ^^^^^^------------ help: remove the unnecessary generics
    |                                                               |
    |                                                               expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_fold.rs:283:10
    |
283 |         pub fn folded() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_fold.rs:19:18
    |
 11 | let _unindexed = optics_un_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _, _>(&l);
    |                  ^^^^^^^^^^^^^^^ expected 7 generic arguments                    --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_fold.rs:18:63
    |
 10 | let l = IndexedFoldPrime::<RcBrand, usize, Vec<i32>, i32, _>::folded::<VecBrand>();
    |                                                               ^^^^^^------------ help: remove the unnecessary generics
    |                                                               |
    |                                                               expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_fold.rs:283:10
    |
283 |         pub fn folded() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_fold.rs:19:18
    |
 11 | let _unindexed = optics_as_index::<ForgetBrand<RcBrand, String>, _, _, _, _, _, _, _>(&l);
    |                  ^^^^^^^^^^^^^^^ expected 7 generic arguments                    --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_getter.rs - types::optics::indexed_getter::inner::IndexedGetter<'a,P,I,S,A>::evaluate (line 14) stdout ----
error[E0277]: the trait bound `i32: Monoid` is not satisfied
   --> fp-library/src/types/optics/indexed_getter.rs:24:43
    |
 13 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&g, pab);
    |                                           ^^^ the trait `Monoid` is not implemented for `i32`
    |
    = help: the following other types implement trait `Monoid`:
              CatList<A>
              Endofunction<'a, FnBrand, A>
              Endomorphism<'a, C, A>
              SendEndofunction<'a, FnBrand, A>
              String
              Thunk<'a, A>
              TryThunk<'a, A, E>
              Vec<A>
note: required by a bound in `fp_library::classes::IndexedFoldOptic::evaluate`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:484:22
    |
484 |     fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
    |                         ^^^^^^ required by this bound in `IndexedFoldOptic::evaluate`

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0277`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::evaluate (line 14) stdout ----
error[E0433]: failed to resolve: use of undeclared type `IndexedFoldOptic`
  --> fp-library/src/types/optics/indexed_fold.rs:35:14
   |
24 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
   |              ^^^^^^^^^^^^^^^^ use of undeclared type `IndexedFoldOptic`
   |
help: a trait with a similar name exists
   |
24 - let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
24 + let result = IndexedFoldFunc::evaluate::<i32, RcBrand>(&l, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedFoldOptic;
   |

error[E0599]: no method named `append` found for type parameter `R` in the current scope
  --> fp-library/src/types/optics/indexed_fold.rs:28:64
   |
12 |     fn apply<R: 'a + Monoid + 'static>(
   |              - method `append` not found for this type parameter
...
17 |         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
   |                                                                      ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
note: the candidate is defined in the trait `Semigroup`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/semigroup.rs:45:2
   |
45 | /     fn append(
46 | |         a: Self,
47 | |         b: Self,
48 | |     ) -> Self;
   | |______________^
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: use associated function syntax instead
   |
17 -         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
17 +         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| R::append(acc, f(i, x)))
   |
help: the following trait defines an item `append`, perhaps you need to restrict type parameter `R` with it:
   |
12 |     fn apply<R: 'a + Monoid + 'static + winnow::error::ParserError</* I */>>(
   |                                       +++++++++++++++++++++++++++++++++++++

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_getter.rs - types::optics::indexed_getter::inner::IndexedGetter<'a,P,I,S,A>::evaluate_indexed (line 11) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_getter.rs:19:17
    |
 11 | let unindexed = optics_un_index::<ForgetBrand<RcBrand, i32>, _, _, _, _, _, _, _>(&g);
    |                 ^^^^^^^^^^^^^^^ expected 7 generic arguments                 --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, ForgetBrand<_, _>, (i32, String), ..., i32, i32>: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/indexed_getter.rs:20:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&unindexed, (42, "hi".to_string())), 42);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, fp_library::types::optics::ForgetBrand<_, _>, (i32, String), (i32, String), i32, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`
   = note: the full name for the type has been written to '/tmp/rustdoctestsWEglC/rust_out.long-type-2599521086947293396.txt'
   = note: consider using `--verbose` to print the full type name to the console

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::clone (line 8) stdout ----
error[E0433]: failed to resolve: use of undeclared type `IndexedFoldOptic`
  --> fp-library/src/types/optics/indexed_fold.rs:31:14
   |
26 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&cloned, pab);
   |              ^^^^^^^^^^^^^^^^ use of undeclared type `IndexedFoldOptic`
   |
help: a trait with a similar name exists
   |
26 - let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&cloned, pab);
26 + let result = IndexedFoldFunc::evaluate::<i32, RcBrand>(&cloned, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedFoldOptic;
   |

error[E0599]: no method named `append` found for type parameter `R` in the current scope
  --> fp-library/src/types/optics/indexed_fold.rs:23:64
   |
13 |     fn apply<R: 'a + Monoid + 'static>(
   |              - method `append` not found for this type parameter
...
18 |         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
   |                                                                      ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
note: the candidate is defined in the trait `Semigroup`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/semigroup.rs:45:2
   |
45 | /     fn append(
46 | |         a: Self,
47 | |         b: Self,
48 | |     ) -> Self;
   | |______________^
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: use associated function syntax instead
   |
18 -         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
18 +         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| R::append(acc, f(i, x)))
   |
help: the following trait defines an item `append`, perhaps you need to restrict type parameter `R` with it:
   |
13 |     fn apply<R: 'a + Monoid + 'static + winnow::error::ParserError</* I */>>(
   |                                       +++++++++++++++++++++++++++++++++++++

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_getter.rs - types::optics::indexed_getter::inner::IndexedGetter<'a,P,I,S,A>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_getter.rs:19:17
    |
 11 | let unindexed = optics_as_index::<ForgetBrand<RcBrand, usize>, _, _, _, _, _, _, _>(&g);
    |                 ^^^^^^^^^^^^^^^ expected 7 generic arguments                   --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, ForgetBrand<_, _>, (i32, String), ..., usize, i32>: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/indexed_getter.rs:20:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&unindexed, (42, "hi".to_string())), 10);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, fp_library::types::optics::ForgetBrand<_, _>, (i32, String), (i32, String), usize, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`
   = note: the full name for the type has been written to '/tmp/rustdoctestkCb8gB/rust_out.long-type-2227677036811565214.txt'
   = note: consider using `--verbose` to print the full type name to the console

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::new (line 136) stdout ----
error[E0433]: failed to resolve: use of undeclared type `IndexedFoldOptic`
  --> fp-library/src/types/optics/indexed_fold.rs:157:14
   |
24 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
   |              ^^^^^^^^^^^^^^^^ use of undeclared type `IndexedFoldOptic`
   |
help: a trait with a similar name exists
   |
24 - let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
24 + let result = IndexedFoldFunc::evaluate::<i32, RcBrand>(&l, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedFoldOptic;
   |

error[E0599]: no method named `append` found for type parameter `R` in the current scope
  --> fp-library/src/types/optics/indexed_fold.rs:150:64
   |
12 |     fn apply<R: 'a + Monoid + 'static>(
   |              - method `append` not found for this type parameter
...
17 |         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
   |                                                                      ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
note: the candidate is defined in the trait `Semigroup`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/semigroup.rs:45:2
   |
45 | /     fn append(
46 | |         a: Self,
47 | |         b: Self,
48 | |     ) -> Self;
   | |______________^
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: use associated function syntax instead
   |
17 -         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
17 +         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| R::append(acc, f(i, x)))
   |
help: the following trait defines an item `append`, perhaps you need to restrict type parameter `R` with it:
   |
12 |     fn apply<R: 'a + Monoid + 'static + winnow::error::ParserError</* I */>>(
   |                                       +++++++++++++++++++++++++++++++++++++

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::evaluate (line 14) stdout ----
error[E0433]: failed to resolve: use of undeclared type `IndexedFoldOptic`
  --> fp-library/src/types/optics/indexed_fold.rs:35:14
   |
24 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
   |              ^^^^^^^^^^^^^^^^ use of undeclared type `IndexedFoldOptic`
   |
help: a trait with a similar name exists
   |
24 - let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
24 + let result = IndexedFoldFunc::evaluate::<i32, RcBrand>(&l, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedFoldOptic;
   |

error[E0599]: no method named `append` found for type parameter `R` in the current scope
  --> fp-library/src/types/optics/indexed_fold.rs:28:64
   |
12 |     fn apply<R: 'a + Monoid + 'static>(
   |              - method `append` not found for this type parameter
...
17 |         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
   |                                                                      ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
note: the candidate is defined in the trait `Semigroup`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/semigroup.rs:45:2
   |
45 | /     fn append(
46 | |         a: Self,
47 | |         b: Self,
48 | |     ) -> Self;
   | |______________^
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: use associated function syntax instead
   |
17 -         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
17 +         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| R::append(acc, f(i, x)))
   |
help: the following trait defines an item `append`, perhaps you need to restrict type parameter `R` with it:
   |
12 |     fn apply<R: 'a + Monoid + 'static + winnow::error::ParserError</* I */>>(
   |                                       +++++++++++++++++++++++++++++++++++++

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,P,I,S,S,A,A>::evaluate (line 14) stdout ----
error[E0277]: the trait bound `i32: Monoid` is not satisfied
   --> fp-library/src/types/optics/indexed_lens.rs:25:43
    |
 14 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
    |                                           ^^^ the trait `Monoid` is not implemented for `i32`
    |
    = help: the following other types implement trait `Monoid`:
              CatList<A>
              Endofunction<'a, FnBrand, A>
              Endomorphism<'a, C, A>
              SendEndofunction<'a, FnBrand, A>
              String
              Thunk<'a, A>
              TryThunk<'a, A, E>
              Vec<A>
note: required by a bound in `fp_library::classes::IndexedFoldOptic::evaluate`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:484:22
    |
484 |     fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
    |                         ^^^^^^ required by this bound in `IndexedFoldOptic::evaluate`

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0277`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::new (line 536) stdout ----
error[E0433]: failed to resolve: use of undeclared type `IndexedFoldOptic`
  --> fp-library/src/types/optics/indexed_fold.rs:557:14
   |
24 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
   |              ^^^^^^^^^^^^^^^^ use of undeclared type `IndexedFoldOptic`
   |
help: a trait with a similar name exists
   |
24 - let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
24 + let result = IndexedFoldFunc::evaluate::<i32, RcBrand>(&l, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedFoldOptic;
   |

error[E0599]: no method named `append` found for type parameter `R` in the current scope
  --> fp-library/src/types/optics/indexed_fold.rs:550:64
   |
12 |     fn apply<R: 'a + Monoid + 'static>(
   |              - method `append` not found for this type parameter
...
17 |         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
   |                                                                      ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
note: the candidate is defined in the trait `Semigroup`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/semigroup.rs:45:2
   |
45 | /     fn append(
46 | |         a: Self,
47 | |         b: Self,
48 | |     ) -> Self;
   | |______________^
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: use associated function syntax instead
   |
17 -         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
17 +         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| R::append(acc, f(i, x)))
   |
help: the following trait defines an item `append`, perhaps you need to restrict type parameter `R` with it:
   |
12 |     fn apply<R: 'a + Monoid + 'static + winnow::error::ParserError</* I */>>(
   |                                       +++++++++++++++++++++++++++++++++++++

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::clone (line 8) stdout ----
error[E0433]: failed to resolve: use of undeclared type `IndexedFoldOptic`
  --> fp-library/src/types/optics/indexed_fold.rs:31:14
   |
26 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&cloned, pab);
   |              ^^^^^^^^^^^^^^^^ use of undeclared type `IndexedFoldOptic`
   |
help: a trait with a similar name exists
   |
26 - let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&cloned, pab);
26 + let result = IndexedFoldFunc::evaluate::<i32, RcBrand>(&cloned, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedFoldOptic;
   |

error[E0599]: no method named `append` found for type parameter `R` in the current scope
  --> fp-library/src/types/optics/indexed_fold.rs:23:64
   |
13 |     fn apply<R: 'a + Monoid + 'static>(
   |              - method `append` not found for this type parameter
...
18 |         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
   |                                                                      ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
note: the candidate is defined in the trait `Semigroup`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/semigroup.rs:45:2
   |
45 | /     fn append(
46 | |         a: Self,
47 | |         b: Self,
48 | |     ) -> Self;
   | |______________^
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: use associated function syntax instead
   |
18 -         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| acc.append(f(i, x)))
18 +         s.into_iter().enumerate().fold(R::empty(), |acc, (i, x)| R::append(acc, f(i, x)))
   |
help: the following trait defines an item `append`, perhaps you need to restrict type parameter `R` with it:
   |
13 |     fn apply<R: 'a + Monoid + 'static + winnow::error::ParserError</* I */>>(
   |                                       +++++++++++++++++++++++++++++++++++++

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,P,I,S,T,A,B>::evaluate (line 11) stdout ----
error[E0107]: method takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_lens.rs:23:22
    |
 15 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |                         ^^^^^^^^----------- help: remove the unnecessary generics
    |                         |
    |                         expected 0 generic arguments
    |
note: method defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:504:5
    |
504 |     fn evaluate(
    |        ^^^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,Q,I,S,T,A,B>::evaluate_indexed (line 11) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_lens.rs:19:17
    |
 11 | let unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                 |
    |                 expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/indexed_lens.rs:20:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&unindexed, (42, "hi".to_string())), 42);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,Q,I,S,T,A,B>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_lens.rs:19:16
    |
 11 | let as_index = optics_as_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                |
    |                expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/indexed_lens.rs:20:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&as_index, (42, "hi".to_string())), 10);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,P,I,S,A>::evaluate (line 11) stdout ----
error[E0107]: method takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_lens.rs:23:22
    |
 15 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |                         ^^^^^^^^----------- help: remove the unnecessary generics
    |                         |
    |                         expected 0 generic arguments
    |
note: method defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:504:5
    |
504 |     fn evaluate(
    |        ^^^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,P,I,S,A>::evaluate (line 14) stdout ----
error[E0277]: the trait bound `i32: Monoid` is not satisfied
   --> fp-library/src/types/optics/indexed_lens.rs:25:43
    |
 14 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
    |                                           ^^^ the trait `Monoid` is not implemented for `i32`
    |
    = help: the following other types implement trait `Monoid`:
              CatList<A>
              Endofunction<'a, FnBrand, A>
              Endomorphism<'a, C, A>
              SendEndofunction<'a, FnBrand, A>
              String
              Thunk<'a, A>
              TryThunk<'a, A, E>
              Vec<A>
note: required by a bound in `fp_library::classes::IndexedFoldOptic::evaluate`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:484:22
    |
484 |     fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
    |                         ^^^^^^ required by this bound in `IndexedFoldOptic::evaluate`

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0277`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
<BrandasKind_cdc7cd43dac7585f>::Of<'a,B>,A,B,Mapped<Brand>>::mapped (line 293) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:301:17
    |
 11 |     IndexedSetter::mapped::<VecBrand>();
    |                    ^^^^^^------------ help: remove the unnecessary generics
    |                    |
    |                    expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:300:10
    |
300 |         pub fn mapped() -> Self {
    |                ^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,Q,I,S,A>::evaluate_indexed (line 11) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_lens.rs:19:17
    |
 11 | let unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                 |
    |                 expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/indexed_lens.rs:20:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&unindexed, (42, "hi".to_string())), 42);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), i32, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,Q,I,S,A>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_lens.rs:19:16
    |
 11 | let as_index = optics_as_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                |
    |                expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_: GetterOptic<'_, _, _>` is not satisfied
  --> fp-library/src/types/optics/indexed_lens.rs:20:35
   |
12 | assert_eq!(optics_view::<RcBrand, _, _, _>(&as_index, (42, "hi".to_string())), 10);
   |                                   ^ the trait `GetterOptic<'_, _, _>` is not implemented for `impl Optic<'_, _, (i32, String), (i32, String), usize, i32> + '_`
   |
   = help: the following other types implement trait `GetterOptic<'a, S, A>`:
             Composed<'a, S, S, M, M, A, A, O1, O2>
             GetterPrime<'a, P, S, A>
             Iso<'a, P, S, S, A, A>
             IsoPrime<'a, P, S, A>
             Lens<'a, P, S, S, A, A>
             LensPrime<'a, P, S, A>
note: required by a bound in `fp_library::types::optics::optics_view`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:79:6
   |
73 |     pub fn optics_view<'a, P, O, S, A>(
   |            ----------- required by a bound in this function
...
79 |         O: GetterOptic<'a, S, A>,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_view`

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,S,T,A,B,F>::evaluate (line 11) stdout ----
error[E0107]: method takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:29:22
    |
 21 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |                         ^^^^^^^^----------- help: remove the unnecessary generics
    |                         |
    |                         expected 0 generic arguments
    |
note: method defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:504:5
    |
504 |     fn evaluate(
    |        ^^^^^^^^

error[E0277]: the trait bound `MySetter: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_setter.rs:29:42
    |
 21 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |     --------------------------------------- ^^ the trait `Clone` is not implemented for `MySetter`
    |     |
    |     required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedSetterOptic<'_, Q, I, S, T, A, B>` is implemented for `fp_library::types::optics::IndexedSetter<'_, P, I, S, T, A, B, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:569:2
    |
569 | /     impl<'a, Q, I: 'a, S: 'a, T: 'a, A: 'a, B: 'a, P, F> IndexedSetterOptic<'a, Q, I, S, T, A, B>
570 | |         for IndexedSetter<'a, P, I, S, T, A, B, F>
571 | |     where
572 | |         F: IndexedSetterFunc<'a, I, S, T, A, B> + Clone + 'a,
573 | |         Q: UnsizedCoercible,
    | |____________________________^
    = note: required for `IndexedSetter<'_, RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MySetter>` to implement `fp_library::classes::IndexedSetterOptic<'_, _, usize, Vec<i32>, Vec<i32>, i32, i32>`
    = note: the full name for the type has been written to '/tmp/rustdoctesthgeWvT/rust_out.long-type-4558099596510927949.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider annotating `MySetter` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MySetter;
    |

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,S,T,A,B,F>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:18:75
    |
 10 | let l = IndexedSetter::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _>::mapped::<VecBrand>();
    |                                                                           ^^^^^^------------ help: remove the unnecessary generics
    |                                                                           |
    |                                                                           expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:300:10
    |
300 |         pub fn mapped() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_setter.rs:19:17
    |
 11 | let unindexed = optics_as_index::<FnBrand<RcBrand>, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^ expected 7 generic arguments        --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, FnBrand<_>, _, _, usize, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_setter.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&unindexed, vec![10, 20], |i| i + 1), vec![1, 2]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, FnBrand<_>, _, _, usize, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,S,T,A,B,F>::evaluate_indexed (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:18:75
    |
 10 | let l = IndexedSetter::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _>::mapped::<VecBrand>();
    |                                                                           ^^^^^^------------ help: remove the unnecessary generics
    |                                                                           |
    |                                                                           expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:300:10
    |
300 |         pub fn mapped() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_setter.rs:19:17
    |
 11 | let unindexed = optics_un_index::<FnBrand<RcBrand>, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^ expected 7 generic arguments        --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, FnBrand<_>, _, _, i32, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_setter.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&unindexed, vec![1, 2], |x| x + 1), vec![2, 3]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, FnBrand<_>, _, _, i32, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,A,
Mapped<Brand>>::mapped (line 492) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:500:22
    |
 11 |     IndexedSetterPrime::mapped::<VecBrand>();
    |                         ^^^^^^------------ help: remove the unnecessary generics
    |                         |
    |                         expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:499:10
    |
499 |         pub fn mapped() -> Self {
    |                ^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,S,A,F>::evaluate (line 11) stdout ----
error[E0107]: method takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:29:22
    |
 21 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |                         ^^^^^^^^----------- help: remove the unnecessary generics
    |                         |
    |                         expected 0 generic arguments
    |
note: method defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:504:5
    |
504 |     fn evaluate(
    |        ^^^^^^^^

error[E0277]: the trait bound `MySetter: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_setter.rs:29:42
    |
 21 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |     --------------------------------------- ^^ the trait `Clone` is not implemented for `MySetter`
    |     |
    |     required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedSetterOptic<'_, Q, I, S, S, A, A>` is implemented for `fp_library::types::optics::IndexedSetterPrime<'_, P, I, S, A, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:514:2
    |
514 | /     impl<'a, Q, I: 'a, S: 'a, A: 'a, P, F> IndexedSetterOptic<'a, Q, I, S, S, A, A>
515 | |         for IndexedSetterPrime<'a, P, I, S, A, F>
516 | |     where
517 | |         P: UnsizedCoercible,
518 | |         Q: UnsizedCoercible,
519 | |         F: IndexedSetterFunc<'a, I, S, S, A, A> + Clone + 'a,
    | |_____________________________________________________________^
    = note: required for `fp_library::types::optics::IndexedSetterPrime<'_, fp_library::brands::RcBrand, usize, Vec<i32>, i32, MySetter>` to implement `fp_library::classes::IndexedSetterOptic<'_, _, usize, Vec<i32>, Vec<i32>, i32, i32>`
help: consider annotating `MySetter` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MySetter;
    |

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,S,A,F>::evaluate_indexed (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:18:65
    |
 10 | let l = IndexedSetterPrime::<RcBrand, usize, Vec<i32>, i32, _>::mapped::<VecBrand>();
    |                                                                 ^^^^^^------------ help: remove the unnecessary generics
    |                                                                 |
    |                                                                 expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:499:10
    |
499 |         pub fn mapped() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_setter.rs:19:17
    |
 11 | let unindexed = optics_un_index::<FnBrand<RcBrand>, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^ expected 7 generic arguments        --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, FnBrand<_>, _, _, i32, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_setter.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&unindexed, vec![1, 2], |x| x + 1), vec![2, 3]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, FnBrand<_>, _, _, i32, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,S,A,F>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_setter.rs:18:65
    |
 10 | let l = IndexedSetterPrime::<RcBrand, usize, Vec<i32>, i32, _>::mapped::<VecBrand>();
    |                                                                 ^^^^^^------------ help: remove the unnecessary generics
    |                                                                 |
    |                                                                 expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_setter.rs:499:10
    |
499 |         pub fn mapped() -> Self {
    |                ^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_setter.rs:19:17
    |
 11 | let unindexed = optics_as_index::<FnBrand<RcBrand>, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^ expected 7 generic arguments        --- help: remove the unnecessary generic argument
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, FnBrand<_>, _, _, usize, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_setter.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&unindexed, vec![10, 20], |i| i + 1), vec![1, 2]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, FnBrand<_>, _, _, usize, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
<BrandasKind_cdc7cd43dac7585f>::Of<'a,B>,A,B,Traversed<Brand>>::traversed (line 168) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:176:20
    |
 11 |     IndexedTraversal::traversed::<VecBrand>();
    |                       ^^^^^^^^^------------ help: remove the unnecessary generics
    |                       |
    |                       expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:175:10
    |
175 |         pub fn traversed() -> Self {
    |                ^^^^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::Mapped<Brand>::apply (line 12) stdout ----
error[E0432]: unresolved import `fp_library::classes::optics::indexed_setter`
 --> fp-library/src/types/optics/indexed_setter.rs:17:19
  |
8 |     classes::optics::indexed_setter::IndexedSetterFunc,
  |                      ^^^^^^^^^^^^^^ could not find `indexed_setter` in `optics`

error[E0603]: module `indexed_setter` is private
  --> fp-library/src/types/optics/indexed_setter.rs:16:17
   |
 7 |     types::optics::indexed_setter::Mapped,
   |                    ^^^^^^^^^^^^^^ private module
   |
note: the module `indexed_setter` is defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics.rs:96:1
   |
96 | mod indexed_setter;
   | ^^^^^^^^^^^^^^^^^^

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0432, E0603.
For more information about an error, try `rustc --explain E0432`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,S,A,A,F>::evaluate (line 14) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:25:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:27:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0053]: method `apply` has an incompatible type for trait
  --> fp-library/src/types/optics/indexed_traversal.rs:25:6
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected associated type, found `()`
   |
   = note: expected signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>, Vec<_>) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, Vec<i32>>`
              found signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) + 'a)>, Vec<_>) -> ()`
help: change the parameter type to match the trait
   |
14 -         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
14 +         f: Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>,
   |

error[E0277]: the trait bound `i32: Monoid` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:37:43
    |
 26 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
    |                                           ^^^ the trait `Monoid` is not implemented for `i32`
    |
    = help: the following other types implement trait `Monoid`:
              CatList<A>
              Endofunction<'a, FnBrand, A>
              Endomorphism<'a, C, A>
              SendEndofunction<'a, FnBrand, A>
              String
              Thunk<'a, A>
              TryThunk<'a, A, E>
              Vec<A>
note: required by a bound in `fp_library::classes::IndexedFoldOptic::evaluate`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:484:22
    |
484 |     fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
    |                         ^^^^^^ required by this bound in `IndexedFoldOptic::evaluate`

error[E0277]: the trait bound `MyTraversal: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:37:57
    |
 26 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
    |              ------------------------------------------ ^^ the trait `Clone` is not implemented for `MyTraversal`
    |              |
    |              required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedFoldOptic<'_, I, S, A>` is implemented for `fp_library::types::optics::IndexedTraversal<'_, P, I, S, S, A, A, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:421:2
    |
421 | /     impl<'a, P, I: Clone + 'a, S: 'a, A: 'a, F> IndexedFoldOptic<'a, I, S, A>
422 | |         for IndexedTraversal<'a, P, I, S, S, A, A, F>
423 | |     where
424 | |         F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
    | |________________________________________________________________^
    = note: required for `IndexedTraversal<'_, RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MyTraversal>` to implement `fp_library::classes::IndexedFoldOptic<'_, usize, Vec<i32>, i32>`
    = note: the full name for the type has been written to '/tmp/rustdoctestE7PTUJ/rust_out.long-type-7649732876253378134.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider annotating `MyTraversal` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MyTraversal;
    |

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:29:62
   |
18 |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
   |             -------- arguments to this function are incorrect         ^^^^^^^ expected associated type, found `()`
   |
   = note: expected associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, i32>`
                    found unit type `()`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, _>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
note: associated function defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/lift.rs:67:5
   |
67 |     fn lift2<'a, A, B, C, Func>(
   |        ^^^^^

error[E0308]: mismatched types
    --> fp-library/src/types/optics/indexed_traversal.rs:28:34
     |
  17 |         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |                                   ---- ^^^^^^^^^^^^^^^ expected `()`, found associated type
     |                                   |
     |                                   arguments to this method are incorrect
     |
     = note:    expected unit type `()`
             found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>`
     = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` to `()`
     = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
help: the return type of this call is `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` due to the type of the argument passed
    --> fp-library/src/types/optics/indexed_traversal.rs:28:3
     |
  17 |           s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |           ^                              --------------- this argument influences the return type of `fold`
     |  _________|
     | |
  18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
  19 | |         })
     | |__________^
note: method defined here
    --> /nix/store/ycv729m2633y9f36xyh8kfcnxwdgikv6-rust-mixed/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2596:8
     |
2596 |     fn fold<B, F>(mut self, init: B, mut f: F) -> B
     |        ^^^^

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:28:3
   |
16 |       ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                     ---------- expected `()` because of return type
17 | /         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
19 | |         })
   | |          ^- help: consider using a semicolon here: `;`
   | |__________|
   |            expected `()`, found associated type
   |
   = note:    expected unit type `()`
           found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html

error: aborting due to 8 previous errors

Some errors have detailed explanations: E0053, E0277, E0308.
For more information about an error, try `rustc --explain E0053`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::new (line 302) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:313:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:315:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0405]: cannot find trait `IndexedTraversalFunc` in this scope
  --> fp-library/src/types/optics/indexed_traversal.rs:310:10
   |
11 | impl<'a> IndexedTraversalFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MyTraversal {
   |          ^^^^^^^^^^^^^^^^^^^^ not found in this scope
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::optics::IndexedTraversalFunc;
   |

error[E0433]: failed to resolve: use of undeclared type `IndexedTraversalOptic`
  --> fp-library/src/types/optics/indexed_traversal.rs:326:2
   |
27 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
   |     ^^^^^^^^^^^^^^^^^^^^^ use of undeclared type `IndexedTraversalOptic`
   |
help: a struct with a similar name exists
   |
27 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
27 +     IndexedTraversal::evaluate::<RcFnBrand>(&l, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedTraversalOptic;
   |

error[E0425]: cannot find type `RcFnBrand` in this scope
   --> fp-library/src/types/optics/indexed_traversal.rs:326:36
    |
 27 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
    |                                       ^^^^^^^^^
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/brands.rs:147:1
    |
147 | pub struct RcBrand;
    | ------------------ similarly named struct `RcBrand` defined here
    |
help: a struct with a similar name exists
    |
 27 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
 27 +     IndexedTraversalOptic::evaluate::<RcBrand>(&l, pab);
    |
help: consider importing this type alias
    |
  2 + use fp_library::brands::RcFnBrand;
    |

error: aborting due to 5 previous errors

Some errors have detailed explanations: E0405, E0425, E0433.
For more information about an error, try `rustc --explain E0405`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,Q,I,S,T,A,B,F>::evaluate_indexed (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:18:78
    |
 10 | let l = IndexedTraversal::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _>::traversed::<VecBrand>();
    |                                                                              ^^^^^^^^^------------ help: remove the unnecessary generics
    |                                                                              |
    |                                                                              expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:175:10
    |
175 |         pub fn traversed() -> Self {
    |                ^^^^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:19:17
    |
 11 | let unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                 |
    |                 expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, _, _, i32, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&unindexed, vec![1, 2], |x| x + 1), vec![2, 3]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, _, _, _, i32, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::clone (line 8) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:20:57
   |
15 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:22:32
   |
17 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0405]: cannot find trait `IndexedTraversalFunc` in this scope
  --> fp-library/src/types/optics/indexed_traversal.rs:17:10
   |
12 | impl<'a> IndexedTraversalFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MyTraversal {
   |          ^^^^^^^^^^^^^^^^^^^^ not found in this scope
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::optics::IndexedTraversalFunc;
   |

error[E0433]: failed to resolve: use of undeclared type `IndexedTraversalOptic`
  --> fp-library/src/types/optics/indexed_traversal.rs:34:2
   |
29 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
   |     ^^^^^^^^^^^^^^^^^^^^^ use of undeclared type `IndexedTraversalOptic`
   |
help: a struct with a similar name exists
   |
29 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
29 +     IndexedTraversal::evaluate::<RcFnBrand>(&cloned, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedTraversalOptic;
   |

error[E0425]: cannot find type `RcFnBrand` in this scope
   --> fp-library/src/types/optics/indexed_traversal.rs:34:36
    |
 29 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
    |                                       ^^^^^^^^^
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/brands.rs:147:1
    |
147 | pub struct RcBrand;
    | ------------------ similarly named struct `RcBrand` defined here
    |
help: a struct with a similar name exists
    |
 29 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
 29 +     IndexedTraversalOptic::evaluate::<RcBrand>(&cloned, pab);
    |
help: consider importing this type alias
    |
  2 + use fp_library::brands::RcFnBrand;
    |

error: aborting due to 5 previous errors

Some errors have detailed explanations: E0405, E0425, E0433.
For more information about an error, try `rustc --explain E0405`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
A,Traversed<Brand>>::traversed (line 210) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:218:25
    |
 11 |     IndexedTraversalPrime::traversed::<VecBrand>();
    |                            ^^^^^^^^^------------ help: remove the unnecessary generics
    |                            |
    |                            expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:217:10
    |
217 |         pub fn traversed() -> Self {
    |                ^^^^^^^^^

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::evaluate (line 11) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:22:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:24:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0053]: method `apply` has an incompatible type for trait
  --> fp-library/src/types/optics/indexed_traversal.rs:22:6
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected associated type, found `()`
   |
   = note: expected signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>, Vec<_>) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, Vec<i32>>`
              found signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) + 'a)>, Vec<_>) -> ()`
help: change the parameter type to match the trait
   |
14 -         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
14 +         f: Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>,
   |

error[E0107]: method takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:35:22
    |
 27 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |                         ^^^^^^^^----------- help: remove the unnecessary generics
    |                         |
    |                         expected 0 generic arguments
    |
note: method defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:504:5
    |
504 |     fn evaluate(
    |        ^^^^^^^^

error[E0277]: the trait bound `MyTraversal: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:35:42
    |
 27 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |     --------------------------------------- ^^ the trait `Clone` is not implemented for `MyTraversal`
    |     |
    |     required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedSetterOptic<'_, Q, I, S, T, A, B>` is implemented for `fp_library::types::optics::IndexedTraversal<'_, P, I, S, T, A, B, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:477:2
    |
477 | /     impl<'a, Q, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a, P, F>
478 | |         IndexedSetterOptic<'a, Q, I, S, T, A, B> for IndexedTraversal<'a, P, I, S, T, A, B, F>
479 | |     where
480 | |         F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
481 | |         Q: UnsizedCoercible,
    | |____________________________^
    = note: required for `IndexedTraversal<'_, RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MyTraversal>` to implement `fp_library::classes::IndexedSetterOptic<'_, _, usize, Vec<i32>, Vec<i32>, i32, i32>`
    = note: the full name for the type has been written to '/tmp/rustdoctestPCceys/rust_out.long-type-15018045553600946199.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider annotating `MyTraversal` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MyTraversal;
    |

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:26:62
   |
18 |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
   |             -------- arguments to this function are incorrect         ^^^^^^^ expected associated type, found `()`
   |
   = note: expected associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, i32>`
                    found unit type `()`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, _>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
note: associated function defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/lift.rs:67:5
   |
67 |     fn lift2<'a, A, B, C, Func>(
   |        ^^^^^

error[E0308]: mismatched types
    --> fp-library/src/types/optics/indexed_traversal.rs:25:34
     |
  17 |         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |                                   ---- ^^^^^^^^^^^^^^^ expected `()`, found associated type
     |                                   |
     |                                   arguments to this method are incorrect
     |
     = note:    expected unit type `()`
             found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>`
     = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` to `()`
     = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
help: the return type of this call is `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` due to the type of the argument passed
    --> fp-library/src/types/optics/indexed_traversal.rs:25:3
     |
  17 |           s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |           ^                              --------------- this argument influences the return type of `fold`
     |  _________|
     | |
  18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
  19 | |         })
     | |__________^
note: method defined here
    --> /nix/store/ycv729m2633y9f36xyh8kfcnxwdgikv6-rust-mixed/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2596:8
     |
2596 |     fn fold<B, F>(mut self, init: B, mut f: F) -> B
     |        ^^^^

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:25:3
   |
16 |       ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                     ---------- expected `()` because of return type
17 | /         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
19 | |         })
   | |          ^- help: consider using a semicolon here: `;`
   | |__________|
   |            expected `()`, found associated type
   |
   = note:    expected unit type `()`
           found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html

error: aborting due to 8 previous errors

Some errors have detailed explanations: E0053, E0107, E0277, E0308.
For more information about an error, try `rustc --explain E0053`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,Q,I,S,T,A,B,F>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:18:78
    |
 10 | let l = IndexedTraversal::<RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, _>::traversed::<VecBrand>();
    |                                                                              ^^^^^^^^^------------ help: remove the unnecessary generics
    |                                                                              |
    |                                                                              expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:175:10
    |
175 |         pub fn traversed() -> Self {
    |                ^^^^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:19:16
    |
 11 | let as_index = optics_as_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                |
    |                expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, _, _, usize, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&as_index, vec![10, 20], |i| i + 1), vec![1, 2]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, _, _, _, usize, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::evaluate (line 13) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:24:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:26:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0053]: method `apply` has an incompatible type for trait
  --> fp-library/src/types/optics/indexed_traversal.rs:24:6
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected associated type, found `()`
   |
   = note: expected signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>, Vec<_>) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, Vec<i32>>`
              found signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) + 'a)>, Vec<_>) -> ()`
help: change the parameter type to match the trait
   |
14 -         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
14 +         f: Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>,
   |

error[E0277]: the trait bound `MyTraversal: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:37:47
    |
 27 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
    |     -------------------------------------------- ^^ the trait `Clone` is not implemented for `MyTraversal`
    |     |
    |     required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedTraversalOptic<'_, I, S, T, A, B>` is implemented for `fp_library::types::optics::IndexedTraversal<'_, P, I, S, T, A, B, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:340:2
    |
340 | /     impl<'a, P, I: Clone + 'a, S: 'a, T: 'a, A: 'a, B: 'a, F>
341 | |         IndexedTraversalOptic<'a, I, S, T, A, B> for IndexedTraversal<'a, P, I, S, T, A, B, F>
342 | |     where
343 | |         F: IndexedTraversalFunc<'a, I, S, T, A, B> + Clone + 'a,
    | |________________________________________________________________^
    = note: required for `IndexedTraversal<'_, RcBrand, usize, Vec<i32>, Vec<i32>, i32, i32, MyTraversal>` to implement `fp_library::classes::IndexedTraversalOptic<'_, usize, Vec<i32>, Vec<i32>, i32, i32>`
    = note: the full name for the type has been written to '/tmp/rustdoctestgZjiAA/rust_out.long-type-12126111163992153634.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: consider annotating `MyTraversal` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MyTraversal;
    |

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:28:62
   |
18 |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
   |             -------- arguments to this function are incorrect         ^^^^^^^ expected associated type, found `()`
   |
   = note: expected associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, i32>`
                    found unit type `()`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, _>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
note: associated function defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/lift.rs:67:5
   |
67 |     fn lift2<'a, A, B, C, Func>(
   |        ^^^^^

error[E0308]: mismatched types
    --> fp-library/src/types/optics/indexed_traversal.rs:27:34
     |
  17 |         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |                                   ---- ^^^^^^^^^^^^^^^ expected `()`, found associated type
     |                                   |
     |                                   arguments to this method are incorrect
     |
     = note:    expected unit type `()`
             found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>`
     = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` to `()`
     = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
help: the return type of this call is `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` due to the type of the argument passed
    --> fp-library/src/types/optics/indexed_traversal.rs:27:3
     |
  17 |           s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |           ^                              --------------- this argument influences the return type of `fold`
     |  _________|
     | |
  18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
  19 | |         })
     | |__________^
note: method defined here
    --> /nix/store/ycv729m2633y9f36xyh8kfcnxwdgikv6-rust-mixed/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2596:8
     |
2596 |     fn fold<B, F>(mut self, init: B, mut f: F) -> B
     |        ^^^^

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:27:3
   |
16 |       ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                     ---------- expected `()` because of return type
17 | /         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
19 | |         })
   | |          ^- help: consider using a semicolon here: `;`
   | |__________|
   |            expected `()`, found associated type
   |
   = note:    expected unit type `()`
           found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html

error: aborting due to 7 previous errors

Some errors have detailed explanations: E0053, E0277, E0308.
For more information about an error, try `rustc --explain E0053`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::clone (line 8) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:20:57
   |
15 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:22:32
   |
17 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0405]: cannot find trait `IndexedTraversalFunc` in this scope
  --> fp-library/src/types/optics/indexed_traversal.rs:17:10
   |
12 | impl<'a> IndexedTraversalFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MyTraversal {
   |          ^^^^^^^^^^^^^^^^^^^^ not found in this scope
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::optics::IndexedTraversalFunc;
   |

error[E0433]: failed to resolve: use of undeclared type `IndexedTraversalOptic`
  --> fp-library/src/types/optics/indexed_traversal.rs:34:2
   |
29 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
   |     ^^^^^^^^^^^^^^^^^^^^^ use of undeclared type `IndexedTraversalOptic`
   |
help: a struct with a similar name exists
   |
29 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
29 +     IndexedTraversal::evaluate::<RcFnBrand>(&cloned, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedTraversalOptic;
   |

error[E0425]: cannot find type `RcFnBrand` in this scope
   --> fp-library/src/types/optics/indexed_traversal.rs:34:36
    |
 29 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
    |                                       ^^^^^^^^^
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/brands.rs:147:1
    |
147 | pub struct RcBrand;
    | ------------------ similarly named struct `RcBrand` defined here
    |
help: a struct with a similar name exists
    |
 29 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&cloned, pab);
 29 +     IndexedTraversalOptic::evaluate::<RcBrand>(&cloned, pab);
    |
help: consider importing this type alias
    |
  2 + use fp_library::brands::RcFnBrand;
    |

error: aborting due to 5 previous errors

Some errors have detailed explanations: E0405, E0425, E0433.
For more information about an error, try `rustc --explain E0405`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,Q,I,S,A,F>::evaluate_indexed (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:18:68
    |
 10 | let l = IndexedTraversalPrime::<RcBrand, usize, Vec<i32>, i32, _>::traversed::<VecBrand>();
    |                                                                    ^^^^^^^^^------------ help: remove the unnecessary generics
    |                                                                    |
    |                                                                    expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:217:10
    |
217 |         pub fn traversed() -> Self {
    |                ^^^^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:19:17
    |
 11 | let unindexed = optics_un_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                 ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                 |
    |                 expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:733:9
    |
733 |     pub fn optics_un_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, _, _, i32, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&unindexed, vec![1, 2], |x| x + 1), vec![2, 3]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, _, _, _, i32, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,Q,I,S,A,F>::evaluate_indexed_discards_focus (line 11) stdout ----
error[E0107]: associated function takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:18:68
    |
 10 | let l = IndexedTraversalPrime::<RcBrand, usize, Vec<i32>, i32, _>::traversed::<VecBrand>();
    |                                                                    ^^^^^^^^^------------ help: remove the unnecessary generics
    |                                                                    |
    |                                                                    expected 0 generic arguments
    |
note: associated function defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:217:10
    |
217 |         pub fn traversed() -> Self {
    |                ^^^^^^^^^

error[E0107]: function takes 7 generic arguments but 8 generic arguments were supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:19:16
    |
 11 | let as_index = optics_as_index::<RcBrand, _, _, _, _, _, _, _>(&l);
    |                ^^^^^^^^^^^^^^^                            --- help: remove the unnecessary generic argument
    |                |
    |                expected 7 generic arguments
    |
note: function defined here, with 7 generic parameters: `P`, `O`, `I`, `S`, `T`, `A`, `B`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:808:9
    |
808 |     pub fn optics_as_index<'a, P, O, I, S, T, A, B>(
    |            ^^^^^^^^^^^^^^^     -  -  -  -  -  -  -

error[E0277]: the trait bound `impl Optic<'_, _, _, _, usize, i32> + '_: SetterOptic<'_, RcBrand, _, _, _, _>` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:20:35
    |
 12 | assert_eq!(optics_over::<RcBrand, _, _, _, _>(&as_index, vec![10, 20], |i| i + 1), vec![1, 2]);
    |                                   ^ the trait `SetterOptic<'_, RcBrand, _, _, _, _>` is not implemented for `impl Optic<'_, _, _, _, usize, i32> + '_`
    |
    = help: the following other types implement trait `SetterOptic<'a, P, S, T, A, B>`:
              `AffineTraversal<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `AffineTraversalPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Composed<'a, S, T, M, N, A, B, O1, O2>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `Grate<'a, P, S, T, A, B>` implements `SetterOptic<'a, P, S, T, A, B>`
              `GratePrime<'a, P, S, A>` implements `SetterOptic<'a, P, S, S, A, A>`
              `Iso<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
              `IsoPrime<'a, P, S, A>` implements `SetterOptic<'a, Q, S, S, A, A>`
              `Lens<'a, P, S, T, A, B>` implements `SetterOptic<'a, Q, S, T, A, B>`
            and 6 others
note: required by a bound in `fp_library::types::optics::optics_over`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/functions.rs:175:6
    |
168 |     pub fn optics_over<'a, Q, O, S, A, F>(
    |            ----------- required by a bound in this function
...
175 |         O: SetterOptic<'a, Q, S, S, A, A>,
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `optics_over`

error: aborting due to 3 previous errors

Some errors have detailed explanations: E0107, E0277.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::evaluate (line 13) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:24:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:26:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0053]: method `apply` has an incompatible type for trait
  --> fp-library/src/types/optics/indexed_traversal.rs:24:6
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected associated type, found `()`
   |
   = note: expected signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>, Vec<_>) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, Vec<i32>>`
              found signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) + 'a)>, Vec<_>) -> ()`
help: change the parameter type to match the trait
   |
14 -         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
14 +         f: Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>,
   |

error[E0277]: the trait bound `MyTraversal: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:37:47
    |
 27 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
    |     -------------------------------------------- ^^ the trait `Clone` is not implemented for `MyTraversal`
    |     |
    |     required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedTraversalOptic<'_, I, S, S, A, A>` is implemented for `fp_library::types::optics::IndexedTraversalPrime<'_, P, I, S, A, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:729:2
    |
729 | /     impl<'a, P, I: Clone + 'a, S: 'a, A: 'a, F> IndexedTraversalOptic<'a, I, S, S, A, A>
730 | |         for IndexedTraversalPrime<'a, P, I, S, A, F>
731 | |     where
732 | |         F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
    | |________________________________________________________________^
    = note: required for `fp_library::types::optics::IndexedTraversalPrime<'_, fp_library::brands::RcBrand, usize, Vec<i32>, i32, MyTraversal>` to implement `fp_library::classes::IndexedTraversalOptic<'_, usize, Vec<i32>, Vec<i32>, i32, i32>`
help: consider annotating `MyTraversal` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MyTraversal;
    |

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:28:62
   |
18 |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
   |             -------- arguments to this function are incorrect         ^^^^^^^ expected associated type, found `()`
   |
   = note: expected associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, i32>`
                    found unit type `()`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, _>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
note: associated function defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/lift.rs:67:5
   |
67 |     fn lift2<'a, A, B, C, Func>(
   |        ^^^^^

error[E0308]: mismatched types
    --> fp-library/src/types/optics/indexed_traversal.rs:27:34
     |
  17 |         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |                                   ---- ^^^^^^^^^^^^^^^ expected `()`, found associated type
     |                                   |
     |                                   arguments to this method are incorrect
     |
     = note:    expected unit type `()`
             found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>`
     = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` to `()`
     = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
help: the return type of this call is `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` due to the type of the argument passed
    --> fp-library/src/types/optics/indexed_traversal.rs:27:3
     |
  17 |           s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |           ^                              --------------- this argument influences the return type of `fold`
     |  _________|
     | |
  18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
  19 | |         })
     | |__________^
note: method defined here
    --> /nix/store/ycv729m2633y9f36xyh8kfcnxwdgikv6-rust-mixed/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2596:8
     |
2596 |     fn fold<B, F>(mut self, init: B, mut f: F) -> B
     |        ^^^^

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:27:3
   |
16 |       ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                     ---------- expected `()` because of return type
17 | /         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
19 | |         })
   | |          ^- help: consider using a semicolon here: `;`
   | |__________|
   |            expected `()`, found associated type
   |
   = note:    expected unit type `()`
           found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html

error: aborting due to 7 previous errors

Some errors have detailed explanations: E0053, E0277, E0308.
For more information about an error, try `rustc --explain E0053`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::evaluate (line 14) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:25:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:27:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0053]: method `apply` has an incompatible type for trait
  --> fp-library/src/types/optics/indexed_traversal.rs:25:6
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected associated type, found `()`
   |
   = note: expected signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>, Vec<_>) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, Vec<i32>>`
              found signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) + 'a)>, Vec<_>) -> ()`
help: change the parameter type to match the trait
   |
14 -         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
14 +         f: Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>,
   |

error[E0277]: the trait bound `i32: Monoid` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:37:43
    |
 26 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
    |                                           ^^^ the trait `Monoid` is not implemented for `i32`
    |
    = help: the following other types implement trait `Monoid`:
              CatList<A>
              Endofunction<'a, FnBrand, A>
              Endomorphism<'a, C, A>
              SendEndofunction<'a, FnBrand, A>
              String
              Thunk<'a, A>
              TryThunk<'a, A, E>
              Vec<A>
note: required by a bound in `fp_library::classes::IndexedFoldOptic::evaluate`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:484:22
    |
484 |     fn evaluate<R: 'a + Monoid + 'static, P: UnsizedCoercible + 'static>(
    |                         ^^^^^^ required by this bound in `IndexedFoldOptic::evaluate`

error[E0277]: the trait bound `MyTraversal: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:37:57
    |
 26 | let result = IndexedFoldOptic::evaluate::<i32, RcBrand>(&l, pab);
    |              ------------------------------------------ ^^ the trait `Clone` is not implemented for `MyTraversal`
    |              |
    |              required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedFoldOptic<'_, I, S, A>` is implemented for `fp_library::types::optics::IndexedTraversalPrime<'_, P, I, S, A, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:810:2
    |
810 | /     impl<'a, P, I: Clone + 'a, S: 'a, A: 'a, F> IndexedFoldOptic<'a, I, S, A>
811 | |         for IndexedTraversalPrime<'a, P, I, S, A, F>
812 | |     where
813 | |         F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
    | |________________________________________________________________^
    = note: required for `fp_library::types::optics::IndexedTraversalPrime<'_, fp_library::brands::RcBrand, usize, Vec<i32>, i32, MyTraversal>` to implement `fp_library::classes::IndexedFoldOptic<'_, usize, Vec<i32>, i32>`
help: consider annotating `MyTraversal` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MyTraversal;
    |

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:29:62
   |
18 |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
   |             -------- arguments to this function are incorrect         ^^^^^^^ expected associated type, found `()`
   |
   = note: expected associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, i32>`
                    found unit type `()`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, _>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
note: associated function defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/lift.rs:67:5
   |
67 |     fn lift2<'a, A, B, C, Func>(
   |        ^^^^^

error[E0308]: mismatched types
    --> fp-library/src/types/optics/indexed_traversal.rs:28:34
     |
  17 |         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |                                   ---- ^^^^^^^^^^^^^^^ expected `()`, found associated type
     |                                   |
     |                                   arguments to this method are incorrect
     |
     = note:    expected unit type `()`
             found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>`
     = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` to `()`
     = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
help: the return type of this call is `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` due to the type of the argument passed
    --> fp-library/src/types/optics/indexed_traversal.rs:28:3
     |
  17 |           s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |           ^                              --------------- this argument influences the return type of `fold`
     |  _________|
     | |
  18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
  19 | |         })
     | |__________^
note: method defined here
    --> /nix/store/ycv729m2633y9f36xyh8kfcnxwdgikv6-rust-mixed/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2596:8
     |
2596 |     fn fold<B, F>(mut self, init: B, mut f: F) -> B
     |        ^^^^

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:28:3
   |
16 |       ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                     ---------- expected `()` because of return type
17 | /         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
19 | |         })
   | |          ^- help: consider using a semicolon here: `;`
   | |__________|
   |            expected `()`, found associated type
   |
   = note:    expected unit type `()`
           found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html

error: aborting due to 8 previous errors

Some errors have detailed explanations: E0053, E0277, E0308.
For more information about an error, try `rustc --explain E0053`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::evaluate (line 11) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:22:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:24:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0053]: method `apply` has an incompatible type for trait
  --> fp-library/src/types/optics/indexed_traversal.rs:22:6
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected associated type, found `()`
   |
   = note: expected signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>, Vec<_>) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, Vec<i32>>`
              found signature `fn(&MyTraversal, Box<(dyn Fn(usize, i32) + 'a)>, Vec<_>) -> ()`
help: change the parameter type to match the trait
   |
14 -         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
14 +         f: Box<(dyn Fn(usize, i32) -> <M as Kind_cdc7cd43dac7585f>::Of<'a, i32> + 'a)>,
   |

error[E0107]: method takes 0 generic arguments but 1 generic argument was supplied
   --> fp-library/src/types/optics/indexed_traversal.rs:35:22
    |
 27 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |                         ^^^^^^^^----------- help: remove the unnecessary generics
    |                         |
    |                         expected 0 generic arguments
    |
note: method defined here, with 0 generic parameters
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics.rs:504:5
    |
504 |     fn evaluate(
    |        ^^^^^^^^

error[E0277]: the trait bound `MyTraversal: Clone` is not satisfied
   --> fp-library/src/types/optics/indexed_traversal.rs:35:42
    |
 27 |     IndexedSetterOptic::evaluate::<RcBrand>(&l, pab);
    |     --------------------------------------- ^^ the trait `Clone` is not implemented for `MyTraversal`
    |     |
    |     required by a bound introduced by this call
    |
help: the trait `fp_library::classes::IndexedSetterOptic<'_, Q, I, S, S, A, A>` is implemented for `fp_library::types::optics::IndexedTraversalPrime<'_, P, I, S, A, F>`
   --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics/indexed_traversal.rs:864:2
    |
864 | /     impl<'a, Q, I: Clone + 'a, S: 'a, A: 'a, P, F> IndexedSetterOptic<'a, Q, I, S, S, A, A>
865 | |         for IndexedTraversalPrime<'a, P, I, S, A, F>
866 | |     where
867 | |         F: IndexedTraversalFunc<'a, I, S, S, A, A> + Clone + 'a,
868 | |         Q: UnsizedCoercible,
    | |____________________________^
    = note: required for `fp_library::types::optics::IndexedTraversalPrime<'_, fp_library::brands::RcBrand, usize, Vec<i32>, i32, MyTraversal>` to implement `fp_library::classes::IndexedSetterOptic<'_, _, usize, Vec<i32>, Vec<i32>, i32, i32>`
help: consider annotating `MyTraversal` with `#[derive(Clone)]`
    |
 10 + #[derive(Clone)]
 11 | struct MyTraversal;
    |

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:26:62
   |
18 |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
   |             -------- arguments to this function are incorrect         ^^^^^^^ expected associated type, found `()`
   |
   = note: expected associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, i32>`
                    found unit type `()`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, _>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
note: associated function defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/lift.rs:67:5
   |
67 |     fn lift2<'a, A, B, C, Func>(
   |        ^^^^^

error[E0308]: mismatched types
    --> fp-library/src/types/optics/indexed_traversal.rs:25:34
     |
  17 |         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |                                   ---- ^^^^^^^^^^^^^^^ expected `()`, found associated type
     |                                   |
     |                                   arguments to this method are incorrect
     |
     = note:    expected unit type `()`
             found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>`
     = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` to `()`
     = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
help: the return type of this call is `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<_>>` due to the type of the argument passed
    --> fp-library/src/types/optics/indexed_traversal.rs:25:3
     |
  17 |           s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
     |           ^                              --------------- this argument influences the return type of `fold`
     |  _________|
     | |
  18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
  19 | |         })
     | |__________^
note: method defined here
    --> /nix/store/ycv729m2633y9f36xyh8kfcnxwdgikv6-rust-mixed/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2596:8
     |
2596 |     fn fold<B, F>(mut self, init: B, mut f: F) -> B
     |        ^^^^

error[E0308]: mismatched types
  --> fp-library/src/types/optics/indexed_traversal.rs:25:3
   |
16 |       ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                     ---------- expected `()` because of return type
17 | /         s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, x)| {
18 | |             M::lift2(|mut v: Vec<i32>, y: i32| { v.push(y); v }, acc, f(i, x))
19 | |         })
   | |          ^- help: consider using a semicolon here: `;`
   | |__________|
   |            expected `()`, found associated type
   |
   = note:    expected unit type `()`
           found associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>`
   = help: consider constraining the associated type `<M as Kind_cdc7cd43dac7585f>::Of<'_, Vec<i32>>` to `()`
   = note: for more information, visit https://doc.rust-lang.org/book/ch19-03-advanced-traits.html

error: aborting due to 8 previous errors

Some errors have detailed explanations: E0053, E0107, E0277, E0308.
For more information about an error, try `rustc --explain E0053`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::new (line 693) stdout ----
error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:704:57
   |
14 |         f: Box<dyn Fn(usize, i32) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, i32>) + 'a>,
   |                                                               ^^^^^^^^^^

error: expected `Kind`
  --> fp-library/src/types/optics/indexed_traversal.rs:706:32
   |
16 |     ) -> fp_library::Apply!(<M as fp_library::kinds::Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, Vec<i32>>) {
   |                                   ^^^^^^^^^^

error[E0405]: cannot find trait `IndexedTraversalFunc` in this scope
  --> fp-library/src/types/optics/indexed_traversal.rs:701:10
   |
11 | impl<'a> IndexedTraversalFunc<'a, usize, Vec<i32>, Vec<i32>, i32, i32> for MyTraversal {
   |          ^^^^^^^^^^^^^^^^^^^^ not found in this scope
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::optics::IndexedTraversalFunc;
   |

error[E0433]: failed to resolve: use of undeclared type `IndexedTraversalOptic`
  --> fp-library/src/types/optics/indexed_traversal.rs:717:2
   |
27 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
   |     ^^^^^^^^^^^^^^^^^^^^^ use of undeclared type `IndexedTraversalOptic`
   |
help: a struct with a similar name exists
   |
27 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
27 +     IndexedTraversal::evaluate::<RcFnBrand>(&l, pab);
   |
help: consider importing this trait
   |
 2 + use fp_library::classes::IndexedTraversalOptic;
   |

error[E0425]: cannot find type `RcFnBrand` in this scope
   --> fp-library/src/types/optics/indexed_traversal.rs:717:36
    |
 27 |     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
    |                                       ^^^^^^^^^
    |
   ::: /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/brands.rs:147:1
    |
147 | pub struct RcBrand;
    | ------------------ similarly named struct `RcBrand` defined here
    |
help: a struct with a similar name exists
    |
 27 -     IndexedTraversalOptic::evaluate::<RcFnBrand>(&l, pab);
 27 +     IndexedTraversalOptic::evaluate::<RcBrand>(&l, pab);
    |
help: consider importing this type alias
    |
  2 + use fp_library::brands::RcFnBrand;
    |

error: aborting due to 5 previous errors

Some errors have detailed explanations: E0405, E0425, E0433.
For more information about an error, try `rustc --explain E0405`.
Couldn't compile the test.
---- fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::Traversed<Brand>::apply (line 14) stdout ----
error[E0603]: module `indexed_traversal` is private
  --> fp-library/src/types/optics/indexed_traversal.rs:18:17
   |
 7 |     types::optics::indexed_traversal::Traversed,
   |                    ^^^^^^^^^^^^^^^^^ private module
   |
note: the module `indexed_traversal` is defined here
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/optics.rs:97:1
   |
97 | mod indexed_traversal;
   | ^^^^^^^^^^^^^^^^^^^^^

error[E0107]: method takes 1 generic argument but 2 generic arguments were supplied
  --> fp-library/src/types/optics/indexed_traversal.rs:28:54
   |
17 | let result: Option<Vec<i32>> = IndexedTraversalFunc::apply::<OptionBrand, _>(
   |                                                      ^^^^^              --- help: remove the unnecessary generic argument
   |                                                      |
   |                                                      expected 1 generic argument
   |
note: method defined here, with 1 generic parameter: `M`
  --> /home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/optics/indexed_traversal.rs:12:5
   |
12 |     fn apply<M: Applicative>(
   |        ^^^^^ -

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0107, E0603.
For more information about an error, try `rustc --explain E0107`.
Couldn't compile the test.

failures:
    fp-library/src/types/optics/composed.rs - types::optics::composed::inner::Composed<'a,S,S,M,M,A,A,O1,O2>::evaluate (line 47)
    fp-library/src/types/optics/composed.rs - types::optics::composed::inner::Composed<'a,S,S,M,M,A,A,O1,O2>::evaluate (line 47)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::PositionsTraversalFunc<F>::apply (line 14)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_as_index (line 815)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_as_index::AsIndex<'a,P,O,I,S,T,A,B>::evaluate (line 840)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_reindexed (line 899)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_reindexed::Reindexed<'a,P,O,I,J,S,T,A,B,F>::evaluate_indexed (line 928)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_un_index (line 740)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::optics_un_index::UnIndex<'a,P,O,I,S,T,A,B>::evaluate (line 765)
    fp-library/src/types/optics/functions.rs - types::optics::functions::inner::positions (line 1036)
    fp-library/src/types/optics/indexed.rs - types::optics::indexed::inner::IndexedBrand<P,I>::dimap (line 22)
    fp-library/src/types/optics/indexed.rs - types::optics::indexed::inner::IndexedBrand<P,I>::wander (line 20)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::Folded<Brand>::apply (line 14)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,A,A,Folded<Brand>>::folded (line 234)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::clone (line 8)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::evaluate (line 14)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFold<'a,P,I,S,T,A,B,F>::new (line 136)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,A,
Folded<Brand>>::folded (line 276)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::clone (line 8)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::evaluate (line 14)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_fold.rs - types::optics::indexed_fold::inner::IndexedFoldPrime<'a,P,I,S,A,F>::new (line 536)
    fp-library/src/types/optics/indexed_getter.rs - types::optics::indexed_getter::inner::IndexedGetter<'a,P,I,S,A>::evaluate (line 14)
    fp-library/src/types/optics/indexed_getter.rs - types::optics::indexed_getter::inner::IndexedGetter<'a,P,I,S,A>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_getter.rs - types::optics::indexed_getter::inner::IndexedGetter<'a,P,I,S,A>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,P,I,S,S,A,A>::evaluate (line 14)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,P,I,S,T,A,B>::evaluate (line 11)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,Q,I,S,T,A,B>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLens<'a,Q,I,S,T,A,B>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,P,I,S,A>::evaluate (line 11)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,P,I,S,A>::evaluate (line 14)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,Q,I,S,A>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_lens.rs - types::optics::indexed_lens::inner::IndexedLensPrime<'a,Q,I,S,A>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
<BrandasKind_cdc7cd43dac7585f>::Of<'a,B>,A,B,Mapped<Brand>>::mapped (line 293)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,S,T,A,B,F>::evaluate (line 11)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,S,T,A,B,F>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetter<'a,P,I,S,T,A,B,F>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,A,
Mapped<Brand>>::mapped (line 492)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,S,A,F>::evaluate (line 11)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,S,A,F>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::IndexedSetterPrime<'a,P,I,S,A,F>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_setter.rs - types::optics::indexed_setter::inner::Mapped<Brand>::apply (line 12)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
<BrandasKind_cdc7cd43dac7585f>::Of<'a,B>,A,B,Traversed<Brand>>::traversed (line 168)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,S,A,A,F>::evaluate (line 14)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::clone (line 8)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::evaluate (line 11)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::evaluate (line 13)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,P,I,S,T,A,B,F>::new (line 302)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,Q,I,S,T,A,B,F>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversal<'a,Q,I,S,T,A,B,F>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,<BrandasKind_cdc7cd43dac7585f>::Of<'a,A>,
A,Traversed<Brand>>::traversed (line 210)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::clone (line 8)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::evaluate (line 11)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::evaluate (line 13)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::evaluate (line 14)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,P,I,S,A,F>::new (line 693)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,Q,I,S,A,F>::evaluate_indexed (line 11)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::IndexedTraversalPrime<'a,Q,I,S,A,F>::evaluate_indexed_discards_focus (line 11)
    fp-library/src/types/optics/indexed_traversal.rs - types::optics::indexed_traversal::inner::Traversed<Brand>::apply (line 14)

test result: FAILED. 899 passed; 62 failed; 8 ignored; 0 measured; 0 filtered out; finished in 7.93s

all doctests ran in 8.22s; merged doctests compilation took 0.27s
error: doctest failed, to rerun pass `-p fp-library --doc`