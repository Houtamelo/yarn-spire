pub mod speech;
pub mod command;
pub mod choices;

use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use choices::ChoicesCompass;
use command::CommandCompass;
use speech::SpeechCompass;
use crate::prelude::{RuntimeError, NodeID, VariableStorage, YarnInstruction};
use crate::rewind::RewindError;

/// # The compass provides directions to navigate a dialogue tree.
/// It's a wrapper around the coroutine that represents your dialogue scripts.
/// 
/// Each variant(except the `NodeEnded`) contains the last yielded Instruction, 
/// to get the next, call the `next` method on the variant,
/// [speech::next](crate::prelude::SpeechCompass::next),
/// [command::next](crate::prelude::CommandCompass::next) 
/// or [choices::next](crate::prelude::ChoicesCompass::next).
pub enum YarnCompass<'a,
                 TStorage: VariableStorage,
                 TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
                 ID: NodeID<Storage = TStorage, Command = TCmd>>  {
	Speech(SpeechCompass<'a, TStorage, TCmd, ID>),
	Command(CommandCompass<'a, TStorage, TCmd, ID>),
	Choices(ChoicesCompass<'a, TStorage, TCmd, ID>),
	NodeEnd(Box<TStorage>, Option<RuntimeError>),
}

/// The result of resuming a dialogue. (Compass/Node)
/// It can either be a [instruction](crate::prelude::YarnInstruction)
/// or a signal that the dialogue has finished.
/// 
/// For more information on instructions, see [YarnInstruction](crate::prelude::YarnInstruction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum YarnYield<TCmd: Clone + PartialEq + Debug> {
	Instruction(YarnInstruction<TCmd>),
	NodeFinished,
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>> 
YarnCompass<'a, TStorage, TCmd, ID> {
	
	/// Returns copy of the coroutine's latest yield.
	/// 
	/// If you don't want the performance hit from copying, you can match on the result directly.
	pub fn yield_result(&self) -> Result<YarnYield<TCmd>, RuntimeError> {
		return match self {
			YarnCompass::Speech(compass) => 
				Ok(YarnYield::Instruction(YarnInstruction::Speech(compass.speech.clone()))),
			YarnCompass::Command(compass) =>
				Ok(YarnYield::Instruction(YarnInstruction::Command(compass.command.clone()))),
			YarnCompass::Choices(compass) =>
					Ok(YarnYield::Instruction(YarnInstruction::Choices(compass.options.clone()))),
			YarnCompass::NodeEnd(_, maybe_error) => 
				match maybe_error {
					None => Ok(YarnYield::NodeFinished),
					Some(err) => Err(err.clone()),
				}
		};
	}
	
	/// Replaces the compass's inner storage with `new_storage`.
	///
	/// This is marked as unsafe because manually replacing the storage
	/// can break the determinism necessary for serialization/deserialization.
	///
	/// See [SerializedYarnNode](crate::prelude::SerializedYarnNode) for more information.
	pub unsafe fn replace_storage(&mut self, new_storage: Box<TStorage>) {
		match self {
			| YarnCompass::Speech(SpeechCompass { storage, .. })
			| YarnCompass::Command(CommandCompass { storage, ..}) 
			| YarnCompass::Choices(ChoicesCompass { storage, ..})
			| YarnCompass::NodeEnd(storage, _) => {
				*storage = new_storage;
			}
		}
	}

	/// Bridge to [Node::rewind_by](crate::prelude::BoxedYarnNode::rewind_by), 
	/// see that method's documentation for more information. 
	/// 
	/// If you know the YarnCompass variant at compile time,
	/// call their versions 
	/// ([Speech](crate::prelude::SpeechCompass::rewind_by),
	/// [Command](crate::prelude::CommandCompass::rewind_by),
	/// [Choices](crate::prelude::ChoicesCompass::rewind_by))
	/// instead, as they have one less failure point.
	pub fn rewind_by(self, steps: usize) -> Result<YarnCompass<'a, TStorage, TCmd, ID>, RewindError<TCmd>> {
		let node =
			match self {
				YarnCompass::Speech(speech_compass) =>
					speech_compass.node,
				YarnCompass::Command(command_compass) =>
					command_compass.node,
				YarnCompass::Choices(choices_compass) =>
					choices_compass.node,
				YarnCompass::NodeEnd(_, _) =>
					return Err(RewindError::CannotRewindFinishedNode),
			};
		
		return node.rewind_by(steps);
	}

	/// Bridge to [Node::rewind_by](crate::prelude::BoxedYarnNode::restart_scene), 
	/// see that method's documentation for more information. 
	///
	/// If you know the YarnCompass variant at compile time,
	/// call their versions 
	/// ([Speech](crate::prelude::SpeechCompass::restart_scene),
	/// [Command](crate::prelude::CommandCompass::restart_scene),
	/// [Choices](crate::prelude::ChoicesCompass::restart_scene))
	/// instead, as they have one less failure point.
	pub fn restart_scene(self) -> Result<YarnCompass<'a, TStorage, TCmd, ID>, RewindError<TCmd>> {
		let storage_at_start = 
			match self {
				YarnCompass::Speech(speech_compass) => 
					speech_compass.node.inner.storage_at_node_start,
				YarnCompass::Command(command_compass) => 
					command_compass.node.inner.storage_at_node_start,
				YarnCompass::Choices(choices_compass) => 
					choices_compass.node.inner.storage_at_node_start,
				YarnCompass::NodeEnd(_, _) => 
					return Err(RewindError::CannotRewindFinishedNode),
			};
		
		return Ok(ID::play(storage_at_start));
	}
}