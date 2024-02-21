use std::str::FromStr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use super::assert_parse;

#[test]
fn test() {
	assert_parse!("fade_in(5.0)", YarnExpr::FunctionCall {
		func_name: "fade_in".to_string(),
		args: vec![YarnExpr::Lit(YarnLit::Float(5.0))],
	});
	assert_parse!("fade_out(3)", YarnExpr::FunctionCall {
		func_name: "fade_out".to_string(),
		args: vec![YarnExpr::Lit(YarnLit::Int(3))],
	});
	assert_parse!("get_random()", YarnExpr::FunctionCall {
		func_name: "get_random".to_string(),
		args: vec![],
	});
	assert_parse!("get_range(5, 2)", YarnExpr::FunctionCall {
		func_name: "get_range".to_string(),
		args: vec![YarnExpr::Lit(YarnLit::Int(5)), YarnExpr::Lit(YarnLit::Int(2))],
	});
	assert_parse!("set_name(\"houtamelo\")", YarnExpr::FunctionCall {
		func_name: "set_name".to_string(),
		args: vec![YarnExpr::Lit(YarnLit::Str("houtamelo".to_string()))],
	});
	assert_parse!("set_name(\"houtamelo\", $player_name)", YarnExpr::FunctionCall {
		func_name: "set_name".to_string(),
		args: vec![YarnExpr::Lit(YarnLit::Str("houtamelo".to_string())), YarnExpr::VarGet("player_name".to_string())],
	});
}