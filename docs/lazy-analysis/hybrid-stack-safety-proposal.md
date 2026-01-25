# Hybrid Stack-Safety Design Proposal

## Executive Summary

This document proposes a hybrid stack-safety design for `fp-library` that combines the best aspects of:

1. **Cats-style Eval** — Ergonomic user-facing API with `pure`, `defer`, `flatMap` combinators
2. **PureScript MonadRec/Trampoline** — Type class interface (`MonadRec`) and O(1) left-associated bind performance via CatList

The design addresses the O(n²) worst-case bind performance of the current [stack-safe evaluation proposal](stack-safe-evaluation-proposal.md) while maintaining compatibility with the [dual-type design proposal](dual-type-design-proposal.md).

### Two-Type Architecture

To resolve the fundamental tension between **HKT lifetime requirements** and **stack-safe type erasure**, this proposal separates concerns into two distinct monadic types:

| Type              | Implementation       | HKT Compatible | Stack-Safe | `'static` Required |
| ----------------- | -------------------- | -------------- | ---------- | ------------------ |
| **`Eval<'a, A>`** | Closure wrapper      | ✅ Yes         | ❌ No      | ❌ No              |
| **`Task<A>`**     | Free monad + CatList | ❌ No          | ✅ Yes     | ✅ Yes             |

- **`Eval<'a, A>`**: A lightweight closure-based monad for HKT abstractions, local glue code, and composing logic with borrowed references. **Not stack-safe** for deep recursion.

- **`Task<A>`**: The "heavy-duty" runtime monad using `Free<ThunkF, A>` with CatList-based bind stack. **Stack-safe** for unlimited recursion, but requires `A: 'static` due to type erasure via `Box<dyn Any>`.

This split acknowledges that no single type can satisfy all constraints simultaneously in Rust.

## Table of Contents

1. [Overview and Goals](#1-overview-and-goals)
2. [CatQueue: O(1) Amortized Double-Ended Queue](#2-catqueue-o1-amortized-double-ended-queue)
3. [CatList: O(1) Catenable List](#3-catlist-o1-catenable-list)
4. [Step Type and MonadRec Trait](#4-step-type-and-monadrec-trait)
5. [Free Monad with CatList-Based Bind Stack](#5-free-monad-with-catlist-based-bind-stack)
6. [The `'static` Constraint: Analysis and Alternatives](#6-the-static-constraint-analysis-and-alternatives)
7. [Task and Eval APIs: Two-Type Architecture](#7-task-api-stack-safe-evaluation-requires-static)
8. [Integration with Dual-Type Design](#8-integration-with-dual-type-design)
9. [Integration with HKT System](#9-integration-with-hkt-system)
10. [Performance Characteristics](#10-performance-characteristics)
11. [Implementation Checklist](#11-implementation-checklist)

---

## 1. Overview and Goals

### 1.1 Problem Statement

The [current stack-safe evaluation proposal](stack-safe-evaluation-proposal.md) uses a `Vec`-based continuation stack:

```rust
pub fn run(self) -> A {
    let mut stack: Vec<Box<dyn FnOnce(Box<dyn Any>) -> Eval<Box<dyn Any>>>> = Vec::new();
    // ...
}
```

This approach has a critical performance issue: **left-associated binds degrade to O(n²)**.

Consider this pattern:

```rust
let eval = Eval::pure(0);
for i in 0..n {
    eval = eval.flat_map(|x| Eval::pure(x + 1));
}
eval.run()
```

Each `flat_map` creates a new `FlatMap` node. When `run()` executes:

1. It pushes all continuations onto the stack: O(n) operations
2. It pops and executes each continuation: O(n) operations
3. **But**: Each `flat_map` call during construction traverses the existing structure

The PureScript Free monad solves this with "Reflection without Remorse" — using a CatList to achieve O(1) amortized bind operations.

### 1.2 Design Goals

| Goal                     | Description                                     | Approach                                    |
| ------------------------ | ----------------------------------------------- | ------------------------------------------- |
| **Stack Safety**         | No stack overflow regardless of recursion depth | Trampoline-style iterative evaluation       |
| **O(1) Bind**            | Left-associated binds should not degrade        | CatList-based continuation queue            |
| **Ergonomic API**        | Clean user-facing combinators                   | Task type with `pure`, `defer`, `flat_map`  |
| **Type Class Interface** | Generic recursion via HKT                       | `MonadRec` trait (Eval only, not Task)      |
| **HKT Integration**      | Works with existing type class system           | Eval implements HKT traits; Task does not   |
| **Three-Layer Design**   | Computation/HKT/caching separation              | Task + Eval for computation, Memo for cache |
| **Minimal Dependencies** | Use std types where possible                    | Custom CatList/CatQueue using std `Vec`     |

### 1.3 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           User-Facing API                                   │
│                                                                             │
│  Task (Stack-Safe)                    │  Eval (HKT-Compatible)              │
│  Task::pure(a)                        │  Eval::new(|| a)                    │
│  Task::defer(|| ta)                   │  Eval::pure(a)                      │
│  task.flat_map(g)                     │  eval.flat_map(g)                   │
└─────────────────────────────────────────────────────────────────────────────┘
                              │                          │
                              ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Internal Representation                              │
│                                                                             │
│  Task: Free<ThunkF, A> + CatList      │  Eval: Box<dyn FnOnce() -> A>       │
│        Trampolined, unlimited depth   │        Nested closures, ~8000 limit │
└─────────────────────────────────────────────────────────────────────────────┘
                              │                          │
                              ▼                          ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Evaluation Engine                                   │
│                                                                             │
│  Task: iterative loop consuming        │  Eval: direct closure execution    │
│        CatList continuations           │        grows call stack            │
└─────────────────────────────────────────────────────────────────────────────┘
                              │                          │
                              └────────────┬─────────────┘
                                           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Caching Layer (Optional)                              │
│              Memo<A> wrapping Task::run() or Eval::run() result             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.4 Key Design Decisions

#### Decision 1: Use CatList Instead of Vec

**Rationale**: CatList provides O(1) amortized `snoc` (append to end) and `uncons` (remove from front), which are the critical operations for a continuation queue. A `Vec` provides O(1) amortized `push` but O(n) `remove(0)`.

**Trade-off**: CatList has higher constant factors than Vec due to indirection, but the asymptotic improvement dominates for deep recursion.

#### Decision 2: Implement CatQueue Using Two Vecs

**Rationale**: The PureScript CatQueue uses two linked lists. In Rust, we can use two `Vec`s to achieve the same amortized bounds with better cache locality. This is the "banker's queue" approach.

**Trade-off**: Still O(n) worst case for a single `uncons`, but O(1) amortized over a sequence of operations.

#### Decision 3: Type Erasure via `Box<dyn Any>`

**Rationale**: Rust cannot express existential types directly. The continuation chain `a -> Eval<b> -> Eval<c> -> ...` has varying intermediate types. We must erase to `Box<dyn Any>` and downcast.

**Trade-off**: Runtime type checking overhead, but this is unavoidable without GATs or other advanced type system features.

#### Decision 4: MonadRec as a Type Class

**Rationale**: Following PureScript, we expose `MonadRec` as a type class so that other monads (Option, Result, etc.) can implement stack-safe recursion. This is more compositional than embedding trampolining only in `Eval`.

**Trade-off**: Slightly more complex API, but enables powerful patterns like stack-safe state machines.

### 1.5 Comparison with Alternatives

| Approach                                             | Bind Complexity    | Stack Safety | API Ergonomics | Type Safety     | HKT Compat |
| ---------------------------------------------------- | ------------------ | ------------ | -------------- | --------------- | ---------- |
| Direct closures                                      | O(1)               | ❌ Overflow  | ✅ Simple      | ✅ Full         | ❌ No      |
| Previous proposal's version of Cats Eval (Vec stack) | O(n²) worst        | ✅ Safe      | ✅ Simple      | ⚠️ Type erasure | ❌ No      |
| **Task (this proposal, CatList)**                    | **O(1) amortized** | ✅ Safe      | ✅ Simple      | ⚠️ Type erasure | ❌ No      |
| **Eval (this proposal, closures)**                   | O(1)               | ❌ ~8000     | ✅ Simple      | ✅ Full         | ✅ Yes     |
| Continuation monad                                   | O(1)               | ✅ Safe      | ⚠️ Complex     | ✅ Full         | ⚠️ Complex |

### 1.6 References

- [Reflection without Remorse](http://okmij.org/ftp/Haskell/zseq.pdf) (Ploeg & Kiselyov, 2014)
- [Purely Functional Data Structures](https://www.cs.cmu.edu/~rwh/theses/okasaki.pdf) (Okasaki, 1996) — CatList/CatQueue
- [Simple and Efficient Purely Functional Queues and Deques](https://www.westpoint.edu/eecs/SiteAssets/SitePages/Faculty%20Publication%20Documents/Okasaki/jfp95queue.pdf) (Okasaki, 1995)
- [Stack Safety for Free](https://functorial.com/stack-safety-for-free/index.pdf) (Freeman, 2015)

---

## 2. CatQueue: O(1) Amortized Double-Ended Queue

### 2.1 Design Rationale

CatQueue is the foundation of CatList. It provides a double-ended queue with O(1) amortized operations using the "two-list queue" technique from Okasaki's work.

**Key insight**: A queue can be implemented with two stacks (or in our case, two Vecs):

- `front`: Elements ready to be dequeued (in order)
- `back`: Elements recently enqueued (in reverse order)

When `front` is empty and we need to dequeue, we reverse `back` and swap it into `front`. This reversal is O(n), but it happens at most once per element over its lifetime, giving O(1) amortized cost.

### 2.2 PureScript Source Analysis

From [`CatQueue.purs`](CatQueue.purs):

```purescript
data CatQueue a = CatQueue (List a) (List a)

uncons :: forall a. CatQueue a -> Maybe (Tuple a (CatQueue a))
uncons (CatQueue Nil Nil) = Nothing
uncons (CatQueue Nil r) = uncons (CatQueue (reverse r) Nil)
uncons (CatQueue (Cons a as) r) = Just (Tuple a (CatQueue as r))

snoc :: forall a. CatQueue a -> a -> CatQueue a
snoc (CatQueue l r) a = CatQueue l (Cons a r)
```

### 2.3 Rust Implementation

````rust
use std::collections::VecDeque;

/// A double-ended queue with O(1) amortized operations.
///
/// This is a "banker's queue" implementation using two `Vec`s.
/// - `front`: Elements in FIFO order (head is next to dequeue)
/// - `back`: Elements in LIFO order (to be reversed when front empties)
///
/// # Complexity
///
/// | Operation | Amortized | Worst Case |
/// |-----------|-----------|------------|
/// | `snoc`    | O(1)      | O(1)       |
/// | `cons`    | O(1)      | O(1)       |
/// | `uncons`  | O(1)      | O(n)       |
/// | `unsnoc`  | O(1)      | O(n)       |
///
/// # Example
///
/// ```rust
/// let mut q = CatQueue::empty();
/// q = q.snoc(1).snoc(2).snoc(3);
///
/// let (a, q) = q.uncons().unwrap();
/// assert_eq!(a, 1);
///
/// let (b, q) = q.uncons().unwrap();
/// assert_eq!(b, 2);
/// ```
#[derive(Clone, Debug)]
pub struct CatQueue<A> {
    /// Elements ready to be dequeued (in order).
    front: Vec<A>,
    /// Elements recently enqueued (in reverse order).
    back: Vec<A>,
}

impl<A> Default for CatQueue<A> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<A> CatQueue<A> {
    /// Creates an empty queue.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub const fn empty() -> Self {
        CatQueue {
            front: Vec::new(),
            back: Vec::new(),
        }
    }

    /// Returns `true` if the queue contains no elements.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.front.is_empty() && self.back.is_empty()
    }

    /// Returns the number of elements in the queue.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn len(&self) -> usize {
        self.front.len() + self.back.len()
    }

    /// Creates a queue containing a single element.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn singleton(a: A) -> Self {
        CatQueue {
            front: vec![a],
            back: Vec::new(),
        }
    }

    /// Appends an element to the front of the queue.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn cons(mut self, a: A) -> Self {
        self.front.push(a);
        // Note: This puts 'a' at the end of front, but we read from the end
        // Actually, we need to reverse our mental model:
        // front is stored in reverse order (last element is head)
        self
    }

    /// Appends an element to the back of the queue.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn snoc(mut self, a: A) -> Self {
        self.back.push(a);
        self
    }

    /// Removes and returns the first element.
    ///
    /// Returns `None` if the queue is empty.
    ///
    /// # Complexity
    /// O(1) amortized, O(n) worst case
    pub fn uncons(mut self) -> Option<(A, Self)> {
        if self.front.is_empty() {
            if self.back.is_empty() {
                return None;
            }
            // Reverse back into front
            self.back.reverse();
            std::mem::swap(&mut self.front, &mut self.back);
        }

        // Pop from the end of front (which is the "head" in our representation)
        let a = self.front.pop()?;
        Some((a, self))
    }

    /// Removes and returns the last element.
    ///
    /// Returns `None` if the queue is empty.
    ///
    /// # Complexity
    /// O(1) amortized, O(n) worst case
    pub fn unsnoc(mut self) -> Option<(A, Self)> {
        if self.back.is_empty() {
            if self.front.is_empty() {
                return None;
            }
            // Reverse front into back
            self.front.reverse();
            std::mem::swap(&mut self.front, &mut self.back);
        }

        let a = self.back.pop()?;
        Some((a, self))
    }
}

// Iteration support for convenient use
impl<A> IntoIterator for CatQueue<A> {
    type Item = A;
    type IntoIter = CatQueueIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        CatQueueIter { queue: self }
    }
}

/// An iterator that consumes a `CatQueue`.
pub struct CatQueueIter<A> {
    queue: CatQueue<A>,
}

impl<A> Iterator for CatQueueIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        let (a, rest) = std::mem::take(&mut self.queue).uncons()?;
        self.queue = rest;
        Some(a)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.queue.len();
        (len, Some(len))
    }
}

impl<A> ExactSizeIterator for CatQueueIter<A> {}
````

### 2.4 Design Decisions for Rust

#### Decision: Use `Vec` Instead of Linked List

**Rationale**: PureScript uses linked lists (`List a`), but Rust's `Vec` offers:

- Better cache locality
- Amortized O(1) `push` and `pop`
- No allocation per element

The trade-off is that `Vec` requires elements to be contiguous, but since we only access from the ends, this works perfectly.

#### Decision: Store Front in Reverse Order

**Rationale**: We store `front` so that `pop()` removes the logical "head". This means:

- `cons(a)` → `front.push(a)` — O(1)
- `uncons()` → `front.pop()` — O(1)
- When `front` is empty, `reverse(back)` gives us the correct order

This matches PureScript's behavior but uses Vec's natural operations.

#### Decision: Consuming Methods

**Rationale**: `uncons` and `unsnoc` consume `self` and return the modified queue. This matches the functional style and avoids interior mutability. For iteration, we provide `IntoIterator`.

### 2.5 Correctness Argument

**Invariant**: Elements in `front` (read from end) followed by elements in `back` (read from start after reversal) form the logical queue order.

**Amortized Analysis**: Each element is:

1. Pushed to `back`: O(1)
2. Moved to `front` (via reversal): O(1) amortized (happens once per element)
3. Popped from `front`: O(1)

Total cost per element: O(1) amortized.

---

## 3. CatList: O(1) Catenable List

### 3.1 Design Rationale

CatList is a "catenable list" — a list that supports O(1) concatenation. This is the key data structure that enables O(1) bind operations in the Free monad.

**Key insight**: A CatList is either empty, or a head element plus a queue of CatLists. When we concatenate two CatLists, we just enqueue the second onto the first's queue — no traversal needed.

```
CatList a = CatNil | CatCons a (CatQueue (CatList a))
```

### 3.2 PureScript Source Analysis

From [`CatList.purs`](CatList.purs):

```purescript
data CatList a = CatNil | CatCons a (Q.CatQueue (CatList a))

-- O(1) append via queue snoc
link :: forall a. CatList a -> CatList a -> CatList a
link CatNil cat = cat
link cat CatNil = cat
link (CatCons a q) cat = CatCons a (Q.snoc q cat)

-- O(1) amortized uncons (may trigger internal foldr)
uncons :: forall a. CatList a -> Maybe (Tuple a (CatList a))
uncons CatNil = Nothing
uncons (CatCons a q) = Just (Tuple a (if Q.null q then CatNil else (foldr link CatNil q)))
```

The `uncons` operation is subtle: when we've exhausted the head, we need to combine all the queued sublists into one. This uses `foldr link CatNil`, which is O(k) where k is the number of sublists, but amortizes to O(1) per element.

### 3.3 Rust Implementation

````rust
/// A catenable list with O(1) append and O(1) amortized uncons.
///
/// This is the "Reflection without Remorse" data structure that enables
/// O(1) left-associated bind operations in the Free monad.
///
/// # Structure
///
/// A CatList is either:
/// - `Nil`: Empty
/// - `Cons(head, sublists)`: A head element plus a queue of CatLists
///
/// # Complexity
///
/// | Operation   | Amortized | Worst Case |
/// |-------------|-----------|------------|
/// | `singleton` | O(1)      | O(1)       |
/// | `append`    | O(1)      | O(1)       |
/// | `snoc`      | O(1)      | O(1)       |
/// | `cons`      | O(1)      | O(1)       |
/// | `uncons`    | O(1)      | O(n)       |
///
/// # Example
///
/// ```rust
/// let list = CatList::singleton(1)
///     .snoc(2)
///     .snoc(3)
///     .append(CatList::singleton(4));
///
/// let mut result = Vec::new();
/// let mut current = list;
/// while let Some((head, tail)) = current.uncons() {
///     result.push(head);
///     current = tail;
/// }
/// assert_eq!(result, vec![1, 2, 3, 4]);
/// ```
#[derive(Clone, Debug)]
pub enum CatList<A> {
    /// Empty list
    Nil,
    /// Head element plus queue of sublists
    Cons(A, CatQueue<CatList<A>>),
}

impl<A> Default for CatList<A> {
    fn default() -> Self {
        CatList::Nil
    }
}

impl<A> CatList<A> {
    /// Creates an empty CatList.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub const fn empty() -> Self {
        CatList::Nil
    }

    /// Returns `true` if the list is empty.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, CatList::Nil)
    }

    /// Creates a CatList with a single element.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn singleton(a: A) -> Self {
        CatList::Cons(a, CatQueue::empty())
    }

    /// Appends an element to the front of the list.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn cons(self, a: A) -> Self {
        Self::link(CatList::singleton(a), self)
    }

    /// Appends an element to the back of the list.
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn snoc(self, a: A) -> Self {
        Self::link(self, CatList::singleton(a))
    }

    /// Concatenates two CatLists.
    ///
    /// This is the key operation that makes CatList special:
    /// concatenation is O(1), not O(n).
    ///
    /// # Complexity
    /// O(1)
    pub fn append(self, other: Self) -> Self {
        Self::link(self, other)
    }

    /// Internal linking operation.
    ///
    /// Links two CatLists by enqueueing the second onto the first's sublist queue.
    fn link(left: Self, right: Self) -> Self {
        match (left, right) {
            (CatList::Nil, cat) => cat,
            (cat, CatList::Nil) => cat,
            (CatList::Cons(a, q), cat) => CatList::Cons(a, q.snoc(cat)),
        }
    }

    /// Removes and returns the first element.
    ///
    /// Returns `None` if the list is empty.
    ///
    /// # Complexity
    /// O(1) amortized, O(n) worst case
    ///
    /// The worst case occurs when we need to flatten the sublist queue.
    /// However, each element is only involved in flattening once during
    /// its lifetime, so the amortized cost is O(1).
    pub fn uncons(self) -> Option<(A, Self)> {
        match self {
            CatList::Nil => None,
            CatList::Cons(a, q) => {
                if q.is_empty() {
                    Some((a, CatList::Nil))
                } else {
                    // Flatten the queue of sublists into a single CatList
                    let tail = Self::flatten_queue(q);
                    Some((a, tail))
                }
            }
        }
    }

    /// Flattens a queue of CatLists into a single CatList.
    ///
    /// This is equivalent to `foldr link CatNil queue` in PureScript.
    ///
    /// # Implementation Note
    ///
    /// We use an iterative approach with an explicit stack to avoid
    /// stack overflow on deeply nested structures.
    fn flatten_queue(queue: CatQueue<CatList<A>>) -> Self {
        // Collect all sublists
        let sublists: Vec<CatList<A>> = queue.into_iter().collect();

        // Right fold: link(list[0], link(list[1], ... link(list[n-1], Nil)))
        // We process from right to left
        sublists.into_iter().rev().fold(CatList::Nil, |acc, list| {
            Self::link(list, acc)
        })
    }

    /// Returns the number of elements.
    ///
    /// # Complexity
    /// O(n)
    ///
    /// Note: This is expensive because CatList doesn't track length.
    /// Use only for debugging/testing.
    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut current = self.clone();
        while let Some((_, tail)) = current.uncons() {
            count += 1;
            current = tail;
        }
        count
    }
}

// Iteration support
impl<A> IntoIterator for CatList<A> {
    type Item = A;
    type IntoIter = CatListIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        CatListIter { list: self }
    }
}

