#![allow(unused)]

mod test_literals;
mod test_var_get;
mod test_unary;
mod test_binary;
mod test_function_call;
mod test_resolve;
mod test_english_operators;

macro_rules! parse_expr {
    ($lit: literal) => {{
		crate::expressions::parse_yarn_expr($lit)
    }};
}

macro_rules! assert_parse {
    ($lit: literal, $pattern: expr) => {{
		let parse_result = crate::expressions::tests::parse_lit!($lit);
		match parse_result {
			Ok(ok)   => { pretty_assertions::assert_eq!(ok, $pattern) }
			Err(err) => { panic!("{err}") }
		}
    }};
}

macro_rules! parse_expect_eq {
    ($lit: literal, $pattern: expr) => {{
		let parse_result = parse_yarn_expr($lit);
		match parse_result {
			Ok(ok)   => { pretty_assertions::assert_eq!(ok, $pattern) }
			Err(err) => { panic!("{err}") }
		}
    }};
}

macro_rules! parse_unwrap {
    ($lit: expr) => {
	    match parse_yarn_expr($lit) {
			Ok(ok)   => { ok }
			Err(err) => { panic!("{err}") }
		}
    };
}

macro_rules! parse_both_expect_eq {
	($left: expr, $right: expr) => {{
		let left_expr = parse_unwrap!($left);
		let right_expr = parse_unwrap!($right);
		assert_eq!(left_expr, right_expr);
    }};
}

pub(crate) use {parse_expr, assert_parse, parse_expect_eq, parse_unwrap, parse_both_expect_eq};
