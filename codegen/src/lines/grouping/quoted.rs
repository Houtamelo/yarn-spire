use parsel::try_parse_quote;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Expr, Lit};
use crate::expressions::yarn_expr::YarnExpr;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use anyhow::{anyhow, Result};
use crate::expressions::yarn_lit::YarnLit;
use crate::lines::grouping::choices::Choices;
use crate::lines::grouping::if_branch::IfBranch;
use crate::lines::grouping::scope::{FlatLine, Flow, YarnScope};
use crate::lines::raw::branches::choices::ChoiceOption;
use crate::lines::raw::command::YarnCommand;
use crate::lines::raw::node_metadata::TrackingSetting;
use crate::lines::raw::speech::Speaker;
use crate::lines::YarnNode;

pub trait Quoted {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident) 
		-> Result<TokenStream>;
}

impl Quoted for YarnLit {
	fn quoted(&self, _: &Ident, _: &Ident) -> Result<TokenStream> {
		return Ok(match self {
			YarnLit::Int(i) => {
				let lit = Literal::i64_unsuffixed(*i);
				quote!(#lit)
			},
			YarnLit::Float(f) => {
				let lit = Literal::f64_unsuffixed(*f);
				quote!(#lit)
			},
			YarnLit::Str(s) => {
				let lit = Literal::string(&s);
				quote!(#lit.into())
			},
			YarnLit::Bool(b) => {
				quote!(#b)
			}
		});
	}
}

impl Quoted for YarnUnaryOp {
	fn quoted(&self, _: &Ident, _: &Ident) -> Result<TokenStream> {
		return match self {
			YarnUnaryOp::Not => Ok(quote! { ! }),
			YarnUnaryOp::Negate => Ok(quote! { - }),
		};
	}
}

impl Quoted for YarnBinaryOp {
	fn quoted(&self, _: &Ident, _: &Ident) -> Result<TokenStream> {
		return Ok(match self {
			YarnBinaryOp::Add => quote! { + },
			YarnBinaryOp::Sub => quote! { - },
			YarnBinaryOp::Mul => quote! { * },
			YarnBinaryOp::Div => quote! { / },
			YarnBinaryOp::Rem => quote! { % },
			YarnBinaryOp::And => quote! { && },
			YarnBinaryOp::Or => quote! { || },
			YarnBinaryOp::Eq => quote! { == },
			YarnBinaryOp::Ne => quote! { != },
			YarnBinaryOp::Lt => quote! { < },
			YarnBinaryOp::Le => quote! { <= },
			YarnBinaryOp::Gt => quote! { > },
			YarnBinaryOp::Ge => quote! { >= },
		});
	}
}

impl Quoted for YarnExpr {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		return match self {
			YarnExpr::Lit(literal) =>
				literal.quoted(storage_ident, command_ident),
			YarnExpr::VarGet(var_name) => {
				let var_name_tokens =
					Ident::new(&var_name, Span::call_site());
				
				Ok(quote!(storage.get_var::< #var_name_tokens >()))
			},
			YarnExpr::Parenthesis(inner_expr) => {
				let inner_expr = 
					inner_expr.quoted(storage_ident, command_ident)?;
				
				Ok(quote!((#inner_expr)))
			},
			YarnExpr::UnaryOp { yarn_op, right } => {
				let yarn_op = 
					yarn_op.quoted(storage_ident, command_ident)?;
				
				let right = 
					right.quoted(storage_ident, command_ident)?;
				
				Ok(quote!(#yarn_op (#right) ))
			},
			YarnExpr::BinaryOp { yarn_op, left, right } => {
				let yarn_op =
					yarn_op.quoted(storage_ident, command_ident)?;
				
				let left = 
					left.quoted(storage_ident, command_ident)?;
				
				let right =
					right.quoted(storage_ident, command_ident)?;
				
				Ok(quote!(#left #yarn_op #right))
			},
			YarnExpr::FunctionCall { func_name, args } => {
				let func_name_tokens = 
					Ident::new(&func_name, Span::call_site());
				
				let args_tokens =
					args.iter()
						.map(|arg| 
							arg.quoted(storage_ident, command_ident)
								.map_err(|err| anyhow!(
									"Could not quote `FunctionCall`: failed to quote an argument.\n\
									 Error: `{err}`\n\
									 Argument: `{arg:?}`\n\
									 Function Name: `{func_name}`\
									 \n\n\
									 Help: This usually happens when the argument is not a valid expression.",
								))
						).collect::<Result<Vec<TokenStream>>>()?;
				
				Ok(quote!(storage. #func_name_tokens(#(#args_tokens),*)))
			},
		};
	}
}

impl Quoted for ChoiceOption {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		let text_tokens = {
			let (literal, args) = &self.line;
			
			let literal_tokens = Literal::string(&literal);
			
			let args_tokens =
				args.iter()
				    .map(|arg| 
					    arg.quoted(storage_ident, command_ident)
						    .map_err(|err| anyhow!(
							    "Could not quote an argument in `ChoiceOption`'s text.\n\
								 Error: `{err}`\n\
								 Argument: `{arg:?}`\n\
							     Choice data: \n\
							     \t{self:?}\
							     \n\n\
							     Help: This usually happens when the argument is not a valid expression."
						    ))
				    ).collect::<Result<Vec<TokenStream>>>()?;
			
			if args_tokens.is_empty() {
				quote!(#literal_tokens.to_string())
			} else {
				quote!(std::format!(#literal_tokens, #(#args_tokens),* ))
			}
		};
		
		let if_condition_tokens = 
			match &self.if_condition {
				Some(expr) => {
					let expr_tokens = 
						expr.quoted(storage_ident, command_ident)
							.map_err(|err| anyhow!(
							    "Could not quote `if condition`'s expression in `ChoiceOption`.\n\
								 Error: `{err}`\n\
							     Argument: `{expr:?}`\n\
								 Choice data:\n\
								 \t{self:?}\n\
							     \n\n\
							     Help: This usually happens when the argument is not a valid expression."
						    ))?;
					quote!(Some(#expr_tokens))
				},
				None => {
					quote!(None)
				},
			};
		
		let metadata_tokens
			= match &self.metadata {
				Some(metadata) => {
					let literal_tokens = Literal::string(&metadata);
					quote!(Some(#literal_tokens))
				},
				None => {
					quote!(None)
				},
			};
		
		Ok(quote! {
			ChoiceOption {
				text: #text_tokens,
				is_available: #if_condition_tokens,
				metadata: #metadata_tokens,
			}
		})
	}
}

impl Quoted for FlatLine {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		return match self {
			FlatLine::Speech(speech) => {
				let speaker_tokens =
					match &speech.speaker {
						Some(Speaker::Literal(literal)) => {
							let literal_tokens = Literal::string(&literal);
							quote!(Some(#literal_tokens.to_string()))
						},
						Some(Speaker::Variable(var_name)) => {
							quote!(Some(storage.get_var::< #var_name >()))
						},
						None => {
							quote!(None)
						},
					};

				let text_tokens = {
					let (literal, args) = &speech.line;
					
					let literal_tokens = Literal::string(&literal);
					
					let args_tokens =
						args.iter()
						    .map(|arg| 
							    arg.quoted(storage_ident, command_ident)
								    .map_err(|err| anyhow!(
									    "Could not quote an argument in `Speech`.\n\
									     Error: `{err}`\n\
									     Argument: `{arg:?}`\n\
									     Speech data: \n\
									     \t{speech:?}\
									     \n\n\
									     Help: This usually happens when the argument is not a valid expression."
								    )))
						    .collect::<Result<Vec<TokenStream>>>()?;
					
					if args_tokens.is_empty() {
						quote!(#literal_tokens.to_string())
					} else {
						quote!(format!(#literal_tokens, #(#args_tokens),* ))
					}
				};

				let metadata_tokens =
					match &speech.metadata {
						Some(metadata) => {
							let literal_tokens = Literal::string(&metadata);
							quote!(Some(#literal_tokens.to_string()))
						}
						None => {
							quote!(None)
						}
					};

				Ok(quote! {
					let speech = Speech {
						speaker: #speaker_tokens,
						text: #text_tokens,
						metadata: #metadata_tokens,
					};
					
					storage = yielder.suspend((storage, YarnInstruction::Speech(speech))).0;
				})
			},
			FlatLine::Command(command) => {
				match command {
					YarnCommand::Set { line_number, var_name, arg } => {
						let var_name_ident = 
							Ident::new(&var_name, Span::call_site());
						
						let arg_tokens = 
							arg.quoted(storage_ident, command_ident)
							   .map_err(|err| anyhow!(
									"Could not quote the argument in a `set` command.\n\
									 Error: `{err}`\n\
									 At line nº{line_number}\n\
									 Variable Name: `{var_name}`\n\
									 Argument expression: `{arg:?}`\
									 \n\n\
									 Help: This usually happens when the argument is not a valid expression."
							   ))?;
						
						Ok(quote! {
							storage.set_var::< #var_name_ident >( #arg_tokens );
						})
					},
					YarnCommand::Other { line_number, variant, args } => {
						let variant_ident =
							Ident::new(&variant, Span::call_site());

						let args_tokens =
							args.iter()
								.map(|arg| 
									arg.quoted(storage_ident, command_ident)
										.map_err(|err| anyhow!(
											"Could not quote an argument in a `YarnCommand`.\n\
											 Error: `{err}`\n\
											 At line nº{line_number}\n\
											 Command Variant: `{variant}`\n\
											 Argument expression: `{arg:?}`\n\
											 All arguments: `{args:?}`\
											 \n\n\
											 Help: This usually happens when the argument is not a valid expression."
										))
								).collect::<Result<Vec<TokenStream>>>()?;

						if args_tokens.is_empty() {
							Ok(quote! {
								let command = #command_ident::#variant_ident;
								storage = yielder.suspend((storage, YarnInstruction::Command(command))).0;
							})
						} else {
							Ok(quote! {
								let command = #command_ident::#variant_ident(#(#args_tokens),*);
								storage = yielder.suspend((storage, YarnInstruction::Command(command))).0;
							})
						}
					},
				}
			},
		};
	}
}

impl Quoted for YarnScope {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		let flow_tokens = 
			self.iter_flows()
				.map(|flow| flow.quoted(storage_ident, command_ident))
				.collect::<Result<Vec<TokenStream>>>()?;
		
		Ok(quote! {
			#(#flow_tokens)*
		})
	}
}

impl Quoted for Option<Box<YarnScope>> {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		return match self {
			Some(scope) => scope.quoted(storage_ident, command_ident),
			None => Ok(TokenStream::new()),
		};
	}
}

impl Quoted for Vec<FlatLine> {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		let tokens =
			self.into_iter()
			    .map(|flat_line| flat_line.quoted(storage_ident, command_ident))
			    .collect::<Result<Vec<TokenStream>>>()?;

		Ok(quote! {
			#(#tokens)*
		})
	}
}

impl Quoted for Choices {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		let (first_option_wrapped, other_options) =
			self.iter_options();
		
		let (first_option_tokens, first_option_scope_tokens) = {
			let (first_option, first_scope) = first_option_wrapped;

			let scope_inner_tokens = first_scope.quoted(storage_ident, command_ident)?;
			
			let scope_tokens =
				quote! {
					Some(0) => {
						#scope_inner_tokens
					},
				};
			
			let option_tokens = first_option.quoted(storage_ident, command_ident)?;
			(option_tokens, scope_tokens)
		};
		
		let (other_options, scopes): (Vec<&ChoiceOption>, Vec<&Option<Box<YarnScope>>>) =
			other_options.map(|tuple| (&tuple.0, &tuple.1)).unzip();
		
		let other_options_tokens =
			other_options
				.into_iter()
				.map(|option| option.quoted(storage_ident, command_ident))
				.collect::<Result<Vec<TokenStream>>>()?;

		let enumerated_scopes =
			scopes
				.into_iter()
				.enumerate()
				.map(|(index, scope_option)|(index + 1, scope_option))
				.map(|(index, scope_option)| {
					let scope_inner_tokens = scope_option.quoted(storage_ident, command_ident)?;

					let index_literal = Literal::usize_unsuffixed(index);

					Ok(quote! {
						  Some(#index_literal) => {
							  #scope_inner_tokens
						  },
					  })
				}).collect::<Result<Vec<TokenStream>>>()?;

		Ok(quote! {
			let options = houtamelo_utils::prelude::CountOrMore::new(
				[#first_option_tokens],
				vec![#(#other_options_tokens),*]);
			
			(storage, player_decision) = yielder.suspend((storage, YarnInstruction::Choices(options)));
			
			match player_decision {
				Some(0) => {
					#first_option_scope_tokens
				},
				
				#(#enumerated_scopes)*
				
				_ => {
					let options_provided =
						houtamelo_utils::prelude::CountOrMore::new(
						[#first_option_tokens],
						vec![#(#other_options_tokens),*]);
					
					return match player_decision {
						Some(invalid) => (storage,
							Err(RuntimeError::InvalidPlayerDecision {
								options_provided,
								got: Some(invalid),
							})),
						None => (storage,
							Err(RuntimeError::ExpectedPlayerDecision {
								options_provided,
							})
						),
					};
				}
			}
		})
	}
}

impl Quoted for IfBranch {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		let if_tokens = {
			let (if_struct, scope) = &self.if_;
			
			let condition_tokens = 
				if_struct.condition.quoted(storage_ident, command_ident)?;
			
			let scope_tokens = 
				scope.quoted(storage_ident, command_ident)?;
			
			quote! {
				if #condition_tokens {
					#scope_tokens
				}
			}
		};
		
		let else_ifs_tokens =
			self.else_ifs
			    .iter()
			    .map(|(else_if, scope)| {
				    let condition_tokens = 
					    else_if.condition.quoted(storage_ident, command_ident)?;
				    
				    let scope_tokens = 
					    scope.quoted(storage_ident, command_ident)?;
				    
				    Ok(quote! {
					    else if #condition_tokens {
						    #scope_tokens
					    }
				    })
			    }).collect::<Result<Vec<TokenStream>>>()?;
		
		let else_tokens = {
			match &self.else_ {
				Some((_else_struct, scope)) => {
					let scope_tokens = scope.quoted(storage_ident, command_ident)?;
					quote! {
						else {
							#scope_tokens
						}
					}
				},
				None => TokenStream::new(),
			}
		};
		
		Ok(quote! {
			#if_tokens
			#(#else_ifs_tokens)*
			#else_tokens
		})
	}
}

impl Quoted for Flow {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		return match self {
			Flow::Flat(flat_lines) => flat_lines.quoted(storage_ident, command_ident),
			Flow::Choices(choices) => choices.quoted(storage_ident, command_ident),
			Flow::IfBranch(branch) => branch.quoted(storage_ident, command_ident),
		};
	}
}

impl Quoted for TrackingSetting {
	fn quoted(&self, _: &Ident, _: &Ident)
	          -> Result<TokenStream> {
		return match self {
			TrackingSetting::Always => {
				let ident = Ident::new("TrackingSetting::Always", Span::call_site());
				Ok(quote!(#ident))
			}
			TrackingSetting::Never => {
				let ident = Ident::new("TrackingSetting::Never", Span::call_site());
				Ok(quote!(#ident))
			}
		};
	}
}

impl Quoted for YarnNode {
	fn quoted(&self, storage_ident: &Ident, command_ident: &Ident)
	          -> Result<TokenStream> {
		let scene_id = {
			let ident = Ident::new(&self.metadata.title, Span::call_site());
			quote! { #ident }
		};
		
		let tags_tokens = {
			let tags = 
				self.metadata
					.tags
					.iter()
					.map(|tag| Literal::string(tag));
			
			quote! {
				&[
					#(#tags),*
				]
			}
		};
		
		let tracking_setting_tokens =
			match self.metadata.tracking {
				Some(tracking) => {
					let tracking_tokens = 
						tracking.quoted(storage_ident, command_ident)?;
					
					quote!(Some(#tracking_tokens))
				},
				None => {
					quote!(None)
				},
			};
		
		let customs_tokens = {
			let customs = 
				self.metadata
					.customs
					.iter()
					.map(|custom| Literal::string(&custom));
			
			quote! {
				&[
					#(#customs),*
				]
			}
		};
		
		let contents = 
			self.contents
			    .iter()
			    .map(|scope| scope.quoted(storage_ident, command_ident))
			    .collect::<Result<Vec<TokenStream>>>()?;

		Ok(quote! {
			#[derive(std::fmt::Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
			pub struct #scene_id;
			
			impl yarn_spinner_aot::prelude::NodeID for #scene_id {
				type VariableStorage = #storage_ident;
				type Command = #command_ident;
				
				const TAGS: &[&'static str] = #tags_tokens;
				const TRACKING: Option<yarn_spinner_aot::prelude::TrackingSetting> = #tracking_setting_tokens;
				const CUSTOMS: &[&'static str] = #customs_tokens;
			
				fn play<'a>(original_storage: Box<Self::VariableStorage>) 
							-> YarnCompass<'a, Self::VariableStorage, Self::Command, Self> {
					use yarn_spinner_aot::prelude::*;
					
					let coroutine = corosensei::ScopedCoroutine::new(|yielder, 
						(mut storage, mut _player_decision): (Box<Self::VariableStorage>, Option<yarn_spinner_aot::prelude::PlayerDecision>)| {
						#(#contents)*
							
						return (storage, Ok(()));
					});
					
					let yarn_scene = BoxedYarnNode::new(#scene_id, &original_storage, coroutine);
					return yarn_scene.next((original_storage, None));
				}
			}
		})
	}
}