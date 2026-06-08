// Verifies that the Erased Free family is intentionally inherent-method
// only (no `FreeBrand`, `RcFreeBrand`, or `ArcFreeBrand`).
//
// Per the Erased/Explicit dispatch split (decisions section 4.4), the
// Erased family does not carry Brand dispatch: typeclass-generic code
// must use the Explicit family (`FreeExplicit`, `RcFreeExplicit`, or
// `ArcFreeExplicit`) and their corresponding `*ExplicitBrand` types.
// Importing a non-existent Erased brand fails to resolve.

use fp_library::brands::FreeBrand;

fn main() {}
