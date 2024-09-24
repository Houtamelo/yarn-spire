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

pub trait INodeTitle {
	#[must_use]
	fn tags(&self) -> &'static[&'static str];
	#[must_use]
	fn tracking(&self) -> TrackingSetting;
	#[must_use]
	fn custom_metadata(&self) -> &'static[&'static str];
	#[must_use]
	fn start(&self, storage: &mut Storage) -> YarnYield;
}

declarative_type_state::unit_enum_delegated! {
	ENUM_OUT: {
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
		pub enum NodeTitle {
			Ch01_Awakening,
			Ch01_First_Fight,
		}
	}
	
	DELEGATES: { 
		impl trait INodeTitle {
			[fn tags(&self) -> &'static [&'static str]]
			[fn tracking(&self) -> TrackingSetting]
			[fn custom_metadata(&self) -> &'static [&'static str]]
			[fn start(&self, storage: &mut Storage) -> YarnYield]
		}
	}
}