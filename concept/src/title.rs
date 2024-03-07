#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::shared_internal::*;

/// The original YarnSpinner's [tracking setting](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackingSetting {
	Always,
	Never,
}

pub trait NodeTitleTrait {
	fn tags(&self) -> &'static[&'static str];
	fn tracking(&self) -> TrackingSetting;
	fn custom_metadata(&self) -> &'static[&'static str];
	fn start(&self, storage: &mut Storage) -> YarnYield;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeTitle {
	Ch01_Awakening,
	Ch01_First_Fight,
}

impl NodeTitleTrait for NodeTitle {
	fn tags(&self) -> &'static [&'static str] {
		return match self {
			NodeTitle::Ch01_Awakening => 
				Ch01_Awakening.tags(),
			NodeTitle::Ch01_First_Fight => 
				Ch01_First_Fight.tags(),
		};
	}

	fn tracking(&self) -> TrackingSetting {
		return match self {
			NodeTitle::Ch01_Awakening => 
				Ch01_Awakening.tracking(),
			NodeTitle::Ch01_First_Fight => 
				Ch01_First_Fight.tracking(),
		};
	}

	fn custom_metadata(&self) -> &'static [&'static str] {
		return match self {
			NodeTitle::Ch01_Awakening => 
				Ch01_Awakening.custom_metadata(),
			NodeTitle::Ch01_First_Fight => 
				Ch01_First_Fight.custom_metadata(),
		};
	}

	fn start(&self, storage: &mut Storage) -> YarnYield {
		return match self {
			NodeTitle::Ch01_Awakening => 
				Ch01_Awakening.start(storage),
			NodeTitle::Ch01_First_Fight => 
				Ch01_First_Fight.start(storage),
		};
	}
}
