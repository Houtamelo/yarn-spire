pub mod util;
pub mod template;
pub mod nodes;

use anyhow::Result;
use crate::config::YarnConfig;
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::parsing::YarnNode;
use crate::quoting::quotable_types::line_ids::convert_to_id_nodes;

pub fn generate_and_write(config: &YarnConfig,
                          nodes: Vec<YarnNode>,
                          var_declarations: Vec<VarDeclaration>)
                          -> Result<()> {
	let id_nodes = 
		convert_to_id_nodes(nodes)?;
	
	template::write_all(config, &id_nodes, &var_declarations)?;
	nodes::write_all(config, &id_nodes)?;
	
	Ok(())
}