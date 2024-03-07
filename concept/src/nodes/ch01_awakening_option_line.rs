#![allow(non_camel_case_types)]

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::instruction::YarnYield;
use crate::options::OptionLineTrait;
use crate::shared_internal::{Ch01_Awakening_Line, Storage};
use crate::shared_internal::vars::ethel_awake;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_OptionLine {
	main_12,
	main_13,
}

#[allow(unused_variables)]
impl OptionLineTrait for Ch01_Awakening_OptionLine {
	fn next(&self, storage: &mut Storage) -> YarnYield {
		return match self {
			Ch01_Awakening_OptionLine::main_12 => 
				YarnYield::Instruction(Ch01_Awakening_Line::option_look_1.into()),
			Ch01_Awakening_OptionLine::main_13 => 
				YarnYield::Instruction(Ch01_Awakening_Line::option_sleep_1.into()),
		};
	}

	fn line_id(&self) -> &'static str {
		todo!()
	}

	fn tags(&self) -> &'static [&'static str] {
		return match self {
			Ch01_Awakening_OptionLine::main_12 => &[],
			Ch01_Awakening_OptionLine::main_13 => &[],
		};
	}

	fn text(&self, storage: &Storage) -> Cow<'static, str> {
		return match self {
			Ch01_Awakening_OptionLine::main_12 => 
				"Look Around".into(),
			Ch01_Awakening_OptionLine::main_13 => 
				"Go to sleep".into(),
		};
	}

	fn is_available(&self, storage: &Storage) -> Option<bool> {
		return match self {
			Ch01_Awakening_OptionLine::main_12 => 
				None,
			Ch01_Awakening_OptionLine::main_13 => 
				Some(storage.get_var::<ethel_awake>())
		};
	}
}
