use crate::config::YarnConfig;
use crate::io::write::util::{delete_file_if_exists, get_or_create_file, write_to_file};
use crate::quoting::core_types::nodes;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use anyhow::Result;
use nodes::{enums, title};

fn write_all_nodes_root(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let tokens = nodes::tokens_all_nodes_root(nodes);
	let path = cfg.destination_os_path.join("nodes/mod.rs");
	let file = get_or_create_file(&path, cfg.allow_overwrite)?;
	write_to_file(&path, file, tokens)
}

fn write_node_specific_roots(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	nodes_mapped.iter().try_for_each(|(node, lines_map)| {
		let tokens = nodes::tokens_node_root(node, lines_map);
		let path = cfg.destination_os_path.join(format!("nodes/{title}/mod.rs", title = &node.metadata.title));
		let file = get_or_create_file(&path, cfg.allow_overwrite)?;
		write_to_file(&path, file, tokens)
	})
}

fn write_title_modules(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let inferred_tracking = title::infer_all_nodes_tracking(nodes)?;

	inferred_tracking
		.into_iter()
		.try_for_each(|(node, tracking)| {
			let tokens = title::all_tokens(cfg, node, tracking);
			let path = cfg.destination_os_path.join(format!("nodes/{title}/title.rs", title = &node.metadata.title));
			let file = get_or_create_file(&path, cfg.allow_overwrite)?;
			write_to_file(&path, file, tokens)
		})
}

/*
fn write_enum_any_modules(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	nodes_mapped.iter().try_for_each(|(node, lines_map)| {
		let path = cfg
			.destination_os_path
			.join(format!("nodes/{title}/enum_any.rs", title = &node.metadata.title));

		if let Some(tokens) = enums::any::all_tokens(cfg, node, lines_map) {
			let file = get_or_create_file(&path, cfg.allow_overwrite)?;
			write_to_file(&path, file, tokens)
		} else if cfg.allow_overwrite {
			delete_file_if_exists(&path)
		} else {
			Ok(())
		}
	})
}
*/

fn write_enum_command_modules(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	nodes_mapped.iter().try_for_each(|(node, lines_map)| {
		let path = cfg
			.destination_os_path
			.join(format!("nodes/{title}/enum_command.rs", title = &node.metadata.title));

		if let Some(tokens) = enums::command::all_tokens(cfg, node, lines_map) {
			let file = get_or_create_file(&path, cfg.allow_overwrite)?;
			write_to_file(&path, file, tokens)
		} else if cfg.allow_overwrite {
			delete_file_if_exists(&path)
		} else {
			Ok(())
		}
	})
}

fn write_enum_options_fork_modules(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	nodes_mapped.iter().try_for_each(|(node, lines_map)| {
		let path = cfg
			.destination_os_path
			.join(format!("nodes/{title}/enum_options_fork.rs", title = &node.metadata.title));

		if let Some(tokens) = enums::options_fork::all_tokens(cfg, node, lines_map) {
			let file = get_or_create_file(&path, cfg.allow_overwrite)?;
			write_to_file(&path, file, tokens)
		} else if cfg.allow_overwrite {
			delete_file_if_exists(&path)
		} else {
			Ok(())
		}
	})
}

fn write_enum_speech_modules(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	nodes_mapped.iter().try_for_each(|(node, lines_map)| {
		let path = cfg
			.destination_os_path
			.join(format!("nodes/{title}/enum_speech.rs", title = &node.metadata.title));

		if let Some(tokens) = enums::speech::all_tokens(cfg, node, lines_map) {
			let file = get_or_create_file(&path, cfg.allow_overwrite)?;
			write_to_file(&path, file, tokens)
		} else if cfg.allow_overwrite {
			delete_file_if_exists(&path)
		} else {
			Ok(())
		}
	})
}

fn write_enum_option_line_modules(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	nodes_mapped.iter().try_for_each(|(node, lines_map)| {
		let path = cfg
			.destination_os_path
			.join(format!("nodes/{title}/enum_option_line.rs", title = &node.metadata.title));

		if let Some(tokens) = enums::option_line::all_tokens(cfg, node, lines_map) {
			let file = get_or_create_file(&path, cfg.allow_overwrite)?;
			write_to_file(&path, file, tokens)
		} else if cfg.allow_overwrite {
			delete_file_if_exists(&path)
		} else {
			Ok(())
		}
	})
}

pub fn write_all(
	cfg: &YarnConfig,
	nodes: &[IDNode],
	nodes_mapped: &[(&IDNode, LinesMap)],
) -> Result<()> {
	write_all_nodes_root(cfg, nodes)?;
	write_node_specific_roots(cfg, nodes_mapped)?;
	write_title_modules(cfg, nodes)?;
	//write_enum_any_modules(cfg, nodes_mapped)?;
	write_enum_command_modules(cfg, nodes_mapped)?;
	write_enum_options_fork_modules(cfg, nodes_mapped)?;
	write_enum_speech_modules(cfg, nodes_mapped)?;
	write_enum_option_line_modules(cfg, nodes_mapped)
}