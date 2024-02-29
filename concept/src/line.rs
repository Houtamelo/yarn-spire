#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use serde::{Deserialize, Serialize};

use crate::shared_internal::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
	Speech(SpeechLine),
	Command(CommandLine),
	Options(OptionsFork),
}

pub enum YarnYield {
	Instruction(Instruction),
	Finished,
}

impl From<Instruction> for YarnYield {
	fn from(value: Instruction) -> Self {
		YarnYield::Instruction(value)
	}
}

impl From<SpeechLine> for Instruction { 
	fn from(speech_line: SpeechLine) -> Self { Instruction::Speech(speech_line) }
}

impl From<CommandLine> for Instruction { 
	fn from(command_line: CommandLine) -> Self { Instruction::Command(command_line) }
}

impl From<OptionsFork> for Instruction { 
	fn from(options_line: OptionsFork) -> Self { Instruction::Options(options_line) }
}
