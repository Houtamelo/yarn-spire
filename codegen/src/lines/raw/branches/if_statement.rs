use std::iter::Peekable;
use std::str::{Chars, FromStr};
use proc_macro2::TokenStream;
use anyhow::{Result, anyhow};
use crate::expressions::parse_expr_from_tokens;
use crate::expressions::yarn_expr::YarnExpr;
use crate::LineNumber;
use crate::lines::macros::{return_if_err, strip_start, strip_start_then_trim, trim_start};
use crate::lines::raw::{ParseRawYarn, Content};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStruct {
	pub line_number: LineNumber,
	pub condition: YarnExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElseIfStruct {
	pub line_number: LineNumber,
	pub condition: YarnExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElseStruct {
	pub line_number: LineNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndIfStruct {
	pub line_number: LineNumber,
}

#[derive(Clone)]
struct ArgBuilder<'a> {
	chars: Peekable<Chars<'a>>,
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
					match next {
						' ' | '\t' | ',' => {}, // ignore whitespace when empty
						'"' => {
							state = ArgState::Building {
								char_state: CharState::StringLiteral,
								previous_char: '"',
								nesting: vec![],
								sum: String::from('"'),
							};
						},
						nest @ ('(' | '{' | '[') => {
							state = ArgState::Building {
								char_state: CharState::Std,
								previous_char: nest,
								nesting: vec![nest],
								sum: String::from(nest),
							};
						},
						unexpected_nest @ (')' | '}' | ']') => {
							return Some(Err(anyhow!(
								"Unexpected closing delimiter `{unexpected_nest}`\n\n\
								 Help: Closing delimiters must be preceded by a matching opening delimiter."
							)));
						},
						'>' => {
							if chars.next_if_eq(&'>').is_some() {
								let remaining = chars.by_ref().collect::<String>();
								let remaining_str = remaining.trim();

								return if remaining_str.is_empty() {
									None
								} else {
									Some(Err(anyhow!(
										"Unexpected characters after closing branch statement(`>>`).\n\
										 Remaining: `{remaining_str}`\n\n\
										 Help: Extra characters are not allowed after `>>`.\n\
										 Help: Branch statements cannot have metadata (which is started with `#`)"
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
						other => {
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
										 Built so far: `{sum}`\n\n\
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
									"Unexpected characters after closing branch statement(`>>`).\n\
									 Remaining: `{remaining_str}`\n\
									 Built so far: `{sum}`\n\n\
									 Help: Extra characters are not allowed after `>>`."
								)));
							}
						},
						' ' | '\t' => {
							if !nesting.is_empty() { //whitespace is ignored inside nesting
								continue;
							}

							// skip whitespace
							while chars.next_if(|ch| matches!(*ch, ' ' | 't')).is_some() {}
							
							if let Some('>') = chars.peek() {
								continue;
							}

							if false == matches!(*previous_char, '+' | '-' | '/' | '*' | '%' | '>' | '<' | '!' | '=') {
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
			ArgState::Building { char_state, previous_char: _previous_char, 
				nesting, sum } => {
				if !nesting.is_empty() {
					return Some(Err(anyhow!(
						"Argument ended without closing previous nesting.\n\
						 Argument: `{sum:?}`\n\
						 Nesting: `{nesting:?}`\
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

fn parse_args(unparsed_args: &[(String, EndedWithDoubleAngleBracket)])
              -> Result<Vec<(YarnExpr, EndedWithDoubleAngleBracket)>> {
	let tokens_list =
		unparsed_args
			.iter()
			.map(|(unparsed, ended_with_double_angle_bracket)|
				TokenStream::from_str(unparsed.as_str())
					.map(|tokens| (tokens, *ended_with_double_angle_bracket))
					.map_err(|err| anyhow!(
						"Could not tokenize argument.\n\
						 Unparsed Argument: `{unparsed:?}`\n\
						 Error: `{err}`"
					))
			).collect::<Result<Vec<(TokenStream, EndedWithDoubleAngleBracket)>>>()?;

	let args =
		tokens_list
			.into_iter()
			.map(|(token_stream, ended_with_double_angle_bracket)| {
				let debug_stream = token_stream.to_string();
				parse_expr_from_tokens(token_stream)
					.map(|tokens| (tokens, ended_with_double_angle_bracket))
					.map_err(|err| anyhow!(
						"Could not parse argument as `YarnExpr`.\n\
						 Argument TokenStream: `{debug_stream:?}`\n\
					     Error: `{err}`"
					))
			}).collect::<Result<Vec<(YarnExpr, EndedWithDoubleAngleBracket)>>>()?;

	return Ok(args);
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
			if strip_start!(line, "else") {
				BranchKind::Else
			} else if strip_start!(line, "endif") { 
				BranchKind::EndIf 
			} else if strip_start!(line, "if") {
				if line.chars().next().is_some_and(|c| c == ' ' || c == '\t') {
					BranchKind::If
				} else {
					return Some(Err(anyhow!(
						"Could not parse `if` statement\n\
						 Error: Expected `if` statement to be followed by whitespace(` ` or `\\t`).\n\
						 Instead Got: `{line}`\n\n\
						 Help: Place a space between `<<if` and `[condition]>>`, like so: `<<if [condition]>>`."
					)))
				}
			} else if strip_start!(line, "elseif") {
				if line.chars().next().is_some_and(|c| c == ' ' || c == '\t') {
					BranchKind::ElseIf
				} else {
					return Some(Err(anyhow!(
						"Could not parse `elseif` statement\n\
						 Error: Expected `elseif` statement to be followed by whitespace(` ` or `\\t`).\n\
						 Instead Got: `{line}`\n\n\
						 Help: Place a space between `<<elseif` and `[condition]>>`, like so: `<<elseif [condition]>>`."
					)))
				}
			}  else {
				return None;
			};

		let args_iter =
			ArgBuilder { chars: line.chars().peekable() };

		let unparsed_args =
			return_if_err!(args_iter.clone().collect::<Result<Vec<_>>>()
				.map_err(|err| anyhow!(
					"Could not stringify argument in `{branch_kind:?}` statement.\n\
					 Error: `{err:?}`\n\
					 Remaining Line: `{}`"
					, args_iter.chars.clone().collect::<String>()
				)));
		
		let args_expr =
			return_if_err!(parse_args(&unparsed_args)
				.map_err(|err| anyhow!(
					"Could not parse argument in `{branch_kind:?}` statement.\n\
					 Error: `{err:?}`\n\
					 Remaining line: `{}`", args_iter.chars.clone().collect::<String>()
				)));
		
		return match branch_kind {
			BranchKind::If => {
				if args_expr.len() != 1 {
					Some(Err(anyhow!(
						"Could not parse `if` statement\n\
						 Error: Expected exactly one argument.\n\
						 Instead Got: `{args_expr:?}`\n\n\
						 Help: `if` statements take exactly one argument.\n\
						 Help: If you wish to use multiple conditions, you can use `&&`, `||`, \
						 `and` (same as `&&`), `or`(same as `||`) to combine multiple conditions.\n\
						 Help: Anything inside delimiters( `{{ }}`, `[ ]`, `( )` ) is considered a single argument, regardless of what's inside."
					)))
				} else if let Some((arg, ended_with_double_bracket)) = args_expr.into_iter().next()
					&& ended_with_double_bracket {
					Some(Ok(Content::If(
						IfStruct { 
							line_number,
							condition: arg,
						})))
				} else {
					Some(Err(anyhow!(
						"Could not parse `if` statement\n\
						 Error: Expected argument to end with `>>`.\n\n\
						 Help: `<<if [condition]>>` statements must end with `>>`."
					)))
				}
			},
			BranchKind::ElseIf => {
				if args_expr.len() != 1 {
					Some(Err(anyhow!(
						"Could not parse `elseif` statement\n\
						 Error: Expected exactly one argument.\n\
						 Instead Got: `{args_expr:?}`\n\n\
						 Help: `elseif` statements take exactly one argument.\n\
						 Help: If you wish to use multiple conditions, you can use `&&`, `||`, \
						 `and` (same as `&&`), `or`(same as `||`) to combine multiple conditions.\n\
						 Help: Anything inside delimiters( `{{ }}`, `[ ]`, `( )` ) is considered a single argument, regardless of what's inside."
					)))
				} else if let Some((arg, ended_with_double_bracket)) = args_expr.into_iter().next()
					&& ended_with_double_bracket {
					Some(Ok(Content::ElseIf(
						ElseIfStruct {
							line_number,
							condition: arg,
						})))
				} else {
					Some(Err(anyhow!(
						"Could not parse `elseif` statement\n\
						 Error: Expected argument to end with `>>`.\n\n\
						 Help: `<<elseif [condition]>>` statements must end with `>>`."
					)))
				}
			},
			BranchKind::Else => {
				if args_expr.is_empty() {
					Some(Ok(Content::Else(ElseStruct { line_number })))
				} else {
					Some(Err(anyhow!(
						"Unexpected arguments after `else` statement.\n\
						 Arguments(unparsed): `{unparsed_args:?}`\n\
						 Arguments(expression): `{args_expr:?}`\n\n\
						 Help: `else` statements do not take any arguments."
					)))
				}
			},
			BranchKind::EndIf => {
				if args_expr.is_empty() {
					Some(Ok(Content::EndIf(EndIfStruct { line_number })))
				} else {
					Some(Err(anyhow!(
						"Unexpected arguments after `endif` statement.\n\
						 Arguments(unparsed): `{unparsed_args:?}`\n\
						 Arguments(expression): `{args_expr:?}`\n\n\
						 Help: `endif` statements do not take any arguments."
					)))
				}
			},
		};
	}
}