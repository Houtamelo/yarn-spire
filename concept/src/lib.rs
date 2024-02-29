#![allow(clippy::needless_return)]

pub mod options;
pub mod line;
pub mod runtime;
pub mod title;
pub mod storage;
pub mod nodes;
pub mod var_trait;
pub mod speech;
pub mod command_line;

pub type PlayerDecision = usize;
pub type YieldCounter = usize;

#[allow(unused)]
pub(crate) mod shared_internal {
	pub use super::{
		PlayerDecision,
		YieldCounter,
	};
	pub use super::command_line::*;
	pub use super::line::*;
	pub use super::nodes::*;
	pub use super::nodes::ch01_awakening::*;
	pub use super::options::*;
	pub use super::runtime::*;
	pub use super::speech::*;
	pub use super::storage::*;
	pub use super::title::*;
}

