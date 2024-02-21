use std::iter::Peekable;
use std::vec::IntoIter;
use crate::Indent;
use anyhow::*;
use scope::read_next_scope;
use crate::lines::grouping::scope;
use crate::lines::grouping::scope::YarnScope;
use crate::lines::raw::branches::choices::ChoiceOption;
use crate::lines::raw::{Content, RawLine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choices {
	pub first_option: (ChoiceOption, Option<Box<YarnScope>>),
	pub options: Vec<(ChoiceOption, Option<Box<YarnScope>>)>,
}

impl Choices {
	fn options_indent(&self) -> Option<Indent> {
		if let Some(indent) = 
			self.first_option.1
				.as_ref()
				.map(|scope| scope.indent()) {
			return Some(indent);
		}

		return self.options
			.iter()
			.find_map(|(_, scope_option)| scope_option
				.as_ref()
				.map(|scope| scope.indent()));
	}

	fn add_option(&mut self, choice_option: ChoiceOption,
	              option_scope: Option<Box<YarnScope>>) -> Result<()> {
		if let (Some(expected), Some(found)) =
			(self.options_indent(), option_scope.as_ref().map(|scope| scope.indent()))
			&& found != expected {
			return Err(anyhow!(
				"Cannot add option to choices list.\n\
				 Error: Indentation mismatch, expected: {expected}, got: {found}.\n\
				 Trying to add: {choice_option:?}.\n\
				 Other options: \n\t{:?}\n\n\
				 Help: Indentation is calculated by: [space + tabs * 4]\n\
				 Help: Every option should have the same indentation."
				, self.options.iter().map(|(o, _)| format!("{o:?}")).collect::<Vec<String>>().join("\n\t")		
			));
		}

		self.options.push((choice_option, option_scope));
		Ok(())
	}
	
	pub fn build(parent_indent: Indent, first_option: ChoiceOption, 
	             lines_iter: &mut Peekable<IntoIter<RawLine>>) -> Result<Choices> {
		let first_option_scope =
			read_next_scope(parent_indent, lines_iter)
				.map_err(|err| anyhow!(
					"Could not build option's child scope.\n\
					 Option data: `{first_option:?}`\n\
					 Error: {err}"
				))?;
		
		let mut choices = Choices {
			first_option: (first_option, first_option_scope.map(Box::from)),
			options: Vec::new(),
		};

		while let Some(Content::ChoiceOption(choice_option)) = 
			lines_iter.next_if(|next| {
				parent_indent == next.indent && matches!(next.content, Content::ChoiceOption(_))
			}).map(|next| next.content)
		{
			let next_option_scope =
				read_next_scope(parent_indent, lines_iter)
					.map_err(|err| anyhow!(
						"Could not build option's child scope.\n\
						 Option data: `{choice_option:?}`\n\
						 Error: {err}\n\
						 Other options: \n\t{:?}"
						, choices.options.iter().map(|(o, _)| format!("{o:?}")).collect::<Vec<String>>().join("\n\t")
					))?;
			
			choices.add_option(choice_option, next_option_scope.map(Box::from))?;
		}

		return Ok(choices);
	}
	
	pub fn into_iter_options(self) -> ((ChoiceOption, Option<Box<YarnScope>>), impl Iterator<Item = (ChoiceOption, Option<Box<YarnScope>>)>) {
		return (self.first_option, self.options.into_iter());
	}
	
	pub fn iter_options(&self) -> (&(ChoiceOption, Option<Box<YarnScope>>), 
	                              impl Iterator<Item = &(ChoiceOption, Option<Box<YarnScope>>)>) {
		return (&self.first_option, self.options.iter());
	}
}