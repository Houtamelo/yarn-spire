#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use serde::{Deserialize, Serialize};

use crate::nodes::ch01_awakening_command::Ch01_Awakening_Command;
use crate::nodes::ch01_awakening_options_fork::Ch01_Awakening_OptionsFork;
use crate::nodes::ch01_awakening_speech::Ch01_Awakening_Speech;
use crate::shared_internal::*;

pub static TAGS: &[&str] = &[];
pub static TRACKING: TrackingSetting = TrackingSetting::Always;
pub static CUSTOM_METADATA: &[&str] = &[];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ch01_Awakening;

impl INodeTitle for Ch01_Awakening {
	fn tags(&self) -> &'static [&'static str] { TAGS }
	fn tracking(&self) -> TrackingSetting { TRACKING }
	fn custom_metadata(&self) -> &'static [&'static str] { CUSTOM_METADATA }
	fn start(&self, storage: &mut Storage) -> YarnYield { YarnYield::Instruction(Ch01_Awakening_Line::main_1.into()) }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ch01_First_Fight;

impl INodeTitle for Ch01_First_Fight {
	fn tags(&self) -> &'static [&'static str] {
		todo!()
	}

	fn tracking(&self) -> TrackingSetting {
		todo!()
	}

	fn custom_metadata(&self) -> &'static [&'static str] {
		todo!()
	}

	fn start(&self, storage: &mut Storage) -> YarnYield {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_Line {
	main_1,
	main_2,
	main_3,
	main_4,
	branch_awake_1,
	branch_awake_2,
	branch_awake_3,
	branch_stamina_1,
	branch_stamina_2,
	branch_else_1,
	branch_else_2,
	main_6,
	main_9,
	main_10,
	main_11,
	options_after_main_11,
	option_look_1,
	option_look_2,
	option_sleep_1,
	option_sleep_2,
	main_14,
}

impl From<Ch01_Awakening_Line> for Instruction {
	fn from(value: Ch01_Awakening_Line) -> Self {
		todo!()
	}
}