use std::str::FromStr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use super::assert_parse;

#[test]
fn test_literal_int() {
	assert_parse!("5", YarnExpr::Lit(YarnLit::Int(5)));
	assert_parse!("  22", YarnExpr::Lit(YarnLit::Int(22)));
	assert_parse!("53  ", YarnExpr::Lit(YarnLit::Int(53)));
	assert_parse!("  1  ", YarnExpr::Lit(YarnLit::Int(1)));
	assert_parse!("-6", YarnExpr::Lit(YarnLit::Int(-6)));
	assert_parse!("  -11", YarnExpr::Lit(YarnLit::Int(-11)));
	assert_parse!("-23 ", YarnExpr::Lit(YarnLit::Int(-23)));
	assert_parse!("-  35 ", YarnExpr::Lit(YarnLit::Int(-35)));
	assert_parse!(" -28 ", YarnExpr::Lit(YarnLit::Int(-28)));
}

#[test]
fn test_literal_float() {
	assert_parse!("5.0", YarnExpr::Lit(YarnLit::Float(5.0)));
	assert_parse!("  22.0", YarnExpr::Lit(YarnLit::Float(22.0)));
	assert_parse!("53.0  ", YarnExpr::Lit(YarnLit::Float(53.0)));
	assert_parse!("  1.0  ", YarnExpr::Lit(YarnLit::Float(1.0)));
	assert_parse!("-6.0", YarnExpr::Lit(YarnLit::Float(-6.0)));
	assert_parse!("  -11.0", YarnExpr::Lit(YarnLit::Float(-11.0)));
	assert_parse!("-23.0 ", YarnExpr::Lit(YarnLit::Float(-23.0)));
	assert_parse!("-  35.0 ", YarnExpr::Lit(YarnLit::Float(-35.0)));
	assert_parse!(" -28.0 ", YarnExpr::Lit(YarnLit::Float(-28.0)));
}

#[test]
fn test_literal_str() {
	let hello_str = "hello".to_string();
	let token_stream = proc_macro2::TokenStream::from_str("\"hello\"").unwrap();
	let parse_result = crate::expressions::parse_yarn_expr(token_stream);
	match parse_result {
		Ok(ok) => { assert_eq!(ok, YarnExpr::Lit(YarnLit::Str(hello_str))) }
		Err(err) => { panic!("{err}") }
	}

	let phrase_str = "hello, world!".to_string();
	let token_stream = proc_macro2::TokenStream::from_str("\"hello, world!\"").unwrap();
	let parse_result = crate::expressions::parse_yarn_expr(token_stream);
	match parse_result {
		Ok(ok) => { assert_eq!(ok, YarnExpr::Lit(YarnLit::Str(phrase_str))) }
		Err(err) => { panic!("{err}") }
	}
}

#[test]
fn test_literal_bool() {
	assert_parse!("true", YarnExpr::Lit(YarnLit::Bool(true)));
	assert_parse!("  false", YarnExpr::Lit(YarnLit::Bool(false)));
	assert_parse!("  true", YarnExpr::Lit(YarnLit::Bool(true)));
	assert_parse!("true  ", YarnExpr::Lit(YarnLit::Bool(true)));
	assert_parse!("false  ", YarnExpr::Lit(YarnLit::Bool(false)));
	assert_parse!("  true  ", YarnExpr::Lit(YarnLit::Bool(true)));
	assert_parse!("  false  ", YarnExpr::Lit(YarnLit::Bool(false)));
}

