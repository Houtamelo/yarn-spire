use std::any::type_name;
use std::iter::Peekable;
use std::str::{Chars, FromStr};
use anyhow::{Result, anyhow};
use proc_macro2::TokenStream;
use expressions::parse_expr_from_tokens;
use crate::{expressions, LineNumber};
use crate::expressions::yarn_expr::YarnExpr;
use crate::lines::macros::{return_if_err, strip_start_then_trim};
use crate::lines::raw::{ParseRawYarn, Content};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceOption {
	pub line_number: LineNumber,
	pub line: (String, Vec<YarnExpr>),
	pub if_condition: Option<YarnExpr>,
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

fn parse_line(chars: &mut Peekable<Chars>, line_number: LineNumber) -> Result<ChoiceOption> {
	let mut state = State::Lit { ignore_next: false };
	let mut literal = String::new();
	let mut args: Vec<String> = vec![];
	let mut if_condition = None;
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
					'<' => {
						let Some('<') = chars.peek() 
							else {
								literal.push('<');
								continue;
							};
						
						let remaining = chars.clone().skip(1).collect::<String>();
						let mut remaining_str = remaining.as_str().trim();
						
						if strip_start_then_trim!(remaining_str, "if") { // if [condition]>>
							let remaining_chars = 
								remaining_str
									.chars()
									.peekable();
							
							let result = 
								parse_if_condition_and_metadata(remaining_chars)?;
							
							if_condition = Some(result.0);
							metadata = result.1;
							break;
						} else {
							return Err(anyhow!(
								"Invalid declaration: `<<` can only be followed by `if [condition]>>`.\n\
								 Help: In a choice option, the `if` condition must follow the pattern `<<if [condition]>>`,\
								  then optionally be followed by `#metadata here`"
							));
						}
					},
					'#' => {
						let built_metadata = chars.by_ref().collect::<String>();
						if built_metadata.len() > 0 {
							metadata = Some(built_metadata)
						}

						break;
					},
					other => {
						literal.push(other);
					},
				}
			},
			State::Arg {
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
								return Err(anyhow!(
									"Could not parse argument.\n\
									 Error: Invalid closing delimiter `{un_nest}`\n\
									 Argument: `{sum}`\n\
									 Nesting: `{nesting:?}`\n\
									 Built so far: \n\
									 \tLiteral: `{literal}`\n\
									 \tArguments: `{args:?}`\n\n\
									 Help: the closing delimiter `{un_nest}` does not match the most-recent opening delimiter `{nest}`.\n\
									 Help: if you want to use '{{', '}}' or '#' as a literal, escape it with a backslash (`\\`)."
								));
							}
						} else if un_nest == '}' {
							sum.push('}');
							args.push(std::mem::take(sum));
							state = State::Lit { ignore_next: false };
						} else {
							return Err(anyhow!(
								"Could not parse argument.\n\
								 Error: Unexpected closing delimiter `{un_nest}`\n\
								 Argument: `{sum}`\n\
								 Nesting: `{nesting:?}`\n\
								 Built so far: \n\
								 \tLiteral: `{literal}`\n\
								 \tArguments: `{args:?}`\n\
								 \n\
								 Remaining line: `{}`\n\n\
								 Help: if you want to use '{{', '}}' or '#' as a literal, escape it with a backslash (`\\`)."
								, chars.collect::<String>()
							)); 
						}
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

	let args_expr =
		build_args(args.clone(), &literal, &if_condition, &metadata)?;

	if literal.is_empty()
		&& args_expr.is_empty() {
		return Err(anyhow!(
			"Both literal and arguments are empty.\n\
			 Built so far: \n\
			 \tLiteral: `{literal}`\n\
			 \tArguments: `{args:?}`\n\
			 \tIf Condition: `{if_condition:?}`\n\
			 \tMetadata: `{metadata:?}`\n"
		));
	}

	return Ok(ChoiceOption {
		line_number,
		line: (literal, args_expr),
		if_condition,
		metadata,
	});
}

// Reference arguments are just for error messages.
fn build_args(unparsed_args: Vec<String>, literal: &String,
              if_condition: &Option<YarnExpr>, metadata: &Option<String>)
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
				         Literal: `{literal}`\n\
						 If Condition: `{if_condition:?}`\n\
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
				         Literal: `{literal}`\n\
						 If Condition: `{if_condition:?}`\n\
				         Metadata: `{metadata:?}`"
					))
			).collect::<Result<Vec<YarnExpr>>>()?;

	return Ok(exprs);
}

