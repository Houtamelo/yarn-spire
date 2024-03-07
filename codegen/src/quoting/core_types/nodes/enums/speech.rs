use genco::prelude::quoted;
use genco::prelude::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::util::SeparatedItems;
use crate::quoting::quotable_types::line_ids::{IDFlatLine, IDFlow, IDSpeech};
use crate::quoting::quotable_types::enums::LineEnum;
use crate::quoting::quotable_types::enums;
use crate::quoting::quotable_types::advance::build_next_fn;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use crate::quoting::quotable_types::scope::IDScope;

fn tokens_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use houtamelo_utils::prelude::*;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		use std::borrow::Cow;
	}
}

fn tokens_enum(speeches: &[(&IDSpeech, LineEnum)], 
               enum_name: &str)
               -> Tokens {
	let enum_variants =
		speeches
			.iter()
			.map(|(_, line_enum)| 
				line_enum.variant_name());

	quote! {
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum $enum_name {
			$(SeparatedItems(enum_variants, ",\n"))
		}
	}
}

fn insert_scope_advance_fns<'a>(next_fns: &mut Vec<(&'a str, Tokens)>,
                                current_scope: &'a IDScope,
                                next_scopes: &[&IDScope],
                                title: &str) {
	let mut flows: Vec<&IDFlow> =
		current_scope
			.flows
			.iter()
			.collect();

	while flows.len() > 0 {
		let flow = flows.remove(0);

		match flow {
			IDFlow::Flat(flat_lines) => {
				let mut flat_lines: Vec<&IDFlatLine> =
					flat_lines
						.iter()
						.collect();

				while flat_lines.len() > 0 {
					let flat_line = flat_lines.remove(0);

					if let IDFlatLine::Speech(speech) = flat_line {
						let next_fn = build_next_fn(
							flat_lines.iter().copied(), //relax, it only copies the references
							flows.iter().copied(),
							next_scopes.iter().copied(),
							title
						);

						next_fns.push((speech.line_id.as_str(), next_fn));
					}
				}
			},
			IDFlow::OptionsFork(options_fork) => {
				for (_, maybe_scope) in options_fork.options.iter() {
					if let Some(option_scope) = maybe_scope {
						insert_scope_advance_fns(next_fns, option_scope, next_scopes, title);
					}
				}
			},
			IDFlow::IfBranch(if_branch) => {
				if let Some(if_scope) = &if_branch.if_.1 {
					insert_scope_advance_fns(next_fns, if_scope, next_scopes, title);
				}
				
				for (_, maybe_scope) in if_branch.else_ifs.iter() {
					if let Some(else_if_scope) = maybe_scope {
						insert_scope_advance_fns(next_fns, else_if_scope, next_scopes, title);
					}
				}
				
				if let Some((_, Some(else_scope))) = &if_branch.else_ {
					insert_scope_advance_fns(next_fns, else_scope, next_scopes, title);
				}
			},
		}
	}
}

fn all_advance_fns<'a>(speeches: &'a [(&IDSpeech, LineEnum)],
                       node: &'a IDNode)
                       -> impl Iterator<Item = (&'a LineEnum<'a>, Tokens)> {
	let title = node.metadata.title.as_str();
	
	let mut next_fns = Vec::new();
	
	let mut scopes: Vec<&IDScope> = 
		node.scopes
			.iter()
			.collect();
	
	while scopes.len() > 0 {
		let scope = scopes.remove(0);
		insert_scope_advance_fns(&mut next_fns, scope, &scopes, title);
	}
	
	if next_fns.len() != speeches.len() {
		panic!(
			"The number of next functions that came from speeches(`{}`) \
			 does not match the number of speeches in the input(`{}`).\n\
			 Node: `{title}`", speeches.len(), next_fns.len());
	}
	
	next_fns
		.into_iter()
		.map(|(line_id, next_fn)| {
			let (_, line_enum) = speeches
				.iter()
				.find(|(_, line_enum)| line_enum.raw_id == line_id)
				.expect(format!(
					"A next function was generated for a speech line that is not included in the input.\n\
					 Line id: {line_id}\n\
					 Next function: {next_fn:?}").as_str());
			
			(line_enum, next_fn)
		})
}

fn tokens_trait_impl<'a>(cfg: &YarnConfig,
                         speeches: &[(&IDSpeech, LineEnum)],
                         node: &IDNode,
                         enum_name: &str)
                         -> Tokens {
	let advance_fns = 
		all_advance_fns(speeches, node)
			.map(|(line_enum, tokens)| {
				quote! {
					$(line_enum.typed_qualified()) => { 
						$(tokens)
					},
				}
			});
	
	let line_ids = 
		speeches
			.iter()
			.map(|(_, line_enum)| 
				quote! { 
					$(line_enum.typed_qualified()) => $(quoted(line_enum.raw_id)),
				});
	
	let tags = 
		speeches
			.iter()
			.map(|(speech, line_enum)| {
				let tags =
					speech.tags
					      .iter()
					      .map(|tag| quoted(tag));
				
				quote! { 
					$(line_enum.typed_qualified()) => &[
						$(SeparatedItems(tags, ",\n"))
					],
				}
			});
	
	let speakers =
		speeches
			.iter()
			.map(|(speech, line_enum)| {
				let speaker_tokens =
					match speech.speaker.as_ref() {
						Some(speaker) => 
							quote!(Some($speaker.into())),
						None => 
							quote!(None),
					};

				quote! {
					$(line_enum.typed_qualified()) => {
						$speaker_tokens
					},
				}
			});
	
	let texts =
		speeches
			.iter()
			.map(|(speech, line_enum)| {
				let (literal, exprs) = &speech.text;
				let quoted_lit = quoted(literal);
				let text = 
					if exprs.is_empty() {
						quote!($quoted_lit)
					} else {
						quote!(format!($quoted_lit, $(SeparatedItems(exprs, ", "))))
					};
				
				quote! {
					$(line_enum.typed_qualified()) => 
						$(text).into(),
				}
			});
	
	quote! {
		impl SpeechTrait for $enum_name {
			fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield {
				match self {
					$(SeparatedItems(advance_fns, "\n"))
				}
			}
			
			fn line_id(&self) -> &'static str {
				return match self {
					$(SeparatedItems(line_ids, "\n"))
				};
			}
			
			fn tags(&self) -> &'static [&'static str] {
				return match self {
					$(SeparatedItems(tags, "\n"))
				};
			}
			
			fn speaker(&self, storage: &$(&cfg.storage_direct)) -> Option<Cow<'static, str>> {
				return match self {
					$(SeparatedItems(speakers, "\n"))
				};
			}
			
			fn text(&self, storage: &$(&cfg.storage_direct)) -> Cow<'static, str> {
				return match self {
					$(SeparatedItems(texts, "\n"))
				};
			}
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  node: &IDNode,
                  lines_map: &LinesMap)
                  -> Option<Tokens> {
	if lines_map.speeches.is_empty() {
		return None;
	}
	
	let enum_name = 
		enums::enum_type_speech(&node.metadata.title);
	
	let tokens_imports =
		tokens_imports(cfg);
	let tokens_enum = 
		tokens_enum(&lines_map.speeches, &enum_name);
	let tokens_trait_impl =
		tokens_trait_impl(cfg, &lines_map.speeches, node, &enum_name);

	Some(quote! {
		$tokens_imports
		
		$tokens_enum
		
		$tokens_trait_impl
	})
}