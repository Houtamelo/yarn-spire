use std::fmt::Debug;
use houtamelo_utils::prelude::CountOrMore;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::compass::YarnCompass;
use crate::instruction::ChoiceOption;
use crate::node::BoxedYarnNode;
use crate::PlayerDecision;
use crate::prelude::{NodeID, VariableStorage};
use crate::rewind::RewindError;

/// # The compass provides directions to navigate a dialogue tree.
/// It's a wrapper around the coroutine that represents your dialogue scripts.
///
/// This variant is returned when the coroutine yields a `Choices` instruction.
/// 
/// It ensures that you must provide a player decision before continuing the dialogue.
/// 
/// You can do so by calling [next](crate::prelude::ChoicesCompass::next).
pub struct ChoicesCompass<'a,
                          TStorage: VariableStorage,
                          TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
                          ID: NodeID<Storage = TStorage, Command = TCmd>> {
	pub(crate) storage: Box<TStorage>,
	pub(crate) node: BoxedYarnNode<'a, TStorage, TCmd, ID>,
	pub(crate) options: CountOrMore<1, ChoiceOption>,
}

impl<'a,
     TStorage: VariableStorage,
     TCmd: Clone + PartialEq + Debug + Serialize + DeserializeOwned,
     ID: NodeID<Storage = TStorage, Command = TCmd>>
ChoicesCompass<'a, TStorage, TCmd, ID> {
	/// Advances the dialogue, returning a compass with the next yield.
	/// 
	/// `player_decision`: a zero-based index representing which choice the player picked.
	///
	/// ___
	///
	/// # Cannot Panic
	pub fn next(self, player_decision: PlayerDecision) -> YarnCompass<'a, TStorage, TCmd, ID> {
		return self.node.next((self.storage, Some(player_decision)));
	}

	/// A vector containing a list of options the player must pick from.
	/// 
	/// See [ChoiceOption](crate::prelude::ChoiceOption) for more information.
	pub fn options(&self) -> &CountOrMore<1, ChoiceOption>{
		return &self.options;
	}

	/// A iterator over the options the player must pick from. 
	/// 
	/// It is guaranteed that at least one option will be present.
	///
	/// See [ChoiceOption](crate::prelude::ChoiceOption) for more information.
	pub fn options_iter(&self) -> impl Iterator<Item = &ChoiceOption> {
		return self.options.iter();
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
	pub unsafe fn deconstruct(self) -> (Box<TStorage>, BoxedYarnNode<'a, TStorage, TCmd, ID>, CountOrMore<1, ChoiceOption>) {
		return (self.storage, self.node, self.options);
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