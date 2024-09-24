use std::fs::File;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use genco::lang::Rust;
use genco::lang::rust::Tokens;

pub fn get_or_create_file(path: &PathBuf, allow_overwrite: bool) -> Result<File> {
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	
	if !path.exists() {
		File::options()
			.create(true)
			.write(true)
			.truncate(true)
			.open(path)
			.map_err(|err| anyhow!(
				"Could not create file at `{path:?}`.\n\
				 Error: {err}"))
	} else if !allow_overwrite {
		Err(anyhow!(
			"File already exists at `{path:?}`, but `allow_overwrite` is false.\n\n\
			 Help: You can enable overwriting by changing `allow_overwrite` to `true` in the config file."))
	} else {
		File::options()
			.create(false)
			.write(true)
			.truncate(true)
			.open(path)
			.map_err(|err| anyhow!(
				"Could not open file at `{path:?}`.\n\
				 Error: {err}"))
	}
}

pub fn write_to_file(path: &PathBuf, mut file: File, tokens: Tokens) -> Result<()> {
	let fmt = genco::fmt::Config::from_lang::<Rust>().with_indentation(genco::fmt::Indentation::Space(4));
	let config = genco::lang::rust::Config::default();
	let mut writer = genco::fmt::IoWriter::new(&mut file);
	tokens.format_file(&mut writer.as_formatter(&fmt), &config)
	      .map_err(|err| anyhow!("Could not write to `{path:?}`.\nError: {err}"))
}

pub fn delete_file_if_exists(path: &PathBuf) -> Result<()> {
	if path.exists() {
		std::fs::remove_file(path)
			.map_err(|err| anyhow!(
				"Could not delete file at `{path:?}`.\n\
				 Error: {err}"))
	} else {
		Ok(())
	}
}