#[cfg(test)] mod tests;

use std::iter::Peekable;
use std::vec::IntoIter;
use crate::Indent;
use crate::lines::raw::command::YarnCommand;
use anyhow::{anyhow, Result};
use crate::lines::grouping::choices::Choices;
use crate::lines::grouping::if_branch::IfBranch;
use crate::lines::raw::{Content, RawLine};
use crate::lines::raw::speech::Speech;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flow {
	Flat(Vec<FlatLine>),
	Choices(Choices),
	IfBranch(IfBranch),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YarnScope {
	indent: Indent,
	flows: Vec<Flow>,
}

impl YarnScope {
	pub fn indent(&self) -> Indent {
		return self.indent;
	}
	
	pub fn into_flows(self) -> impl Iterator<Item = Flow> {
		return self.flows.into_iter();
	}

	pub fn iter_flows(&self) -> impl Iterator<Item = &Flow> {
		return self.flows.iter();
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlatLine {
	Speech(Speech),
	Command(YarnCommand),
}

fn peek_next_line_indent(lines_iter: &mut Peekable<IntoIter<RawLine>>)
                         -> Option<Indent> {
	return lines_iter
		.peek()
		.map(|line| line.indent);
}

pub fn read_next_scope(parent_indent: Indent, lines_iter: &mut Peekable<IntoIter<RawLine>>)
                       -> Result<Option<YarnScope>> {
	let Some(self_indent) = peek_next_line_indent(lines_iter)
		else { return Ok(None) };
	
	if self_indent <= parent_indent {
		return Ok(None);
	}
	
	let mut flows = Vec::new();
	let mut flat_lines = Vec::new();
	
	loop {
		let Some(next_line) = lines_iter.next_if(|line| line.indent <= self_indent)
			else { break };

		if next_line.indent > self_indent {
			return Err(anyhow!(
				"Unexpected indentation increase.\n\
				 Expected lower or equal to: `{self_indent}`, Found: `{}`\n\
				 Offending line: `{:?}`\n\n\
				 Help: Only branches (started with `<<if [condition]>>`) and \
				 choice options (started with `-> Option Text`) are allowed to increase indentation."
				, next_line.indent, lines_iter.next()));
		}

		if next_line.indent < self_indent {
			break;
		}

		// UNWRAP SOUNDNESS: no advance was made since peek above.
		let next_line = lines_iter.next().unwrap();

		match next_line.content {
			Content::ChoiceOption(first_option) => {
				let choices = Choices::build(self_indent, first_option, lines_iter)?;

				if flat_lines.len() > 0 {
					flows.push(Flow::Flat(flat_lines));
					flat_lines = Vec::new();
				}

				flows.push(Flow::Choices(choices));
			},
			Content::If(if_) => {
				let if_branch = IfBranch::build(self_indent, if_, lines_iter)?;

				if flat_lines.len() > 0 {
					flows.push(Flow::Flat(flat_lines));
					flat_lines = Vec::new();
				}

				flows.push(Flow::IfBranch(if_branch));
			},
			Content::Speech(speech) => {
				flat_lines.push(FlatLine::Speech(speech));
			},
			Content::Command(command) => {
				flat_lines.push(FlatLine::Command(command));
			},
			fork @ (| Content::ElseIf(_)
			| Content::Else(_)
			| Content::EndIf(_)) => {
				return Err(anyhow!(
					"Orphan fork(`<<elseif`, `<<else`, `<<endif`) detected.\n\
					 Offending line: `{fork:?}`\n\n\
					 Help: Forks are only allowed when parallel with a `<<if [condition]>>` branch starter."
				))
			},
		}
	}

	if flat_lines.len() > 0 {
		flows.push(Flow::Flat(flat_lines));
	}

	return if flows.len() > 0 {
		Ok(Some(
			YarnScope {
				indent: self_indent,
				flows,
			}
		))
	} else {
		Ok(None)
	};
}