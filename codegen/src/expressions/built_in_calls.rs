use genco::prelude::FormatInto;
use genco::lang::Rust;
use genco::{quote_in, Tokens};
use crate::expressions::yarn_expr::YarnExpr;
use crate::quoting::quotable_types::enums::enum_type_title;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInFunctionCall {
	Visited(String),
	VisitedCount(String),
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

impl FormatInto<Rust> for &BuiltInFunctionCall {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		match self {
			BuiltInFunctionCall::FormatInvariant(expr) => {
				quote_in!(*tokens => std::format!("{}", $(expr.as_ref())))
			},
			BuiltInFunctionCall::Random => {
				quote_in!(*tokens => storage.random())
			},
			BuiltInFunctionCall::RandomRange(min_expr, max_expr) => {
				quote_in!(*tokens => storage.random_range($(min_expr.as_ref()).into(), $(max_expr.as_ref()).into()))
			},
			BuiltInFunctionCall::Dice(expr) => {
				quote_in!(*tokens => storage.dice($(expr.as_ref()).into()))
			},
			BuiltInFunctionCall::Round(expr) => {
				quote_in!(*tokens => built_in_functions::round($(expr.as_ref()).into()))
			},
			BuiltInFunctionCall::RoundPlaces(num_expr, places_expr) => {
				quote_in!(*tokens => built_in_functions::round_places($(num_expr.as_ref()).into(), $(places_expr.as_ref()).into()))
			},
			BuiltInFunctionCall::Floor(num_expr) => {
				quote_in!(*tokens => built_in_functions::floor($(num_expr.as_ref()).into()))
			},
			BuiltInFunctionCall::Ceil(num_expr) => {
				quote_in!(*tokens => built_in_functions::ceil($(num_expr.as_ref()).into()))
			},
			BuiltInFunctionCall::Inc(num_expr) => {
				quote_in!(*tokens => built_in_functions::inc($(num_expr.as_ref())))
			},
			BuiltInFunctionCall::Dec(num_expr) => {
				quote_in!(*tokens => built_in_functions::dec($(num_expr.as_ref())))
			},
			BuiltInFunctionCall::Decimal(num_expr) => {
				quote_in!(*tokens => built_in_functions::decimal($(num_expr.as_ref())))
			},
			BuiltInFunctionCall::Int(num_expr) => {
				quote_in!(*tokens => built_in_functions::int($(num_expr.as_ref())))
			},
			BuiltInFunctionCall::Visited(node_title) => {
				quote_in!(*tokens => storage.visited(&$(enum_type_title(node_title))))
			},
			BuiltInFunctionCall::VisitedCount(node_title) => {
				quote_in!(*tokens => storage.visited_count(&$(enum_type_title(node_title))))
			},
		}
	}
}
