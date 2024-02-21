#[cfg(test)]
mod tests;

use std::any::type_name;
use std::iter::{Peekable};
use std::str::{Chars, FromStr};
use proc_macro2::TokenStream;
use anyhow::{anyhow, Result};
use expressions::parse_expr_from_tokens;
use crate::{expressions, LineNumber};
use crate::expressions::yarn_expr::YarnExpr;
use crate::lines::macros::{return_if_err, starts_with_any, strip_start_then_trim};
use crate::lines::raw::{ParseRawYarn, Content};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YarnCommand {
	Set   { line_number: LineNumber, var_name: String, arg: YarnExpr },
	Other { line_number: LineNumber, variant: String, args: Vec<YarnExpr> }
}

impl YarnCommand {
	pub fn line_number(&self) -> LineNumber {
		match self {
			YarnCommand::Set { line_number, .. } => *line_number,
			YarnCommand::Other { line_number, .. } => *line_number,
		}
	}
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
							if chars.next_if_eq(&'o')
							        .is_some_and(|_| matches!(chars.peek(), Some(' ' | '\t'))) {
								chars.next();
								self.allow_yarn_set_syntax = false;
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

fn parse_command_name(args_iter: &mut ArgBuilder)
                      -> Result<(String, EndedWithDoubleAngleBracket)> {
	let (unparsed_string, ended_at_name) =
		args_iter.next()
		         .unwrap_or_else(|| Err(anyhow!(
			         "`ArgBuilder` returned None."
		         )))?;

	let first_char =
		unparsed_string
			.chars()
			.next()
			.unwrap();

	if !first_char.is_alphabetic() && first_char != '_' {
		return Err(anyhow!(
			"Invalid first character `{first_char}` in `command name` argument.\n\
			 Command name: `{unparsed_string}`\n\n\
			 Help: Command names must start with a letter or an underscore."
		));
	}

	if let Some(invalid_char) =
		unparsed_string.chars()
		               .find(|ch| !ch.is_ascii_alphanumeric() && *ch != '_') {
		return Err(anyhow!(
			"Invalid character `{invalid_char}` in `command name` argument.\n\
			 Command name: `{unparsed_string}`\n\n\
			 Help: Command names can only contain letters, numbers and underscores."
		));
	}

	Ok((unparsed_string, ended_at_name))
}

fn parse_set_command(args_iter: &mut ArgBuilder, ended_at_name: EndedWithDoubleAngleBracket,
                     line_number: LineNumber) -> Result<YarnCommand> {
	if ended_at_name {
		return Err(anyhow!(
			"Ended with `>>` without any arguments.\n\
			 Expected two: variable name then value."
		));
	}

	let var_name_expr = {
		let build_result =
			args_iter
				.next()
				.unwrap_or_else(|| Err(anyhow!(
					"Could not build `variable name` argument.\n\
				     Error: `ArgBuilder` returned None."
				)));

		let unparsed_string =
			match build_result {
				Ok((var_name, ended)) => {
					if ended {
						return Err(anyhow!(
							"Ended with `>>` after only one argument.\n\
							 Expected two: variable name then value.\n\
							 Variable name: {var_name}"
						));
					}

					var_name
				},
				Err(err) => {
					return Err(anyhow!(
						"Could not build `variable name` argument.\n\
						 Error: {err}"
					));
				},
			};

		let tokens =
			match TokenStream::from_str(unparsed_string.as_str()) {
				Ok(tokens) => tokens,
				Err(err) => {
					return Err(anyhow!(
						"Could not tokenize `variable name` argument.\n\
						 Error: {err}\n\
						 Variable name: {unparsed_string}"
					));
				}
			};

		let expr =
			match parse_expr_from_tokens(tokens.clone()) {
				Ok(expr) => expr,
				Err(err) => {
					return Err(anyhow!(
						"Could not parse `variable name` argument as `YarnExpr`.\n\
						 Error: {err}\n\
						 Variable name: {unparsed_string}\n\
						 TokenStream: {tokens:?}"
					));
				}
			};

		expr
	};

	let value_expr = {
		args_iter.allow_yarn_set_syntax = true;

		let build_result =
			args_iter
				.next()
				.unwrap_or_else(|| Err(anyhow!(
					"Could not build `variable value` argument.\n\
					 Error: `ArgBuilder` returned None.
		             Variable name: {var_name_expr:?}"
				)));

		let unparsed_string =
			match build_result {
				Ok((unparsed_string, ended)) => {
					if !ended {
						return Err(anyhow!(
							"Did not end with `>>` after both arguments (variable name and value).\n\
			                 Variable name: {var_name_expr:?}"
						));
					}

					unparsed_string
				},
				Err(err) => {
					return Err(anyhow!(
						"Could not build `variable value` argument.\n\
						 Error: {err}\n\
						 Variable name: {var_name_expr:?}"
					));
				},
			};

		let tokens =
			match TokenStream::from_str(unparsed_string.as_str()) {
				Ok(tokens) => tokens,
				Err(err) => {
					return Err(anyhow!(
						"Could not tokenize `variable value` argument.\n\
						 Error: `{err}`\n\
						 Variable name: `{var_name_expr:?}`"
					));
				}
			};

		let expr =
			match parse_expr_from_tokens(tokens.clone()) {
				Ok(expr) => expr,
				Err(err) => {
					return Err(anyhow!(
						"Could not parse `variable value` argument as a `YarnExpr`.\n\
						 Error: `{err}`\n\
						 Variable name: `{var_name_expr:?}`\n\
						 TokenStream: `{tokens:?}`"
					));
				}
			};

		args_iter.allow_yarn_set_syntax = false;

		expr
	};

	if let Some(invalid_next) = args_iter.next() {
		return Err(anyhow!(
			"Found more than two arguments.\n\
			 Expected only variable name then value.\n\
			 Variable name: `{var_name_expr:?}`\n\
			 Value: `{value_expr:?}`\n\
			 Extra argument: `{invalid_next:?}`"
		));
	}

	return if let YarnExpr::VarGet(var_name) = var_name_expr {
		Ok(YarnCommand::Set {
			line_number,
			var_name,
			arg: value_expr,
		})
	} else {
		Err(anyhow!(
			"Expected `variable name` argument to be `YarnExpr::VarGet(var_name)`.\n\
			 Instead got: `{var_name_expr:?}`\n\
			 Value: `{value_expr:?}`"
		))
	};
}

fn parse_other_command(args_iter: &mut ArgBuilder, command_name: String, 
                       line_number: LineNumber) -> Result<YarnCommand> {
	let unparsed_args = {
		let mut temp = Vec::new();
		for result in args_iter {
			match result {
				Ok((unparsed_arg, _)) => {
					temp.push(unparsed_arg);
				},
				Err(err) => {
					return Err(anyhow!(
						"Could not build argument list.\n\
						 Command name: `{command_name}`\n\
						 Error: `{err}`\n\
						 Successfully built arguments: `{temp:?}`"
					));
				},
			}
		}

		temp
	};

	let tokens_list =
		unparsed_args
			.into_iter()
			.map(|unparsed|
				TokenStream::from_str(unparsed.as_str())
					.map_err(|err|anyhow!(
						"Could not tokenize an argument.\n\
						 Unparsed Argument: `{unparsed}`\n\
						 Command name: `{command_name}`\n\
						 Error: `{err}`"
					))
			).collect::<Result<Vec<TokenStream>>>()?;

	let args =
		tokens_list
			.into_iter()
			.map(|token_stream| {
				let debug_stream = token_stream.to_string();
				parse_expr_from_tokens(token_stream)
					.map_err(|err| anyhow!(
					"Could not parse an argument as `YarnExpr`.\n\
					 Argument TokenStream: `{debug_stream:?}`\n\
				     Command name: `{command_name}`\n\
				     Error: `{err}`"
				))
			}).collect::<Result<Vec<YarnExpr>>>()?;

	Ok(YarnCommand::Other {
		line_number,
		variant: command_name,
		args,
	})
}

impl ParseRawYarn for YarnCommand {
	fn parse_raw_yarn(line: &str, line_number: LineNumber)
	                  -> Option<Result<Content>> {
		let mut line = line.trim();
		
		if !strip_start_then_trim!(line, "<<") {
			return None;
		}
		
		if starts_with_any!(line, "if" | "elseif" | "else" | "endif") {
			return None;
		}

		let mut args_iter = 
			ArgBuilder { 
				chars: line.chars().peekable(),
				allow_yarn_set_syntax: false,
			};
		
		let (command_name, ended_at_name) = 
			return_if_err!(parse_command_name(&mut args_iter)
				.map_err(|err| anyhow!(
					"Could not parse `command name` in `{}`.\n\
					 Remaining Line: {}.\n\
					 Error: `{err}`", type_name::<YarnCommand>(), args_iter.chars.clone().collect::<String>()
				)));

		return if command_name == "set" {
			match parse_set_command(&mut args_iter, ended_at_name, line_number) {
				Ok(ok) => Some(Ok(Content::Command(ok))),
				Err(err) => Some(Err(anyhow!(
					"Could not parse line as `set` command(`{}`).\n\
				     Remaining Line: {}.\n\
					 Error: `{err}`", type_name::<YarnCommand>(), args_iter.chars.collect::<String>()
				)))
			}
		} else {
			match parse_other_command(&mut args_iter, command_name, line_number) {
				Ok(ok) => Some(Ok(Content::Command(ok))),
				Err(err) => Some(Err(anyhow!(
					"Could not parse line as `other`(not `set`) command(`{}`).\n\
				     Remaining Line: {}.\n\
					 Error: `{err}`", type_name::<YarnCommand>(), args_iter.chars.collect::<String>()
				)))
			}
		};
	}
}
