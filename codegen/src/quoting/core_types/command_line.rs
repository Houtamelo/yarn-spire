use crate::config::YarnConfig;
use crate::quoting::quotable_types::enums::SUFFIX_COMMAND;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use crate::quoting::util::SeparatedItems;
use genco::lang::rust::Tokens;
use genco::quote;

pub fn all_tokens(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Tokens {
	let imports_and_trait = tokens_imports_and_trait(cfg);
	let enum_tokens = tokens_enum(cfg, nodes_mapped);

	quote! {
		$(imports_and_trait)
		$(enum_tokens)
	}
}

fn tokens_imports_and_trait(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		pub trait ICommandLine {
			fn line_id(&self) -> &'static str;
			fn command(&self, storage: &$(&cfg.storage_direct)) -> $(&cfg.command_direct);
			fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
		}
	}
}

fn tokens_enum(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Tokens {
	let titles = nodes_mapped
		.iter()
		.filter_map(|(node, lines_map)| {
			if !lines_map.commands.is_empty() {
				let title = node.metadata.title.clone() + SUFFIX_COMMAND;
				Some(quote! { $(title) })
			} else {
				None
			}
		});

	quote! {
		declarative_type_state::delegated_enum! {
			ENUM_OUT: {
				#[derive(Debug, Copy, Clone)]
				#[derive(PartialEq)]
				#[derive(Serialize, Deserialize)]
				pub enum CommandLine {
					$(SeparatedItems(titles, ",\n"))
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
	}
}
