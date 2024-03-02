use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Deserialize;

pub struct YarnConfig {
	pub storage_qualified: String,
	pub storage_direct: String,
	pub command_qualified: String,
	pub command_direct: String,
	pub shared_qualified: String,
	pub vars_qualified: String,
	pub allow_overwrite: bool,
	pub generate_storage: bool,
	pub destination_os_path: PathBuf,
	pub yarn_root_folder: PathBuf,
	pub exclude_yarn_folders: Vec<PathBuf>,
}

#[derive(Deserialize)]
struct DeserializableConfig {
	storage_module_path: String,
	storage_type_name: String,
	command_module_path: String,
	command_type_name: String,
	vars_module_path: String,
	allow_overwrite: bool,
	generate_storage: bool,
	destination_os_path: String,
	destination_module_path: String,
	yarn_root_folder: String,
	exclude_yarn_folders: Vec<String>,
}

fn read_file() -> Result<String> {
	let path_buf =
		PathBuf::from_str("yarn_project.toml")
			.map_err(|err| anyhow!(
				"Could not parse `Config` file path into `PathBuf`.\n\
				 Path: `yarn_project.toml`\n\
				 Error: `{err}`\n\n\
				 Help: Ensure the `Config` file path is a valid OS file-system path. (like `/src/dialogues/nodes/config.toml`)")
			)?;

	let file =
		std::fs::File::open(path_buf)
			.map_err(|err| anyhow!(
				"Could not open `Config` file.\
				 Path: `yarn_project.toml`\n\
				 Error: `{err}`\n\n\
				 Help: This program might need additional permissions to access this file.\n\
				 Help: Try running the program as an administrator.")
			)?;

	let decoded_file =
		DecodeReaderBytesBuilder::new()
			.encoding(None)
			.bom_sniffing(true)
			.build(file);

	let mut buffer = String::new();
	let mut reader =
		std::io::BufReader::new(decoded_file);

	reader.read_to_string(&mut buffer)
	      .map_err(|err| anyhow!(
			"Could not read `Config` file.\
			 Path: `yarn_project.toml`\n\
			 Error: `{err}`\n\n\
			 Help: This program might need additional permissions to access this file.\n\
			 Help: Try running the program as an administrator.")
	      )?;

	Ok(buffer)
}

