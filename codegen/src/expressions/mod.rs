#[cfg(test)] pub mod tests;
pub mod yarn_lit;
pub mod yarn_ops;
pub mod yarn_expr;
mod custom_parser;

use std::fmt::Display;
use std::ops::Deref;
use fmtools::format as format;
use trim_in_place::TrimInPlace;
use yarn_expr::YarnExpr;
use yarn_lit::YarnLit;
use yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use crate::expressions::custom_parser::CustomExpr;
use thiserror::Error;
use anyhow::Result;

type SynExpr = syn::Expr;
type SynError = syn::Error;
type SynBinOp = syn::BinOp;
type SynLit = syn::Lit;
type SynUnaryOp = syn::UnOp;

#[derive(Debug, Clone)]
#[derive(Error)]
pub struct ParseError {
	err: ParseErrorType,
	compiler_line: u32,
	compiler_file: &'static str,
}

impl Display for ParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		return match &self.err {
			ParseErrorType::Syn(syn_err) => write!(f, "Parse error in file: {}.\n\
			At line: {}\n\
			Error: {syn_err}", self.compiler_file, self.compiler_line),
			ParseErrorType::Yarn(yarn_err) => write!(f, "Parse error in file: {}.\n\
			At line: {}\n\
			Error: {yarn_err}", self.compiler_file, self.compiler_line),
		};
	}
}

impl PartialEq for ParseError {
	fn eq(&self, other: &Self) -> bool {
		if self.compiler_line != other.compiler_line
			|| self.compiler_file != other.compiler_file {
			return false;
		}
			
		return match (&self.err, &other.err) {
			(ParseErrorType::Yarn(yarn_err), ParseErrorType::Yarn(other_yarn_err)) => { 
				yarn_err == other_yarn_err
			},
			_ => false,
		};
	}
}


#[derive(Debug, Clone)]
pub enum ParseErrorType {
	Syn(SynError),
	Yarn(String),
}

pub fn parse_expr_from_tokens(token_stream: proc_macro2::TokenStream) -> Result<YarnExpr> {
	return syn::parse2::<CustomExpr>(token_stream)
		.map(|custom| custom.0)
		.map_err(|err| anyhow::anyhow!("Syn error when parsing: {err}"))
		.and_then(|expr| 
			parse_from_syn_expr(expr));
}

