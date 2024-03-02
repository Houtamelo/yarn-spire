use proc_macro2::{Ident, Punct, Spacing, Span};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use syn::Expr;
use proc_macro2::{TokenStream, TokenTree};
use proc_macro2::token_stream::IntoIter;
use quote::{TokenStreamExt, ToTokens};
use syn::parse::Parse;

struct TokenIterator {
	stream: IntoIter,
	inner: Option<Box<TokenIterator>>,
}

impl TokenIterator {
	fn new(into_iter: IntoIter) -> Self {
		TokenIterator {
			stream: into_iter,
			inner: None,
		}
	}
	
	fn next(&mut self) -> Option<TokenTree> {
		if let Some(inner_iterator) = &mut self.inner {
			if let Some(inner_next) = inner_iterator.next() {
				return Some(inner_next);
			} else {
				self.inner = None;
				return Some(TokenTree::Punct(proc_macro2::Punct::new(')', proc_macro2::Spacing::Alone)));
			}
		}
		
		return self.stream
			.next()
			.map(|token| match token {
				TokenTree::Group(group) => {
					
					
					let inner_iter = Box::from(TokenIterator::new(group.stream().into_iter()));
					self.inner = Some(inner_iter);
					Some(TokenTree::Punct(proc_macro2::Punct::new('(', proc_macro2::Spacing::Alone)))
				}
				others => Some(others),
			}).flatten();
	}
}

pub struct CustomExpr(pub Expr);

impl CustomExpr {
	pub fn parse_str(input: &str) -> Result<CustomExpr> {
		let tokens =
			proc_macro2::TokenStream::from_str(input)
				.map_err(|err| anyhow!(
					 "Could not parse input into token stream.\n\
					  Input: `{input}`\n\
					  Error: {err}")
				)?.into_iter();

		let mut iterator = TokenIterator::new(tokens);
		let mut new_tokens = TokenStream::new();

		while let Some(token) = iterator.next() {
			match token {
				TokenTree::Punct(punct) if punct.as_char() == '$' => {
					match iterator.next() {
						Some(TokenTree::Ident(ident)) => {
							new_tokens.append(TokenTree::Ident(Ident::new("get_var", Span::call_site())));
							new_tokens.append(TokenTree::Punct(Punct::new('(', Spacing::Alone)));
							new_tokens.append(TokenTree::Ident(ident));
							new_tokens.append(TokenTree::Punct(Punct::new(')', Spacing::Alone)));
						}
						invalid => {
							return Err(anyhow!(
								"Expected identifier after $\n\
								 Got: `{invalid:?}`"));
						}
					}
				},
				_ => new_tokens.extend(std::iter::once(token)),
			}
		}
		
		let final_tokens = 
			TokenStream::from_str(new_tokens.to_string().as_str())
				.map_err(|err| anyhow!(
					"Could not convert modified expression input into `TokenStream`\n\
					 TokenStream(as str): `{new_tokens:?}`\n\
					 Error: {err}")
				)?;

		let expr =
			syn::parse::Parser::parse2(Expr::parse, final_tokens.clone())
				.or_else(|_| syn::parse::Parser::parse2(Expr::parse_without_eager_brace, final_tokens.clone()))
				.map_err(|err| anyhow!(
					"Could not parse token stream into `syn::Expr`.\n\
					 Initial TokenStream: `{new_tokens:?}`\n\
					 Final TokenStream: `{final_tokens:?}`\n\
					 Error: {err}")
				)?;

		Ok(CustomExpr(expr))
	}
}

impl Debug for CustomExpr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0.to_token_stream())
	}
}

impl Deref for CustomExpr {
	type Target = Expr;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}