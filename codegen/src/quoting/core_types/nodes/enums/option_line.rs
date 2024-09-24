use crate::config::YarnConfig;
use crate::quoting::quotable_types::advance::build_next_fn;
use crate::quoting::quotable_types::enums;
use crate::quoting::quotable_types::enums::OptionLineEnum;
use crate::quoting::quotable_types::line_ids::{IDFlow, IDOptionLine};
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
	if lines_map.option_lines.is_empty() {
		return None;
	}

	let enum_name = enums::enum_type_option_line(&node.metadata.title);

	let tokens_imports = tokens_imports(cfg);
	let tokens_enum = tokens_enum(cfg, &lines_map.option_lines, &enum_name);
	let tokens_trait_impl = tokens_trait_impl(cfg, &lines_map.option_lines, node);

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
		
		use houtamelo_utils::prelude::*;
		use $(&cfg.shared_qualified)::*;
		use std::borrow::Cow;
		use serde::{Deserialize, Serialize};
	}
}

fn tokens_enum(
	cfg: &YarnConfig,
	options: &[(&IDOptionLine, OptionLineEnum)],
	enum_name: &str,
) -> Tokens {
	let enum_variants = options.iter().map(|(_, line_enum)| line_enum.variant_name());

	let structs = options.iter().map(|(_, line_enum)| {
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
				impl trait IOptionLine {
					[fn line_id(&self) -> &'static str]
					[fn tags(&self) -> &'static [&'static str]]
					[fn is_available(&self, storage: &$(&cfg.storage_direct)) -> Option<bool>]
					[fn text(&self, storage: &$(&cfg.storage_direct)) -> Cow<'static, str>]
					[fn fork(&self) -> OptionsFork]
					[fn index_on_fork(&self) -> usize]
					[fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield]
				}
			}
		}
					
		
		$(SeparatedItems(structs, "\n"))
	}
}

fn tokens_trait_impl(
	cfg: &YarnConfig,
	options: &[(&IDOptionLine, OptionLineEnum)],
	node: &IDNode,
) -> Tokens {
	let impls = all_advance_fns(node, options)
		.map(|(option, line_enum, advance_fn)| {
			let line_id_impl = quote! {
				fn line_id(&self) -> &'static str {
					$(quoted(line_enum.raw_id))
				}
			};
			
			let tags_impl =
				if !option.tags.is_empty() {
					quote! {
						fn tags(&self) -> &'static [&'static str] {
							&[ 
								$(SeparatedItems(option.tags.iter().map(quoted), ",\n"))
							],
						}
					}
				} else {
					Tokens::new()
				};
			
			let text_impl = {
				let (literal, exprs) = &option.text;
				let quoted_lit = quoted(literal);
				
				let text = 
					if exprs.is_empty() {
						quote!($quoted_lit)
					} else {
						quote!(format!($quoted_lit, $(SeparatedItems(exprs, ", "))))
					};
				
				quote! {
					fn text(&self, storage: &$(&cfg.storage_direct)) -> Cow<'static, str> {
						$text.into()
					}
				}
			};
			
			let is_available_impl = 
				if let Some(condition) = &option.if_condition {
					quote! {
						fn is_available(&self, storage: &$(&cfg.storage_direct)) -> Option<bool> {
							Some($condition)
						}
					}
				} else {
					Tokens::new()
				};
			
			let fork_impl = quote! {
				fn fork(&self) -> OptionsFork {
					$(&*option.fork_qualified)
				}
			};
			
			let index_impl = quote! {
				fn index_on_fork(&self) -> usize {
					$(option.index_on_fork)
				}
			};
			
			let advance_impl = quote! {
				fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield {
					$advance_fn
				}
			};
			
			quote! {
				impl IOptionLine for $(line_enum.variant_name()) {
					$line_id_impl
					$tags_impl
					$text_impl
					$is_available_impl
					$fork_impl
					$index_impl
					$advance_impl
				}
			}
		});

	quote! { $(SeparatedItems(impls, "\n")) }
}

fn all_advance_fns<'a>(
	node: &'a IDNode,
	options: &'a [(&IDOptionLine, OptionLineEnum)],
) -> impl Iterator<Item = (&'a IDOptionLine, &'a OptionLineEnum<'a>, Tokens)> {
	let title = node.metadata.title.as_str();
	let mut next_fns = Vec::new();
	let mut scopes: Vec<&IDScope> = node.scopes.iter().collect();

	while !scopes.is_empty() {
		let scope = scopes.remove(0);
		insert_scope_advance_fns(&mut next_fns, scope, &scopes, title);
	}

	if next_fns.len() != options.len() {
		panic!(
			"The number of next functions that came from option lines(`{}`) \
			 does not match the number of option lines in the input(`{}`).\n\
			 Node: `{title}`", options.len(), next_fns.len());
	}

	next_fns.into_iter().map(|(line_id, next_fn)| {
		let (option_line, line_enum) = options
			.iter()
			.find(|(_, line_enum)| line_enum.raw_id == line_id)
			.unwrap_or_else(|| panic!(
				"A next function was generated for a option line that is not included in the input.\n\
					 Line id: {line_id}\n\
					 Next function: {next_fn:?}"));

		(*option_line, line_enum, next_fn)
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
			IDFlow::OptionsFork(options_fork) => {
				for (line, maybe_scope) in options_fork.options.iter() {
					let next_fn = build_next_fn(
						&[],
						flows.iter().copied(), //relax, it only copies the references
						next_scopes.iter().copied(),
						title,
					);

					next_fns.push((line.line_id.as_str(), next_fn));

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
			IDFlow::Flat(_) => {}
		}
	}
}
