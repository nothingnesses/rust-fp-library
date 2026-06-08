use {
	super::{
		super::generator_descriptors::{
			self,
			EffectName,
			RunWrapperMethod,
			WrapperName,
		},
		impl_items_from_tokens,
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::ImplItem,
};

pub(super) fn kv_store_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::KVStore, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Lookup) => run_lookup_tokens(),
		(WrapperName::Run, RunWrapperMethod::Update) => run_update_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunKVStore) => run_run_kv_store_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Lookup) => rcrun_lookup_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Update) => rcrun_update_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunKVStore) => rcrun_run_kv_store_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Lookup) => arcrun_lookup_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Update) => arcrun_update_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunKVStore) => arcrun_run_kv_store_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Lookup) => run_explicit_lookup_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Update) => run_explicit_update_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunKVStore) =>
			run_explicit_run_kv_store_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Lookup) => rcrun_explicit_lookup_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Update) => rcrun_explicit_update_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunKVStore) =>
			rcrun_explicit_run_kv_store_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Lookup) => arcrun_explicit_lookup_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Update) => arcrun_explicit_update_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunKVStore) =>
			arcrun_explicit_run_kv_store_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_lookup_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore lookup effect into the Run program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to look up.")]
		#[document_returns("A `Run` program returning the current value for the key, if any.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxKVStoreBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, Option<i32>> = Run::lookup::<_, _>("a");
		/// let handled: Run<CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract().0, Some(7));
		/// ```
		#[inline]
		pub fn lookup<K, Idx>(key: K) -> Self
		where
			K: 'static,
			V: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Option<V>>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<
						'static,
						crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>,
						Option<V>,
					>,
					Idx,
				>, {
			let effect: crate::types::effects::kv_store::BoxKVStore<
				'static,
				crate::brands::BoxBrand,
				K,
				V,
				Option<V>,
			> = crate::types::effects::kv_store::BoxKVStore::Lookup(
				key,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|value: Option<V>| value),
			);
			Self::lift::<
				crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn run_update_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore update effect into the Run program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to update.", "The replacement value, or `None` to delete.")]
		#[document_returns("A `Run` program returning unit after the update is handled.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxKVStoreBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, ()> = Run::update::<_, i32, _>("a", Some(9));
		/// let handled: Run<CNilBrand, CNilBrand, ((), BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::new());
		/// assert_eq!(handled.extract().1.get("a"), Some(&9));
		/// ```
		#[inline]
		pub fn update<K, V, Idx>(
			key: K,
			value: Option<V>,
		) -> Self
		where
			K: 'static,
			V: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<
						'static,
						crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>,
						(),
					>,
					Idx,
				>, {
			let effect: crate::types::effects::kv_store::BoxKVStore<
				'static,
				crate::brands::BoxBrand,
				K,
				V,
				(),
			> = crate::types::effects::kv_store::BoxKVStore::Update(
				key,
				value,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|unit: ()| unit),
			);
			Self::lift::<
				crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn rcrun_lookup_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore lookup effect into the `RcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to look up.")]
		#[document_returns("An `RcRun` program returning the current value for the key, if any.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<KVStoreBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, Option<i32>> = RcRun::lookup::<_, _>("a");
		/// let handled: RcRun<CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract().0, Some(7));
		/// ```
		#[inline]
		pub fn lookup<K, Idx>(key: K) -> Self
		where
			K: Clone + 'static,
			V: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Option<V>>):
				Member<RcCoyoneda<'static, crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>, Option<V>>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::kv_store::KVStore<
				'static,
				crate::brands::RcBrand,
				K,
				V,
				Option<V>,
			> = crate::types::effects::kv_store::KVStore::Lookup(
				key,
				<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|value: Option<V>| value),
			);
			Self::lift::<
				crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn rcrun_update_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore update effect into the `RcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to update.", "The replacement value, or `None` to delete.")]
		#[document_returns("An `RcRun` program returning unit after the update is handled.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<KVStoreBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, ()> = RcRun::update::<_, i32, _>("a", Some(9));
		/// let handled: RcRun<CNilBrand, CNilBrand, ((), BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::new());
		/// assert_eq!(handled.extract().1.get("a"), Some(&9));
		/// ```
		#[inline]
		pub fn update<K, V, Idx>(
			key: K,
			value: Option<V>,
		) -> Self
		where
			K: Clone + 'static,
			V: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<RcCoyoneda<'static, crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::kv_store::KVStore<
				'static,
				crate::brands::RcBrand,
				K,
				V,
				(),
			> = crate::types::effects::kv_store::KVStore::Update(
				key,
				value,
				<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|unit: ()| unit),
			);
			Self::lift::<
				crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn arcrun_lookup_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore lookup effect into the `ArcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to look up.")]
		#[document_returns("An `ArcRun` program returning the current value for the key, if any.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendKVStoreBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, Option<i32>> = ArcRun::lookup::<_, _>("a");
		/// let handled: ArcRun<CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract().0, Some(7));
		/// ```
		#[inline]
		pub fn lookup<K, Idx>(key: K) -> Self
		where
			K: Clone + Send + Sync + 'static,
			V: Clone + Send + Sync + 'static,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Option<V>>): Member<
				ArcCoyoneda<'static, crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>, Option<V>>,
				Idx,
			>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::kv_store::SendKVStore<
				'static,
				crate::brands::ArcBrand,
				K,
				V,
				Option<V>,
			> = crate::types::effects::kv_store::SendKVStore::Lookup(
				key,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|value: Option<V>| value),
			);
			Self::lift::<
				crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn arcrun_update_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore update effect into the `ArcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to update.", "The replacement value, or `None` to delete.")]
		#[document_returns("An `ArcRun` program returning unit after the update is handled.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendKVStoreBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, ()> = ArcRun::update::<_, i32, _>("a", Some(9));
		/// let handled: ArcRun<CNilBrand, CNilBrand, ((), BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::new());
		/// assert_eq!(handled.extract().1.get("a"), Some(&9));
		/// ```
		#[inline]
		pub fn update<K, V, Idx>(
			key: K,
			value: Option<V>,
		) -> Self
		where
			K: Clone + Send + Sync + 'static,
			V: Clone + Send + Sync + 'static,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>): Member<
				ArcCoyoneda<'static, crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>, ()>,
				Idx,
			>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::kv_store::SendKVStore<
				'static,
				crate::brands::ArcBrand,
				K,
				V,
				(),
			> = crate::types::effects::kv_store::SendKVStore::Update(
				key,
				value,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|unit: ()| unit),
			);
			Self::lift::<
				crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn run_explicit_lookup_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore lookup effect into the `RunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to look up.")]
		#[document_returns("A `RunExplicit` program returning the current value for the key, if any.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxKVStoreBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, Option<i32>> =
		/// 	RunExplicit::lookup::<_, _>("a");
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract().0, Some(7));
		/// ```
		#[inline]
		pub fn lookup<K, Idx>(key: K) -> Self
		where
			K: 'static + 'a,
			V: 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<V>>):
				Member<Coyoneda<'a, crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>, Option<V>>, Idx>, {
			let effect: crate::types::effects::kv_store::BoxKVStore<
				'a,
				crate::brands::BoxBrand,
				K,
				V,
				Option<V>,
			> = crate::types::effects::kv_store::BoxKVStore::Lookup(
				key,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|value: Option<V>| value),
			);
			Self::lift::<
				crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn run_explicit_update_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore update effect into the `RunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to update.", "The replacement value, or `None` to delete.")]
		#[document_returns("A `RunExplicit` program returning unit after the update is handled.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxKVStoreBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, ()> =
		/// 	RunExplicit::update::<_, i32, _>("a", Some(9));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, ((), BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::new());
		/// assert_eq!(handled.extract().1.get("a"), Some(&9));
		/// ```
		#[inline]
		pub fn update<K, V, Idx>(
			key: K,
			value: Option<V>,
		) -> Self
		where
			K: 'static + 'a,
			V: 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<Coyoneda<'a, crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>, ()>, Idx>, {
			let effect: crate::types::effects::kv_store::BoxKVStore<
				'a,
				crate::brands::BoxBrand,
				K,
				V,
				(),
			> = crate::types::effects::kv_store::BoxKVStore::Update(
				key,
				value,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|unit: ()| unit),
			);
			Self::lift::<
				crate::brands::BoxKVStoreBrand<crate::brands::BoxBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn rcrun_explicit_lookup_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore lookup effect into the `RcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to look up.")]
		#[document_returns("An `RcRunExplicit` program returning the current value for the key, if any.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<KVStoreBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, Option<i32>> =
		/// 	RcRunExplicit::lookup::<_, _>("a");
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract().0, Some(7));
		/// ```
		#[inline]
		pub fn lookup<K, Idx>(key: K) -> Self
		where
			K: Clone + 'static + 'a,
			V: Clone + 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<V>>):
				Member<RcCoyoneda<'a, crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>, Option<V>>, Idx>, {
			let effect: crate::types::effects::kv_store::KVStore<
				'a,
				crate::brands::RcBrand,
				K,
				V,
				Option<V>,
			> = crate::types::effects::kv_store::KVStore::Lookup(
				key,
				<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|value: Option<V>| value),
			);
			Self::lift::<
				crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn rcrun_explicit_update_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore update effect into the `RcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to update.", "The replacement value, or `None` to delete.")]
		#[document_returns("An `RcRunExplicit` program returning unit after the update is handled.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<KVStoreBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, ()> =
		/// 	RcRunExplicit::update::<_, i32, _>("a", Some(9));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, ((), BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::new());
		/// assert_eq!(handled.extract().1.get("a"), Some(&9));
		/// ```
		#[inline]
		pub fn update<K, V, Idx>(
			key: K,
			value: Option<V>,
		) -> Self
		where
			K: Clone + 'static + 'a,
			V: Clone + 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<RcCoyoneda<'a, crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>, ()>, Idx>, {
			let effect: crate::types::effects::kv_store::KVStore<
				'a,
				crate::brands::RcBrand,
				K,
				V,
				(),
			> = crate::types::effects::kv_store::KVStore::Update(
				key,
				value,
				<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|unit: ()| unit),
			);
			Self::lift::<
				crate::brands::KVStoreBrand<crate::brands::RcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn arcrun_explicit_lookup_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore lookup effect into the `ArcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to look up.")]
		#[document_returns("An `ArcRunExplicit` program returning the current value for the key, if any.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendKVStoreBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, Option<i32>> =
		/// 	ArcRunExplicit::lookup::<_, _>("a");
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract().0, Some(7));
		/// ```
		#[inline]
		pub fn lookup<K, Idx>(key: K) -> Self
		where
			K: Clone + Send + Sync + 'static + 'a,
			V: Clone + Send + Sync + 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<V>>):
				Member<ArcCoyoneda<'a, crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>, Option<V>>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, Option<V>>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, Option<V>>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, Option<V>>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::kv_store::SendKVStore<
				'a,
				crate::brands::ArcBrand,
				K,
				V,
				Option<V>,
			> = crate::types::effects::kv_store::SendKVStore::Lookup(
				key,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|value: Option<V>| value),
			);
			Self::lift::<
				crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn arcrun_explicit_update_tokens() -> TokenStream {
	quote! {
		/// Lifts a KVStore update effect into the `ArcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The key to update.", "The replacement value, or `None` to delete.")]
		#[document_returns("An `ArcRunExplicit` program returning unit after the update is handled.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendKVStoreBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, ()> =
		/// 	ArcRunExplicit::update::<_, i32, _>("a", Some(9));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::new());
		/// assert_eq!(handled.extract().1.get("a"), Some(&9));
		/// ```
		#[inline]
		pub fn update<K, V, Idx>(
			key: K,
			value: Option<V>,
		) -> Self
		where
			K: Clone + Send + Sync + 'static + 'a,
			V: Clone + Send + Sync + 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<ArcCoyoneda<'a, crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>, ()>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::kv_store::SendKVStore<
				'a,
				crate::brands::ArcBrand,
				K,
				V,
				(),
			> = crate::types::effects::kv_store::SendKVStore::Update(
				key,
				value,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|unit: ()| unit),
			);
			Self::lift::<
				crate::brands::SendKVStoreBrand<crate::brands::ArcBrand, K, V>,
				Idx,
			>(effect)
		}
	}
}

