use std::collections::HashSet;
use genco::lang::rust::Tokens;
use genco::prelude::quoted;
use genco::quote;
use crate::config::YarnConfig;
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_lit::YarnLit;
use crate::parsing::raw::node_metadata::TrackingSetting;
use crate::quoting::helper::SeparatedItems;
use crate::quoting::quotable_types;
use crate::quoting::quotable_types::line_ids::{BuiltInCommand, IDFlatLine, IDFlow};
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

macro_rules! yield_items {
    ($iter:expr) => {
	    for _item in $iter {
		    yield _item;
	    }
    };
}

fn scope_exprs(scope: &IDScope) -> impl Iterator<Item = &YarnExpr> {
	std::iter::from_coroutine(Box::pin(move || {
		for flow in &scope.flows {
			match flow {
				IDFlow::Flat(flat_lines) => {
					for line in flat_lines {
						match line {
							IDFlatLine::Speech(speech) => {
								for arg in &speech.text.1 {
									yield_items!(arg.iter_exprs());
								}
							},
							IDFlatLine::CustomCommand(custom_command) => {
								for arg in &custom_command.args {
									yield_items!(arg.iter_exprs());
								}
							},
							IDFlatLine::BuiltInCommand(built_in_command) => {
								if let BuiltInCommand::Set { value, .. } = built_in_command {
									yield_items!(value.iter_exprs());
								}
							},
						}
					}
				},
				IDFlow::OptionsFork(options_fork) => {
					for (line, scope_option) in options_fork.options.iter() {
						for arg in &line.text.1 {
							yield_items!(arg.iter_exprs());
						}

						if let Some(if_condition) = &line.if_condition {
							yield_items!(if_condition.iter_exprs());
						}

						if let Some(scope_inside_option) = scope_option {
							yield_items!(scope_exprs(&scope_inside_option));
						}
					}
				}
				IDFlow::IfBranch(if_branch) => {
					yield_items!(if_branch.if_.0.condition.iter_exprs());
					
					if let Some(if_scope) = &if_branch.if_.1 {
						yield_items!(scope_exprs(&if_scope));
					}
					
					for (else_if, else_if_scope_option) in &if_branch.else_ifs {
						yield_items!(else_if.condition.iter_exprs());
						
						if let Some(else_if_scope) = else_if_scope_option {
							yield_items!(scope_exprs(&else_if_scope));
						}
					}
				}
			}
		}
	}))
}

fn args_in_visited_calls(node: &IDNode) -> impl Iterator<Item = &str> {
	node.scopes
	    .iter()
	    .flat_map(scope_exprs)
		.filter_map(|expr| {
			if let YarnExpr::CustomFunctionCall { func_name, args } = expr 
				&& let "visited" | "visited_count" = func_name.as_str() {
				Some(
					args.iter()
						.filter_map(|arg| {
							match arg {
								YarnExpr::Lit(YarnLit::Str(str)) =>
									Some(str.as_str()),
								YarnExpr::Identifier(ident) =>
									Some(ident.as_str()),
								_ => None,
							}
						}))
			} else {
				None
			} 
		}).flatten()
}

pub fn infer_all_nodes_tracking(nodes: &[IDNode]) -> Vec<(&IDNode, TrackingSetting)> {
	let nodes_to_track: HashSet<&String> = {
		let args_in_visited_calls: HashSet<&str> =
			nodes.iter()
				 .flat_map(args_in_visited_calls)
				 .collect();

		nodes.iter()
		     .map(|node| &node.metadata.title)
		     .filter(|title| args_in_visited_calls.contains(title.as_str()))
		     .collect()
	};
	
	nodes.iter()
	     .map(|node| {
		     let tracking = node.metadata.tracking;
		     let inferred =
			     match tracking {
				     Some(tracking) => tracking,
				     None => {
					     if nodes_to_track.contains(&node.metadata.title) {
						     TrackingSetting::Always
					     } else {
						     TrackingSetting::Never
					     }
				     }
			     };

		     (node, inferred)
	     }).collect()
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