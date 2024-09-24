use anyhow::{Result, anyhow};
use crate::expressions::parse_yarn_expr;
use crate::expressions::yarn_expr::YarnExpr;
use crate::LineNumber;
use crate::parsing::macros::{return_if_err, strip_end_then_trim, strip_start, strip_start_then_trim};
use crate::parsing::raw::{ParseRawYarn, Content};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If_ {
	pub line_number: LineNumber,
	pub condition: YarnExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElseIf_ {
	pub line_number: LineNumber,
	pub condition: YarnExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Else_ {
	pub line_number: LineNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndIf_ {
	pub line_number: LineNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchKind {
	If,
	ElseIf,
	Else,
	EndIf,
}

impl ParseRawYarn for BranchKind {
	fn parse_raw_yarn(line: &str, line_number: LineNumber,) -> Option<Result<Content>> {
		let mut line = line.trim();

		if !strip_start_then_trim!(line, "<<") {
			return None;
		}
		
		let branch_kind =
			if strip_start_then_trim!(line, "if ") 
			|| strip_start_then_trim!(line, "if\t"){
				BranchKind::If
			} else if strip_start!(line, "endif")
				&& line.starts_with([' ', '>', '\t']) {
				BranchKind::EndIf
			} else if strip_start_then_trim!(line, "elseif ")
			|| strip_start_then_trim!(line, "elseif\t") {
				BranchKind::ElseIf
			} else if strip_start!(line, "else") 
				&& line.starts_with([' ', '>', '\t']) {
				BranchKind::Else
			}  else {
				return None;
			};
		
		if !strip_end_then_trim!(line, ">>") {
			return Some(Err(anyhow!(
				"{branch_kind:?} statement did not end with `>>`.\n\
				 Remaining Line: `{line}`")));
		}
		
		match branch_kind {
			BranchKind::If => {
				let condition = return_if_err!(
					parse_yarn_expr(line)
						.map_err(|err| anyhow!(
							"Could not parse condition in `<<if [condition]>>` statement.\n\
							 Error: `{err:?}`\n\
							 Condition as String: `{line}`")));
				
				Some(Ok(Content::If(
					If_ {
						line_number,
						condition,
					})))
			},
			BranchKind::ElseIf => {
				let condition =
					return_if_err!(
						parse_yarn_expr(line)
							.map_err(|err| anyhow!(
								"Could not parse condition in `<<elseif [condition]>>` statement.\n\
								 Error: `{err:?}`\n\
								 Condition as String: `{line}`"))
					);
				
				Some(Ok(Content::ElseIf(
					ElseIf_ {
						line_number,
						condition,
					})))
			},
			BranchKind::Else => {
				Some(Ok(Content::Else(Else_ { line_number })))
			},
			BranchKind::EndIf => {
				Some(Ok(Content::EndIf(EndIf_ { line_number })))
			},
		}
	}
}