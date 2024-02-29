use std::fs::File;
use std::path::PathBuf;
use anyhow::anyhow;
use genco::lang::Rust;
use genco::lang::rust::Tokens;

pub fn get_or_create_file(path: &PathBuf, allow_overwrite: bool) -> anyhow::Result<File> {
	if !path.exists() {
		File::options()
			.create(true)
			.write(true)
			.truncate(true)
			.open(&path)
			.map_err(anyhow::Error::from)
	} else if !allow_overwrite {
		Err(anyhow!(
				"File already exists at `{path:?}`, but `allow_overwrite` is false.\n\n\
			     Help: You can enable overwriting by changing `allow_overwrite` to `true` in the config file."))
	} else {
		File::options()
			.create(false)
			.write(true)
			.truncate(true)
			.open(&path)
			.map_err(anyhow::Error::from)
	}
}

pub fn write_to_file(path: &PathBuf, mut file: File, tokens: Tokens) -> anyhow::Result<()> {
	let fmt =
		genco::fmt::Config::from_lang::<Rust>()
			.with_indentation(genco::fmt::Indentation::Space(4));

	let config =
		genco::lang::rust::Config::default();

	let mut writer =
		genco::fmt::IoWriter::new(&mut file);

	tokens.format_file(&mut writer.as_formatter(&fmt), &config)
	      .map_err(|err| anyhow!(
			"Could not write to `{path:?}`.\n\
			 Error: {err}"))
}