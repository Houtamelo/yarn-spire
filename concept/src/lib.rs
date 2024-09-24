#![allow(clippy::tabs_in_doc_comments)]

use crate::shared_internal::*;

pub mod options;
pub mod instruction;
pub mod runtime;
pub mod title;
pub mod storage;
pub mod nodes;
pub mod var_trait;
pub mod speech;
pub mod command_line;
mod built_in_functions;

pub type PlayerDecision = usize;
pub type YieldCounter = usize;

#[allow(unused)]
pub(crate) mod shared_internal {
	pub use super::{
		PlayerDecision,
		YieldCounter,
	};
	pub use super::command_line::*;
	pub use super::instruction::*;
	pub use super::nodes::*;
	pub use super::nodes::ch01_awakening::*;
	pub use super::options::*;
	pub use super::runtime::*;
	pub use super::speech::*;
	pub use super::storage::*;
	pub use super::title::*;
	pub use super::var_trait::*;
}

/*
fn test() {
	let mut storage = Storage::new();
	
	let node = Ch01_Awakening;
	
	match node.start(&mut storage) {
		YarnYield::Instruction(instruction) => {
			match instruction {
				Instruction::Speech(speech) => {
					speech.next(&mut storage);
				}
				Instruction::Command(cmd) => {}
				Instruction::Options(options) => {}
			}
		}
		YarnYield::Finished => return,
	}
	
}
*/

