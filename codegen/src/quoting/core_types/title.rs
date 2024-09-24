use crate::config::YarnConfig;
use crate::quoting::quotable_types::node::IDNode;
use crate::quoting::util::{Comments, SeparatedItems};
use genco::lang::rust::Tokens;
use genco::quote;

pub fn all_tokens(cfg: &YarnConfig, nodes: &[IDNode]) -> Tokens {
	let imports_and_trait = tokens_imports_and_trait(cfg);
	let enum_tokens = tokens_enum(cfg, nodes);

	quote! {
		$imports_and_trait
		$enum_tokens
	}
}

fn tokens_imports_and_trait(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::fmt::Debug;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		pub trait INodeTitle {
			#[must_use]
			fn tags(&self) -> &'static[&'static str];
			#[must_use]
			fn tracking(&self) -> TrackingSetting;
			#[must_use]
			fn custom_metadata(&self) -> &'static[&'static str];
			#[must_use]
			fn start(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
		}
		
		$(Comments([
			"The original YarnSpinner's \
			[tracking setting](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header)."]))
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum TrackingSetting {
			Always,
			Never,
		}
	}
}

fn tokens_enum(cfg: &YarnConfig, nodes: &[IDNode]) -> Tokens {
	let variants = nodes
		.iter()
		.map(|node| quote! { $(&node.metadata.title) });

	quote! {
		declarative_type_state::unit_enum_delegated! {
			ENUM_OUT: {
				#[derive(Debug, Copy, Clone)]
				#[derive(PartialEq, Eq, Hash)]
				#[derive(Serialize, Deserialize)]
				pub enum NodeTitle {
					$(SeparatedItems(variants, ",\n"))
				}
			}
			
			DELEGATES: {
				impl trait INodeTitle {
					[fn tags(&self) -> &'static [&'static str]]
					[fn tracking(&self) -> TrackingSetting]
					[fn custom_metadata(&self) -> &'static [&'static str]]
					[fn start(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield]
				}
			}
		}
	}
}
