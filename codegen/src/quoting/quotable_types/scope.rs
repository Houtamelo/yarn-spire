use crate::Indent;
use crate::quoting::quotable_types::enums::{LineEnum, OptionLineEnum};
use crate::quoting::quotable_types::line_ids::{IDCustomCommand, IDFlatLine, IDFlow, IDOptionLine, IDOptionsFork, IDSpeech, InstructionKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IDScope {
	pub indent: Indent,
	pub flows: Vec<IDFlow>,
}

impl IDScope {
	pub fn iter_speeches(&self) -> impl Iterator<Item = &IDSpeech> {
		std::iter::from_coroutine(move || {
			for flow in &self.flows {
				match flow {
					IDFlow::Flat(lines) => {
						for line in lines {
							match line {
								IDFlatLine::Speech(_speech) => {
									yield _speech;
								},
								IDFlatLine::CustomCommand(_) => {}
								IDFlatLine::BuiltInCommand(_) => {}
							}
						}
					}
					IDFlow::OptionsFork(options_fork) => {
						for (_line, scope_option) in options_fork.options.iter() {
							if let Some(scope) = scope_option {
								let mut scope_routine = Box::from(scope.iter_speeches());
								while let Some(_speech) = scope_routine.next() {
									yield _speech;
								}
							}
						}
					}
					IDFlow::IfBranch(if_branch) => {
						if let Some(if_scope) = &if_branch.if_.1 {
							let mut scope_routine = Box::from(if_scope.iter_speeches());
							while let Some(_speech) = scope_routine.next() {
								yield _speech;
							}
						}

						for (_, else_if_scope_option) in &if_branch.else_ifs {
							if let Some(else_if_scope) = else_if_scope_option {
								let mut scope_routine = Box::from(else_if_scope.iter_speeches());
								while let Some(_speech) = scope_routine.next() {
									yield _speech;
								}
							}
						}

						if let Some((_, Some(else_scope))) = &if_branch.else_ {
							let mut scope_routine = Box::from(else_scope.iter_speeches());
							while let Some(_speech) = scope_routine.next() {
								yield _speech;
							}
						}
					}
				}
			}
		})
	}
	
	pub fn iter_line_ids(&self) -> impl Iterator<Item = &str> {
		std::iter::from_coroutine(move || {
			for flow in &self.flows {
				match flow {
					IDFlow::Flat(lines) => {
						for line in lines {
							match line {
								IDFlatLine::Speech(_speech) => {
									yield _speech.line_id.as_str();
								},
								IDFlatLine::CustomCommand(_custom_command) => {
									yield _custom_command.line_id.as_str();
								},
								IDFlatLine::BuiltInCommand(_) => {}
							}
						}
					}
					IDFlow::OptionsFork(options_fork) => {
						yield options_fork.virtual_id.as_str();

						for (_line, scope_option) in options_fork.options.iter() {
							yield _line.line_id.as_str();
							
							if let Some(scope) = scope_option {
								let mut scope_routine = Box::from(scope.iter_line_ids());
								while let Some(_line_id) = scope_routine.next() {
									yield _line_id;
								}
							}
						}
					}
					IDFlow::IfBranch(if_branch) => {
						if let Some(if_scope) = &if_branch.if_.1 {
							let mut scope_routine = Box::from(if_scope.iter_line_ids());
							while let Some(_line_id) = scope_routine.next() {
								yield _line_id;
							}
						}
						
						for (_, else_if_scope_option) in &if_branch.else_ifs {
							if let Some(else_if_scope) = else_if_scope_option {
								let mut scope_routine = Box::from(else_if_scope.iter_line_ids());
								while let Some(_line_id) = scope_routine.next() {
									yield _line_id;
								}
							}
						}
						
						if let Some((_, Some(else_scope))) = &if_branch.else_ {
							let mut scope_routine = Box::from(else_scope.iter_line_ids());
							while let Some(_line_id) = scope_routine.next() {
								yield _line_id;
							}
						}
					}
				}
			}
		})
	}
	
	pub fn iter_instructions_kind(&self) -> impl Iterator<Item = (InstructionKind, &str)> {
		std::iter::from_coroutine(move || {
			for flow in &self.flows {
				match flow {
					IDFlow::Flat(lines) => {
						for line in lines {
							match line {
								IDFlatLine::Speech(_speech) => {
									yield (InstructionKind::Speech, _speech.line_id.as_str());
								},
								IDFlatLine::CustomCommand(_custom_command) => {
									yield (InstructionKind::Speech, _custom_command.line_id.as_str());
								},
								IDFlatLine::BuiltInCommand(_) => {}
							}
						}
					}
					IDFlow::OptionsFork(options_fork) => {
						yield (InstructionKind::OptionsFork, options_fork.virtual_id.as_str());

						for (_line, scope_option) in options_fork.options.iter() {
							if let Some(scope) = scope_option {
								let mut scope_routine = Box::from(scope.iter_instructions_kind());
								while let Some(_mapped_line) = scope_routine.next() {
									yield _mapped_line;
								}
							}
						}
					}
					IDFlow::IfBranch(if_branch) => {
						if let Some(if_scope) = &if_branch.if_.1 {
							let mut scope_routine = Box::from(if_scope.iter_instructions_kind());
							while let Some(_mapped_line) = scope_routine.next() {
								yield _mapped_line;
							}
						}

						for (_, else_if_scope_option) in &if_branch.else_ifs {
							if let Some(else_if_scope) = else_if_scope_option {
								let mut scope_routine = Box::from(else_if_scope.iter_instructions_kind());
								while let Some(_mapped_line) = scope_routine.next() {
									yield _mapped_line;
								}
							}
						}

						if let Some((_, Some(else_scope))) = &if_branch.else_ {
							let mut scope_routine = Box::from(else_scope.iter_instructions_kind());
							while let Some(_mapped_line) = scope_routine.next() {
								yield _mapped_line;
							}
						}
					}
				}
			}
		})
	}

	pub fn map_lines<'a>(&'a self,
	                     node_title: &'a str,
	                     speeches: &mut Vec<(&'a IDSpeech, LineEnum<'a>)>,
	                     commands: &mut Vec<(&'a IDCustomCommand, LineEnum<'a>)>,
	                     options_forks: &mut Vec<(&'a IDOptionsFork, LineEnum<'a>)>,
	                     option_lines: &mut Vec<(&'a IDOptionLine, OptionLineEnum<'a>)>) {
		for flow in &self.flows {
			match flow {
				IDFlow::Flat(flat_lines) => {
					for line in flat_lines {
						match line {
							IDFlatLine::Speech(speech) => {
								let line_enum =
									LineEnum {
										node_title,
										raw_id: &speech.line_id,
										instruction_kind: InstructionKind::Speech
									};
								
								speeches.push((speech, line_enum));
							}
							IDFlatLine::CustomCommand(command) => {
								let line_enum =
									LineEnum {
										node_title,
										raw_id: &command.line_id,
										instruction_kind: InstructionKind::Command
									};
								
								commands.push((command, line_enum));
							}
							IDFlatLine::BuiltInCommand(_) => {}
						}
					}
				}
				IDFlow::OptionsFork(options_fork) => {
					let line_enum =
						LineEnum {
							node_title,
							raw_id: &options_fork.virtual_id,
							instruction_kind: InstructionKind::OptionsFork
						};
					
					options_forks.push((options_fork, line_enum));
					
					for (line, scope_option) in options_fork.options.iter() {
						let line_enum =
							OptionLineEnum {
								node_title,
								raw_id: &line.line_id,
							};
						
						option_lines.push((line, line_enum));
						
						if let Some(scope) = scope_option {
							scope.map_lines(node_title, speeches, commands, options_forks, option_lines);
						}
					}
				}
				IDFlow::IfBranch(if_branch) => {
					if let Some(if_scope) = &if_branch.if_.1 {
						if_scope.map_lines(node_title, speeches, commands, options_forks, option_lines);
					}
					
					for (_, else_if_scope_option) in &if_branch.else_ifs {
						if let Some(else_if_scope) = else_if_scope_option {
							else_if_scope.map_lines(node_title, speeches, commands, options_forks, option_lines);
						}
					}
					
					if let Some((_, Some(else_scope))) = &if_branch.else_ {
						else_scope.map_lines(node_title, speeches, commands, options_forks, option_lines);
					}
				}
			}
		}
	}
}