fn run_run_kv_store_tokens() -> TokenStream {
	quote! {
		/// Interprets one KVStore effect with a `BTreeMap` store.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness for the KVStore effect.",
			"The first-order row brand with the KVStore effect removed."
		)]
		#[document_parameters("The initial map.")]
		#[document_returns("A first-order-only `Run` program returning `(result, final_map)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxKVStoreBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, Option<i32>> =
		/// 	Run::<Row, CNilBrand, ()>::update::<_, i32, _>("a", None)
		/// 		.bind(|()| Run::<Row, CNilBrand, Option<i32>>::lookup::<_, _>("a"));
		/// let handled: Run<CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract(), (None, BTreeMap::new()));
		/// ```
		#[inline]
		pub fn run_kv_store<K, V, Idx, RMinusKVStore>(
			self,
			initial: std::collections::BTreeMap<K, V>,
		) -> Run<RMinusKVStore, CNilBrand, (A, std::collections::BTreeMap<K, V>)>
		where
			K: Ord + Clone + 'static,
			V: Clone + 'static,
			RMinusKVStore: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
				Coyoneda<'static, BoxKVStoreBrand<BoxBrand, K, V>, Run<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, CNilBrand, A>,
					>
				),
			>, {
			let store = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_store = std::rc::Rc::clone(&store);
			let handled = self.handle_with::<BoxKVStoreBrand<BoxBrand, K, V>, Idx, RMinusKVStore>(
				move |op: BoxKVStore<'static, BoxBrand, K, V, Run<RMinusKVStore, CNilBrand, A>>| {
					match op {
						BoxKVStore::Lookup(key, k) => {
							let value = {
								handler_store.borrow().get(&key).cloned()
							};
							k(value)
						}
						BoxKVStore::Update(key, value, k) => {
							{
								let mut store = handler_store.borrow_mut();
								match value {
									Some(value) => {
										store.insert(key, value);
									}
									None => {
										store.remove(&key);
									}
								}
							}
							k(())
						}
					}
				},
			);
			handled.map(move |result| (result, store.borrow().clone()))
		}
	}
}

