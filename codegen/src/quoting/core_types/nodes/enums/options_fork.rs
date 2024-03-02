use genco::prelude::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::util::SeparatedItems;
use crate::quoting::quotable_types::line_ids::IDOptionsFork;
use crate::quoting::quotable_types::enums::{LineEnum, OptionLineEnum};
use crate::quoting::quotable_types::enums;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};

fn tokens_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use houtamelo_utils::prelude::*;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
	}
}

fn tokens_enum(forks: &[(&IDOptionsFork, LineEnum)],
               enum_name: &str)
               -> Tokens {
	let enum_variants =
		forks.iter()
			.map(|(_, line_enum)|
				line_enum.variant_name());

	quote! {
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum $enum_name {
			$(SeparatedItems(enum_variants, ",\n"))
		}
	}
}

fn tokens_trait_impl<'a>(forks: &[(&IDOptionsFork, LineEnum)],
                         node: &IDNode,
                         enum_name: &str)
                         -> Tokens {
	let node_title = node.metadata.title.as_str();
	
	let forks_options =
		forks.iter()
			 .map(|(fork, line_enum)| {
				 let (first_line, _) = 
				    fork.options
					    .get(0)
					    .unwrap();
				 
				 let first_line_enum = OptionLineEnum {
					 node_title,
					 raw_id: first_line.line_id.as_str(),
				 };
				 
				 let first_tokens = quote!($(first_line_enum.qualified()).into());
				 
				 let others_tokens = 
				    fork.options
					    .iter()
						.skip(1)
					    .map(|(other_line, _)| {
						    let other_enum = OptionLineEnum {
							    node_title,
							    raw_id: other_line.line_id.as_str(),
						    };
						    
						    quote!($(other_enum.qualified()).into())
					    });
				 
				 quote! {
					 $(line_enum.typed_qualified()) => {
						 CountOrMore::new(
							 [$first_tokens], 
							 vec![
								 $(SeparatedItems(others_tokens, ",\n"))
							 ])
					 }
				 }
			 });

	quote! {
		impl OptionsForkTrait for $enum_name {
			fn options(&self) -> CountOrMore<1, OptionLine> {
				return match self {
					$(SeparatedItems(forks_options, "\n"))
				};
			}
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  node: &IDNode,
                  lines_map: &LinesMap)
                  -> Option<Tokens> {
	if lines_map.options_forks.is_empty() {
		return None;
	}
	
	let enum_name = 
		enums::enum_type_options_fork(&node.metadata.title);

	let tokens_imports =
		tokens_imports(cfg);
	let tokens_enum = 
		tokens_enum(&lines_map.options_forks, &enum_name);
	let tokens_trait_impl = 
		tokens_trait_impl(&lines_map.options_forks, node, &enum_name);

	Some(quote! {
		$tokens_imports
		
		$tokens_enum
		
		$tokens_trait_impl
	})
}