fn parse_if_condition_and_metadata(mut chars: Peekable<Chars>)
                                   -> Result<(YarnExpr, Option<String>)> {
	let mut char_state = CharState::Std;
	let mut nesting: Vec<char> = Vec::new();
	let mut sum = String::new();

	while let Some(next) = chars.next() {
		match char_state {
			CharState::Std => {
				match next {
					'"' => {
						char_state = CharState::StringLiteral;
						sum.push('"');
					},
					'>' => {
						if nesting.len() > 0
						|| chars.next_if_eq(&'>').is_none() {
							sum.push('>');
							continue;
						}
						
						if sum.is_empty() {
							return Err(anyhow!(
								"`if condition` delimiters(`<<` and `>>`) exist but argument is empty.\n\
								 Nesting: `{nesting:?}`"
							));
						}
						
						let remaining = chars.collect::<String>();
						let mut remaining_str = remaining.as_str().trim();
						let metadata = 
							if strip_start_then_trim!(remaining_str, '#')
								&& !remaining_str.is_empty() { 
								Some(remaining_str.to_owned())
							} else if remaining_str.is_empty() {
								None
							} else {
								let invalid_char = 
									remaining_str
										.chars()
										.next()
										.unwrap();
								
								return Err(anyhow!(
									"Invalid character `{invalid_char}` after `<<if [condition]>>` statement.\n\
									 Argument: `{sum}`\n\n\
									 Help: In a choice option, the `if` condition must follow the pattern `<<if [condition]>>`,\
									  then optionally be followed by `#metadata here`"
								));
							};
						
						let tokens =
							TokenStream::from_str(sum.as_str())
								.map_err(|err| anyhow!(
									"Could not tokenize `if condition`.\n\
									 Error: `{err:?}`\n\
									 Argument: `{sum}`"
								))?;
						
						let expr =
							parse_expr_from_tokens(tokens)
								.map_err(|err| anyhow!(
									"Could not parse `if condition` as `YarnExpr`.\n\
									 Error: `{err:?}`\n\
									 Argument: `{sum}`"
								))?;
						
						return Ok((expr, metadata));
					},
					nest @ ('(' | '{' | '[') => {
						nesting.push(nest);
						sum.push(nest);
					},
					un_nest @ (')' | '}' | ']') => {
						if let Some(nest) = nesting.pop() {
							if matches!((nest, un_nest), ('(', ')') | ('{', '}') | ('[', ']')) {
								sum.push(un_nest);
							} else {
								return Err(anyhow!(
									"Invalid closing delimiter `{un_nest}` in `if condition` argument.\n\
									 Argument: `{sum}`\n\
									 Nesting: `{nesting:?}`\n\n\
									 Help: the closing delimiter `{un_nest}` does not match the most-recent opening delimiter `{nest}`."
								));
							}
						} else {
							return Err(anyhow!(
								"Unexpected closing delimiter `{un_nest}` in `if condition` argument.\n\
								 Argument: `{sum}`\n\
								 Nesting: `{nesting:?}`\n\n\
								 Help: the characters ('{{', '(', '[') are treated as opening delimiters when inside arguments.\n\
								 Help: for every opening delimiter, there must be a matching closing delimiter (and vice-versa)."
							));
						}
					},
					other => {
						sum.push(other);
					},
				}
			},
			CharState::StringLiteral => {
				match next {
					'"' => {
						char_state = CharState::Std;
						sum.push('"');
					},
					'\\' => {
						char_state = CharState::StringLiteralIgnoreNext;
					},
					other => {
						sum.push(other);
					},
				}
			},
			CharState::StringLiteralIgnoreNext => {
				char_state = CharState::StringLiteral;
				sum.push(next);
			},
		}
	}
	
	if !nesting.is_empty() {
		return Err(anyhow!(
			"Unclosed delimiter(s) in `if condition` argument.\
			 Delimiter(s): {nesting:?}\n\
			 Argument: `{sum}`\n\n\
			 Help: the characters ('{{', '(', '[') are treated as opening delimiters when inside arguments.\n\
			 Help: for every opening delimiter, there must be a matching closing delimiter (and vice-versa)."
		));
	}
	
	return match char_state {
		CharState::Std => Err(anyhow!(
			"`if condition` argument is empty.\n\
			 Argument: `{sum}`"
		)),
		| CharState::StringLiteral 
		| CharState::StringLiteralIgnoreNext => {
			Err(anyhow!(
				"Unclosed string literal in `if condition` argument.\n\
				 Argument: `{sum}`"
			))
		},
	};
}

impl ParseRawYarn for ChoiceOption {
	fn parse_raw_yarn(line: &str, line_number: LineNumber) -> Option<Result<Content>> {
		let mut line = line.trim();

		if !strip_start_then_trim!(line, "->") {
			return None;
		}
		
		let mut chars = 
			line.chars()
				.peekable();

		let choice_option =
			return_if_err!(parse_line(&mut chars, line_number)
				.map_err(|err| anyhow!(
					"Could not parse line as `{}`.\n\
					 Error: `{err:?}`\n\
					 Remaining line: `{}`", type_name::<ChoiceOption>(), chars.collect::<String>() 
				)));
		
		return Some(Ok(Content::ChoiceOption(choice_option)));
	}
}