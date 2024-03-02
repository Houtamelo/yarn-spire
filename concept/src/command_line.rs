#![allow(non_camel_case_types)]
#![allow(unused)]

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::instruction::YarnYield;
use crate::nodes::ch01_awakening_command::Ch01_Awakening_Command;
use crate::shared_internal::{Storage, YarnCommand};

#[enum_dispatch(CommandLine)]
pub(crate) trait CommandLineTrait {
	fn next(&self, storage: &mut Storage) -> YarnYield;
	
	/// The line's unique identifier, for more, 
	/// see [metadata#line](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#line)
	fn line_id(&self) -> &'static str;
	
	fn command(&self, storage: &Storage) -> YarnCommand;
}

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandLine {
	Ch01_Awakening_Command,
}
