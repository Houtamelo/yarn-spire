#[cfg(test)]
mod tests;

use std::any::type_name;
use std::iter::Peekable;
use std::str::Chars;
use anyhow::{anyhow, Result};
use expressions::parse_yarn_expr;
use crate::{expressions, LineNumber};
use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::macros::{return_if_err, starts_with_any, strip_start_then_trim};
use crate::parsing::raw::{ParseRawYarn, Content};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YarnCommand {
	pub line_number: LineNumber,
	pub variant: CommandVariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandVariant {
	Set { var_name: String, arg: YarnExpr },
	Jump { node_name: String },
	Stop,
	Other { variant: String, args: Vec<YarnExpr> },
}

struct ArgBuilder<'a> {
	chars: Peekable<Chars<'a>>,
	allow_yarn_set_syntax: bool,
}

enum CharState {
	Std,
	StringLiteral,
	StringLiteralIgnoreNext,
}

enum ArgState {
	Empty,
	Building {
		char_state: CharState,
		previous_char: char,
		nesting: Vec<char>,
		sum: String,
	},
}

type EndedWithDoubleAngleBracket = bool;

impl<'a> Iterator for ArgBuilder<'a> {
	type Item = Result<(String, EndedWithDoubleAngleBracket)>;

	fn next(&mut self) -> Option<Self::Item> {
		let chars = &mut self.chars;
		let mut state = ArgState::Empty;
		let mut ended_with_double_angle_bracket = false;
		while let Some(next) = chars.next() {
			match &mut state {
				ArgState::Empty => {
					match (next, self.allow_yarn_set_syntax) {
						('t', true) =>
							if chars.next_if_eq(&'o').is_some() {
								if matches!(chars.peek(), Some(' ' | '\t')) {
									chars.next();
									self.allow_yarn_set_syntax = false;
								} else {
									state = ArgState::Building {
										char_state: CharState::Std,
										previous_char: 'o',
										nesting: vec![],
										sum: String::from("to"),
									};
								}
							} else {
								state = ArgState::Building {
									char_state: CharState::Std,
									previous_char: 't',
									nesting: vec![],
									sum: String::from('t'),
								};
							},
						('=', true) => {
							self.allow_yarn_set_syntax = false;
						},
						(' ' | '\t' | ',', _) => {}, // ignore whitespace when empty
						('"', _) => {
							state = ArgState::Building {
								char_state: CharState::StringLiteral,
								previous_char: '"',
								nesting: vec![],
								sum: String::from('"'),
							};
						},
						(nest @ ('(' | '{' | '['), _) => {
							state = ArgState::Building {
								char_state: CharState::Std,
								previous_char: nest,
								nesting: vec![nest],
								sum: String::from(nest),
							};
						},
						(unexpected_nest @ (')' | '}' | ']'), _) => {
							return Some(Err(anyhow!(
								"Unexpected closing delimiter `{unexpected_nest}`\n\n\
								 Help: Closing delimiters must be preceded by a matching opening delimiter."
							)));
						},
						('>', _) => {
							if chars.next_if_eq(&'>').is_some() {
								let remaining = chars.by_ref().collect::<String>();
								let remaining_str = remaining.trim();

								return if remaining_str.is_empty() {
									None
								} else {
									Some(Err(anyhow!(
										"Unexpected characters after closing command(`>>`).\n\
										 Remaining: `{remaining_str}`\n\n\
										 Help: Extra characters are not allowed after `>>`.\n\
										 Help: Command statements cannot have metadata (which is started with `#`)"
									)))
								};
							} else {
								state = ArgState::Building {
									char_state: CharState::Std,
									previous_char: '>',
									nesting: vec![],
									sum: String::from('>'),
								};
							}
						},
						(other, _) => {
							state = ArgState::Building {
								char_state: CharState::Std,
								previous_char: other,
								nesting: vec![],
								sum: String::from(other),
							};
						},
					}
				},
				ArgState::Building {
					char_state: char_state @ CharState::Std,
					previous_char, nesting, sum
				} => {
					match next {
						'"' => {
							*char_state = CharState::StringLiteral;
							*previous_char = '"';
							sum.push('"');
						},
						nest @ ('(' | '{' | '[') => {
							*previous_char = nest;
							nesting.push(nest);
							sum.push(nest);
						},
						un_nest @ (')' | '}' | ']') => {
							if let Some(nest) = nesting.pop() {
								if matches!((nest, un_nest), ('(', ')') | ('{', '}') | ('[', ']')) {
									*previous_char = un_nest;
									sum.push(un_nest);
								} else {
									return Some(Err(anyhow!(
										"Invalid closing delimiter `{un_nest}` when parsing argument.\n\
										 Argument: `{sum}`\n\
										 Nesting: `{nesting:?}`\n\
										 Built so far: `{sum}`\n\
										 Help: the closing delimiter `{un_nest}` does not match the most-recent opening delimiter `{nest}`.\n\
										 Help: if you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."
									)));
								}
							} else {
								return Some(Err(anyhow!(
									"Unexpected closing delimiter `{un_nest}` when parsing argument.\n\
									 Argument: `{sum}`\n\
									 Nesting: `{nesting:?}`\n\
									 Built so far: {sum}\n\
									 Help: if you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."
								)));
							}
						},
						'>' => {
							if !nesting.is_empty()
								|| chars.next_if_eq(&'>').is_none() {
								*previous_char = '>';
								sum.push('>');
							}

							let remaining = chars.by_ref().collect::<String>();
							let remaining_str = remaining.trim();

							if remaining_str.is_empty() {
								ended_with_double_angle_bracket = true;
								break;
							} else {
								return Some(Err(anyhow!(
									"Unexpected characters after closing command(`>>`).\n\
									 Remaining: `{remaining_str}`\n\
									 Built so far: `{sum}`\n\n\
									 Help: Extra characters are not allowed after `>>`.\n\
									 Help: Branch statements cannot have metadata (which is started with `#`)"
								)));
							}
						},
						' ' | '\t' => {
							if !nesting.is_empty() { //whitespace is ignored inside nesting
								continue;
							}

							// skip whitespace
							while chars.next_if(|ch| matches!(*ch, ' ' | 't')).is_some() {}

							if false == matches!(*previous_char, '+' | '-' | '/' | '*' | '%' | '>' | '<' | '!' | '=')
								|| chars.peek().is_none() {
								break;
							}
						},
						',' => {
							if nesting.is_empty() {
								break;
							}
						},
						other => {
							*previous_char = other;
							sum.push(other);
						},
					}
				},
				ArgState::Building {
					char_state: char_state @ CharState::StringLiteral,
					previous_char, sum, nesting: _nesting
				} => {
					match next {
						'"' => {
							*char_state = CharState::Std;
							*previous_char = '"';
							sum.push('"');
						},
						'\\' => {
							*char_state = CharState::StringLiteralIgnoreNext;
						},
						other => {
							*previous_char = other;
							sum.push(other);
						},
					}
				},
				ArgState::Building {
					char_state: char_state @ CharState::StringLiteralIgnoreNext,
					previous_char, sum, nesting: _nesting
				} => {
					*char_state = CharState::StringLiteral;
					*previous_char = next;
					sum.push(next);
				},
			}
		}
		
		match state {
			ArgState::Building { char_state, previous_char: _previous_char, nesting, sum } => {
				if !nesting.is_empty() {
					return Some(Err(anyhow!(
						"Argument ended without closing previous nesting.\n\
						 Argument: `{sum:?}`\n\
						 Nesting: `{nesting:?}`\n\n\
						 Help: The argument `{sum}` is not closed.\n\
				         Help: For every opening delimiter(`(`, `{{`, `[`), there must be a matching closing delimiter(`)`, `}}`, `]`).\n\
				         Help: If you want to use `{{` or `}}` inside a string literal, escape it with a backslash (`\\`)."
					)));
				}
				
				if sum.len() == 0 {
					return None;
				}
				
				return match char_state {
					CharState::Std => 
						Some(Ok((sum, ended_with_double_angle_bracket))),
					| CharState::StringLiteral 
					| CharState::StringLiteralIgnoreNext =>
						Some(Err(anyhow!(
							"Argument ended without closing string literal\n\
							 Argument: `{sum}`\n\
							 Nesting: `{nesting:?}`\n\n\
							 Help: every string literal(started with `\"`) must end with another `\"`."
						))),
				};
			}
			ArgState::Empty => return None,
		}
	}
}