fn rcrun_run_kv_store_tokens() -> TokenStream {
	quote! {
		/// Interprets one KVStore effect with a `BTreeMap` store.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness for the KVStore effect.",
			"The first-order row brand with the KVStore effect removed."
		)]
		#[document_parameters("The initial map.")]
		#[document_returns("A first-order-only `RcRun` program returning `(result, final_map)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<KVStoreBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, Option<i32>> =
		/// 	RcRun::<Row, CNilBrand, ()>::update::<_, i32, _>("a", None)
		/// 		.bind(|()| RcRun::<Row, CNilBrand, Option<i32>>::lookup::<_, _>("a"));
		/// let handled: RcRun<CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract(), (None, BTreeMap::new()));
		/// ```
		#[inline]
		pub fn run_kv_store<K, V, Idx, RMinusKVStore>(
			self,
			initial: std::collections::BTreeMap<K, V>,
		) -> RcRun<RMinusKVStore, CNilBrand, (A, std::collections::BTreeMap<K, V>)>
		where
			A: Clone,
			K: Ord + Clone + 'static,
			V: Clone + 'static,
			RMinusKVStore: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusKVStore, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusKVStore, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'static, KVStoreBrand<RcBrand, K, V>, RcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let store = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_store = std::rc::Rc::clone(&store);
			let handled = self.handle_with::<KVStoreBrand<RcBrand, K, V>, Idx, RMinusKVStore>(
				move |op: KVStore<'static, RcBrand, K, V, RcRun<RMinusKVStore, CNilBrand, A>>| {
					match op {
						KVStore::Lookup(key, k) => {
							let value = {
								handler_store.borrow().get(&key).cloned()
							};
							(*k)(value)
						}
						KVStore::Update(key, value, k) => {
							{
								let mut store = handler_store.borrow_mut();
								match value {
									Some(value) => {
										store.insert(key, value);
									}
									None => {
										store.remove(&key);
									}
								}
							}
							(*k)(())
						}
					}
				},
			);
			handled.map(move |result| (result, store.borrow().clone()))
		}
	}
}

