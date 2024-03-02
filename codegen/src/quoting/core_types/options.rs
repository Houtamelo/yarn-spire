use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::util::{Comments, SeparatedItems};
use crate::quoting::quotable_types::enums::{SUFFIX_OPTION_LINE, SUFFIX_OPTIONS_FORK};
use crate::quoting::quotable_types::node::{IDNode, LinesMap};

fn tokens_traits_and_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_snake_case)]
		#![allow(non_camel_case_types)]
		#![allow(unused)]
		
		use std::borrow::Cow;
		use enum_dispatch::enum_dispatch;
		use houtamelo_utils::prelude::CountOrMore;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		
		#[enum_dispatch(OptionsFork)]
		pub trait OptionsForkTrait {
			fn options(&self) -> CountOrMore<1, OptionLine>;
		}
		
		#[enum_dispatch(OptionLine)]
		pub trait OptionLineTrait {
			fn next(&self, storage: &mut $(&cfg.storage_direct)) -> YarnYield;
		
			$(Comments([
				"The line's unique identifier, if specified, for more, \n\
				 see [metadata#line](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#line)"]))
			fn line_id(&self) -> Option<&'static str>;

			$(Comments([
				r#"The list of tags this line has, if any."#,
				r#"Each element contains everything between two hashtags (`#` ~ `#`) or (# ~ end of line)."#,
			    r#"This means that each hashtag ends the previous tag and starts a new one."#,
			    r#"Note that, although `line_id` is also declared with a hashtag, it is not considered a tag and has it's dedicated field."#,
			    r#"___"#,
			    r#"### Example"#,
			    r#"Consider the line: `-> Here's your option A #houtamelo:happy #narrator:sad`"#,
			    r#"The tags list would be: `vec!["houtamelo:happy", "narrator:sad"]`"#]))
			fn tags(&self) -> &'static [&'static str];
		
			$(Comments([
				r#"The text representing the choice the player can make."#,
				r#"___"#,
				r#"### Example"#,
				r#"Consider the line: `-> Jump off the cliff`"#,
				r#"The text would be: `Jump off the cliff`"#]))
			fn text(&self, storage: &$(&cfg.storage_direct)) -> Cow<'static, str>;
		
			$(Comments([
				r#"The evaluated condition, if any."#,
				r#"This will only be `Some` if the option's line has a condition(`<<if [condition]>>`)."#,
				r#"___"#,
				r#"# Example"#,
				r#"Consider the line: `-> Jump off the cliff <<if $player_has_parachute>>`"#,
				"When reaching this line, the variable `player_has_parachute` will be fetched from the [VariableStorage](crate::shared_internal::VariableStorage), \n\
				 making `is_available` be `Some(storage.get_var::<player_has_parachute>())`.",
				"Note that, in this case, it is expected for the return value of `get_var::<player_has_parachute>()` to be a boolean, \n\
				 if it isn't, the code won't compile.",
				r#"___"#,
				r#"# Usage"#,
				r#"- Although Evaluating the condition is done by YarnSpinner, it is up to the developer to decide what to do with the result,"#,
				r#" YarnSpinner will not forbid the player from picking an option even if it has a condition evaluated to `false`."#,
				r#"- The `[condition]` argument can be any valid expression in the YarnSpinner syntax (`{5 + 3 > 8}`, `$player_awake and $gold > 10`, ...)"#]))
			fn is_available(&self, storage: &$(&cfg.storage_direct)) -> Option<bool>;
		}
	}
}

fn tokens_enums(nodes_mapped: &[(&IDNode, LinesMap)]) -> Tokens {
	let forks =
		nodes_mapped
			.iter()
			.filter_map(|(node, lines_map)| {
				if lines_map.options_forks.len() > 0 {
					let title = node.metadata.title.clone() + SUFFIX_OPTIONS_FORK;
					Some(quote! { $(title) })
				} else {
					None
				}
			});
	
	let lines =
		nodes_mapped
			.iter()
			.filter_map(|(node, lines_map)| {
				if lines_map.option_lines.len() > 0 {
					let title = node.metadata.title.clone() + SUFFIX_OPTION_LINE;
					Some(quote! { $(title) })
				} else {
					None
				}
			});
	
	quote! {
		#[enum_dispatch]
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum OptionsFork {
			$(SeparatedItems(forks, ",\n"))
		}
		
		#[enum_dispatch]
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum OptionLine {
			$(SeparatedItems(lines, ",\n"))
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  nodes_mapped: &[(&IDNode, LinesMap)])
                  -> Tokens {
	let traits_and_imports = 
		tokens_traits_and_imports(cfg);
	let enum_tokens = 
		tokens_enums(nodes_mapped);
	
	quote! {
		$(traits_and_imports)
		
		$(enum_tokens)
	}
}