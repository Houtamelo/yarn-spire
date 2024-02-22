#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use serde::{Deserialize, Serialize};
use enum_dispatch::enum_dispatch;
use houtamelo_utils::prelude::*;
use crate::shared_internal::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Line {
	Speech(SpeechLine),
	Command(CommandLine),
	Options(OptionsLine),
}

pub enum YieldResult {
	Line(Line),
	Finished,
}

impl From<SpeechLine> for Line { 
	fn from(speech_line: SpeechLine) -> Self { Line::Speech(speech_line) }
}

impl From<CommandLine> for Line { 
	fn from(command_line: CommandLine) -> Self { Line::Command(command_line) }
}

impl From<OptionsLine> for Line { 
	fn from(options_line: OptionsLine) -> Self { Line::Options(options_line) }
}

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpeechLine {
	Ch01_Awakening_Speech,
}

#[enum_dispatch(SpeechLine)]
pub(crate) trait SpeechTrait {
	fn next(&self, storage: &mut Storage) -> YieldResult;
	fn speech(&self) -> Speech;
}

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandLine {
	Ch01_Awakening_Command,
}

#[enum_dispatch(CommandLine)]
pub(crate) trait CommandTrait {
	fn next(&self, storage: &mut Storage) -> YieldResult;
	fn command(&self) -> YarnCommand;
}

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptionsLine {
	Ch01_Awakening_Options,
}

#[enum_dispatch(OptionsLine)]
pub(crate) trait OptionsTrait {
	fn next(&self, storage: &mut Storage, player_decision: PlayerDecision) -> YieldResult;
	fn options(&self) -> CountOrMore<1, ChoiceOption>;
}
