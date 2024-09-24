use std::collections::HashMap;
use crate::parsing::raw::node_metadata::NodeMetadata;
use crate::quoting::quotable_types::enums::{LineEnum, OptionLineEnum};
use crate::quoting::quotable_types::line_ids::{IDCustomCommand, IDOptionLine, IDOptionsFork, IDSpeech, InstructionKind};
use crate::quoting::quotable_types::scope::IDScope;

pub struct LinesMap<'a> {
	pub speeches: Vec<(&'a IDSpeech, LineEnum<'a>)>,
	pub commands: Vec<(&'a IDCustomCommand, LineEnum<'a>)>,
	pub options_forks: Vec<(&'a IDOptionsFork, LineEnum<'a>)>,
	pub option_lines: Vec<(&'a IDOptionLine, OptionLineEnum<'a>)>,
}

pub struct IDNode {
	pub metadata: NodeMetadata,
	pub scopes: Vec<IDScope>,
}

impl IDNode {
	pub fn map_lines(&self) -> LinesMap {
		let title = self.metadata.title.as_str();
		
		let mut speeches = vec![];
		let mut commands = vec![];
		let mut options_forks = vec![];
		let mut option_lines = vec![];
		
		for scope in &self.scopes {
			scope.map_lines(title, &mut speeches, &mut commands, &mut options_forks, &mut option_lines);
		}
		
		LinesMap {
			speeches, 
			commands, 
			options_forks, 
			option_lines
		}
	}

	#[allow(unused)]
	pub fn speeches(&self) -> impl Iterator<Item = &IDSpeech> {
		std::iter::from_coroutine(#[coroutine] move || {
			for scope in &self.scopes {
				for _speech in scope.iter_speeches() {
					yield _speech;
				}
			}
		})
	}
	
	#[allow(unused)]
	pub fn line_ids(&self) -> impl Iterator<Item = &str> {
		std::iter::from_coroutine(#[coroutine] move || {
			for scope in &self.scopes {
				for _line_id in scope.iter_line_ids() {
					yield _line_id;
				}
			}
		})
	}

	#[allow(unused)]
	pub fn line_ids_by_instruction(&self) -> HashMap<InstructionKind, Vec<&str>> {
		let mut speeches = vec![];
		let mut commands = vec![];
		let mut options_forks = vec![];
		
		for scope in &self.scopes {
			for (kind, line_id) in scope.iter_instructions_kind() {
				match kind {
					InstructionKind::Speech => speeches.push(line_id),
					InstructionKind::Command => commands.push(line_id),
					InstructionKind::OptionsFork => options_forks.push(line_id),
				}
			}
		}
		
		HashMap::from_iter([
			(InstructionKind::Speech, speeches),
			(InstructionKind::Command, commands),
			(InstructionKind::OptionsFork, options_forks),
		])
	}
}