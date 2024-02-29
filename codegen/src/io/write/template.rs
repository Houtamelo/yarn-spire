use quoting::template;
use quoting::template::command_line;
use crate::config::YarnConfig;
use crate::io::write::util::{get_or_create_file, write_to_file};
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::quoting;
use crate::quoting::quotable_types::node::IDNode;
use anyhow::Result;
use template::{default_storage, instruction, options, runtime, speech, title, var_trait};

fn write_root(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/mod.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		template::tokens_root_module(cfg, nodes);
	return write_to_file(&path, file, tokens);
}

fn write_command(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/command_line.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		command_line::all_tokens(cfg, nodes);
	return write_to_file(&path, file, tokens);
}

fn write_default_storage(cfg: &YarnConfig, var_declarations: &[VarDeclaration]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/default_storage.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		default_storage::all_tokens(cfg, var_declarations);
	return write_to_file(&path, file, tokens);
}

fn write_instruction(cfg: &YarnConfig) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/instruction.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		instruction::all_tokens(cfg);
	return write_to_file(&path, file, tokens);
}

fn write_options(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/options.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		options::all_tokens(cfg, nodes);
	return write_to_file(&path, file, tokens);
}

fn write_runtime(cfg: &YarnConfig) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/runtime.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		runtime::all_tokens(cfg);
	return write_to_file(&path, file, tokens);
}

fn write_speech(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/speech.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		speech::all_tokens(cfg, nodes);
	return write_to_file(&path, file, tokens);
}

fn write_title(cfg: &YarnConfig, nodes: &[IDNode]) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/title.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		title::all_tokens(cfg, nodes);
	return write_to_file(&path, file, tokens);
}

fn write_var_trait(cfg: &YarnConfig) -> Result<()> {
	let path =
		cfg.destination_os_path.join("yarn_nodes/var_trait.rs");
	let file =
		get_or_create_file(&path, cfg.allow_overwrite)?;
	let tokens =
		var_trait::all_tokens(cfg);
	return write_to_file(&path, file, tokens);
}

pub fn write_all(config: &YarnConfig,
				 nodes: &[IDNode],
				 var_declarations: &[VarDeclaration])
				 -> Result<()> {
	write_root(&config, nodes)?;
	write_command(&config, nodes)?;
	write_instruction(&config)?;
	write_options(&config, nodes)?;
	write_runtime(&config)?;
	write_speech(&config, nodes)?;
	write_title(&config, nodes)?;
	write_var_trait(&config)?;
	
	if config.generate_storage {
		write_default_storage(&config, var_declarations)?;
	}
	
	return Ok(());
}