# Step 01: Data Structures

## Goal
Implement the foundational data structures `CatQueue` and `CatList` required for the "Reflection without Remorse" optimization in the Free monad. These structures provide O(1) amortized append and uncons operations, enabling stack-safe left-associated binds.

## Files to Create
- `fp-library/src/types/cat_queue.rs`
- `fp-library/src/types/cat_list.rs`

## Files to Modify
- `fp-library/src/types.rs` (to expose the new modules)

## Implementation Details

### CatQueue: O(1) Amortized Double-Ended Queue

A double-ended queue implemented using two `Vec`s ("banker's queue").

#### Design Rationale

CatQueue is the foundation of CatList. It provides a double-ended queue with O(1) amortized operations using the "two-list queue" technique from Okasaki's work.

**Key insight**: A queue can be implemented with two stacks (or in our case, two Vecs):

- `front`: Elements ready to be dequeued (in order)
- `back`: Elements recently enqueued (in reverse order)

When `front` is empty and we need to dequeue, we reverse `back` and swap it into `front`. This reversal is O(n), but it happens at most once per element over its lifetime, giving O(1) amortized cost.

**Invariant**: `front` contains elements in FIFO order (head at end), `back` contains elements in LIFO order.

#### Rust Implementation

```rust
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
```

#### Design Decisions for Rust

**Decision: Use `Vec` Instead of Linked List**
**Rationale**: PureScript uses linked lists (`List a`), but Rust's `Vec` offers:
- Better cache locality
- Amortized O(1) `push` and `pop`
- No allocation per element
The trade-off is that `Vec` requires elements to be contiguous, but since we only access from the ends, this works perfectly.

**Decision: Store Front in Reverse Order**
**Rationale**: We store `front` so that `pop()` removes the logical "head". This means:
- `cons(a)` → `front.push(a)` — O(1)
- `uncons()` → `front.pop()` — O(1)
- When `front` is empty, `reverse(back)` gives us the correct order
This matches PureScript's behavior but uses Vec's natural operations.

**Decision: Consuming Methods**
**Rationale**: `uncons` and `unsnoc` consume `self` and return the modified queue. This matches the functional style and avoids interior mutability. For iteration, we provide `IntoIterator`.

#### Correctness Argument

**Invariant**: Elements in `front` (read from end) followed by elements in `back` (read from start after reversal) form the logical queue order.

**Amortized Analysis**: Each element is:
1. Pushed to `back`: O(1)
2. Moved to `front` (via reversal): O(1) amortized (happens once per element)
3. Popped from `front`: O(1)

Total cost per element: O(1) amortized.

### CatList: O(1) Catenable List

A catenable list supporting O(1) concatenation.
- **Nil**: Empty list.
- **Cons**: Head element + Queue of sublists.

#### Design Rationale

CatList is a "catenable list" — a list that supports O(1) concatenation. This is the key data structure that enables O(1) bind operations in the Free monad.

**Key insight**: A CatList is either empty, or a head element plus a queue of CatLists. When we concatenate two CatLists, we just enqueue the second onto the first's queue — no traversal needed.

```
CatList a = CatNil | CatCons a (CatQueue (CatList a))
```

#### Rust Implementation

```rust
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
```

#### Design Decisions for Rust

**Decision: Enum Representation**
**Rationale**: We use a Rust enum directly mirroring PureScript's algebraic data type. This gives us:
- Pattern matching for clean code
- Zero-cost abstraction (no boxing for the enum discriminant)
- Clear structural representation

**Decision: Consuming vs Borrowing `uncons`**
**Rationale**: We chose consuming `uncons(self) -> Option<(A, Self)>` because:
1. CatList is typically used linearly (process once, discard)
2. Avoids lifetime complexity
3. Matches PureScript's functional style
If shared access is needed, wrap in `Rc<RefCell<CatList<A>>>`.

**Decision: Iterative `flatten_queue`**
**Rationale**: PureScript's `foldr link CatNil q` is recursive. In Rust, we must be careful about stack depth. Our implementation:
1. Collects sublists into a Vec (iterative)
2. Right-folds using `.rev().fold()` (iterative)
This ensures stack safety even for deeply nested CatLists.

#### Amortized Analysis

**Claim**: A sequence of n `snoc` operations followed by n `uncons` operations takes O(n) total time.

**Proof sketch**:
1. Each `snoc` is O(1) — just appends to queue
2. Each element is involved in at most one `link` during `flatten_queue`
3. Using the "banker's method": each element pays for its own flattening

**Potential function**: Φ(CatList) = number of sublists in all queues.
- `snoc`: increases Φ by 1, actual cost O(1), amortized cost O(1)
- `uncons`: decreases Φ by k (number of sublists flattened), actual cost O(k), amortized cost O(1)

#### Why Not Use VecDeque?

One might ask: why not just use `std::collections::VecDeque`?

**Answer**: VecDeque has O(n) concatenation, which defeats the purpose. The key feature of CatList is O(1) concatenation, which requires the nested structure with deferred flattening.

| Operation  | CatList        | VecDeque       |
| ---------- | -------------- | -------------- |
| push_back  | O(1)           | O(1) amortized |
| pop_front  | O(1) amortized | O(1)           |
| **concat** | **O(1)**       | **O(n)**       |

For the Free monad's bind stack, we need O(1) concat to avoid O(n²) left-bind degradation.

## Tests

### CatQueue Tests
1.  **Basic Operations**: `empty`, `is_empty`, `len`, `singleton`.
2.  **FIFO Behavior**: `snoc` multiple items, `uncons` them, verify order.
3.  **Amortization Logic**: `snoc` many items, `uncons` all. Verify `front` is refilled from `back`.
4.  **Double-Ended**: `cons` (push front) and `unsnoc` (pop back) behavior.

### CatList Tests
1.  **Basic Operations**: `empty`, `singleton`, `cons`, `snoc`.
2.  **Concatenation**: `append` two lists, verify order.
3.  **Flattening**: Create a nested structure via multiple appends, `uncons` all elements to verify `flatten_queue` logic works correctly and iteratively.
4.  **Iteration**: Verify `IntoIterator` works correctly.

## Checklist
- [ ] Create `fp-library/src/types/cat_queue.rs`
    - [ ] Implement struct and `new`/`empty`
    - [ ] Implement `snoc`, `cons`
    - [ ] Implement `uncons`, `unsnoc` (with reversal logic)
    - [ ] Implement `IntoIterator`
    - [ ] Add unit tests
- [ ] Create `fp-library/src/types/cat_list.rs`
    - [ ] Implement enum `CatList`
    - [ ] Implement `link` helper
    - [ ] Implement `append`
    - [ ] Implement `uncons` with iterative `flatten_queue`
    - [ ] Implement `IntoIterator`
    - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs` to export `cat_queue` and `cat_list`
