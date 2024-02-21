#![feature(let_chains)]
#![allow(clippy::needless_return)]

use crate::node::RuntimeError;
use crate::prelude::YarnInstruction;

#[cfg(test)] mod tests;

mod traits;
mod node;
mod compass;
mod instruction;
mod end_reason;
mod serialization;
mod rewind;
mod default_storage;

pub type PlayerDecision = usize;
pub type YieldCounter = usize;
type CoroutineInput<TStorage> = (Box<TStorage>, Option<PlayerDecision>);
pub type CoroutineOutput<TStorage, TCmd> = (Box<TStorage>, YarnInstruction<TCmd>);
pub type CoroutineEndput<TStorage> = (Box<TStorage>, Result<(), RuntimeError>);

pub mod prelude {
	pub use corosensei::*;
	pub use corosensei::stack::DefaultStack;

	pub use yarn_spire_codegen::yarn_file;

	pub use crate::{
		CoroutineEndput,
		CoroutineOutput,
		PlayerDecision,
	};
	pub use crate::compass::*;
	pub use crate::compass::choices::*;
	pub use crate::compass::command::*;
	pub use crate::compass::speech::*;
	pub use crate::default_storage;
	pub use crate::instruction::*;
	pub use crate::instruction::YarnInstruction;
	pub use crate::node::*;
	pub use crate::serialization::*;
	pub use crate::traits::*;
}

