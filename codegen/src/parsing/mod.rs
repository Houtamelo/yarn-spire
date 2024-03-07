pub mod util;
pub mod grouping;
pub mod raw;
pub mod macros;

use anyhow::*;
use grouping::parse_node_contents;
use raw::parse_raw_nodes;
use grouping::scope::YarnScope;
use raw::node_metadata::NodeMetadata;
use crate::io::read::YarnFile;
use crate::parsing::raw::var_declaration::VarDeclaration;

pub struct YarnNode {
	pub metadata: NodeMetadata,
	pub contents: Vec<YarnScope>,
}

pub fn parse_nodes(yarn_file: YarnFile) -> Result<(Vec<YarnNode>, Vec<VarDeclaration>)> {
	let (raw_nodes, var_declarations) = 
		parse_raw_nodes(yarn_file.lines)
			.map_err(|err| anyhow!(
				"Could not parse raw nodes from file.\n\
				 Path: {}\n\
				 Error: {err}", yarn_file.path.display())
			)?;

	let finished_nodes =
		raw_nodes
			.into_iter()
			.map(parse_node_contents)
			.try_collect()?;
	
	return Ok((finished_nodes, var_declarations));
}
