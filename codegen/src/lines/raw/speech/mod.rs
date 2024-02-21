#[cfg(test)] mod tests;

use std::any::type_name;
use std::iter::Peekable;
use std::str::{Chars, FromStr};
use anyhow::{Result, anyhow};
use houtamelo_utils::prelude::None;
use proc_macro2::TokenStream;
use expressions::parse_expr_from_tokens;
use crate::{expressions, LineNumber};
use crate::expressions::yarn_expr::YarnExpr;
use crate::lines::macros::{return_if_err, starts_with_any};
use crate::lines::raw::{ParseRawYarn, Content};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Speaker {
	Literal(String),
	Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speech {
	pub line_number: LineNumber,
	pub speaker: Option<Speaker>,
	pub line: (String, Vec<YarnExpr>),
	pub metadata: Option<String>,
}

enum CharState {
	Std,
	StringLiteral,
	StringLiteralIgnoreNext,
}

enum State {
	Lit {
		ignore_next: bool,
	},
	Arg {
		char_state: CharState,
		previous_char: char,
		nesting: Vec<char>,
		sum: String,
	},
}

fn parse_line(chars: &mut Peekable<Chars>, line_number: LineNumber) -> Result<Speech> {
	let mut state = State::Lit { ignore_next: false };
	let mut speaker = None;
	let mut literal = String::new();
	let mut args: Vec<String> = vec![];
	let mut metadata = None;

	while let Some(next) = chars.next() {
		match &mut state {
			State::Lit { ignore_next: ignore_next @ true } => {
				*ignore_next = false;
				literal.push(next);
			},
			State::Lit { ignore_next: ignore_next @ false } => {
				match next {
					'\\' => {
						*ignore_next = true;
					},
					'{' => {
						state = State::Arg {
							char_state: CharState::Std,
							previous_char: next,
							nesting: vec!['{'],
							sum: String::from('{'),
						};

						literal.push_str("{}");
					},
					'#' => {
						let built_metadata = chars.collect::<String>();
						if built_metadata.len() > 0 {
							metadata = Some(built_metadata)
						}

						break;
					},
					':' => {
						if speaker.is_none()
							&& literal.len() > 0
							&& literal.chars().none(char::is_whitespace)
							&& args.is_empty() {
							speaker = Some(Speaker::Literal(literal));
							literal = String::new();
						} else if speaker.is_none()
							&& literal.is_empty()
							&& args.len() == 1 {
							
							let unparsed_speaker = args.remove(0);

							let tokens =
								TokenStream::from_str(unparsed_speaker.as_str())
									.map_err(|err| anyhow!(
										"Could not tokenize `speaker variable`.\n\
										 Error: `{err:?}`\n\
										 Unparsed: `{unparsed_speaker}`\n\
										 Built so far: \n\
										 \tLiteral: `{literal}`\n\
										 \tArguments: `{args:?}`\n"
									))?;
							
							let debug_stream = tokens.to_string();
							let expr =
								parse_expr_from_tokens(tokens)
									.map_err(|err| anyhow!(
										"Could not parse `speaker variable` as `YarnExpr`.\n\
										 Error: {err:?}\n\
										 Unparsed: `{unparsed_speaker}`\n\
										 Built so far: \n\
										 \tLiteral: `{literal}`\n\
										 \tArguments: `{args:?}`\n"
									))?;

							let YarnExpr::VarGet(speaker_var_name) = expr
								else {
									return Err(anyhow!(
										"Invalid `speaker variable` argument.\n\
										 Expected expression of type `YarnExpr::VarGet(var_name)`,\
										 Got: {expr:?}\n\
										 Unparsed: `{unparsed_speaker}`\n\
										 TokenStream: `{debug_stream:?}`\n\
										 Built so far: \n\
										 \tLiteral: `{literal}`\n\
										 \tArguments: `{args:?}`\n\
										 \n\
										 Help: the speaker argument can only be a string literal(`John`) or a variable name(`{{$variable_name}}`)."
									));
								};

							speaker = Some(Speaker::Variable(speaker_var_name));
						} else {
							literal.push(next);
						}
					},
					other => {
						literal.push(other);
					},
				}
			},
			State::Arg {
				char_state: char_state @ CharState::Std, previous_char, 
				nesting, sum
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
					un_nest @ (')' | '}' | ']') =>
						if let Some(nest) = nesting.pop() {
							if matches!((nest, un_nest), ('(', ')') | ('{', '}') | ('[', ']')) {
								*previous_char = un_nest;
								sum.push(un_nest);
							} else {
								return Err(anyhow!(
									"Invalid closing delimiter `{un_nest}` when parsing argument.\n\
									 Argument: `{sum}`\n\
									 Nesting: `{nesting:?}`\n\
									 Built so far: \n\
									 \tLiteral: `{literal}`\n\
									 \tArguments: `{args:?}`\n\
									 \n\
									 Help: the closing delimiter `{un_nest}` does not match the most-recent opening delimiter `{nest}`.\n\
									 Help: if you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."
								));
							}
						} else if un_nest == '}' {
							sum.push('}');
							args.push(std::mem::take(sum));
							state = State::Lit { ignore_next: false };
						} else {
							return Err(anyhow!(
								"Unexpected closing delimiter `{un_nest}` when parsing argument.\n\
								 Argument: `{sum}`\n\
								 Nesting: `{nesting:?}`\n\
								 Built so far: \n\
								 \tLiteral: `{literal}`\n\
								 \tArguments: `{args:?}`\n\
								 \n\
								 Help: if you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."
							));
						},
					other => {
						*previous_char = other;
						sum.push(other);
					},
				}
			},
			State::Arg {
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
			State::Arg {
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
		State::Lit { ignore_next } => {
			if ignore_next {
				return Err(anyhow!(
					"Speech ended with an escape character (`\\`).\n\
					 Built so far: \n\
					 \tLiteral: `{literal}`\n\
					 \tArguments: `{args:?}`\n\n\
					 Help: The escape character(`\\`) means nothing if there's no character after it."
				));
			}
		},
		State::Arg { char_state: _char_state, previous_char: _previous_char,
			nesting, sum } => {
			return Err(anyhow!(
				"Speech ended with an open delimiter (building an argument).\n\
				 Argument: `{sum}`\n\
				 Nesting: `{nesting:?}`\n\
				 Built so far: \n\
				 \tLiteral: `{literal}`\n\
				 \tArguments: `{args:?}`\n\n\
				 Help: The argument `{sum}` is not closed.\n\
				 Help: For every opening delimiter(`(`, `{{`, `[`), there must be a matching closing delimiter(`)`, `}}`, `]`).\n\
				 Help: If you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."
			));
		},
	}

	let args_expr =
		build_args(args.clone(), &literal, &speaker, &metadata)?;

	if literal.is_empty() 
		&& args_expr.is_empty() {
		return Err(anyhow!(
			"Both literal and arguments are empty.\n\
			 Built so far: \n\
			 \tLiteral: `{literal}`\n\
			 \tArguments: `{args:?}`\n"
		));
	}

	return Ok(Speech {
		line_number,
		speaker,
		line: (literal, args_expr),
		metadata,
	});
}

// Reference arguments are just for error messages.
fn build_args(unparsed_args: Vec<String>, literal: &String,
              speaker: &Option<Speaker>, metadata: &Option<String>)
              -> Result<Vec<YarnExpr>> {
	let tokens =
		unparsed_args
			.iter()
			.map(|unparsed_arg|
				TokenStream::from_str(unparsed_arg.as_str())
					.map_err(|err| anyhow!(
						"Could not tokenize argument.\n\
				         Error: `{err:?}`\n\
				         Unparsed Argument: `{unparsed_arg}`\n\
				         All Unparsed Arguments: `{unparsed_args:?}`\n\
				         Speaker: `{speaker:?}`\n\
				         Literal: `{literal}`\n\
				         Metadata: `{metadata:?}`"
					))
			).collect::<Result<Vec<TokenStream>>>()?;

	let exprs =
		tokens
			.iter()
			.map(|token_stream|
				parse_expr_from_tokens(token_stream.clone())
					.map_err(|err| anyhow!(
						"Could not parse argument as `YarnExpr`.\n\
				         Error: `{err:?}`\n\
				         TokenStream: `{token_stream:?}`\n\
				         All Unparsed Arguments: `{unparsed_args:?}`\n\
						 Speaker: `{speaker:?}`\n\
				         Literal: `{literal}`\n\
				         Metadata: `{metadata:?}`"
					))
			).collect::<Result<Vec<YarnExpr>>>()?;

	return Ok(exprs);
}

impl ParseRawYarn for Speech {
	fn parse_raw_yarn(line: &str, line_number: LineNumber) -> Option<Result<Content>> {
		let line = line.trim();
		
		if starts_with_any!(line, "<<" | "->") {
			return None;
		}
		
		let mut chars = 
			line.chars()
				.peekable();
		
		let speech = 
			return_if_err!(parse_line(&mut chars, line_number)
				.map_err(|err|anyhow!(
					"Could not parse line as `{}`.\n\
					 Remaining line: `{}`\n\
					 Error: `{err:?}`\n\
					 ", type_name::<Speech>(), chars.collect::<String>()
				)));
		
		return Some(Ok(Content::Speech(speech)));
	}
}