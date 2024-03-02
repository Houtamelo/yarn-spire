use genco::prelude::{FormatInto, Rust};
use genco::{quote_in, Tokens};
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};
use anyhow::{anyhow, Result};
use crate::expressions::built_in_calls::BuiltInFunctionCall;
use crate::expressions::declaration_ty::DeclarationTy;
use crate::quoting::util::SeparatedItems;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YarnExpr {
	Lit(YarnLit),
	GetVar(String),
	Parenthesis(Box<YarnExpr>),
	UnaryOp {
		yarn_op: YarnUnaryOp,
		right: Box<YarnExpr>,
	},
	BinaryOp {
		yarn_op: YarnBinaryOp,
		left: Box<YarnExpr>,
		right: Box<YarnExpr>,
	},
	CustomFunctionCall {
		func_name: String,
		args: Vec<YarnExpr>,
	},
	BuiltInFunctionCall(BuiltInFunctionCall),
	Identifier(String),
	Cast {
		cast_ty: DeclarationTy,
		expr: Box<YarnExpr>,
	},
}

impl YarnExpr {
	pub fn parse_call(func_name: String, mut args: Vec<YarnExpr>) -> Result<Self> {
		macro_rules! one_arg_or_bail {
		    () => {
			    if args.len() == 1 {
				    args.pop().unwrap()
			    } else {
					return Err(anyhow!(
						"`{}` function takes exactly one argument, but got {}.\n\
						 Arguments: {args:?}", func_name, args.len()
					));
				}
		    };
		}
		
		return match func_name.as_str() {
			"get_var" => {
				let arg = one_arg_or_bail!();
				match arg {
					YarnExpr::Identifier(ident) =>
						Ok(YarnExpr::GetVar(ident)),
					invalid =>
						Err(anyhow!(
							"`get_var` function call has one argument but it's not a `Identifier`\n\
							 Argument: {invalid:?}")),
				}
			},
			"visited" => {
				let arg = one_arg_or_bail!();
				
				if let YarnExpr::Lit(YarnLit::Str(node_name)) | YarnExpr::Identifier(node_name) = arg {
					Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Visited(node_name)))
				} else {
					Err(anyhow!(
					"`visited` function takes only a string or identifier argument.\n\
					 Got: {arg:?}"))
				}
			},
			"visited_count" => {
				let arg = one_arg_or_bail!();
				
				if let YarnExpr::Lit(YarnLit::Str(node_name)) | YarnExpr::Identifier(node_name) = arg {
					Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::VisitedCount(node_name)))
				} else {
					Err(anyhow!(
					"`visited_count` function only takes a string or identifier argument.\n\
					 Got: {arg:?}"))
				}
			},
			"format_invariant" => {
				let arg = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::FormatInvariant(Box::new(arg))))
			},
			"random" => {
				if args.is_empty() {
					Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Random))
				} else {
					Err(anyhow!(
						"`random` function takes no arguments, but got {}.\n\
						 Arguments: {args:?}", args.len()))
				}
			},
			"random_range" => {
				if args.len() == 2 {
					let max = args.pop().unwrap();
					let min = args.pop().unwrap();
					Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::RandomRange(Box::new(min), Box::new(max))))
				} else {
					Err(anyhow!(
						"`random_range` function takes exactly two arguments, but got {}.\n\
						 Arguments: {args:?}", args.len()))
				}
			},
			"dice" => {
				let sides = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Dice(Box::new(sides))))
			},
			"round" => {
				let num = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Round(Box::new(num))))
			},
			"round_places" => {
				if args.len() == 2 {
					let places = args.pop().unwrap();
					let num = args.pop().unwrap();
					Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::RoundPlaces(Box::new(num), Box::new(places))))
				} else {
					Err(anyhow!(
						"`round_places` function takes exactly two arguments, but got {}.\n\
						 Arguments: {args:?}", args.len()))
				}
			},
			"floor" => {
				let num = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Floor(Box::new(num))))
			},
			"ceil" => {
				let num = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Ceil(Box::new(num))))
			},
			"inc" => {
				let num = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Inc(Box::new(num))))
			},
			"dec" => {
				let num = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Dec(Box::new(num))))
			},
			"decimal" => {
				let num = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Decimal(Box::new(num))))
			},
			"int" => {
				let num = one_arg_or_bail!();
				Ok(YarnExpr::BuiltInFunctionCall(BuiltInFunctionCall::Int(Box::new(num))))
			},
			_ => Ok(YarnExpr::CustomFunctionCall {
				func_name,
				args,
			})
		};
	}
	
	pub fn iter_exprs(&self) -> impl Iterator<Item = &YarnExpr> {
		return ExprIter::from_expr(self);
	}

	pub fn infer_ty(&self) -> Option<DeclarationTy> {
		return match self {
			YarnExpr::Lit(lit) => {
				match lit {
					YarnLit::Int(_) => Some(DeclarationTy::isize),
					YarnLit::Float(_) => Some(DeclarationTy::f64),
					YarnLit::Str(_) => Some(DeclarationTy::String),
					YarnLit::Bool(_) => Some(DeclarationTy::bool),
				}
			}
			| YarnExpr::Parenthesis(inner_expr)
			| YarnExpr::UnaryOp { right: inner_expr, .. } =>
				inner_expr.infer_ty(),
			YarnExpr::BinaryOp { left, right, .. } =>
				left.infer_ty().or_else(|| right.infer_ty()),
			YarnExpr::BuiltInFunctionCall(built_in_call) => {
				match built_in_call {
					| BuiltInFunctionCall::Visited(_) =>
						Some(DeclarationTy::bool),
					| BuiltInFunctionCall::FormatInvariant(_) =>
						Some(DeclarationTy::String),
					BuiltInFunctionCall::Random =>
						Some(DeclarationTy::f64),
					BuiltInFunctionCall::RandomRange(lower, upper) =>
						lower.infer_ty().or_else(|| upper.infer_ty()),
					| BuiltInFunctionCall::VisitedCount(_)
					| BuiltInFunctionCall::Dice(_)
					| BuiltInFunctionCall::Round(_)
					| BuiltInFunctionCall::RoundPlaces(_, _)
					| BuiltInFunctionCall::Floor(_)
					| BuiltInFunctionCall::Ceil(_)
					| BuiltInFunctionCall::Inc(_)
					| BuiltInFunctionCall::Dec(_)
					| BuiltInFunctionCall::Int(_) =>
						Some(DeclarationTy::isize),
					BuiltInFunctionCall::Decimal(_) =>
						Some(DeclarationTy::f64),
				}
			}
			YarnExpr::Cast { cast_ty, .. } =>
				Some(*cast_ty),
			| YarnExpr::Identifier(_)
			| YarnExpr::CustomFunctionCall {..}
			| YarnExpr::GetVar(_) => {
				None
			}
		};
	}
}

