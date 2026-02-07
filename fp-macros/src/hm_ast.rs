use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum HMType {
	Variable(String),
	Constructor(String, Vec<HMType>),
	Arrow(Box<HMType>, Box<HMType>),
	Tuple(Vec<HMType>),
	List(Box<HMType>), // For [T]
	Reference(Box<HMType>),
	MutableReference(Box<HMType>),
	Unit,
	TraitObject(Box<HMType>),
}

impl fmt::Display for HMType {
	fn fmt(
		&self,
		f: &mut fmt::Formatter<'_>,
	) -> fmt::Result {
		self.fmt_with_precedence(f, 0)
	}
}

impl HMType {
	fn precedence(&self) -> u8 {
		match self {
			// Atoms: always safe
			HMType::Variable(_)
			| HMType::Unit
			| HMType::List(_)
			| HMType::Tuple(_)
			| HMType::Reference(_)
			| HMType::MutableReference(_)
			| HMType::TraitObject(_) => 3,
			// Application: binds tight
			HMType::Constructor(_, args) => {
				if args.is_empty() {
					3
				} else {
					2
				}
			}
			// Arrow: binds loose
			HMType::Arrow(_, _) => 1,
		}
	}

	fn fmt_with_precedence(
		&self,
		f: &mut fmt::Formatter<'_>,
		parent_prec: u8,
	) -> fmt::Result {
		let prec = self.precedence();
		let needs_parens = prec < parent_prec;

		if needs_parens {
			write!(f, "(")?;
		}

		match self {
			HMType::Variable(name) => write!(f, "{}", name)?,
			HMType::Unit => write!(f, "()")?,
			HMType::List(inner) => write!(f, "[{}]", inner)?,
			HMType::Reference(inner) => {
				write!(f, "&")?;
				inner.fmt_with_precedence(f, 2)?;
			}
			HMType::MutableReference(inner) => {
				write!(f, "&mut ")?;
				inner.fmt_with_precedence(f, 2)?;
			}
			HMType::Tuple(args) => {
				write!(f, "(")?;
				for (i, arg) in args.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					// Tuples reset precedence for children, as comma is the separator
					arg.fmt_with_precedence(f, 0)?;
				}
				write!(f, ")")?;
			}
			HMType::Constructor(name, args) => {
				write!(f, "{}", name)?;
				for arg in args {
					write!(f, " ")?;
					// Function application (Constructor arg)
					// Arguments must be atoms (prec 3) or wrapped
					arg.fmt_with_precedence(f, 3)?;
				}
			}
			HMType::Arrow(input, output) => {
				// Input needs to be higher precedence than Arrow (1), or wrapped.
				// A -> B -> C parses as A -> (B -> C).
				// So left child needs parens if it is an Arrow.
				input.fmt_with_precedence(f, 2)?;
				write!(f, " -> ")?;
				// Right child is right-associative, so it can be an Arrow without parens.
				output.fmt_with_precedence(f, 1)?;
			}
			HMType::TraitObject(inner) => {
				write!(f, "dyn ")?;
				inner.fmt_with_precedence(f, 3)?;
			}
		}

		if needs_parens {
			write!(f, ")")?;
		}

		Ok(())
	}
}
