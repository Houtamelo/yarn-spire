/*
use std::assert_matches::assert_matches;
use houtamelo_utils::own;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::YarnBinaryOp;
use crate::parsing::raw::{Content, ParseRawYarn};
use crate::parsing::raw::speech::{Speech, Speaker};

// Parsing a dialogue line with only speaker and line
#[test]
fn test_parse_dialogue_with_speaker_and_line() {
	let line = "Speaker: This is the dialogue line";
	let expected = Speech {
		line_number: 0,
		speaker: Some(Speaker::Literal(own!("Speaker"))),
		text: (own!("This is the dialogue line"), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with only speaker and line
#[test]
fn test_parse_dialogue_with_speaker_var() {
	let line = "$player_name: This is the dialogue line";
	let expected = Speech {
		line_number: 0,
		speaker: Some(Speaker::Variable(own!("$player_name"))),
		text: (own!("This is the dialogue line"), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with only speaker and line
#[test]
fn test_parse_dialogue_with_speaker_var_and_args() {
	let line = "$player_name: This is the {5 + 3} dialogue line, the player name is ${player_name}";
	let expected = Speech {
		line_number: 0,
		speaker: Some(Speaker::Variable(own!("$player_name"))),
		text: (own!("This is the dialogue line"), 
			vec![
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Add,
					left: Box::new(YarnExpr::Lit(YarnLit::Int(5))),
					right: Box::new(YarnExpr::Lit(YarnLit::Int(3))),
				},
				YarnExpr::VarGet(own!("player_name")),
			]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with only speaker and line
#[test]
fn test_parse_dialogue_with_speaker_and_args() {
	let line = "Speaker: This is { 3 / 10 } the dialogue line";
	let expected = Speech {
		line_number: 0,
		speaker: Some(Speaker::Literal(own!("Speaker"))),
		text: (own!("This is the dialogue line"), 
			vec![
				YarnExpr::BinaryOp {
					yarn_op: YarnBinaryOp::Div,
					left: Box::new(YarnExpr::Lit(YarnLit::Int(3))),
					right: Box::new(YarnExpr::Lit(YarnLit::Int(10))),
				},
			]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with speaker and metadata
#[test]
fn test_parse_dialogue_with_speaker_and_metadata() {
	let line = "Speaker: This is the dialogue line #metadata";
	let expected = Speech {
		line_number: 0,
		speaker: Some(Speaker::Literal(own!("Speaker"))),
		text: (own!("This is the dialogue line"), vec![]),
		metadata: Some(own!("metadata")),
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with only speaker and metadata
#[test]
fn test_parse_dialogue_with_speaker_and_only_metadata() {
	let line = "Speaker: #metadata";
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_matches!(result, Err(_)); // empty lines are invalid
}

// Parsing a dialogue line with only line
#[test]
fn test_parse_dialogue_with_only_line() {
	let line = "This is the dialogue line";
	let expected = Speech {
		line_number: 0,
		speaker: None,
		text: (own!("This is the dialogue line"), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with escaped quotes
#[test]
fn test_parse_dialogue_with_escaped_quotes() {
	let line = "Speaker: \"This is the \\\"dialogue\\\" line\"";
	let expected = Speech {
		line_number: 0,
		speaker: Some(Speaker::Literal(own!("Speaker"))),
		text: (own!("\"This is the \\\"dialogue\\\" line\""), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with empty speaker
#[test]
fn test_parse_dialogue_with_empty_speaker() {
	let line = ": This is the dialogue line";
	let expected = Speech {
		line_number: 0,
		speaker: None,
		text: (own!(": This is the dialogue line"), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with only colon
#[test]
fn test_parse_dialogue_with_only_colon() {
	let line = ":";
	let expected = Speech {
		line_number: 0,
		speaker: None,
		text: (own!(":"), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with only whitespace
#[test]
fn test_parse_dialogue_with_only_whitespace() {
	assert_matches!(Speech::parse_raw_yarn("   ", 0), None);
}

// Parsing a dialogue line with colon in line
#[test]
fn test_parse_dialogue_with_colon_in_line() {
	let line = "This is the dialogue line with a colon: in it";
	let expected = Speech {
		line_number: 0,
		speaker: None,
		text: (own!("This is the dialogue line with a colon: in it"), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

// Parsing a dialogue line with colon in line and speaker
#[test]
fn test_parse_dialogue_with_colon_in_line_and_speaker() {
	let line = "Speaker: This is the dialogue line with a colon: in it";
	let expected = Speech {
		line_number: 0,
		speaker: Some(Speaker::Literal(own!("Speaker"))),
		text: (own!("This is the dialogue line with a colon: in it"), vec![]),
		metadata: None,
	};
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_eq!(result, Ok(expected));
}

#[test]
fn test_from_str_starts_with_double_angle_bracket() {
	let line = "<< This is a test line";
	let result = Speech::parse_raw_yarn(line, 0);
	assert_eq!(result, None);
}

#[test]
fn test_from_str_starts_with_right_arrow() {
	let line = "-> This is a test line";
	let result = Speech::parse_raw_yarn(line, 0);
	assert_eq!(result, None);
}

#[test]
fn test_from_str_with_only_metadata() {
	let line = "#metadata";
	let result = Speech::parse_raw_yarn(line, 0).unwrap();
	assert_matches!(result, Err(_));
}

#[test]
fn test() {
	use std::assert_matches::assert_matches;
	use crate::expressions::yarn_lit::YarnLit;
	use crate::expressions::yarn_ops::YarnBinaryOp;
	use houtamelo_utils::own;

	let source_text = [
		"title: Ch01_Awakening",
		"   tags: #hello there good sir",
		"---",
		"<<fade_in 1>>",
		"<<cg \"CG_ch01_Not-yet-awake\">>",
		"\tYou wake up. Something you shouldn't have done.",
		"Ethel: hey there, {$player_name} after var",
		"<<fade_out 1>>",
		"-> Option A Do that",
		"\t  \t-> Option B Do this # with tags",
		"<<if $condition_true>>",
		"<<elseif false>> // with comments",
		"   \t<<else>>",
		"<<endif>>",
		"===",
		"Ethel: hey there, {as + 1312sa} after var",
		"Ethel: hey there, I played this game {(5 + 7) * 10} times!",
	];

	assert_eq!(Speech::parse_raw_yarn(source_text[5], 5).unwrap(),
		Content::Speech(
			Speech{
				line_number: 5,
				speaker: None,
				text: (own!("\tYou wake up. Something you shouldn't have done."), Vec::new()),
				metadata: None,
			}));

	assert_eq!(Speech::parse_raw_yarn(source_text[6], 6).unwrap(),
		Content::Speech(
			Speech {
				line_number: 6,
				speaker: Some(Speaker::Literal(own!("Ethel"))),
				text: (own!("Ethel: hey there, {} after var"), vec![YarnExpr::VarGet(own!("player_name"))]),
				metadata: None,
			}));

	assert_matches!(Speech::parse_raw_yarn(source_text[15], 15), Err(_));

	assert_eq!(Speech::parse_raw_yarn(source_text[16], 16).unwrap(),
		Content::Speech(
			Speech {
				speaker: Some(Speaker::Literal(own!("Ethel"))),
				line_number: 16,
				text: (own!("Ethel: hey there, I played this game {} times!"),
					vec![
						YarnExpr::BinaryOp {
							yarn_op: YarnBinaryOp::Mul,
							left: Box::from(
								YarnExpr::Parenthesis(
									Box::from(
										YarnExpr::BinaryOp {
											yarn_op: YarnBinaryOp::Add,
											left: Box::new(YarnExpr::Lit(YarnLit::Int(5))),
											right: Box::new(YarnExpr::Lit(YarnLit::Int(7))),
										}
									)
								)
							),
							right: Box::new(YarnExpr::Lit(YarnLit::Int(10)))
						}
					]),
				metadata: None,
			}
		));
}
*/