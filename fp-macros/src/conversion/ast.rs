use std::fmt;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum HmAst {
	Variable(String),
	Constructor(String, Vec<HmAst>),
	Arrow(Box<HmAst>, Box<HmAst>),
	Tuple(Vec<HmAst>),
	List(Box<HmAst>), // For [T]
	Reference(Box<HmAst>),
	MutableReference(Box<HmAst>),
	#[default]
	Unit,
	TraitObject(Box<HmAst>),
}

impl fmt::Display for HmAst {
	fn fmt(
		&self,
		f: &mut fmt::Formatter<'_>,
	) -> fmt::Result {
		self.fmt_with_precedence(f, 0)
	}
}

impl HmAst {
	fn precedence(&self) -> u8 {
		match self {
			// Atoms: always safe
			HmAst::Variable(_)
			| HmAst::Unit
			| HmAst::List(_)
			| HmAst::Tuple(_)
			| HmAst::Reference(_)
			| HmAst::MutableReference(_)
			| HmAst::TraitObject(_) => 3,
			// Application: binds tight
			HmAst::Constructor(_, args) => {
				if args.is_empty() {
					3
				} else {
					2
				}
			}
			// Arrow: binds loose
			HmAst::Arrow(_, _) => 1,
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
			HmAst::Variable(name) => write!(f, "{}", name)?,
			HmAst::Unit => write!(f, "()")?,
			HmAst::List(inner) => write!(f, "[{}]", inner)?,
			HmAst::Reference(inner) => {
				write!(f, "&")?;
				inner.fmt_with_precedence(f, 2)?;
			}
			HmAst::MutableReference(inner) => {
				write!(f, "&mut ")?;
				inner.fmt_with_precedence(f, 2)?;
			}
			HmAst::Tuple(args) => {
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
			HmAst::Constructor(name, args) => {
				write!(f, "{}", name)?;
				for arg in args {
					write!(f, " ")?;
					// Function application (Constructor arg)
					// Arguments must be atoms (prec 3) or wrapped
					arg.fmt_with_precedence(f, 3)?;
				}
			}
			HmAst::Arrow(input, output) => {
				// Input needs to be higher precedence than Arrow (1), or wrapped.
				// A -> B -> C parses as A -> (B -> C).
				// So left child needs parens if it is an Arrow.
				input.fmt_with_precedence(f, 2)?;
				write!(f, " -> ")?;
				// Right child is right-associative, so it can be an Arrow without parens.
				output.fmt_with_precedence(f, 1)?;
			}
			HmAst::TraitObject(inner) => {
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
