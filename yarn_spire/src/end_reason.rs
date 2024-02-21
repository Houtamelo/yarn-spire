use std::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::compass::YarnCompass;
use crate::prelude::{NodeID, RuntimeError, VariableStorage};

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
	ReachedNodeEnd,
	StopCommand,
	RuntimeError(RuntimeError),
}

pub trait DynTest {
	type Storage: VariableStorage;
	type Command: Clone + Debug + Serialize + DeserializeOwned;//Clone + PartialEq + Debug + Serialize + DeserializeOwned;

	
}

fn test<T, T1>(input: Box<dyn DynTest<Storage = T, Command = T1>>) {
	
}


