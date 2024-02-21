use std::str::FromStr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::YarnBinaryOp;
use super::parse_expr;

#[test]
fn test() {
	let parse_result = parse_expr!("$my_var").unwrap();
	assert_eq!(parse_result, YarnExpr::VarGet("my_var".to_string()));
	assert_eq!(parse_result.resolve(), "controller.get::<my_var>()");
	
	let parse_result = parse_expr!("$my_var + 5").unwrap();
	assert_eq!(parse_result, YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Add,
		left: Box::from(YarnExpr::VarGet("my_var".to_string())),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
	});
	assert_eq!(parse_result.resolve(), "controller.get::<my_var>() + 5");
	
	let parse_result = parse_expr!("($my_var - 5) / 8").unwrap();
	assert_eq!(parse_result, YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Div,
		left: Box::from(YarnExpr::Parenthesis(Box::from(YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Sub,
			left: Box::from(YarnExpr::VarGet("my_var".to_string())),
			right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
		}))),
		right: Box::from(YarnExpr::Lit(YarnLit::Int(8))),
	});
	assert_eq!(parse_result.resolve(), "(controller.get::<my_var>() - 5) / 8");
	
	let parse_result = parse_expr!("($my_var - 5) / ($my_var + 8)").unwrap();
	assert_eq!(parse_result, YarnExpr::BinaryOp {
		yarn_op: YarnBinaryOp::Div,
		left: Box::from(YarnExpr::Parenthesis(Box::from(YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Sub,
			left: Box::from(YarnExpr::VarGet("my_var".to_string())),
			right: Box::from(YarnExpr::Lit(YarnLit::Int(5))),
		}))),
		right: Box::from(YarnExpr::Parenthesis(Box::from(YarnExpr::BinaryOp {
			yarn_op: YarnBinaryOp::Add,
			left: Box::from(YarnExpr::VarGet("my_var".to_string())),
			right: Box::from(YarnExpr::Lit(YarnLit::Int(8))),
		}))),
	});
	assert_eq!(parse_result.resolve(), "(controller.get::<my_var>() - 5) / (controller.get::<my_var>() + 8)");
}