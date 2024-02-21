//! # Usage example
//! 
//! ```rs
//! let storage: Box<VariableStorage> = ...;
//! let mut compass: YarnCompass<_> = NodeID::play(storage);
//! 
//! // Advance the dialogue a few times
//! loop {
//!     let compass = 
//!         match compass {
//!             /../
//!         };
//! }
//! 
//! // Create a serializable data type from the compass
//! let serialized_node = SerializedYarnNode::from_compass_owned(compass);
//! let json = serde_json::to_string(&serialized_node).unwrap();
//! 
//! // Deserialize the serializable data type back into a compass
//! let deserialized_node: SerializedYarnNode<_, _> = serde_json::from_str(&json).unwrap();
//! let compass = deserialized_node.into_compass().unwrap();
//! 
//! // Continue the dialogue
//! loop {
//!     let compass = 
//!         match compass {
//!             /../
//!         };
//! }
//! ```


use std::fmt::Debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use crate::{PlayerDecision, rewind, YieldCounter};
use crate::compass::choices::ChoicesCompass;
use crate::compass::command::CommandCompass;
use crate::compass::speech::SpeechCompass;
use crate::compass::YarnCompass;
use crate::instruction::YarnInstruction;
use crate::node::RuntimeError;
use crate::prelude::{NodeID, VariableStorage};
use thiserror::Error;
use rewind::attempt_rewind;
use crate::rewind::RewindError;

/// Types of errors that can happen when trying to deserialize a [SerializedYarnNode](crate::prelude::SerializedYarnNode) 
/// into a [YarnCompass](crate::prelude::YarnCompass).
///
/// See each variant's documentation for more.
#[derive(Debug, Clone, Error)]
pub enum DeserializationError<TStorage: VariableStorage, TCmd: Clone + PartialEq + Debug> {
	/// The Deserialization process uses rewinding to achieve it's result,
	/// meaning it's subject to the same errors as rewinding.
	/// 
	/// See [RewindError](crate::rewind::RewindError) for more information.
	#[error("Error during rewind: {0}")]
	RewindError(RewindError<TCmd>),
	/// See error message bellow.
	#[error("The coroutine's storage at the end did not match the expected storage, which was saved when serializing the node.\n\
	Expected: {expected:?}\n\
	Found: {found:?}")]
	VariableStorageResultMismatch {
		expected: Box<TStorage>,
		found: Box<TStorage>,
	},
	/// See error message bellow.
	#[error("The coroutine ended after rewind, serializing finished nodes is not allowed.\n\
	Maybe error: {0:?}")]
	NodeEnded(Option<RuntimeError>),
}

/// # Usage
/// - Data type containing everything necessary to serialize/deserialize a running Yarn node.
/// 
/// ___
/// 
/// # How to build
/// - You can create a SerializedYarnNode by calling [from_compass_ref](SerializedYarnNode::from_compass_ref) 
/// or [from_compass_owned](SerializedYarnNode::from_compass_owned) 
/// or [from_speech_compass](SerializedYarnNode::from_speech_compass),
/// or [from_command_compass](SerializedYarnNode::from_command_compass),
/// or [from_choices_compass](SerializedYarnNode::from_choices_compass).
/// 
/// ___
/// 
/// # Serialization
/// - You can serialize this data type using [serde](https://crates.io/crates/serde)'s [Serialize](serde::Serialize) trait.
/// 
/// ___
/// 
/// # Deserialization
/// - You can deserialize this data type using [serde](https://crates.io/crates/serde)'s [Deserialize](serde::de::Deserialize) trait, 
/// then call [into_compass](SerializedYarnNode::into_compass) to get a runtime [Compass](YarnCompass).
/// 
/// ___
/// 
/// Read each field's documentation for more information.
#[derive(Serialize, Clone)]
pub struct SerializedYarnNode<
	TStorage: VariableStorage,
	TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
	ID: NodeID<Storage = TStorage, Command = TCmd>,
> {
	/// - An opaque type that identifies the Node. 
	/// It implements [SceneID](crate::prelude::NodeID),
	/// acting like a key that tells us what the Node is and how to play it.
	/// - This type is generated by the [yarn_file!](yarn_spinner_aot_proc_macros::yarn_file) macro.  
	pub id: ID,
	/// - A list, in order, of each output the Node yielded before being saved.
	pub output_history: Vec<YarnInstruction<TCmd>>,

	/// - The decisions made by the player during the execution of the node.
	/// - This is used to resume the dialogue from the same state it was saved.
	pub player_decisions: HashMap<YieldCounter, PlayerDecision>,
	/// A copy of the storage made just before playing the Node.
	pub storage_at_node_start: Box<TStorage>,
	/// A copy of the storage made just before serializing the Node.
	pub storage_at_save: Box<TStorage>,
}