fn arcrun_run_kv_store_tokens() -> TokenStream {
	quote! {
		/// Interprets one KVStore effect with a `BTreeMap` store.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness for the KVStore effect.",
			"The first-order row brand with the KVStore effect removed."
		)]
		#[document_parameters("The initial map.")]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, final_map)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendKVStoreBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, Option<i32>> =
		/// 	ArcRun::<Row, CNilBrand, ()>::update::<_, i32, _>("a", None)
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, Option<i32>>::lookup::<_, _>("a"));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract(), (None, BTreeMap::new()));
		/// ```
		#[inline]
		pub fn run_kv_store<K, V, Idx, RMinusKVStore>(
			self,
			initial: std::collections::BTreeMap<K, V>,
		) -> ArcRun<RMinusKVStore, CNilBrand, (A, std::collections::BTreeMap<K, V>)>
		where
			A: Clone + Send + Sync,
			K: Ord + Clone + Send + Sync + 'static,
			V: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusKVStore: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusKVStore, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusKVStore, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusKVStore, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusKVStore, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'static, SendKVStoreBrand<ArcBrand, K, V>, ArcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let store = std::sync::Arc::new(std::sync::Mutex::new(initial));
			let handler_store = std::sync::Arc::clone(&store);
			let handled = self.handle_with::<SendKVStoreBrand<ArcBrand, K, V>, Idx, RMinusKVStore>(
				move |op: SendKVStore<'static, ArcBrand, K, V, ArcRun<RMinusKVStore, CNilBrand, A>>| {
					match op {
						SendKVStore::Lookup(key, k) => {
							let value = {
								let guard = match handler_store.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.get(&key).cloned()
							};
							(*k)(value)
						}
						SendKVStore::Update(key, value, k) => {
							{
								let mut store = match handler_store.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								match value {
									Some(value) => {
										store.insert(key, value);
									}
									None => {
										store.remove(&key);
									}
								}
							}
							(*k)(())
						}
					}
				},
			);
			handled.map(move |result| {
				let guard = match store.lock() {
					Ok(guard) => guard,
					Err(poisoned) => poisoned.into_inner(),
				};
				(result, guard.clone())
			})
		}
	}
}

