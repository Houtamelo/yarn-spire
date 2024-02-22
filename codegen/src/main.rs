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
#![feature(iterator_try_collect)]

mod expressions;
mod lines;
mod io;
mod config;

use anyhow::Result;
use lines::parse_as_nodes;
use crate::config::YarnConfig;
use crate::lines::YarnNode;

type LineNumber = usize;
type Indent = isize;

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnparsedLine {
	line_number: LineNumber,
	text: String,
}

pub fn main() -> Result<()> {
	let config = 
		YarnConfig::parse_file("/yarn_project.toml")?;
	
	let yarn_files = 
		io::find_and_read_yarn_files(config.yarn_folder(), config.folders_to_exclude())?;
	
	let nodes =
		yarn_files
			.into_iter()
			.map(|source_lines| 
				parse_as_nodes(source_lines))
			.try_fold(Vec::new(), |sum, node_result| {
				let node = node_result?;
				sum.extend(node);
				Result::<Vec<YarnNode>>::Ok(sum)
			})?;
	
	Ok(())
}