/// An iterator that consumes a `CatList`.
pub struct CatListIter<A> {
    list: CatList<A>,
}

impl<A> Iterator for CatListIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        let (head, tail) = std::mem::take(&mut self.list).uncons()?;
        self.list = tail;
        Some(head)
    }
}

// FromIterator for easy construction
impl<A> FromIterator<A> for CatList<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        iter.into_iter().fold(CatList::Nil, |acc, a| acc.snoc(a))
    }
}
````

### 3.4 Design Decisions for Rust

#### Decision: Enum Representation

**Rationale**: We use a Rust enum directly mirroring PureScript's algebraic data type. This gives us:

- Pattern matching for clean code
- Zero-cost abstraction (no boxing for the enum discriminant)
- Clear structural representation

#### Decision: Consuming vs Borrowing `uncons`

**Rationale**: We chose consuming `uncons(self) -> Option<(A, Self)>` because:

1. CatList is typically used linearly (process once, discard)
2. Avoids lifetime complexity
3. Matches PureScript's functional style

If shared access is needed, wrap in `Rc<RefCell<CatList<A>>>`.

#### Decision: Iterative `flatten_queue`

**Rationale**: PureScript's `foldr link CatNil q` is recursive. In Rust, we must be careful about stack depth. Our implementation:

1. Collects sublists into a Vec (iterative)
2. Right-folds using `.rev().fold()` (iterative)

This ensures stack safety even for deeply nested CatLists.

### 3.5 Amortized Analysis

**Claim**: A sequence of n `snoc` operations followed by n `uncons` operations takes O(n) total time.

**Proof sketch**:

1. Each `snoc` is O(1) — just appends to queue
2. Each element is involved in at most one `link` during `flatten_queue`
3. Using the "banker's method": each element pays for its own flattening

**Potential function**: Φ(CatList) = number of sublists in all queues.

- `snoc`: increases Φ by 1, actual cost O(1), amortized cost O(1)
- `uncons`: decreases Φ by k (number of sublists flattened), actual cost O(k), amortized cost O(1)

### 3.6 Why Not Use VecDeque?

One might ask: why not just use `std::collections::VecDeque`?

**Answer**: VecDeque has O(n) concatenation, which defeats the purpose. The key feature of CatList is O(1) concatenation, which requires the nested structure with deferred flattening.

| Operation  | CatList        | VecDeque       |
| ---------- | -------------- | -------------- |
| push_back  | O(1)           | O(1) amortized |
| pop_front  | O(1) amortized | O(1)           |
| **concat** | **O(1)**       | **O(n)**       |

For the Free monad's bind stack, we need O(1) concat to avoid O(n²) left-bind degradation.

---

## 4. Step Type and MonadRec Trait

### 4.1 Design Rationale

The `Step` type and `MonadRec` trait are the foundation of stack-safe recursion. Rather than embedding trampolining logic into specific types, we define a generic interface that any monad can implement.

**Key insight from PureScript**:

```purescript
data Step a b = Loop a | Done b

class Monad m <= MonadRec m where
  tailRecM :: forall a b. (a -> m (Step a b)) -> a -> m b
```

The `tailRecM` function repeatedly applies `f` until it returns `Done`. The key constraint is that `m` must support this without growing the stack.

### 4.2 Step Type

````rust
/// Represents the result of a single step in a tail-recursive computation.
///
/// This type is fundamental to stack-safe recursion via `MonadRec`.
///
/// # Type Parameters
///
/// - `A`: The "loop" type - when we return `Loop(a)`, we continue with `a`
/// - `B`: The "done" type - when we return `Done(b)`, we're finished
///
/// # Example
///
/// ```rust
/// // Count down from n to 0, accumulating the sum
/// fn sum_to_zero(n: i32, acc: i32) -> Step<(i32, i32), i32> {
///     if n <= 0 {
///         Step::Done(acc)
///     } else {
///         Step::Loop((n - 1, acc + n))
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step<A, B> {
    /// Continue the loop with a new value
    Loop(A),
    /// Finish the computation with a final value
    Done(B),
}

impl<A, B> Step<A, B> {
    /// Returns `true` if this is a `Loop` variant.
    #[inline]
    pub fn is_loop(&self) -> bool {
        matches!(self, Step::Loop(_))
    }

    /// Returns `true` if this is a `Done` variant.
    #[inline]
    pub fn is_done(&self) -> bool {
        matches!(self, Step::Done(_))
    }

    /// Maps a function over the `Loop` variant.
    pub fn map_loop<C>(self, f: impl FnOnce(A) -> C) -> Step<C, B> {
        match self {
            Step::Loop(a) => Step::Loop(f(a)),
            Step::Done(b) => Step::Done(b),
        }
    }

    /// Maps a function over the `Done` variant.
    pub fn map_done<C>(self, f: impl FnOnce(B) -> C) -> Step<A, C> {
        match self {
            Step::Loop(a) => Step::Loop(a),
            Step::Done(b) => Step::Done(f(b)),
        }
    }

    /// Applies functions to both variants (bifunctor map).
    pub fn bimap<C, D>(
        self,
        f: impl FnOnce(A) -> C,
        g: impl FnOnce(B) -> D,
    ) -> Step<C, D> {
        match self {
            Step::Loop(a) => Step::Loop(f(a)),
            Step::Done(b) => Step::Done(g(b)),
        }
    }
}
````

### 4.3 MonadRec Trait

````rust
use crate::{Apply, kinds::*};

/// A type class for monads that support stack-safe tail recursion.
///
/// Any monad implementing `MonadRec` guarantees that `tail_rec_m` will not
/// overflow the stack, regardless of how many iterations are required.
///
/// # Laws
///
/// 1. **Equivalence to recursion**: For a total function `f: A -> M<Step<A, B>>`,
///    `tail_rec_m(f, a)` should produce the same result as the (potentially
///    stack-overflowing) recursive definition:
///    ```text
///    rec(a) = f(a).bind(|step| match step {
///        Step::Loop(a') => rec(a'),
///        Step::Done(b) => pure(b),
///    })
///    ```
///
/// 2. **Stack safety**: `tail_rec_m` must not overflow the stack for any
///    terminating `f`, even with millions of iterations.
///
/// # Example
///
/// ```rust
/// use fp_library::{classes::MonadRec, types::Step};
///
/// // Factorial using tail recursion
/// fn factorial<M: MonadRec>(n: u64) -> Apply!(M, u64) {
///     M::tail_rec_m(|(n, acc)| {
///         if n <= 1 {
///             M::pure(Step::Done(acc))
///         } else {
///             M::pure(Step::Loop((n - 1, n * acc)))
///         }
///     }, (n, 1))
/// }
/// ```
pub trait MonadRec: Monad {
    /// Performs tail-recursive monadic computation.
    ///
    /// Repeatedly applies `f` to the current state until `f` returns `Done`.
    ///
    /// # Type Parameters
    ///
    /// - `A`: The loop state type
    /// - `B`: The final result type
    ///
    /// # Parameters
    ///
    /// - `f`: A function that takes the current state and returns a monadic
    ///        `Step`, either continuing with `Loop(a)` or finishing with `Done(b)`.
    ///        **Must be `Clone`** because the function is called multiple times
    ///        across recursive iterations, with each iteration potentially
    ///        needing its own owned copy of the closure.
    /// - `a`: The initial state
    ///
    /// # Returns
    ///
    /// A monadic value containing the final result `B`
    ///
    /// # Clone Bound Rationale
    ///
    /// The `Clone` bound on `F` is necessary because:
    /// 1. Each recursive step needs to pass `f` to the next iteration
    /// 2. In trampolined implementations, `f` must be moved into closures
    ///    multiple times (once per `defer` or continuation)
    /// 3. Most closures naturally implement `Clone` when their captures do
    ///
    /// For closures that cannot implement `Clone`, use `tail_rec_m_shared`
    /// which wraps `f` in `Arc` internally (with a small performance cost).
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        a: A,
    ) -> Apply!(Self::Brand, B)
    where
        F: Fn(A) -> Apply!(Self::Brand, Step<A, B>) + Clone + 'a,
        Self::Brand: Kind_cdc7cd43dac7585f;  // type Of<'a, T: 'a>: 'a
}

