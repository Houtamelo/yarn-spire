use std::str::FromStr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use super::assert_parse;

#[test]
fn test() {
	assert_parse!("5 + 2", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Add,
		left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
	});
	assert_parse!("3 - 7", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Sub,
		left: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(7))),
	});
	assert_parse!("5 * 2", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Mul,
		left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
	});
	assert_parse!("6 / 3", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Div,
		left: Box::from(YarnExpr::Lit(YarnLit::Int(6))),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
	});
	assert_parse!("5 % 2", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Rem,
		left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
	});
	assert_parse!("5 + 2 * 3", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Add,
		left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
		right: Box::from(YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Mul,
			left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
			right: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
		}),
	});
	assert_parse!("(5 + 2) * 3", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Mul,
		left: Box::from(YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
			})
		)),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
	});
	assert_parse!("5 * (2 + 3)", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Mul,
		left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
		right: Box::from(YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
				right: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
			})
		)),
	});
	assert_parse!("5 * (2 + 3) * 4", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Mul,
		left: Box::from(YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Mul,
			left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
			right: Box::from(YarnExpr::Parenthesis(Box::from(
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Add,
					left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
					right: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
				})
			)),
		}),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(4))),
	});
	assert_parse!("-(5 * (2 + 3)) / 5", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Div,
		left: Box::from(YarnExpr::UnaryOp {
			yarn_op: YarnUnaryOp::Negate,
			right: Box::from(YarnExpr::Parenthesis(Box::from(
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Mul,
					left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
					right: Box::from(YarnExpr::Parenthesis(Box::from(
						YarnExpr::BinaryOp {
							yarn_op: YarnBinaryOp::Add,
							left: Box::from(YarnExpr::Lit(YarnLit::Int(2))),
							right: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
						})
					)),
				})
			)),
		}),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
	});
	assert_parse!("-(5 * (2.2 + 3.5)) / 5.8", YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Div,
		left: Box::from(YarnExpr::UnaryOp {
			yarn_op: YarnUnaryOp::Negate,
			right: Box::from(YarnExpr::Parenthesis(Box::from(
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Mul,
					left: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
					right: Box::from(YarnExpr::Parenthesis(Box::from(
						YarnExpr::BinaryOp {
							yarn_op: YarnBinaryOp::Add,
							left: Box::from(YarnExpr::Lit(YarnLit::Float(2.2))),
							right: Box::from(YarnExpr::Lit(YarnLit::Float(3.5))),
						})
					)),
				})
			)),
		}),
		right: Box::from(YarnExpr::Lit(YarnLit::Float(5.8))),
	});
}

#[test]
fn test_string_eq() {
	assert_parse!(r#"$player_name == "Alice""#, 
		YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Eq,
			left: Box::from(YarnExpr::VarGet("player_name".to_owned())),
			right: Box::from(YarnExpr::Lit(YarnLit::Str("Alice".to_string()))),
		});
	
	assert_parse!(r#"$player_name == "Bob""#,
		YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Eq,
			left: Box::from(YarnExpr::VarGet("player_name".to_owned())),
			right: Box::from(YarnExpr::Lit(YarnLit::Str("Bob".to_string()))),
		});
}