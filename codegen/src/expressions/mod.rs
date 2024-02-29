#[cfg(test)] pub mod tests;
pub mod yarn_lit;
pub mod yarn_ops;
pub mod yarn_expr;
mod custom_parser;

use custom_parser::CustomExpr;
use std::ops::Deref;
use yarn_expr::YarnExpr;
use yarn_lit::YarnLit;
use yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use anyhow::{Result, anyhow};
use quote::ToTokens;
use syn::Stmt;
use crate::expressions::yarn_expr::DeclarationTy;

type SynExpr = syn::Expr;
type SynError = syn::Error;
type SynBinOp = syn::BinOp;
type SynLit = syn::Lit;
type SynUnaryOp = syn::UnOp;


#[derive(Debug, Clone)]
pub enum ParseErrorType {
	Syn(SynError),
	Yarn(String),
}

pub fn parse_yarn_expr(input: &str) -> Result<YarnExpr> {
	syn::parse_str::<CustomExpr>(input)
		.map(|custom| custom.0)
		.map_err(|err| anyhow!(
			"Could not parse input: {input}\n\
			 Syn Error: {err}"))
		.and_then(|expr|
			parse_from_syn_expr(expr))
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
			let func_name = ToTokens::into_token_stream(call.func).to_string();
			let args =
				call.args
				    .into_iter()
				    .map(parse_from_syn_expr)
				    .try_collect()?;

			Ok(YarnExpr::CustomFunctionCall {
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
				Err(anyhow!("Array expressions are not allowed: {array:?}"))
			} else {
				parse_from_syn_expr(array.elems.into_iter().next().unwrap())
			}
		},
		SynExpr::Block(mut inner) => {
			if inner.block.stmts.len() != 1 {
				Err(anyhow!("Rust code is not allowed: {inner:?}"))
			} else {
				let stmt = inner.block.stmts.remove(0);
				match stmt.clone() {
					Stmt::Expr(expr, punct) => {
						if punct.is_some() {
							Err(anyhow!("Rust code is not allowed: {stmt:?}"))
						} else {
							parse_from_syn_expr(expr)
						}
					}
					| Stmt::Item(_)
					| Stmt::Macro(_)
					| Stmt::Local(_) => {
						Err(anyhow!("Rust code is not allowed: {stmt:?}"))
					}
				}
			}
		},
		SynExpr::Cast(cast) => {
			let expr = parse_from_syn_expr(*cast.expr)?;
			
			let unparsed_ty = *cast.ty;
			let Some(cast_ty) = DeclarationTy::from_syn(unparsed_ty.clone()) 
				else {
					return Err(anyhow!("Invalid cast type: {unparsed_ty:?}"));
				};
			
			Ok(YarnExpr::Cast {
				cast_ty,
				expr: Box::from(expr),
			})
		},
		SynExpr::Verbatim(verbatim) => {
			let verb_str = verbatim.to_string().trim().to_string();
			if verb_str.is_empty() {
				Err(anyhow!("Verbatim expression only contained whitespace."))
			} else if verb_str.chars().any(|c| !c.is_alphanumeric() && c != '_') {
				Err(anyhow!(
					"Could not parse verbatim expression as Identifier.\n\
				     Error: expression contained invalid characters: {verb_str:?}"))
			} else {
				Ok(YarnExpr::Identifier(verb_str))
			}
		},
		SynExpr::Path(path) => {
			if path.qself.is_some() {
				Err(anyhow!("Path expressions are not allowed: {path:?}"))
			} else if let Some(ident) = path.path.get_ident() {
				Ok(YarnExpr::Identifier(ident.to_string()))
			} else {
				Err(anyhow!("Path expressions are not allowed: {path:?}"))
			}
		},
		invalid_expr => {
			let ty_msg = match invalid_expr {
				SynExpr::Array(_) => "array",
				SynExpr::Assign(_) => "assign",
				SynExpr::Async(_) => "async",
				SynExpr::Await(_) => "await",
				SynExpr::Break(_) => "break",
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
				SynExpr::Range(_) => "range",
				SynExpr::Reference(_) => "reference",
				SynExpr::Repeat(_) => "repeat",
				SynExpr::Return(_) => "return",
				SynExpr::Struct(_) => "struct",
				SynExpr::Try(_) => "try",
				SynExpr::TryBlock(_) => "try block",
				SynExpr::Tuple(_) => "tuple",
				SynExpr::Unsafe(_) => "unsafe",
				SynExpr::While(_) => "while",
				SynExpr::Yield(_) => "yield",
				_ => unreachable!()
			};
			
			Err(anyhow!("Invalid expression type: {ty_msg}: {invalid_expr:?}"))
		}
	};
}