/// Free function version of `tail_rec_m`.
pub fn tail_rec_m<'a, M, A: 'a, B: 'a, F>(
    f: F,
    a: A,
) -> Apply!(M::Brand, B)
where
    M: MonadRec,
    F: Fn(A) -> Apply!(M::Brand, Step<A, B>) + Clone + 'a,
    M::Brand: Kind_cdc7cd43dac7585f,
{
    M::tail_rec_m(f, a)
}

/// Arc-wrapped version of `tail_rec_m` for non-Clone closures.
///
/// This function wraps the provided closure in `Arc` internally, allowing
/// closures that don't implement `Clone` to be used with `tail_rec_m`.
///
/// # Trade-offs
///
/// - **Pro**: Works with any `Fn` closure, not just `Clone` ones
/// - **Con**: Small overhead from Arc allocation and atomic reference counting
///
/// # When to Use
///
/// Use this when your closure captures non-Clone state:
///
/// ```rust
/// // This closure captures a non-Clone Sender
/// let sender: Sender<i32> = /* ... */;
/// tail_rec_m_shared::<EvalBrand, _, _, _>(
///     |n| {
///         sender.send(n).ok();
///         if n == 0 { Eval::now(Step::Done(())) }
///         else { Eval::now(Step::Loop(n - 1)) }
///     },
///     100
/// )
/// ```
pub fn tail_rec_m_shared<'a, M, A: 'a, B: 'a, F>(
    f: F,
    a: A,
) -> Apply!(M::Brand, B)
where
    M: MonadRec,
    F: Fn(A) -> Apply!(M::Brand, Step<A, B>) + 'a,
    M::Brand: Kind_cdc7cd43dac7585f,
{
    use std::sync::Arc;

    // Wrap f in Arc to make it Clone
    let f = Arc::new(f);

    // Create a Clone wrapper that delegates to the Arc
    let wrapper = move |a: A| {
        let f = Arc::clone(&f);
        f(a)
    };

    M::tail_rec_m(wrapper, a)
}
````

### 4.4 Standard Implementations

#### Identity (trivial case)

```rust
impl MonadRec for IdentityInstance {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        f: impl Fn(A) -> Identity<Step<A, B>> + 'a,
        mut a: A,
    ) -> Identity<B> {
        loop {
            match f(a).0 {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Identity(b),
            }
        }
    }
}
```

#### Option

```rust
impl MonadRec for OptionInstance {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        f: impl Fn(A) -> Option<Step<A, B>> + 'a,
        mut a: A,
    ) -> Option<B> {
        loop {
            match f(a)? {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Some(b),
            }
        }
    }
}
```

#### Result

```rust
impl<E> MonadRec for ResultInstance<E> {
    fn tail_rec_m<'a, A: 'a, B: 'a>(
        f: impl Fn(A) -> Result<Step<A, B>, E> + 'a,
        mut a: A,
    ) -> Result<B, E> {
        loop {
            match f(a)? {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Ok(b),
            }
        }
    }
}
```

### 4.5 Why MonadRec Matters

Without stack-safe recursion, this function overflows:

```rust
// Using Eval (NOT stack-safe) - this will overflow for large n!
fn countdown_eval(n: u64) -> Eval<'static, u64> {
    if n == 0 {
        Eval::pure(0)
    } else {
        Eval::new(move || countdown_eval(n - 1).run())  // Stack overflow!
    }
}
```

With `Task::tail_rec_m`, we achieve guaranteed stack safety:

```rust
// Using Task (stack-safe) - works for any n
fn countdown(n: u64) -> Task<u64> {
    Task::tail_rec_m(|n| {
        if n == 0 {
            Task::now(Step::Done(0))
        } else {
            Task::now(Step::Loop(n - 1))
        }
    }, n)
}
```

The key difference: instead of building a chain of deferred computations, we express the loop _structure_ explicitly with `Step`, and let `tail_rec_m` handle it iteratively.

### 4.6 Relationship to Trampoline

`Trampoline` is essentially `MonadRec` specialized to the "thunk" monad:

```rust
type Trampoline<A> = Free<ThunkF, A>;

// Trampoline::done(a) ≈ Free::Pure(a)
// Trampoline::suspend(f) ≈ Free::Roll(ThunkF(f))
```

In this proposal, **`Task`** serves as our `Trampoline`, using the Free monad with CatList-based bind stack for guaranteed stack safety. Note that `Task` does NOT implement the HKT-based `MonadRec` trait (due to `'static` constraint conflicts), but provides equivalent standalone `tail_rec_m` methods.

The separate **`Eval<'a, A>`** type (closure-based) CAN implement HKT traits including `MonadRec`, but is NOT stack-safe for deep recursion.

---

## 5. Free Monad with CatList-Based Bind Stack

### 5.1 Design Rationale

The Free monad provides a generic way to build a monad from any functor `F`. The key insight of "Reflection without Remorse" is that by storing continuations in a CatList instead of nesting them directly, we achieve O(1) bind performance.

**PureScript's Free monad structure**:

```purescript
data Free f a = Pure a | Free (f (Free f a)) | Bind (Free f Val) (CatList (Val -> Free f Val))
```

The `Bind` constructor stores:

1. A suspended computation producing some value (type-erased as `Val`)
2. A CatList of continuations to apply (also type-erased)

### 5.2 Type-Erased Value Type

Since Rust's type system cannot express existential types directly, we use `Box<dyn Any>` for type erasure:

```rust
use std::any::Any;

/// A type-erased value used internally by Free.
///
/// This is the equivalent of PureScript's `Val` type or the polymorphic
/// existential in the "Reflection without Remorse" paper.
pub type Val = Box<dyn Any + Send>;

/// A type-erased continuation: Val -> Free<F, Val>
pub type ErasedCont<F> = Box<dyn FnOnce(Val) -> Free<F, Val> + Send>;
```

### 5.3 Free Monad Implementation

````rust
use std::any::Any;
use std::marker::PhantomData;

/// A type-erased value for internal use.
type Val = Box<dyn Any + Send>;

/// A type-erased continuation.
type Cont<F> = Box<dyn FnOnce(Val) -> Free<F, Val> + Send>;

/// The Free monad with O(1) bind via CatList.
///
/// This implementation follows "Reflection without Remorse" to ensure
/// that left-associated binds do not degrade performance.
///
/// # Type Parameters
///
/// - `F`: The base functor (must implement `Functor`)
/// - `A`: The result type
///
/// # Variants
///
/// - `Pure(a)`: A finished computation with result `a`
/// - `Roll(f)`: A suspended computation `f` containing a `Free<F, A>`
/// - `Bind(free, conts)`: A computation `free` with continuations `conts`
///
/// # Example
///
/// ```rust
/// // ThunkF is () -> A, making Free<ThunkF, A> a Trampoline
/// let free = Free::pure(42)
///     .flat_map(|x| Free::pure(x + 1))
///     .flat_map(|x| Free::pure(x * 2));
///
/// assert_eq!(free.run(), 86);
/// ```
pub enum Free<F, A>
where
    F: Functor,
{
    /// A pure value, computation finished.
    Pure(A),

    /// A suspended effect containing a continuation.
    Roll(Apply!(F::Brand, Free<F, A>)),

    /// A computation with a CatList of continuations.
    /// Uses type erasure internally for heterogeneous continuation chains.
    Bind {
        /// The initial computation (type-erased)
        head: Box<Free<F, Val>>,
        /// The queue of continuations (type-erased)
        conts: CatList<Cont<F>>,
        /// Phantom data for the result type
        _marker: PhantomData<A>,
    },
}

impl<F: Functor, A: 'static + Send> Free<F, A> {
    /// Creates a pure Free value.
    #[inline]
    pub fn pure(a: A) -> Self {
        Free::Pure(a)
    }

    /// Creates a suspended computation from a functor value.
    pub fn roll(fa: Apply!(F::Brand, Free<F, A>)) -> Self {
        Free::Roll(fa)
    }

    /// Monadic bind (flatMap) with O(1) complexity.
    ///
    /// This is where the CatList magic happens: instead of nesting
    /// the continuation, we snoc it onto the CatList.
    pub fn flat_map<B: 'static + Send>(
        self,
        f: impl FnOnce(A) -> Free<F, B> + 'static + Send,
    ) -> Free<F, B> {
        // Type-erase the continuation
        let erased_f: Cont<F> = Box::new(move |val: Val| {
            let a: A = *val.downcast().expect("Type mismatch in Free::flat_map");
            let free_b: Free<F, B> = f(a);
            free_b.erase_type()
        });

        match self {
            // Pure: create a Bind with this continuation
            Free::Pure(a) => {
                let head: Free<F, Val> = Free::Pure(Box::new(a) as Val);
                Free::Bind {
                    head: Box::new(head),
                    conts: CatList::singleton(erased_f),
                    _marker: PhantomData,
                }
            }

            // Roll: wrap in a Bind
            Free::Roll(fa) => {
                let head = Free::Roll(fa).erase_type_boxed();
                Free::Bind {
                    head,
                    conts: CatList::singleton(erased_f),
                    _marker: PhantomData,
                }
            }

            // Bind: snoc the new continuation onto the CatList (O(1)!)
            Free::Bind { head, conts, .. } => {
                Free::Bind {
                    head,
                    conts: conts.snoc(erased_f),
                    _marker: PhantomData,
                }
            }
        }
    }

    /// Converts to type-erased form.
    fn erase_type(self) -> Free<F, Val> {
        match self {
            Free::Pure(a) => Free::Pure(Box::new(a) as Val),
            Free::Roll(fa) => {
                // Map over the functor to erase the inner type
                let erased = F::map(|inner: Free<F, A>| inner.erase_type(), fa);
                Free::Roll(erased)
            }
            Free::Bind { head, conts, .. } => Free::Bind {
                head,
                conts,
                _marker: PhantomData,
            },
        }
    }

    /// Converts to boxed type-erased form.
    fn erase_type_boxed(self) -> Box<Free<F, Val>> {
        Box::new(self.erase_type())
    }
}
````

### 5.4 The Run Loop (Interpreter)

The evaluation loop processes the Free structure iteratively:

```rust
impl<F, A> Free<F, A>
where
    F: Functor,
    A: 'static + Send,
{
    /// Executes the Free computation, returning the final result.
    ///
    /// This is the "trampoline" that iteratively processes the
    /// CatList of continuations without growing the stack.
    ///
    /// # Requirements
    ///
    /// `F` must be a "runnable" functor (e.g., ThunkF where we can
    /// force the thunk to get the inner value).
    pub fn run(self) -> A
    where
        F: Runnable,
    {
        // Start with a type-erased version
        let mut current: Free<F, Val> = self.erase_type();
        let mut conts: CatList<Cont<F>> = CatList::empty();

        loop {
            match current {
                Free::Pure(val) => {
                    // Try to apply the next continuation
                    match conts.uncons() {
                        Some((cont, rest)) => {
                            current = cont(val);
                            conts = rest;
                        }
                        None => {
                            // No more continuations - we're done!
                            return *val.downcast::<A>()
                                .expect("Type mismatch in Free::run final downcast");
                        }
                    }
                }

                Free::Roll(fa) => {
                    // Run the effect to get the inner Free
                    current = F::run_effect(fa);
                }

                Free::Bind { head, conts: inner_conts, .. } => {
                    // Merge the inner continuations with outer ones
                    // This is where CatList's O(1) append shines!
                    current = *head;
                    conts = inner_conts.append(conts);
                }
            }
        }
    }
}

/// A functor whose effects can be "run" to produce the inner value.
pub trait Runnable: Functor {
    /// Runs the effect, producing the inner value.
    fn run_effect<A>(fa: Apply!(Self::Brand, A)) -> A;
}
```

### 5.5 ThunkF: The Thunk Functor

For `Eval`, we use `ThunkF` — a functor representing suspended computations:

```rust
/// A thunk functor: `() -> A`
///
/// This is the simplest functor for building a trampoline.
/// `Free<ThunkF, A>` is equivalent to PureScript's `Trampoline`.
pub struct ThunkF;

/// The concrete type for ThunkF applied to A.
pub struct Thunk<A>(Box<dyn FnOnce() -> A + Send>);

impl<A> Thunk<A> {
    pub fn new(f: impl FnOnce() -> A + Send + 'static) -> Self {
        Thunk(Box::new(f))
    }

    pub fn force(self) -> A {
        (self.0)()
    }
}

// Brand for HKT
pub struct ThunkFBrand;

impl Kind_cdc7cd43dac7585f for ThunkFBrand {
    type Of<'a, A: 'a> = Thunk<A>;
}

impl Functor for ThunkFInstance {
    type Brand = ThunkFBrand;

    fn map<A, B>(f: impl FnOnce(A) -> B, fa: Thunk<A>) -> Thunk<B> {
        Thunk::new(move || f(fa.force()))
    }
}

impl Runnable for ThunkFInstance {
    fn run_effect<A>(fa: Thunk<A>) -> A {
        fa.force()
    }
}
```

### 5.6 Why This Achieves O(1) Bind

Consider this sequence of binds:

```rust
Free::pure(0)
    .flat_map(|x| Free::pure(x + 1))
    .flat_map(|x| Free::pure(x + 2))
    .flat_map(|x| Free::pure(x + 3))
```

**Traditional nested structure** (O(n²)):

```
FlatMap(FlatMap(FlatMap(Pure(0), f1), f2), f3)
```

Concatenating requires traversing to the innermost `Pure`.

**CatList structure** (O(1)):

```
Bind {
    head: Pure(0),
    conts: CatList[f1, f2, f3]
}
```

Each `flat_map` just does `conts.snoc(f)` — O(1)!

### 5.7 Memory and Performance Considerations

**Allocation**:

- Each continuation is boxed: `Box<dyn FnOnce(Val) -> Free<F, Val>>`
- Each value is boxed for type erasure: `Box<dyn Any>`

**Downcasting**:

- `downcast` is a simple discriminant check + pointer cast
- Extremely cheap, but adds a small constant factor

