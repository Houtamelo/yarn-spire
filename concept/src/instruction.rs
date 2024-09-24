#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use serde::{Deserialize, Serialize};

use crate::shared_internal::*;

declarative_type_state::delegated_enum! {
	ENUM_OUT: {
		#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
		pub enum Instruction {
			Speech(SpeechLine),
			Command(CommandLine),
			Options(OptionsFork),
		}
	}
	
	DELEGATES: {
		
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum YarnYield {
	Instruction(Instruction),
	Finished,
}

impl From<Instruction> for YarnYield {
	fn from(value: Instruction) -> Self {
		YarnYield::Instruction(value)
	}
}