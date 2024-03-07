use std::collections::{HashMap, HashSet};
use anyhow::{anyhow, Result};
use genco::prelude::rust::Tokens;
use genco::quote;
use crate::config::YarnConfig;
use crate::expressions::declaration_ty::DeclarationTy;
use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::raw::speech::Speaker;
use crate::parsing::raw::var_declaration::VarDeclaration;
use crate::quoting::quotable_types::line_ids::{BuiltInCommand, IDFlatLine, IDFlow};
use crate::quoting::quotable_types::node::IDNode;
use crate::quoting::quotable_types::scope::IDScope;
use crate::quoting::util::{Comments, SeparatedItems};

fn tokens_macro_declaration() -> Tokens {
	let declaration_str =
"macro_rules! default_storage {
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
			rng: Xoshiro256PlusPlus,
			visited_counters: HashMap<NodeTitle, usize>,
		    vars: StorageVars,
		}
		
		impl $storage_name {
		    pub fn new() -> Self {
			    Self {
					rng: Xoshiro256PlusPlus::from_entropy(),
				    visited_counters: HashMap::new(),
				    vars: StorageVars {
					    $($name: <$var_ty>::from($default)),*
				    }
			    }
		    }
		    
			pub fn increment_visited(&mut self, title: NodeTitle) {
				let counter = self.visited_counters.entry(title).or_insert(0);
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
			pub fn visited(&self, node_title: NodeTitle) -> bool {
				return self.visited_count(node_title) > 0;
			}
		
			/// Returns the number of times the node has been visited.
			///
			/// - This is used to implement the original 
			/// [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.
			/// - This will always return `0` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`.
			///
			/// For more information, see [TrackingSetting](crate::traits::TrackingSetting).
			pub fn visited_count(&self, node_title: NodeTitle) -> usize {
				return *self.visited_counters.get(&node_title).unwrap_or(&0);
			}

			pub fn random(&mut self) -> f64 {
				return self.rng.gen_range(0.0..1.0);
			}
			
			pub fn random_range(&mut self, lower: f64, upper: f64) -> f64 {
				return self.rng.gen_range(lower..upper);
			}
			
			pub fn dice(&mut self, sides: usize) -> usize {
				return self.rng.gen_range(1..=sides);
			}
		}

		pub mod vars {
		    pub use super::*;
			
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
		}
    };
}";
	
	quote! {
		$declaration_str
	}
}

fn iter_insert_args_usages<'a>(root_arg: &'a YarnExpr,
                               in_exprs: &mut Vec<(&'a str, &'a YarnExpr)>) {
	root_arg
		.iter_exprs()
		.for_each(|expr|
			if let YarnExpr::GetVar(var_name) = expr {
				in_exprs.push((var_name, root_arg))
			});
}

fn insert_var_usages<'a>(scope: &'a IDScope,
                         in_exprs: &mut Vec<(&'a str, &'a YarnExpr)>,
                         in_speakers: &mut Vec<&'a str>) {
	for flow in &scope.flows {
		match flow {
			IDFlow::Flat(lines) => {
				for line in lines {
					match line {
						IDFlatLine::Speech(speech) => {
							let (_, args) = &speech.text;
							args.iter()
							    .for_each(|arg| iter_insert_args_usages(arg, in_exprs));
							
							if let Some(Speaker::Variable(var_name)) = &speech.speaker {
								in_speakers.push(var_name);
							}
						},
						IDFlatLine::CustomCommand(cmd) => {
							cmd.args
							   .iter()
							   .for_each(|arg| iter_insert_args_usages(arg, in_exprs));
						},
						IDFlatLine::BuiltInCommand(built_in) => {
							match built_in {
								BuiltInCommand::Set { 
									var_name, value, 
									op: _, line_number: _,
								} => {
									in_exprs.push((var_name, value))
								}
								BuiltInCommand::Jump { .. } => {}
								BuiltInCommand::Stop { .. } => {}
							}
						},
					}
				}
			}
			IDFlow::OptionsFork(options_fork) => {
				options_fork
					.options
					.iter()
					.for_each(|(line, scope_option)| {
						let (_, args) = &line.text;
						args.iter()
						    .for_each(|arg| iter_insert_args_usages(arg, in_exprs));
						
						if let Some(if_condition) = &line.if_condition {
							iter_insert_args_usages(if_condition, in_exprs);
						}
						
						if let Some(scope) = scope_option {
							insert_var_usages(scope, in_exprs, in_speakers);
						}
					})
			}
			IDFlow::IfBranch(if_branch) => {
				iter_insert_args_usages(&if_branch.if_.0.condition, in_exprs);
				
				if let Some(scope) = &if_branch.if_.1 {
					insert_var_usages(scope, in_exprs, in_speakers);
				}
				
				for (else_if, else_if_scope_option) in &if_branch.else_ifs {
					iter_insert_args_usages(&else_if.condition, in_exprs);
					
					if let Some(else_if_scope) = else_if_scope_option {
						insert_var_usages(else_if_scope, in_exprs, in_speakers);
					}
				}
				
				if let Some((_, Some(else_scope))) = &if_branch.else_ {
					insert_var_usages(else_scope, in_exprs, in_speakers);
				}
			}
		}
	}
}