**CatList overhead**:

- The nested CatList structure adds indirection
- But this is amortized across all operations

**When to use**:

- Use `Free`/`Eval` for _deep_ recursion or _long_ chains (1000+ binds)
- For shallow chains (<100 binds), direct closures may be faster
- The crossover point depends on the specific use case

---

## 6. The `'static` Constraint: Analysis and Alternatives

### 6.1 Understanding the Constraint

The hybrid stack-safety design uses type erasure for heterogeneous continuation chains:

```rust
/// A type-erased value used internally by Free.
pub type Val = Box<dyn Any + Send>;
```

This definition has an important implication: **the underlying type `A` must be `'static`**. This is because `Any` is defined as:

```rust
pub trait Any: 'static {
    fn type_id(&self) -> TypeId;
}
```

The `'static` bound on `Any` means any type you want to erase to `Box<dyn Any>` must not contain non-`'static` references.

### 6.2 Practical Impact on Eval&lt;A&gt;

This constraint manifests in the `Eval<A>` API:

```rust
impl<A: 'static + Send> Eval<A> {
    pub fn now(a: A) -> Self { /* ... */ }
    pub fn later<F>(f: F) -> Self where F: FnOnce() -> A + Send + 'static { /* ... */ }
    pub fn flat_map<B: 'static + Send, F>(self, f: F) -> Eval<B> { /* ... */ }
}
```

This means:

| Works                | Does Not Work                                       |
| -------------------- | --------------------------------------------------- |
| `Eval<String>`       | `Eval<&str>` (borrowed)                             |
| `Eval<Vec<i32>>`     | `Eval<&[i32]>` (borrowed)                           |
| `Eval<Arc<Data>>`    | `Eval<&Data>` (borrowed)                            |
| `Eval<Result<T, E>>` | `Eval<T>` where `T: 'a` for some non-`'static` `'a` |

### 6.3 Why This is Generally NOT a Significant Limitation

Despite the `'static` requirement, this constraint is **acceptable for a functional programming library** for several reasons:

#### Reason 1: FP Emphasizes Owned Data

Functional programming inherently favors:

- **Immutable values** — typically owned, not borrowed
- **Pure transformations** — `A -> B` where both are concrete types
- **Composition** — chaining operations on owned data

In idiomatic FP Rust:

```rust
// Idiomatic: owned data throughout
Eval::later(|| compute_string())
    .flat_map(|s| Eval::now(s.len()))   // String -> usize, both owned
    .map(|len| len * 2)                  // usize -> usize, owned

// Rare: borrowed data in lazy contexts
// (typically you'd clone or own the data)
```

#### Reason 2: Closures Naturally Require Owned Captures

Deferred computations store closures. Closures that outlive their scope need `'static` data anyway:

```rust
// This closure needs 'static because it's stored
Eval::later(move || expensive_computation(&data))
// Even without type erasure, `data` needs 'static or to be moved/cloned
```

#### Reason 3: Established Libraries Have Similar Constraints

| Library      | Constraint                          | Reason               |
| ------------ | ----------------------------------- | -------------------- |
| Tokio        | `Future`s are `'static` for `spawn` | Task storage         |
| Rayon        | Work items are `'static`            | Thread pool transfer |
| Cats (Scala) | No issue (JVM GC handles lifetimes) | N/A                  |
| PureScript   | No issue (runtime manages memory)   | N/A                  |

Rust libraries dealing with deferred execution universally require `'static` for similar reasons.

#### Reason 4: Workarounds Exist

When you genuinely need non-`'static` data:

```rust
// Option 1: Clone the data
let data = borrowed_slice.to_vec();
Eval::now(data).flat_map(|v| /* ... */)

// Option 2: Use Arc for shared access
let data = Arc::new(borrowed_slice.to_vec());
Eval::now(data).flat_map(|d| /* ... */)

// Option 3: Structure code to avoid the need
fn process_sync(data: &[u8]) -> Result {
    // Do borrowed work synchronously
}
let result = process_sync(borrowed);
Eval::now(result)  // Only defer the owned result
```

### 6.4 Alternative Approaches Considered

Several alternative approaches could theoretically eliminate or reduce the `'static` constraint. Each was evaluated:

#### Alternative 1: async/await with Future

**Idea**: Use Rust's built-in async state machine generation:

```rust
pub struct Eval<A>(Pin<Box<dyn Future<Output = A> + Send>>);

impl<A: Send + 'static> Eval<A> {
    pub fn later<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = A> + Send,
    {
        Eval(Box::pin(async move { f().await }))
    }

    pub fn flat_map<B, F, Fut>(self, f: F) -> Eval<B>
    where
        B: Send + 'static,
        F: FnOnce(A) -> Fut + Send + 'static,
        Fut: Future<Output = A> + Send,
    {
        Eval(Box::pin(async move {
            let a = self.0.await;
            f(a).await
        }))
    }
}
```

**Evaluation**:

| Aspect                | Assessment                                                 |
| --------------------- | ---------------------------------------------------------- |
| Eliminates `'static`? | **No** — `Box<dyn Future>` still requires `'static`        |
| Stack safety          | ✅ Yes — async desugars to state machine                   |
| O(1) bind             | ⚠️ Depends — may still build nested futures                |
| Semantic match        | ❌ No — Futures imply async I/O, not pure lazy computation |
| Complexity            | Higher — requires async runtime understanding              |

**Verdict**: Does not eliminate `'static` and introduces semantic mismatch. Not recommended.

#### Alternative 2: Generators via fauxgen

**Idea**: Use the `fauxgen` crate to emulate generators:

```rust
use fauxgen::{generator, GeneratorToken};

pub struct Eval<A> {
    gen: Box<dyn FnOnce() -> A + Send>,
}

fn eval_generator<A>(token: GeneratorToken<(), A>) {
    // Use generator-style control flow
}
```

**Evaluation**:

| Aspect                | Assessment                                              |
| --------------------- | ------------------------------------------------------- |
| Eliminates `'static`? | **No** — still needs `Box<dyn ...>` for type erasure    |
| Stack safety          | ✅ Yes — generators are stackless                       |
| O(1) bind             | ⚠️ Depends on implementation                            |
| API clarity           | ❌ Poor — generator API doesn't map to monad operations |
| Stability             | ⚠️ External dependency                                  |

**Verdict**: Does not solve the core issue and introduces API complexity. Not recommended.

#### Alternative 3: Manual Defunctionalization

**Idea**: Replace closures with explicit enum variants:

```rust
enum EvalOp<A> {
    Pure(A),
    Map { f: fn(Box<dyn Any>) -> Box<dyn Any>, inner: Box<EvalOp<Box<dyn Any>>> },
    FlatMap { f: fn(Box<dyn Any>) -> EvalOp<Box<dyn Any>>, inner: Box<EvalOp<Box<dyn Any>>> },
}
```

**Evaluation**:

| Aspect                | Assessment                                             |
| --------------------- | ------------------------------------------------------ |
| Eliminates `'static`? | **Partially** — can parameterize by lifetime with care |
| Ergonomics            | ❌ Poor — loses closure convenience                    |
| Type safety           | ❌ Poor — everything becomes `fn(Box<dyn Any>) -> ...` |
| Complexity            | ❌ High — manual dispatch tables                       |

**Verdict**: Trades type safety and ergonomics for marginal lifetime flexibility. Not recommended.

#### Alternative 4: Arena-Based Allocation

**Idea**: Use a memory arena with lifetime-parameterized allocations:

```rust
pub struct Eval<'a, A> {
    inner: Free<'a, ThunkF, A>,
    arena: &'a Arena,
}
```

**Evaluation**:

| Aspect                | Assessment                                           |
| --------------------- | ---------------------------------------------------- |
| Eliminates `'static`? | **Yes** — can allocate `'a` data                     |
| Complexity            | ❌ High — arena management, lifetime threading       |
| Composability         | ❌ Poor — arenas don't compose cleanly               |
| Ergonomics            | ❌ Poor — explicit arena parameter everywhere        |
| Performance           | ⚠️ Mixed — fast allocation, but complex deallocation |

**Verdict**: Adds significant complexity for limited benefit. Not recommended for general use.

#### Alternative 5: Unsafe Type Erasure with ManuallyDrop

**Idea**: Use unsafe code to avoid `Any`:

```rust
struct ErasedVal {
    data: *mut (),
    drop_fn: fn(*mut ()),
    type_id: TypeId,
}
```

**Evaluation**:

| Aspect                | Assessment                                       |
| --------------------- | ------------------------------------------------ |
| Eliminates `'static`? | **Theoretically** yes, but extremely dangerous   |
| Safety                | ❌ Extremely Poor — lifetime violations, UB risk |
| Maintenance           | ❌ Poor — complex unsafe reasoning               |
| Auditability          | ❌ Poor — hard to verify correctness             |

**Verdict**: Unacceptably dangerous. Never recommended.

### 6.5 Comparison Summary

| Approach             | Eliminates `'static` | Stack Safe | O(1) Bind | Ergonomics           | Safety       | Recommendation     |
| -------------------- | -------------------- | ---------- | --------- | -------------------- | ------------ | ------------------ |
| **CatList proposal** | ❌ No                | ✅ Yes     | ✅ Yes    | ✅ Good              | ✅ Safe      | ✅ **Recommended** |
| async/Future         | ❌ No                | ✅ Yes     | ⚠️ Maybe  | ⚠️ Semantic mismatch | ✅ Safe      | ❌ Not recommended |
| fauxgen generators   | ❌ No                | ✅ Yes     | ⚠️ Maybe  | ⚠️ Poor API fit      | ✅ Safe      | ❌ Not recommended |
| Defunctionalization  | ⚠️ Partial           | ✅ Yes     | ✅ Yes    | ❌ Poor              | ⚠️ Reduced   | ❌ Not recommended |
| Arena allocation     | ✅ Yes               | ✅ Yes     | ✅ Yes    | ❌ Poor              | ⚠️ Complex   | ❌ Not recommended |
| Unsafe erasure       | ✅ Yes               | ✅ Yes     | ✅ Yes    | ⚠️ Ok                | ❌ Dangerous | ❌ Never           |

### 6.6 Recommendations

Based on this analysis, the hybrid CatList-based approach with `Box<dyn Any + Send>` is recommended because:

1. **The `'static` constraint is acceptable** — FP idioms naturally use owned data
2. **Alternatives don't eliminate `'static` anyway** — async and generators have the same requirement
3. **Alternatives that eliminate `'static` have worse trade-offs** — arena allocation and defunctionalization hurt ergonomics
4. **The constraint matches ecosystem expectations** — similar to Tokio, Rayon, and other deferred execution libraries
5. **Workarounds are straightforward** — clone, Arc, or restructure code

#### Design Decision

> **Decision**: Accept the `'static` constraint as an acceptable trade-off for type erasure via `Box<dyn Any + Send>`.
>
> **Rationale**: The constraint aligns with FP idioms (owned data), matches Rust ecosystem patterns (async, thread pools), and alternatives either don't solve the issue or introduce worse trade-offs.
>
> **Documentation**: API documentation should clearly state the `'static` requirement with examples of workarounds for users who encounter constraint issues.

### 6.7 API Documentation Guidance

Public API documentation should include:

````rust
/// Creates a lazy computation that produces `A`.
///
/// # Type Requirements
///
/// The result type `A` must be `'static + Send` due to internal type erasure.
/// This means `A` cannot contain borrowed references with non-`'static` lifetimes.
///
/// ## If You Need Non-`'static` Data
///
/// 1. **Clone the data**: Convert borrowed data to owned
///    ```rust
///    let owned: Vec<u8> = borrowed_slice.to_vec();
///    Eval::now(owned)
///    ```
///
/// 2. **Use Arc**: For shared ownership without cloning
///    ```rust
///    let shared = Arc::new(data);
///    Eval::now(shared)
///    ```
///
/// 3. **Restructure**: Do borrowed work synchronously, defer only owned results
///    ```rust
///    let result = compute_with_borrowed(&borrowed);
///    Eval::now(result)  // Defer only the owned result
///    ```
pub fn later<F>(f: F) -> Self
where
    F: FnOnce() -> A + Send + 'static,
{ /* ... */ }
````

## 7. Task API: Stack-Safe Evaluation (requires `'static`)

### 7.1 Design Philosophy

This section describes `Task<A>`, the **stack-safe** monadic type built on the Free monad with CatList-based bind stack. `Task` requires `A: 'static` due to type erasure via `Box<dyn Any>`.

**Key distinction**: `Task` is separate from `Eval<'a, A>` (the closure-based, HKT-compatible type described in Section 7.5). Use:

- **`Eval<'a, A>`** when you need HKT integration or non-`'static` lifetimes
- **`Task<A>`** when you need guaranteed stack safety for deep recursion

| Cats Eval           | Our Task               | Purpose                    |
| ------------------- | ---------------------- | -------------------------- |
| `Eval.now(a)`       | `Task::now(a)`         | Already computed value     |
| `Eval.later { a }`  | `Task::later(\|\| a)`  | Lazy, memoized             |
| `Eval.always { a }` | `Task::always(\|\| a)` | Lazy, NOT memoized         |
| `Eval.defer { ea }` | `Task::defer(\|\| ea)` | Deferred Task construction |
| `ea.flatMap(f)`     | `ta.flat_map(f)`       | Monadic bind               |
| `ea.map(f)`         | `ta.map(f)`            | Functor map                |
| `ea.value`          | `ta.run()`             | Force evaluation           |

**Key difference from Cats**: In this proposal, `Task` handles only _computation_. Memoization is handled by the separate `Memo` type from the [dual-type design](dual-type-design-proposal.md). This separation yields cleaner semantics.

### 7.2 Task Type Definition

````rust
/// A lazy, stack-safe computation that produces a value of type `A`.
///
/// `Task` is the "heavy-duty" monadic type for deferred computations that
/// require **guaranteed stack safety**. It is built on `Free<ThunkF, A>` with
/// CatList-based bind stack, ensuring O(1) bind operations and unlimited recursion
/// depth without stack overflow.
///
/// # Requirements
///
/// - `A: 'static + Send` — Required due to type erasure via `Box<dyn Any>`
///
/// # Guarantees
///
/// - **Stack safe**: Will not overflow regardless of recursion depth
/// - **O(1) bind**: Left-associated `flat_map` chains don't degrade
/// - **Lazy**: Computation is deferred until `run()` is called
///
/// # When to Use Task vs Eval
///
/// - Use **`Task<A>`** for deep recursion (1000+ levels), heavy monadic pipelines
/// - Use **`Eval<'a, A>`** for HKT integration, borrowed references, glue code
///
/// # Memoization
///
/// `Task` does NOT memoize. Each call to `run()` re-evaluates.
/// For memoization, wrap in `Memo`:
///
/// ```rust
/// let memo: Memo<i32> = Memo::new(|| Task::later(|| expensive()).run());
/// memo.get(); // Computes
/// memo.get(); // Returns cached
/// ```
///
/// # Example
///
/// ```rust
/// let task = Task::later(|| 1 + 1)
///     .flat_map(|x| Task::later(move || x * 2))
///     .flat_map(|x| Task::later(move || x + 10));
///
/// assert_eq!(task.run(), 14);
/// ```
pub struct Task<A> {
    inner: Free<ThunkFInstance, A>,
}

