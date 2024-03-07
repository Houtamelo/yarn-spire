use std::iter::Peekable;
use std::vec::IntoIter;
use anyhow::{anyhow, Result};
use scope::read_next_scope;
use crate::Indent;
use crate::parsing::grouping::scope;
use crate::parsing::grouping::scope::YarnScope;
use crate::parsing::raw::branches::if_statement::{ElseIf_, Else_, If_};
use crate::parsing::raw::{Content, RawLine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfBranch {
	pub if_: (If_, Option<Box<YarnScope>>),
	pub else_ifs: Vec<(ElseIf_, Option<Box<YarnScope>>)>,
	pub else_: Option<(Else_, Option<Box<YarnScope>>)>,
}

impl IfBranch {
	fn branches_indent(&self) -> Option<Indent> {
		if let Some(indent) = 
			self.if_.1
				.as_ref()
				.map(|scope| scope.indent()) {
			return Some(indent);
		}

		if let Some(indent) =
			self.else_ifs
			    .iter()
			    .find_map(|(_, scope_option)| 
				    scope_option.as_ref().map(|scope| scope.indent())) {
			return Some(indent);
		}

		return self.else_
			.as_ref()
			.map(|(_, scope_option)| 
				scope_option.as_ref().map(|scope| scope.indent()))
			.flatten();
	}

	fn add_elseif(&mut self, else_if: ElseIf_,
	              scope_option: Option<Box<YarnScope>>) -> Result<()> {
		if let (Some(expected), Some(found)) 
			= (self.branches_indent(), scope_option.as_ref().map(|scope| scope.indent()))
			&& found != expected {
			return Err(anyhow!(
				"Cannot add `elseif` to fork list.\n\
				 Error: Indentation mismatch, expected: {expected}, got: {found}.\n\
				 Trying to add: {else_if:?}.\n\
				 Help: Indentation is calculated by: [space + tabs * 4]\n\
				 Help: Every fork(`<<if`, `<<elseif`, `<<else`) needs to have the same indentation."
			));
		}

		self.else_ifs.push((else_if, scope_option));
		Ok(())
	}

	fn add_else(&mut self, else_: Else_,
	            scope_option: Option<Box<YarnScope>>) -> Result<()> {
		if let (Some(expected), Some(found))
			= (self.branches_indent(), scope_option.as_ref().map(|scope| scope.indent()))
			&& found != expected {
			return Err(anyhow!(
				"Cannot add `else` to fork list.\n\
				 Error: Indentation mismatch, expected: {expected}, got: {found}.\n\
				 Trying to add: {else_:?}.\n\n\
				 Help: Indentation is calculated by: [space + tabs * 4]\n\
				 Help: Every fork(`<<if`, `<<elseif`, `<<else`) needs to have the same indentation."
			));
		}
		
		return if let Some((already_else, _)) = &self.else_ {
			Err(anyhow!(
				"Cannot add `else` to fork list.\n\
				 Error: branch already contains another `else`, data: {already_else:?}.\n\
				 Trying to add: {else_:?}.\n\n\
				 Help: There can only be one `else` fork per branch."
			))
		} else {
			self.else_ = Some((else_, scope_option));
			Ok(())
		};
	}

	pub fn build(parent_indent: Indent, if_line: If_,
	             lines_iter: &mut Peekable<IntoIter<RawLine>>) -> Result<IfBranch> {
		let if_scope = 
			read_next_scope(parent_indent, lines_iter)
				.map_err(|err| anyhow!(
					"Could not build `if`'s child scope.\n\
					 Fork data: `{if_line:?}`\n\
					 Error: {err}")
				)?;
		
		let mut if_branch =
			IfBranch {
				if_: (if_line, if_scope.map(Box::from)),
				else_ifs: Vec::new(),
				else_: None,
			};

		while let Some(Content::ElseIf(elseif)) =
			lines_iter.next_if(|next|
				parent_indent == next.indent && matches!(next.content, Content::ElseIf(_)))
					  .map(|next| next.content)
		{
			let next_else_if_scope =
				read_next_scope(parent_indent, lines_iter)
					.map_err(|err| anyhow!(
						"Could not build `elseif`'s child scope.\n\
						 Fork data: `{elseif:?}`\n\
						 Error: `{err}`"
					))?;
			
			if_branch.add_elseif(elseif, next_else_if_scope.map(Box::from))?;
		}

		// Else or EndIf
		{
			let Some(next_line) = lines_iter.next()
				else { return Err(anyhow!(
					"Node ended before branch was closed, expected `<<endif>>`.\n\
					 Unclosed branch started at: `{:?}`\n\n\
					 Help: Branches are started with `<<if [condition]>>`, then ended with `<<endif>>`."
					, if_branch.if_))
				};

			if next_line.indent != parent_indent {
				return Err(anyhow!(
					"Indentation mismatch in `<<else>>` or `<<endif>>`, expected: {parent_indent}, got: {}\n\
					 At line `{:?}`\n\n\
					 Help: Indentation is calculated by: [space + tabs * 4]\n\
					 Help: Every fork(`<<if`, `<<elseif`, `<<else`) needs to have the same indentation."
					, next_line.indent, next_line.content));
			}

			match next_line.content {
				invalid_content @ (
				| Content::Speech(_)
				| Content::Command(_)
				| Content::OptionLine(_)
				| Content::EndOptions(_)
				| Content::If(_)
				| Content::ElseIf(_)) => {
					return Err(anyhow!(
						"Expected `<<else>>` or `<<endif>>`\n\
						 Got: {:?}\n\n\
						 Help: Branches are started with `<<if [condition]>>`, then closed with `<<endif>>`.\n\
						 Help: As long as a Branch is open, parallel(same indentation, not necessarily adjacent) lines must be \
						 either `<<elseif [condition]>>`, `<<else>>` or `<<endif>>`.\n\
						 Help: To close a branch, use `<<endif>>`."
						, invalid_content));
				},
				Content::EndIf(_) => {
					return Ok(if_branch);
				},
				Content::Else(else_) => {
					let else_child_scope =
						read_next_scope(parent_indent, lines_iter)
							.map_err(|err| anyhow!(
								"Could not build `else`'s child scope.\n\
						         Else data: `{else_:?}`\n\
								 Error: `{err}`")
							)?;
					
					if_branch.add_else(else_, else_child_scope.map(Box::from))?;
				},
			}
		}

		// Only Endif
		{
			let Some(next_line) = lines_iter.next()
				else { return Err(anyhow!(
					"Node ended before branch was closed, expected `<<endif>>`.\n\
					 Unclosed branch started at: `{:?}`\n\n\
					 Help: Branches are started with `<<if [condition]>>`, then ended with `<<endif>>`."
					, if_branch.if_))
				};

			if next_line.indent != parent_indent {
				return Err(anyhow!(
					"Indentation mismatch in `<<endif>>`, expected: {parent_indent}, got: {}\n\n\
					 Help: Indentation is calculated by: [space + tabs * 4]\n\
					 Help: Every fork(`<<if`, `<<elseif`, `<<else`) needs to have the same indentation."
					, next_line.indent));
			}

			return match next_line.content {
				invalid_content @(
				| Content::Speech(_)
				| Content::Command(_)
				| Content::OptionLine(_)
				| Content::EndOptions(_)
				| Content::If(_)
				| Content::ElseIf(_)
				| Content::Else(_)) => {
					Err(anyhow!(
						"Expected `<<endif>>`\n\
						 Got: {:?}\n\n\
						 Help: Branches are started with `<<if [condition]>>`, then closed with `<<endif>>`.\n\
						 Help: As long as a Branch is open, parallel(same indentation, not necessarily adjacent) lines must be \
						 either `<<elseif [condition]>>`, `<<else>>` or `<<endif>>`.\n\
						 Help: To close a branch, use `<<endif>>`."
						, invalid_content))
				},
				Content::EndIf(_) => {
					Ok(if_branch)
				},
			};
		}
	}
}