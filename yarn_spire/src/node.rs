use thiserror::Error;
use std::fmt::Debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use corosensei::{CoroutineResult, ScopedCoroutine};
use corosensei::stack::DefaultStack;
use houtamelo_utils::prelude::CountOrMore;
use crate::{CoroutineEndput, CoroutineInput, CoroutineOutput, PlayerDecision, rewind, YieldCounter};
use crate::compass::YarnCompass;
use crate::compass::choices::ChoicesCompass;
use crate::compass::command::CommandCompass;
use crate::compass::speech::SpeechCompass;
use crate::prelude::{ChoiceOption, NodeID, VariableStorage, YarnInstruction};
use crate::rewind::RewindError;

/// Represents the current position of the dialogue in a Node.
/// 
/// It is not recommended to interact with this directly, 
/// instead, use the [YarnCompass](crate::prelude::YarnCompass) type provided by [ID::play()](crate::prelude::NodeID::play).
/// 
/// # Serialization
/// See [SerializedYarnNode](crate::prelude::SerializedYarnNode)
pub struct YarnNode<
	'a,
	TStorage: VariableStorage,
	TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
	ID: NodeID<Storage = TStorage, Command = TCmd>,
> {
	pub(crate) id: ID,
	pub(crate) coroutine: ScopedCoroutine<'a, CoroutineInput<TStorage>, CoroutineOutput<TStorage, TCmd>, CoroutineEndput<TStorage>, DefaultStack>,
	pub(crate) output_history: Vec<YarnInstruction<TCmd>>,
	pub(crate) player_decisions: HashMap<YieldCounter, PlayerDecision>,
	pub(crate) storage_at_node_start: Box<TStorage>,
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>,
> YarnNode<'a, TStorage, TCmd, ID> {
	/// The unique marker type representing a Node, this serves the same purpose as using the Node name as a String to play a scene.
	///
	/// See [NodeID](crate::prelude::NodeID) for more information.
	pub fn id(&self) -> ID {
		return self.id;
	}
	
	/// A list of all the instructions that have been yielded by the coroutine so far.
	/// 
	/// You can use this to show the player a history of the dialogue.
	/// 
	/// The output_history is also extremely important for serialization and deserialization. 
	/// 
	/// See [SerializedYarnNode](crate::prelude::SerializedYarnNode) for more information.
	pub fn output_history(&self) -> &[YarnInstruction<TCmd>] {
		return &self.output_history;
	}
	
	/// A map of all the decisions made by the player decisions so far (in this Node).
	/// 
	/// - Key: The index of the instruction that the player made a decision on.
	/// - Value: The decision the player made.
	/// 
	/// This is extremely important for serialization and deserialization.
	/// 
	/// See [SerializedYarnNode](crate::prelude::SerializedYarnNode) for more information.
	pub fn player_decisions(&self) -> &HashMap<YieldCounter, PlayerDecision> {
		return &self.player_decisions;
	}
}

/// A wrapper around a [YarnNode](crate::prelude::YarnNode).
/// 
/// The operations in this crate move `YarnNodes` around a lot, 
/// this box type is meant to reduce the costs of moving.
///
/// It is not recommended to interact with this directly, 
/// instead, use the [YarnCompass](crate::prelude::YarnCompass) type provided by [ID::play()](crate::prelude::NodeID::play).
pub struct BoxedYarnNode<
	'a,
	TStorage: VariableStorage,
	TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
	ID: NodeID<Storage = TStorage, Command = TCmd>,
