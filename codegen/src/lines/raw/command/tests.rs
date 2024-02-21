use houtamelo_utils::own;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::lines::raw::command::YarnCommand;
use crate::lines::raw::ParseRawYarn;
use crate::expressions::tests::parse_expr;

macro_rules! parse_unwrap {
    ($lit: literal) => {
	    YarnCommand::parse_raw_yarn($lit)
			.unwrap()
			.unwrap()
    };
}

#[test]
fn test_valid_command() {
	assert_eq!(parse_unwrap!("  << command >>"), 
		YarnCommand::Other { 
			line_number: 0,
			variant: own!("command"), 
			args: Vec::new() 
		});
	
	assert_eq!(parse_unwrap!("  << fade 1 2 3 >>"),
		YarnCommand::Other {
			line_number: 0,
			variant: own!("fade"), 
			args: vec![
				YarnExpr::Lit(YarnLit::Int(1)),
				YarnExpr::Lit(YarnLit::Int(2)),
				YarnExpr::Lit(YarnLit::Int(3)),
			]
		});
	
	assert_eq!(parse_unwrap!("  << fade_out 1.2>>"),
		YarnCommand::Other {
			line_number: 0,
			variant: own!("fade_out"), 
			args: vec![
				parse_expr!("1.2").unwrap(),
			]
		});
	
	assert_eq!(parse_unwrap!("  << play_scene \"SceneName\" 3 5.6 >>"),
		YarnCommand::Other {
			line_number: 0,
			variant: own!("play_scene"), 
			args: vec![
				parse_expr!("\"SceneName\"").unwrap(),
				parse_expr!("3").unwrap(),
				parse_expr!("5.6").unwrap(),
			]
		});
	
	assert_eq!(parse_unwrap!("<<increment_var $var_name 1>>"),
		YarnCommand::Other {
			line_number: 0,
			variant: own!("increment_var"), 
			args: vec![
				parse_expr!("$var_name").unwrap(),
				parse_expr!("1").unwrap(),
			]
		});

	assert_eq!(parse_unwrap!("<<set $var_name \"john\">>"),
		YarnCommand::Set {
			line_number: 0,
			var_name: own!("var_name"),
			arg: parse_expr!("\"john\"").unwrap(),
		});
}