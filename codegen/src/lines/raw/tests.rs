use houtamelo_utils::own;
use crate::lines::raw::command::YarnCommand;
use crate::UnparsedLine;
use super::{parse_source_lines_into_raw_nodes, RawLine};
use super::Content;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::lines::raw::branches::choices::ChoiceOption;
use crate::lines::raw::branches::if_statement::{ElseIfStruct, ElseStruct, EndIfStruct, IfStruct};
use crate::lines::raw::speech::Speech;

#[test]
fn test_parsing() {
	let source_text = [
		"title: Ch01_Awakening",
		"   tags: #hello there good sir",
		"---",
		"<<fade_in 1>>",
		"<<cg \"CG_ch01_Not-yet-awake\">>",
		"\tYou wake up. Something you shouldn't have done. // comments should be ignored",
		"<<fade_out 1>>",
		"-> Option A Do that",
		"\t  \t-> Option B Do this # with tags",
		"<<if $condition_true>>",
		"<<elseif false>> // with comments",
		"   \t<<else>>",
		"<<endif>>",
		"===",
		"<<set $num1 to 50>>",
	];

	let unparsed_lines =
		source_text
			.into_iter()
			.enumerate()
			.map(|(line_number, text)| UnparsedLine { line_number, text: text.to_string() })
			.collect::<Vec<_>>();
	
	let raw_lines = 
		parse_source_lines_into_raw_nodes(unparsed_lines)
			.unwrap();
	
	assert_eq!(raw_lines[3], RawLine { indent: 0, content: Content::Command(
		YarnCommand::Other { line_number: 3, variant: own!("fade_in"), args: vec![YarnExpr::Lit(YarnLit::Int(1))] }) });
	assert_eq!(raw_lines[4], RawLine { indent: 0, content: Content::Command(
		YarnCommand::Other { line_number: 4, variant: own!("cg"), args: vec![YarnExpr::Lit(YarnLit::Str(own!("CG_ch01_Not-yet-awake")))] }) });
	assert_eq!(raw_lines[5], RawLine { indent: 4, content: Content::Speech(
		Speech {
			line_number: 5,
			speaker: None,
			line: (own!("You wake up. Something you shouldn't have done."), vec![]),
			metadata: None,
		})
	});
	assert_eq!(raw_lines[6], RawLine { indent: 0, content: Content::Command(
		YarnCommand::Other { line_number: 6, variant: own!("fade_out"), args: vec![YarnExpr::Lit(YarnLit::Int(1))] }) });
	assert_eq!(raw_lines[7], RawLine { indent: 0, content: Content::ChoiceOption(
		ChoiceOption { line_number: 7, line: (own!("Option A Do that"), vec![]), if_condition: None, metadata: None }) });
	assert_eq!(raw_lines[8], RawLine { indent: 10, content: Content::ChoiceOption(
		ChoiceOption { line_number: 8, line: (own!("Option B Do this"), vec![]), if_condition: None, metadata: Some(own!("with tags")) }) });
	assert_eq!(raw_lines[9], RawLine { indent: 0, content: Content::If(
		IfStruct { line_number: 9, condition: YarnExpr::VarGet(own!("condition_true")) }) });
	assert_eq!(raw_lines[10], RawLine { indent: 0, content: Content::ElseIf(
		ElseIfStruct { line_number: 10, condition: YarnExpr::Lit(YarnLit::Bool(false)) }) });
	assert_eq!(raw_lines[11], RawLine { indent: 7, content: Content::Else(
		ElseStruct { line_number: 11 }) });
	assert_eq!(raw_lines[12], RawLine { indent: 0, content: Content::EndIf(
		EndIfStruct { line_number: 12 }) });
	assert_eq!(raw_lines[14], RawLine { indent: 0, content: Content::Command(
		YarnCommand::Set { line_number: 14, var_name: own!("num1"), arg: YarnExpr::Lit(YarnLit::Int(50)) }) });
}