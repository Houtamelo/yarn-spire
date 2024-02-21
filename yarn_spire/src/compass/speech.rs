use std::fmt::Debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::compass::YarnCompass;
use crate::node::BoxedYarnNode;
use crate::prelude::{NodeID, Speech, VariableStorage};
use crate::rewind::RewindError;

/// # The compass provides directions to navigate a dialogue tree.
/// It's a wrapper around the coroutine that represents your dialogue scripts.
///
/// This variant is returned when the coroutine yields a `Speech` instruction
/// (AKA regular dialogue line)
///
/// It ensures that you do not mistakenly provide 
/// a player decision when continuing the dialogue.
///
/// You can do so by calling [next](crate::prelude::SpeechCompass::next).
pub struct SpeechCompass<'a,
                         TStorage: VariableStorage,
                         TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
                         ID: NodeID<Storage = TStorage, Command = TCmd>> {
	pub(crate) storage: Box<TStorage>,
	pub(crate) node: BoxedYarnNode<'a, TStorage, TCmd, ID>,
	pub(crate) speech: Speech,
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>> 
SpeechCompass<'a, TStorage, TCmd, ID> {
	/// Advances the dialogue, returning a compass with the next yield.
	///
	/// ___
	///
	/// # Cannot Panic
	pub fn next(self) -> YarnCompass<'a, TStorage, TCmd, ID> {
		return self.node.next((self.storage, None));
	}

	/// The speech yielded when this compass was created.
	/// 
	/// See [Speech](crate::prelude::Speech) for more information.
	pub fn speech(&self) -> &Speech {
		return &self.speech;
	}
	
	/// Readonly access to the storage owned by this compass.
	pub fn storage(&self) -> &TStorage {
		return &self.storage;
	}

	/// Mutable access to the storage owned by this compass.
	///
	/// This is marked as unsafe because manually modifying the storage
	/// can break the determinism necessary for serialization/deserialization.
	pub unsafe fn storage_mut(&mut self) -> &mut TStorage {
		return &mut *self.storage;
	}

	/// Discards the coroutine and returns the storage previously owned by this compass.
	pub fn into_storage(self) -> Box<TStorage> {
		return self.storage;
	}

	/// Readonly access to the runtime node state owned by this compass.
	///
	/// See [YarnNode](crate::prelude::YarnNode) for more information.
	pub fn node(&self) -> &BoxedYarnNode<'a, TStorage, TCmd, ID> {
		return &self.node;
	}

	/// Mutable access to the runtime node state owned by this compass. 
	///
	/// This is marked as unsafe because manually modifying the storage
	/// can break the determinism necessary for serialization/deserialization.
	pub unsafe fn node_mut(&mut self) -> &mut BoxedYarnNode<'a, TStorage, TCmd, ID> {
		return &mut self.node;
	}

	/// Gives access to all the fields owned by this compass.
	///
	/// This is marked as unsafe because manually modifying the storage (or the node itself)
	/// can break the determinism necessary for serialization/deserialization.
	pub fn deconstruct(self) -> (Box<TStorage>, BoxedYarnNode<'a, TStorage, TCmd, ID>, Speech) {
		return (self.storage, self.node, self.speech);
	}

	/// Rewinds the dialogue by a number of steps.
	///
	/// This is just a bridge to [Node::rewind_by](crate::prelude::BoxedYarnNode::rewind_by),
	/// see that method's documentation for more information.
	pub fn rewind_by(self, steps: usize) -> Result<YarnCompass<'a, TStorage, TCmd, ID>, RewindError<TCmd>> {
		return self.node.rewind_by(steps);
	}

	/// Restarts the dialogue scene.
	///
	/// This is just a bridge to [Node::restart_scene](crate::prelude::BoxedYarnNode::restart_scene),
	/// see that method's documentation for more information.
	pub fn restart_scene(self) -> YarnCompass<'a, TStorage, TCmd, ID> {
		return self.node.restart_scene();
	}
}
