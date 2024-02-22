use std::borrow::Cow;

/// Represents a single line of dialogue,
/// these are made by the proc-macro based on the standard YarnSpinner syntax:
///
/// ```yarn
/// Houtamelo: This is the first line #line:1230
/// Houtamelo: This is the second line #my:tag
/// ```
///
/// In this case, the first line would be:
/// ```rs
/// Speech {
///     speaker: Some(Cow::Borrowed("Houtamelo")),
///     text: Cow::Borrowed("This is the first line"),
///     line_id: Some("1230"),
///     tags: vec![],
/// }
/// ```
///
/// And the second line would be:
/// ```rs
/// Speech {
///     speaker: Some(Cow::Borrowed("Houtamelo")),
///     text: Cow::Borrowed("This is the second line"),
///     line_id: None,
///     tags: vec!["my:tag"],
/// }
/// ```
///
/// You'll receive this struct through the [YarnInstruction::Speech](crate::shared_internal::YarnInstruction::Speech) enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speech {
	speaker: Option<Cow<'static, str>>,
	text: Cow<'static, str>,
	line_id: Option<&'static str>,
	tags: &'static [&'static str],
}

impl Speech {
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
	pub fn speaker(&self) -> Option<&str> {
		return self.speaker.as_ref().map(|cow| cow.as_ref());
	}

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
	pub fn text(&self) -> &str {
		return &self.text;
	}
	
	/// The line's unique identifier, if specified, for more, 
	/// see [metadata#line](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#line)
	pub fn line_id(&self) -> &Option<&'static str> {
		return &self.line_id;
	}

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
	pub fn tags(&self) -> &'static [&'static str] {
		return self.tags;
	}
}