use crate::parsing::raw::branches::if_statement::{BranchKind, ElseIf_, Else_, EndIf_, If_};
use crate::parsing::raw::{Content, ParseRawYarn};
use pretty_assertions::{assert_eq, assert_matches};

macro_rules! expr {
    ($lit: literal) => {
	    crate::expressions::parse_yarn_expr($lit).unwrap()
    };
}

macro_rules! parse_unwrap {
    ($lit: literal) => {
        BranchKind::parse_raw_yarn($lit, 0).unwrap().unwrap()
    };
}

macro_rules! parse {
    ($lit: literal) => {
	    BranchKind::parse_raw_yarn($lit, 0)
    };
}

macro_rules! expect_err {
    ($lit: literal) => {
	    assert_matches!(parse!($lit), Some(Err(_)));
    };
}

macro_rules! branch {
    (if $condition: literal) => {
	    Content::If(If_ { line_number: 0, condition: expr!($condition) })
    };
	(else if $condition: literal) => {
	    Content::ElseIf(ElseIf_ { line_number: 0, condition: expr!($condition) })
	};
	(else) => {
	    Content::Else(Else_ { line_number: 0 })
	};
	(end if) => {
	    Content::EndIf(EndIf_ { line_number: 0 })
	};
}

#[test]
fn test_std() {
	assert_eq!(
		parse_unwrap!("<< if true >>"), 
		branch!{ if "true" });
	assert_eq!(
		parse_unwrap!("<< elseif 5 > 3 >>"),
		branch!{ else if "5 > 3" });
	assert_eq!(
		parse_unwrap!("<<   else>>"), 
		branch!{ else });
	assert_eq!(
		parse_unwrap!("<<endif >>"), 
		branch!{ end if });
}

#[test]
fn test_tabs() {
	assert_eq!(
		parse_unwrap!("<<\t if true \t>>"),
		branch!{ if "true" });
	assert_eq!(
		parse_unwrap!("<< elseif 5\t > 3 >>"),
		branch!{ else if "5 > 3" });
	assert_eq!(
		parse_unwrap!("<< else\t >>"),
		branch!{ else });
	assert_eq!(
		parse_unwrap!("<<\t endif\t>>"), 
		branch!{ end if });
}

#[test]
fn test_invalid_missing_double_end() {
	expect_err!("<<if {5 + 3 / 2}");
	expect_err!("<<\telseif true");
	expect_err!("<<else >");
	expect_err!("<<  endif\t_");
}

#[test]
fn test_complex_conditions() {
	assert_eq!(
		parse_unwrap!("<< if (5 + 3) > (2 * 5) >>"),
		branch!(if "(5 + 3) > (2 * 5)"));
	assert_eq!(
		parse_unwrap!("<< elseif (5 + 3) > (2 * 5) >>"),
		branch!(else if "(5 + 3) > (2 * 5)"));
	assert_eq!(
		parse_unwrap!("<<if $affection > 5>>"),
		branch!(if "$affection > 5"));
	assert_eq!(
		parse_unwrap!("<<elseif $affection < 10>>"),
		branch!(else if "$affection < 10"));
	assert_eq!(
		parse_unwrap!("<<if (($gold / 2) + ($silver * 3)) / 10 % 2 == 2  >>"),
		branch!(if "(($gold / 2) + ($silver * 3)) / 10 % 2 == 2"));
	assert_eq!(
		parse_unwrap!("<<if ((gold() / 2) + (random_range(3, 5) * 3)) / (10 % silver() + visited(NodeName) - visited_count(\"NodeTest\")) == 2  >>"),
		branch!(if "((gold() / 2) + (random_range(3, 5) * 3)) / (10 % silver() + visited(NodeName) - visited_count(\"NodeTest\")) == 2  "));
	assert_eq!(
		parse_unwrap!("\t<<\tif\t(5\t+\t3)\t>\t(2\t*\t5)\t>>"),
		branch!(if "(5 + 3) > (2 * 5)"));
}

#[test]
fn test_string_conditions() {
	assert_eq!(
		parse_unwrap!(r##"<<if $player_name == "Alice">>"##),
		branch!(if r#"$player_name == "Alice""#));
	assert_eq!(
		parse_unwrap!(r##"<<elseif $player_name == "Bob">>"##),
		branch!(else if r#"$player_name == "Bob""#));
}

macro_rules! expect_none {
    ($lit: literal) => {
	    assert_matches!(parse!($lit), None);
    };
}

#[test]
fn test_invalid_identifier() {
	expect_none!("<<if_  condition >>");
	expect_none!("<<\telseif_  _condition >>");
	expect_none!("<<else_condition >>");
	expect_none!("<<  endif_condition >>");
}

#[test]
fn test_none() {
	expect_none!("if condition");
	expect_none!("<< condition >>");
}

#[test]
fn test_not_confused_with_commands() {
	expect_none!("<<set $some_var 5.3>>");
	expect_none!("<<  set $hey_var = \"StringLiteral\">>");
	expect_none!("<<no_arg_command>>");
	expect_none!("<<command>>");
	expect_none!("<<triple_arg_command 2 \"I LoveLiterals \" 3.>>");
}

#[test]
fn test_not_confused_with_options() {
	expect_none!("-> hello");
	expect_none!("-> hello # metadata   ");
	expect_none!("->Another option but big");
	expect_none!("->Another option but with condition: <<if $player_awake>>");
}

