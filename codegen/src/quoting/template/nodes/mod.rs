pub mod title;
pub mod enums;

use genco::prelude::rust::Tokens;
use genco::quote;
use crate::quoting::quotable_types::enums::{
	enum_type_any, 
	enum_type_command, 
	enum_type_option_line, 
	enum_type_options_fork, 
	enum_type_speech
};

pub fn tokens_module_level(node_title: &str) -> Tokens {
	let any_enum = enum_type_any(node_title);
	let speech_enum = enum_type_speech(node_title);
	let command_enum = enum_type_command(node_title);
	let options_fork_enum = enum_type_options_fork(node_title);
	let option_line_enum = enum_type_option_line(node_title);

	quote! {
		pub mod title;
		pub use title::$node_title;
		
		pub mod enum_any;
		pub mod enum_command;
		pub mod enum_option_line;
		pub mod enum_options_fork;
		pub mod enum_speech;
		
		pub use enum_any::$any_enum;
		pub use enum_command::$command_enum;
		pub use enum_option_line::$option_line_enum;
		pub use enum_options_fork::$options_fork_enum;
		pub use enum_speech::$speech_enum;
	}
}