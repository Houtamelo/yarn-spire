use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::util::{Comments, SeparatedItems};
use crate::quoting::quotable_types::enums::SUFFIX_SPEECH;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};

fn tokens_imports_and_trait(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::borrow::Cow;
		use enum_dispatch::enum_dispatch;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		#[enum_dispatch(SpeechLine)]
		pub trait SpeechTrait {
			fn advance(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
			
			$(Comments([
				"The line's unique identifier, if specified, for more, \n\
				 see [metadata#line](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#line)"]))
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
			fn tags(&self) -> &'static [&'static str];
		
			$(Comments([
				r#"The name of the character that's speaking, if any."#,
				r#"___"#,
				r#"### Example"#,
				r#"Consider the line: `Houtamelo: This is the first line`"#,
				r#"The speaker would be: `Some("Houtamelo")`"#,
				r#"Then consider the line: `$player_name: This is the first line`"#,
				r#"The speaker would be: `Some(storage.get_var::<player_name>())`"#,
				"On the case above, it is expected that `get_var::<player_name>()` returns a string, \n\
				 if it doesn't, the code won't compile."])),
			fn speaker(&self, storage: &$(&cfg.storage_direct)) -> Option<Cow<'static, str>>;
		
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
			fn text(&self, storage: &$(&cfg.storage_direct)) -> Cow<'static, str>;
		}
	}
}

fn tokens_enum(nodes_mapped: &[(&IDNode, LinesMap)]) -> Tokens {
	let titles =
		nodes_mapped
			.iter()
			.filter_map(|(node, lines_map)| {
				if lines_map.speeches.len() > 0 {
					let title = node.metadata.title.clone() + SUFFIX_SPEECH;
					Some(quote! { $(title) })
				} else {
					None
				}
			});
	
	quote! {
		#[enum_dispatch]
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum SpeechLine {
			$(SeparatedItems(titles, ",\n"))
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  nodes_mapped: &[(&IDNode, LinesMap)])
                  -> Tokens {
	let imports_and_trait = 
		tokens_imports_and_trait(cfg);
	let enum_tokens =
		tokens_enum(nodes_mapped);
	
	quote! {
		$(imports_and_trait)
		
		$(enum_tokens)
	}
}