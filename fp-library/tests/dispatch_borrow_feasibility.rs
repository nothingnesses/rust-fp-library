#![expect(clippy::unimplemented, reason = "Tests use panicking operations for brevity and clarity")]

//! Feasibility tests for adding a container type parameter to dispatch traits,
//! enabling dispatch free functions to accept both owned and borrowed containers.

#[expect(unused, reason = "Feasibility test helpers used only within module")]
#[cfg(test)]
mod tests {
	// ================================================================
	// Minimal HKT setup
	// ================================================================

	trait Kind {
		type Of<A>;
	}

	struct VecBrand;
	impl Kind for VecBrand {
		type Of<A> = Vec<A>;
	}

	struct OptionBrand;
	impl Kind for OptionBrand {
		type Of<A> = Option<A>;
	}

	// Dispatch markers
	struct Val;
	struct Ref;

	// ================================================================
	// Note: A fully generic FunctorDispatch<Brand, A, B, FA, Marker>
	// with three impls (owned+Val, borrowed+Ref, owned+Ref) causes
	// E0119 (conflicting impls) because the compiler cannot rule out
	// Brand::Of<A> being a reference type. This is a limitation of
	// generic associated types with blanket impls.
	//
	// The actual dispatch system uses concrete Brand types in its impls
	// (via Apply! macro), so this is not an issue in practice. The tests
	// below use concrete per-type dispatch traits to validate the pattern.
	// ================================================================

	// ================================================================
	// Test 1: Concrete Vec dispatch with all three impls
	// ================================================================

	trait VecFunctorDispatch<A, B, FA, Marker> {
		fn dispatch(
			self,
			fa: FA,
		) -> Vec<B>;
	}

	// Val: owned Vec, Fn(A) -> B
	impl<A, B, F> VecFunctorDispatch<A, B, Vec<A>, Val> for F
	where
		F: Fn(A) -> B,
	{
		fn dispatch(
			self,
			fa: Vec<A>,
		) -> Vec<B> {
			fa.into_iter().map(self).collect()
		}
	}

	// Ref: borrowed Vec, Fn(&A) -> B
	impl<'b, A, B, F> VecFunctorDispatch<A, B, &'b Vec<A>, Ref> for F
	where
		F: Fn(&A) -> B,
	{
		fn dispatch(
			self,
			fa: &'b Vec<A>,
		) -> Vec<B> {
			fa.iter().map(self).collect()
		}
	}

	// Ref: owned Vec, Fn(&A) -> B (borrows internally)
	impl<A, B, F> VecFunctorDispatch<A, B, Vec<A>, Ref> for F
	where
		F: Fn(&A) -> B,
	{
		fn dispatch(
			self,
			fa: Vec<A>,
		) -> Vec<B> {
			fa.iter().map(self).collect()
		}
	}

	// Unified free function
	fn vec_map<A, B, FA, Marker>(
		f: impl VecFunctorDispatch<A, B, FA, Marker>,
		fa: FA,
	) -> Vec<B> {
		f.dispatch(fa)
	}

