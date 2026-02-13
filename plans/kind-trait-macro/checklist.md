# Task Checklist

- [ ] **Setup**
    - [ ] Verify `fp-macros` crate structure and `Cargo.toml`.
    - [ ] Add dependencies (`syn`, `quote`, `proc-macro2`) to `fp-macros/Cargo.toml`.

- [ ] **Implementation: Parsing**
    - [ ] Define `KindInput` struct to represent parsed arguments.
    - [ ] Implement `Parse` trait for `KindInput`.
    - [ ] Handle lifetimes tuple `('a, 'b)`.
    - [ ] Handle types tuple with bounds `(T: 'a, U)`.
    - [ ] Handle output bounds `(: 'a)`.

- [ ] **Implementation: Logic**
    - [ ] Implement `Canonicalizer` to map identifiers to indices.
    - [ ] Implement logic to generate the `Kind_...` string.
    - [ ] Ensure bounds are sorted/normalized.

- [ ] **Implementation: Macros**
    - [ ] Implement `Kind!` proc-macro (generates `Ident`).
    - [ ] Implement `def_kind!` proc-macro (generates `Trait` definition).

- [ ] **Integration & Testing**
    - [ ] Add `fp-macros` as dependency to `fp-library`.
    - [ ] Replace manual usages in `fp-library` with `def_kind!`.
    - [ ] Verify compilation and correctness.