> {
	pub(crate) inner: Box<YarnNode<'a, TStorage, TCmd, ID>>,
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>,
> From<YarnNode<'a, TStorage, TCmd, ID>> for BoxedYarnNode<'a, TStorage, TCmd, ID> 
{
	fn from(value: YarnNode<'a, TStorage, TCmd, ID>) -> Self {
		return BoxedYarnNode { inner: Box::new(value) };
	}
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>
> Deref for BoxedYarnNode<'a, TStorage, TCmd, ID> 
{
	type Target = YarnNode<'a, TStorage, TCmd, ID>;

	fn deref(&self) -> &Self::Target {
		return &self.inner;
	}
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>
> DerefMut for BoxedYarnNode<'a, TStorage, TCmd, ID>
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		return &mut self.inner;
	}
}

/// Represents an error that occurred when trying to advance the dialogue.
/// 
/// This error should not be exposed to users in most cases, 
/// they are generally avoided by simply using the [YarnCompass](crate::prelude::YarnCompass) type.
/// 
/// If any of these errors happened despite using a YarnCompass, 
/// then it's likely a programming error and should be reported to Houtamelo.
/// 
/// See each variant's documentation for more information.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RuntimeError {
	#[error("Did not expect player decision, got: {got:?}")]
	UnexpectedPlayerDecision {
		got: PlayerDecision,
	},
	#[error("Expected player decision, but none was provided.\n\
	Options: {options_provided:?}")]
	ExpectedPlayerDecision {
		options_provided: CountOrMore<1, ChoiceOption>,
	},
	#[error("Invalid player decision provided.\n\
	Options: {options_provided:?}\n\
	Max index: {}\n\
	Got: {got:?}", options_provided.len() - 1)]
	InvalidPlayerDecision {
		options_provided: CountOrMore<1, ChoiceOption>, 
		got: Option<PlayerDecision>,
	},
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>> 
BoxedYarnNode<'a, TStorage, TCmd, ID> {
	/// Creates a new runtime Node assuming the coroutine has not been started yet.  
	///
	/// It is not recommended to call this directly, 
	/// if you wish to start a scene, call [NodeID::play](crate::prelude::NodeID::play)
	/// on the ID type that represents the desired scene...
	/// 
	/// ___
	/// 
	/// # Cannot Panic
	pub fn new(id: ID, storage_to_backup: &Box<TStorage>,
	           coroutine: ScopedCoroutine<
		           'a,
		           (Box<TStorage>, Option<PlayerDecision>),
		           (Box<TStorage>, YarnInstruction<TCmd>),
		           (Box<TStorage>, Result<(), RuntimeError>),
		           DefaultStack
	           >) -> Self {
		return BoxedYarnNode::from(
			YarnNode {
				id,
				coroutine,
				output_history: Vec::new(),
				player_decisions: HashMap::new(),
				storage_at_node_start: storage_to_backup.clone(),
			});
	}

	/// Advances the dialogue, returning a [compass](crate::prelude::YarnCompass) containing the next yield.
	///
	/// It is not recommended to call this directly, as providing incorrect input **will** lead to a runtime error.
	///
	/// Instead, you should be interacting with the [YarnCompass](crate::prelude::YarnCompass) type,
	/// which provides type safety to avoid incorrect inputs.
	///
	/// ___
	///
	/// # Cannot Panic
	pub fn next(mut self, input: CoroutineInput<TStorage>)
	            -> YarnCompass<'a , TStorage, TCmd, ID> {
		if let Some(choice) = input.1 {
			let index = self.output_history.len();
			self.player_decisions
				.insert(index, choice);
		}

		let result = self.inner.coroutine.resume(input);
		return match result {
			CoroutineResult::Yield((storage, instruction)) => {
				self.output_history.push(instruction.clone());
				match instruction {
					YarnInstruction::Speech(speech) =>
						YarnCompass::Speech(SpeechCompass {
							storage,
							node: self,
							speech,
						}),
					YarnInstruction::Command(command) => 
						YarnCompass::Command(CommandCompass {
							storage,
							node: self,
							command,
						}),
					YarnInstruction::Choices(choices) => 
						YarnCompass::Choices(ChoicesCompass {
							storage,
							node: self,
							options: choices,
						}),
				}
			}
			CoroutineResult::Return((storage, maybe_error)) =>
				YarnCompass::NodeEnd(storage, maybe_error.err()),
		};
	}

	/// # Behavior
	/// - Attempts to rewind dialogue by `steps` amount of yields.
	/// - This effectively "goes backwards" inside the dialogue.
	/// 
	/// ___
	/// 
	/// # Parameters
	/// - `steps`: The amount of yields to rewind by, this is how many instructions the dialogue should go "backwards",
	/// effectively undoing the last `steps` yields.
	/// - If `steps` >= output_history, then this is the same as calling [restart_scene](crate::prelude::BoxedYarnNode::restart_scene).
	/// 
	/// ___
	/// 
	/// # Returns
	/// - Ok: A compass starting at the rewinded point.
	/// - Err: An error explaining why the rewind failed.
	/// 
	/// ___
	/// 
	/// # Usage
	/// - Use this to implement a "rewind" feature in your game, often used in Visual Novels,
	///  where the player can go back to a previous point in the dialogue.
	/// This feature is the flagship of the popular VN engine Ren'Py.
	/// Please keep in mind that this does not alter the state of the rest of your game,
	/// you'll have to figure out how to handle that yourself.
	/// The examples folder may have an example on how to do this.
	/// - If you're unsure about how many steps to take, you can check the output_history to pick a point to rewind to,
	/// the amount of steps should be the length of the output_history minus the index you want to rewind to.
	/// 
	/// ___
	/// 
	/// # Caution
	/// - The rewinded process is the same as used when converting a [SerializedYarnNode](crate::prelude::SerializedYarnNode),
	/// into a [YarnCompass](crate::prelude::YarnCompass), which is achieved by calling [into_compass](crate::prelude::SerializedYarnNode::into_compass).
	/// This means that the same rules apply, TLDR: your storage needs to produce deterministic results, 
	/// read the documentation on [into_compass](crate::prelude::SerializedYarnNode::into_compass) for more.
	/// 
	/// ___
	/// 
	/// # Cannot Panic
	/// 
	/// ___
	/// 
	/// # Example
	/// 
	/// ```rs
	/// let compass = ...;
	/// 
	/// let advanced_compass = compass.next((storage, None));
	/// 
	/// let rewinded_compass = advanced_compass.rewind_by(1);
	/// 
	///  // This comparison is technically impossible because YarnCompass does not implement PartialEq.
	///  // Still, I wrote it here to show that rewinding by 1 is the opposite of calling `next`. 
	/// assert_eq!(compass, rewinded_compass);
	/// ```
	pub fn rewind_by(self, steps: usize) -> Result<YarnCompass<'a, TStorage, TCmd, ID>, RewindError<TCmd>> {
		return rewind::attempt_rewind(
			self.inner.output_history, 
			self.inner.player_decisions, 
			self.inner.storage_at_node_start, 
			steps
		);
	}
	
	/// # Behavior
	/// - Restarts the scene, effectively rewinding the dialogue to the start of the Node.
	/// - Unlike [rewind_by](crate::prelude::BoxedYarnNode::rewind_by)
	/// and [SerializedYarnNode::into_compass](crate::prelude::SerializedYarnNode::into_compass),
	/// this is completely safe and doesn't care about the determinism rules 
	/// explained on [SerializedYarnNode::into_compass](crate::prelude::SerializedYarnNode::into_compass)
	/// 
	/// ___
	/// 
	/// # Returns
	/// - A compass starting at the beginning of the Node.
	/// 
	/// ___
	/// 
	/// # Cannot Panic
	pub fn restart_scene(self) -> YarnCompass<'a, TStorage, TCmd, ID> {
		return ID::play(self.inner.storage_at_node_start);
	}
}