fn deserialize(toml_input: String) -> Result<DeserializableConfig> {
	return toml::from_str(&toml_input)
		.map_err(|err| anyhow!(
			"Could not parse `Config` file as `DeserializableConfig`.\n\
			 Input: `{toml_input}`\n\
			 Error: `{err}`\n\n\
			 Ensure the file is in a valid format, example:\n\
			 ```toml\n\
			 # Storage variable's module path without the type name.
			  (This should be empty if `generate_storage` is `true`)
			 storage_path = \"crate::dialogues\"\n\
			 # Storage variable's type name. (do not include the path)
			 storage_type_name = \"MyVariablesStorage\"\n\
			 # Command's module path without the type name.
			 command_path = \"crate::yarn_command\"\n\
			 # Command's type name. (do not include the path)
			 command_type_name = \"MyYarnCommand\"\n\
			 # The module path of the structs that implement `YarnVar`.
			 vars_module_path = \"crate::dialogues::yarn_nodes::vars\"\n\
			 # If true, the program will overwrite files in the destination folder.
			 allow_overwrite = true\n\
			 # If true, the program will generate a Storage struct for you, using the variable declarations provided.
			 # The name of the generated struct will be the same as the `storage_type_name`.
			 generate_storage = true\n\
			 # The OS(Operational System) destination folder for the generated files.
			 destination_os_path = \"/src/nodes/quoting\"\n\
			 # The Rust module path of the destination folder.
			 destination_module_path = \"crate::dialogue::yarn_nodes\"\n\
			 # The root folder of the Yarn scripts this program will attempt to parse.
			 yarn_root_folder = \"../yarn_scripts\"\n\
			 # The folders inside `yarn_root_folder` that will be excluded from parsing.
			 exclude_yarn_folders = [\"test\", \"yarn.lock\", \"prototype\"]\n\
			 ```"));
}

fn ensure_not_empty(input: &DeserializableConfig) -> Result<(String, String)> {
	macro_rules! ensure_not_empty {
		($field:expr, $field_name:expr, $example:expr) => {
			if $field.is_empty() {
				return Err(anyhow!(
					"Config file is missing `{}` field.\n\n\
					 Help: You can declare the field like this:\n\
					 {} = {}", $field_name, $example, $field_name));
			}
		};
	}

	ensure_not_empty!(input.storage_type_name, "storage_type_name", "\"MyVariablesStorage\"");
	ensure_not_empty!(input.command_module_path, "command_module_path", "\"crate::yarn_commands\"");
	ensure_not_empty!(input.command_type_name, "command_type_name", "\"MyYarnCommand\"");
	ensure_not_empty!(input.destination_os_path, "destination_os_path", "\"/src/dialogues/nodes\"");
	ensure_not_empty!(input.destination_module_path, "destination_module_path", "\"crate::dialogues::nodes\"");
	ensure_not_empty!(input.yarn_root_folder, "yarn_root_folder", "\"../yarn_scripts\"");

	return match (input.storage_module_path.is_empty() || input.vars_module_path.is_empty(), input.generate_storage) {
		(true, true) => {
			let storage_qualified = 
				format!("{mod_path}::default_storage::{type_name}", 
					mod_path = input.destination_module_path, type_name = input.storage_type_name);
			
			let vars_qualified = 
				format!("{mod_path}::default_storage::vars",
					mod_path = input.destination_module_path);
			
			Ok((storage_qualified, vars_qualified))
		},
		(false, false) => {
			let storage_qualified =
				format!("{mod_path}::{type_name}",
					mod_path = input.storage_module_path, type_name = input.storage_type_name);
			
			let vars_qualified =
				format!("{mod_path}",
					mod_path = input.vars_module_path);
			
			Ok((storage_qualified, vars_qualified))
		},
		(true, false) => {
			Err(anyhow!(
				"Config file has `generate_storage` set to `false`, but either `storage_module_path` or `vars_module_path` is empty.\n\
				 Help: If `generate_storage` is false. Then:\n\
				  - `storage_path` should be the path to the storage type's module(without the type)\
				  - `vars_module_path` should be the path to the module that contains the structs that implement `YarnVar`."))
		},
		(false, true) => {
			Err(anyhow!(
				"Config file has `generate_storage` set to `true`, but either `storage_module_path` or `vars_module_path` is not empty.\n\
				 Help: If `generate_storage` is true, then `storage_module_path` and `vars_module_path` should both be empty. \
				 This is because this program will generate the storage struct and variables for you, then set their paths manually."))
		},
	};
}

impl YarnConfig {
	pub fn parse_file() -> Result<YarnConfig> {
		let toml_input = read_file()?;
		let toml = deserialize(toml_input)?;
		
		let (storage_qualified, vars_qualified) =
			ensure_not_empty(&toml)?;
		
		let command_qualified =
			format!("{mod_path}::{type_name}", 
				mod_path = &toml.command_module_path, type_name = &toml.command_type_name);
		
		let shared_qualified =
			format!("{mod_path}::shared_internal",
				mod_path = toml.destination_module_path);
		
		let destination_os_path =
			PathBuf::from_str(&toml.destination_os_path)
				.map_err(|err| anyhow!(
					"Could not parse `destination_os_path` into `PathBuf`.\n\
					 Path: `{}`\n\
					 Error: {err}\n\n\
					 Help: Ensure the `destination_os_path` is a valid OS file-system path. (like `src/dialogues/nodes/`)"
					, toml.destination_os_path)
				)?;
		
		std::fs::create_dir_all(&destination_os_path)
			.map_err(|err| anyhow!(
				"Directory `destination_os_path` does not exist or could not be created.\n\
				 Path: {destination_os_path:?}\n\
				 Error: {err}\n\n\
				 Help: This program might need additional permissions to access or create the directory.\n\
				 Help: Try running the program as an administrator.")
			)?;
		
		let yarn_root_folder =
			PathBuf::from_str(&toml.yarn_root_folder)
				.map_err(|err| anyhow!(
					"Could not parse `yarn_root_folder` into `PathBuf`.\n\
					 Path: `{}`\n\
					 Error: `{err}`\n\n\
					 Help: Ensure the `yarn_root_folder` is a valid OS file-system path. (like \"/../yarn_scripts\".)"
					, toml.yarn_root_folder)
				)?;

		let exclude_yarn_folders =
			toml.exclude_yarn_folders
			    .iter()
			    .map(|folder| 
				    yarn_root_folder.join(folder))
			    .collect();
		
		Ok(YarnConfig {
			storage_qualified,
			storage_direct: toml.storage_type_name,
			command_qualified,
			command_direct: toml.command_type_name,
			shared_qualified,
			vars_qualified,
			allow_overwrite: toml.allow_overwrite,
			generate_storage: toml.generate_storage,
			destination_os_path,
			yarn_root_folder,
			exclude_yarn_folders,
		})
	}
}