use genco::lang::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::quoting::util::Comments;

pub fn all_tokens(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_snake_case)]
		#![allow(non_camel_case_types)]
		#![allow(unused)]
		use $(&cfg.shared_qualified)::*;

		$(Comments([
			r#" - A type that provides type-safe information about a variable that can be read/written inside dialogues."#,
			r#" - You must implement this trait for every single variable that needs to be accessed/mutated inside dialogues."#,
			r#" - This may seem like boilerplate, but this allows the compiler to **ensure type safety inside Nodes**. "#,
			" It also **ensures** that you can't make typos when writing variable names or perform operations that don't make sense \n\
			  for a given variable type, such as comparing a `String` with a number.",
			r#" # Example"#,
			r#" Assuming you already have a type that implements [VariableStorage](crate::traits::VariableStorage):"#,
			r#" ```rs"#,
			r#" use yarn_spire::shared_internal::*;"#,
			r#" #[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]"#,
			r#" pub struct CustomVariableStorage { "#,
			r#" 	pub player_name: String,"#,
			r#" }"#,
			r#" // You can implement [IVar](crate::traits::IVar) for a custom variable like this:"#,
			r#" #[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]"#,
			r#" pub struct PlayerName; // This is just a marker type, it doesn't have to be the actual variable type, but it can."#,
			r#" impl IVar for PlayerName {"#,
			r#"     type Return = String;"#,
			r#"     fn get(storage: &Self::VariableStorage) -> Self::Return {"#,
			r#"         storage.player_name.clone()"#,
			r#"     }"#,
			r#"     fn set(storage: &mut Self::VariableStorage, value: Self::Return) {"#,
			r#"         storage.player_name = value;"#,
			r#"     }"#,
			r#" }"#,
			r#" // Then, inside dialogues, this is how they get used:"#,
			r#" let mut storage = CustomVariableStorage { player_name: "John" };"#,
			r#" // get"#,
			r#" // Notice that we don't pass the variable `key` or `name` as an argument (which is the case in the original YarnSpinner's `get()`),"#,
			r#" // instead, we pass the type that indicates everything we need to know about the variable at compile time."#,
			r#" if storage.get_var::<PlayerName>() == "John" { "#,
			r#"     // play some dialogue"#,
			r#" }"#,
			r#" // set"#,
			r#" // Notice that we don't pass the variable `key` or `name` as an argument (which is the case in the original YarnSpinner's `set()`),"#,
			r#" // instead, we pass the type that indicates everything we need to know about the variable at compile time."#,
			r#" storage.set_var::<PlayerName>("Jane");"#,
			r#" ```"#,
			"Although using IVar requires additional \"boilerplate\" code, \
			 it saves time and effort in the long run by preventing runtime errors.",
			r#" **This is the philosophy behind the design of this library.**"#]))
		pub trait IVar {
			type Return;
			fn get(storage: &$(&cfg.storage_direct)) -> Self::Return;
			fn set(storage: &mut $(&cfg.storage_direct), value: Self::Return);
		}
	}
}