use std::str::FromStr;
use houtamelo_utils::own;
use syn::Expr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use pretty_assertions::{assert_matches, assert_eq};
use proc_macro2::{Delimiter, Group, Ident, Punct, TokenStream};
use quote::{TokenStreamExt, ToTokens};
use expressions::parse_yarn_expr;
use crate::expressions;
use crate::expressions::tests::parse_expect_eq;

#[test]
fn test_nested() {
	parse_expect_eq!("($MyVar)", YarnExpr::Parenthesis(Box::from(YarnExpr::GetVar(own!("MyVar")))));
	parse_expect_eq!("($MyVar + 5)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
			})
		)
	);
	parse_expect_eq!("($MyVar+6)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(6))),
			})
		)
	);
	parse_expect_eq!("-($MyVar+6)",
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
	parse_expect_eq!("($MyVar - 7)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Sub,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(7))),
			})
		)
	);
	parse_expect_eq!("($MyVar * 10)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Mul,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(10))),
			})
		)
	);
	parse_expect_eq!("($MyVar / 12)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Div,
				left: Box::from(YarnExpr::GetVar(own!("MyVar"))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(12))),
			})
		)
	);
	parse_expect_eq!("($MyVar % 15) + (2 * $other_var)", 
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
	parse_expect_eq!("(($MyVar % 15) + (2 * $other_var)) / 5.0", 
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
	parse_expect_eq!("$MyVar", 
		YarnExpr::GetVar(own!("MyVar")));
	parse_expect_eq!("$SlightComplex_var_Name", 
		YarnExpr::GetVar(own!("SlightComplex_var_Name")));
	parse_expect_eq!("$SlightComplex_var_Name_5_num95", 
		YarnExpr::GetVar(own!("SlightComplex_var_Name_5_num95")));
}

