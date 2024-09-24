use std::collections::VecDeque;
use std::fmt::Display;
use anyhow::Result;
use proc_macro2::{TokenStream, TokenTree};
use crate::expressions;
use crate::expressions::yarn_expr::YarnExpr;

pub struct ArgsIter {
	pub tokens: VecDeque<TokenTree>,
}

impl Display for ArgsIter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", TokenStream::from_iter(self.tokens.clone()))
	}
}

impl Iterator for ArgsIter {
	type Item = Result<YarnExpr>;

	fn next(&mut self) -> Option<Self::Item> {
		let token_stream = &mut self.tokens;
		let mut results = vec![];

		while let Some(next) = token_stream.front() {
			let Some(previous) = results.last()
				else {
					results.push(token_stream.pop_front()?);
					continue;
				};
			
			if let TokenTree::Punct(punct) = next {
				match punct.as_char() {
					',' | ';' => {
						token_stream.pop_front();
						break;
					},
					'$' if let TokenTree::Ident(_) 
							 | TokenTree::Literal(_) 
							 | TokenTree::Group(_) = previous => { 
						break;
					},
					_ => {
						results.push(token_stream.pop_front()?);
					},
				}
			} else if let TokenTree::Punct(_) = previous {
				results.push(token_stream.pop_front()?);
			} else {
				break;
			}
		}

		if results.is_empty() {
			None
		} else {
			let tokens_str = TokenStream::from_iter(results).to_string();
			Some(expressions::parse_yarn_expr(tokens_str.as_str()))
		}
	}
}

#[cfg(test)]
mod tests;