fn parse_raw_rust_ident(args_iter: &mut ArgBuilder) -> Result<(String, EndedWithDoubleAngleBracket)> {
	let (unparsed_string, ended_at_name) =
		args_iter
			.next()
			.unwrap_or_else(|| Err(anyhow!(
				"Expected `Rust identifier` argument but `Argument Builder` returned None."))
			)?;

	let first_char =
		unparsed_string
			.chars()
			.next()
			.ok_or(anyhow!(
				"Argument iterator returned empty string, this should never happen.\n\
				 Help: Please send an e-mail to Houtamelo: `houtamelo@pm.me`")
			)?;
	

	if !first_char.is_alphabetic() 
		&& first_char != '_' {
		return Err(anyhow!(
			"Invalid first character `{first_char}` in `Rust identifier` argument.\n\
			 Unparsed: `{unparsed_string}`\n\n\
			 Help: Rust identifiers must start with a letter or an underscore."));
	}

	return if let Some(invalid_char) =
		unparsed_string
			.chars()
			.find(|ch| 
				!ch.is_ascii_alphanumeric() && *ch != '_') {
		Err(anyhow!(
			"Invalid character `{invalid_char}` in `Rust identifier` argument.\n\
			 Command name: `{unparsed_string}`\n\n\
			 Help: Rust identifiers can only contain letters, numbers and underscores."))
	} else {
		Ok((unparsed_string, ended_at_name))
	};
}

