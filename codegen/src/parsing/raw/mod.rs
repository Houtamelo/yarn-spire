#[cfg(test)]
mod tests;

pub mod branches;
pub mod command;
pub mod speech;
pub mod node_metadata;
pub mod splitting;
pub mod var_declaration;
pub mod arg_parser;

use crate::parsing::macros::{strip_start_then_trim, trim};
use crate::parsing::raw::branches::if_statement::{BranchKind, Else_, EndIf_};
use crate::parsing::raw::branches::options::EndOptions;
use crate::parsing::raw::node_metadata::NodeMetadata;
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::parsing::util;
use crate::{Indent, LineNumber, UnparsedLine};
use anyhow::{anyhow, Result};
use branches::if_statement::{ElseIf_, If_};
use branches::options::OptionLine;
use command::YarnCommand;
use node_metadata::parse_metadata;
use speech::Speech;
use splitting::split_into_unparsed_nodes;

pub trait ParseRawYarn {
	fn parse_raw_yarn(line: &str, line_number: LineNumber) -> Option<Result<Content>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
	Speech(Speech),
	Command(YarnCommand),
	OptionLine(OptionLine),
	EndOptions(EndOptions),
	If(If_),
	ElseIf(ElseIf_),
	Else(Else_),
	EndIf(EndIf_),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLine {
	pub indent: Indent,
	pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawNode {
	pub metadata: NodeMetadata,
	pub lines: Vec<RawLine>,
}

impl RawLine {
	#[allow(unused)]
	pub fn number(&self) -> LineNumber {
		match &self.content {
			Content::Speech(speech) => speech.line_number,
			Content::Command(command) => command.line_number,
			Content::OptionLine(option_line) => option_line.line_number,
			Content::EndOptions(end_options) => end_options.line_number,
			Content::If(if_struct) => if_struct.line_number,
			Content::ElseIf(else_if_struct) => else_if_struct.line_number,
			Content::Else(else_struct) => else_struct.line_number,
			Content::EndIf(end_if_struct) => end_if_struct.line_number,
		}
	}
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
	try_parse!(OptionLine);
	try_parse!(BranchKind);

	Err(anyhow!(
		"Line could not be parsed as any YarnSyntax.\n\
		 Line nº{line_number}: `{line}"
	))
}

pub fn parse_raw_nodes(mut source_lines: Vec<UnparsedLine>) -> Result<(Vec<RawNode>, Vec<VarDeclaration>)> {
	source_lines
		.retain_mut(|line| {
			if let Some(comment_index) = line.text.find("//") {
				line.text.truncate(comment_index);
			}

			!line.text.is_empty()
		});

	let var_declarations: Vec<VarDeclaration> =
		source_lines
			.extract_if(|unparsed_line| {
				let mut temp = unparsed_line.text.as_str().trim();
				strip_start_then_trim!(temp, "<<") && temp.starts_with("declare")
			})
			.map(|unparsed_line|
				VarDeclaration::try_parse(&unparsed_line)
					.ok_or(anyhow!(
						"Could not parse line as variable declaration(`<<declare $var_name (=) [default_value]>>`).\n\
						 Line nº{}: `{}`", unparsed_line.line_number, unparsed_line.text)
					)?
			).try_collect()?;

	let unparsed_nodes = split_into_unparsed_nodes(&source_lines)
		.map_err(|err| anyhow!("Could not split file into nodes.\nError: {err}"))?;

	let raw_nodes = unparsed_nodes
		.into_iter()
		.map(|unparsed_node| {
			let metadata = parse_metadata(unparsed_node.outer_lines)
				.map_err(|err| anyhow!("Could not parse node metadata.\nError: {err}"))?;

			let lines: Vec<RawLine> = unparsed_node
				.inner_lines
				.into_iter()
				.filter_map(|line| {
					let mut text = line.text.as_str();
					let indent = util::indent_level(&text);
					trim!(text);

					if !text.is_empty() {
						Some(parse_line(line.line_number, indent, &text))
					} else {
						None
					}
				}).try_collect()?;

			Result::<_>::Ok(RawNode { metadata, lines })
		}).try_collect()?;

	Ok((raw_nodes, var_declarations))
}
