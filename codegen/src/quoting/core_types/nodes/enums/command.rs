use crate::config::YarnConfig;
use crate::quoting::quotable_types::advance::build_next_fn;
use crate::quoting::quotable_types::enums;
use crate::quoting::quotable_types::enums::LineEnum;
use crate::quoting::quotable_types::line_ids::{IDCustomCommand, IDFlatLine, IDFlow};
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use crate::quoting::quotable_types::scope::IDScope;
use crate::quoting::util::SeparatedItems;
use genco::prelude::quoted;
use genco::prelude::rust::Tokens;
use genco::quote;

pub fn all_tokens(
	cfg: &YarnConfig,
	node: &IDNode,
	lines_map: &LinesMap,
) -> Option<Tokens> {
	if lines_map.commands.is_empty() {
		return None;
	}

	let enum_name = enums::enum_type_command(&node.metadata.title);

	let tokens_imports = tokens_imports(cfg);
	let tokens_enum = tokens_enum(cfg, &lines_map.commands, &enum_name);
	let tokens_trait_impl = tokens_trait_impl(cfg, &lines_map.commands, node);

	Some(quote! {
		$tokens_imports
		$tokens_enum
		$tokens_trait_impl
	})
}

fn tokens_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
	}
}

fn tokens_enum(
	cfg: &YarnConfig,
	commands: &[(&IDCustomCommand, LineEnum)],
	enum_name: &str,
) -> Tokens {
	let enum_variants = commands.iter().map(|(_, line_enum)| line_enum.variant_name());

	let structs = commands.iter().map(|(_, line_enum)| {
		let name = line_enum.variant_name();

		quote! {
			#[derive(Debug, Copy, Clone)]
			#[derive(PartialEq, Eq, Hash)]
			#[derive(Serialize, Deserialize)]
			pub struct $name;
		}
	});

	quote! {
		declarative_type_state::unit_enum_delegated! {
			ENUM_OUT: {
				#[derive(Debug, Copy, Clone)]
				#[derive(PartialEq, Eq, Hash)]
				#[derive(Serialize, Deserialize)]
				pub enum $enum_name {
					$(SeparatedItems(enum_variants, ",\n"))
				}
			}
			DELEGATES: {
				impl trait ICommandLine {
					[fn line_id(&self) -> &'static str]
					[fn command(&self, storage: &$(&cfg.storage_direct)) -> $(&cfg.command_direct)]
					[fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield]
				}
			}
		}
		
		$(SeparatedItems(structs, "\n"))
	}
}

fn tokens_trait_impl(
	cfg: &YarnConfig,
	commands: &[(&IDCustomCommand, LineEnum)],
	node: &IDNode,
) -> Tokens {
	let impls = all_advance_fns(commands, node)
		.map(|(command, line_enum, advance_fn)| {
			let line_id_impl = quote! {
				fn line_id(&self) -> &'static str {
					$(quoted(line_enum.raw_id))
				}
			};
			
			let command_impl = {
				let expr = 
					if command.args.is_empty() {
						quote! {
							$(&cfg.command_direct)::$(&command.variant)
						}
					} else {
						quote! {
							$(&cfg.command_direct)::$(&command.variant)($(SeparatedItems(&command.args, ", "))),
						}
					};
				
				quote! {
					fn command(&self, storage: &$(&cfg.storage_direct)) -> $(&cfg.command_direct) {
						$expr
					}
				}
			};
			
			let advance_impl = quote! {
				fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield {
					$advance_fn
				}
			};
			
			quote! {
				impl ICommandLine for $(line_enum.variant_name()) {
					$line_id_impl
					$command_impl
					$advance_impl
				}
			}
		});

	quote! { $(SeparatedItems(impls, "\n")) }
}

fn all_advance_fns<'a>(
	commands: &'a [(&IDCustomCommand, LineEnum)],
	node: &'a IDNode,
) -> impl Iterator<Item = (&'a IDCustomCommand, &'a LineEnum<'a>, Tokens)> {
	let title = node.metadata.title.as_str();
	let mut next_fns = Vec::new();
	let mut scopes: Vec<&IDScope> = node.scopes.iter().collect();

	while !scopes.is_empty() {
		let scope = scopes.remove(0);
		insert_scope_advance_fns(&mut next_fns, scope, &scopes, title);
	}

	if next_fns.len() != commands.len() {
		panic!(
			"The number of next functions that came from commands(`{}`) \
			 does not match the number of commands in the input(`{}`).\n\
			 Node: `{title}`", commands.len(), next_fns.len());
	}

	next_fns
		.into_iter()
		.map(|(line_id, next_fn)| {
			let (cmd, line_enum) = commands
				.iter()
				.find(|(_, line_enum)| line_enum.raw_id == line_id)
				.unwrap_or_else(|| panic!(
					"A next function was generated for a command line that is not included in the input.\n\
					 Line id: {line_id}\n\
					 Next function: {next_fn:?}"));

			(*cmd, line_enum, next_fn)
		})
}

fn insert_scope_advance_fns<'a>(
	next_fns: &mut Vec<(&'a str, Tokens)>,
	current_scope: &'a IDScope,
	next_scopes: &[&IDScope],
	title: &str,
) {
	let mut flows: Vec<&IDFlow> = current_scope.flows.iter().collect();

	while !flows.is_empty() {
		let flow = flows.remove(0);

		match flow {
			IDFlow::Flat(flat_lines) => {
				let mut flat_lines: Vec<&IDFlatLine> = flat_lines.iter().collect();

				while !flat_lines.is_empty() {
					let flat_line = flat_lines.remove(0);

					if let IDFlatLine::CustomCommand(command) = flat_line {
						let next_fn = build_next_fn(
							flat_lines.iter().copied(), //relax, it only copies the references
							flows.iter().copied(),
							next_scopes.iter().copied(),
							title,
						);

						next_fns.push((command.line_id.as_str(), next_fn));
					}
				}
			}
			IDFlow::OptionsFork(options_fork) => {
				for (_, maybe_scope) in options_fork.options.iter() {
					if let Some(option_scope) = maybe_scope {
						insert_scope_advance_fns(next_fns, option_scope, next_scopes, title);
					}
				}
			}
			IDFlow::IfBranch(if_branch) => {
				if let Some(if_scope) = &if_branch.if_.1 {
					insert_scope_advance_fns(next_fns, if_scope, next_scopes, title);
				}

				for (_, maybe_scope) in if_branch.else_ifs.iter() {
					if let Some(else_if_scope) = maybe_scope {
						insert_scope_advance_fns(next_fns, else_if_scope, next_scopes, title);
					}
				}

				if let Some((_, Some(else_scope))) = &if_branch.else_ {
					insert_scope_advance_fns(next_fns, else_scope, next_scopes, title);
				}
			}
		}
	}
}