impl<A: 'static + Send> Task<A> {
    /// Creates a `Task` from an already-computed value.
    ///
    /// Equivalent to Cats' `Eval.now`.
    ///
    /// # Complexity
    /// O(1) creation, O(1) run
    ///
    /// # Example
    ///
    /// ```rust
    /// let task = Task::now(42);
    /// assert_eq!(task.run(), 42);
    /// ```
    #[inline]
    pub fn now(a: A) -> Self {
        Task {
            inner: Free::pure(a),
        }
    }

    /// Alias for `now` - PureScript style.
    #[inline]
    pub fn pure(a: A) -> Self {
        Self::now(a)
    }

    /// Creates a lazy `Task` that computes `f` on first `run()`.
    ///
    /// This is equivalent to Cats' `Eval.later`, but note that
    /// in our design, `Task` does NOT memoize — each `run()`
    /// re-evaluates. Use `Memo` for caching.
    ///
    /// # Complexity
    /// O(1) creation
    ///
    /// # Example
    ///
    /// ```rust
    /// let task = Task::later(|| {
    ///     println!("Computing!");
    ///     expensive_computation()
    /// });
    ///
    /// // Nothing printed yet
    /// let result = task.run(); // Prints "Computing!"
    /// ```
    #[inline]
    pub fn later<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static,
    {
        Task {
            inner: Free::roll(Thunk::new(move || Free::pure(f()))),
        }
    }

    /// Alias for `later` - semantically same since we don't memoize.
    ///
    /// In Cats, `always` differs from `later` in that it re-evaluates.
    /// Since our `Task` always re-evaluates, this is just an alias.
    #[inline]
    pub fn always<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static,
    {
        Self::later(f)
    }

    /// Defers the construction of a `Task` itself.
    ///
    /// This is critical for stack-safe recursion: instead of
    /// building a chain of `Task`s directly (which grows the stack),
    /// we defer the construction.
    ///
    /// # Example
    ///
    /// ```rust
    /// fn recursive_sum(n: u64, acc: u64) -> Task<u64> {
    ///     if n == 0 {
    ///         Task::now(acc)
    ///     } else {
    ///         // Defer construction to avoid stack growth
    ///         Task::defer(move || recursive_sum(n - 1, acc + n))
    ///     }
    /// }
    ///
    /// // This works for n = 1_000_000 without stack overflow!
    /// let result = recursive_sum(1_000_000, 0).run();
    /// ```
    #[inline]
    pub fn defer<F>(f: F) -> Self
    where
        F: FnOnce() -> Task<A> + Send + 'static,
    {
        Task {
            inner: Free::roll(Thunk::new(move || f().inner)),
        }
    }

    /// Monadic bind (flatMap) with O(1) complexity.
    ///
    /// Chains computations together. The key property is that
    /// left-associated chains don't degrade to O(n²):
    ///
    /// ```rust
    /// // This is O(n), not O(n²)
    /// let mut task = Task::now(0);
    /// for i in 0..10000 {
    ///     task = task.flat_map(move |x| Task::now(x + i));
    /// }
    /// ```
    #[inline]
    pub fn flat_map<B: 'static + Send, F>(self, f: F) -> Task<B>
    where
        F: FnOnce(A) -> Task<B> + Send + 'static,
    {
        Task {
            inner: self.inner.flat_map(move |a| f(a).inner),
        }
    }

    /// Functor map: transforms the result without changing structure.
    #[inline]
    pub fn map<B: 'static + Send, F>(self, f: F) -> Task<B>
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        self.flat_map(move |a| Task::now(f(a)))
    }

    /// Forces evaluation and returns the result.
    ///
    /// This runs the trampoline loop, iteratively processing
    /// the CatList of continuations without growing the stack.
    ///
    /// # Example
    ///
    /// ```rust
    /// let task = Task::later(|| 1 + 1);
    /// assert_eq!(task.run(), 2);
    /// ```
    pub fn run(self) -> A {
        self.inner.run()
    }

    /// Combines two `Task`s, running both and combining results.
    pub fn map2<B: 'static + Send, C: 'static + Send, F>(
        self,
        other: Task<B>,
        f: F,
    ) -> Task<C>
    where
        F: FnOnce(A, B) -> C + Send + 'static,
    {
        self.flat_map(move |a| other.map(move |b| f(a, b)))
    }

    /// Sequences two `Task`s, discarding the first result.
    pub fn and_then<B: 'static + Send>(self, other: Task<B>) -> Task<B> {
        self.flat_map(move |_| other)
    }

    /// Creates a `Task` from a memoized value (via Memo).
    ///
    /// This is a convenience for integrating with the dual-type design.
    /// The Memo provides caching; Task provides computation structure.
    pub fn from_memo(memo: &Memo<A>) -> Self
    where
        A: Clone,
    {
        let value = memo.get().clone();
        Task::now(value)
    }
}
````

### 7.3 MonadRec Implementation for Task

Note: `Task` does **not** implement the HKT-based `MonadRec` trait due to its `'static` requirement conflicting with HKT's `for<'a>` bounds. Instead, `Task` provides standalone `tail_rec_m` methods:

````rust
// Task provides its own tail_rec_m, not the trait-based MonadRec
impl<A: 'static + Send> Task<A> {
    /// Stack-safe tail recursion within Task.
    ///
    /// # Clone Bound
    ///
    /// The function `f` must implement `Clone` because each iteration
    /// of the recursion may need its own copy. Most closures naturally
    /// implement `Clone` when all their captures implement `Clone`.
    ///
    /// For closures that don't implement `Clone`, use `tail_rec_m_shared`
    /// which wraps the closure in `Arc` internally.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Fibonacci using tail recursion
    /// fn fib(n: u64) -> Task<u64> {
    ///     Task::tail_rec_m(|(n, a, b)| {
    ///         if n == 0 {
    ///             Task::now(Step::Done(a))
    ///         } else {
    ///             Task::now(Step::Loop((n - 1, b, a + b)))
    ///         }
    ///     }, (n, 0u64, 1u64))
    /// }
    ///
    /// assert_eq!(fib(50).run(), 12586269025);
    /// ```
    pub fn tail_rec_m<S: 'static + Send, F>(
        f: F,
        initial: S,
    ) -> Self
    where
        F: Fn(S) -> Task<Step<S, A>> + Clone + Send + 'static,
    {
        // Use defer to ensure each step is trampolined.
        fn go<A: 'static + Send, B: 'static + Send, F>(
            f: F,
            a: A,
        ) -> Task<B>
        where
            F: Fn(A) -> Task<Step<A, B>> + Clone + Send + 'static,
        {
            let f_clone = f.clone();
            Task::defer(move || {
                f(a).flat_map(move |step| match step {
                    Step::Loop(next) => go(f_clone.clone(), next),
                    Step::Done(b) => Task::now(b),
                })
            })
        }

        go(f, initial)
    }

    /// Arc-wrapped version for non-Clone closures.
    ///
    /// Use this when your closure captures non-Clone state.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Closure captures non-Clone state
    /// let counter = SomeNonCloneCounter::new();
    /// Task::tail_rec_m_shared(|n| {
    ///     counter.increment();
    ///     if n == 0 {
    ///         Task::now(Step::Done(counter.get()))
    ///     } else {
    ///         Task::now(Step::Loop(n - 1))
    ///     }
    /// }, 100)
    /// ```
    pub fn tail_rec_m_shared<S: 'static + Send, F>(
        f: F,
        initial: S,
    ) -> Self
    where
        F: Fn(S) -> Task<Step<S, A>> + Send + 'static,
    {
        use std::sync::Arc;
        let f = Arc::new(f);
        let wrapper = move |s: S| {
            let f = Arc::clone(&f);
            f(s)
        };
        Self::tail_rec_m(wrapper, initial)
    }
}
````

### 7.4 TryTask: Fallible Stack-Safe Computations

For computations that might fail, we provide `TryTask`:

```rust
/// A lazy, stack-safe computation that may fail with an error.
///
/// This is `Task<Result<A, E>>` with ergonomic combinators.
pub struct TryTask<A, E> {
    inner: Task<Result<A, E>>,
}

impl<A: 'static + Send, E: 'static + Send> TryTask<A, E> {
    /// Creates a successful `TryTask`.
    pub fn ok(a: A) -> Self {
        TryTask {
            inner: Task::now(Ok(a)),
        }
    }

    /// Creates a failed `TryTask`.
    pub fn err(e: E) -> Self {
        TryTask {
            inner: Task::now(Err(e)),
        }
    }

    /// Creates a lazy `TryTask` that may fail.
    pub fn try_later<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<A, E> + Send + 'static,
    {
        TryTask {
            inner: Task::later(f),
        }
    }

    /// Maps over the success value.
    pub fn map<B: 'static + Send, F>(self, f: F) -> TryTask<B, E>
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        TryTask {
            inner: self.inner.map(|result| result.map(f)),
        }
    }

    /// Maps over the error value.
    pub fn map_err<E2: 'static + Send, F>(self, f: F) -> TryTask<A, E2>
    where
        F: FnOnce(E) -> E2 + Send + 'static,
    {
        TryTask {
            inner: self.inner.map(|result| result.map_err(f)),
        }
    }

    /// Chains fallible computations.
    pub fn and_then<B: 'static + Send, F>(self, f: F) -> TryTask<B, E>
    where
        F: FnOnce(A) -> TryTask<B, E> + Send + 'static,
    {
        TryTask {
            inner: self.inner.flat_map(|result| match result {
                Ok(a) => f(a).inner,
                Err(e) => Task::now(Err(e)),
            }),
        }
    }

    /// Recovers from an error.
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce(E) -> TryTask<A, E> + Send + 'static,
    {
        TryTask {
            inner: self.inner.flat_map(|result| match result {
                Ok(a) => Task::now(Ok(a)),
                Err(e) => f(e).inner,
            }),
        }
    }

    /// Runs the computation, returning the result.
    pub fn run(self) -> Result<A, E> {
        self.inner.run()
    }
}
```

### 7.5 Eval<'a, A>: The Closure-Based HKT-Compatible Type

Before showing Task usage examples, we introduce `Eval<'a, A>` — the lightweight closure-based type that IS compatible with the HKT system:

````rust
/// A closure-based lazy computation that supports HKT integration.
///
/// Unlike `Task<A>`, `Eval` does NOT require `'static` and CAN implement
/// HKT traits like `Functor`, `Semimonad`, etc.
///
/// # Trade-offs vs Task
///
/// | Aspect         | Eval<'a, A>               | Task<A>                    |
/// |----------------|---------------------------|----------------------------|
/// | HKT compatible | ✅ Yes                    | ❌ No (requires `'static`) |
/// | Stack-safe     | ❌ No (~8000 calls limit) | ✅ Yes (unlimited)         |
/// | Lifetime       | `'a` (can borrow)         | `'static` only             |
/// | Use case       | Glue code, composition    | Deep recursion, pipelines  |
///
/// # Example
///
/// ```rust
/// let eval = Eval::new(|| 42)
///     .flat_map(|x| Eval::new(move || x * 2))
///     .map(|x| x + 1);
///
/// assert_eq!(eval.run(), 85);
/// ```
pub struct Eval<'a, A>(Box<dyn FnOnce() -> A + 'a>);

impl<'a, A> Eval<'a, A> {
    /// Creates a new Eval from a thunk.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> A + 'a,
    {
        Eval(Box::new(f))
    }

    /// Returns a pure value (already computed).
    pub fn pure(a: A) -> Self
    where
        A: 'a,
    {
        Eval::new(move || a)
    }

    /// Monadic bind: chains computations.
    ///
    /// Note: Each `flat_map` adds to the call stack. For deep recursion
    /// (>1000 levels), use `Task` instead.
    pub fn flat_map<B, F>(self, f: F) -> Eval<'a, B>
    where
        F: FnOnce(A) -> Eval<'a, B> + 'a,
    {
        Eval::new(move || {
            let a = (self.0)();
            let eval_b = f(a);
            (eval_b.0)()
        })
    }

    /// Functor map: transforms the result.
    pub fn map<B, F>(self, f: F) -> Eval<'a, B>
    where
        F: FnOnce(A) -> B + 'a,
    {
        Eval::new(move || f((self.0)()))
    }

    /// Forces evaluation and returns the result.
    pub fn run(self) -> A {
        (self.0)()
    }
}
````

The key insight is that intermediate types in `flat_map` chains exist only **inside the closure body**, avoiding the need for type erasure:

```rust
// This works because B and C are hidden inside closures, not exposed in the type
let pipeline: Eval<'_, i32> = Eval::new(|| "hello")      // Eval<'_, &str>
    .flat_map(|s| Eval::new(move || s.len()))             // Eval<'_, usize> (hidden)
    .flat_map(|n| Eval::new(move || (n * 2) as i32));     // Eval<'_, i32>
```

### 7.6 API Comparison with Cats

| Feature            | Cats Eval           | Our Task               | Our Eval               |
| ------------------ | ------------------- | ---------------------- | ---------------------- |
| Immediate value    | `Eval.now(a)`       | `Task::now(a)`         | `Eval::pure(a)`        |
| Lazy execution     | `Eval.later { a }`  | `Task::later(\|\| a)`  | `Eval::new(\|\| a)`    |
| Defer construction | `Eval.defer { ea }` | `Task::defer(\|\| ta)` | N/A                    |
| Map                | `ea.map(f)`         | `ta.map(f)`            | `ea.map(f)`            |
| FlatMap            | `ea.flatMap(f)`     | `ta.flat_map(f)`       | `ea.flat_map(f)`       |
| Force              | `ea.value`          | `ta.run()`             | `ea.run()`             |
| Memoize            | Built-in            | Use `Memo` wrapper     | Use `Memo` wrapper     |
| Tail recursion     | Via trampolining    | `Task::tail_rec_m`     | Via `Task` (if needed) |
| HKT compatible     | Yes (Scala)         | ❌ No                  | ✅ Yes                 |
| Stack safety       | Yes                 | ✅ Yes                 | ❌ No (~8000 calls)    |

