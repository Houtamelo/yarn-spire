use genco::lang::rust::Tokens;
use genco::quote_in;
use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::raw::command::SetOperation;
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
				if !tokens.is_empty() {
					tokens.push();
				}
				
				let line_enum = LineEnum { 
					node_title, 
					raw_id: line_id.as_str(), 
					instruction_kind: InstructionKind::Speech
				};
				
				quote_in!(*tokens => return YarnYield::Instruction($(line_enum.any_qualified()).into()); );
				true
			},
			IDFlatLine::CustomCommand(IDCustomCommand { line_id, .. }) => {
				if !tokens.is_empty() {
					tokens.push();
				}
				
				let line_enum = LineEnum { 
					node_title, 
					raw_id: line_id.as_str(), 
					instruction_kind: InstructionKind::Command
				};
				
				quote_in!(*tokens => return YarnYield::Instruction($(line_enum.any_qualified()).into()); );
				true
			},
			IDFlatLine::BuiltInCommand(built_in_command) => {
				match built_in_command {
					BuiltInCommand::Set { 
						var_name, op, value,
						line_number: _, 
					} => {
						if !tokens.is_empty() {
							tokens.push();
						}
						
						match op {
							SetOperation::Assign => {
								quote_in!(*tokens => storage.set_var::<$var_name>($value););
							}
							SetOperation::Add => {
								let get_var = &YarnExpr::GetVar(var_name.to_string());
								quote_in!(*tokens => storage.set_var::<$var_name>($get_var + { $value }););
							}
							SetOperation::Sub => {
								let get_var = &YarnExpr::GetVar(var_name.to_string());
								quote_in!(*tokens => storage.set_var::<$var_name>($get_var - { $value }););
							}
							SetOperation::Mul => {
								let get_var = &YarnExpr::GetVar(var_name.to_string());
								quote_in!(*tokens => storage.set_var::<$var_name>($get_var * { $value }););
							}
							SetOperation::Div => {
								let get_var = &YarnExpr::GetVar(var_name.to_string());
								quote_in!(*tokens => storage.set_var::<$var_name>($get_var / { $value }););
							}
							SetOperation::Rem => {
								let get_var = &YarnExpr::GetVar(var_name.to_string());
								quote_in!(*tokens => storage.set_var::<$var_name>($get_var % { $value }););
							}
						}
						
						false
					},
					BuiltInCommand::Jump { node_destination_title, .. } => {
						if !tokens.is_empty() {
							tokens.push();
						}
						
						quote_in!(*tokens =>
							match $node_title.tracking() {
								TrackingSetting::Always => {
									storage.increment_visited(NodeTitle::$node_title);
								},
								TrackingSetting::Never => {},
							}
										
							return $node_destination_title.start(storage);
						);

						true
					},
					BuiltInCommand::Stop { .. } => {
						if !tokens.is_empty() {
							tokens.push();
						}
						
						quote_in!(*tokens => return YarnYield::Finished;);
						true
					},
				}
			},
		}
	}
}

impl NextFn for &IDOptionsFork {
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool {
		if !tokens.is_empty() {
			tokens.push();
		}
		
		let line_enum = LineEnum { 
			node_title, 
			raw_id: self.virtual_id.as_str(), 
			instruction_kind: InstructionKind::OptionsFork
		}; 
		
		quote_in!(*tokens => return YarnYield::Instruction($(line_enum.any_qualified()).into()););
		true
	}
}

impl NextFn for &IDIfBranch {
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool {
		if !tokens.is_empty() { 
			tokens.line();
		}
		
		let mut all_returned = true;

		let inside_if = 
			match &self.if_.1 {
				Some(scope) => {
					let mut if_tokens = Tokens::new();
					all_returned &= scope.quote_next_fn(&mut if_tokens, node_title);
					if_tokens
				},
				None => {
					all_returned = false;
					Tokens::new()
				},
			};
		
		quote_in!(*tokens => 
			if $(&self.if_.0.condition) {
				$inside_if
			}
		);
		tokens.append(" ");
		
		for (else_if, scope_option) in &self.else_ifs {
			let inside_else_if = 
				match scope_option {
					Some(scope) => {
						let mut else_if_tokens = Tokens::new();
						all_returned &= scope.quote_next_fn(&mut else_if_tokens, node_title);
						else_if_tokens
					},
					None => {
						all_returned = false;
						Tokens::new()
					},
				};
			
			quote_in!(*tokens => 
				else if $(&else_if.condition) {
					$inside_else_if
				}
			);
			tokens.append(" ");
		}
		
		if let Some((_, scope_option)) = &self.else_ {
			let inside_else = 
				match scope_option {
					Some(scope) => {
						let mut else_tokens = Tokens::new();
						all_returned &= scope.quote_next_fn(&mut else_tokens, node_title);
						else_tokens
					},
					None => {
						all_returned = false;
						Tokens::new()
					},
				};
			
			quote_in!(*tokens => 
				else {
					$inside_else
				}
			);
		} else {
			all_returned = false;
		}
		
		all_returned
	}
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
	
	false
}


impl NextFn for &IDScope {
	fn quote_next_fn(self, tokens: &mut Tokens, node_title: &str) -> bool {
		flows_next_fn(&self.flows, tokens, node_title)
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

	if !tokens.is_empty() {
		tokens.push();
	}
	
	quote_in!(tokens => 
		return YarnYield::Finished;
	);
	
	tokens
}
