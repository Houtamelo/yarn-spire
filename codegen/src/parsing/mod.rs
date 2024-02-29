pub mod util;
pub mod grouping;
pub mod raw;
pub mod macros;

use anyhow::*;
use grouping::raw_lines_into_scopes;
use raw::parse_raw_nodes;
use grouping::scope::YarnScope;
use raw::node_metadata::NodeMetadata;
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::UnparsedLine;

pub struct YarnNode {
	pub metadata: NodeMetadata,
	pub contents: Vec<YarnScope>,
}

pub fn parse_as_nodes(source_lines: Vec<UnparsedLine>) -> Result<(Vec<YarnNode>, Vec<VarDeclaration>)> {
	let (raw_nodes, var_declarations) = 
		parse_raw_nodes(source_lines)?;

	let finished_nodes =
		raw_nodes
			.into_iter()
			.map(|node|
				raw_lines_into_scopes(node.lines)
					.map(|scope|
						YarnNode {
							metadata: node.metadata,
							contents: scope,
						}))
			.try_collect()?;
	
	return Ok((finished_nodes, var_declarations));
}
