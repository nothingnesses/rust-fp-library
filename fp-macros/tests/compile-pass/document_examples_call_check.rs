use fp_macros::{
	document_examples,
	document_module,
};

#[document_examples]
///
/// ```
/// let value = documented_function();
/// assert_eq!(value, 3);
/// ```
fn documented_function() -> i32 {
	3
}

struct Receiver;

impl Receiver {
	#[document_examples]
	///
	/// ```
	/// let receiver = Receiver;
	/// assert_eq!(receiver.documented_method(), 4);
	/// ```
	fn documented_method(&self) -> i32 {
		4
	}
}

#[document_examples(skip_call_check)]
///
/// ```
/// let value = helper();
/// assert_eq!(value, 5);
/// ```
fn documented_without_direct_call() -> i32 {
	5
}

fn helper() -> i32 {
	documented_without_direct_call()
}

#[document_module]
#[allow(dead_code)]
mod documented_module {
	use fp_macros::{
		document_examples,
		document_parameters,
		document_returns,
	};

	pub struct ModuleReceiver;

	#[document_parameters("The module receiver.")]
	impl ModuleReceiver {
		#[fp_macros::document_signature]
		#[document_returns("The documented value.")]
		#[document_examples]
		///
		/// ```
		/// let receiver = ModuleReceiver;
		/// assert_eq!(receiver.module_method(), 6);
		/// ```
		pub fn module_method(&self) -> i32 {
			6
		}

		#[fp_macros::document_signature]
		#[document_returns("The indirectly documented value.")]
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// let value = 7;
		/// assert_eq!(value, 7);
		/// ```
		pub fn module_without_direct_call(&self) -> i32 {
			7
		}
	}
}

fn main() {
	let receiver = Receiver;
	assert_eq!(documented_function(), 3);
	assert_eq!(receiver.documented_method(), 4);
	assert_eq!(documented_without_direct_call(), 5);
	assert_eq!(helper(), 5);
}
