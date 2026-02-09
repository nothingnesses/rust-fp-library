use std::fmt;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum HMAST {
	Variable(String),
	Constructor(String, Vec<HMAST>),
	Arrow(Box<HMAST>, Box<HMAST>),
	Tuple(Vec<HMAST>),
	List(Box<HMAST>), // For [T]
	Reference(Box<HMAST>),
	MutableReference(Box<HMAST>),
	#[default]
	Unit,
	TraitObject(Box<HMAST>),
}

impl fmt::Display for HMAST {
	fn fmt(
		&self,
		f: &mut fmt::Formatter<'_>,
	) -> fmt::Result {
		self.fmt_with_precedence(f, 0)
	}
}

impl HMAST {
	fn precedence(&self) -> u8 {
		match self {
			// Atoms: always safe
			HMAST::Variable(_)
			| HMAST::Unit
			| HMAST::List(_)
			| HMAST::Tuple(_)
			| HMAST::Reference(_)
			| HMAST::MutableReference(_)
			| HMAST::TraitObject(_) => 3,
			// Application: binds tight
			HMAST::Constructor(_, args) => {
				if args.is_empty() {
					3
				} else {
					2
				}
			}
			// Arrow: binds loose
			HMAST::Arrow(_, _) => 1,
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
			HMAST::Variable(name) => write!(f, "{}", name)?,
			HMAST::Unit => write!(f, "()")?,
			HMAST::List(inner) => write!(f, "[{}]", inner)?,
			HMAST::Reference(inner) => {
				write!(f, "&")?;
				inner.fmt_with_precedence(f, 2)?;
			}
			HMAST::MutableReference(inner) => {
				write!(f, "&mut ")?;
				inner.fmt_with_precedence(f, 2)?;
			}
			HMAST::Tuple(args) => {
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
			HMAST::Constructor(name, args) => {
				write!(f, "{}", name)?;
				for arg in args {
					write!(f, " ")?;
					// Function application (Constructor arg)
					// Arguments must be atoms (prec 3) or wrapped
					arg.fmt_with_precedence(f, 3)?;
				}
			}
			HMAST::Arrow(input, output) => {
				// Input needs to be higher precedence than Arrow (1), or wrapped.
				// A -> B -> C parses as A -> (B -> C).
				// So left child needs parens if it is an Arrow.
				input.fmt_with_precedence(f, 2)?;
				write!(f, " -> ")?;
				// Right child is right-associative, so it can be an Arrow without parens.
				output.fmt_with_precedence(f, 1)?;
			}
			HMAST::TraitObject(inner) => {
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
