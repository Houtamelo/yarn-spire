#![feature(let_chains)]
#![allow(clippy::needless_return)]

pub mod choice_option;
pub mod line;
pub mod runtime;
pub mod speech;
pub mod title;
pub mod storage;
pub mod nodes;
pub mod var_trait;

pub type PlayerDecision = usize;
pub type YieldCounter = usize;

#[allow(unused)]
pub(crate) mod shared_internal {
	pub use yarn_spire_codegen::yarn_file;

	pub use crate::{
		PlayerDecision,
		YieldCounter,
	};
	pub use crate::choice_option::*;
	pub use crate::title::*;
	pub use crate::runtime::*;
	pub use crate::line::*;
	pub use crate::speech::*;
	pub use crate::storage::*;
	pub use crate::nodes::*;
	pub use crate::nodes::ch01_awakening::*;
}

