use std::iter::Peekable;
use std::str::Chars;
use anyhow::{Result, anyhow};
use trim_in_place::TrimInPlace;
use expressions::parse_yarn_expr;
use crate::{expressions, LineNumber};
use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::macros::{return_if_err, strip_start_then_trim};
use crate::parsing::raw::{ParseRawYarn, Content};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionLine {
	pub line_number: LineNumber,
	pub line_id: Option<String>,
	pub text: (String, Vec<YarnExpr>),
	pub if_condition: Option<YarnExpr>,
	pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndOptions {
	pub line_number: LineNumber,
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

fn parse_line(chars: &mut Peekable<Chars>, line_number: LineNumber) -> Result<OptionLine> {
	let mut state = State::Lit { ignore_next: false };
	let mut literal = String::new();
	let mut args: Vec<String> = vec![];
	let mut if_condition = None;
	let mut metadata_option = None;

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
							nesting: vec![],
							sum: String::new(),
						};

						literal.push_str("{}");
					},
					'}' => {
						return Err(anyhow!(
							"Unexpected closing delimiter `}}` when parsing literal.\n\
							 Built so far: \n\
							 \tLiteral: `{literal}`\n\
							 \tArguments: `{args:?}`\n\n\
							 Help: The closing delimiter `}}` does not match any opening delimiter `{{`.\n\
							 Help: If you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."));
					},
					'<' => {
						let Some('<') = chars.peek() 
							else {
								literal.push('<');
								continue;
							};

						let remaining =
							chars.clone()
							     .skip(1)
							     .collect::<String>();
						
						let mut remaining_str = 
							remaining.as_str().trim();
						
						if strip_start_then_trim!(remaining_str, "if") { // if [condition]>>
							let remaining_chars = 
								remaining_str
									.chars()
									.peekable();
							
							let result = 
								parse_if_condition_and_metadata(remaining_chars)?;
							
							if_condition = Some(result.0);
							metadata_option = result.1;
							break;
						} else {
							return Err(anyhow!(
								"Invalid declaration: `<<` can only be followed by `if [condition]>>`.\n\
								 Help: In a choice option, the `if` condition must follow the pattern `<<if [condition]>>`,\
								  then optionally be followed by `#metadata here`"));
						}
					},
					'#' => {
						let built_metadata =
							std::iter::once('#')
								.chain(chars.by_ref())
								.collect::<String>();

						if built_metadata.len() > 1 {
							metadata_option = Some(built_metadata)
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
									"Invalid closing delimiter `{un_nest}` when parsing argument.\n\
									 Argument: `{sum}`\n\
									 Nesting: `{nesting:?}`\n\
									 Built so far: \n\
									 \tLiteral: `{literal}`\n\
									 \tArguments: `{args:?}`\n\n\
									 Help: the closing delimiter `{un_nest}` does not match the most-recent opening delimiter `{nest}`.\n\
									 Help: if you want to use '{{', '}}' or '#' as a literal, escape it with a backslash (`\\`)."));
							}
						} else if un_nest == '}' {
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
								, chars.collect::<String>())); 
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
		build_args(args.clone())
			.map_err(|err| anyhow!(
				"Could not parse argument as `YarnExpr`.\n\
				 Error: `{err:?}`\n\
		         Literal: `{literal}`\n\
				 If Condition: `{if_condition:?}`\n\
		         Metadata: `{metadata_option:?}`")
			)?;
	
	literal.trim_in_place();

	if literal.is_empty()
	&& args_expr.is_empty() {
		return Err(anyhow!(
			"Both literal and arguments are empty.\n\
			 Built so far: \n\
			 \tLiteral: `{literal}`\n\
			 \tArguments: `{args:?}`\n\
			 \tIf Condition: `{if_condition:?}`\n\
			 \tMetadata: `{metadata_option:?}`\n"));
	}

	let Some(metadata) = &metadata_option
		else {
			return Ok(OptionLine {
				line_number,
				line_id: None,
				text: (literal, args_expr),
				if_condition,
				tags: vec![],
			});
		};

	let mut tags: Vec<String> =
		metadata
			.split('#')
			.filter_map(|tag| {
				let trimmed = tag.trim();
				if trimmed.is_empty() {
					None
				} else {
					Some(trimmed.to_string())
				}
			}).collect();

	let line_id: Vec<String> =
		tags.extract_if(|tag| {
			let mut temp = tag.as_str();
			if strip_start_then_trim!(temp, "line")
			&& strip_start_then_trim!(temp, ":") {
				*tag = temp.to_string();
				true
			} else {
				false
			}
		}).collect();

	return match line_id.len() {
		0 => Ok(OptionLine {
			line_number,
			line_id: None,
			text: (literal, args_expr),
			if_condition,
			tags,
		}),
		1 => Ok(OptionLine {
			line_number,
			line_id: line_id.into_iter().next(),
			text: (literal, args_expr),
			if_condition,
			tags,
		}),
		_ => Err(anyhow!(
			"More than one `line_id` tag found.\n\
			 Ids found: `{}`\n\
			 Tags: `{}`\n\
			 Built so far: \n\
			 \tLiteral: `{literal}`\n\
			 \tArguments: `{args:?}`\n\
			 \tIf Condition: `{if_condition:?}`\n\
			 \tMetadata: `{metadata:?}`\n"
			, line_id.join(", "), tags.join(", "))),
	};
}

