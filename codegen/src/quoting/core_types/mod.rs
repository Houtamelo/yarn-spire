pub mod runtime;
pub mod speech;
pub mod command_line;
pub mod instruction;
pub mod options;
pub mod title;
pub mod var_trait;
pub mod nodes;
pub mod default_storage;
pub mod built_in_functions;

use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::util::SeparatedItems;
use crate::quoting::quotable_types::node::IDNode;

pub fn tokens_root_module(cfg: &YarnConfig,
                          nodes: &[IDNode])
                          -> Tokens {
	let nodes_exports = 
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! { pub use super::nodes::$title::*; }
		     });
	
	let default_storage_mod= 
		if cfg.generate_storage {
			quote! { pub mod default_storage; }
		} else {
			Tokens::new()
		};

	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]

		pub mod options;
		pub mod instruction;
		pub mod runtime;
		pub mod title;
		pub mod nodes;
		pub mod var_trait;
		pub mod speech;
		pub mod command_line;
		pub mod built_in_functions;
		$default_storage_mod
		
		pub type PlayerDecision = usize;
		pub type YieldCounter = usize;
		
		#[allow(unused)]
		pub(crate) mod shared_internal {
			pub use $(&cfg.storage_qualified);
			pub use $(&cfg.command_qualified);
			pub use $(&cfg.vars_qualified)::*;
			
			pub use super::{
				PlayerDecision,
				YieldCounter,
			};
			
			pub use super::options::*;
			pub use super::instruction::*;
			pub use super::runtime::*;
			pub use super::title::*;
			pub use super::nodes::*;
			pub use super::var_trait::*;
			pub use super::speech::*;
			pub use super::command_line::*;
			pub use super::built_in_functions;
			
			$(SeparatedItems(nodes_exports, "\n"))
		}
	}
}
