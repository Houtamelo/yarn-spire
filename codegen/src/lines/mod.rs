pub mod util;
mod raw;
mod macros;
pub mod grouping;

use anyhow::*;
use grouping::raw_lines_into_scopes;
use raw::parse_source_lines_into_raw_nodes;
use crate::lines::grouping::scope::YarnScope;
use crate::lines::raw::node_metadata::NodeMetadata;
use crate::UnparsedLine;

pub struct YarnNode {
	metadata: NodeMetadata,
	contents: Vec<YarnScope>,
}

pub fn parse_as_nodes(source_lines: Vec<UnparsedLine>) -> Result<Vec<YarnNode>> {
	let raw_nodes = 
		parse_source_lines_into_raw_nodes(source_lines)?;
	
	return
		raw_nodes
			.into_iter()
			.map(|node| {
				let scopes_result = 
					raw_lines_into_scopes(node.lines);

				scopes_result.map(|scope| {
					YarnNode {
						metadata: node.metadata,
						contents: scope,
					}
				})
			}).collect();
}
