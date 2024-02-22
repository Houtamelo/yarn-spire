use fmtools::format;
use crate::expressions::{ParseError, ParseErrorType, SynBinOp, SynUnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YarnUnaryOp {
	Not,
	Negate,
}

impl YarnUnaryOp {
	pub(super) fn resolve(self) -> &'static str {
		return match self {
			YarnUnaryOp::Not => "!",
			YarnUnaryOp::Negate => "-",
		};
	}
	
	pub(super) fn try_from_syn(unary_syn: SynUnaryOp) -> Result<Self, ParseError> {
		return match unary_syn {
			SynUnaryOp::Not(_) => Ok(YarnUnaryOp::Not),
			SynUnaryOp::Neg(_) => Ok(YarnUnaryOp::Negate),
			invalid_op => { 
				let error = ParseError { 
					err: ParseErrorType::Yarn(format!("Invalid unary operator: "{invalid_op:?})),
					compiler_line: line!(), 
					compiler_file: file!(), 
				};
				Err(error)
			},
		};
	}
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YarnBinaryOp {
	Add, Sub,
	Mul,
	Div, Rem,
	And, Or,
	Eq, Ne,
	Lt, Le,
	Gt, Ge,
}

impl YarnBinaryOp {
	pub(super) fn try_from_syn(syn_op: SynBinOp) -> Result<Self, ParseError> {
		return match syn_op {
			SynBinOp::Add(_) => Ok(YarnBinaryOp::Add),
			SynBinOp::Sub(_) => Ok(YarnBinaryOp::Sub),
			SynBinOp::Mul(_) => Ok(YarnBinaryOp::Mul), 
			SynBinOp::Div(_) => Ok(YarnBinaryOp::Div),
			SynBinOp::Rem(_) => Ok(YarnBinaryOp::Rem),
			SynBinOp::And(_) => Ok(YarnBinaryOp::And),
			SynBinOp::Or (_) => Ok(YarnBinaryOp::Or),
			SynBinOp::Eq (_) => Ok(YarnBinaryOp::Eq), 
			SynBinOp::Ne (_) => Ok(YarnBinaryOp::Ne),
			SynBinOp::Lt (_) => Ok(YarnBinaryOp::Lt), 
			SynBinOp::Le (_) => Ok(YarnBinaryOp::Le),
			SynBinOp::Gt (_) => Ok(YarnBinaryOp::Gt), 
			SynBinOp::Ge (_) => Ok(YarnBinaryOp::Ge),
			invalid_op => { 
				let error = ParseError { 
					err: ParseErrorType::Yarn(format!("Invalid binary operator: "{invalid_op:?})),
					compiler_line: line!(), 
					compiler_file: file!(), 
				};
				Err(error)
			},
		};
	}

	pub(super) fn resolve(self) -> &'static str {
		return match self {
			YarnBinaryOp::Add => "+" , YarnBinaryOp::Sub => "-" ,
			YarnBinaryOp::Mul => "*" ,
			YarnBinaryOp::Div => "/" , YarnBinaryOp::Rem => "%" ,
			YarnBinaryOp::And => "&&", YarnBinaryOp::Or  => "||",
			YarnBinaryOp::Eq  => "==", YarnBinaryOp::Ne  => "!=",
			YarnBinaryOp::Lt  => "<" , YarnBinaryOp::Le  => "<=",
			YarnBinaryOp::Gt  => ">" , YarnBinaryOp::Ge  => ">=",
		};
	}
}
