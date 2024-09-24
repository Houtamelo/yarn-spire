#![allow(clippy::type_complexity)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(string_remove_matches)]
#![feature(assert_matches)]
#![feature(coroutines)]
#![feature(pattern)]
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_expand)]
#![feature(iterator_try_collect)]
#![feature(extract_if)]
#![feature(coroutine_trait)]
#![feature(iter_from_coroutine)]
#![feature(gen_blocks)]

extern crate core;

mod expressions;
mod parsing;
mod io;
mod config;
mod quoting;

use anyhow::Result;
use io::read;
use parsing::parse_nodes;
use crate::config::YarnConfig;

type LineNumber = usize;
type Indent = isize;

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnparsedLine {
	line_number: LineNumber,
	text: String,
}

pub fn main() -> Result<()> {
	let config = 
		YarnConfig::parse_file()?;
	
	let yarn_files = 
		read::find_and_read_yarn_files(&config)?;
	
	let (nodes, var_declarations) =
		yarn_files
			.into_iter()
			.map(parse_nodes)
			.try_fold((vec![], vec![]), |(mut nodes_sum, mut vars_sum), node_result| {
				let (nodes, var_declarations) = node_result?;
				nodes_sum.extend(nodes);
				vars_sum.extend(var_declarations);
				Result::<_>::Ok((nodes_sum, vars_sum))
			})?;
	
	io::write::generate_and_write(&config, nodes, var_declarations)?;
	
	println!("Code generated successfully!");
	Ok(())
}

