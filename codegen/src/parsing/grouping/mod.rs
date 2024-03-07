use crate::parsing::grouping::scope::read_next_scope;
use crate::parsing::raw::RawNode;
use anyhow::{anyhow, Result};
use crate::parsing::YarnNode;

pub mod options;
pub mod if_branch;
pub mod scope;

pub fn parse_node_contents(node: RawNode) -> Result<YarnNode> {
	let raw_lines = node.lines;
	let mut stream_buffer =
		raw_lines
			.into_iter()
			.peekable();

	let mut contents = Vec::new();
	while stream_buffer.peek().is_some() {
		let next_scope_option =
			read_next_scope(-1, &mut stream_buffer)
				.map_err(|err| anyhow!(
					"Could not read next scope.\n\
					 Node: {}\n\
					 Error: {err}", node.metadata.title))?;

		if let Some(next_scope) = next_scope_option {
			contents.push(next_scope);
		}
	}

	return Ok(YarnNode {
		metadata: node.metadata,
		contents,
	});
}