use crate::expressions::SynLit;
use anyhow::{Result, anyhow};
use genco::lang::Rust;
use genco::prelude::{FormatInto, quoted};
use genco::{quote_in, Tokens};

#[derive(Debug, Clone)]
pub enum YarnLit {
	Int(i64),
	Float(f64),
	Str(String),
	Bool(bool),
}

impl YarnLit {
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

impl FormatInto<Rust> for &YarnLit {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		match self {
			YarnLit::Int(i) => {
				quote_in!(*tokens => $(*i));
			},
			YarnLit::Float(f) => {
				fn count_decimal_cases(input: &f64) -> usize {
					let mut count = 0;
					let mut current = *input;

					while current.fract() != 0.0 && count < 6 {
						current *= 10.0;
						count += 1;
					}

					return usize::max(count, 1);
				}
				
				let cases = count_decimal_cases(f);
				quote_in!(*tokens => $(format!("{f:.0$}", cases)));
			},
			YarnLit::Str(s) => {
				quote_in!(*tokens => $(quoted(s)));
			},
			YarnLit::Bool(b) => {
				match b {
					true => {
						quote_in!(*tokens => true);
					}
					false => {
						quote_in!(*tokens => false);
					}
				}
			}
		}
	}
}

impl Eq for YarnLit {}