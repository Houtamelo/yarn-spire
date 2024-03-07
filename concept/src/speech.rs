#![allow(non_camel_case_types)]
#![allow(unused)]

use std::borrow::Cow;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::shared_internal::*;
use crate::shared_internal::ch01_awakening_speech::Ch01_Awakening_Speech;

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpeechLine {
	Ch01_Awakening_Speech,
}

#[enum_dispatch(SpeechLine)]
pub trait SpeechTrait {
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
	/// Consider the line: `Houtamelo: This is the second line #houtamelo:happy #narrator:sad`
	///
	/// The tags list would be: `vec!["houtamelo:happy", "narrator:sad"]`
	fn tags(&self) -> &'static [&'static str];

	/// The name of the character that's speaking, if any.
	///
	/// ___
	///
	/// ### Example
	/// Consider the line: `Houtamelo: This is the first line`
	///
	/// The speaker would be: `Some("Houtamelo")`
	///
	/// Then consider the line: `$player_name: This is the first line`
	///
	/// The speaker would be: `Some(storage.get_var::<player_name>())`
	///
	/// On the case above, it is expected that `get_var::<player_name>()` returns a string, 
	/// if it doesn't, the code won't compile.
	fn speaker(&self, storage: &Storage) -> Option<Cow<'static, str>>;

	/// What's being spoken.
	///
	/// ___
	///
	/// ### Example
	///
	/// Consider the line: `Houtamelo: This is the first line`
	///
	/// The text would be: `"This is the first line"`
	///
	/// Then consider the line: `Houtamelo: Hello there, {$player_name}!`
	///
	/// The text would be: `format!("Hello there, {}!", storage.get_var::<player_name>())`
	///
	/// Unlike in `speaker`, the arguments inside the line can be anything that implements [Display](std::fmt::Display).
	///
	/// A line may have an unlimited amount of arguments, as long as each is a valid expression in the YarnSpinner syntax.
	fn text(&self, storage: &Storage) -> Cow<'static, str>;
}
