use crate::expressions::SynLit;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub enum YarnLit {
	Int(i64),
	Float(f64),
	Str(String),
	Bool(bool),
}

impl YarnLit {
	pub(super) fn resolve(self) -> String {
		return match self {
			YarnLit::Int(i) => i.to_string(),
			YarnLit::Float(f) => f.to_string(),
			YarnLit::Str(s) => format!("\"{s}\""),
			YarnLit::Bool(b) => b.to_string(),
		};
	}

	pub(super) fn try_from_syn(syn_lit: SynLit) -> Result<Self> {
		return match syn_lit {
			SynLit::Str (str_lit  ) => Ok(YarnLit::Str(str_lit.value())),
			SynLit::Bool(bool_lit) => Ok(YarnLit::Bool(bool_lit.value)),
			SynLit::Int(int_lit) => match int_lit.base10_parse() {
				Ok(int) => Ok(YarnLit::Int(int)),
				Err(error) => {
					Err(anyhow!("Invalid integer literal: {error}"))
				}
			},
			SynLit::Float(float_lit) => match float_lit.base10_parse() {
				Ok(float) => Ok(YarnLit::Float(float)),
				Err(error) => {
					Err(anyhow!("Invalid float literal: {error}"))
				}
			},
			invalid_lit => {
				Err(anyhow!("Invalid literal: {invalid_lit:?}"))
			},
		};
	}
}

impl PartialEq for YarnLit {
	fn eq(&self, other: &Self) -> bool {
		return match (self, other) {
			(YarnLit::Int(i1), YarnLit::Int(i2)) => i1 == i2,
			(YarnLit::Float(f1), YarnLit::Float(f2)) => f64::abs(f1 - f2) <= 0.000001,
			(YarnLit::Str(s1), YarnLit::Str(s2)) => s1 == s2,
			(YarnLit::Bool(b1), YarnLit::Bool(b2)) => b1 == b2,
			_ => false,
		};
	}
}

impl Eq for YarnLit {}
