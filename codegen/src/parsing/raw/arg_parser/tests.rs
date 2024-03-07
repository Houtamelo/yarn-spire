use std::str::FromStr;
use anyhow::anyhow;
use super::*;
use pretty_assertions::{assert_eq, assert_matches};

macro_rules! expect_single {
    ($str: literal) => {
	    let input_stream =
	        TokenStream::from_str($str).unwrap();
	    
	    let tokens =
	        input_stream
	            .into_iter()
	            .collect::<VecDeque<_>>();
	    
	    let iter = ArgsIter { tokens };
	    
	    let results: Vec<_> =
	        iter.into_iter()
	            .try_collect()
	            .unwrap();
	    
	    assert_eq!(results.len(), 1);
	    assert_eq!(results.into_iter().next().unwrap(), expr_unwrap!($str));
    };
}

macro_rules! expect_args {
    ($str: literal == [ $($args: literal),* ]) => {{
	    let input_stream =
	        TokenStream::from_str($str).unwrap();
	    
	    let tokens =
	        input_stream
	            .into_iter()
	            .collect::<VecDeque<_>>();
	    
	    let iter = ArgsIter { tokens };
	    
	    let results: Vec<_> =
	        iter.into_iter()
	            .try_collect()
	            .unwrap();
	    
	    let args = vec![$(expr_unwrap!($args)),*];
	    
	    assert_eq!(args.len(), results.len());
	    for (expected, result) in args.into_iter().zip(results.into_iter()) {
		    assert_eq!(expected, result);
	    }
    }};
}

macro_rules! expr_unwrap {
    ($lit: literal) => {{
		crate::expressions::parse_yarn_expr($lit).unwrap()
    }};
}

#[test]
fn test_ok() {
	expect_single!("5 + 3");
	expect_single!("5 - 3");
	expect_single!("5 * 3");
	expect_single!("(5 / 3)");
	expect_single!("(5 / 3) + 5");
	expect_single!("(5 / 3) - 3.2");
	expect_single!("(5 / 3) / 2");
	expect_single!("(5 / 3) * 5.3");
	expect_single!("(5 / 3) != 3");
	expect_single!("(5 / 3) == 2");
	expect_single!("(5 / 3) >= 5.3");
	expect_single!("(5 / 3) > 2");
	expect_single!("(5 / 3) <= (56)");
	expect_single!("(5 / 3) < (56)");
	expect_single!("$var + 5");
	expect_single!("$var - 8");
	expect_single!("$var * 15");
	expect_single!("$var / 5");
	expect_single!("($var / 5) % ($other)");
	expect_single!("($var / 5) % ($other) + \"literal string\"");
	expect_single!("{$var / 5} % ($other) + 5.3");
	expect_single!("($var / 5) % {$other} + 90");

	expect_single!("$_hyphen_var");
	expect_single!("$___hyphen_var");
	expect_single!("$_5hyphen_var");
	
	expect_args!("5  3" == ["5", "3"]);
	expect_args!("5 + 3  6" == ["5+3", "6"]);
	expect_args!("5 - 3  6" == ["5-3", "6"]);
	expect_args!("5 * 3  6" == ["5*3", "6"]);
	expect_args!("(5 * 3)  {6}" == ["(5*3)", "{6}"]);
	expect_args!("(5 * 3) / {6}" == ["(5*3) / {6}"]);
	expect_args!("(5 * 3) / {6}  $var" == ["(5 * 3) / {6}", "$var"]);
	expect_args!("(5 * 3) / {6}  $var + 7" == ["(5 * 3) / {6}", "$var + 7"]);
	expect_args!("(5 * 3) / {6}  ($var + 7)" == ["(5 * 3) / {6}", "($var + 7)"]);
	expect_args!("(5 * 3) / {6}  {$var + 7} % (5 + 3)" == ["(5 * 3) / {6}", "{$var + 7} % (5 + 3)"]);
	expect_args!("(5 * 3) / {6}  {$var + 7} % (5 + 3) $corr + $corr * 2" == ["(5 * 3) / {6}", "{$var + 7} % (5 + 3)", "$corr + $corr * 2"]);
	expect_args!(r#"set $corruption 5"# == ["set", "$corruption", "5"]);
}

macro_rules! expect_fail {
    ($lit: literal) => {
	    let input_stream =
	        TokenStream::from_str($lit)
	            .map_err(|err| anyhow!("{err}"))
	            .and_then(|stream| { 
		            let tokens = stream.into_iter().collect::<VecDeque<_>>();
		            (ArgsIter{ tokens }).into_iter().try_collect::<Vec<YarnExpr>>()
	            });
	    
	    assert_matches!(input_stream, Err(_));
    };
}

#[test]
fn test_fail() {
	expect_fail!("5 += 2");
	expect_fail!("5 -= 2");
	expect_fail!("5 *= 2");
	expect_fail!("5 /= 2");
	expect_fail!("5 %= 2");

	expect_fail!("(5 += 2) / 2");
	expect_fail!("{5} *= 2 / 2");
	expect_fail!("[5] *= 2 / 2");
	expect_fail!("5 *= [2 / 2]");
	expect_fail!("5 += (2 / 2)");
	expect_fail!("{5 *= (2} / 2)");
	expect_fail!("{5 + 3]");
	
	expect_fail!("$5ash");
	expect_fail!("$-5ash");
	expect_fail!("$-5_ash");
	expect_fail!("$5invalid_var_name");
	expect_fail!("($1231_invalid + 5)");
}