impl<TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>> 
SerializedYarnNode<TStorage, TCmd, ID> {
	
	/// # Behavior
	/// - Attempts to deserialize into a runtime [Compass](YarnCompass).
	/// 
	/// ___
	///
	/// # Deserialization
	/// - Is achieved by re-running the node from the start, giving the coroutine the same inputs as the player once did.
	/// (Saved in [player_decisions](SerializedYarnNode::player_decisions))
	/// - We also keep track of the outputs during the process, comparing them to the [output_history](SerializedYarnNode::output_history) 
	/// to ensure the deserialization is restoring the dialogue state to the same as it was before saving.
	/// - At the end of the process, we also compare the mutated storage with the one provided by the serialized data.
	/// - All of this means that deserialization requires two things to run successfully:
	///    - The input/output of the Node needs to provide deterministic results, the same input must always provide the same output. 
	/// This rule can be broken if, for example, one of storage's functions uses RNG without keeping track of the RNG seed.
	///    - You may not manually modify the storage's state during the coroutine's execution.
	/// Modifying the storage's state can possibly make its functions return different outputs for the same inputs, breaking the first rule.  
	/// This is why functions that allow modifying the storage are marked as unsafe.
	///
	/// ___
	///
	/// # Returns
	/// - Ok if the deserialization was successful.
	/// - [Err](crate::serialization::DeserializationError) if the deserialization failed.
	/// 
	/// ___
	/// 
	/// # Cannot panic
	pub fn into_compass<'a>(self) 
		-> Result<YarnCompass<'a, TStorage, TCmd, ID>, DeserializationError<TStorage, TCmd>> {
		let compass_result =
			attempt_rewind(self.output_history, self.player_decisions, self.storage_at_node_start, 0)
				.map_err(DeserializationError::RewindError)?;
		
		return match compass_result {
			YarnCompass::Speech(compass) =>
				if compass.storage == self.storage_at_save {
					Ok(YarnCompass::Speech(compass))
				} else {
					Err(DeserializationError::VariableStorageResultMismatch {
						expected: self.storage_at_save,
						found: compass.storage,
					})
				},
			YarnCompass::Command(compass) =>
				if compass.storage == self.storage_at_save {
					Ok(YarnCompass::Command(compass))
				} else {
					Err(DeserializationError::VariableStorageResultMismatch {
						expected: self.storage_at_save,
						found: compass.storage,
					})
				},
			YarnCompass::Choices(compass) =>
				if compass.storage == self.storage_at_save {
					Ok(YarnCompass::Choices(compass))
				} else {
					Err(DeserializationError::VariableStorageResultMismatch {
						expected: self.storage_at_save,
						found: compass.storage,
					})
				},
			
			YarnCompass::NodeEnd(_, maybe_error) => 
				Err(DeserializationError::NodeEnded(maybe_error)),
		};
	}

	/// # Behavior
	/// - Creates a [SerializedYarnNode] from a compass, you can use [Serialize](serde::Serialize) to save the dialogue state and resume it later.
	/// - Please keep in mind this is achieved by cloning, use [from_compass_owned](SerializedYarnNode::from_compass_owned) if you want to avoid cloning.
	/// ___
	/// 
	/// # Returns
	/// - None if compass == [YarnCompass::NodeEnd]
	/// - Some otherwise.
	/// 
	/// ___
	///
	/// # Usage
	/// - Only use this if you don't know the type at compile time, if you do,
	/// use [from_speech_compass](SerializedYarnNode::from_speech_compass),
	/// [from_command_compass](SerializedYarnNode::from_command_compass) 
	/// or [from_choices_compass](SerializedYarnNode::from_choices_compass),
	/// as those operations are guaranteed to return Some().
	/// 
	/// ___
	/// 
	/// # Cannot panic
	pub fn from_compass_ref(compass: &YarnCompass<TStorage, TCmd, ID>) -> Option<Self> {
		let (node, storage_at_save) = match compass {
			| YarnCompass::Speech(SpeechCompass { node, storage, .. }) 
			| YarnCompass::Command(CommandCompass { node, storage, ..})
			| YarnCompass::Choices(ChoicesCompass { node, storage, .. }) => {
				(node, storage)
			},
			YarnCompass::NodeEnd(_, _) => return None,
		};
		
		return Some(Self {
			id: node.id.clone(),
			output_history: node.output_history.clone(),
			player_decisions: node.player_decisions.clone(),
			storage_at_node_start: node.storage_at_node_start.clone(),
			storage_at_save: storage_at_save.clone(),
		});
	}

	/// # Behavior
	/// - Creates a [SerializedYarnNode] from a compass, you can use [Serialize](serde::Serialize) to save the dialogue state and resume it later.
	///  ___
	///
	/// # Returns
	/// - None if compass == [YarnCompass::NodeEnd]
	/// - Some otherwise.
	///
	/// ___
	///
	/// # Usage
	/// - Only use this if you don't know the type at compile time, if you do,
	/// use [from_speech_compass](SerializedYarnNode::from_speech_compass),
	/// [from_command_compass](SerializedYarnNode::from_command_compass) 
	/// or [from_choices_compass](SerializedYarnNode::from_choices_compass),
	/// as those operations are guaranteed to return Some().
	///
	/// ___
	///
	/// # Cannot panic
	pub fn from_compass_owned(compass: YarnCompass<TStorage, TCmd, ID>) -> Option<Self> {
		let (node, storage_at_save) = match compass {
			| YarnCompass::Speech(SpeechCompass { node, storage, .. })
			| YarnCompass::Command(CommandCompass { node, storage, ..})
			| YarnCompass::Choices(ChoicesCompass { node, storage, .. }) => {
				(*node.inner, storage)
			},
			YarnCompass::NodeEnd(_, _) => return None,
		};

		return Some(Self {
			id: node.id,
			output_history: node.output_history,
			player_decisions: node.player_decisions,
			storage_at_node_start: node.storage_at_node_start,
			storage_at_save,
		});
	}

	/// # Behavior
	/// - Creates a [SerializedYarnNode] from a compass, you can use [Serialize](serde::Serialize) to save the dialogue state and resume it later.
	///  ___
	///
	/// # Returns
	/// - The serializable data type.
	///
	/// ___
	///
	/// # Usage
	/// - Use this if you know the **type** at compile time, as the return value is guaranteed to be Some(),
	/// otherwise, use [from_compass_ref](SerializedYarnNode::from_compass_ref)
	/// or [from_compass_owned](SerializedYarnNode::from_compass_owned).
	///
	/// ___
	///
	/// # Cannot panic
	pub fn from_speech_compass(compass: SpeechCompass<TStorage, TCmd, ID>) -> Self {
		let (node, storage_at_save) = 
			(*compass.node.inner, compass.storage);

		return Self {
			id: node.id,
			output_history: node.output_history,
			player_decisions: node.player_decisions,
			storage_at_node_start: node.storage_at_node_start,
			storage_at_save,
		};
	}

	/// # Behavior
	/// - Creates a [SerializedYarnNode] from a compass, you can use [Serialize](serde::Serialize) to save the dialogue state and resume it later.
	///  ___
	///
	/// # Returns
	/// - The serializable data type.
	///
	/// ___
	///
	/// # Usage
	/// - Use this if you know the **type** at compile time, as the return value is guaranteed to be Some(),
	/// otherwise, use [from_compass_ref](SerializedYarnNode::from_compass_ref)
	/// or [from_compass_owned](SerializedYarnNode::from_compass_owned).
	///
	/// ___
	///
	/// # Cannot panic
	pub fn from_command_compass(compass: CommandCompass<TStorage, TCmd, ID>) -> Self {
		let (node, storage_at_save) = 
			(*compass.node.inner, compass.storage);

		return Self {
			id: node.id,
			output_history: node.output_history,
			player_decisions: node.player_decisions,
			storage_at_node_start: node.storage_at_node_start,
			storage_at_save,
		};
	}

	/// # Behavior
	/// - Creates a [SerializedYarnNode] from a compass, you can use [Serialize](serde::Serialize) to save the dialogue state and resume it later.
	///  ___
	///
	/// # Returns
	/// - The serializable data type.
	///
	/// ___
	///
	/// # Usage
	/// - Use this if you know the **type** at compile time, as the return value is guaranteed to be Some(),
	/// otherwise, use [from_compass_ref](SerializedYarnNode::from_compass_ref)
	/// or [from_compass_owned](SerializedYarnNode::from_compass_owned).
	///
	/// ___
	///
	/// # Cannot panic
	pub fn from_choices_compass(compass: ChoicesCompass<TStorage, TCmd, ID>) -> Self {
		let (node, storage_at_save) = 
			(*compass.node.inner, compass.storage);

		return Self {
			id: node.id,
			output_history: node.output_history,
			player_decisions: node.player_decisions,
			storage_at_node_start: node.storage_at_node_start,
			storage_at_save,
		};
	}
}
