use std::iter::Peekable;
use std::vec::IntoIter;
use crate::Indent;
use anyhow::*;
use houtamelo_utils::prelude::CountOrMore;
use scope::read_next_scope;
use crate::parsing::grouping::scope;
use crate::parsing::grouping::scope::YarnScope;
use crate::parsing::raw::branches::choices::OptionLine;
use crate::parsing::raw::{Content, RawLine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsFork {
	pub options: CountOrMore<1, (OptionLine, Option<Box<YarnScope>>)>,
}

impl OptionsFork {
	fn options_indent(&self) -> Option<Indent> {
		self.options
			.iter()
			.find_map(|(_, scope_option)| 
				scope_option
					.as_ref()
					.map(|scope| scope.indent()))
	}

	fn add_option(&mut self, choice_option: OptionLine,
	              option_scope: Option<Box<YarnScope>>) -> Result<()> {
		if let (Some(expected), Some(found)) =
			(self.options_indent(), option_scope.as_ref().map(|scope| scope.indent()))
			&& found != expected {
			return Err(anyhow!(
				"Cannot add `option line` to `options fork`.\n\
				 Error: Indentation mismatch, expected: {expected}, got: {found}.\n\
				 Trying to add: {choice_option:?}.\n\
				 Other options: \n\t{:?}\n\n\
				 Help: Indentation is calculated by: [space + tabs * 4]\n\
				 Help: Every option line should have the same indentation."
				, self.options.iter().map(|(o, _)| format!("{o:?}")).collect::<Vec<String>>().join("\n\t")));
		}

		self.options.push((choice_option, option_scope));
		return Ok(());
	}
	
	pub fn build(parent_indent: Indent, first_option: OptionLine,
	             lines_iter: &mut Peekable<IntoIter<RawLine>>) -> Result<OptionsFork> {
		let first_option_scope =
			read_next_scope(parent_indent, lines_iter)
				.map_err(|err| anyhow!(
					"Could not build option's child scope.\n\
					 Option data: `{first_option:?}`\n\
					 Error: {err}")
				)?;
		
		let mut choices = OptionsFork {
			options: CountOrMore::new([
				(first_option, first_option_scope.map(Box::from))], 
				vec![])
		};

		while let Some(Content::OptionLine(choice_option)) = 
			lines_iter.next_if(|next| {
				parent_indent == next.indent && matches!(next.content, Content::OptionLine(_))
			}).map(|next| next.content)
		{
			let next_option_scope =
				read_next_scope(parent_indent, lines_iter)
					.map_err(|err| anyhow!(
						"Could not build option's child scope.\n\
						 Option data: `{choice_option:?}`\n\
						 Error: {err}\n\
						 Other options: \n\t{:?}"
						, choices.options.iter().map(|(o, _)| format!("{o:?}")).collect::<Vec<String>>().join("\n\t"))
					)?;
			
			choices.add_option(choice_option, next_option_scope.map(Box::from))?;
		}

		return Ok(choices);
	}
	
	pub fn into_iter_options(self) -> impl Iterator<Item = (OptionLine, Option<Box<YarnScope>>)> {
		return self.options.into_iter();
	}
	
	pub fn iter_options(&self) -> impl Iterator<Item = &(OptionLine, Option<Box<YarnScope>>)> {
		return self.options.iter();
	}
}