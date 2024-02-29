use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use genco::lang::rust;
use genco::prelude::rust::Import;
use serde::Deserialize;

pub struct YarnConfig {
	pub storage_qualified: Import,
	pub storage_direct: Import,
	pub command_qualified: Import,
	pub command_direct: Import,
	pub shared_qualified: Import,
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
	allow_overwrite: bool,
	generate_storage: bool,
	destination_os_path: String,
	destination_module_path: String,
	yarn_root_folder: String,
	exclude_yarn_folders: Vec<String>,
}

impl YarnConfig {
	fn read_file(path: &str) -> Result<String> {
		let file = std::fs::File::open(path)
			.map_err(|err| anyhow!(
				"Could not open `Config` file.\
				 Path: `{path}`\n\
				 Error: `{err}`\n\n\
				 Help: This program might need additional permissions to access this file.\n\
				 Help: Try running the program as an administrator."
			))?;
		
		let mut buffer = String::new();
		let mut reader = std::io::BufReader::new(file);
		reader.read_to_string(&mut buffer)
			.map_err(|err| anyhow!(
				"Could not read `Config` file.\
				 Path: `{path}`\n\
				 Error: `{err}`\n\n\
				 Help: This program might need additional permissions to access this file.\n\
				 Help: Try running the program as an administrator."
			))?;
		
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
	
	fn ensure_not_empty(input: &DeserializableConfig) -> Result<Import> {
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
		
		return match (input.storage_module_path.is_empty(), input.generate_storage) {
			(true, true) => {
				Ok(rust::import(format!("{}::default_storage", input.destination_module_path), &input.storage_type_name))
			},
			(false, false) => {
				Ok(rust::import(&input.storage_module_path, &input.storage_type_name))
			},
			(true, false) => {
				Err(anyhow!(
					"Config file has `generate_storage` set to `false`, but `storage_module_path` is empty.\n\
					 storage_module_path: `{}`\n\
					 Help: If `generate_storage` is false, then `storage_path` should be the path to the storage type's module(without the type)."
					, input.storage_module_path))
			},
			(false, true) => {
				Err(anyhow!(
					"Config file has `generate_storage` set to `true`, but `storage_module_path` is not empty.\n\
					 storage_module_path: `{}`\n\
					 Help: If `generate_storage` is true, then `storage_path` should be empty. \
					 This is because this program will generate the storage struct for you and set it's path manually."
					, input.storage_module_path))
			},
		};
	}

	pub fn parse_file(path: &str) -> Result<YarnConfig> {
		let toml_input = 
			Self::read_file(path)?;
		let toml =
			Self::deserialize(toml_input)?;

		let storage_import = 
			Self::ensure_not_empty(&toml)?;
		let storage_direct =
			storage_import.clone().direct();
		let storage_qualified = 
			storage_import.qualified();
		
		let command_qualified =
			rust::import(&toml.command_module_path, &toml.command_type_name).qualified();
		let command_direct =
			rust::import(toml.command_module_path, toml.command_type_name).direct();
		
		let shared_qualified =
			rust::import(toml.destination_module_path, "shared_internal");
		
		let destination_os_path =
			PathBuf::from_str(&toml.destination_os_path)
				.map_err(|err| anyhow!(
					"Could not parse `destination_os_path` into `PathBuf`.\n\
					 Path: `{}`\n\
					 Error: {err}\n\n\
					 Help: Ensure the `destination_os_path` is a valid OS file-system path. (like `/src/dialogues/nodes/`)"
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
					 Help: Ensure the `yarn_root_folder` is in the format \"../yarn_scripts\"."
					, toml.yarn_root_folder)
				)?;

		let exclude_yarn_folders =
			toml.exclude_yarn_folders
			    .iter()
			    .map(|folder| 
					PathBuf::from_str(folder)
						.map_err(|err| anyhow!(
							"Could not parse a folder in `exclude_yarn_folders` into `PathBuf`.\n\
							 Path: `{folder}`\n\
							 Error: `{err}`\n\n\
							 Help: Ensure the folders in `exclude_yarn_folders` have paths in relation to `yarn_root_folder`."
						)))
			    .try_collect()?;
		
		Ok(YarnConfig {
			storage_qualified,
			storage_direct,
			command_qualified,
			command_direct,
			shared_qualified,
			allow_overwrite: toml.allow_overwrite,
			generate_storage: toml.generate_storage,
			destination_os_path,
			yarn_root_folder,
			exclude_yarn_folders,
		})
	}
}