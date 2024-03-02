use genco::quote;
use genco::lang::rust::Tokens;
use crate::config::YarnConfig;
use crate::quoting::util::SeparatedItems;
use crate::quoting::quotable_types::enums::SUFFIX_COMMAND;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};

fn tokens_imports_and_trait(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use enum_dispatch::enum_dispatch;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		#[enum_dispatch(CommandLine)]
		pub trait CommandLineTrait {
			fn next(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
			fn line_id(&self) -> &'static str;
			fn command(&self, storage: &$(&cfg.storage_direct)) -> $(&cfg.command_direct);
		}
	}
}

fn tokens_enum(nodes_mapped: &[(&IDNode, LinesMap)]) -> Tokens {
	let titles =
		nodes_mapped
			.iter()
			.filter_map(|(node, lines_map)| {
				if lines_map.commands.len() > 0 {
					let title = node.metadata.title.clone() + SUFFIX_COMMAND;
					Some(quote! { $(title) })
				} else {
					None
				}
			});
	
	quote! {
		#[enum_dispatch]
		#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
		pub enum CommandLine {
			$(SeparatedItems(titles, ",\n"))
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  nodes_mapped: &[(&IDNode, LinesMap)])
                  -> Tokens {
	let imports_and_trait = 
		tokens_imports_and_trait(cfg);
	let enum_tokens = 
		tokens_enum(nodes_mapped);
	
	quote! {
		$(imports_and_trait)
		
		$(enum_tokens)
	}
}