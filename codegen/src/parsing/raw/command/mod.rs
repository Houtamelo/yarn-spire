#[cfg(test)]
mod tests;

use std::any::type_name;
use std::collections::VecDeque;
use std::str::FromStr;
use anyhow::{anyhow, Result};
use proc_macro2::{TokenStream, TokenTree};
use expressions::parse_yarn_expr;
use crate::{expressions, LineNumber};
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::parsing::macros::{return_if_err, starts_with_any, strip_end_then_trim, strip_start_then_trim};
use crate::parsing::raw::{ParseRawYarn, Content};
use crate::parsing::raw::arg_parser::ArgsIter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YarnCommand {
	pub line_number: LineNumber,
	pub variant: CommandVariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandVariant {
	Set { var_name: String, op: SetOperation, value: YarnExpr },
	Jump { node_name: String },
	Stop,
	Other { variant: String, args: Vec<YarnExpr> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SetOperation {
	Assign,
	Add,
	Sub,
	Mul,
	Div,
	Rem,
}

fn parse_command_name(args_iter: &mut ArgsIter) -> Result<String> {
	let expr =
		args_iter
			.next()
			.unwrap_or_else(|| Err(anyhow!(
				"Expected `Rust identifier` argument but `{}` returned None.", type_name::<ArgsIter>()))
			)?;
	
	return if let YarnExpr::Identifier(cmd_name) = expr {
		Ok(cmd_name)
	} else {
		Err(anyhow!(
			"Expected `command_name` to be `Rust identifier` argument.\
			 Got: `{expr:?}`.\n\n\
			 Help: Command names can only contain letters, numbers and underscores. \
			 They must also start with either a letter or underscore."))
	};
}

fn parse_set_command(remaining_line: String) -> Result<(String, SetOperation, YarnExpr)> {
	let mut remaining_str = 
		remaining_line.trim();
	
	if !strip_start_then_trim!(remaining_str, "$") {
		return Err(anyhow!(
			"Expected `variable name` argument to start with `$`.\n\
			 Variable name should have started at: {remaining_str}"));
	}
	
	let Some((var_name, rest)) = remaining_str.split_once(' ')
		else {
			return Err(anyhow!(
				"Expected whitespace after `variable name` argument."));
		};

	let mut remaining_str = 
		rest.trim();

	let operation =
		if strip_start_then_trim!(remaining_str, "=")
			|| strip_start_then_trim!(remaining_str, "to ") {
			SetOperation::Assign
		} else if strip_start_then_trim!(remaining_str, "+=") {
			SetOperation::Add
		} else if strip_start_then_trim!(remaining_str, "-=") {
			SetOperation::Sub
		} else if strip_start_then_trim!(remaining_str, "*=") {
			SetOperation::Mul
		} else if strip_start_then_trim!(remaining_str, "/=") {
			SetOperation::Div
		} else if strip_start_then_trim!(remaining_str, "%=") {
			SetOperation::Rem
		} else {
			return Err(anyhow!(
				"Expected `set operation type` argument (`=`|`to`|`+=`|`-=`|`*=`|`/=`|`%=`).\n\
				 `set operation type` should have started at: {remaining_str}\n\
				 Variable name: {var_name:?}"));
		};
	
	let value_expr =
		parse_yarn_expr(remaining_str)
			.map_err(|err| anyhow!(
				"Could not parse `variable value` argument as `YarnExpr`.\n\
				 Error: `{err}`\n\
				 Variable name: `{var_name:?}`")
			)?;

	return Ok((var_name.to_string(), operation, value_expr));
}

fn parse_jump_command(args_iter: &mut ArgsIter) -> Result<String> {
	let remaining_line =
		args_iter.to_string();
	
	let remaining_str = 
		remaining_line.trim();

	let expr = 
		parse_yarn_expr(remaining_str)
			.map_err(|err| anyhow!(
				"Could not parse `node name` argument as `YarnExpr`.\n\
				 Error: `{err}`")
			)?;

	return match expr {
		| YarnExpr::Lit(YarnLit::Str(node_name))
		| YarnExpr::Identifier(node_name) => {
			Ok(node_name)
		},
		_ => {
			Err(anyhow!(
							"Expected `node name` argument to be `YarnExpr::Lit(YarnLit::Str(node_name))` or `YarnExpr::Identifier(node_name)`.\n\
							 Instead got: `{expr:?}`\n\n\
							 Help: `node names` can only contain letters, numbers and underscores. \
							 They must also start with either a letter or underscore."))
		}
	};
}

fn parse_other_command(args_iter: &mut ArgsIter,
                       command_name: String)
                       -> Result<(String, Vec<YarnExpr>)> {
	let args =
		args_iter
			.try_collect()
			.map_err(|err| anyhow!(
				"Could not build argument list.\n\
				 Command name: `{command_name}`\n\
				 Error: `{err}`")
			)?;

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
		
		if !strip_end_then_trim!(line, ">>") {
			return Some(Err(anyhow!(
				"Command did not end with `>>`.\n\
				 Remaining Line: `{line}`")));
		}
		
		let mut tokens: VecDeque<TokenTree> =
			return_if_err!(
				TokenStream::from_str(line)
					.map_err(|err| anyhow!(
						"Could not convert line into `TokenStream`.\n\
						 Line: `{line}`\n\
						 Error: `{err}`"))
					.map(|stream| 
						stream.into_iter().collect())
			);

		let command_name =
			match tokens.pop_front() {
				Some(TokenTree::Ident(name)) => {
					name.to_string()
				},
				unexpected => {
					return Some(Err(anyhow!(
						"Expected `command_name` argument to be Some(TokenTree::Ident(command_name)).\n\
						 Got `{unexpected:?}` returned None.")));
				}
			};
		
		let mut args_iter = 
			ArgsIter { tokens };
		
		return match command_name.as_str() {
			"set" => {
				match parse_set_command(args_iter.to_string()) {
					Ok((var_name, op, value)) => Some(Ok(
						Content::Command(YarnCommand {
							line_number,
							variant: CommandVariant::Set { var_name, op, value },
						}))),
					Err(err) => Some(Err(anyhow!(
						"Could not parse line as `set` command(`<<set $var_name (`=`|`to`|`+=`|`-=`|`*=`|`/=`|`%=`) [value]>>`).\n\
					     Remaining Line: {args_iter}.\n\
						 Error: `{err}`")))
				}
			},
			"jump" => {
				match parse_jump_command(&mut args_iter) {
					Ok(node_name) => Some(Ok(
						Content::Command(YarnCommand {
							line_number,
							variant: CommandVariant::Jump { node_name }
						}))),
					Err(err) => Some(Err(anyhow!(
						"Could not parse line as `jump` command(`<<jump [NodeName]>>`).\n\
					     Remaining Line: {args_iter}.\n\
						 Error: `{err}`")))
				}
			},
			"stop" => {
				Some(Ok(
					Content::Command(YarnCommand {
						line_number,
						variant: CommandVariant::Stop,
					})))
			},
			_ => {
				match parse_other_command(&mut args_iter, command_name) {
					Ok((variant, args)) => Some(Ok(
						Content::Command(YarnCommand {
							line_number,
							variant: CommandVariant::Other { variant, args }
						}))),
					Err(err) => Some(Err(anyhow!(
						"Could not parse line as `other`(not `set`, `jump` or `stop`) command(`{}`).\n\
					     Remaining Line: `{args_iter}`.\n\
						 Error: `{err}`", type_name::<YarnCommand>())))
				}
			},
		};
	}
}
