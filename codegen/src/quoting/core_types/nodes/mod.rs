pub mod title;
pub mod enums;

use genco::prelude::rust::Tokens;
use genco::quote;
use crate::quoting::util::SeparatedItems;
use crate::quoting::quotable_types::enums::{
	enum_type_any, 
	enum_type_command, 
	enum_type_option_line, 
	enum_type_options_fork, 
	enum_type_speech
};
use crate::quoting::quotable_types::node::{IDNode, LinesMap};

pub fn tokens_all_nodes_root(nodes: &[IDNode]) -> Tokens {
	let module_declarations = 
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! { pub mod $title; }
		     });
	
	let module_exports =
		nodes.iter()
		     .map(|node| {
			     let title = &node.metadata.title;
			     quote! { pub use $title::*; }
		     });
	
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		$(SeparatedItems(module_declarations, "\n"))
		$(SeparatedItems(module_exports, "\n"))
	}
}

pub fn tokens_node_root(node: &IDNode, lines_map: &LinesMap) -> Tokens {
	let node_title =
		node.metadata.title.as_str();
	
	let any_enum_tokens =
		if lines_map.speeches.len() > 0 
		|| lines_map.commands.len() > 0
		|| lines_map.options_forks.len() > 0 {
			let any_enum = enum_type_any(&node.metadata.title);
			quote! { 
				pub mod enum_any;
				pub use enum_any::$any_enum;
			}
		} else {
			Tokens::new()
		};
	
	let speech_enum_tokens = 
		if lines_map.speeches.len() > 0 {
			let speech_enum = enum_type_speech(&node.metadata.title);
			quote! { 
				pub mod enum_speech;
				pub use enum_speech::$speech_enum;
			}
		} else {
			Tokens::new()
		};
	
	let command_enum_tokens =
		if lines_map.commands.len() > 0 {
			let command_enum = enum_type_command(&node.metadata.title);
			quote! { 
				pub mod enum_command;
				pub use enum_command::$command_enum;
			}
		} else {
			Tokens::new()
		};
	
	let options_fork_enum_tokens =
		if lines_map.options_forks.len() > 0 {
			let options_fork_enum = enum_type_options_fork(&node.metadata.title);
			quote! { 
				pub mod enum_options_fork;
				pub use enum_options_fork::$options_fork_enum;
			}
		} else {
			Tokens::new()
		};
	
	let option_line_enum_tokens =
		if lines_map.option_lines.len() > 0 {
			let option_line_enum = enum_type_option_line(&node.metadata.title);
			quote! { 
				pub mod enum_option_line;
				pub use enum_option_line::$option_line_enum;
			}
		} else {
			Tokens::new()
		};

	quote! {
		pub mod title;
		pub use title::$node_title;
		
		$any_enum_tokens
		$speech_enum_tokens
		$command_enum_tokens
		$options_fork_enum_tokens
		$option_line_enum_tokens
	}
}