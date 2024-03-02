/*

use std::str::FromStr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use super::assert_parse;


#[test]
fn test() {
	assert_parse!("-(  5  + 3)", YarnExpr::UnaryOp {
		yarn_op: YarnUnaryOp::Negate,
		right: Box::from(YarnExpr::Parenthesis(Box::from(
			YarnExpr::BinaryOp { 
				yarn_op: YarnBinaryOp::Add, 
				left: Box::from(YarnExpr::Lit(YarnLit::Int(5))), 
				right: Box::from(YarnExpr::Lit(YarnLit::Int(3))),
			})
		)),
	});
	assert_parse!("!true", YarnExpr::UnaryOp {
		yarn_op: YarnUnaryOp::Not,
		right: Box::from(YarnExpr::Lit(YarnLit::Bool(true))),
	});
	assert_parse!("!false", YarnExpr::UnaryOp {
		yarn_op: YarnUnaryOp::Not,
		right: Box::from(YarnExpr::Lit(YarnLit::Bool(false))),
	});
	assert_parse!("!($MyVar)", YarnExpr::UnaryOp {
		yarn_op: YarnUnaryOp::Not,
		right: Box::from(YarnExpr::Parenthesis(Box::from(YarnExpr::VarGet("MyVar".to_string())))),
	});
	assert_parse!("!($MyVar + 5)", YarnExpr::UnaryOp {
		yarn_op: YarnUnaryOp::Not,
		right: Box::from(
			YarnExpr::Parenthesis(Box::from(
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Add,
					left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
					right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
				})
			)
		)
	});
	assert_parse!("!($MyVar+6)", YarnExpr::UnaryOp {
		yarn_op: YarnUnaryOp::Not,
		right: Box::from(
			YarnExpr::Parenthesis(Box::from(
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Add,
					left: Box::from(YarnExpr::VarGet("MyVar".to_string())),
					right: Box::from(YarnExpr::Lit(YarnLit::Int(6))),
				})
			)
		)
	});
}
*/