use genco::prelude::{FormatInto, Rust};
use genco::Tokens;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInFunctionCall {
	FormatInvariant(Box<YarnExpr>),
	Random,
	RandomRange(Box<YarnExpr>, Box<YarnExpr>),
	Dice(Box<YarnExpr>),
	Round(Box<YarnExpr>),
	RoundPlaces(Box<YarnExpr>, Box<YarnExpr>),
	Floor(Box<YarnExpr>),
	Ceil(Box<YarnExpr>),
	Inc(Box<YarnExpr>),
	Dec(Box<YarnExpr>),
	Decimal(Box<YarnExpr>),
	Int(Box<YarnExpr>),
}

impl YarnExpr {
	pub fn iter_exprs(&self) -> impl Iterator<Item = &YarnExpr> {
		return ExprIter::from_expr(self);
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
					BuiltInFunctionCall::Random => {}
				}
			}
			| YarnExpr::Lit(_)
			| YarnExpr::VarGet(_) 
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

/*
format_invariant(number n)
format_invariant returns a string representation of n, formatted using the invariant culture. This is useful for embedding numbers in commands, where the command expects the number to be formatted using the invariant culture. For example, <<give_gold {$gold}>>, which might end up as give_gold 4,51 in German, but give_gold 4.51 in English, can now be <<give_gold {format_invariant($gold)}>>, which will always be give_gold 4.51.

random()
random returns a random number between 0 and 1 each time you call it.

random_range(number a, number b)
random_range returns a random number between a and b, inclusive.

dice(number sides)
dice returns a random integer between 1 and sides, inclusive.
For example, dice(6) returns a number between 1 and 6, just like rolling a six-sided die.

round(number n)
round rounds n to the nearest integer.

round_places(number n, number places)
round_places rounds n to the nearest number with places decimal points.

floor(number n)
floor rounds n down to the nearest integer, towards negative infinity.

ceil(number n)
ceil rounds n up to the nearest integer, towards positive infinity.

inc(number n)
inc rounds n up to the nearest integer. If n is already an integer, inc returns n+1.

dec(number n)
dec rounds n down to the nearest integer. If n is already an integer, dec returns n-1.

decimal(number n)
decimal returns the decimal portion of n. This will always be a number between 0 and 1. For example, decimal(4.51) will return 0.51.

int(number n)
int rounds n down to the nearest integer, towards zero.
*/

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationTy {
	String,
	bool,
	number,
	i8,
	i16,
	i32,
	i64,
	i128,
	isize,
	u8,
	u16,
	u32,
	u64,
	u128,
	usize,
	f32,
	f64,
}

impl DeclarationTy {
	pub fn from_syn(syn_ty: syn::Type) -> Option<Self> {
		let ty_str =
			match syn_ty {
				syn::Type::Group(group) => {
					return Self::from_syn(*group.elem);
				},
				syn::Type::Paren(paren) => {
					return Self::from_syn(*paren.elem);
				},
				syn::Type::Path(path) => {
					if let Some(ident) = path.path.get_ident() {
						ident.to_string()
					} else {
						return None;
					}
				},
				syn::Type::Verbatim(verbatim) => {
					verbatim.to_string()
				},
				_ => {
					return None;
				}
			};
		
		return Self::from_str(&ty_str);
	}
	
	pub fn from_str(str: &str) -> Option<Self> {
		return Some(match str.to_ascii_lowercase().as_str() {
			"string" => DeclarationTy::String,
			"bool" => DeclarationTy::bool,
			"number" => DeclarationTy::number,
			"i8" => DeclarationTy::i8,
			"i16" => DeclarationTy::i16,
			"i32" => DeclarationTy::i32,
			"i64" => DeclarationTy::i64,
			"i128" => DeclarationTy::i128,
			"isize" => DeclarationTy::isize,
			"u8" => DeclarationTy::u8,
			"u16" => DeclarationTy::u16,
			"u32" => DeclarationTy::u32,
			"u64" => DeclarationTy::u64,
			"u128" => DeclarationTy::u128,
			"usize" => DeclarationTy::usize,
			"f32" => DeclarationTy::f32,
			"f64" => DeclarationTy::f64,
			_ => {
				return None;
			}
		});
	}
}

impl FormatInto<Rust> for &DeclarationTy {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		match self {
			DeclarationTy::String => tokens.append("String"),
			DeclarationTy::bool => tokens.append("bool"),
			DeclarationTy::number => tokens.append("number"),
			DeclarationTy::i8 => tokens.append("i8"),
			DeclarationTy::i16 => tokens.append("i16"),
			DeclarationTy::i32 => tokens.append("i32"),
			DeclarationTy::i64 => tokens.append("i64"),
			DeclarationTy::i128 => tokens.append("i128"),
			DeclarationTy::isize => tokens.append("isize"),
			DeclarationTy::u8 => tokens.append("u8"),
			DeclarationTy::u16 => tokens.append("u16"),
			DeclarationTy::u32 => tokens.append("u32"),
			DeclarationTy::u64 => tokens.append("u64"),
			DeclarationTy::u128 => tokens.append("u128"),
			DeclarationTy::usize => tokens.append("usize"),
			DeclarationTy::f32 => tokens.append("f32"),
			DeclarationTy::f64 => tokens.append("f64"),
		}
	}
}