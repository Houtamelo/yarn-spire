use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::util::{Comments, SeparatedItems};
use crate::quoting::quotable_types::node::IDNode;

fn tokens_imports_and_trait(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::fmt::Debug;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		$(Comments([
			"The original YarnSpinner's \n\
			[tracking setting](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header)."]))
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum TrackingSetting {
			Always,
			Never,
		}
		
		pub trait NodeTitleTrait {
			fn tags(&self) -> &'static[&'static str];
			fn tracking(&self) -> TrackingSetting;
			fn custom_metadata(&self) -> &'static[&'static str];
			fn start(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
		}
	}
}

fn tokens_enum(cfg: &YarnConfig,
               nodes: &[IDNode])
               -> Tokens {
	let variants =
		nodes.iter()
		     .map(|node| {
			     quote! { $(&node.metadata.title) }
		     });
	
	let tags_impls =
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! {
			     	NodeTitle::$title =>{ 
			     		$title.tags()
				     }
			     }
		     });
	
	let tracking_impls =
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! {
			     	NodeTitle::$title => {
			     		$title.tracking()
				     }
			     }
		     });
	
	let custom_metadata_impls =
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! {
			     	NodeTitle::$title => {
			     		$title.custom_metadata()
				     }
			     }
		     });
	
	let start_impls =
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! {
			     	NodeTitle::$title => {
					     $title.start(storage)
				     }
			     }
		     });

	quote! {
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
		pub enum NodeTitle {
			$(SeparatedItems(variants, ",\n"))
		}
		
		impl NodeTitleTrait for NodeTitle {
			fn tags(&self) -> &'static [&'static str] {
				return match self {
					$(SeparatedItems(tags_impls, ",\n"))
				};
			}
		
			fn tracking(&self) -> TrackingSetting {
				return match self {
					$(SeparatedItems(tracking_impls, ",\n"))
				};
			}
		
			fn custom_metadata(&self) -> &'static [&'static str] {
				return match self {
					$(SeparatedItems(custom_metadata_impls, ",\n"))
				};
			}
		
			fn start(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield {
				return match self {
					$(SeparatedItems(start_impls, ",\n"))
				};
			}
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  nodes: &[IDNode])
                  -> Tokens {
	let imports_and_trait = 
		tokens_imports_and_trait(cfg);
	let enum_tokens = 
		tokens_enum(cfg, nodes);

	quote! {
		$imports_and_trait
		
		$enum_tokens
	}
}