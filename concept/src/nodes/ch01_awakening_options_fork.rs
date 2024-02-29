#![allow(non_camel_case_types)]

use houtamelo_utils::prelude::CountOrMore;
use serde::{Deserialize, Serialize};

use crate::nodes::ch01_awakening_option_line::Ch01_Awakening_OptionLine;
use crate::options::{OptionLine, OptionsForkTrait};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_OptionsFork {
	after_main_11,
}

impl OptionsForkTrait for Ch01_Awakening_OptionsFork {
	fn options(&self) -> CountOrMore<1, OptionLine> {
		return match self {
			Ch01_Awakening_OptionsFork::after_main_11 => {
				CountOrMore::new([Ch01_Awakening_OptionLine::main_12.into()], 
					vec![Ch01_Awakening_OptionLine::main_13.into()])
			}
		};
	}
}