fn all_var_usages(nodes: &[IDNode]) -> (Vec<(&str, &YarnExpr)>, Vec<&str>) {
	let mut in_exprs = vec![];
	let mut in_speakers = vec![];
	
	for node in nodes {
		for scope in &node.scopes {
			insert_var_usages(scope, &mut in_exprs, &mut in_speakers);
		}
	}
	
	return (in_exprs, in_speakers);
}

fn infer_types_from_usages(nodes: &[IDNode]) -> HashMap<&str, HashSet<DeclarationTy>> {
	let (in_exprs, in_speakers) = 
		all_var_usages(nodes);
	
	let mut inferred_vars = HashMap::new();

	in_exprs
		.into_iter()
		.filter_map(|(var_name, expr)|
			expr.infer_ty().map(|ty| (var_name, ty)))
		.for_each(|(var_name, ty)| {
			inferred_vars
				.entry(var_name)
				.or_insert_with(HashSet::new)
				.insert(ty);
		});
	
	in_speakers
		.into_iter()
		.for_each(|var_name| {
			inferred_vars
				.entry(var_name)
				.or_insert_with(HashSet::new)
				.insert(DeclarationTy::String);
		});
	
	return inferred_vars;
}

fn assemble_inferred_vars<'a>(nodes: &'a [IDNode],
                              var_declarations: &'a [VarDeclaration])
                              -> Result<HashMap<&'a str, (Option<&'a YarnExpr>, Option<DeclarationTy>)>> {
	let mut assembled_vars: HashMap<&str, (Option<&YarnExpr>, Option<DeclarationTy>)> = {
		let mut temp: HashMap<&str, &VarDeclaration> = HashMap::new();
		
		var_declarations
			.iter()
			.try_for_each(|declaration| {
				let var_name = declaration.var_name.as_str();

				match temp.get_key_value(var_name) {
					Some((_, already_declaration)) => {
						Err(anyhow!(
							"Variable `{var_name}` is declared more than once.\n\
							 First time at line nº{first_num}, with default value: `{first_expr:?}`\n\
							 Second time at line nº{second_num}, with default value: `{second_expr:?}`", 
							 first_num = already_declaration.line_number, second_num = declaration.line_number,
							 first_expr = already_declaration.default_value, second_expr = declaration.default_value))
					}
					None => {
						temp.insert(var_name, declaration);
						Ok(())
					}
				}
			})?;
		
		temp.into_iter()
			.map(|(var_name, declaration)| 
				(var_name, (Some(&declaration.default_value), declaration.infer_ty())))
			.collect()
	};
	
	let inferred_usages =
		infer_types_from_usages(nodes);
	
	for (inferred_name, possible_types) in inferred_usages {
		match assembled_vars.get(inferred_name) {
			None => {
				if possible_types.len() == 1 {
					assembled_vars.insert(inferred_name, (None, possible_types.into_iter().next()));
				} else { // if 0 or 2+, leave as None
					assembled_vars.insert(inferred_name, (None, None));
				}
			}
			Some(_) => {} // do nothing, prioritize declaration
		} 
	} 
	
	return Ok(assembled_vars);
}

fn tokens_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use std::collections::HashMap;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
		use rand::{Rng, SeedableRng};
		use rand_xoshiro::Xoshiro256PlusPlus;
	}
}

