use genco::prelude::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::quoting::helper::{Comments, SeparatedItems};

fn tokens_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::collections::HashMap;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
	}
}

fn tokens_macro_declaration() -> Tokens {
	let declaration_str = "
		macro_rules! default_storage {
		    (pub struct $storage_name: ident { 
		        vars: {
		            $($name:ident: $var_ty:ty = $default: expr),*
		            $(,)?
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
		}";
	
	quote! {
		$declaration_str
	}
}

fn tokens_macro_expansion(cfg: &YarnConfig,
                          var_declarations: &[VarDeclaration])
                          -> Tokens {
	let var_types = 
		var_declarations
			.iter()
			.map(|declaration| {
				let ty_tokens =
					match declaration.infer_ty() {
						Some(ty) => {
							quote!($(&ty))
						}
						None => {
							quote!(todo!)
						},
					};

				(declaration, ty_tokens)
			}).collect::<Vec<_>>();
	
	let vars_ty_tokens = 
		var_types
			.iter()
			.map(|(declaration, ty_tokens)| 
				quote! {
					$(&declaration.var_name): $ty_tokens
				});
	
	let vars_default_value_tokens =
		var_declarations
			.iter()
			.map(|declaration| 
				quote! {
					$(&declaration.var_name): { $(&declaration.default_value) }
				});
	
	let vars_trait_tokens =
		var_types
			.iter()
			.map(|(declaration, ty_tokens)| {
				let var_name = &declaration.var_name;
				
				quote! {
					pub struct $(var_name);
			
					impl YarnVar for $(var_name) {
						type Return = $ty_tokens;
				
						fn get(storage: &$(&cfg.storage_direct)) -> Self::Return {
							return storage.vars.$var_name.clone();
						}
				
						fn set(storage: &mut $(&cfg.storage_direct), value: Self::Return) {
							storage.vars.$var_name = value;
						}
					}
				}
			}); 
	
	quote! {
		#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
		struct StorageVars {
			$(SeparatedItems(vars_ty_tokens, ",\n"))
		}
	
		#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
		pub struct $(&cfg.storage_direct) {
			visited_counters: HashMap<NodeTitle, usize>,
			vars: StorageVars,
		}
	
		impl $(&cfg.storage_direct) {
			pub fn new() -> Self {
				Self {
					visited_counters: HashMap::new(),
					vars: StorageVars {
						$(SeparatedItems(vars_default_value_tokens, ",\n"))
					}
				}
			}

			pub fn increment_visited(&mut self, title: &NodeTitle) {
				let counter = self.visited_counters.entry(title.clone()).or_insert(0);
				*counter += 1;
			}

			$(Comments([
				r#"Provided a given variable marker type, returns a copy of its value contained in the storage."#, 
				r#"This method is used inside coroutines to fetch variable values, using the regular syntax: `$variable_name`"#,
			]))
			pub fn get_var<T: YarnVar>(&self) -> T::Return {
				return T::get(self);
			}

			$(Comments([
				r#"Provided a given variable marker type, sets the value of the variable contained in the storage."#,
				r#"This method is used inside coroutines to set variable values, using the `set command`: `<<set $variable_name = value>>`"#,
			]))
			pub fn set_var<T: YarnVar>(&mut self, value: T::Return) {
				T::set(self, value);
			}
	
			$(Comments([
				r#"Returns `true` if the node has been visited at least once."#,
				r#"- This is used to implement the original 
				[tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting."#,
				r#"- This will always return `false` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`."#,
				r#"For more information, see [TrackingSetting](crate::traits::TrackingSetting)."#,
			]))
			pub fn visited(&self, node_title: &NodeTitle) -> bool {
				return self.visited_count(node_title) > 0;
			}
	
			$(Comments([
				r#"Returns the number of times the node has been visited."#,
				r#"- This is used to implement the original 
				[tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting."#,
				r#"- This will always return `0` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`."#,
				r#"For more information, see [TrackingSetting](crate::traits::TrackingSetting)."#,
			]))
			pub fn visited_count(&self, node_title: &NodeTitle) -> usize {
				return *self.visited_counters.get(node_title).unwrap_or(&0);
			}
		
			$(SeparatedItems(vars_trait_tokens, "\n"))
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  var_declarations: &[VarDeclaration])
                  -> Tokens {
	let imports =
		tokens_imports(cfg);
	let macro_declaration =
		tokens_macro_declaration();
	let macro_expansion =
		tokens_macro_expansion(cfg, var_declarations);
	
	quote! {
		$imports
		
		$macro_declaration
		
		$macro_expansion
	}
}