	#[test]
	fn val_owned() {
		// Fn(i32) -> String with owned Vec
		let v = vec![1, 2, 3];
		let result = vec_map(|x: i32| x.to_string(), v);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	#[test]
	fn ref_owned() {
		// Fn(&i32) -> String with owned Vec
		let v = vec![1, 2, 3];
		let result = vec_map(|x: &i32| x.to_string(), v);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	#[test]
	fn ref_borrowed() {
		// Fn(&i32) -> String with borrowed Vec
		let v = vec![1, 2, 3];
		let result = vec_map(|x: &i32| x.to_string(), &v);
		assert_eq!(result, vec!["1", "2", "3"]);
		// v is still usable
		assert_eq!(v, vec![1, 2, 3]);
	}

	#[test]
	fn ref_borrowed_reuse() {
		// Multiple calls reusing the same Vec
		let v = vec![1, 2, 3];
		let a = vec_map(|x: &i32| x.to_string(), &v);
		let b = vec_map(|x: &i32| x * 2, &v);
		assert_eq!(a, vec!["1", "2", "3"]);
		assert_eq!(b, vec![2, 4, 6]);
		assert_eq!(v, vec![1, 2, 3]);
	}

	// ================================================================
	// Test 3: Option dispatch with all three impls
	// ================================================================

	trait OptionFunctorDispatch<A, B, FA, Marker> {
		fn dispatch(
			self,
			fa: FA,
		) -> Option<B>;
	}

	impl<A, B, F> OptionFunctorDispatch<A, B, Option<A>, Val> for F
	where
		F: Fn(A) -> B,
	{
		fn dispatch(
			self,
			fa: Option<A>,
		) -> Option<B> {
			fa.map(self)
		}
	}

	impl<'b, A, B, F> OptionFunctorDispatch<A, B, &'b Option<A>, Ref> for F
	where
		F: Fn(&A) -> B,
	{
		fn dispatch(
			self,
			fa: &'b Option<A>,
		) -> Option<B> {
			fa.as_ref().map(self)
		}
	}

	impl<A, B, F> OptionFunctorDispatch<A, B, Option<A>, Ref> for F
	where
		F: Fn(&A) -> B,
	{
		fn dispatch(
			self,
			fa: Option<A>,
		) -> Option<B> {
			fa.as_ref().map(self)
		}
	}

	fn option_map<A, B, FA, Marker>(
		f: impl OptionFunctorDispatch<A, B, FA, Marker>,
		fa: FA,
	) -> Option<B> {
		f.dispatch(fa)
	}

	#[test]
	fn option_val_owned() {
		let result = option_map(|x: i32| x.to_string(), Some(42));
		assert_eq!(result, Some("42".to_string()));
	}

	#[test]
	fn option_ref_owned() {
		let result = option_map(|x: &i32| x.to_string(), Some(42));
		assert_eq!(result, Some("42".to_string()));
	}

	#[test]
	fn option_ref_borrowed() {
		let v = Some(42);
		let result = option_map(|x: &i32| x.to_string(), &v);
		assert_eq!(result, Some("42".to_string()));
		assert_eq!(v, Some(42));
	}

	// ================================================================
	// Test 4: BindDispatch with FA type parameter
	// ================================================================

	trait VecBindDispatch<A, B, FA, Marker> {
		fn dispatch(
			self,
			fa: FA,
		) -> Vec<B>;
	}

	impl<A, B, F> VecBindDispatch<A, B, Vec<A>, Val> for F
	where
		F: Fn(A) -> Vec<B>,
	{
		fn dispatch(
			self,
			fa: Vec<A>,
		) -> Vec<B> {
			fa.into_iter().flat_map(self).collect()
		}
	}

	impl<'b, A, B, F> VecBindDispatch<A, B, &'b Vec<A>, Ref> for F
	where
		F: Fn(&A) -> Vec<B>,
	{
		fn dispatch(
			self,
			fa: &'b Vec<A>,
		) -> Vec<B> {
			fa.iter().flat_map(self).collect()
		}
	}

	impl<A, B, F> VecBindDispatch<A, B, Vec<A>, Ref> for F
	where
		F: Fn(&A) -> Vec<B>,
	{
		fn dispatch(
			self,
			fa: Vec<A>,
		) -> Vec<B> {
			fa.iter().flat_map(self).collect()
		}
	}

	fn vec_bind<A, B, FA, Marker>(
		fa: FA,
		f: impl VecBindDispatch<A, B, FA, Marker>,
	) -> Vec<B> {
		f.dispatch(fa)
	}

	#[test]
	fn bind_val_owned() {
		let result = vec_bind(vec![1, 2, 3], |x: i32| vec![x, x * 10]);
		assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
	}

	#[test]
	fn bind_ref_owned() {
		let result = vec_bind(vec![1, 2, 3], |x: &i32| vec![*x, *x * 10]);
		assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
	}

	#[test]
	fn bind_ref_borrowed() {
		let v = vec![1, 2, 3];
		let result = vec_bind(&v, |x: &i32| vec![*x, *x * 10]);
		assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
		assert_eq!(v, vec![1, 2, 3]);
	}

	// ================================================================
	// Test 5: Lift2Dispatch with FA/FB type parameters
	// ================================================================

	trait VecLift2Dispatch<A, B, C, FA, FB, Marker> {
		fn dispatch(
			self,
			fa: FA,
			fb: FB,
		) -> Vec<C>;
	}

	impl<A, B, C, F> VecLift2Dispatch<A, B, C, Vec<A>, Vec<B>, Val> for F
	where
		F: Fn(A, B) -> C,
		A: Clone,
		B: Clone,
	{
		fn dispatch(
			self,
			fa: Vec<A>,
			fb: Vec<B>,
		) -> Vec<C> {
			fa.into_iter()
				.flat_map(|a| fb.iter().map(move |b| (a.clone(), b.clone())))
				.map(|(a, b)| (self)(a, b))
				.collect()
		}
	}

	impl<'b1, 'b2, A, B, C, F> VecLift2Dispatch<A, B, C, &'b1 Vec<A>, &'b2 Vec<B>, Ref> for F
	where
		F: Fn(&A, &B) -> C + Clone,
	{
		fn dispatch(
			self,
			fa: &'b1 Vec<A>,
			fb: &'b2 Vec<B>,
		) -> Vec<C> {
			fa.iter()
				.flat_map(|a| {
					let f = self.clone();
					fb.iter().map(move |b| f(a, b))
				})
				.collect()
		}
	}

	fn vec_lift2<A, B, C, FA, FB, Marker>(
		f: impl VecLift2Dispatch<A, B, C, FA, FB, Marker>,
		fa: FA,
		fb: FB,
	) -> Vec<C> {
		f.dispatch(fa, fb)
	}

	#[test]
	fn lift2_val_owned() {
		let result = vec_lift2(|x: i32, y: i32| x + y, vec![1, 2], vec![10, 20]);
		assert_eq!(result, vec![11, 21, 12, 22]);
	}

	#[test]
	fn lift2_ref_borrowed() {
		let a = vec![1, 2];
		let b = vec![10, 20];
		let result = vec_lift2(|x: &i32, y: &i32| x + y, &a, &b);
		assert_eq!(result, vec![11, 21, 12, 22]);
		assert_eq!(a, vec![1, 2]);
		assert_eq!(b, vec![10, 20]);
	}

	// ================================================================
	// Test 6: Temporary borrow in dispatch
	// ================================================================

	fn make_vec() -> Vec<i32> {
		vec![1, 2, 3]
	}

	#[test]
	fn dispatch_temporary_borrow() {
		// &make_vec() as a temporary borrow passed to dispatch
		let result = vec_map(|x: &i32| x * 2, &make_vec());
		assert_eq!(result, vec![2, 4, 6]);
	}

	// ================================================================
	// Test 7: Nested dispatch with temporaries
	// ================================================================

	#[test]
	fn dispatch_nested_bind_borrowed() {
		let v = vec![1, 2];
		// Inner bind produces a temporary Vec, outer bind borrows it
		let result = vec_bind(&vec_bind(&v, |x: &i32| vec![*x, *x * 10]), |y: &i32| vec![*y + 100]);
		assert_eq!(result, vec![101, 110, 102, 120]);
		assert_eq!(v, vec![1, 2]);
	}

	// ================================================================
	// Test 8: Mixed owned/borrowed in same scope
	// ================================================================

	#[test]
	fn dispatch_mixed_modes() {
		let v = vec![1, 2, 3];

		// Borrow for ref map
		let strings = vec_map(|x: &i32| x.to_string(), &v);

		// Consume for val map (moves v)
		let doubled = vec_map(|x: i32| x * 2, v);

		assert_eq!(strings, vec!["1", "2", "3"]);
		assert_eq!(doubled, vec![2, 4, 6]);
		// v is no longer available here (consumed by val map) - correct!
	}

	// ================================================================
	// Test 9: Inference without explicit type annotations
	// ================================================================

	#[test]
	fn dispatch_inference_typed_val() {
		// Inference requires type annotation on closure param (same as current dispatch)
		let v = vec![1, 2, 3];
		let result: Vec<String> = vec_map(|x: i32| x.to_string(), v);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	#[test]
	fn dispatch_inference_ref_borrow() {
		// Can the compiler infer Marker=Ref and FA=&Vec<i32>?
		let v = vec![1, 2, 3];
		let result: Vec<String> = vec_map(|x: &i32| x.to_string(), &v);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	// ================================================================
	// Test 10: Verify that passing owned to a Ref closure still works
	// (backward compatibility with current dispatch behavior)
	// ================================================================

	#[test]
	fn dispatch_ref_closure_owned_container() {
		let result = vec_map(|x: &i32| x.to_string(), vec![1, 2, 3]);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	// ================================================================
	// Test 11: Verify conflicting impls don't cause ambiguity
	// When FA=Vec<A> and closure is Fn(&A) -> B, only the Ref+Owned impl
	// should be selected, not the Val impl.
	// ================================================================

	#[test]
	fn dispatch_no_ambiguity_ref_owned() {
		// This closure is Fn(&i32) -> String, not Fn(i32) -> String
		// So Marker should be inferred as Ref, not Val
		// FA is Vec<i32> (owned), so the Ref+Owned impl should be selected
		let result: Vec<String> = vec_map(|x: &i32| x.to_string(), vec![42]);
		assert_eq!(result, vec!["42"]);
	}

	#[test]
	fn dispatch_no_ambiguity_ref_borrowed() {
		// This closure is Fn(&i32) -> String
		// FA is &Vec<i32> (borrowed), so the Ref+Borrowed impl should be selected
		let v = vec![42];
		let result: Vec<String> = vec_map(|x: &i32| x.to_string(), &v);
		assert_eq!(result, vec!["42"]);
		assert_eq!(v, vec![42]);
	}

	// ================================================================
	// Test 12: GAT projection dispatch (simulates Apply! macro)
	// Two-impl pattern: Val takes owned, Ref takes borrowed.
	// No "owned+Ref" impl (avoids E0119).
	// ================================================================

	trait GatKind {
		type Of<A>;
	}

	struct GatVec;
	impl GatKind for GatVec {
		type Of<A> = Vec<A>;
	}

	struct GatOption;
	impl GatKind for GatOption {
		type Of<A> = Option<A>;
	}

	trait GatFunctorDispatch<'a, Brand: GatKind, A, B, FA, Marker> {
		fn dispatch(
			self,
			fa: FA,
		) -> Brand::Of<B>;
	}

	// Val: FA = <Brand as GatKind>::Of<A> (owned)
	impl<'a, Brand: GatKind, A, B, F>
		GatFunctorDispatch<'a, Brand, A, B, <Brand as GatKind>::Of<A>, Val> for F
	where
		F: Fn(A) -> B,
	{
		fn dispatch(
			self,
			_fa: <Brand as GatKind>::Of<A>,
		) -> Brand::Of<B> {
			unimplemented!("generic val - use concrete tests")
		}
	}

	// Ref: FA = &<Brand as GatKind>::Of<A> (borrowed)
	impl<'a, 'b, Brand: GatKind, A, B, F>
		GatFunctorDispatch<'a, Brand, A, B, &'b <Brand as GatKind>::Of<A>, Ref> for F
	where
		F: Fn(&A) -> B,
	{
		fn dispatch(
			self,
			_fa: &'b <Brand as GatKind>::Of<A>,
		) -> Brand::Of<B> {
			unimplemented!("generic ref - use concrete tests")
		}
	}

	// Unified free function: FA is inferred from the argument
	fn gat_map<'a, Brand: GatKind, A, B, FA, Marker>(
		f: impl GatFunctorDispatch<'a, Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Brand::Of<B> {
		f.dispatch(fa)
	}

	// This compiles, proving the two-impl pattern works with GAT projections.
	// The Val impl uses FA = Brand::Of<A>, the Ref impl uses FA = &Brand::Of<A>.
	// No overlap because the compiler CAN distinguish T from &T when one is
	// a projection type and the other is a reference to that projection.

	// ================================================================
	// Test 13: Concrete two-impl dispatch (Vec)
	// Val: owned, Ref: borrowed only (no owned+Ref fallback)
	// ================================================================

	trait TwoImplVecDispatch<A, B, FA, Marker> {
		fn dispatch(
			self,
			fa: FA,
		) -> Vec<B>;
	}

	// Val: owned Vec, Fn(A) -> B
	impl<A, B, F> TwoImplVecDispatch<A, B, Vec<A>, Val> for F
	where
		F: Fn(A) -> B,
	{
		fn dispatch(
			self,
			fa: Vec<A>,
		) -> Vec<B> {
			fa.into_iter().map(self).collect()
		}
	}

	// Ref: borrowed Vec, Fn(&A) -> B
	impl<'b, A, B, F> TwoImplVecDispatch<A, B, &'b Vec<A>, Ref> for F
	where
		F: Fn(&A) -> B,
	{
		fn dispatch(
			self,
			fa: &'b Vec<A>,
		) -> Vec<B> {
			fa.iter().map(self).collect()
		}
	}

	fn two_impl_map<A, B, FA, Marker>(
		f: impl TwoImplVecDispatch<A, B, FA, Marker>,
		fa: FA,
	) -> Vec<B> {
		f.dispatch(fa)
	}

	#[test]
	fn two_impl_val_owned() {
		let result = two_impl_map(|x: i32| x.to_string(), vec![1, 2, 3]);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	#[test]
	fn two_impl_ref_borrowed() {
		let v = vec![1, 2, 3];
		let result = two_impl_map(|x: &i32| x.to_string(), &v);
		assert_eq!(result, vec!["1", "2", "3"]);
		assert_eq!(v, vec![1, 2, 3]);
	}

	#[test]
	fn two_impl_ref_borrowed_reuse() {
		let v = vec![1, 2, 3];
		let a = two_impl_map(|x: &i32| x.to_string(), &v);
		let b = two_impl_map(|x: &i32| x * 2, &v);
		assert_eq!(a, vec!["1", "2", "3"]);
		assert_eq!(b, vec![2, 4, 6]);
	}

	#[test]
	fn two_impl_ref_temporary() {
		let result = two_impl_map(|x: &i32| x * 2, &vec![1, 2, 3]);
		assert_eq!(result, vec![2, 4, 6]);
	}

	#[test]
	fn two_impl_nested_bind() {
		trait TwoImplVecBind<A, B, FA, Marker> {
			fn dispatch(
				self,
				fa: FA,
			) -> Vec<B>;
		}

		impl<A, B, F> TwoImplVecBind<A, B, Vec<A>, Val> for F
		where
			F: Fn(A) -> Vec<B>,
		{
			fn dispatch(
				self,
				fa: Vec<A>,
			) -> Vec<B> {
				fa.into_iter().flat_map(self).collect()
			}
		}

		impl<'b, A, B, F> TwoImplVecBind<A, B, &'b Vec<A>, Ref> for F
		where
			F: Fn(&A) -> Vec<B>,
		{
			fn dispatch(
				self,
				fa: &'b Vec<A>,
			) -> Vec<B> {
				fa.iter().flat_map(self).collect()
			}
		}

		fn two_bind<A, B, FA, Marker>(
			fa: FA,
			f: impl TwoImplVecBind<A, B, FA, Marker>,
		) -> Vec<B> {
			f.dispatch(fa)
		}

		let v = vec![1, 2];
		// Nested: inner produces temporary, outer borrows it
		let result = two_bind(&two_bind(&v, |x: &i32| vec![*x, *x * 10]), |y: &i32| vec![*y + 100]);
		assert_eq!(result, vec![101, 110, 102, 120]);
		assert_eq!(v, vec![1, 2]);
	}

	// ================================================================
	// Test 14: Verify that Ref closure with owned container does NOT compile
	// (This is the intentional breaking change)
	// ================================================================

	// Uncomment to verify this does NOT compile:
	// fn _two_impl_ref_owned_should_fail() {
	//     // Fn(&i32) -> String with owned Vec: no matching impl
	//     let _ = two_impl_map(|x: &i32| x.to_string(), vec![1, 2, 3]);
	// }
	// Expected error: no implementation for `{closure} : TwoImplVecDispatch<_, _, Vec<i32>, _>`

	// ================================================================
	// Test 15: Mixed usage in same scope
	// ================================================================

	#[test]
	fn two_impl_mixed_modes() {
		let v = vec![1, 2, 3];

		// First: borrow for ref map (v preserved)
		let strings = two_impl_map(|x: &i32| x.to_string(), &v);

		// Then: consume for val map (v moved)
		let doubled = two_impl_map(|x: i32| x * 2, v);

		assert_eq!(strings, vec!["1", "2", "3"]);
		assert_eq!(doubled, vec![2, 4, 6]);
	}

	// ================================================================
	// Test 16: Two-impl with GAT projection (the real dispatch pattern)
	// ================================================================

	trait GatVecFunctorDispatch<'a, A, B, FA, Marker> {
		fn dispatch(
			self,
			fa: FA,
		) -> <GatVec as GatKind>::Of<B>;
	}

	impl<'a, A, B, F> GatVecFunctorDispatch<'a, A, B, <GatVec as GatKind>::Of<A>, Val> for F
	where
		F: Fn(A) -> B,
	{
		fn dispatch(
			self,
			fa: <GatVec as GatKind>::Of<A>,
		) -> Vec<B> {
			fa.into_iter().map(self).collect()
		}
	}

	impl<'a, 'b, A, B, F> GatVecFunctorDispatch<'a, A, B, &'b <GatVec as GatKind>::Of<A>, Ref> for F
	where
		F: Fn(&A) -> B,
	{
		fn dispatch(
			self,
			fa: &'b <GatVec as GatKind>::Of<A>,
		) -> Vec<B> {
			fa.iter().map(self).collect()
		}
	}

	fn gat_vec_map<'a, A, B, FA, Marker>(
		f: impl GatVecFunctorDispatch<'a, A, B, FA, Marker>,
		fa: FA,
	) -> Vec<B> {
		f.dispatch(fa)
	}

	#[test]
	fn gat_projection_val() {
		let result = gat_vec_map(|x: i32| x.to_string(), vec![1, 2, 3]);
		assert_eq!(result, vec!["1", "2", "3"]);
	}

	#[test]
	fn gat_projection_ref_borrowed() {
		let v = vec![1, 2, 3];
		let result = gat_vec_map(|x: &i32| x.to_string(), &v);
		assert_eq!(result, vec!["1", "2", "3"]);
		assert_eq!(v, vec![1, 2, 3]);
	}
}
