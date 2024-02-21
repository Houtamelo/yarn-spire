#![feature(let_chains)]
#![feature(if_let_guard)]
#![allow(clippy::single_char_add_str)]
#![feature(string_remove_matches)]
#![feature(assert_matches)]
#![allow(clippy::bool_comparison)]
#![allow(clippy::needless_return)]
#![allow(dead_code)]
#![feature(coroutines)]
#![feature(pattern)]
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_expand)]

mod expressions;
mod lines;
mod io;

extern crate proc_macro;
use proc_macro::{TokenStream};
use quote::quote_spanned;
use anyhow::Result;
use proc_macro2::Span;
use lines::parse_as_nodes;
use crate::lines::grouping::quoted::Quoted;

type LineNumber = usize;
type Indent = isize;

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnparsedLine {
	line_number: LineNumber,
	text: String,
}

type ControllerType = String;
type CommandEnum = String;

#[proc_macro]
pub fn yarn_file(_input: TokenStream) -> TokenStream {
	let (source_lines, controller_type, command_enum) =
		io::read_file(_input);
	
	let scenes =
		match parse_as_nodes(source_lines) {
			Err(_err) => {
				panic!("{_err}");
			}
			Ok(yarn_lines) => yarn_lines,
		};
	
	let controller_ident = 
		syn::Ident::new(&controller_type, Span::call_site());
	let command_enum_ident = 
		syn::Ident::new(&command_enum, Span::call_site());
	
	let quoted_scenes_result =
		scenes
			.into_iter()
			.map(|scene| scene.quoted(&controller_ident, &command_enum_ident))
			.collect::<Result<Vec<_>>>();
	
	let quoted_scenes =
		match quoted_scenes_result {
			Ok(ok) => ok,
			Err(_err) => {
				panic!("{_err}");
			}
		};

	return quote_spanned! {Span::call_site() =>
		#(#quoted_scenes)*
	}.into();
}

