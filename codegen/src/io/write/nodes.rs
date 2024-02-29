use anyhow::Result;
use crate::config::YarnConfig;
use crate::io::write::util::{get_or_create_file, write_to_file};
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use crate::quoting::template::nodes;

fn write_root_modules(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	nodes.iter()
		 .map(|node| node.metadata.title.as_str())
		 .try_for_each(|node_title| {
			 let path =
				 cfg.destination_os_path.join(format!("nodes/{node_title}/mod.rs"));
			 let file =
				 get_or_create_file(&path, cfg.allow_overwrite)?;
			 let tokens =
				 nodes::tokens_module_level(node_title);
			 
			 write_to_file(&path, file, tokens)
		 })
}

fn write_title_modules(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let inferred_tracking = 
		nodes::title::infer_all_nodes_tracking(nodes);
	
	inferred_tracking
		.into_iter()
		.try_for_each(|(node, tracking)| {
			let node_title = &node.metadata.title;
			
			let path =
				cfg.destination_os_path.join(format!("nodes/{node_title}/title.rs"));
			let file =
				get_or_create_file(&path, cfg.allow_overwrite)?;
			let tokens =
				nodes::title::all_tokens(cfg, node, tracking);
			
			write_to_file(&path, file, tokens)
		})
}

fn write_enum_any_modules(cfg: &YarnConfig,
                          nodes_lines: &[(&IDNode, LinesMap)])
                          -> Result<()> {
	nodes_lines
		.iter()
		.try_for_each(|(node, lines_map)| {
			let node_title = &node.metadata.title;
			
			let path =
				cfg.destination_os_path.join(format!("nodes/{node_title}/enum_any.rs"));
			let file =
				get_or_create_file(&path, cfg.allow_overwrite)?;
			let tokens =
				nodes::enums::any::all_tokens(cfg, node_title, lines_map);
			
			write_to_file(&path, file, tokens)
		})
}

fn write_enum_command_modules(cfg: &YarnConfig,
							  nodes_lines: &[(&IDNode, LinesMap)])
							  -> Result<()> {
	nodes_lines
		.iter()
		.try_for_each(|(node, lines_map)| {
			let node_title = &node.metadata.title;
			
			let path =
				cfg.destination_os_path.join(format!("nodes/{node_title}/enum_command.rs"));
			let file =
				get_or_create_file(&path, cfg.allow_overwrite)?;
			let tokens =
				nodes::enums::command::all_tokens(cfg, node, lines_map);
			
			write_to_file(&path, file, tokens)
		})
}

fn write_enum_options_fork_modules(cfg: &YarnConfig,
								   nodes_lines: &[(&IDNode, LinesMap)])
								   -> Result<()> {
	nodes_lines
		.iter()
		.try_for_each(|(node, lines_map)| {
			let node_title = &node.metadata.title;
			
			let path =
				cfg.destination_os_path.join(format!("nodes/{node_title}/enum_options_fork.rs"));
			let file =
				get_or_create_file(&path, cfg.allow_overwrite)?;
			let tokens =
				nodes::enums::options_fork::all_tokens(cfg, node, lines_map);
			
			write_to_file(&path, file, tokens)
		})
}

fn write_enum_speech_modules(cfg: &YarnConfig,
							 nodes_lines: &[(&IDNode, LinesMap)])
							 -> Result<()> {
	nodes_lines
		.iter()
		.try_for_each(|(node, lines_map)| {
			let node_title = &node.metadata.title;
			
			let path =
				cfg.destination_os_path.join(format!("nodes/{node_title}/enum_speech.rs"));
			let file =
				get_or_create_file(&path, cfg.allow_overwrite)?;
			let tokens =
				nodes::enums::speech::all_tokens(cfg, node, lines_map);
			
			write_to_file(&path, file, tokens)
		})
}

fn write_enum_option_line_modules(cfg: &YarnConfig,
								  nodes_lines: &[(&IDNode, LinesMap)])
								  -> Result<()> {
	nodes_lines
		.iter()
		.try_for_each(|(node, lines_map)| {
			let node_title = &node.metadata.title;
			
			let path =
				cfg.destination_os_path.join(format!("nodes/{node_title}/enum_option_line.rs"));
			let file =
				get_or_create_file(&path, cfg.allow_overwrite)?;
			let tokens =
				nodes::enums::option_line::all_tokens(cfg, node, lines_map);
			
			write_to_file(&path, file, tokens)
		})
}

pub fn write_all(cfg: &YarnConfig,
                 nodes: &[IDNode])
                 -> Result<()> {
	let nodes_lines =
		nodes.iter()
		     .map(|node| {
			     (node, node.map_lines())
		     }).collect::<Vec<_>>();
	
	write_root_modules(cfg, nodes)?;
	write_title_modules(cfg, nodes)?;
	write_enum_any_modules(cfg, &nodes_lines)?;
	write_enum_command_modules(cfg, &nodes_lines)?;
	write_enum_options_fork_modules(cfg, &nodes_lines)?;
	write_enum_speech_modules(cfg, &nodes_lines)?;
	write_enum_option_line_modules(cfg, &nodes_lines)
}