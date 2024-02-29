use std::iter::Peekable;
use std::str::Chars;
use crate::expressions::yarn_expr::{BuiltInFunctionCall, DeclarationTy, YarnExpr};
use crate::{LineNumber, UnparsedLine};
use anyhow::{Result, anyhow};
use crate::expressions::parse_yarn_expr;
use crate::expressions::yarn_lit::YarnLit;
use crate::parsing::macros::{strip_end_then_trim, strip_start_then_trim, trim};

pub struct VarDeclaration {
	pub line_number: LineNumber,
	pub var_name: String,
	pub default_value: YarnExpr,
	pub cast_ty: Option<DeclarationTy>,
}

impl VarDeclaration {
	pub fn infer_ty(&self) -> Option<DeclarationTy> {
		if let Some(cast_ty) = self.cast_ty {
			return Some(cast_ty);
		}
		
		return infer_expr(&self.default_value);
		
		fn infer_expr(expr: &YarnExpr) -> Option<DeclarationTy> {
			return match expr {
				YarnExpr::Lit(lit) => {
					match lit {
						YarnLit::Int(_) => Some(DeclarationTy::isize),
						YarnLit::Float(_) => Some(DeclarationTy::f64),
						YarnLit::Str(_) => Some(DeclarationTy::String),
						YarnLit::Bool(_) => Some(DeclarationTy::bool),
					}
				}
				| YarnExpr::Parenthesis(inner_expr) 
				| YarnExpr::UnaryOp { right: inner_expr, .. } => 
					infer_expr(inner_expr),
				YarnExpr::BinaryOp { left, right, .. } => 
					infer_expr(left).or_else(|| infer_expr(right)),
				YarnExpr::BuiltInFunctionCall(built_in_call) => {
					match built_in_call {
						BuiltInFunctionCall::FormatInvariant(_) => 
							Some(DeclarationTy::String),
						BuiltInFunctionCall::Random => 
							Some(DeclarationTy::f64),
						BuiltInFunctionCall::RandomRange(lower, upper) => 
							infer_expr(lower).or_else(|| infer_expr(upper)),
						| BuiltInFunctionCall::Dice(_)
						| BuiltInFunctionCall::Round(_)
						| BuiltInFunctionCall::RoundPlaces(_, _)
						| BuiltInFunctionCall::Floor(_)
						| BuiltInFunctionCall::Ceil(_)
						| BuiltInFunctionCall::Inc(_) 
						| BuiltInFunctionCall::Dec(_)
						| BuiltInFunctionCall::Int(_) => 
							Some(DeclarationTy::isize),
						BuiltInFunctionCall::Decimal(_) =>
							Some(DeclarationTy::f64),
					}
				}
				YarnExpr::Cast { cast_ty, .. } => 
					Some(*cast_ty),
				| YarnExpr::Identifier(_)
				| YarnExpr::CustomFunctionCall {..}
				| YarnExpr::VarGet(_) => {
					None
				}
			};
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

fn parse_args(args_iter: &mut ArgBuilder) -> Result<(String, YarnExpr, Option<DeclarationTy>)> {
	let var_name = {
		let unparsed_string =
			args_iter.next()
				.unwrap_or_else(|| Err(anyhow!(
					"Could not build `variable name` argument.\n\
				     Error: `ArgBuilder` returned None.")))
				.and_then(|(var_name, ended)|
					if !ended {
						Ok(var_name)
					} else {
						Err(anyhow!(
							"Ended with `>>` after only one argument.\n\
							 Expected two: `variable name` then `default value`.\n\
							 Variable name: {var_name}"))
					}
				)?;

		let expr =
			parse_yarn_expr(unparsed_string.as_str())
				.map_err(|err| anyhow!(
					"Could not parse `variable name` argument as `YarnExpr`.\n\
					 Error: {err}\n\
					 Variable name: {unparsed_string}")
				)?;

		let YarnExpr::VarGet(name) = expr 
			else {
				return Err(anyhow!(
					"Expected `variable name` argument to be `YarnExpr::VarGet(var_name)`.\n\
					 Instead got: `{expr:?}`\n\n\
					 Help: `variable names` follow the syntax: `$var_name_here`, did you include the `$`?"));
			};

		name
	};

	let (value_expr, already_ended) = {
		args_iter.allow_yarn_set_syntax = true;

		let (unparsed_string, ended) =
			args_iter
				.next()
				.unwrap_or_else(|| Err(anyhow!(
					"Could not build `variable value` argument.\n\
					 Error: `ArgBuilder` returned None.
		             Variable name: {var_name:?}"))
				)?;

		let expr =
			parse_yarn_expr(&unparsed_string)
				.map_err(|err| anyhow!(
					"Could not parse `variable value` argument as `YarnExpr`.\n\
					 Error: `{err}`\n\
					 Variable name: `{var_name:?}`")
				)?;

		args_iter.allow_yarn_set_syntax = false;

		(expr, ended)
	};
	
	if already_ended {
		return Ok((var_name, value_expr, None));
	}
	
	let remaining = args_iter.chars.by_ref().collect::<String>();
	let mut remaining_str = remaining.as_str();
	trim!(remaining_str);
	
	if !strip_end_then_trim!(remaining_str, ">>")
		|| !strip_start_then_trim!(remaining_str, "as"){
		return Err(anyhow!(
			"Invalid argument after `variable value`. Expected nothing or `as [type]`.\n\
			 Instead got: `{remaining_str}`"));
	}
	
	return if let Some(cast_ty) = DeclarationTy::from_str(remaining_str) {
		Ok((var_name, value_expr, Some(cast_ty)))
	} else {
		Err(anyhow!(
			"Invalid cast type: `{remaining_str}`.\n\
			 Expected one of(case-insensitive): `string`, `number`, `bool`,\
			  any rust int (`i8`, `u32`, ..) or a float (`f32`, `f64`)"))
	};
}

impl VarDeclaration {
	pub fn try_parse(unparsed_line: &UnparsedLine) -> Option<Result<VarDeclaration>> {
		let mut line = unparsed_line.text.trim();

		if !strip_start_then_trim!(line, "<<")
			&& !strip_start_then_trim!(line, "declare") {
			return None;
		}

		let mut args_iter =
			ArgBuilder {
				chars: line.chars().peekable(),
				allow_yarn_set_syntax: false,
			};

		match parse_args(&mut args_iter) {
			Ok((var_name, default_value, cast_ty)) => {
				if default_value.iter_exprs().any(|expr| matches!(expr, YarnExpr::VarGet(_))) {
					Some(Err(anyhow!(
						"Could not parse line as `declare` statement(`<<declare $var_name [default_value]>>`).\n\
						 Error: Custom variables(`$var_name`) are not allowed in default values.")))
				} else {
					Some(Ok(
						VarDeclaration {
							line_number: unparsed_line.line_number,
							var_name,
							default_value,
							cast_ty,
						}))
				}
			},
			Err(err) =>
				Some(Err(anyhow!(
					"Could not parse line as `declare` statement(`<<declare $var_name [default_value]>>`).\n\
				     Remaining Line: {}.\n\
					 Error: `{err}`"
					, args_iter.chars.collect::<String>())))
		}
	}
}