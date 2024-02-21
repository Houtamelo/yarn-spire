#[cfg(test)] mod tests;

pub mod branches;
pub mod command;
pub mod speech;
pub mod node_metadata;
pub mod splitting;

use anyhow::{Result, anyhow};
use command::YarnCommand;
use branches::choices::ChoiceOption;
use branches::if_statement::{ElseIfStruct, IfStruct};
use node_metadata::parse_metadata;
use speech::Speech;
use splitting::split_into_unparsed_nodes;
use crate::lines::util;
use crate::{Indent, LineNumber, UnparsedLine};
use crate::lines::macros::trim;
use crate::lines::raw::branches::if_statement::{BranchKind, ElseStruct, EndIfStruct};
use crate::lines::raw::node_metadata::NodeMetadata;

pub trait ParseRawYarn {
	fn parse_raw_yarn(line: &str, line_number: LineNumber) -> Option<Result<Content>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
	Speech(Speech),
	Command(YarnCommand),
	ChoiceOption(ChoiceOption),
	If(IfStruct),
	ElseIf(ElseIfStruct),
	Else(ElseStruct),
	EndIf(EndIfStruct),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLine {
	pub indent: Indent,
	pub content: Content,
}

impl RawLine {
	pub fn number(&self) -> LineNumber {
		return match &self.content {
			Content::Speech(speech) => speech.line_number,
			Content::Command(command) => command.line_number(),
			Content::ChoiceOption(choice) => choice.line_number,
			Content::If(if_struct) => if_struct.line_number,
			Content::ElseIf(else_if_struct) => else_if_struct.line_number,
			Content::Else(else_struct) => else_struct.line_number,
			Content::EndIf(end_if_struct) => end_if_struct.line_number,
		};
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawNode {
	pub metadata: NodeMetadata,
	pub lines: Vec<RawLine>,
}

fn parse_line(line_number: LineNumber, indent: Indent, line: &impl AsRef<str>)
              -> Result<RawLine> {
	let line = line.as_ref();

	macro_rules! try_parse {
	    ($parse_ty: ty) => {
		    match <$parse_ty>::parse_raw_yarn(line, line_number) {
				Some(Ok(content)) => { 
					return Ok(RawLine {
						content, 
						indent 
					})
				},
				Some(Err(err)) => {
					return Err(anyhow!(
						"Could not parse line as `{}`\n\
						 Line nº{line_number}: `{line}`\n\
						 Error: {err}", std::any::type_name::<$parse_ty>())
					);
				},
			    None => {}
			}
	    };
	}

	try_parse!(Speech);
	try_parse!(YarnCommand);
	try_parse!(ChoiceOption);
	try_parse!(BranchKind);
	
	return Err(anyhow!(
		"Line could not be parsed as any YarnSyntax.\n\
		 Line nº{line_number}: `{line}"
	));
}

pub fn parse_source_lines_into_raw_nodes(mut source_lines: Vec<UnparsedLine>)
                                         -> Result<Vec<RawNode>> {
	source_lines
		.retain_mut(|line| {
			if let Some(comment_index) = line.text.find("//") {
				line.text.truncate(comment_index);
			}

			line.text.len() > 0
		});
	
	let unparsed_nodes = 
		split_into_unparsed_nodes(&source_lines)
			.map_err(|err| anyhow!(
				"Could not split file into nodes.\n\
				 Error: {err}"
			))?;
	
	let raw_nodes =
		unparsed_nodes
			.into_iter()
			.map(|unparsed_node| {
				let metadata = 
					parse_metadata(unparsed_node.outer_lines)
						.map_err(|err| anyhow!(
							"Could not parse node metadata. \n\
							 Error: {err}"
						))?;
				
				let lines =
					unparsed_node
						.inner_lines
						.into_iter()
						.filter_map(|line| {
							let mut text = line.text.as_str();
							let indent = util::indent_level(&text);
							trim!(text);

							if text.len() > 0 {
								Some(parse_line(line.line_number, indent, &text))
							} else {
								None
							}
						}).collect::<Result<Vec<RawLine>>>()?;

				Ok(RawNode { metadata, lines })
			}).collect::<Result<Vec<RawNode>>>()?;
	
	return Ok(raw_nodes);
}
