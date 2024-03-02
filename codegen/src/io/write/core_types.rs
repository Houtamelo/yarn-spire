use quoting::core_types;
use quoting::core_types::command_line;
use crate::config::YarnConfig;
use crate::io::write::util::{get_or_create_file, write_to_file};
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::quoting;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use anyhow::Result;
use core_types::{default_storage, instruction, options, runtime, speech, title, var_trait};

fn write_root(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("mod.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		core_types::tokens_root_module(cfg, nodes);
	return write_to_file(&path, file, tokens);
}

fn write_built_in_functions(cfg: &YarnConfig) -> Result<()> {
	let path =
		cfg.destination_os_path.join("built_in_functions.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		core_types::built_in_functions::all_tokens();
	return write_to_file(&path, file, tokens);
}

fn write_command(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("command_line.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		command_line::all_tokens(cfg, nodes_mapped);
	return write_to_file(&path, file, tokens);
}

fn write_default_storage(cfg: &YarnConfig,
                         nodes: &[IDNode],
                         var_declarations: &[VarDeclaration])
                         -> Result<()> {
	let tokens =
		default_storage::all_tokens(cfg, nodes, var_declarations)?;
	let path =
		cfg.destination_os_path.join("default_storage.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	return write_to_file(&path, file, tokens);
}

fn write_instruction(cfg: &YarnConfig) -> Result<()> {
	let path =
		cfg.destination_os_path.join("instruction.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		instruction::all_tokens(cfg);
	return write_to_file(&path, file, tokens);
}

fn write_options(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("options.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		options::all_tokens(cfg, nodes_mapped);
	return write_to_file(&path, file, tokens);
}

fn write_runtime(cfg: &YarnConfig) -> Result<()> {
	let path =
		cfg.destination_os_path.join("runtime.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		runtime::all_tokens(cfg);
	return write_to_file(&path, file, tokens);
}

fn write_speech(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("speech.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		speech::all_tokens(cfg, nodes_mapped);
	return write_to_file(&path, file, tokens);
}

fn write_title(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("title.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		title::all_tokens(cfg, nodes);
	return write_to_file(&path, file, tokens);
}

fn write_var_trait(cfg: &YarnConfig) -> Result<()> {
	let path =
		cfg.destination_os_path.join("var_trait.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		var_trait::all_tokens(cfg);
	return write_to_file(&path, file, tokens);
}

pub fn write_all(config: &YarnConfig,
				 nodes: &[IDNode],
				 nodes_mapped: &[(&IDNode, LinesMap)],
				 var_declarations: &[VarDeclaration])
				 -> Result<()> {
	write_root(&config, nodes)?;
	write_built_in_functions(&config)?;
	write_command(&config, nodes_mapped)?;
	write_instruction(&config)?;
	write_options(&config, nodes_mapped)?;
	write_runtime(&config)?;
	write_speech(&config, nodes_mapped)?;
	write_title(&config, nodes)?;
	write_var_trait(&config)?;
	
	if config.generate_storage {
		write_default_storage(&config, nodes, var_declarations)?;
	}
	
	return Ok(());
}