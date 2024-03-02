#![allow(non_camel_case_types)]

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::instruction::YarnYield;
use crate::shared_internal::{Ch01_Awakening, Ch01_Awakening_Line, mouth_taste, narrator, NodeTitle, SpeechTrait, Storage};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_Speech {
	main_3,
	branch_awake_1,
	branch_awake_2,
	branch_awake_3,
	branch_stamina_1,
	branch_stamina_2,
	branch_else_1,
	main_10,
	main_11,
	main_14,
	option_look_1,
	option_look_2,
}

impl SpeechTrait for Ch01_Awakening_Speech {
	fn next(&self, storage: &mut Storage) -> YarnYield {
		return match self {
			Ch01_Awakening_Speech::main_3 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_4.into()),
			Ch01_Awakening_Speech::branch_awake_1 => 
				YarnYield::Instruction(Ch01_Awakening_Line::branch_awake_2.into()),
			Ch01_Awakening_Speech::branch_awake_2 => 
				YarnYield::Instruction(Ch01_Awakening_Line::branch_awake_3.into()),
			Ch01_Awakening_Speech::branch_awake_3 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_6.into()),
			Ch01_Awakening_Speech::branch_stamina_1 => 
				YarnYield::Instruction(Ch01_Awakening_Line::branch_stamina_2.into()),
			Ch01_Awakening_Speech::branch_stamina_2 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_6.into()),
			Ch01_Awakening_Speech::branch_else_1 => {
				storage.increment_visited(&NodeTitle::Ch01_Awakening(Ch01_Awakening));
				YarnYield::Finished 
			},
			Ch01_Awakening_Speech::main_10 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_11.into()),
			Ch01_Awakening_Speech::main_11 => 
				YarnYield::Instruction(Ch01_Awakening_Line::options_after_main_11.into()),
			Ch01_Awakening_Speech::option_look_1 => 
				YarnYield::Instruction(Ch01_Awakening_Line::option_look_2.into()),
			Ch01_Awakening_Speech::option_look_2 => 
				YarnYield::Instruction(Ch01_Awakening_Line::main_14.into()),
			Ch01_Awakening_Speech::main_14 => 
				YarnYield::Instruction(Ch01_Awakening_Line::option_sleep_1.into()),
		};
	}

	fn line_id(&self) -> &'static str {
		todo!()
	}

	fn tags(&self) -> &'static [&'static str] {
		return match self {
			Ch01_Awakening_Speech::main_3 => &[],
			Ch01_Awakening_Speech::branch_awake_1 => &["right:ethel_disgust"],
			Ch01_Awakening_Speech::branch_awake_2 => &["right:ethel_tired"],
			Ch01_Awakening_Speech::branch_awake_3 => &["right:ethel_tired"],
			Ch01_Awakening_Speech::branch_stamina_1 => &["right:ethel_tired"],
			Ch01_Awakening_Speech::branch_stamina_2 => &["right:ethel_worried"],
			Ch01_Awakening_Speech::branch_else_1 => &[],
			Ch01_Awakening_Speech::main_10 => &[],
			Ch01_Awakening_Speech::main_11 => &[],
			Ch01_Awakening_Speech::option_look_1 => &["left:nema_tired"],
			Ch01_Awakening_Speech::option_look_2 => &["left:nema_tired"],
			Ch01_Awakening_Speech::main_14 => &[],
		};
	}

	fn speaker(&self, storage: &Storage) -> Option<Cow<'static, str>> {
		return match self {
			Ch01_Awakening_Speech::main_3 => Some(storage.get_var::<narrator>().into()),
			Ch01_Awakening_Speech::branch_awake_1 => Some("Ethel".into()),
			Ch01_Awakening_Speech::branch_awake_2 => Some("Ethel".into()),
			Ch01_Awakening_Speech::branch_awake_3 => None,
			Ch01_Awakening_Speech::branch_stamina_1 => Some("Ethel".into()),
			Ch01_Awakening_Speech::branch_stamina_2 => Some("Ethel".into()),
			Ch01_Awakening_Speech::branch_else_1 => Some("Narrator".into()),
			Ch01_Awakening_Speech::main_10 => None,
			Ch01_Awakening_Speech::main_11 => None,
			Ch01_Awakening_Speech::option_look_1 => Some("Nema".into()),
			Ch01_Awakening_Speech::option_look_2 => None,
			Ch01_Awakening_Speech::main_14 => Some(storage.get_var::<narrator>().into()),
		};
	}

	fn text(&self, storage: &Storage) -> Cow<'static, str> {
		return match self {
			Ch01_Awakening_Speech::main_3 =>
				"You wake up. Something you shouldn't have done.".into(),
			Ch01_Awakening_Speech::branch_awake_1 =>
				"Ugh... where am I? Why am I in a... tomb? ... Why am I naked? ...\
				 Why am I sticky? ... At this point: Do I want to know?".into(),
			Ch01_Awakening_Speech::branch_awake_2 => 
				"Last thing I remember was... I feel like I have a reverse hangover...".into(),
			Ch01_Awakening_Speech::branch_awake_3 => 
				format!("You rub your eyes. Your mouth tastes of {} a stale metallic.",
					storage.get_var::<mouth_taste>()).into(),
			Ch01_Awakening_Speech::branch_stamina_1 => 
				"Ugh, at least I didn't wake up in a tomb this time...".into(),
			Ch01_Awakening_Speech::branch_stamina_2 => 
				"... I hope.".into(),
			Ch01_Awakening_Speech::branch_else_1 => 
				"Ethel couldn't wake up.".into(),
			Ch01_Awakening_Speech::main_10 => 
				"You scan the area. Something feels... off. ''Very'' off.".into(),
			Ch01_Awakening_Speech::main_11 => 
				"You're not alone...".into(),
			Ch01_Awakening_Speech::option_look_1 => 
				"Sis!".into(),
			Ch01_Awakening_Speech::option_look_2 =>
				"You slip trying to get up.".into(),
			Ch01_Awakening_Speech::main_14 =>
				"And the scene ended here.".into(),
		};
	}
}
