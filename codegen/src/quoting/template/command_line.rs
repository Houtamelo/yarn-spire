use genco::quote;
use genco::lang::rust::Tokens;
use crate::config::YarnConfig;
use crate::quoting::helper::SeparatedItems;
use crate::quoting::quotable_types::enums::SUFFIX_COMMAND;
use crate::quoting::quotable_types::node::IDNode;

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
			fn command(&self, storage: &$(&cfg.storage_direct)) -> $(&cfg.command_direct);
		}
	}
}

fn tokens_enum(nodes: &[IDNode]) -> Tokens {
	let titles =
		nodes.iter()
		     .map(|node| {
			     let title = node.metadata.title.clone() + SUFFIX_COMMAND;
			     quote! { $(title) }
		     });
	
	quote! {
		#[enum_dispatch]
		#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
		pub enum CommandLine {
			$(SeparatedItems(titles, ",\n"))
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  nodes: &[IDNode])
                  -> Tokens {
	let imports_and_trait = 
		tokens_imports_and_trait(cfg);
	let enum_tokens = 
		tokens_enum(nodes);
	
	quote! {
		$(imports_and_trait)
		
		$(enum_tokens)
	}
}