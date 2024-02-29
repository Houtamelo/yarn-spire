pub mod runtime;
pub mod speech;
pub mod command_line;
pub mod instruction;
pub mod options;
pub mod title;
pub mod var_trait;
pub mod nodes;
pub mod default_storage;

use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::helper::SeparatedItems;
use crate::quoting::quotable_types::node::IDNode;

pub fn tokens_root_module(cfg: &YarnConfig,
                          nodes: &[IDNode])
                          -> Tokens {
	let nodes_exports_tokens = 
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! { pub use super::nodes::$title::*; }
		     });
	
	let tokens_default_storage_mod = 
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
		pub mod line;
		pub mod runtime;
		pub mod title;
		pub mod nodes;
		pub mod var_trait;
		pub mod speech;
		pub mod command_line;
		$tokens_default_storage_mod
		
		pub type PlayerDecision = usize;
		pub type YieldCounter = usize;
		
		#[allow(unused)]
		pub(crate) mod shared_internal {
			pub use $(&cfg.storage_qualified);
			pub use $(&cfg.command_qualified);
			
			pub use super::{
				PlayerDecision,
				YieldCounter,
			};
			
			pub use super::options::*;
			pub use super::line::*;
			pub use super::runtime::*;
			pub use super::title::*;
			pub use super::nodes::*;
			pub use super::var_trait::*;
			pub use super::speech::*;
			pub use super::command_line::*;
			
			$(SeparatedItems(nodes_exports_tokens, "\n"))
		}
	}
}
