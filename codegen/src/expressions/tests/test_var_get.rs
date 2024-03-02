use std::str::FromStr;
use houtamelo_utils::own;
use syn::Expr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use pretty_assertions::assert_matches;
use proc_macro2::{Delimiter, Group, Ident, Punct, TokenStream};
use quote::{TokenStreamExt, ToTokens};

macro_rules! assert_string_parse {
    ($lit: literal, $pattern: expr) => {{
		let parse_result = crate::expressions::parse_yarn_expr($lit);
		match parse_result {
			Ok(ok)   => { pretty_assertions::assert_eq!(ok, $pattern) }
			Err(err) => { panic!("{err}") }
		}
    }};
}

#[test]
fn test_nested() {
	assert_string_parse!("($MyVar)", YarnExpr::Parenthesis(Box::from(YarnExpr::GetVar(own!("MyVar")))));
	assert_string_parse!("($MyVar + 5)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
			})
		)
	);
	assert_string_parse!("($MyVar+6)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(6))),
			})
		)
	);
	assert_string_parse!("-($MyVar+6)",
		YarnExpr::UnaryOp {
			yarn_op: YarnUnaryOp::Negate,
			right: Box::from(
				YarnExpr::Parenthesis(Box::from(
					YarnExpr::BinaryOp {
						yarn_op: YarnBinaryOp::Add,
						left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
						right: Box::from(YarnExpr::Lit(YarnLit::Int(6))),
					})
				)
			)
		}
	);
	assert_string_parse!("($MyVar - 7)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Sub,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(7))),
			})
		)
	);
	assert_string_parse!("($MyVar * 10)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Mul,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(10))),
			})
		)
	);
	assert_string_parse!("($MyVar / 12)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Div,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(12))),
			})
		)
	);
	assert_string_parse!("($MyVar % 15) + (2 * $other_var)", 
		YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Add,
			left: Box::from(YarnExpr::Parenthesis(Box::from(
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Rem,
					left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
					right: Box::from(YarnExpr::Lit(YarnLit::Int(15))),
				})
			)),
			right: Box::from(
				YarnExpr::Parenthesis(Box::from(
					YarnExpr::BinaryOp {
						yarn_op: YarnBinaryOp::Mul,
						left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
						right: Box::from(YarnExpr::GetVar(own!("other_var"))),
					})
				)
			),
		}
	);
	assert_string_parse!("(($MyVar % 15) + (2 * $other_var)) / 5.0", 
		YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Div,
			left: Box::from(YarnExpr::Parenthesis(Box::from(YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::Parenthesis(Box::from(
					YarnExpr::BinaryOp {
						yarn_op: YarnBinaryOp::Rem,
						left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
						right: Box::from(YarnExpr::Lit(YarnLit::Int(15))),
					})
				)),
				right: Box::from(
					YarnExpr::Parenthesis(Box::from(
						YarnExpr::BinaryOp {
							yarn_op: YarnBinaryOp::Mul,
							left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
							right: Box::from(YarnExpr::GetVar(own!("other_var"))),
						})
					)
				),
			}))),
			right: Box::from(YarnExpr::Lit(YarnLit::Float(5.0))),
		}
	);
}

#[test]
fn test() {
	assert_string_parse!("$MyVar", 
		YarnExpr::GetVar(own!("MyVar")));
	assert_string_parse!("$SlightComplex_var_Name", 
		YarnExpr::GetVar(own!("SlightComplex_var_Name")));
	assert_string_parse!("$SlightComplex_var_Name_5_num95", 
		YarnExpr::GetVar(own!("SlightComplex_var_Name_5_num95")));
}
