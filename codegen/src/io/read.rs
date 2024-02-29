use std::path::PathBuf;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use houtamelo_utils::prelude::None;
use trim_in_place::TrimInPlace;
use crate::config::YarnConfig;
use crate::UnparsedLine;

type YarnFile = Vec<UnparsedLine>;

fn find_yarn_files(relative_path: &str,
                   exclude: &[PathBuf])
                   -> Result<Vec<PathBuf>> {
	let options = glob::MatchOptions {
		case_sensitive: true,
		require_literal_separator: false,
		require_literal_leading_dot: false,
	};

	let pattern =
		relative_path.to_string() + "/**/*.yarn";

	let path_iter =
		glob::glob_with(&pattern, options)
			.map_err(|err| anyhow!(
				"Could create glob pattern.\n\
				 Error: {err}"
			))?;

	let paths: Vec<PathBuf> =
		path_iter
			.into_iter()
			.try_collect()
			.map_err(|err| anyhow!(
				"Could not glob path.\n\
				 Error: {err}"
			))?;

	let files: Vec<PathBuf> =
		paths.into_iter()
		     .filter(|path|
			    exclude.iter().none(|excluded| path.starts_with(excluded)))
		     .collect();

	return Ok(files);
}

fn read_lines(file: File) -> Result<YarnFile> {
	let reader = BufReader::new(file);

	let source_lines: Vec<(usize, String)> =
		reader
			.lines()
			.enumerate()
			.map(|(line_number, result)|
				result
					.map(|text| (line_number, text))
					.map_err(|err| anyhow!(
						"Could not read line from file.\n\
				         Error: {err}"
					))
			).try_collect()?;
	
	let unparsed_lines =
		source_lines
			.into_iter()
			.filter_map(|(line_number, mut text)| {
				let comment_start = text.find("//");
				if let Some(start) = comment_start {
					text.truncate(start);
				}

				text.trim_end_in_place();

				if text.is_empty() || text.chars().all(char::is_whitespace) {
					None
				} else {
					Some(UnparsedLine { line_number, text })
				}
			}).collect();
	
	return Ok(unparsed_lines);
}

fn read_files(paths: Vec<PathBuf>) -> Result<Vec<YarnFile>> {
	paths.into_iter()
	     .map(|path|
			File::open(&path)
				.map_err(|err| anyhow!(
					 "Could not open file at path: {path:?}\n\
				      Error: {err}"
				 )))
	     .try_fold(Vec::new(), |mut sum, file| {
			let file = file?;
			let lines = read_lines(file)?;

			sum.push(lines);
			Ok(sum)
		})
}

pub fn find_and_read_yarn_files(cfg: &YarnConfig) -> Result<Vec<Vec<UnparsedLine>>> {
	let yarn_root_path =
		cfg.yarn_root_folder
		   .to_str()
		   .ok_or_else(|| anyhow!(
				"Could not convert `yarn_root_folder` to `str`.\n\
				 Path: {:?}\n\n\
				 Help: The current search algorithm requires utf-8 valid strings, but the provided path has non-utf-8 chars."
				, cfg.yarn_root_folder)
		   )?;
	
	let paths = 
		find_yarn_files(yarn_root_path, &cfg.exclude_yarn_folders)?;
	
	return read_files(paths);
}