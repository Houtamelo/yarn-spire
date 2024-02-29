use std::assert_matches::assert_matches;
use crate::expressions::tests::parse_expr;
use crate::parsing::raw::branches::if_statement::{BranchKind, ElseIfStruct, ElseStruct, EndIfStruct, IfStruct};
use std::str::FromStr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::raw::{ParseRawYarn, Content};
use crate::parsing::raw::branches::choices::OptionLine;

macro_rules! parse_unwrap {
    ($lit: literal) => {
	    BranchKind::parse_raw_yarn($lit, 0)
			.unwrap()
			.unwrap()
    };
}

macro_rules! parse {
    ($lit: literal) => {
	    BranchKind::parse_raw_yarn($lit, 0)
    };
}

#[test]
fn test_std() {
	assert_eq!(parse_unwrap!("<< if true >>"), 
		Content::If(IfStruct { line_number: 0, condition: parse_expr!("true").unwrap() }));
	assert_eq!(parse_unwrap!("<< elseif 5 > 3 >>"),
		Content::ElseIf(ElseIfStruct { line_number: 0, condition: parse_expr!("5 > 3").unwrap() }));
	assert_eq!(parse_unwrap!("<<   else>>"), Content::Else(ElseStruct { line_number: 0 }));
	assert_eq!(parse_unwrap!("<<endif >>"), Content::EndIf(EndIfStruct { line_number: 0 }));
}

#[test]
fn test_tabs() {
	assert_eq!(parse_unwrap!("<<\t if true \t>>"),
		Content::If(IfStruct { line_number: 0, condition: parse_expr!("true").unwrap() }));
	assert_eq!(parse_unwrap!("<< elseif 5\t > 3 >>"),
		Content::ElseIf(ElseIfStruct { line_number: 0, condition: parse_expr!("5 > 3").unwrap() }));
	assert_eq!(parse_unwrap!("<< else\t >>"), Content::Else(ElseStruct { line_number: 0 }));
	assert_eq!(parse_unwrap!("<<\t endif\t>>"), Content::EndIf(EndIfStruct { line_number: 0 }));
}

#[test]
fn test_none() {
	assert_matches!(parse!("if condition"), None);
	assert_matches!(parse!("<< condition >>"), None);
}

#[test]
fn test_invalid_identifier() {
	assert_matches!(parse!("<<if_  condition >>").unwrap(), Err(_));
	assert_matches!(parse!("<<\telseif  _condition >>").unwrap(), Err(_));
	assert_matches!(parse!("<<else_condition >>").unwrap(), Err(_));
	assert_matches!(parse!("<<  endif_condition >>").unwrap(), Err(_));
}

#[test]
fn test_invalid_missing_double_end() {
	assert_matches!(parse!("<<if {5 + 3 / 2}").unwrap(), Err(_));
	assert_matches!(parse!("<<\telseif true").unwrap(), Err(_));
	assert_matches!(parse!("<<else").unwrap(), Err(_));
	assert_matches!(parse!("<<  endif\t").unwrap(), Err(_));
}

#[test]
fn test_invalid_has_metadata() {
	assert_matches!(parse!("<<if true>> #metadata").unwrap(), Err(_));
	assert_matches!(parse!("<<elseif 5 > 3>> # other metadata # with more tags").unwrap(), Err(_));
	assert_matches!(parse!("<<else>> #data here").unwrap(), Err(_));
	assert_matches!(parse!("<<endif>> # not allowed after conditionals").unwrap(), Err(_));
}

#[test]
fn test_complex_conditions() {
	assert_eq!(parse_unwrap!("<< if (5 + 3) > (2 * 5) >>"),
		IfStruct { line_number: 0, condition: parse_expr!("(5 + 3) > (2 * 5)").unwrap() });
	assert_eq!(parse_unwrap!("<< elseif (5 + 3) > (2 * 5) >>"),
		ElseIfStruct { line_number: 0, condition: parse_expr!("(5 + 3) > (2 * 5)").unwrap() });
	assert_eq!(parse_unwrap!("<<if $affection > 5>>"),
		IfStruct { line_number: 0, condition: parse_expr!("$affection > 5").unwrap() });
	assert_eq!(parse_unwrap!("<<elseif $affection < 10>>"),
		ElseIfStruct { line_number: 0, condition: parse_expr!("$affection < 10").unwrap() });
	assert_eq!(parse_unwrap!("<<if (($gold / 2) + ($silver * 3)) / 10 % 2 == 2  >>"),
		IfStruct { line_number: 0, condition: parse_expr!("(($gold / 2) + ($silver * 3)) / 10 % 2 == 2").unwrap() });
	assert_eq!(parse_unwrap!("<<if ((gold() / 2) + (random(3, 5) * 3)) / 10 % silver() == 2  >>"),
		IfStruct { line_number: 0, condition: parse_expr!("((gold() / 2) + (random(3, 5) * 3)) / 10 % silver() == 2").unwrap() });
	assert_eq!(parse_unwrap!("\t<<\tif\t(5\t+\t3)\t>\t(2\t*\t5)\t>>"),
		IfStruct { line_number: 0, condition: parse_expr!("(5 + 3) > (2 * 5)").unwrap() });
}