fn run_explicit_run_kv_store_tokens() -> TokenStream {
	quote! {
		/// Interprets one KVStore effect with a `BTreeMap` store.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness for the KVStore effect.",
			"The first-order row brand with the KVStore effect removed."
		)]
		#[document_parameters("The initial map.")]
		#[document_returns("A first-order-only `RunExplicit` program returning `(result, final_map)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxKVStoreBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, Option<i32>> =
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::update::<_, i32, _>("a", None)
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, Option<i32>>::lookup::<_, _>("a"));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract(), (None, BTreeMap::new()));
		/// ```
		#[inline]
		pub fn run_kv_store<K, V, Idx, RMinusKVStore>(
			self,
			initial: std::collections::BTreeMap<K, V>,
		) -> RunExplicit<'a, RMinusKVStore, CNilBrand, (A, std::collections::BTreeMap<K, V>)>
		where
			K: Ord + Clone + 'static + 'a,
			V: Clone + 'static + 'a,
			RMinusKVStore: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				Coyoneda<'a, BoxKVStoreBrand<BoxBrand, K, V>, RunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let store = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_store = std::rc::Rc::clone(&store);
			let handled = self.handle_with::<BoxKVStoreBrand<BoxBrand, K, V>, Idx, RMinusKVStore>(
				move |op: BoxKVStore<'a, BoxBrand, K, V, RunExplicit<'a, RMinusKVStore, CNilBrand, A>>| {
					match op {
						BoxKVStore::Lookup(key, k) => {
							let value = {
								handler_store.borrow().get(&key).cloned()
							};
							k(value)
						}
						BoxKVStore::Update(key, value, k) => {
							{
								let mut store = handler_store.borrow_mut();
								match value {
									Some(value) => {
										store.insert(key, value);
									}
									None => {
										store.remove(&key);
									}
								}
							}
							k(())
						}
					}
				},
			);
			handled.map(move |result| (result, store.borrow().clone()))
		}
	}
}

