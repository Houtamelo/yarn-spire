#![allow(non_camel_case_types)]

use houtamelo_utils::prelude::CountOrMore;
use serde::{Deserialize, Serialize};

use crate::options::{OptionLine, IOptionsFork};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ch01_Awakening_OptionsFork {
	after_main_11,
}

impl IOptionsFork for Ch01_Awakening_OptionsFork {
	fn options(&self) -> CountOrMore<1, OptionLine> {
		todo!()
	}
}
