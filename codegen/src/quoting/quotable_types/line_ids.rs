use std::collections::HashSet;
use houtamelo_utils::prelude::CountOrMore;
use anyhow::{anyhow, Result};
use crate::LineNumber;
use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::raw::speech::Speaker;
use crate::parsing::YarnNode;
use crate::parsing::grouping::options::OptionsFork;
use crate::parsing::grouping::scope::{FlatLine, Flow, YarnScope};
use crate::parsing::raw::branches::if_statement::{ElseIfStruct, ElseStruct, IfStruct};
use crate::parsing::raw::command::CommandVariant;
use crate::quoting::quotable_types::node::IDNode;
use crate::quoting::quotable_types::scope::IDScope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IDOptionLine {
	pub line_number: LineNumber,
	pub line_id: String,
	pub text: (String, Vec<YarnExpr>),
	pub if_condition: Option<YarnExpr>,
	pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IDOptionsFork {
	pub virtual_id: String,
	pub options: CountOrMore<1, (IDOptionLine, Option<Box<IDScope>>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IDFlow {
	Flat(Vec<IDFlatLine>),
	OptionsFork(IDOptionsFork),
	IfBranch(IDIfBranch),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IDFlatLine {
	Speech(IDSpeech),
	CustomCommand(IDCustomCommand),
	BuiltInCommand(BuiltInCommand), // can't have ID
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IDSpeech {
	pub line_number: LineNumber,
	pub line_id: String,
	pub speaker: Option<Speaker>,
	pub text: (String, Vec<YarnExpr>),
	pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IDCustomCommand {
	pub line_number: LineNumber,
	pub line_id: String,
	pub variant: String,
	pub args: Vec<YarnExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInCommand {
	Set { line_number: LineNumber, var_name: String, value: YarnExpr },
	Jump { line_number: LineNumber, node_destination_title: String },
	Stop { line_number: LineNumber },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IDIfBranch {
	pub if_: (IfStruct, Option<Box<IDScope>>),
	pub else_ifs: Vec<(ElseIfStruct, Option<Box<IDScope>>)>,
	pub else_: Option<(ElseStruct, Option<Box<IDScope>>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionKind {
	Speech,
	Command,
	OptionsFork,
}

fn insert_flat_ids(flat_lines: &[FlatLine], taken_ids: &mut HashSet<String>) -> Result<()> {
	let line_ids =
		flat_lines
			.iter()
			.filter_map(|flat|
				match flat {
					FlatLine::Speech(speech) =>
						speech.line_id.as_ref().map(|id| id.as_str()),
					FlatLine::Command(_) => None,
				});

	for id in line_ids {
		if taken_ids.contains(id) {
			return Err(anyhow!( "Duplicate line id found: {}" , id ));
		} else {
			taken_ids.insert(id.to_owned());
		}
	}
	
	return Ok(());
}

fn insert_options_ids(options_fork: &OptionsFork, taken_ids: &mut HashSet<String>) -> Result<()> {
	for (option, scope_option) in options_fork.iter_options() {
		if let Some(id) = &(option.line_id) {
			if taken_ids.contains(id.as_str()) {
				return Err(anyhow!( "Duplicate line id found: {}" , id ));
			} else {
				taken_ids.insert(id.to_owned());
			}
		}

		if let Some(scope) = scope_option {
			insert_scope_ids(scope, taken_ids)?;
		}
	}

	Ok(())
}

fn insert_scope_ids(scope: &YarnScope, taken_ids: &mut HashSet<String>) -> Result<()> {
	for flow in scope.flows() {
		match flow {
			Flow::Flat(flat_lines) => {
				insert_flat_ids(flat_lines, taken_ids)?;
			},
			Flow::OptionsFork(options_fork) => {
				insert_options_ids(options_fork, taken_ids)?;
			},
			Flow::IfBranch(if_branch) => {
				if_branch
					.if_.1
					.as_ref()
					.map(|if_scope|
						insert_scope_ids(if_scope, taken_ids));
				
				if_branch
					.else_ifs
					.iter()
					.try_for_each(|(_, scope_option)| {
						if let Some(else_if_scope) = scope_option {
							insert_scope_ids(else_if_scope, taken_ids)?;
						}
						
						Result::<_, anyhow::Error>::Ok(())
					})?;
				
				if let Some((_, Some(else_scope))) = &if_branch.else_{
					insert_scope_ids(else_scope, taken_ids)?;
				}
			}
		}
	}
	
	Ok(())
}

fn fill_existing_ids(node: &YarnNode, taken_ids: &mut HashSet<String>) -> Result<()> {
	node.contents
		.iter()
		.try_for_each(|scope| {
			insert_scope_ids(scope, taken_ids)
		})
}

macro_rules! gen_id {
    ($prefix: ident, $counter: ident) => {{
	    let id = format!("{}{}", $prefix, $counter);
	    *$counter += 1;
	    id
    }};
}

fn convert_to_id_scope(scope: YarnScope, taken_ids: &mut HashSet<String>,
                       id_prefix: &str, id_counter: &mut usize) -> Result<IDScope> {
	let indent = scope.indent();
	
	let id_flows =
		scope.into_flows()
			.map(|flow|
				match flow {
					Flow::Flat(flat_lines) => {
						let id_flat_lines = flat_lines
							.into_iter()
							.map(|flat|
								match flat {
									FlatLine::Speech(speech) => {
										let line_id = speech
											.line_id
											.unwrap_or_else(|| gen_id!(id_prefix, id_counter));

										IDFlatLine::Speech(IDSpeech {
											line_number: speech.line_number,
											line_id,
											speaker: speech.speaker,
											text: speech.text,
											tags: speech.tags,
										})
									},
									FlatLine::Command(command) => {
										match command.variant {
											CommandVariant::Set { var_name, arg } => {
												IDFlatLine::BuiltInCommand(BuiltInCommand::Set {
													line_number: command.line_number,
													var_name,
													value: arg,
												})
											},
											CommandVariant::Jump { node_name } => {
												IDFlatLine::BuiltInCommand(BuiltInCommand::Jump {
													line_number: command.line_number,
													node_destination_title: node_name,
												})
											},
											CommandVariant::Stop => {
												IDFlatLine::BuiltInCommand(BuiltInCommand::Stop {
													line_number: command.line_number,
												})
											},
											CommandVariant::Other { variant, args } => {
												IDFlatLine::CustomCommand(IDCustomCommand {
													line_number: command.line_number,
													line_id: gen_id!(id_prefix, id_counter),
													variant,
													args,
												})
											},
										}
									}
								})
							.collect();

						Result::<_, anyhow::Error>::Ok(IDFlow::Flat(id_flat_lines))
					},
					Flow::OptionsFork(options_fork) => {
						let mut id_options_iter =
							options_fork
								.options
								.into_iter()
								.map(|(line, scope)| {
									let line_id = line
										.line_id
										.unwrap_or_else(|| gen_id!(id_prefix, id_counter));

									let id_scope = scope
										.map(|scope|
											convert_to_id_scope(*scope, taken_ids, id_prefix, id_counter)
												.map(Box::from))
										.transpose()?;

									Result::<_, anyhow::Error>::Ok((IDOptionLine {
										line_number: line.line_number,
										line_id,
										text: line.text,
										if_condition: line.if_condition,
										tags: line.tags,
									}, id_scope))
								});

						let first_id_option =
							id_options_iter
								.next()
								.unwrap()?;

						let other_id_options =
							id_options_iter.try_collect()?;

						Ok(IDFlow::OptionsFork(IDOptionsFork {
							virtual_id: gen_id!(id_prefix, id_counter),
							options: CountOrMore::new([first_id_option], other_id_options),
						}))
					},
					Flow::IfBranch(if_branch) => {
						let id_if_scope = 
							if_branch
								.if_.1
								.map(|if_scope|
									convert_to_id_scope(*if_scope, taken_ids, id_prefix, id_counter)
										.map(Box::from))
								.transpose()?;

						let id_else_ifs = 
							if_branch
								.else_ifs
								.into_iter()
								.map(|(elseif, scope_option)| {
									let id_scope =
										scope_option
											.map(|scope|
												convert_to_id_scope(*scope, taken_ids, id_prefix, id_counter)
													.map(Box::from))
											.transpose()?;

									Result::<_, anyhow::Error>::Ok((elseif, id_scope))
								}).try_collect()?;

						let id_else = 
							if_branch
								.else_
								.map(|(else_, scope_option)| {
									let id_scope =
										scope_option
											.map(|scope|
												convert_to_id_scope(*scope, taken_ids, id_prefix, id_counter)
													.map(Box::from))
											.transpose()?;

									Result::<_, anyhow::Error>::Ok((else_, id_scope))
								}).transpose()?;

						Ok(IDFlow::IfBranch(IDIfBranch {
							if_: (if_branch.if_.0, id_if_scope),
							else_ifs: id_else_ifs,
							else_: id_else,
						}))
					},
				})
			.try_collect()?;
	
	return Ok(IDScope {
		indent,
		flows: id_flows,
	});
}

fn pick_prefix(title: &str, taken_prefixes: &mut HashSet<String>) -> String {
	if title.chars().count() <= 2  {
		if taken_prefixes.contains(title) {
			for num in 0_usize.. {
				let prefix = format!("{}{}", title, num);
				if !taken_prefixes.contains(&prefix) {
					taken_prefixes.insert(prefix.clone());
					return prefix;
				}
			}
			
			panic!("Could not build prefix for title: {title}, somehow all strings {title}(0..usize::max) are taken.");
		} else {
			taken_prefixes.insert(title.to_owned());
			return title.to_owned();
		}
	}
	
	let words = 
		if title.chars().any(|c| c.is_ascii_lowercase()) {
			title.split_inclusive(|c: char| c.is_ascii_uppercase() || c == '_' || c == '-')
			     .filter(|word| !word.is_empty())
			     .collect::<Vec<&str>>()
		} else {
			title.split_inclusive(&['_', '-'][..])
			     .filter(|word| !word.is_empty())
			     .collect::<Vec<&str>>()
		};
	
	if words.len() >= 2 {
		let prefix: String = [
			words[0]
				.chars()
				.next()
				.unwrap()
				.to_ascii_lowercase(),
			words[1]
				.chars()
				.next()
				.unwrap()
				.to_ascii_lowercase()
		].into_iter().collect();
		
		if !taken_prefixes.contains(&prefix) {
			taken_prefixes.insert(prefix.clone());
			return prefix;
		}
	}
	
	let first_char = 
		title.chars().next().unwrap().to_ascii_lowercase();
	
	for ch in title.chars().skip(1).map(|c| c.to_ascii_lowercase()) {
		let prefix = [first_char, ch].into_iter().collect::<String>();
		if !taken_prefixes.contains(&prefix) {
			taken_prefixes.insert(prefix.clone());
			return prefix;
		}
	}
	
	for num in taken_prefixes.len().. {
		let prefix = format!("{}{}", first_char, num);
		if !taken_prefixes.contains(&prefix) {
			taken_prefixes.insert(prefix.clone());
			return prefix;
		}
	} 
	
	panic!("Could not build prefix for title: {title}, \
	somehow all strings {first_char}({taken_len}..usize::max) are taken."
		, taken_len = taken_prefixes.len());
}

fn generate_prefixes(nodes: Vec<YarnNode>) -> Vec<(YarnNode, String)> {
	nodes.into_iter()
		.scan(HashSet::new(), |taken_prefixes, node| {
			let prefix = pick_prefix(&node.metadata.title, taken_prefixes);
			Some((node, prefix))
		}).collect()
}

pub fn convert_to_id_nodes(nodes: Vec<YarnNode>) -> Result<Vec<IDNode>> {
	let mut taken_ids = HashSet::new();
	let mut id_counter = 0;
	
	nodes.iter()
		.try_for_each(|node| 
			fill_existing_ids(node, &mut taken_ids))?;
	
	generate_prefixes(nodes)
		.into_iter()
		.map(|(node, prefix)| {
			let id_scopes = 
				node.contents
					.into_iter()
					.map(|scope|
						convert_to_id_scope(scope, &mut taken_ids, &prefix, &mut id_counter))
					.try_collect()?;
			
			Ok(IDNode {
				metadata: node.metadata,
				scopes: id_scopes,
			})
		}).try_collect()
}