fn parse_from_syn_expr(expr: SynExpr) -> Result<YarnExpr> {
	return match expr {
		SynExpr::Binary(binary_expr) => {
			let yarn_op = YarnBinaryOp::try_from_syn(binary_expr.op)?;
			let left = parse_from_syn_expr(*binary_expr.left)?;
			let right = parse_from_syn_expr(*binary_expr.right)?;
			Ok(YarnExpr::BinaryOp {
				yarn_op,
				left: Box::from(left),
				right: Box::from(right),
			})
		},
		SynExpr::Call(call) => {
			let func_name = quote::ToTokens::into_token_stream(call.func).to_string();
			let args = call.args
				.into_iter()
				.map(|arg| parse_from_syn_expr(arg))
				.collect::<Result<Vec<YarnExpr>>>()?;

			Ok(YarnExpr::FunctionCall {
				func_name,
				args,
			})
		},
		SynExpr::Group(group) => {
			let expr = parse_from_syn_expr(*group.expr)?;
			Ok(YarnExpr::Parenthesis(Box::from(expr)))
		},
		SynExpr::Lit(literal) => {
			let lit = YarnLit::try_from_syn(literal.lit)?;
			Ok(YarnExpr::Lit(lit))
		},
		SynExpr::Paren(parenthesized_expr) => {
			let expr = parse_from_syn_expr(*parenthesized_expr.expr)?;
			Ok(YarnExpr::Parenthesis(Box::from(expr)))
		},
		SynExpr::Unary(unary_expr) => {
			fn unwraps_into_int(expr: &Box<YarnExpr>) -> Option<i64> {
				let unwrapped = without_parenthesis(expr);
				return match unwrapped.deref() {
					YarnExpr::Lit(YarnLit::Int(i)) => Some(*i),
					_ => None,
				};

				fn without_parenthesis(expr: &Box<YarnExpr>) -> &Box<YarnExpr> {
					match expr.deref() {
						YarnExpr::Parenthesis(inner) => without_parenthesis(inner),
						_ => expr,
					}
				}
			}
			fn unwraps_into_float(expr: &Box<YarnExpr>) -> Option<f64> {
				let unwrapped = without_parenthesis(expr);
				return match unwrapped.deref() {
					YarnExpr::Lit(YarnLit::Float(i)) => Some(*i),
					_ => None,
				};

				fn without_parenthesis(expr: &Box<YarnExpr>) -> &Box<YarnExpr> {
					match expr.deref() {
						YarnExpr::Parenthesis(inner) => without_parenthesis(inner),
						_ => expr,
					}
				}
			}
			
			let yarn_op = YarnUnaryOp::try_from_syn(unary_expr.op)?;
			let right = Box::from(parse_from_syn_expr(*unary_expr.expr)?);
			
			match yarn_op {
				YarnUnaryOp::Negate => {
					if let Some(int) = unwraps_into_int(&right) {
						Ok(YarnExpr::Lit(YarnLit::Int(-int)))
					} else if let Some(float) = unwraps_into_float(&right) {
						Ok(YarnExpr::Lit(YarnLit::Float(-float)))
					} else {
						Ok(YarnExpr::UnaryOp {
							yarn_op,
							right,
						})
					}
				},
				YarnUnaryOp::Not => {
					Ok(YarnExpr::UnaryOp {
						yarn_op,
						right,
					})
				}
			}
		},
		SynExpr::Array(array) => {
			if array.elems.len() != 1 {
				return Err(ParseError { 
					err: ParseErrorType::Yarn(format!("Array expressions are not allowed: "{array:?})),
					compiler_line: line!(),
					compiler_file: file!(),
				}.into());
			}
			
			let variable_name = {
				let mut temp = quote::ToTokens::to_token_stream(&array.elems[0]).to_string();
				temp.trim_in_place();
				temp
			};

			if variable_name.chars()
				.all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
				Ok(YarnExpr::VarGet(variable_name.to_string()))
			}
			else {
				let error = ParseError {
					err: ParseErrorType::Yarn(format!("Invalid verbatim expression: "{variable_name}"\n\
						The only valid verbatim expression is getting variables (Pattern: `$var_name`).\
						This expression starts with a dollar sign but contains characters that aren't `_` or alphanumeric.")),
					compiler_line: line!(),
					compiler_file: file!(),
				};

				Err(error.into())
			}
		}
		invalid_expr => {
			let ty_msg = match invalid_expr {
				SynExpr::Array(_) => "array",
				SynExpr::Assign(_) => "assign",
				SynExpr::Async(_) => "async",
				SynExpr::Await(_) => "await",
				SynExpr::Block(_) => "block",
				SynExpr::Break(_) => "break",
				SynExpr::Cast(_) => "cast",
				SynExpr::Closure(_) => "closure",
				SynExpr::Const(_) => "const",
				SynExpr::Continue(_) => "continue",
				SynExpr::Field(_) => "field",
				SynExpr::ForLoop(_) => "for loop",
				SynExpr::If(_) => "if",
				SynExpr::Infer(_) => "infer",
				SynExpr::Let(_) => "let",
				SynExpr::Loop(_) => "loop",
				SynExpr::Macro(_) => "macro",
				SynExpr::Match(_) => "match",
				SynExpr::MethodCall(_) => "method call",
				SynExpr::Path(_) => "path",
				SynExpr::Range(_) => "range",
				SynExpr::Reference(_) => "reference",
				SynExpr::Repeat(_) => "repeat",
				SynExpr::Return(_) => "return",
				SynExpr::Struct(_) => "struct",
				SynExpr::Try(_) => "try",
				SynExpr::TryBlock(_) => "try block",
				SynExpr::Tuple(_) => "tuple",
				SynExpr::Unsafe(_) => "unsafe",
				SynExpr::Verbatim(_) => "verbatim",
				SynExpr::While(_) => "while",
				SynExpr::Yield(_) => "compass",
				_ => unreachable!()
			};
			
			let error = ParseError {
				err: ParseErrorType::Yarn(format!("Invalid expression type: "{ty_msg}": "{invalid_expr:?})),
				compiler_line: line!(), 
				compiler_file: file!(), 
			};
			Err(error.into())
		}
	};
}
