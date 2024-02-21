use fmtools::format;
use crate::expressions::{ParseError, ParseErrorType, SynLit};

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
			YarnLit::Str(s) => format!("\""{s}"\""),
			YarnLit::Bool(b) => b.to_string(),
		};
	}

	pub(super) fn try_from_syn(syn_lit: SynLit) -> Result<Self, ParseError> {
		return match syn_lit {
			SynLit::Str (str_lit  ) => Ok(YarnLit::Str(str_lit.value())),
			SynLit::Bool(bool_lit) => Ok(YarnLit::Bool(bool_lit.value)),
			SynLit::Int(int_lit) => match int_lit.base10_parse() {
				Ok(int) => Ok(YarnLit::Int(int)),
				Err(error) => {
					let error = ParseError { 
						err: ParseErrorType::Yarn(format!("Invalid integer literal: "{error})),
						compiler_line: line!(), 
						compiler_file: file!(), 
					};
					Err(error)
				}
			},
			SynLit::Float(float_lit) => match float_lit.base10_parse() {
				Ok(float) => Ok(YarnLit::Float(float)),
				Err(error) => {
					let error = ParseError { 
						err: ParseErrorType::Yarn(format!("Invalid float literal: "{error})),
						compiler_line: line!(), 
						compiler_file: file!(), 
					};
					Err(error)
				}
			},
			invalid_lit => { 
				let error = ParseError { 
					err: ParseErrorType::Yarn(format!("Invalid literal: "{invalid_lit:?})),
					compiler_line: line!(), 
					compiler_file: file!(), 
				};
				Err(error)
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
