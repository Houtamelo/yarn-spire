use std::path::PathBuf;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use encoding_rs_io::DecodeReaderBytesBuilder;
use houtamelo_utils::prelude::None;
use trim_in_place::TrimInPlace;
use crate::config::YarnConfig;
use crate::UnparsedLine;

pub struct YarnFile {
	pub path: PathBuf,
	pub lines: Vec<UnparsedLine>,
}

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

enum State {
	OutsideLiteral,
	OutsideLiteralIgnoreNext,
	InsideLiteral,
	InsideLiteralIgnoreNext,
}

fn filter_comments(line: &mut String) {
	let mut state = 
		State::OutsideLiteral;
	
	let mut iter = 
		line.char_indices()
			.peekable();
	
	while let Some((index, ch)) = iter.next() {
		match state {
			State::InsideLiteral => {
				match ch {
					'\\' => {
						state = State::InsideLiteralIgnoreNext;
					}
					'\"' => {
						state = State::OutsideLiteral;
					}
					_ => {}
				}
			}
			State::InsideLiteralIgnoreNext => {
				state = State::InsideLiteral;
			}
			State::OutsideLiteral => {
				match ch {
					'\\' => {
						state = State::OutsideLiteralIgnoreNext;
					}
					'\"' => {
						state = State::InsideLiteral;
					}
					'/' => {
						if iter.next_if(|(_, ch)| *ch == '/').is_some() {
							line.truncate(index);
							return;
						}
					}
					_ => {}
				}
			}
			State::OutsideLiteralIgnoreNext => {
				state = State::OutsideLiteral;
			}
		}
	}
}

#[test]
fn test_filter() {
	use pretty_assertions::{assert_eq};
	macro_rules! assert_filter {
	    ($input: literal, $expect: literal) => {
		    let mut input = $input.to_string();
		    filter_comments(&mut input);
		    assert_eq!(&input, $expect);
	    };
	}
	
	assert_filter!(
		"//", 
		"");
	assert_filter!(
		"Normal line but // ignore these", 
		"Normal line but ");
	assert_filter!(
		"Normal line but // ignore these\n", 
		"Normal line but ");
	assert_filter!(
		"\\/\\/", 
		"\\/\\/");
	assert_filter!(
		"Line with some backslashes \\/\\/", 
		"Line with some backslashes \\/\\/");
	assert_filter!(
		"Speaker: Line with some backslashes \\/\\/ but real comments // here",
		"Speaker: Line with some backslashes \\/\\/ but real comments ");
	assert_filter!(
		"<<if $condition>>// comments ",
		"<<if $condition>>");
	assert_filter!(
		"-> Option line but //comments",
		"-> Option line but ");
	assert_filter!(
		"Hello there, \"Comments // inside literals should be ignored\"",
		"Hello there, \"Comments // inside literals should be ignored\"");
	assert_filter!(
		"Hello there, \"Comments // inside literals should be ignored\" // but outside shouldn't",
		"Hello there, \"Comments // inside literals should be ignored\" ");
}

fn read_lines(file: File, path: PathBuf) -> Result<YarnFile> {
	let decoded_file = 
		DecodeReaderBytesBuilder::new()
			.encoding(None)
			.bom_sniffing(true)
			.build(file);
	
	let reader = 
		BufReader::new(decoded_file);

	let source_lines: Vec<(usize, String)> =
		reader.lines()
		      .enumerate()
		      .map(|(line_number, result)|
			      Ok((line_number + 1, result?)))
		      .try_collect()
			  .map_err(|err: anyhow::Error| anyhow!(
				  "Could not read line from file.\n\
		 		   Error: {err}")
			  )?;
	
	let lines =
		source_lines
			.into_iter()
			.filter_map(|(line_number, mut text)| {
				filter_comments(&mut text);
				text.trim_end_in_place();
				text.shrink_to_fit();

				if text.is_empty() || text.chars().all(char::is_whitespace) {
					None
				} else {
					Some(UnparsedLine { line_number, text })
				}
			}).collect();
	
	Ok(YarnFile {
		path,
		lines,
	})
}

fn read_files(paths: Vec<PathBuf>) -> Result<Vec<YarnFile>> {
	paths.into_iter()
	     .map(|path| {
		     let file =
			     File::open(&path)
				     .map_err(|err| anyhow!(
						 "Could not open file at path: {path:?}\n\
					      Error: {err}")
				     )?;

		     let lines = read_lines(file, path)?;
		     Ok(lines)
	     }).try_collect()
}

pub fn find_and_read_yarn_files(cfg: &YarnConfig) -> Result<Vec<YarnFile>> {
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
