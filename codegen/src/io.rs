extern crate proc_macro;

use syn::__private::str;
use proc_macro::TokenStream;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read};
use std::path::PathBuf;
use proc_macro2::Span;
use quote::quote_spanned;
use trim_in_place::TrimInPlace;
use crate::{CommandEnum, ControllerType, UnparsedLine};
use anyhow::{anyhow, Result};

pub fn read_file(input: TokenStream) -> (Vec<UnparsedLine>, ControllerType, CommandEnum) {
	let input_str = input.to_string();
	
	let args = 
		input_str
		.split(',')
		.collect::<Vec<_>>();
	
	if args.len() != 3 {
		panic!(
			"Expected 3 arguments. \n\
			`File name: \"name_here.yarn\"`, Controller type: `DialogueController`, Command type: `CommandEnum`\n\
			Got: {}", args.join(", "));
	}
	
	let mut file_name = args[0].to_string();
	file_name.retain(|c| c != '\"');
	file_name.trim_in_place();

	if !file_name.ends_with(".yarn") {
		panic!("{}!{{file_name, controller_type, command_type}}'s `file_name` only accepts .yarn files.\n\
		Got: {file_name}", {houtamelo_utils::fn_name(&crate::yarn_file)});
	}
	
	let path =
		find_invocation_path_by_expand().unwrap_or_else(|_| {
			let macro_invocation = fmtools::format!(""{houtamelo_utils::fn_name(&crate::yarn_file)}"!{"{input_str}"}");
			match find_invocation_path_by_glob(macro_invocation.as_str()) {
				Ok(ok) => ok,
				Err(_err) => panic!("Source file containing macro_invocation: {macro_invocation} not found!\nError: {_err}")
			}
		});
	

	let Some(folder) = path.parent()
		else { 
			panic!("Source file containing macro_invocation does not have a parent.\n\
			Path: {path:?}")
		};

	let file_path = folder.join(file_name);
	let file =
		match File::open(file_path) {
			Ok(file) => file,
			Err(_err) => {
				panic!("Could not open file, error: {_err}")
			}
		};
	
	let reader = BufReader::new(file);
	
	let source_lines_result = 
		reader.lines()
		      .enumerate()
		      //.par_bridge()
		      .map(|(line_number, result)| result.map(|text| (line_number, text)))
		      .collect::<Result<Vec<(usize, String)>, Error>>();
	
	let source = 
		match source_lines_result {
			Ok(source_lines) => source_lines,
			Err(_err) => {
				panic!("Could not read file, error: {_err}");
			}
		};
	
	let mut unsorted_lines: Vec<UnparsedLine> = 
		source.into_iter()//.into_par_iter()
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
	
	unsorted_lines.sort_by_key(|line| line.line_number);
	return (unsorted_lines, args[1].trim().to_owned(), args[2].trim().to_owned());
}

fn file_by_expand(file_name: &String) -> File {
	let path =
		match find_invocation_path_by_expand() {
			Ok(ok) => ok,
			Err(_err) => panic!("Source file containing macro_invocation not found!\nError: {_err}")
		};

	let Some(folder) = path.parent()
		else {
			panic!("Source file containing macro_invocation does not have a parent.\n\
			Path: {path:?}")
		};

	let file_path = folder.join(file_name);
	return File::open(file_path)
		.unwrap_or_else(|_err| { panic!("Could not open file, error: {_err}") });
}

fn file_by_glob(file_name: &String, macro_invocation: &String) -> File {
	let path =
		match find_invocation_path_by_glob(macro_invocation.as_str())
		{
			Ok(ok) => ok,
			Err(_err) => panic!("Source file containing macro_invocation: {macro_invocation} not found!\nError: {_err}")
		};
	

	let Some(folder) = path.parent()
		else {
			panic!("Source file containing macro_invocation {macro_invocation} does not have a parent.\n\
			Path: {path:?}")
		};

	let file_path = folder.join(file_name);
	return File::open(file_path)
		.unwrap_or_else(|_err| { panic!("Could not open file, error: {_err}") });
}

fn find_invocation_path_by_expand() -> Result<PathBuf> {
	let proc_2 = quote_spanned!(Span::call_site() => file!());
	let proc_1: TokenStream = proc_2.into();
	let path_tokens =
		proc_1.expand_expr()
		      .map_err(|err| anyhow!("Error expanding invocation path: {err}"))?;

	let path_str = path_tokens.to_string();
	let clean_path =
		path_str.replace("\\\\", "\\")
		        .replace("\\\\", "\\")
		        .replace("\"", "");

	let path_buf = PathBuf::from(clean_path);
	let mut file = File::open(&path_buf)?;
	file.read_to_string(&mut String::new())?;
	Ok(path_buf)
}

fn find_invocation_path_by_glob(pattern: &str) -> Result<PathBuf> {
	let mut pattern = pattern.to_string();
	pattern.remove_matches(' ');
	let pattern = pattern.as_str();
	
	for path in glob::glob("**/*.rs").unwrap() {
		if let Ok(path) = path 
			&& let Ok(mut f) = File::open(&path) {
			let mut contents = String::new();
			f.read_to_string(&mut contents).ok();
			
			contents.remove_matches(' ');
			
			if contents.contains(pattern) {
				return Ok(path.to_owned());
			}
		}
	}

	return Err(anyhow!("Pattern: {pattern} not found in any source file.").into());

}