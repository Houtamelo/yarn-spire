#![allow(unused)]
#![allow(non_camel_case_types)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::shared_internal::*;
use crate::var_trait::YarnVar;

macro_rules! default_storage {
    (pub struct $storage_name: ident {
	    vars: {
		    $($name:ident: $var_ty:ty = $default: expr),*
		    $(,)?
	    }
	}) => {
	    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
	    struct StorageVars {
		    $($name: $var_ty),*
	    }
	    
	    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
		pub struct $storage_name {
			visited_counters: HashMap<NodeTitle, usize>,
		    vars: StorageVars,
		}
		
		impl $storage_name {
		    pub fn new() -> Self {
			    Self {
				    visited_counters: HashMap::new(),
				    vars: StorageVars {
					    $($name: <$var_ty>::from($default)),*
				    }
			    }
		    }
		    
			pub fn increment_visited(&mut self, title: &NodeTitle) {
				let counter = self.visited_counters.entry(title.clone()).or_insert(0);
				*counter += 1;
			}
		
			/// Provided a given variable marker type, returns a copy of its value contained in the storage. 
			/// This method is used inside coroutines to fetch variable values, using the regular syntax: `$variable_name`
			pub fn get_var<T: YarnVar>(&self) -> T::Return {
				return T::get(self);
			}
		
			/// Provided a given variable marker type, sets the value of the variable contained in the storage.
			/// This method is used inside coroutines to set variable values, using the `set command`: `<<set $variable_name = value>>`
			pub fn set_var<T: YarnVar>(&mut self, value: T::Return) {
				T::set(self, value);
			}
		
			/// Returns `true` if the node has been visited at least once.
			///
			/// - This is used to implement the original 
			/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
			/// - This will always return `false` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
			///
			/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
			pub fn visited(&self, node_title: &NodeTitle) -> bool {
				return self.visited_count(node_title) > 0;
			}
		
			/// Returns the number of times the node has been visited.
			///
			/// - This is used to implement the original 
			/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
			/// - This will always return `0` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
			///
			/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
			pub fn visited_count(&self, node_title: &NodeTitle) -> usize {
				return *self.visited_counters.get(node_title).unwrap_or(&0);
			}
		}
	    
	    
	    $(
		    pub struct $name;
			
			impl YarnVar for $name {
				type Return = $var_ty;
			
				fn get(storage: &$storage_name) -> Self::Return {
					return storage.vars.$name.clone();
				}
			
				fn set(storage: &mut $storage_name, value: Self::Return) {
					storage.vars.$name = value;
				}
			}
	    )*
    };
}

default_storage!(
	pub struct Storage {
		vars: {
			narrator: String = "Narrator",
			mouth_taste: String = "Metallic", 
			ethel_awake: bool = true,
			ethel_stamina: isize = 100_isize,
		}
	}
);

pub enum YarnCommand {
	fade_in(f64),
	fade_out(f64),
	sfx_wait(String),
	cg(String),
}