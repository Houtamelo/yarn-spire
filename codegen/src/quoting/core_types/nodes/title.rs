use std::collections::HashSet;
use anyhow::{anyhow, Result};
use genco::lang::rust::Tokens;
use genco::prelude::quoted;
use genco::quote;
use crate::config::YarnConfig;
use crate::expressions::built_in_calls::BuiltInFunctionCall;
use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::raw::node_metadata::TrackingSetting;
use crate::quoting::util::SeparatedItems;
use crate::quoting::quotable_types;
use crate::quoting::quotable_types::node::IDNode;
use crate::quoting::quotable_types::scope::IDScope;

fn tokens_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::borrow::Cow;
		use houtamelo_utils::prelude::*;
		use serde::{Deserialize, Serialize};
		use strum::IntoStaticStr;
		use $(&cfg.shared_qualified)::*;
	}
}

fn tokens_title_trait_impl(cfg: &YarnConfig,
                           node: &IDNode,
                           inferred_tracking_setting: TrackingSetting)
                           -> Tokens {
	let metadata = &node.metadata;
	let title = &metadata.title;
	
	let tags = 
		metadata.tags.iter().map(quoted);
	
	let customs = 
		metadata.customs.iter().map(quoted);
	
	let tracking_setting =
		match inferred_tracking_setting {
			TrackingSetting::Always => quote! { TrackingSetting::Always },
			TrackingSetting::Never => quote! { TrackingSetting::Never },
		};

	let tokens_first_line = 
		quotable_types::next::build_next_fn(&[],&[], &node.scopes, &node.metadata.title);
	
	quote! {
		pub static TAGS: &'static[&'static str] = &[
			$(SeparatedItems(tags, ",\n"))
		];
		
		pub static TRACKING: TrackingSetting = $tracking_setting;
		
		pub static CUSTOM_METADATA: &'static [&'static str] = &[
			$(SeparatedItems(customs, ",\n"))
		];
		
		#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
		pub struct $title;
		
		impl NodeTitleTrait for $title {
			fn tags(&self) -> &'static [&'static str] { TAGS }
			fn tracking(&self) -> TrackingSetting { TRACKING }
			fn custom_metadata(&self) -> &'static [&'static str] { CUSTOM_METADATA }
			
			fn start(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield { 
				$tokens_first_line
			}
		}
	}
}

fn node_names_in_args(node: &IDNode) -> impl Iterator<Item = &str> {
	node.scopes
	    .iter()
	    .flat_map(IDScope::iter_exprs)
	    .filter_map(|expr| {
		    if let YarnExpr::BuiltInFunctionCall(
			    | BuiltInFunctionCall::Visited(node_name) 
			    | BuiltInFunctionCall::VisitedCount(node_name)
		    ) = expr {
			    Some(node_name.as_str())
		    } else {
			    None
		    }
	    })
}

pub fn infer_all_nodes_tracking(nodes: &[IDNode]) -> Result<Vec<(&IDNode, TrackingSetting)>> {
	let nodes_to_track: HashSet<&str> = {
		let node_names_in_visited_calls: HashSet<&str> =
			nodes.iter()
			     .flat_map(node_names_in_args)
			     .collect();

		let titles_in_files: HashSet<&str> =
			nodes.iter()
			     .map(|node| node.metadata.title.as_str())
			     .collect();

		let used_nodes_that_dont_exist =
			node_names_in_visited_calls
				.iter()
				.filter_map(|used_title|
					if !titles_in_files.contains(used_title) {
						Some(*used_title)
					} else {
						None
					})
				.collect::<Vec<&str>>();

		if used_nodes_that_dont_exist.len() > 0 {
			return Err(anyhow!(
				"Found node names in `visited([name])` or `visited_count([name])` that do not exist in any of the provided files.\n\
				 Node names: {}\n\
				 Please make sure that the node names are correct."
				, used_nodes_that_dont_exist.join(", ")));
		}

		node_names_in_visited_calls
	};

	Ok(
		nodes.iter()
		     .map(|node| {
			     let inferred =
				     node.metadata
					     .tracking
					     .unwrap_or_else(||
						     if nodes_to_track.contains(node.metadata.title.as_str()) {
							     TrackingSetting::Always
						     } else {
							     TrackingSetting::Never
						     });

			     (node, inferred)
		     }).collect()
	)
}

pub fn all_tokens(cfg: &YarnConfig,
                  node: &IDNode,
                  inferred_tracking: TrackingSetting)
                  -> Tokens {
	let imports =
		tokens_imports(cfg);
	let trait_impl =
		tokens_title_trait_impl(cfg, node, inferred_tracking);
	
	quote! {
		$imports
		
		$trait_impl
	}
}