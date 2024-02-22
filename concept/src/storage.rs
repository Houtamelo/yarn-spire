#![allow(unused)]

use serde::{Deserialize, Serialize};
use crate::shared_internal::*;
use crate::var_trait::YarnVar;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Storage { }

impl Storage {
	pub fn increment_visited(&mut self, title: NodeTitle) {
		todo!()
	}

	/// Provided a given variable marker type, returns a copy of its value contained in the storage. 
	/// This method is used inside coroutines to fetch variable values, using the regular syntax: `$variable_name`
	fn get_var<T: YarnVar>(&self) -> T::Return {
		return T::get(self);
	}

	/// Provided a given variable marker type, sets the value of the variable contained in the storage.
	/// This method is used inside coroutines to set variable values, using the `set command`: `<<set $variable_name = value>>`
	fn set_var<T: YarnVar>(&mut self, value: T::Return) {
		T::set(self, value);
	}

	/// Returns `true` if the node has been visited at least once.
	///
	/// - This is used to implement the original 
	/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
	/// - This will always return `false` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
	///
	/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
	fn visited(&self, node_title: &NodeTitle) -> bool {
		return self.visited_count(node_title) > 0;
	}

	/// Returns the number of times the node has been visited.
	///
	/// - This is used to implement the original 
	/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
	/// - This will always return `0` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
	///
	/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
	fn visited_count(&self, node_title: &NodeTitle) -> usize {
		todo!()
	}
}

pub enum YarnCommand {
	Set,
}