**Key semantic differences**:

- Cats Eval: `later` memoizes, `always` doesn't
- Our Task/Eval: Never memoizes; use `Memo` for caching
- Task vs Eval: Choose based on stack safety needs vs HKT requirements

### 7.7 Usage Examples

#### Example 1: Deep Recursion with Task

```rust
/// Computes factorial using stack-safe recursion.
fn factorial(n: u64) -> Task<u64> {
    Task::tail_rec_m(|(n, acc)| {
        if n <= 1 {
            Task::now(Step::Done(acc))
        } else {
            Task::now(Step::Loop((n - 1, n * acc)))
        }
    }, (n, 1u64))
}

// Works for any n without stack overflow
assert_eq!(factorial(100_000).run(), /* very large number */);
```

#### Example 2: Lazy Tree Traversal with Task

```rust
enum Tree<A> {
    Leaf(A),
    Branch(Box<Tree<A>>, Box<Tree<A>>),
}

fn sum_tree(tree: Tree<i64>) -> Task<i64> {
    match tree {
        Tree::Leaf(x) => Task::now(x),
        Tree::Branch(left, right) => {
            // Defer to avoid stack growth on deep trees
            Task::defer(move || {
                sum_tree(*left).flat_map(move |l| {
                    sum_tree(*right).map(move |r| l + r)
                })
            })
        }
    }
}
```

#### Example 3: HKT Composition with Eval

```rust
/// Using Eval with generic Functor operations
fn double_in_context<F: Functor>(fa: Apply!(F, i32)) -> Apply!(F, i32)
where
    F::Brand: Kind_cdc7cd43dac7585f,
{
    F::map(|x| x * 2, fa)
}

// Works with Eval because Eval implements Functor
let eval = Eval::new(|| 21);
let doubled: Eval<'_, i32> = double_in_context::<EvalBrand>(eval);
assert_eq!(doubled.run(), 42);
```

#### Example 4: Stack-Safe Pipeline with TryTask

```rust
fn parse_config(path: &Path) -> TryTask<Config, ConfigError> {
    let path = path.to_owned();
    TryTask::try_later(move || {
        let content = std::fs::read_to_string(&path)?;
        parse_toml(&content)
    })
}

fn validate_config(config: Config) -> TryTask<ValidConfig, ConfigError> {
    TryTask::try_later(move || config.validate())
}

fn load_config(path: &Path) -> TryTask<ValidConfig, ConfigError> {
    parse_config(path)
        .and_then(validate_config)
        .map(|c| c.normalize())
}

// Nothing executes until .run()
let result = load_config(Path::new("app.toml")).run();
```

#### Example 5: With Memoization

```rust
use std::sync::Arc;

// Expensive computation via Task
let counter = Arc::new(AtomicUsize::new(0));
let counter_clone = Arc::clone(&counter);

// Without Memo: runs every time
let task1 = Task::later({
    let c = Arc::clone(&counter);
    move || { c.fetch_add(1, Ordering::SeqCst); heavy_computation() }
});
let task2 = Task::later({
    let c = Arc::clone(&counter);
    move || { c.fetch_add(1, Ordering::SeqCst); heavy_computation() }
});
let result1 = task1.run();
let result2 = task2.run();
assert_eq!(counter.load(Ordering::SeqCst), 2); // Ran twice

// With Memo: memoized
let memoized = Memo::new({
    let c = Arc::clone(&counter);
    move || { c.fetch_add(1, Ordering::SeqCst); heavy_computation() }
});
let result3 = memoized.get();
let result4 = memoized.get();
assert_eq!(counter.load(Ordering::SeqCst), 3); // Only ran once more
assert_eq!(result3, result4);
```

---

## 8. Integration with Dual-Type Design

This section describes how the **two-type architecture** (`Task` and `Eval`) integrates with the [dual-type design proposal](dual-type-design-proposal.md), which separates computation from memoization through computation types and `Memo` types.

### 8.1 Architecture Recap

The full architecture now includes **three layers**:

```
┌───────────────────────────────────────────────────────────────────────────┐
│                         Computation Layer                                 │
│                                                                           │
│  ┌─────────────────────────────┐    ┌───────────────────────────────────┐ │
│  │      Task<A>                │    │      Eval<'a, A>                  │ │
│  │ - Free monad + CatList      │    │ - Closure wrapper                 │ │
│  │ - Stack-safe, unlimited     │    │ - NOT stack-safe, ~8000 calls     │ │
│  │ - Requires A: 'static       │    │ - Works with any lifetime 'a      │ │
│  │ - NO HKT traits             │    │ - HKT-compatible: Functor, Monad  │ │
│  │ - Use for: deep recursion   │    │ - Use for: composition, glue code │ │
│  └─────────────────────────────┘    └───────────────────────────────────┘ │
│                    │                              │                       │
└────────────────────┼──────────────────────────────┼───────────────────────┘
                     │ .run()                       │ .run()
                     ▼                              ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                        Memoization Layer                                  │
│                      Memo<A> / MemoSync<A>                                │
│ - Lazy initialization via LazyCell/LazyLock                               │
│ - Thread-local or thread-safe                                             │
│ - Caches result of Task::run or Eval::run                                 │
└───────────────────────────────────────────────────────────────────────────┘
```

**Choosing between Task and Eval**:

| Scenario                        | Use Type      | Reason                          |
| ------------------------------- | ------------- | ------------------------------- |
| Deep recursion (>1000 calls)    | `Task<A>`     | Stack-safe via trampolining     |
| Long `flat_map` chains          | `Task<A>`     | O(1) bind via CatList           |
| Generic HKT code                | `Eval<'a, A>` | Implements Functor, Monad, etc. |
| Borrowed references in closures | `Eval<'a, A>` | No `'static` constraint         |
| Quick glue code                 | `Eval<'a, A>` | Simpler, lighter weight         |

### 8.2 Memo Types (From Dual-Type Proposal)

The dual-type proposal defines a single generic `Memo` type parameterized by a configuration trait. This works with **both** `Task<A>` and `Eval<'a, A>`:

```rust
use std::cell::LazyCell;
use std::sync::LazyLock;

/// A lazily-computed, memoized value with shared semantics.
///
/// The computation runs at most once; subsequent accesses return the cached value.
/// Cloning a `Memo` shares the underlying cache - all clones see the same value.
///
/// # Type Parameters
///
/// - `A`: The type of the computed value
/// - `Config`: The memoization configuration (determines Rc vs Arc)
pub struct Memo<A, Config: MemoConfig = RcMemoConfig> {
    inner: Config::Lazy<A>,
}

// Type aliases for convenience
pub type RcMemo<A> = Memo<A, RcMemoConfig>;
pub type ArcMemo<A> = Memo<A, ArcMemoConfig>;

impl<A, Config: MemoConfig> Memo<A, Config> {
    /// Creates a new Memo that will run `f` on first access.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> A + Send + 'static, // Send bound depends on Config, simplified here
    {
        Memo {
            inner: Config::new_lazy(f),
        }
    }

    /// Creates a Memo from a Task (stack-safe computation).
    pub fn from_task(task: Task<A>) -> Self
    where
        A: 'static + Send,
    {
        Memo::new(move || task.run())
    }

    /// Creates a Memo from an Eval (HKT-compatible computation).
    /// Note: Eval<'static, A> required because Memo stores the thunk.
    pub fn from_eval(eval: Eval<'static, A>) -> Self {
        Memo::new(move || eval.run())
    }

    /// Gets the memoized value, computing on first access.
    pub fn get(&self) -> &A {
        Config::force(&self.inner)
    }
}
```

### 8.3 Combining Task/Eval with Memo

The key insight is that computation types (`Task` and `Eval`) are complementary with `Memo`:

| Concern              | Task<A>              | Eval<'a, A>          | Memo                           |
| -------------------- | -------------------- | -------------------- | ------------------------------ |
| Deferred computation | ✅ Yes               | ✅ Yes               | ❌ No (runs on first `.get()`) |
| Composable chains    | ✅ `flat_map`, `map` | ✅ `flat_map`, `map` | ❌ Just caches                 |
| Stack safety         | ✅ Unlimited         | ❌ ~8000 calls       | N/A                            |
| HKT compatible       | ❌ No                | ✅ Yes               | N/A                            |
| Memoization          | ❌ No                | ❌ No                | ✅ Yes                         |
| Thread safety        | ✅ (requires Send)   | ⚠️ Depends on 'a     | ✅ (ArcMemo)                   |

**Pattern 1: Build with Task for deep recursion, cache with Memo**

```rust
// Stack-safe deep computation with Task
fn traverse_deep_tree(tree: Tree<i64>) -> Task<i64> {
    Task::tail_rec_m(|state| {
        // ... stack-safe recursive traversal
    }, initial_state)
}

// Cache the result for repeated access
let cached: Memo<i64> = Memo::from_task(traverse_deep_tree(big_tree));

// First call computes; subsequent calls return cached
let result1 = cached.get();
let result2 = cached.get(); // Same reference, no recomputation
```

**Pattern 2: Build with Eval for HKT code, cache with Memo**

```rust
// Build computation using generic HKT functions
fn apply_transforms<F: Functor>(fa: Apply!(F, Data)) -> Apply!(F, Data) {
    F::map(|d| d.transform(), fa)
}

let eval = Eval::new(|| load_data());
let transformed = apply_transforms::<EvalBrand>(eval);

// Cache the Eval result (note: Eval must be 'static for Memo)
let cached: Memo<Data> = Memo::new(move || transformed.run());
```

### 8.4 Lazy Recursive Structures

One powerful pattern is lazy recursive data structures. For potentially deep structures, use `Task` to ensure stack safety:

```rust
/// A lazy stream that computes elements on demand.
/// Uses Task for stack-safe recursive construction.
pub struct Stream<A> {
    head: A,
    tail: ArcMemo<Option<Stream<A>>>,
}

impl<A: Clone + Send + Sync + 'static> Stream<A> {
    /// Creates a finite stream from an iterator.
    /// Uses Task::defer for stack-safe lazy construction.
    pub fn from_iter<I: IntoIterator<Item = A> + Send + 'static>(iter: I) -> Option<Self> {
        let mut iter = iter.into_iter();
        iter.next().map(|head| {
            // Use Task for stack-safe deferred construction
            let tail = ArcMemo::from_task(
                Task::defer(move || {
                    Task::now(Self::from_iter(iter))
                })
            );
            Stream { head, tail }
        })
    }

    /// Maps a function over the stream lazily.
    pub fn map<B, F>(self, f: F) -> Stream<B>
    where
        B: Clone + Send + Sync + 'static,
        F: Fn(A) -> B + Clone + Send + Sync + 'static,
    {
        let f_clone = f.clone();
        Stream {
            head: f(self.head),
            tail: ArcMemo::from_task(
                Task::defer(move || {
                    Task::now(self.tail.get().clone().map(|t| t.map(f_clone)))
                })
            ),
        }
    }

    /// Takes the first n elements.
    /// Iterative, so no stack concerns.
    pub fn take(self, n: usize) -> Vec<A> {
        let mut result = Vec::with_capacity(n);
        let mut current = Some(self);

        for _ in 0..n {
            match current {
                Some(stream) => {
                    result.push(stream.head);
                    current = stream.tail.get().clone();
                }
                None => break,
            }
        }

        result
    }
}

// For shallow recursive structures, Eval is simpler:
fn shallow_lazy_value<'a>() -> Eval<'a, String> {
    Eval::new(|| format!("computed at: {:?}", std::time::Instant::now()))
}
```

### 8.5 The MemoConfig Trait

The dual-type proposal introduces `MemoConfig` to abstract over `Rc`/`Arc`:

```rust
/// Configuration trait for memoization wrapper types.
pub trait MemoConfig {
    /// The reference-counted pointer type (Rc or Arc).
    type Ptr<T>: Pointer<T>;

    /// The lazy cell type (LazyCell or LazyLock).
    type Lazy<T, F: FnOnce() -> T>: LazyInit<T, F>;
}

/// Single-threaded configuration.
pub struct LocalConfig;

impl MemoConfig for LocalConfig {
    type Ptr<T> = Rc<T>;
    type Lazy<T, F: FnOnce() -> T> = LazyCell<T, F>;
}

/// Thread-safe configuration.
pub struct SyncConfig;

impl MemoConfig for SyncConfig {
    type Ptr<T> = Arc<T>;
    type Lazy<T, F: FnOnce() -> T> = LazyLock<T, F>;
}
```

This enables generic code over thread-safety:

```rust
/// A memoized computation parameterized by configuration.
pub struct GenericMemo<A, C: MemoConfig> {
    cell: C::Lazy<A, Box<dyn FnOnce() -> A>>,
}

impl<A, C: MemoConfig> GenericMemo<A, C> {
    pub fn from_eval(eval: Eval<A>) -> Self
    where
        A: 'static + Send,
    {
        GenericMemo {
            cell: C::Lazy::new(Box::new(move || eval.run())),
        }
    }
}

// Use as:
type LocalMemo<A> = GenericMemo<A, LocalConfig>;
type SharedMemo<A> = GenericMemo<A, SyncConfig>;
```

### 8.6 Migration Path from Existing Lazy

The current [`types/lazy.rs`](../../fp-library/src/types/lazy.rs) provides a `Lazy<A>` type. With this proposal:

| Current             | Proposed Replacement                       |
| ------------------- | ------------------------------------------ |
| `Lazy::new(\|\| a)` | `Eval::new(\|\| a)` for HKT composition    |
| `Lazy::new(\|\| a)` | `Task::later(\|\| a)` for stack safety     |
| `Lazy::new(\|\| a)` | `Memo::new(\|\| a)` for memoization        |
| `lazy.force()`      | `eval.run()`, `task.run()` or `memo.get()` |
| `lazy.map(f)`       | `eval.map(f)` or `task.map(f)`             |

**Migration strategy**:

1. **Phase 1**: Introduce `Task`, `Eval`, and `Memo` as new types alongside existing `Lazy`
2. **Phase 2**: Deprecate `Lazy` with migration guidance (with clear guidance on when to use Task vs Eval)
3. **Phase 3**: Remove `Lazy` in next major version

### 8.7 Benefits of the Three-Layer Architecture

