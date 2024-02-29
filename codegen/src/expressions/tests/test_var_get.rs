macro_rules! assert_string_parse {
    ($lit: literal, $pattern: expr) => {{
		let token_stream = proc_macro2::TokenStream::from_str($lit).unwrap();
		let parse_result = crate::expressions::parse_yarn_expr(token_stream);
		match parse_result {
			Ok(ok)   => { assert_eq!(ok, $pattern) }
			Err(err) => { panic!("{err}") }
		}
    }};
}

use std::str::FromStr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};

#[test]
fn test_nested() {
	assert_string_parse!("($MyVar)", YarnExpr::Parenthesis(Box::from(YarnExpr::VarGet("MyVar".to_string()))));
	assert_string_parse!("($MyVar + 5)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
			})
		)
	);
	assert_string_parse!("($MyVar+6)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
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
						left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
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
				left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(7))),
			})
		)
	);
	assert_string_parse!("($MyVar * 10)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Mul,
				left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(10))),
			})
		)
	);
	assert_string_parse!("($MyVar / 12)", 
		YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Div,
				left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
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
					left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
					right: Box::from(YarnExpr::Lit(YarnLit::Int(15))),
				})
			)),
			right: Box::from(
				YarnExpr::Parenthesis(Box::from(
					YarnExpr::BinaryOp {
						yarn_op: YarnBinaryOp::Mul,
						left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
						right: Box::from(YarnExpr::VarGet("other_var".to_string())),
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
						left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
						right: Box::from(YarnExpr::Lit(YarnLit::Int(15))),
					})
				)),
				right: Box::from(
					YarnExpr::Parenthesis(Box::from(
						YarnExpr::BinaryOp {
							yarn_op: YarnBinaryOp::Mul,
							left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
							right: Box::from(YarnExpr::VarGet("other_var".to_string())),
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
	assert_string_parse!("$MyVar", YarnExpr::VarGet("MyVar".to_string()));
	assert_string_parse!("$SlightComplex_var_Name", YarnExpr::VarGet("SlightComplex_var_Name".to_string()));
	assert_string_parse!("$SlightComplex_var_Name_5_num95", YarnExpr::VarGet("SlightComplex_var_Name_5_num95".to_string()));
}