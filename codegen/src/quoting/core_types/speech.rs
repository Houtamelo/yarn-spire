use crate::config::YarnConfig;
use crate::quoting::quotable_types::enums::SUFFIX_SPEECH;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use crate::quoting::util::{Comments, SeparatedItems};
use genco::lang::rust::Tokens;
use genco::quote;

pub fn all_tokens(cfg: &YarnConfig, nodes_mapped: &[(&IDNode, LinesMap)]) -> Tokens {
	let imports_and_trait = tokens_imports_and_trait(cfg);
	let enum_tokens = tokens_enum(cfg, nodes_mapped);

	quote! {
		$imports_and_trait
		$enum_tokens
	}
}

fn tokens_imports_and_trait(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::borrow::Cow;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		pub trait ISpeechLine {
			$(Comments([
				"The line's unique identifier, if specified, for more, \n\
				 see [metadata#line](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#line)"]))
			#[must_use]
			fn line_id(&self) -> &'static str;
		
			$(Comments([
				r#"The list of tags this line has, if any."#,
				r#"Each element contains everything between two hashtags (`#` ~ `#`) or (# ~ end of line)."#,
				r#"This means that each hashtag ends the previous tag and starts a new one."#,
				r#"Note that, although `line_id` is also declared with a hashtag, it is not considered a tag and has it's dedicated field."#,
				r#"___"#,
				r#"### Example"#,
				r#"Consider the line: `Houtamelo: This is the second line #houtamelo:happy #narrator:sad`"#,
				r#"The tags list would be: `vec!["houtamelo:happy", "narrator:sad"]`"#]))
			#[must_use]
			fn tags(&self) -> &'static [&'static str] { 
				&[]
			}
		
			$(Comments([
				r#"The name of the character that's speaking, if any."#,
				r#"___"#,
				r#"### Example"#,
				r#"Consider the line: `Houtamelo: This is the first line`"#,
				r#"The speaker would be: `Some("Houtamelo")`"#,
				r#"Then consider the line: `$player_name: This is the first line`"#,
				r#"The speaker would be: `Some(storage.get_var::<player_name>())`"#,
				"On the case above, it is expected that `get_var::<player_name>()` returns a string, \n\
				 if it doesn't, the code won't compile."]))
			#[must_use]
			fn speaker(&self, storage: &$(&cfg.storage_direct)) -> Option<Cow<'static, str>> {
				None
			}
		
			$(Comments([
				r#"What's being spoken."#,
			    r#"___"#,
			    r#"### Example"#,
			    r#"Consider the line: `Houtamelo: This is the first line`"#,
			    r#"The text would be: `"This is the first line"`"#,
			    r#"Then consider the line: `Houtamelo: Hello there, {$player_name}!`"#,
			    r#"The text would be: `format!("Hello there, {}!", storage.get_var::<player_name>())`"#,
			    r#"Unlike in `speaker`, the arguments inside the line can be anything that implements [Display](std::fmt::Display)."#,
			    r#"A line may have an unlimited amount of arguments, as long as each is a valid expression in the YarnSpinner syntax."#]))
			#[must_use]
			fn text(&self, storage: &$(&cfg.storage_direct)) -> Cow<'static, str>;
			
			#[must_use]
			fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
		}
	}
}

fn tokens_enum(
	cfg: &YarnConfig,
	nodes_mapped: &[(&IDNode, LinesMap)],
) -> Tokens {
	let titles = nodes_mapped
		.iter()
		.filter_map(|(node, lines_map)| {
			if !lines_map.speeches.is_empty() {
				let title = node.metadata.title.clone() + SUFFIX_SPEECH;
				Some(quote! { $(title) })
			} else {
				None
			}
		});

	quote! {
		declarative_type_state::delegated_enum! {
			ENUM_OUT: {
				#[derive(Debug, Copy, Clone)]
				#[derive(PartialEq, Eq, Hash)]
				#[derive(Serialize, Deserialize)]
				pub enum SpeechLine {
					$(SeparatedItems(titles, ",\n"))
				}
			}
			
			DELEGATES: {
				impl trait ISpeechLine {
					[fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield]
					[fn line_id(&self) -> &'static str]
					[fn tags(&self) -> &'static [&'static str]]
					[fn speaker(&self, storage: &$(&cfg.storage_direct)) -> Option<Cow<'static, str>>]
					[fn text(&self, storage: &$(&cfg.storage_direct)) -> Cow<'static, str>]
				}
			}
		}
	}
}