#[test]
fn test_string_conditions() {
	assert_eq!(parse_unwrap!(r##"<<if $player_name == "Alice">> #metadata"##),
		IfStruct { line_number: 0, condition: parse_expr!(r#"$player_name == "Alice""#).unwrap() });
	assert_eq!(parse_unwrap!(r##"<<elseif $player_name == "Bob">> #metadata: some_stuff, #tag: other_stuff"##),
		ElseIfStruct { line_number: 0, condition: parse_expr!(r#"$player_name == "Bob""#).unwrap() });
}

#[test]
fn test_not_confused_with_commands() {
	assert_matches!(parse!("<<set $some_var 5.3>>"), None);
	assert_matches!(parse!("<<  set $hey_var = \"StringLiteral\">>"), None);
	assert_matches!(parse!("<<no_arg_command>>"), None);
	assert_matches!(parse!("<<command>>"), None);
	assert_matches!(parse!("<<triple_arg_command 2 \"I LoveLiterals \" 3.>>"), None);
}

#[test]
fn test_not_confused_with_options() {
	assert_matches!(parse!("-> hello"), None);
	assert_matches!(parse!("-> hello # metadata   "), None);
	assert_matches!(parse!("->Another option but big"), None);
	assert_matches!(parse!("->Another option but with condition: <<if $player_awake>>"), None);
}

#[test]
fn test_no_if() {
	use houtamelo_utils::own;

	assert_eq!(OptionLine::parse_raw_yarn("-> hello", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: None,
			metadata: None,
		});

	assert_eq!(OptionLine::parse_raw_yarn("-> hello # metadata", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: None,
			metadata: Some(own!("metadata")),
		});

	assert_eq!(OptionLine::parse_raw_yarn("-> hello# test data", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: None,
			metadata: Some(own!("test data")),
		});

	assert_eq!(OptionLine::parse_raw_yarn("->Very big choice my dude \"not metadata or \\#\\# tags\"", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("Very big choice my dude \"not metadata or ## tags\""), vec![]),
			if_condition: None,
			metadata: None,
		});

	assert_eq!(OptionLine::parse_raw_yarn("->Very big choice my dude \"not metadata ## tags\" # actual metadata here, cant escape this #", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("Very big choice my dude \"not metadata ## tags\""), vec![]),
			if_condition: None,
			metadata: Some(own!("actual metadata here, cant escape this #")),
		});
}

#[test]
fn test_with_if() {
	use houtamelo_utils::own;
	use crate::expressions::yarn_lit::YarnLit;
	use crate::expressions::yarn_ops::YarnBinaryOp;

	assert_eq!(OptionLine::parse_raw_yarn("-> <<if $condition>> hello", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: Some(YarnExpr::VarGet(own!("condition"))),
			metadata: None,
		});

	assert_eq!(OptionLine::parse_raw_yarn("-> <<if $condition>> hello # metadata", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: Some(YarnExpr::VarGet(own!("condition"))),
			metadata: Some(own!("metadata")),
		});

	assert_eq!(OptionLine::parse_raw_yarn("-> <<if $condition>> hello # metadata # more metadata", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: Some(YarnExpr::VarGet(own!("condition"))),
			metadata: Some(own!("metadata # more metadata")),
		});

	assert_eq!(OptionLine::parse_raw_yarn("-> <<if 5 > 3>> hello # metadata # more metadata", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: Some(YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Gt,
				left: Box::new(YarnExpr::Lit(YarnLit::Int(5))),
				right: Box::new(YarnExpr::Lit(YarnLit::Int(3))),
			}),
			metadata: Some(own!("metadata # more metadata")),
		});

	// nested

	assert_eq!(OptionLine::parse_raw_yarn("-> <<if {(5 > 3) / 3 + 2}>> hello # metadata # more metadata", 0).unwrap(),
		OptionLine {
			line_number: 0,
			text: (own!("hello"), vec![]),
			if_condition: Some(YarnExpr::BinaryOp {
				yarn_op: YarnBinaryOp::Add,
				left: Box::new(YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Div,
					left: Box::new(YarnExpr::BinaryOp {
						yarn_op: YarnBinaryOp::Gt,
						left: Box::new(YarnExpr::Lit(YarnLit::Int(5))),
						right: Box::new(YarnExpr::Lit(YarnLit::Int(3))),
					}),
					right: Box::new(YarnExpr::Lit(YarnLit::Int(3))),
				}),
				right: Box::new(YarnExpr::Lit(YarnLit::Int(2))),
			}),
			metadata: Some(own!("metadata # more metadata")),
		});

}

#[test]
fn test_invalid() {
	use std::assert_matches::assert_matches;

	assert_matches!(OptionLine::parse_raw_yarn("-> <<if $condition>>", 0).unwrap(),
		Err(_));

	assert_matches!(OptionLine::parse_raw_yarn("-> <<if $condition>> # metadata", 0).unwrap(),
		Err(_));

	assert_matches!(OptionLine::parse_raw_yarn("-> Hey there <<if $condition # metadata", 0).unwrap(),
		Err(_));

	assert_matches!(OptionLine::parse_raw_yarn("-> Hey there <<if $condition>> metadata", 0).unwrap(),
		Err(_));

	assert_matches!(OptionLine::parse_raw_yarn("-> Hey there <<if >> #metadata", 0).unwrap(),
		Err(_));

	assert_matches!(OptionLine::parse_raw_yarn("-> Hey there <<$condition>> metadata", 0).unwrap(),
		Err(_));

	assert_matches!(OptionLine::parse_raw_yarn("-> Hey there <<not_if $condition>> #metadata", 0).unwrap(),
		Err(_));
}
