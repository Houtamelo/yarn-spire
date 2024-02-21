use fmtools::format;
use crate::expressions::yarn_lit::YarnLit;
use crate::expressions::yarn_ops::{YarnBinaryOp, YarnUnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YarnExpr {
	Lit(YarnLit),
	VarGet(String),
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
	FunctionCall {
		func_name: String,
		args: Vec<YarnExpr>,
	},
}

impl YarnExpr {
	pub fn resolve(self) -> String {
		return match self {
			YarnExpr::Lit(literal) 
				=> literal.resolve(),
			YarnExpr::VarGet(var_name) 
				=> format!("controller.get::<"{var_name}">()"),
			YarnExpr::Parenthesis(inner_expr) 
				=> std::format!("({})", inner_expr.resolve()),
			YarnExpr::UnaryOp { yarn_op, right }
				=> std::format!("{}({})", yarn_op.resolve(), right.resolve()),
			YarnExpr::BinaryOp { yarn_op, left, right } 
				=> std::format!("{} {} {}", left.resolve(), yarn_op.resolve(), right.resolve()),
			YarnExpr::FunctionCall { func_name, args } 
				=> {
				let args_str = args
					.into_iter()
					.map(|arg| arg.resolve())
					.fold(String::new(), |mut acc, arg| {
						if acc.len() > 0 {
							acc.push(',');
							acc.push_str(arg.as_str());
							acc
						} else {
							arg
						}
					});
				format!("controller."{func_name}"("{args_str}")") 
			}
		};
	}
}
