pub mod util;
pub mod core_types;
pub mod nodes;

use crate::config::YarnConfig;
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::parsing::YarnNode;
use crate::quoting::quotable_types::line_ids::{convert_to_id_nodes, BuiltInCommand, IDFlatLine};
use crate::quoting::quotable_types::node::IDNode;
use crate::quoting::quotable_types::scope::IDScope;
use anyhow::{anyhow, Result};
use std::collections::HashSet;

fn check_nodes_in_jumps(nodes: &[IDNode]) -> Result<()> {
	let built_nodes: HashSet<&str> = nodes
		.iter()
		.map(|node| node.metadata.title.as_str())
		.collect();

	let nodes_in_jump = nodes
		.iter()
		.flat_map(|node|
			node.scopes
			    .iter()
			    .flat_map(IDScope::iter_flat_lines)
			    .filter_map(|line|
				    if let IDFlatLine::BuiltInCommand(BuiltInCommand::Jump { node_destination_title, .. }) = line {
					    Some(node_destination_title.as_str())
				    } else {
					    None
				    }))
		.collect::<Vec<&str>>();

	let nodes_not_found = nodes_in_jump
		.into_iter()
		.filter(|node| !built_nodes.contains(node))
		.collect::<Vec<_>>();

	if nodes_not_found.is_empty() {
		Ok(())
	} else {
		Err(anyhow!(
			"Some nodes mentioned in `jump`(`<<jump [node_name]>>`) commands are not present in the provided files.\n\
			 Node names: {}\n\
			 Make sure that the node names are correct.", nodes_not_found.join(", ")))
	}
}

pub fn generate_and_write(
	config: &YarnConfig,
	nodes: Vec<YarnNode>,
	var_declarations: Vec<VarDeclaration>,
) -> Result<()> {
	let id_nodes = convert_to_id_nodes(nodes)?;

	check_nodes_in_jumps(&id_nodes)?;

	let nodes_mapped = id_nodes
		.iter()
		.map(|node| (node, node.map_lines()))
		.collect::<Vec<_>>();

	core_types::write_all(config, &id_nodes, &nodes_mapped, &var_declarations)?;
	nodes::write_all(config, &id_nodes, &nodes_mapped)?;

	Ok(())
}