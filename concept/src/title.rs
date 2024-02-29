#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::shared_internal::*;

/// The original YarnSpinner's [tracking setting](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackingSetting {
	Always,
	Never,
}

#[enum_dispatch(NodeTitle)]
pub(crate) trait NodeTitleTrait {
	fn tags(&self) -> &'static[&'static str];
	fn tracking(&self) -> TrackingSetting;
	fn custom_metadata(&self) -> &'static[&'static str];
	fn start(&self, storage: &mut Storage) -> YarnYield;
}

#[enum_dispatch]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeTitle {
	Ch01_Awakening,
	Ch01_First_Fight,
}
