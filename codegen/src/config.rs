use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use genco::lang::rust;
use serde::Deserialize;

pub struct YarnConfig {
	storage_import: rust::Import,
	command_import: rust::Import,
	yarn_folder: PathBuf,
	destination_module: PathBuf,
	allow_overwrite: bool,
	folders_to_exclude: Vec<PathBuf>,
}

impl YarnConfig {
	pub fn storage_import(&self) -> &rust::Import { &self.storage_import }
	pub fn command_import(&self) -> &rust::Import { &self.command_import }
	pub fn yarn_folder(&self) -> &PathBuf { &self.yarn_folder }
	pub fn destination_module(&self) -> &PathBuf { &self.destination_module }
	pub fn allow_overwrite(&self) -> &bool { &self.allow_overwrite }
	pub fn folders_to_exclude(&self) -> &Vec<PathBuf> { &self.folders_to_exclude }
}

#[derive(Deserialize)]
struct DeserializableConfig {
	storage_path: String,
	command_path: String,
	yarn_folder: String,
	destination_module: String,
	allow_overwrite: bool,
	folders_to_exclude: Vec<String>,
}

impl YarnConfig {
	fn read_file(path: &str) -> Result<String> {
		let file = std::fs::File::open(path)
			.map_err(|err| anyhow!(
				"Could not open `Config` file.\
				 Path: `{path}`\n\
				 Error: `{err}`\n\n\
				 Help: This program might need additional permissions to access this file."
			))?;
		
		let mut buffer = String::new();
		let reader = std::io::BufReader::new(file);
		reader.read_to_string(&mut buffer)
			.map_err(|err| anyhow!(
				"Could not read `Config` file.\
				 Path: `{path}`\n\
				 Error: `{err}`\n\n\
				 Help: This program might need additional permissions to access this file."
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
				 storage_path = \"crate::MyStorage\"\n\
				 command_path = \"crate::command::Command\"\n\
				 yarn_folder = \"../yarn_scripts\"\n\
				 destination_module = \"/src/nodes/mod.rs\"\n\
				 allow_overwrite = \"true\"\n\
				 folders_to_exclude = [\"/Test\", \"/Unfinished\"]\n\
				 ```"
			));
	}
	
	fn ensure_not_empty(input: &DeserializableConfig) -> Result<()> {
		macro_rules! ensure_not_empty {
				($field:expr, $field_name:expr, $example:expr) => {
					if $field.is_empty() {
						return Err(anyhow!(
							"Config file is missing `{}` field.\n\n\
							 Help: You can declare the field like this:\n\
							 {} = {}", $field_name, $example, $field_name
						));
					}
				};
			}
		
		ensure_not_empty!(input.storage_path, "storage_path", "\"crate::MyStorage\"");
		ensure_not_empty!(input.command_path, "command_path", "\"crate::command::Command\"");
		ensure_not_empty!(input.yarn_folder, "yarn_folder", "\"../yarn_scripts\"");
		ensure_not_empty!(input.destination_module, "destination_module", "\"/src/nodes/mod.rs\"");
		ensure_not_empty!(input.allow_overwrite.to_string(), "allow_overwrite", "\"true\"");
		Ok(())
	}
	
	fn split_path_name(type_path: &str) -> Result<(&str, &str)> {
		return 
			type_path
				.rsplit_once("::")
				.ok_or(anyhow!(
					"Could not split `type_path` into `module_path` and `type_name`.\n\
					 type_path: `{type_path}`\n\n\
					 Help: Ensure the `type_path` is in the format `crate::..::module::Type`."
				));
	}

	pub fn parse_file(path: &str) -> Result<YarnConfig> {
		let toml_input = Self::read_file(path)?;
		let deserialized = Self::deserialize(toml_input)?;

		Self::ensure_not_empty(&deserialized)?;
		
		let (storage_module_path, storage_name) = 
			Self::split_path_name(&deserialized.storage_path)?;
		
		let storage_import =
			rust::import(storage_module_path, storage_name);
		
		let (command_module_path, command_name) =
			Self::split_path_name(&deserialized.command_path)?;
		
		let command_import =
			rust::import(command_module_path, command_name);
		
		let yarn_folder =
			PathBuf::from_str(&deserialized.yarn_folder)
				.map_err(|err| anyhow!(
					"Could not parse `yarn_folder` into `PathBuf`.\n\
					 yarn_folder: `{}`\n\
					 Error: `{err}`\n\n\
					 Help: Ensure the `yarn_folder` is in the format \"../yarn_scripts\"."
					, deserialized.yarn_folder
				))?;
		
		let destination_module =
			PathBuf::from_str(&deserialized.destination_module)
				.map_err(|err| anyhow!(
					"Could not parse `destination_module` into `PathBuf`.\n\
					 destination_module: `{}`\n\
					 Error: `{err}`\n\n\
					 Help: Ensure the `destination_module` is in the format \"/src/nodes/mod.rs\"."
					, deserialized.destination_module
				))?;
		
		let folders_to_exclude =
			deserialized.folders_to_exclude
				.iter()
				.map(|folder| 
					PathBuf::from_str(folder)
						.map_err(|err| anyhow!(
							"Could not parse a folder in `folders_to_exclude` into `PathBuf`.\n\
							 folder: `{folder}`\n\
							 Error: `{err}`\n\n\
							 Help: Ensure the folders in `folders_to_exclude` have paths in relation to `yarn_folder`."
						)))
				.collect::<Result<Vec<_>>>()?;
		
		return Ok(YarnConfig {
			storage_import,
			command_import,
			yarn_folder,
			destination_module,
			allow_overwrite: deserialized.allow_overwrite,
			folders_to_exclude,
		});
	}
}