fn build_args(unparsed_args: Vec<String>) -> Result<Vec<YarnExpr>> {
	let exprs: Vec<YarnExpr> =
		unparsed_args
			.iter()
			.map(|unparsed_arg|
				parse_yarn_expr(unparsed_arg)
					.map_err(|err| anyhow!(
						"{err:?}\n\
				         All Unparsed Arguments: `{unparsed_args:?}`")))
			.try_collect()?;

	return Ok(exprs);
}

fn parse_if_condition_and_metadata(mut chars: Peekable<Chars>) -> Result<(YarnExpr, Option<String>)> {
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
								 Nesting: `{nesting:?}`"));
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
									  then optionally be followed by `#metadata here`"));
							};
						
						let expr =
							parse_yarn_expr(&sum)
								.map_err(|err| anyhow!(
									"Could not parse `if condition` as `YarnExpr`.\n\
									 Error: `{err:?}`\n\
									 Argument: `{sum}`")
								)?;
						
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
									 Help: the closing delimiter `{un_nest}` does not match the most-recent opening delimiter `{nest}`."));
							}
						} else {
							return Err(anyhow!(
								"Unexpected closing delimiter `{un_nest}` in `if condition` argument.\n\
								 Argument: `{sum}`\n\
								 Nesting: `{nesting:?}`\n\n\
								 Help: the characters ('{{', '(', '[') are treated as opening delimiters when inside arguments.\n\
								 Help: for every opening delimiter, there must be a matching closing delimiter (and vice-versa)."));
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
			 Help: for every opening delimiter, there must be a matching closing delimiter (and vice-versa)."));
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

impl ParseRawYarn for OptionLine {
	fn parse_raw_yarn(line: &str, line_number: LineNumber) -> Option<Result<Content>> {
		let mut line = line.trim();
		
		if strip_start_then_trim!(line, "<-") {
			return if line.is_empty() {
				Some(Ok(Content::EndOptions(
					EndOptions {
						line_number
					})))
			} else {
				Some(Err(anyhow!(
					"Could not parse `EndOptions`(`<-`).\n\
					 Error: Unexpected characters after `<-`\n\
					 Remaining line: `{line}`\n\n\
					 Help: Extra characters (like #metadata) are not allowed in `EndOptions`.")))
			};
		}

		if !strip_start_then_trim!(line, "->") {
			return None;
		}
		
		let mut chars = 
			line.chars()
				.peekable();

		let choice_option =
			return_if_err!(
				parse_line(&mut chars, line_number)
					.map_err(|err| anyhow!(
						"Could not parse line as `ChoiceOption`.\n\
						 Error: `{err:?}`\n\
						 Remaining line: `{}`"
						, chars.collect::<String>()))
			);
		
		return Some(Ok(Content::OptionLine(choice_option)));
	}
}