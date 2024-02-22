#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use houtamelo_utils::prelude::*;
use serde::{Deserialize, Serialize};
use crate::shared_internal::*;

pub static TAGS: &'static[&'static str] = &[];
pub static TRACKING: Option<TrackingSetting> = None;
pub static CUSTOM_METADATA: &'static [&'static str] = &[];

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ch01_Awakening {
	
}

impl NodeTitleTrait for Ch01_Awakening {
	fn tags(&self) -> &'static [&'static str] { TAGS }
	fn tracking(&self) -> Option<TrackingSetting> { TRACKING }
	fn custom_metadata(&self) -> &'static [&'static str] { CUSTOM_METADATA }
	fn start(&self) -> YieldResult { YieldResult::Line(Ch01_Awakening_Line::L_01.into()) }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_Line {
	L_01,
	L_02,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_Speech {
	L_01,
	L_02,
}

impl From<Ch01_Awakening_Line> for Line {
	fn from(value: Ch01_Awakening_Line) -> Self {
		return match value {
			Ch01_Awakening_Line::L_01 => {
				Line::Speech(Ch01_Awakening_Speech::L_01.into())
			},
			Ch01_Awakening_Line::L_02 => {
				Line::Speech(Ch01_Awakening_Speech::L_02.into())
			},
		};
	}
}

impl SpeechTrait for Ch01_Awakening_Speech {
	fn next(&self, storage: &mut Storage) -> YieldResult {
		todo!()
	}
	
	fn speech(&self) -> Speech {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_Command {
	L_03,
	L_04,
}

impl CommandTrait for Ch01_Awakening_Command {
	fn next(&self, storage: &mut Storage) -> YieldResult {
		todo!()
	}

	fn command(&self) -> YarnCommand {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_Options {
	L_06,
	L_07,
}

impl OptionsTrait for Ch01_Awakening_Options {
	fn next(&self, storage: &mut Storage, player_decision: PlayerDecision) -> YieldResult {
		todo!()
	}

	fn options(&self) -> CountOrMore<1, ChoiceOption> {
		todo!()
	}
}
