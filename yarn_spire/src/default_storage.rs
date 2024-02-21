/// Generates a basic variable storage type and its associated YarnVar types.
/// 
/// For more information on VariableStorage and YarnVar, 
/// see [VariableStorage](crate::prelude::VariableStorage) and [YarnVar](crate::prelude::YarnVar).
/// 
/// # Example
/// ```
/// use yarn_spinner_aot::prelude::*;
///
/// default_storage! (
///	 pub struct Storage {
///		 player_name: String,
///		 player_age: u64,
/// 	 gold: i64,
/// });
///
/// 
/// let mut storage = Storage {
/// 	player_name: "John".to_string(),
/// 	player_age: 20,
/// 	gold: 100,
/// };
/// 
/// assert_eq!(storage.get_var::<player_name>(), "John");
/// assert_eq!(storage.get_var::<player_age>(), 20);
/// assert_eq!(storage.get_var::<gold>(), 100);
/// ```
/// 
/// This macro generates the following code:
/// 
/// ```
/// use serde::{Serialize, Deserialize};
/// use yarn_spinner_aot::prelude::*;
/// 
/// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// #[allow(non_camel_case_types)]
/// pub struct Storage {
/// 	player_name: String,
/// 	player_age: u64,
/// 	gold: i64,
/// }
/// 
/// impl VariableStorage for Storage {}
/// 
/// #[allow(non_camel_case_types)]
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct player_name;
/// 
/// impl YarnVar for player_name {
/// 	type Return = String;
/// 	type VariableStorage = Storage;
///     
/// 	fn get(storage: &Self::VariableStorage) -> Self::Return {
/// 		return storage.player_name.clone();
/// 	}
/// 
/// 	fn set(storage: &mut Self::VariableStorage, value: Self::Return) {
/// 		storage.player_name = value;
/// 	}
/// }
/// 
/// #[allow(non_camel_case_types)]
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct player_age;
/// 
/// impl YarnVar for player_age {
/// 	type Return = u64;
/// 	type VariableStorage = Storage;
/// 
/// 	fn get(storage: &Self::VariableStorage) -> Self::Return {
/// 		return storage.player_age;
/// 	}
/// 
/// 	fn set(storage: &mut Self::VariableStorage, value: Self::Return) {
/// 		storage.player_age = value;
/// 	}
/// }
/// 
/// #[allow(non_camel_case_types)]
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct gold;
/// 
/// impl YarnVar for gold {
/// 	type Return = i64;
/// 	type VariableStorage = Storage;
/// 
/// 	fn get(storage: &Self::VariableStorage) -> Self::Return {
/// 		return storage.gold;
/// 	}
/// 
/// 	fn set(storage: &mut Self::VariableStorage, value: Self::Return) {
/// 		storage.gold = value;
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! default_storage {
    (pub struct $type_name: ident {
	    $($var_name: ident: $var_type: tt),* $(,)?
    }) => {
		use $crate::prelude::*;
	
	    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
	    #[allow(non_camel_case_types)]
	    pub struct $type_name {
		    $($var_name: $var_type),*
	    }
	    
	    impl VariableStorage for $type_name {}
	    
	    $(
	    #[allow(non_camel_case_types)]
	    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
	    pub struct $var_name;
	    
	    impl YarnVar for $var_name {
		    type Return = $var_type;
			type VariableStorage = $type_name;

			fn get(storage: &Self::VariableStorage) -> Self::Return {
			    return storage.$var_name.clone();
		    }
		    
			fn set(storage: &mut Self::VariableStorage, value: Self::Return) {
			    storage.$var_name = value;
		    }
	    } 
	    )*
    };
}

default_storage!(
	pub struct Storage {
		player_name: i32,
	}
);