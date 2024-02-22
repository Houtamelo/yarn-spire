use std::borrow::Cow;

/// Represents a single option the player can make,
/// these are made by the proc-macro based on the regular YarnSpinner syntax:
///
/// ```yarn
/// -> Option A #line:5450a
/// 	Option A results
/// -> Option B #my:tag
/// 	Option B results
/// ```
///
/// In this case, option A would be:
/// ```rs
/// ChoiceOption {
///     text: Cow::Borrowed("Option A"),
///     is_available: None,
///     line_id: Some("line:5450a"),
///     tags: &[]
/// }
/// ```
///
/// And option B would be:
/// ```rs
/// ChoiceOption {
///     text: Cow::Borrowed("Option B"),
///     is_available: None,
///     line_id: None,
///     tags: &["my:tag"]
/// }
/// ```
///
/// You'll receive this struct through a list in the [YarnInstruction::Choices](crate::shared_internal::YarnInstruction::Choices) enum.
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceOption {
	text: Cow<'static, str>,
	is_available: Option<bool>,
	line_id: Option<&'static str>,
	tags: &'static [&'static str],
}

impl ChoiceOption {
	/// The text representing the choice the player can make.
	///
	/// ___
	///
	/// ### Example
	/// Consider the line: `-> Jump off the cliff`
	///
	/// The text would be: `Jump off the cliff`
	pub fn text(&self) -> &str {
		return self.text.as_ref();
	}
	
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
	pub fn is_available(&self) -> Option<bool> {
		return self.is_available;
	}

	pub fn line_id(&self) -> Option<&'static str> {
		return self.line_id;
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
	/// Consider the line: `-> Here's your option A #houtamelo:happy #narrator:sad`
	///
	/// The tags list would be: `vec!["houtamelo:happy", "narrator:sad"]`
	pub fn tags(&self) -> &'static [&'static str] {
		return self.tags;
	}
}