The separation into Task (stack-safe), Eval (HKT), and Memo (caching) provides:

1. **Clarity**: Stack safety vs HKT compatibility vs caching is explicit in the type
2. **Composability**: Both `Task` and `Eval` chain with `flat_map`; `Memo` just caches
3. **Predictability**: Each `run()` produces a fresh computation; caching is explicit via `Memo`
4. **Flexibility**: Choose computation type based on needs, then optionally memoize
5. **Stack Safety**: `Task` guarantees unlimited recursion; `Eval` is lightweight for shallow chains
6. **HKT Integration**: `Eval` implements Functor, Semimonad, etc. for generic programming
7. **Performance**: `Task` has O(1) bind via CatList; `Eval` has minimal overhead for simple cases

---

## 9. Integration with HKT System

This section describes how the **closure-based `Eval<'a, A>`** integrates with the existing HKT (higher-kinded types) system in `fp-library`.

**Critical distinction**: Only `Eval<'a, A>` (the closure-based type) can implement HKT traits. `Task<A>` (the Free monad type) **cannot** implement HKT traits due to its `'static` requirement conflicting with the `for<'a>` bounds in HKT method signatures.

### 9.1 Why Task Cannot Implement HKT Traits

The fundamental conflict:

```rust
// HKT trait methods require working for ANY lifetime 'a
trait Semimonad {
    fn bind<'a, A: 'a, B: 'a, F>(
        ma: Apply!(Self::Of<'a, A>),
        f: F,
    ) -> Apply!(Self::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(Self::Of<'a, B>) + 'a;
}

// But Task requires 'static due to Box<dyn Any>
impl Task<A> where A: 'static + Send {
    // ...
}
```

These constraints are **mutually exclusive**:

- HKT: "must work for any `'a`"
- Task: "requires `'static`" (which is a specific lifetime, not any `'a`)

**Solution**: The closure-based `Eval<'a, A>` has no `'static` constraint and CAN implement HKT traits.

### 9.2 Overview of Project HKT System

The project uses a macro-based HKT encoding via [`def_kind!`](../../fp-library/src/kinds.rs) and [`impl_kind!`](../../fp-library/src/kinds.rs) macros:

```rust
// In kinds.rs - defines the core Kind trait
def_kind!(type Of<'a, A: 'a>: 'a);
// This generates: trait Kind_cdc7cd43dac7585f {
//     type Of<'a, A: 'a>: 'a;
// }
```

Type constructors are represented via "brand" types that implement this trait:

```rust
// Brand type for Option
pub struct OptionBrand;

// Maps OptionBrand to Option<A>
impl_kind! {
    for OptionBrand {
        type Of<'a, A: 'a>: 'a = Option<A>;
    }
}
```

Type class methods use the `Apply!` and `Kind!` macros for type-level application:

```rust
impl Functor for OptionBrand {
    fn map<'a, B: 'a, A: 'a, F>(
        f: F,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> B + 'a,
    {
        fa.map(f)
    }
}
```

### 9.3 EvalBrand and HKT Integration

We define `EvalBrand` for the **closure-based** `Eval<'a, A>`:

```rust
/// Brand type for the closure-based Eval in the HKT system.
///
/// Note: This is for Eval<'a, A>, NOT for Task<A>.
/// Task cannot implement HKT traits due to its 'static requirement.
pub struct EvalBrand;

impl_kind! {
    for EvalBrand {
        // The lifetime 'a flows through to Eval<'a, A>
        type Of<'a, A: 'a>: 'a = Eval<'a, A>;
    }
}
```

### 9.4 Functor Implementation for Eval

```rust
impl Functor for EvalBrand {
    /// Maps a function over the result of an Eval computation.
    ///
    /// ### Type Signature
    ///
    /// `forall b a. Functor Eval => (a -> b, Eval a) -> Eval b`
    fn map<'a, B: 'a, A: 'a, F>(
        f: F,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> B + 'a,
    {
        fa.map(f)  // No Send bound needed - Eval<'a, A> is flexible
    }
}
```

### 9.5 Pointed Implementation for Eval

```rust
impl Pointed for EvalBrand {
    /// Wraps a value in an Eval context.
    ///
    /// ### Type Signature
    ///
    /// `forall a. Pointed Eval => a -> Eval a`
    fn pure<'a, A: 'a>(
        a: A
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
    {
        Eval::pure(a)  // No Send bound needed
    }
}
```

### 9.6 Semimonad Implementation for Eval

```rust
impl Semimonad for EvalBrand {
    /// Chains Eval computations.
    ///
    /// ### Type Signature
    ///
    /// `forall b a. Semimonad Eval => (Eval a, a -> Eval b) -> Eval b`
    fn bind<'a, B: 'a, A: 'a, F>(
        ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
        f: F,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
    {
        // Closure-based flat_map works for any 'a, not just 'static
        ma.flat_map(move |a| f(a))
    }
}
```

### 9.7 MonadRec Trait Definition

The `MonadRec` trait follows the project's HKT patterns. **Note**: `Eval` can implement this trait for HKT polymorphism, but for truly stack-safe deep recursion, use `Task::tail_rec_m` directly:

```rust
use crate::{Apply, kinds::*, classes::Monad};

/// A type class for monads that support stack-safe tail recursion.
///
/// ### Important Design Note
///
/// `Eval<'a, A>` CAN implement this trait (HKT-compatible).
/// `Task<A>` CANNOT implement this trait (requires `'static`).
///
/// For deep recursion (10,000+ calls), prefer `Task::tail_rec_m` which is
/// guaranteed stack-safe. `Eval`'s trait-based `tail_rec_m` will overflow
/// the stack at ~8000 recursive calls.
///
/// ### Laws
///
/// 1. **Equivalence**: `tail_rec_m(f, a)` produces the same result as the
///    recursive definition.
///
/// 2. **Safety varies**: Eval is NOT stack-safe for deep recursion.
///    Use Task for guaranteed stack safety.
pub trait MonadRec: Monad {
    /// Performs tail-recursive monadic computation.
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        a: A,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + 'a;
}

/// Free function version of tail_rec_m.
pub fn tail_rec_m<'a, Brand, A: 'a, B: 'a, F>(
    f: F,
    a: A,
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
    Brand: MonadRec,
    F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + 'a,
{
    Brand::tail_rec_m(f, a)
}
```

### 9.7 MonadRec Implementation for OptionBrand

```rust
impl MonadRec for OptionBrand {
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        mut a: A,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + 'a,
    {
        loop {
            match f(a)? {
                Step::Loop(next) => a = next,
                Step::Done(b) => return Some(b),
            }
        }
    }
}
```

### 9.8 MonadRec Implementation for EvalBrand

```rust
impl MonadRec for EvalBrand {
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(
        f: F,
        initial: A,
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>) + Clone + 'a,
        A: Send + ToConstraint<Self::Constraint>,
        B: Send + ToConstraint<Self::Constraint>,
    {
        // Use defer for trampolining.
        // The Clone bound allows us to clone `f` for each recursive step.
        fn go<A: Send + 'static, B: Send + 'static, F>(
            f: F,
            a: A,
        ) -> Eval<B>
        where
            F: Fn(A) -> Eval<Step<A, B>> + Clone + Send + 'static,
        {
            let f_clone = f.clone();  // Clone for the recursive call
            Eval::defer(move || {
                f(a).flat_map(move |step| match step {
                    Step::Loop(next) => go(f_clone.clone(), next),
                    Step::Done(b) => Eval::now(b),
                })
            })
        }

        go(f, initial)
    }
}
```

### 9.9 Foldable Implementation for Eval

```rust
impl Foldable for EvalBrand {
    fn fold_right<'a, FnBrand, B: 'a, A: 'a, Func>(
        func: Func,
        initial: B,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> B
    where
        Func: Fn(A, B) -> B + 'a,
        FnBrand: CloneableFn + 'a,
        A: Send,
    {
        func(fa.run(), initial)
    }

    fn fold_left<'a, FnBrand, B: 'a, A: 'a, Func>(
        func: Func,
        initial: B,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> B
    where
        Func: Fn(B, A) -> B + 'a,
        FnBrand: CloneableFn + 'a,
        A: Send,
    {
        func(initial, fa.run())
    }

