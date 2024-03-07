use houtamelo_utils::own;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::YarnBinaryOp;
use crate::parsing::raw::command::{CommandVariant, SetOperation, YarnCommand};
use crate::parsing::raw::{Content, ParseRawYarn};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_matches};macro_rules! parse {
    ($lit: literal) => {
	    YarnCommand::parse_raw_yarn($lit, 0)
    };
}

macro_rules! parse_unwrap {
    ($lit: literal) => {
	    YarnCommand::parse_raw_yarn($lit, 0)
			.unwrap()
			.unwrap()
    };
}

macro_rules! arg {
    ($lit: literal) => {{
		crate::expressions::parse_yarn_expr($lit).unwrap()
    }};
}

macro_rules! set_cmd {
    ($var_name: literal, $arg: expr, $op: expr $(,)?) => {
	    Content::Command(YarnCommand { 
			line_number: 0,
			variant: CommandVariant::Set {
				var_name: own!($var_name), 
				value: $arg,
				op: $op, 
			}
		})
    };
}

macro_rules! jump_cmd {
	($node: literal) => {
		Content::Command(YarnCommand { 
			line_number: 0,
			variant: CommandVariant::Jump {
				node_name: own!($node),
			}
		})
	};
}

macro_rules! stop_cmd {
    () => {
	    Content::Command(YarnCommand {
			line_number: 0,
			variant: CommandVariant::Stop,
		})
    };
}

macro_rules! other_cmd {
    ($name: literal, $args: expr) => {
	    Content::Command(YarnCommand { 
			line_number: 0,
			variant: CommandVariant::Other {
				variant: own!($name),
				args: $args,
			}
		})
    };
}

#[test]
fn test_valid_command() {
	assert_eq!(
		parse_unwrap!("  << command >>"), 
		other_cmd! { "command", vec![] });
	
	assert_eq!(
		parse_unwrap!("<<fade_in_async -1>>"),
		other_cmd! { "fade_in_async", vec![arg!("-1")] });
	
	assert_eq!(parse_unwrap!("  << fade 1 2 3 >>"),
		other_cmd! { "fade", 
			vec![
				YarnExpr::Lit(YarnLit::Int(1)),
				YarnExpr::Lit(YarnLit::Int(2)),
				YarnExpr::Lit(YarnLit::Int(3)),
			]
		});
	
	assert_eq!(parse_unwrap!("  << fade_out 1.2>>"),
		other_cmd! {
			"fade_out", 
			vec![
				arg!("1.2"),
			]
		});
	
	assert_eq!(parse_unwrap!("  << play_scene \"SceneName\" 3 5.6 >>"),
		other_cmd! {
			"play_scene", 
			vec![
				arg!("\"SceneName\""),
				arg!("3"),
				arg!("5.6"),
			]
		});
	
	assert_eq!(parse_unwrap!("<<increment_var $var_name 1>>"),
		other_cmd! {
			"increment_var", 
			vec![
				arg!("$var_name"),
				arg!("1"),
			]
		});
	
	assert_matches!(parse!("<<set $var_name \"john\""), Some(Err(_)));

	assert_eq!(parse_unwrap!("<<set $var_name to \"john\">>"),
		set_cmd! {
			"var_name",
			arg!("\"john\""),
			SetOperation::Assign,
		});
	
	assert_eq!(parse_unwrap!("<<set $ethel_Corruption = $ethel_Corruption + 2>>"), 
		set_cmd! {
			"ethel_Corruption",
			YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::new(arg!("$ethel_Corruption")),
				right: Box::new(arg!("2")),
			},
			SetOperation::Assign,
		});

	assert_eq!(parse_unwrap!("<<set $ethel_Corruption += 2>>"),
		set_cmd! {
			"ethel_Corruption",
			arg!("2"),
			SetOperation::Add,
		});


	assert_eq!(parse_unwrap!("<<set $ethel_Corruption -= 2>>"),
		set_cmd! {
			"ethel_Corruption",
			arg!("2"),
			SetOperation::Sub,
		});


	assert_eq!(parse_unwrap!("<<set $ethel_Corruption *= 3>>"),
		set_cmd! {
			"ethel_Corruption",
			arg!("3"),
			SetOperation::Mul,
		});

	assert_eq!(parse_unwrap!("<<set $ethel_Corruption /= 4>>"),
		set_cmd! {
			"ethel_Corruption",
			arg!("4"),
			SetOperation::Div,
		});

	assert_eq!(parse_unwrap!("<<set $ethel_Corruption %= 2  >>"),
		set_cmd! {
			"ethel_Corruption",
			arg!("2"),
			SetOperation::Rem,
		});
	
	assert_eq!(parse_unwrap!("<<jump \"NodeName\">>"), jump_cmd! { "NodeName" });
	assert_eq!(parse_unwrap!("<<jump NodeName>>"), jump_cmd! { "NodeName" });
	assert_eq!(parse_unwrap!("<<jump otherName>>"), jump_cmd! { "otherName" });
	assert_eq!(parse_unwrap!("<<jump _otherName>>"), jump_cmd! { "_otherName" });
	assert_eq!(parse_unwrap!("<<jump _5otherName6>>"), jump_cmd! { "_5otherName6" });
	
	assert_eq!(parse_unwrap!("<<stop>>"), stop_cmd!());
	assert_eq!(parse_unwrap!("<<stop  >>"), stop_cmd!());
	assert_eq!(parse_unwrap!("<<   stop>>"), stop_cmd!());
	assert_eq!(parse_unwrap!("<<   stop   >>"), stop_cmd!());
	assert_eq!(parse_unwrap!("<<   stop\t>>"), stop_cmd!());
}