fn tokens_macro_expansion(cfg: &YarnConfig,
                          inferred_vars: HashMap<&str, (Option<&YarnExpr>, Option<DeclarationTy>)>)
                          -> Tokens {
	let var_types =
		inferred_vars
			.iter()
			.map(|(var_name, (_, inferred_ty))| {
				let ty_tokens =
					match inferred_ty {
						Some(ty) => {
							quote!($(ty))
						}
						None => {
							quote!(todo!)
						},
					};

				(var_name, ty_tokens)
			}).collect::<Vec<_>>();

	let vars_ty_tokens =
		var_types
			.iter()
			.map(|(var_name, ty_tokens)|
				quote! {
					$(**var_name): $ty_tokens
				});

	let vars_default_value_tokens =
		inferred_vars
			.iter()
			.map(|(var_name, (default_value, _))| {
				let default_value_tokens =
					match default_value {
						None => quote!(todo!()),
						Some(expr) => quote!($(*expr))
					};

				quote! {
						$(*var_name): { $default_value_tokens }
					}
			});

	let vars_trait_tokens =
		var_types
			.iter()
			.map(|(var_name, ty_tokens)| {
				let var_name = **var_name;

				quote! {
					pub struct $(var_name);
			
					impl YarnVar for $(var_name) {
						type Return = $ty_tokens;
				
						fn get(storage: &$(&cfg.storage_direct)) -> Self::Return {
							return storage.vars.$(var_name).clone();
						}
				
						fn set(storage: &mut $(&cfg.storage_direct), value: Self::Return) {
							storage.vars.$(var_name) = value;
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
			rng: Xoshiro256PlusPlus,
			visited_counters: HashMap<NodeTitle, usize>,
			vars: StorageVars,
		}
	
		impl $(&cfg.storage_direct) {
			pub fn new() -> Self {
				Self {
					rng: Xoshiro256PlusPlus::from_entropy(),
					visited_counters: HashMap::new(),
					vars: StorageVars {
						$(SeparatedItems(vars_default_value_tokens, ",\n"))
					}
				}
			}

			pub fn increment_visited(&mut self, title: NodeTitle) {
				let counter = self.visited_counters.entry(title).or_insert(0);
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
				"- This is used to implement the original \n\
				 [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.",
				r#"- This will always return `false` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`."#,
				r#"For more information, see [TrackingSetting](crate::traits::TrackingSetting)."#,
			]))
			pub fn visited(&self, node_title: NodeTitle) -> bool {
				return self.visited_count(node_title) > 0;
			}
	
			$(Comments([
				r#"Returns the number of times the node has been visited."#,
				"- This is used to implement the original \n\
				 [tracking](https://docs.yarnspinner.dev/getting-started/writing-in-yarn/tags-metadata#the-tracking-header) setting.",
				r#"- This will always return `0` if [TRACKING](crate::traits::NodeID::TRACKING) is `Some(Never)`."#,
				r#"For more information, see [TrackingSetting](crate::traits::TrackingSetting)."#,
			]))
			pub fn visited_count(&self, node_title: NodeTitle) -> usize {
				return *self.visited_counters.get(&node_title).unwrap_or(&0);
			}
			
			pub fn random(&mut self) -> f64 {
				return self.rng.gen_range(0.0..1.0);
			}
			
			pub fn random_range(&mut self, lower: f64, upper: f64) -> f64 {
				return self.rng.gen_range(lower..upper);
			}
			
			pub fn dice(&mut self, sides: usize) -> usize {
				return self.rng.gen_range(1..=sides);
			}
		}
		
		pub mod vars {
			pub use super::*;
			
			$(SeparatedItems(vars_trait_tokens, "\n"))
		}
	}
}

pub fn all_tokens(cfg: &YarnConfig,
                  nodes: &[IDNode],
                  var_declarations: &[VarDeclaration])
                  -> Result<Tokens> {
	let imports =
		tokens_imports(cfg);
	let macro_declaration =
		tokens_macro_declaration();
	
	let inferred_vars = 
		assemble_inferred_vars(nodes, var_declarations)?;
	let macro_expansion =
		tokens_macro_expansion(cfg, inferred_vars);
	
	Ok(quote! {
		$imports
		
		$macro_declaration
		
		$macro_expansion
	})
}