fn rcrun_explicit_run_kv_store_tokens() -> TokenStream {
	quote! {
		/// Interprets one KVStore effect with a `BTreeMap` store.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness for the KVStore effect.",
			"The first-order row brand with the KVStore effect removed."
		)]
		#[document_parameters("The initial map.")]
		#[document_returns("A first-order-only `RcRunExplicit` program returning `(result, final_map)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<KVStoreBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, Option<i32>> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::update::<_, i32, _>("a", None)
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, Option<i32>>::lookup::<_, _>("a"));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract(), (None, BTreeMap::new()));
		/// ```
		#[inline]
		pub fn run_kv_store<K, V, Idx, RMinusKVStore>(
			self,
			initial: std::collections::BTreeMap<K, V>,
		) -> RcRunExplicit<'a, RMinusKVStore, CNilBrand, (A, std::collections::BTreeMap<K, V>)>
		where
			A: Clone,
			K: Ord + Clone + 'static + 'a,
			V: Clone + 'static + 'a,
			RMinusKVStore: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusKVStore, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusKVStore, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusKVStore, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<
					'a,
					NodeBrand<RMinusKVStore, CNilBrand>,
					(A, std::collections::BTreeMap<K, V>),
				>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'a, KVStoreBrand<RcBrand, K, V>, RcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let store = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_store = std::rc::Rc::clone(&store);
			let handled = self.handle_with::<KVStoreBrand<RcBrand, K, V>, Idx, RMinusKVStore>(
				move |op: KVStore<'a, RcBrand, K, V, RcRunExplicit<'a, RMinusKVStore, CNilBrand, A>>| {
					match op {
						KVStore::Lookup(key, k) => {
							let value = {
								handler_store.borrow().get(&key).cloned()
							};
							(*k)(value)
						}
						KVStore::Update(key, value, k) => {
							{
								let mut store = handler_store.borrow_mut();
								match value {
									Some(value) => {
										store.insert(key, value);
									}
									None => {
										store.remove(&key);
									}
								}
							}
							(*k)(())
						}
					}
				},
			);
			handled.map(move |result| (result, store.borrow().clone()))
		}
	}
}

fn arcrun_explicit_run_kv_store_tokens() -> TokenStream {
	quote! {
		/// Interprets one KVStore effect with a `BTreeMap` store.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The key type.",
			"The stored value type.",
			"The type-level Member-position witness for the KVStore effect.",
			"The first-order row brand with the KVStore effect removed."
		)]
		#[document_parameters("The initial map.")]
		#[document_returns("A first-order-only `ArcRunExplicit` program returning `(result, final_map)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		/// use std::collections::BTreeMap;
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendKVStoreBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, Option<i32>> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, ()>::update::<_, i32, _>("a", None)
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, Option<i32>>::lookup::<_, _>("a"));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, BTreeMap<&'static str, i32>)> =
		/// 	program.run_kv_store::<_, i32, _, CNilBrand>(BTreeMap::from([("a", 7)]));
		/// assert_eq!(handled.extract(), (None, BTreeMap::new()));
		/// ```
		#[inline]
		pub fn run_kv_store<K, V, Idx, RMinusKVStore>(
			self,
			initial: std::collections::BTreeMap<K, V>,
		) -> ArcRunExplicit<'a, RMinusKVStore, CNilBrand, (A, std::collections::BTreeMap<K, V>)>
		where
			A: Clone + Send + Sync,
			K: Ord + Clone + Send + Sync + 'static + 'a,
			V: Clone + Send + Sync + 'static + 'a,
			RMinusKVStore: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusKVStore, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusKVStore, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusKVStore, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusKVStore, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<RMinusKVStore, CNilBrand>,
					(A, std::collections::BTreeMap<K, V>),
				>,
			>): Clone + Send + Sync,
			Apply!(<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusKVStore, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<RMinusKVStore, CNilBrand>,
					(A, std::collections::BTreeMap<K, V>),
				>,
			>): Send + Sync,
			Apply!(<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusKVStore, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusKVStore, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<RMinusKVStore, CNilBrand>,
					(A, std::collections::BTreeMap<K, V>),
				>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusKVStore, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<
					'a,
					SendKVStoreBrand<ArcBrand, K, V>,
					ArcRunExplicit<'a, R, CNilBrand, A>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusKVStore as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let store = std::sync::Arc::new(std::sync::Mutex::new(initial));
			let handler_store = std::sync::Arc::clone(&store);
			let handled = self.handle_with::<SendKVStoreBrand<ArcBrand, K, V>, Idx, RMinusKVStore>(
				move |op: SendKVStore<
					'a,
					ArcBrand,
					K,
					V,
					ArcRunExplicit<'a, RMinusKVStore, CNilBrand, A>,
				>| {
					match op {
						SendKVStore::Lookup(key, k) => {
							let value = {
								let guard = match handler_store.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.get(&key).cloned()
							};
							(*k)(value)
						}
						SendKVStore::Update(key, value, k) => {
							{
								let mut store = match handler_store.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								match value {
									Some(value) => {
										store.insert(key, value);
									}
									None => {
										store.remove(&key);
									}
								}
							}
							(*k)(())
						}
					}
				},
			);
			handled.map(move |result| {
				let guard = match store.lock() {
					Ok(guard) => guard,
					Err(poisoned) => poisoned.into_inner(),
				};
				(result, guard.clone())
			})
		}
	}
}
