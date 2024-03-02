#![allow(unused)]

mod test_literals;
mod test_var_get;
mod test_unary;
mod test_binary;
mod test_function_call;
mod test_resolve;

macro_rules! parse_expr {
    ($lit: literal) => {{
	    let token_stream = proc_macro2::TokenStream::from_str($lit).unwrap();
		crate::expressions::parse_yarn_expr(token_stream)
    }};
}

macro_rules! assert_parse {
    ($lit: literal, $pattern: expr) => {{
		let parse_result = crate::expressions::tests::parse_lit!($lit);
		match parse_result {
			Ok(ok)   => { assert_eq!(ok, $pattern) }
			Err(err) => { panic!("{err}") }
		}
    }};
}

pub(crate) use parse_expr;
pub(crate) use assert_parse;
