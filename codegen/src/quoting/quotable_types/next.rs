use genco::lang::rust::Tokens;
use genco::quote_in;
use crate::quoting::quotable_types::enums::LineEnum;
use crate::quoting::quotable_types::line_ids::*;
use crate::quoting::quotable_types::scope::IDScope;

trait NextFn {
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool;
}

impl NextFn for &IDFlatLine {
	/// # Returns
	/// If the `next` function returned.(If yes, iteration should stop)
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool {
		match self {
			IDFlatLine::Speech(IDSpeech { line_id, .. }) => {
				tokens.line();
				
				let line_enum = LineEnum { 
					node_title, 
					raw_id: line_id.as_str(), 
					instruction_kind: InstructionKind::Speech
				};
				
				quote_in!(*tokens => return YarnYield::Instruction($(line_enum.any_qualified()).into()); );
				return true;
			},
			IDFlatLine::CustomCommand(IDCustomCommand { line_id, .. }) => {
				tokens.line();
				
				let line_enum = LineEnum { 
					node_title, 
					raw_id: line_id.as_str(), 
					instruction_kind: InstructionKind::Command
				};
				
				quote_in!(*tokens => return YarnYield::Instruction($(line_enum.any_qualified()).into()); );
				return true;
			},
			IDFlatLine::BuiltInCommand(built_in_command) => {
				match built_in_command {
					BuiltInCommand::Set { var_name, value, .. } => {
						tokens.line();
						quote_in!(*tokens => storage.set_var::<$var_name>($value););
						return false;
					},
					BuiltInCommand::Jump { node_destination_title, .. } => {
						tokens.line();
						quote_in!(*tokens => 
							let current_title = NodeTitle::$node_title($node_title);
							match current_title.tracking() {
								TrackingSetting::Always => {
									storage.increment_visited(&current_title);
								},
								TrackingSetting::Never => {},
							}
										
							return NodeTitle::$node_destination_title($node_destination_title).start(storage);
						);

						return true;
					},
					BuiltInCommand::Stop { .. } => {
						tokens.line();
						quote_in!(*tokens => return YarnYield::Finished;);
						return true;
					},
				}
			},
		}
	}
}

impl NextFn for &IDOptionsFork {
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool {
		tokens.line();
		
		let line_enum = LineEnum { 
			node_title, 
			raw_id: self.virtual_id.as_str(), 
			instruction_kind: InstructionKind::OptionsFork
		}; 
		
		quote_in!(*tokens => return YarnYield::Instruction($(line_enum.any_qualified()).into()););
		return true;
	}
}

impl NextFn for &IDIfBranch {
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool {
		tokens.line();
		
		let mut all_returned = true;

		let inside_if = 
			match &self.if_.1 {
				Some(scope) => {
					let mut if_tokens = Tokens::new();
					all_returned &= scope.quote_next_fn(&mut if_tokens, node_title);
					if_tokens
				},
				None => Tokens::new(),
			};
		
		quote_in!(*tokens => 
			if $(&self.if_.0.condition) {
				$inside_if
			}
		);
		
		for (else_if, scope_option) in &self.else_ifs {
			let inside_else_if = 
				match scope_option {
					Some(scope) => {
						let mut else_if_tokens = Tokens::new();
						all_returned &= scope.quote_next_fn(&mut else_if_tokens, node_title);
						else_if_tokens
					},
					None => Tokens::new(),
				};
			
			quote_in!(*tokens => 
				else if $(&else_if.condition) {
					$inside_else_if
				}
			);
		}
		
		if let Some((_, scope_option)) = &self.else_ {
			let inside_else = 
				match scope_option {
					Some(scope) => {
						let mut else_tokens = Tokens::new();
						all_returned &= scope.quote_next_fn(&mut else_tokens, node_title);
						else_tokens
					},
					None => Tokens::new(),
				};
			
			quote_in!(*tokens => 
				else {
					$inside_else
				}
			);
		}
		
		return all_returned;
	}
}

fn flat_lines_next_fn<'a>(flat_lines: impl IntoIterator<Item = &'a IDFlatLine>, 
                          tokens: &mut Tokens, node_title: &str) -> bool {
	for line in flat_lines {
		if line.quote_next_fn(tokens, node_title) {
			return true;
		}
	}

	return false;
}


fn flows_next_fn<'a>(flows: impl IntoIterator<Item = &'a IDFlow>, tokens: &mut Tokens, node_title: &str) -> bool {
	for flow in flows {
		match flow {
			IDFlow::Flat(flat_lines) => {
				for line in flat_lines {
					if line.quote_next_fn(tokens, node_title) {
						return true;
					}
				}
			},
			IDFlow::OptionsFork(options_fork) => {
				if options_fork.quote_next_fn(tokens, node_title) {
					return true;
				}
			},
			IDFlow::IfBranch(if_branch) => {
				if if_branch.quote_next_fn(tokens, node_title) {
					return true;
				}
			},
		}
	}
	
	return false;
}


impl NextFn for &IDScope {
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool {
		return flows_next_fn(&self.flows, tokens, node_title);
	}
}

pub fn build_next_fn<'a>(flats_after: impl IntoIterator<Item = &'a IDFlatLine>,
                         flows_after: impl IntoIterator<Item = &'a IDFlow>,
                         scopes_after: impl IntoIterator<Item = &'a IDScope>,
                         node_title: &str) -> Tokens {
	let mut tokens = Tokens::new();
	
	for line in flats_after {
		if line.quote_next_fn(&mut tokens, node_title) {
			return tokens;
		}
	}

	if flows_next_fn(flows_after, &mut tokens, node_title) {
		return tokens;
	}
	
	for scope in scopes_after {
		if scope.quote_next_fn(&mut tokens, node_title) {
			return tokens;
		}
	}
	
	tokens.line();
	quote_in!(tokens => 
		return YarnYield::Finished;
	);
	
	return tokens;
}