    fn fold_map<'a, FnBrand, M, A: 'a, Func>(
        func: Func,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> M
    where
        M: Monoid + 'a,
        Func: Fn(A) -> M + 'a,
        FnBrand: CloneableFn + 'a,
        A: Send,
    {
        func(fa.run())
    }
}
```

### 9.10 ThunkF Brand and Functor

```rust
/// Brand type for ThunkF - the functor underlying trampolining.
pub struct ThunkFBrand;

impl_kind! {
    for ThunkFBrand {
        type Of<'a, A: 'a>: 'a = Thunk<A> where A: Send;
    }
}

impl Functor for ThunkFBrand {
    fn map<'a, B: 'a, A: 'a, F>(
        f: F,
        fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
    where
        F: Fn(A) -> B + 'a,
        A: Send,
        B: Send,
    {
        Thunk::new(move || f(fa.force()))
    }
}
```

### 9.11 FreeBrand (Higher-Kinded Free)

The `Free` monad is itself parameterized by a functor. To represent this in the HKT system, we use a "curried" brand:

```rust
/// Brand for Free monad parameterized by a functor brand.
pub struct FreeBrand<FBrand>(PhantomData<FBrand>);

impl<FBrand: Functor> Kind_cdc7cd43dac7585f for FreeBrand<FBrand> {
    type Of<'a, A: 'a> = Free<FBrand, A>
    where
        A: Send;
}
```

This allows writing generic code over any Free monad:

```rust
fn lift_free<'a, FBrand: Functor, A: 'a + Send>(
    fa: Apply!(<FBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
) -> Apply!(<FreeBrand<FBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
    Free::roll(FBrand::map(Free::pure, fa))
}
```

### 9.12 Type Class Hierarchy

The stack-safe types fit into the existing hierarchy:

```
                    Kind_cdc7cd43dac7585f
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
    Functor             Pointed             Foldable
        │                   │                   │
        └─────────┬─────────┘                   │
                  ▼                             │
            Semiapplicative                     │
                  │                             │
                  ▼                             │
              Applicative ◄─────────────────────┘
                  │                     Traversable
                  ▼
              Semimonad
                  │
                  ▼
                Monad
                  │
                  ▼
              MonadRec  ◄─── NEW: Stack-safe recursion
```

### 9.13 Example: Generic Stack-Safe Algorithm

With the HKT integration, we can write generic stack-safe algorithms:

```rust
/// Folds a list using a monadic function, stack-safely.
fn fold_m<'a, M, A, B, F>(
    xs: Vec<A>,
    init: B,
    f: F,
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
    M: MonadRec,
    A: 'a + Clone,
    B: 'a + Clone,
    F: Fn(B, A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a + Clone,
{
    M::tail_rec_m(
        move |(mut xs, acc): (Vec<A>, B)| {
            if xs.is_empty() {
                M::pure(Step::Done(acc))
            } else {
                let head = xs.remove(0);
                M::bind(f.clone()(acc, head), move |new_acc| {
                    M::pure(Step::Loop((xs, new_acc)))
                })
            }
        },
        (xs, init),
    )
}

// Usage with Eval
let result: Eval<i64> = fold_m::<EvalBrand, _, _, _>(
    vec![1, 2, 3, 4, 5],
    0i64,
    |acc, x| Eval::now(acc + x),
);
assert_eq!(result.run(), 15);

// Usage with Option
let result: Option<i64> = fold_m::<OptionBrand, _, _, _>(
    vec![1, 2, 3, 4, 5],
    0i64,
    |acc, x| Some(acc + x),
);
assert_eq!(result, Some(15));
```

---

## 10. Performance Characteristics

### 10.1 Complexity Summary

| Operation         | CatQueue | CatList | Free | Task | Eval |
| ----------------- | -------- | ------- | ---- | ---- | ---- |
| `empty`/`pure`    | O(1)     | O(1)    | O(1) | O(1) | O(1) |
| `snoc`/`cons`     | O(1)     | O(1)    | N/A  | N/A  | N/A  |
| `uncons`          | O(1)\*   | O(1)\*  | N/A  | N/A  | N/A  |
| `append`          | N/A      | O(1)    | N/A  | N/A  | N/A  |
| `flat_map`/`bind` | N/A      | N/A     | O(1) | O(1) | O(1) |
| `map`             | N/A      | N/A     | O(1) | O(1) | O(1) |
| `run`             | N/A      | N/A     | O(n) | O(n) | O(n) |
| Stack growth      | N/A      | N/A     | O(1) | O(1) | O(n) |

\*Amortized, O(n) worst case

**Key difference**: `Task` maintains O(1) stack depth via trampolining. `Eval` grows the stack with each nested `flat_map` call.

### 10.2 Left-Associated Bind Analysis

**Scenario**: Build a chain of n binds, then run.

For **Task** (stack-safe, uses CatList):

```rust
let mut task = Task::pure(0);
for i in 0..n {
    task = task.flat_map(move |x| Task::pure(x + 1));
}
task.run()
```

For **Eval** (closure-based, simple nesting):

```rust
let mut eval = Eval::pure(0);
for i in 0..n {
    eval = eval.flat_map(move |x| Eval::pure(x + 1));
}
eval.run()  // ⚠️ Stack overflow for large n!
```

**Comparison**:

| Approach        | Build Time | Run Time | Stack Growth | Max Safe n |
| --------------- | ---------- | -------- | ------------ | ---------- |
| Task (CatList)  | O(n)       | O(n)     | O(1)         | Unlimited  |
| Eval (closures) | O(n)       | O(n)     | O(n)         | ~8000      |
| Vec-based (old) | O(n)       | O(n²)    | O(1)         | Unlimited  |

**Task's CatList advantage**:

- Each `flat_map`: O(1) — `snoc` onto CatList
- `run()`: O(n) — linear in number of continuations
- Stack: O(1) — trampolined execution
- Total: O(n) time, O(1) stack

**Eval's simplicity trade-off**:

- Each `flat_map`: O(1) — creates nested closure
- `run()`: O(n) — linear execution
- Stack: O(n) — each closure call adds a frame
- Suitable for n < 1000, use Task for larger

### 10.3 Memory Overhead

**Per continuation**:

```rust
type Cont<F> = Box<dyn FnOnce(Val) -> Free<F, Val> + Send>;
// Size: 2 words (fat pointer) + closure capture
```

**Per CatList node**:

```rust
enum CatList<A> {
    Nil,                              // 0 bytes payload
    Cons(A, CatQueue<CatList<A>>),    // A + 2 Vecs
}
```

**Estimated overhead per element**:

- Vec entry: 0 (amortized, stored inline)
- Box for continuation: 16 bytes (2 words)
- Box for type erasure: 16 bytes (2 words)
- Total: ~32-48 bytes per continuation

**Comparison**:

- Direct closure chain: 16-24 bytes per closure
- Vec-based: 16-24 bytes per entry + Vec overhead
- CatList-based: 32-48 bytes per entry but O(1) ops

The ~2x memory overhead is acceptable for the asymptotic improvement.

### 10.4 Benchmarking Recommendations

Implement benchmarks comparing Task and Eval:

```rust
#[bench]
fn bench_left_bind_task(b: &mut Bencher) {
    b.iter(|| {
        let mut task = Task::pure(0);
        for _ in 0..10000 {
            task = task.flat_map(|x| Task::pure(x + 1));
        }
        task.run()
    });
}

#[bench]
fn bench_left_bind_eval_shallow(b: &mut Bencher) {
    // Eval is fine for shallow chains
    b.iter(|| {
        let mut eval = Eval::pure(0);
        for _ in 0..100 {  // Keep under stack limit
            eval = eval.flat_map(|x| Eval::pure(x + 1));
        }
        eval.run()
    });
}

#[bench]
fn bench_deep_recursion_task(b: &mut Bencher) {
    b.iter(|| {
        Task::tail_rec_m(|n: i32| {
            if n == 0 {
                Task::now(Step::Done(0))
            } else {
                Task::now(Step::Loop(n - 1))
            }
        }, 100000).run()
    });
}

#[bench]
fn bench_hkt_generic_eval(b: &mut Bencher) {
    // Eval's strength: HKT polymorphism
    fn double<F: Functor>(fa: Apply!(F, i32)) -> Apply!(F, i32) {
        F::map(|x| x * 2, fa)
    }

    b.iter(|| {
        let eval = Eval::pure(21);
        double::<EvalBrand>(eval).run()
    });
}
```

**Expected results**:

- Deep recursion: Task handles any depth, Eval overflows at ~8000
- Left-bind: Task maintains O(n), Eval is O(n) but stack-limited
- HKT code: Eval is simpler, Task cannot be used for HKT
- Small chains (<100): Eval has lower overhead, prefer Eval

### 10.5 When to Use Task vs Eval vs Direct Code

| Use Case                          | Recommendation             |
| --------------------------------- | -------------------------- |
| Shallow chains (<100 levels)      | `Eval` or direct closures  |
| Deep recursion (>1000 levels)     | `Task::tail_rec_m`         |
| Long bind chains (>1000)          | `Task` (stack-safe)        |
| HKT polymorphic code              | `Eval` (implements traits) |
| Borrowed references in closures   | `Eval<'a, A>` (no 'static) |
| Performance-critical inner loops  | Direct code                |
| Compositional pipelines (deep)    | `Task` for structure       |
| Compositional pipelines (shallow) | `Eval` for HKT             |
| Memoization needed                | `Memo` wrapping either     |

### 10.6 Stack Depth Guarantees

**Task — Guaranteed stack-safe**:

- `Task::flat_map` — O(1) stack depth
- `Task::defer` — Defers to trampoline
- `Task::tail_rec_m` — Iterative loop
- `Task::run` — Bounded stack regardless of depth

**Eval — NOT stack-safe**:

- `Eval::flat_map` — Adds stack frame per call
- `Eval::run` — Stack depth = nesting depth
- Safe for ~8000 nested calls (OS-dependent)
- Use for HKT code and shallow chains only

**Internal operations**:

- `CatList::flatten_queue` — Uses iterative fold, safe
- `Free::erase_type` — Bounded by structure depth, not chain length

### 10.7 Comparison with Other Approaches

| Approach            | Bind Complexity | Stack Safety | Type Safety | HKT Compat | Ergonomics |
| ------------------- | --------------- | ------------ | ----------- | ---------- | ---------- |
| async/await         | O(1)            | ✅           | ✅          | ❌         | ✅         |
| Generator (nightly) | O(1)            | ✅           | ✅          | ❌         | ⚠️         |
| Continuation monad  | O(1)            | ✅           | ✅          | ⚠️         | ⚠️         |
| Vec-based (old)     | O(n²) worst     | ✅           | ⚠️          | ❌         | ✅         |
| **Task (CatList)**  | **O(1)**        | ✅           | ⚠️          | ❌         | ✅         |
| **Eval (closure)**  | O(1)            | ❌ (~8000)   | ✅          | ✅         | ✅         |

**The two-type architecture** offers the best balance for a pure FP library:

- **Task**: Stack-safe with O(1) bind for deep recursion and long pipelines
- **Eval**: HKT-compatible with simple implementation for composition and glue code

This split acknowledges that no single Rust type can satisfy all constraints (stack safety + HKT + non-`'static` lifetimes) simultaneously.

---

## 11. Implementation Checklist

### 11.1 Core Data Structures

- [ ] **CatQueue** — `fp-library/src/types/cat_queue.rs`

  - [ ] `CatQueue<A>` struct with `front: Vec<A>`, `back: Vec<A>`
  - [ ] `empty`, `singleton`, `is_empty`, `len` methods
  - [ ] `cons`, `snoc` methods
  - [ ] `uncons`, `unsnoc` methods
  - [ ] `IntoIterator` implementation
  - [ ] Unit tests for all operations
  - [ ] Property tests for queue invariants

- [ ] **CatList** — `fp-library/src/types/cat_list.rs`
  - [ ] `CatList<A>` enum with `Nil` and `Cons` variants
  - [ ] `empty`, `singleton`, `is_empty` methods
  - [ ] `cons`, `snoc`, `append` methods
  - [ ] `uncons` method with iterative `flatten_queue`
  - [ ] `IntoIterator` and `FromIterator` implementations
  - [ ] Unit tests for all operations
  - [ ] Property tests for list invariants

### 11.2 Core Types

- [ ] **Step** — `fp-library/src/types/step.rs`

  - [ ] `Step<A, B>` enum with `Loop` and `Done` variants
  - [ ] `is_loop`, `is_done` methods
  - [ ] `map_loop`, `map_done`, `bimap` methods
  - [ ] `Step` brand and HKT integration
  - [ ] Bifunctor implementation

- [ ] **Thunk** — `fp-library/src/types/thunk.rs`

  - [ ] `Thunk<A>` struct wrapping `Box<dyn FnOnce() -> A + Send>`
  - [ ] `new`, `force` methods
  - [ ] `ThunkFBrand` and HKT integration
  - [ ] Functor implementation

- [ ] **Free** — `fp-library/src/types/free.rs`
  - [ ] `Free<F, A>` enum with `Pure`, `Roll`, `Bind` variants
  - [ ] Type erasure via `Val = Box<dyn Any + Send>`
  - [ ] `pure`, `roll`, `flat_map`, `map` methods
  - [ ] `run` method with iterative trampoline
  - [ ] `FreeBrand` parameterized by functor brand
  - [ ] Functor, Pointed, Semimonad implementations

### 11.3 User-Facing API — Two-Type Architecture

#### 11.3.1 Task — Stack-Safe Computation (requires `'static`)

- [ ] **Task** — `fp-library/src/types/task.rs`

  - [ ] `Task<A>` struct wrapping `Free<ThunkFBrand, A>`
  - [ ] `now`, `pure`, `later`, `always`, `defer` constructors
  - [ ] `flat_map`, `map`, `map2`, `and_then` combinators
  - [ ] `run` method
  - [ ] `tail_rec_m` standalone method (NOT trait-based)
  - [ ] `tail_rec_m_shared` for non-Clone closures
  - [ ] Unit tests for stack safety (1M+ iterations)
  - [ ] Property tests for monad laws (where testable)

- [ ] **TryTask** — `fp-library/src/types/try_task.rs`
  - [ ] `TryTask<A, E>` for fallible computations
  - [ ] `ok`, `err`, `try_later` constructors
  - [ ] `map`, `map_err`, `and_then`, `or_else` combinators
  - [ ] `run` method returning `Result<A, E>`

#### 11.3.2 Eval — HKT-Compatible Computation (no `'static` required)

- [ ] **Eval** — `fp-library/src/types/eval.rs`

  - [ ] `Eval<'a, A>` struct wrapping `Box<dyn FnOnce() -> A + 'a>`
  - [ ] `new`, `pure` constructors
  - [ ] `flat_map`, `map` combinators
  - [ ] `run` method
  - [ ] `EvalBrand` and HKT integration
  - [ ] Functor, Pointed, Semimonad implementations
  - [ ] MonadRec implementation (note: NOT stack-safe for deep calls)
  - [ ] Documentation warning about ~8000 call stack limit

- [ ] **TryEval** — `fp-library/src/types/try_eval.rs`
  - [ ] `TryEval<'a, A, E>` for fallible computations
  - [ ] `new`, `pure` constructors
  - [ ] `flat_map`, `map`, `map_err` combinators
  - [ ] `run` method returning `Result<A, E>`

### 11.4 Type Class Extensions

- [ ] **MonadRec trait** — `fp-library/src/classes/monad_rec.rs`

  - [ ] `MonadRec` trait definition with `tail_rec_m`
  - [ ] Clone bound on `F` parameter (see Section 4.3)
  - [ ] Free function `tail_rec_m` with Clone bound
  - [ ] Free function `tail_rec_m_shared` with Arc wrapper for non-Clone closures
  - [ ] Documentation with laws and examples
  - [ ] Note: Task CANNOT implement this trait (requires `'static`)

- [ ] **MonadRec implementations** (for HKT-compatible types only)
  - [ ] `OptionBrand` implementation
  - [ ] `ResultBrand<E>` implementation
  - [ ] `IdentityBrand` implementation
  - [ ] `EvalBrand` implementation (note: not stack-safe for deep calls)
  - [ ] `VecBrand` implementation (if applicable)
  - [ ] Documentation warning that Eval's MonadRec is NOT stack-safe

### 11.5 Memoization Integration

- [ ] **Memo updates** — `fp-library/src/types/memo.rs`

  - [ ] `from_task` constructor for Task<A>
  - [ ] `from_eval` constructor for Eval<'static, A>
  - [ ] `map_task` method for transforming Task before memoizing
  - [ ] Ensure compatibility with both Task and Eval APIs

- [ ] **MemoSync updates** — `fp-library/src/types/memo_sync.rs`
  - [ ] Remove `MemoSync` struct (replaced by `ArcMemo` alias)
  - [ ] Ensure `ArcMemo` (via `Memo<A, ArcMemoConfig>`) supports `from_task` and `from_eval`

### 11.6 Module Organization

- [ ] **Update `types.rs`**

  - [ ] Add `mod cat_queue;`
  - [ ] Add `mod cat_list;`
  - [ ] Add `mod step;`
  - [ ] Add `mod thunk;`
  - [ ] Add `mod free;`
  - [ ] Add `mod task;` (stack-safe, Free-based)
  - [ ] Add `mod eval;` (HKT-compatible, closure-based)
  - [ ] Add `mod try_task;`
  - [ ] Add re-exports

- [ ] **Update `classes.rs`**

  - [ ] Add `mod monad_rec;`
  - [ ] Add re-exports

- [ ] **Update `brands.rs`**
  - [ ] Add `EvalBrand` (for closure-based Eval)
  - [ ] Add `ThunkFBrand`
  - [ ] Add `FreeBrand`
  - [ ] Note: NO TaskBrand (Task cannot implement HKT traits)

### 11.7 Testing

- [ ] **Unit tests** for each module

  - [ ] CatQueue: push/pop sequences, edge cases
  - [ ] CatList: append chains, uncons sequences
  - [ ] Free: bind chains, run correctness
  - [ ] Task: stack safety, API completeness
  - [ ] Eval: HKT compliance, shallow chain behavior

- [ ] **Property tests** — `fp-library/tests/property_tests.rs`

  - [ ] CatQueue behaves like a queue
  - [ ] CatList preserves elements through operations
  - [ ] Task respects monad laws (manual tests, not trait-based)
  - [ ] Eval respects monad laws (trait-based via EvalBrand)
  - [ ] MonadRec produces correct results

- [ ] **Stack safety tests** — `fp-library/tests/stack_safety.rs`

  - [ ] Task: Deep recursion with `tail_rec_m` (1M+ iterations)
  - [ ] Task: Long left-associated bind chains (10K+)
  - [ ] Task: Nested defer chains
  - [ ] Eval: Verify ~8000 call limit and document
  - [ ] Eval: Confirm HKT operations work at shallow depth

- [ ] **Benchmark tests** — `fp-library/benches/`
  - [ ] Task: Left vs right associated binds
  - [ ] Task vs Eval: Overhead comparison for shallow chains
  - [ ] Comparison with baseline Vec approach
  - [ ] Memory usage measurements for both types

### 11.8 Documentation

- [ ] **API documentation**

  - [ ] Doc comments on all public items (Task and Eval)
  - [ ] Examples in doc comments showing appropriate use cases
  - [ ] Links to related types/methods
  - [ ] Clear guidance on when to use Task vs Eval

- [ ] **Module documentation**

  - [ ] Overview of two-type architecture design decisions
  - [ ] Performance characteristics for both types
  - [ ] Migration guide from existing Lazy
  - [ ] HKT compatibility notes (Eval only)
  - [ ] Stack safety warnings for Eval

- [ ] **Architecture documentation**
  - [ ] Update `docs/architecture.md` with Task/Eval dual design
  - [ ] Add Mermaid diagrams for data flow
  - [ ] Document the HKT vs `'static` trade-off

### 11.9 Migration

- [ ] **Deprecate existing Lazy** (in future version)

  - [ ] Add deprecation warnings with migration hints
  - [ ] Guide users to Task (for stack safety) or Eval (for HKT)
  - [ ] Provide adapter methods if needed

- [ ] **Update dependent code**
  - [ ] Identify usages of current Lazy
  - [ ] Plan migration to Task/Eval/Memo based on use case

### 11.10 Future Enhancements (Not in initial scope)

- [ ] **Parallel Task** — `Task::par_map2` for parallel combination
- [ ] **Resource-safe Task** — Integration with RAII patterns
- [ ] **Async interop** — `Task::into_future` for async/await
- [ ] **Trampolined IO** — Extension point for effectful trampolines
- [ ] **Eval optimization** — Investigate TCO hints for deeper Eval chains
- [ ] **Task-to-Eval bridge** — Utilities for converting between types when safe