impl FormatInto<Rust> for &YarnExpr {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		return match self {
			YarnExpr::Lit(literal) => {
				literal.format_into(tokens);
			},
			YarnExpr::GetVar(var_name) => {
				quote_in!(*tokens => storage.get_var::<$var_name>());
			},
			YarnExpr::Parenthesis(inner_expr) => {
				quote_in!(*tokens => ($(inner_expr.as_ref())))
			},
			YarnExpr::UnaryOp { yarn_op: unary_op, right } => {
				quote_in!(*tokens => $unary_op ($(right.as_ref())))
			},
			YarnExpr::BinaryOp { yarn_op, left, right } => {
				quote_in!(*tokens => $(left.as_ref()) $yarn_op $(right.as_ref()))
			},
			YarnExpr::CustomFunctionCall { func_name, args } => {
				quote_in!(*tokens => storage.$func_name($(SeparatedItems(args, ", "))) );
			},
			YarnExpr::BuiltInFunctionCall(built_in_call) => {
				built_in_call.format_into(tokens);
			},
			YarnExpr::Identifier(ident_str) => {
				quote_in!(*tokens => $ident_str);
			}
			YarnExpr::Cast { cast_ty, expr } => {
				quote_in!( *tokens => $(expr.as_ref()) as $cast_ty );
			}
		};
	}
}

struct ExprIter<'a> {
	exprs: Vec<&'a YarnExpr>,
}

impl<'a> ExprIter<'a> {
	fn from_expr(expr: &'a YarnExpr) -> Self {
		let mut exprs = vec![];
		Self::fill_exprs(&mut exprs, expr);
		return Self { exprs };
	}
	
	fn fill_exprs(fill_me: &mut Vec<&'a YarnExpr>, expr: &'a YarnExpr) {
		fill_me.push(expr);
		
		match expr {
			| YarnExpr::Parenthesis(expr) 
			| YarnExpr::UnaryOp { right: expr, .. } 
			| YarnExpr::Cast { expr, .. } => {
				Self::fill_exprs(fill_me, expr);
			},
			YarnExpr::BinaryOp { left, right, .. } => {
				Self::fill_exprs(fill_me, left);
				Self::fill_exprs(fill_me, right);
			},
			YarnExpr::CustomFunctionCall { args, .. } => {
				args.iter()
					.for_each(|arg| 
						Self::fill_exprs(fill_me, arg));
			},
			YarnExpr::BuiltInFunctionCall(call_ty) => {
				match call_ty {
					BuiltInFunctionCall::RandomRange(min_expr, max_expr) => {
						Self::fill_exprs(fill_me, min_expr);
						Self::fill_exprs(fill_me, max_expr);
					},
					BuiltInFunctionCall::RoundPlaces(num_expr, places_expr) => {
						Self::fill_exprs(fill_me, num_expr);
						Self::fill_exprs(fill_me, places_expr);
					},
					| BuiltInFunctionCall::FormatInvariant(input_expr) 
					| BuiltInFunctionCall::Dice(input_expr) 
					| BuiltInFunctionCall::Round(input_expr)
					| BuiltInFunctionCall::Floor(input_expr)
					| BuiltInFunctionCall::Ceil(input_expr) 
					| BuiltInFunctionCall::Inc(input_expr) 
					| BuiltInFunctionCall::Dec(input_expr) 
					| BuiltInFunctionCall::Decimal(input_expr)
					| BuiltInFunctionCall::Int(input_expr) => {
						Self::fill_exprs(fill_me, input_expr);
					},
					| BuiltInFunctionCall::Random 
					| BuiltInFunctionCall::Visited(_)
					| BuiltInFunctionCall::VisitedCount(_) => { }
				}
			}
			| YarnExpr::Lit(_)
			| YarnExpr::GetVar(_) 
			| YarnExpr::Identifier(_) => {}
		}
	}
}

impl<'a> Iterator for ExprIter<'a> {
	type Item = &'a YarnExpr;

	fn next(&mut self) -> Option<Self::Item> {
		return self.exprs.pop();
	}
}