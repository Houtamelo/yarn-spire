#![allow(non_camel_case_types)]

use houtamelo_utils::own;
use serde::{Deserialize, Serialize};

use crate::command_line::CommandLineTrait;
use crate::instruction::YarnYield;
use crate::shared_internal::{Ch01_Awakening_Line, Ch01_First_Fight, NodeTitle, INodeTitle, Storage, YarnCommand};
use crate::shared_internal::vars::{ethel_awake, ethel_stamina, mouth_taste};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_Command {
	main_1,
	main_2,
	main_4,
	main_6,
	main_9,
	option_sleep_1,
	option_sleep_2,
}

#[allow(unused_variables)]
impl CommandLineTrait for Ch01_Awakening_Command {
	fn next(&self, storage: &mut Storage) -> YarnYield {
		match self {
			Ch01_Awakening_Command::main_1 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_2.into()),
			Ch01_Awakening_Command::main_2 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_3.into()),
			Ch01_Awakening_Command::main_4 => {
				if storage.get_var::<ethel_awake>() {
					YarnYield::Instruction(Ch01_Awakening_Line::branch_awake_1.into())
				} else if storage.get_var::<ethel_stamina>() > 30 {
					YarnYield::Instruction(Ch01_Awakening_Line::branch_stamina_1.into())
				} else {
					YarnYield::Instruction(Ch01_Awakening_Line::branch_else_1.into())
				}
			},
			Ch01_Awakening_Command::main_6 => {
				storage.set_var::<mouth_taste>(own!("gold"));
				YarnYield::Instruction(Ch01_Awakening_Line::main_9.into())
			},
			Ch01_Awakening_Command::main_9 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_10.into()),
			Ch01_Awakening_Command::option_sleep_1 => 
				YarnYield::Instruction(Ch01_Awakening_Line::option_sleep_2.into()),
			Ch01_Awakening_Command::option_sleep_2 => {
				storage.increment_visited(&NodeTitle::Ch01_Awakening);
				Ch01_First_Fight.start(storage)
			},
		}
	}

	fn line_id(&self) -> &'static str {
		todo!()
	}

	fn command(&self, storage: &Storage) -> YarnCommand {
		match self {
			Ch01_Awakening_Command::main_1 => 
				YarnCommand::fade_in(1.0),
			Ch01_Awakening_Command::main_2 => 
				YarnCommand::cg(own!("CG_ch01_Not-yet-awake")),
			Ch01_Awakening_Command::main_4 => 
				YarnCommand::fade_out(1.0),
			Ch01_Awakening_Command::main_6 => 
				YarnCommand::cg(own!("CG_ch01_Awakening")),
			Ch01_Awakening_Command::main_9 => 
				YarnCommand::fade_out(1.0),
			Ch01_Awakening_Command::option_sleep_1 => 
				YarnCommand::fade_in(1.0),
			Ch01_Awakening_Command::option_sleep_2 => 
				YarnCommand::sfx_wait(own!("dressing")),
		}
	}
}
