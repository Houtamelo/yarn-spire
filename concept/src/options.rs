#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use std::borrow::Cow;

use enum_dispatch::enum_dispatch;
use houtamelo_utils::prelude::CountOrMore;
use serde::{Deserialize, Serialize};

use crate::shared_internal::*;
use crate::shared_internal::ch01_awakening_option_line::Ch01_Awakening_OptionLine;
use crate::shared_internal::ch01_awakening_options_fork::Ch01_Awakening_OptionsFork;

#[enum_dispatch(OptionsFork)]
pub(crate) trait OptionsForkTrait {
	fn options(&self) -> CountOrMore<1, OptionLine>;
}

#[enum_dispatch(OptionLine)]
pub(crate) trait OptionLineTrait {
	fn next(&self, storage: &mut Storage) -> YarnYield;

	/// The line's unique identifier, for more, 
	/// see [metadata#line](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#line)
	fn line_id(&self) -> &'static str;

	/// The list of tags this line has, if any.
	///
	/// Each element contains everything between two hashtags (`#` ~ `#`) or (# ~ end of line).
	///
	/// This means that each hashtag ends the previous tag and starts a new one.
	///
	/// Note that, although `line_id` is also declared with a hashtag, it is not considered a tag and has it's dedicated field. 
	///
	/// ___
	///
	/// ### Example
	/// Consider the line: `-> Here's your option A #houtamelo:happy #narrator:sad`
	///
	/// The tags list would be: `vec!["houtamelo:happy", "narrator:sad"]`
	fn tags(&self) -> &'static [&'static str];

	/// The text representing the choice the player can make.
	///
	/// ___
	///
	/// ### Example
	/// Consider the line: `-> Jump off the cliff`
	///
	/// The text would be: `Jump off the cliff`
	fn text(&self, storage: &Storage) -> Cow<'static, str>;

	/// The evaluated condition, if any.
	///
	/// This will only be `Some` if the option's line has a condition(`<<if [condition]>>`).
	///
	/// ___
	///
	/// # Example
	/// Consider the line: `-> Jump off the cliff <<if $player_has_parachute>>`
	///
	/// When reaching this line, the variable `player_has_parachute` will be fetched from the [VariableStorage](crate::shared_internal::VariableStorage), 
	///  making `is_available` be `Some(storage.get_var::<player_has_parachute>())`.
	///
	/// Note that, in this case, it is expected for the return value of `get_var::<player_has_parachute>()` to be a boolean,
	/// if it isn't, the code won't compile.
	///
	/// ___
	///
	/// # Usage
	/// - Although Evaluating the condition is done by YarnSpinner, it is up to the developer to decide what to do with the result,
	///  YarnSpinner will not forbid the player from picking an option even if it has a condition evaluated to `false`.
	/// - The `[condition]` argument can be any valid expression in the YarnSpinner syntax (`{5 + 3 > 8}`, `$player_awake and $gold > 10`, ...)
	fn is_available(&self, storage: &Storage) -> Option<bool>;
}

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptionsFork {
	Ch01_Awakening_OptionsFork,
}

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq)]
pub enum OptionLine {
	Ch01_Awakening_OptionLine,
}