fn parse_set_command(args_iter: &mut ArgBuilder, ended_at_name: EndedWithDoubleAngleBracket)
                     -> Result<(String, YarnExpr)> {
	if ended_at_name {
		return Err(anyhow!(
			"Ended with `>>` without any arguments.\n\
			 Expected two: variable name then value."));
	}

	let var_name = {
		let unparsed_string =
			args_iter
				.next()
				.unwrap_or_else(|| Err(anyhow!(
					"Could not build `variable name` argument.\n\
				     Error: `ArgBuilder` returned None.")))
				.and_then(|(var_name, ended)|
					if !ended {
						Ok(var_name)
					} else {
						Err(anyhow!(
							"Ended with `>>` after only one argument.\n\
							 Expected two: variable name then value.\n\
							 Variable name: {var_name}"))
					}
				)?;

		let expr =
			parse_yarn_expr(&unparsed_string)
				.map_err(|err| anyhow!(
					"Could not parse `variable name` argument as `YarnExpr`.\n\
					 Error: {err}\n\
					 Variable name: {unparsed_string}")
				)?;

		let YarnExpr::GetVar(name) = expr
			else {
				return Err(anyhow!(
					"Expected `variable name` argument to be `YarnExpr::VarGet(var_name)`.\n\
					 Instead got: `{expr:?}`\n\n\
					 Help: `variable names` follow the syntax: `$var_name_here`, did you include the `$`?"));
			};
		
		name
	};

	let value_expr = {
		args_iter.allow_yarn_set_syntax = true;

		let unparsed_string =
			args_iter
				.next()
				.unwrap_or_else(|| Err(anyhow!(
					"Could not build `variable value` argument.\n\
					 Error: `ArgBuilder` returned None.
		             Variable name: {var_name:?}")))
				.and_then(|(unparsed_value, ended)|
					if !ended {
						Err(anyhow!(
							"Did not end with `>>` after both arguments (variable name and value).\n\
			                 Variable name: {var_name:?}"
						))
					} else {    
						Ok(unparsed_value) 
					})?;

		let expr =
			parse_yarn_expr(&unparsed_string)
				.map_err(|err| anyhow!(
					"Could not parse `variable value` argument as a `YarnExpr`.\n\
					 Error: `{err}`\n\
					 Variable name: `{var_name:?}`"
				))?;

		args_iter.allow_yarn_set_syntax = false;
		
		expr
	};

	if let Some(invalid_next) = args_iter.next() {
		return Err(anyhow!(
			"Found more than two arguments.\n\
			 Expected only variable name then value.\n\
			 Variable name: `{var_name:?}`\n\
			 Value: `{value_expr:?}`\n\
			 Extra argument: `{invalid_next:?}`"));
	}

	return Ok((var_name, value_expr));
}

fn parse_jump_command(args_iter: &mut ArgBuilder, ended_at_name: EndedWithDoubleAngleBracket)
                      -> Result<String> {
	if ended_at_name {
		return Err(anyhow!(
			"Ended with `>>` without any arguments.\n\
			 Expected one: `node name`."));
	}
	
	let node_name =
		parse_raw_rust_ident(args_iter)
			.map_err(|err| anyhow!(
					"Could not parse `node name` argument.\n\
					 Error: `{err}`"))
			.and_then(|(name, ended)|
				if ended {
					Ok(name)
				} else {
					Err(anyhow!(
						"Expected one argument (`node name`) but declaration did not end(with `>>`) after the first argument.\n\n\
						 Help: `<<jump [node_name]>>` commands can only have one argument: `node name`.\n\
						 Help: Maybe you included whitespace in the Node name? That is not allowed."))
				}
			)?;
	
	return if let Some(invalid_next) = args_iter.next() {
		Err(anyhow!(
			"Found more than one argument.\n\
			 Expected only `node name`.\n\
			 Node name: `{node_name}`\n\
			 Extra argument: `{invalid_next:?}`"))
	} else {
		Ok(node_name)
	};
}

fn parse_other_command(args_iter: &mut ArgBuilder, command_name: String)
                       -> Result<(String, Vec<YarnExpr>)> {
	let unparsed_args: Vec<(String, EndedWithDoubleAngleBracket)> =
		args_iter
			.try_collect()
			.map_err(|err| anyhow!(
				"Could not build argument list.\n\
				 Command name: `{command_name}`\n\
				 Error: `{err}`")
			)?;
	
	if unparsed_args
		.last()
		.is_some_and(|(_, ended)|
			*ended == false) {
		return Err(anyhow!(
			"Expected `>>` after last argument.\n\
			 Command name: `{command_name}`"));
	}

	let args: Vec<YarnExpr> =
		unparsed_args
			.into_iter()
			.map(|(unparsed_str, _)| {
				parse_yarn_expr(&unparsed_str)
					.map_err(|err| anyhow!(
						"Could not parse an argument as `YarnExpr`.\n\
						 Argument TokenStream: `{unparsed_str:?}`\n\
					     Command name: `{command_name}`\n\
					     Error: `{err}`")) 
			}).try_collect()?;

	Ok((command_name, args))
}

impl ParseRawYarn for YarnCommand {
	fn parse_raw_yarn(line: &str, line_number: LineNumber)
	                  -> Option<Result<Content>> {
		let mut line = line.trim();
		
		if !strip_start_then_trim!(line, "<<") {
			return None;
		}
		
		if starts_with_any!(line, "if" | "elseif" | "else" | "endif" | "declare") {
			return None;
		}

		let mut args_iter = 
			ArgBuilder { 
				chars: line.chars().peekable(),
				allow_yarn_set_syntax: false, 
			};
		
		let (command_name, ended_at_name) = 
			return_if_err!(parse_raw_rust_ident(&mut args_iter)
				.map_err(|err| anyhow!(
					"Could not parse `command name`.\n\
					 Remaining Line: {}.\n\
					 Error: `{err}`"
					, args_iter.chars.clone().collect::<String>()
				)));
		
		return match command_name.as_str() {
			"set" => {
				match parse_set_command(&mut args_iter, ended_at_name) {
					Ok((var_name, expr)) => Some(Ok(Content::Command(YarnCommand {
						line_number,
						variant: CommandVariant::Set { var_name, arg: expr }
					}))),
					Err(err) => Some(Err(anyhow!(
						"Could not parse line as `set` command(`<<set $var_name (to) [value]>>`).\n\
					     Remaining Line: {}.\n\
						 Error: `{err}`"
						, args_iter.chars.collect::<String>()
					)))
				}
			},
			"jump" => {
				match parse_jump_command(&mut args_iter, ended_at_name) {
					Ok(node_name) => Some(Ok(Content::Command(YarnCommand {
						line_number,
						variant: CommandVariant::Jump { node_name }
					}))),
					Err(err) => Some(Err(anyhow!(
						"Could not parse line as `jump` command(`<<jump [NodeName]>>`).\n\
					     Remaining Line: {}.\n\
						 Error: `{err}`"
						, args_iter.chars.collect::<String>()
					)))
				}
			},
			"stop" => {
				if ended_at_name {
					Some(Ok(Content::Command(YarnCommand {
						line_number,
						variant: CommandVariant::Stop
					})))
				} else {
					Some(Err(anyhow!(
						"Could not parse line as `stop` command(`<<stop>>`).\n\
						 Remaining Line: `{}`\n\
						 Error: Expected no arguments, but line did not end after `stop`."
						, args_iter.chars.collect::<String>()
					)))
				}
			},
			_ => {
				match parse_other_command(&mut args_iter, command_name) {
					Ok((variant, args)) => Some(Ok(Content::Command(YarnCommand {
						line_number,
						variant: CommandVariant::Other { variant, args }
					}))),
					Err(err) => Some(Err(anyhow!(
						"Could not parse line as `other`(not `set`, `jump` or `stop`) command(`{}`).\n\
					     Remaining Line: `{}`.\n\
						 Error: `{err}`", type_name::<YarnCommand>()
						, args_iter.chars.collect::<String>()
					)))
				}
			},
		};
	}
}
