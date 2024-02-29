use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::helper::{Comments, SeparatedItems};
use crate::quoting::quotable_types::node::IDNode;

fn tokens_imports_and_trait(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::fmt::Debug;
		use enum_dispatch::enum_dispatch;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		$(Comments([
			r#"The original YarnSpinner's 
			[tracking setting](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header)."#]))
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum TrackingSetting {
			Always,
			Never,
		}
		
		#[enum_dispatch(NodeTitle)]
		pub trait NodeTitleTrait {
			fn tags(&self) -> &'static[&'static str];
			fn tracking(&self) -> TrackingSetting;
			fn custom_metadata(&self) -> &'static[&'static str];
			fn start(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
		}
	}
}

fn tokens_enum(nodes: &[IDNode]) -> Tokens {
	let variants =
		nodes.iter()
		     .map(|node| {
			     quote! { $(&node.metadata.title) }
		     });

	quote! {
		#[enum_dispatch]
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
		pub enum NodeTitle {
			$(SeparatedItems(variants, ",\n"))
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
		$imports_and_trait
		
		$enum_tokens
	}
}